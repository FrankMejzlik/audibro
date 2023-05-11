
use std::{
    fs::File,
    sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender},
    time::{Duration, Instant},
};

// ---
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SupportedStreamConfig,
};
use minimp3::{Decoder, Frame};
use mp3lame_encoder::{Builder, Encoder, FlushNoGap, InterleavedPcm};
use std::io::{Cursor, Read};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};
// ---
#[allow(unused_imports)]
use hab::{debug, error, info, log_input, trace, warn};

/// Represents an MP3 file that can be broadcasted.
#[derive(Debug)]
pub struct AudioFile {
    pub artist: String,
    pub title: String,
    pub filepath: String,
    pub bitrate: u16,
}

#[derive(Debug, Clone)]
pub struct AudioSourceData {
    file: Option<String>,
}

impl AudioSourceData {
    pub fn new_file(file: &str) -> Self {
        AudioSourceData {
            file: Some(file.to_string()),
        }
    }
}

pub struct AudioSource {}

impl AudioSource {
    pub fn new(rx: MpscReceiver<AudioSourceData>, data_tx: MpscSender<Vec<u8>>) -> Self {
        let buffer_interval = 2.0;

        // Get the input device
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .expect("Failed to get default input device");
        let config = device
            .default_input_config()
            .expect("Failed to get default input config");
        let config_clone = config.clone();
        info!("Default input device: {:?}", device.name());
        info!("Default input format: {:?}", config);
        let data_tx_clone = data_tx.clone();

        let (txx, rxx) = mpsc::channel::<Vec<f64>>();
        // Spawn a new thread
        std::thread::spawn(move || {
            let num_channels = config_clone.channels();
            let sample_rate = config_clone.sample_rate().0;
            let mut mp3_encoder = build_mp3_encoder(sample_rate);
            loop {
                let received = rxx.recv().unwrap();
                let mp3_buffer = encode_waveform_f64(&received, num_channels, &mut mp3_encoder);
                data_tx_clone.send(mp3_buffer).expect("!");
            }
        });

        // Spawn audio processing
        std::thread::spawn(move || {
            let mut currently_playing: Option<AudioSourceData> = None;

            loop {
                if let Some(curr_play) = currently_playing.clone() {
                    // Microphone input
                    if curr_play.file.as_ref().unwrap() == "MICROPHONE" {
                        stream_mic(
                            &device,
                            config.clone(),
                            &rx,
                            &mut currently_playing,
                            buffer_interval,
                            txx.clone(),
                        );
                    }
                    // MP3 file input
                    else {
                        stream_mp3(
                            &curr_play,
                            &rx,
                            &mut currently_playing,
                            buffer_interval,
                            &data_tx,
                        );
                    }
                } else if let Ok(audio_data) = rx.recv() {
                    currently_playing = Some(audio_data);
                    warn!("Switching to '{currently_playing:?}'...");
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        });

        Self {}
    }
}

fn stream_mic(
    device: &Device,
    config: SupportedStreamConfig,
    rx: &MpscReceiver<AudioSourceData>,
    currently_playing: &mut Option<AudioSourceData>,
    buffer_interval: f64,
    txx: MpscSender<Vec<f64>>,
) {
    match config.sample_format() {
        cpal::SampleFormat::F64 => run::<f64>(
            device,
            config,
            rx,
            currently_playing,
            buffer_interval,
            txx,
            |x| x,
        ),
        cpal::SampleFormat::F32 => run::<f32>(
            device,
            config,
            rx,
            currently_playing,
            buffer_interval,
            txx,
            f32_to_f64,
        ),
        cpal::SampleFormat::I16 => run::<i16>(
            device,
            config,
            rx,
            currently_playing,
            buffer_interval,
            txx,
            i16_to_f64,
        ),
        cpal::SampleFormat::U16 => run::<u16>(
            device,
            config,
            rx,
            currently_playing,
            buffer_interval,
            txx,
            u16_to_f64,
        ),
        _ => panic!("Unsupported sample format"),
    };
}

fn stream_mp3(
    curr_play: &AudioSourceData,
    rx: &MpscReceiver<AudioSourceData>,
    currently_playing: &mut Option<AudioSourceData>,
    buffer_interval: f64,
    data_tx: &MpscSender<Vec<u8>>,
) {
    // Open the MP3 file.
    let mut file_data = Vec::new();
    let mut file =
        File::open(curr_play.file.as_ref().unwrap()).expect("Failed to open the MP3 file");
    file.read_to_end(&mut file_data)
        .expect("Failed to read MP3 file data");
    let file_data_clone = file_data.clone();

    let mut decoder = Decoder::new(Cursor::new(file_data));
    let mut current_duration = 0.0;
    let mut prev_duration = 0.0;

    // Save the current position in the input data.
    let mut frame_start = decoder.reader().position() as usize;
    loop {
        if let Ok(audio_data) = rx.try_recv() {
            *currently_playing = Some(audio_data);
            warn!("Switching to '{currently_playing:?}'...");
            return;
        }

        let prepos = decoder.reader().position() as usize;
        // Decode the next frame.
        match decoder.next_frame() {
            Ok(Frame {
                data,
                sample_rate,
                channels,
                ..
            }) => {
                // Update the current position in the input data.
                let current_position = decoder.reader().position() as usize;

                // Calculate frame duration based on frame samples
                let frame_duration = data.len() as f64 / (sample_rate * channels as i32) as f64;

                current_duration += frame_duration;
                if current_position > prepos && current_duration >= prev_duration + buffer_interval
                {
                    // Calculate the raw frame data.
                    let raw_frame_data = &file_data_clone[frame_start..current_position];
                    // 	warn!(
                    // 	"Frame [{frame_start}, {current_position}) with size {} to duration {}.",
                    // 	raw_frame_data.len(),
                    // 	current_duration
                    // );

                    frame_start = current_position;
                    let interval_played = current_duration - prev_duration;
                    prev_duration = current_duration;

                    // Sleep for the interval duration to simulate processing.
                    std::thread::sleep(Duration::from_secs_f64(interval_played));
                    data_tx.send(raw_frame_data.to_vec()).expect("!");
                }
            }
            Err(minimp3::Error::Eof) => {
                // The end of the file has been reached.
                return;
            }
            Err(e) => {
                eprintln!("Error decoding MP3 frame: {:?}", e);
            }
        }
    }
}

fn encode_waveform_f64(
    wave_buffer: &[f64],
    num_channels: u16,
    mp3_encoder: &mut mp3lame_encoder::Encoder,
) -> Vec<u8> {
	
	let mut new_wave_buffer = vec![];

    if num_channels == 1 {
		
        for w in wave_buffer {
			new_wave_buffer.push(*w);
            new_wave_buffer.push(*w);
        }
    } else if num_channels == 2 {
		new_wave_buffer.extend_from_slice(wave_buffer);
	} else {
		for (i, w) in wave_buffer.iter().enumerate() {
			if i % num_channels as usize == 0 {
				new_wave_buffer.push(*w);
			}
        }
	}
	let wave_buffer = &new_wave_buffer;
	let num_channels = 2;

	let input = InterleavedPcm(&wave_buffer);
	let mut mp3_out_buffer = Vec::new();
	mp3_out_buffer.reserve(mp3lame_encoder::max_required_buffer_size(
		input.0.len() / num_channels as usize
	));

	let encoded_size = mp3_encoder
		.encode(input, mp3_out_buffer.spare_capacity_mut())
		.expect("To encode");
	unsafe {
		mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
	}
	let encoded_size = mp3_encoder
		.flush::<FlushNoGap>(mp3_out_buffer.spare_capacity_mut())
		.expect("to flush");
	unsafe {
		mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
	}
	return mp3_out_buffer;
}

fn build_mp3_encoder(sample_rate: u32) -> Encoder {
    let mut mp3_encoder = Builder::new().expect("Create LAME builder");
    mp3_encoder.set_num_channels(2).expect("set channels");
    mp3_encoder
        .set_sample_rate(sample_rate)
        .expect("set sample rate");
    mp3_encoder
        .set_brate(mp3lame_encoder::Birtate::Kbps320)
        .expect("set brate");
    mp3_encoder
        .set_quality(mp3lame_encoder::Quality::Best)
        .expect("set quality");
    mp3_encoder.build().expect("To initialize LAME encoder")
}

fn run<T>(
    device: &cpal::Device,
    config: SupportedStreamConfig,
    rx: &MpscReceiver<AudioSourceData>,
    currently_playing: &mut Option<AudioSourceData>,
    buffer_interval: f64,
    txx: MpscSender<Vec<f64>>,
    f: impl Fn(Vec<T>) -> Vec<f64>,
) where
    T: cpal::Sample + cpal::SizedSample + Debug + Sync + Send + 'static,
{
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let buffer = Arc::new(Mutex::new(Vec::<T>::new()));
    let buffer2 = buffer.clone();

    let stream = device
        .build_input_stream(
            &config.into(),
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                buffer2.lock().unwrap().extend_from_slice(data);
            },
            err_fn,
            None,
        )
        .unwrap();
    stream.play().unwrap();

    let mut until = Instant::now() + Duration::from_secs_f64(buffer_interval);
    loop {
        // Loop until enough data buffered
        while until > Instant::now() {
            if let Ok(audio_data) = rx.try_recv() {
                *currently_playing = Some(audio_data);
                warn!("Switching to '{currently_playing:?}'...");
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        until += Duration::from_secs_f64(buffer_interval);
        let mut data = buffer.lock().unwrap();
        let data_cpy = std::mem::take(&mut *data);
        let wave_buffer = f(data_cpy);
        txx.send(wave_buffer).unwrap();
    }
}

fn u16_to_f64(data: Vec<u16>) -> Vec<f64> {
    data.into_iter()
        .map(|x| (x as f64 - u16::MAX as f64 / 2.0) / (u16::MAX as f64 / 2.0))
        .collect()
}

fn i16_to_f64(data: Vec<i16>) -> Vec<f64> {
    data.into_iter()
        .map(|x| x as f64 / i16::MAX as f64)
        .collect()
}

fn f32_to_f64(data: Vec<f32>) -> Vec<f64> {
    data.into_iter().map(|x| x as f64).collect()
}
