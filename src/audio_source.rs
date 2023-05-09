use rodio::Decoder as RodioDecoder;
use std::{
    fs::File,
    sync::mpsc::{Receiver as MpscReceiver, Sender as MpscSender},
    time::{Duration, Instant},
};

// ---
use minimp3::{Decoder, Frame};
use std::io::{Cursor, Read};
// ---
#[allow(unused_imports)]
use hab::{debug, error, info, log_input, trace, warn};

#[derive(Debug)]
pub struct AudioSourceData {
    file: Option<String>,
    #[allow(dead_code)]
    device: Option<()>,
}

impl AudioSourceData {
    pub fn new_file(file: &str) -> Self {
        AudioSourceData {
            file: Some(file.to_string()),
            device: None,
        }
    }
}

pub struct AudioSource {}

impl AudioSource {
    pub fn new(rx: MpscReceiver<AudioSourceData>, data_tx: MpscSender<Vec<u8>>) -> Self {
        let buffer_interval = 2.0;

        // Spawn audio processing
        std::thread::spawn(move || {
            let mut currently_playing: Option<AudioSourceData> = None;

            loop {
                if let Some(curr_play) = &currently_playing {
                    // Open the MP3 file.
                    let mut file_data = Vec::new();
                    let mut file = File::open(curr_play.file.as_ref().unwrap())
                        .expect("Failed to open the MP3 file");
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
                            currently_playing = Some(audio_data);
                            warn!("Switching to '{currently_playing:?}'...");
                            break;
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

                                //warn!("channels: {channels}, sample_rate: {sample_rate}");

                                // Calculate frame duration based on frame samples
                                let frame_duration =
                                    data.len() as f64 / (sample_rate * channels as i32) as f64;

                                current_duration += frame_duration;
                                if current_position > prepos {
                                    if current_duration >= prev_duration + buffer_interval {
                                        // Calculate the raw frame data.
                                        let raw_frame_data =
                                            &file_data_clone[frame_start..current_position];
                                        // 	warn!(
                                        // 	"Frame [{frame_start}, {current_position}) with size {} to duration {}.",
                                        // 	raw_frame_data.len(),
                                        // 	current_duration
                                        // );

                                        frame_start = current_position;
                                        prev_duration = current_duration;

                                        // let start_time = Instant::now();
                                        // let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
                                        // let sink = rodio::Sink::try_new(&handle).unwrap();

                                        // let source = match RodioDecoder::new(Cursor::new(raw_frame_data.to_vec())) {
                                        // 	Ok(x) => x,

                                        // 	Err(_) => {
                                        // 		println!("Waiting for data!");
                                        // 		//std::thread::sleep(Duration::from_millis(1000));
                                        // 		continue;
                                        // 	}
                                        // };

                                        // sink.append(source);
                                        // sink.sleep_until_end();
                                        // let duration = start_time.elapsed();
                                        // warn!("Time elapsed in expensive_function() is: {:?}", duration);

                                        // Sleep for the interval duration to simulate processing.
                                        std::thread::sleep(Duration::from_secs_f64(
                                            buffer_interval,
                                        ));

                                        //my_buffer_clone.append(raw_frame_data);
                                        data_tx.send(raw_frame_data.to_vec()).expect("!");
                                    }
                                }
                            }
                            Err(minimp3::Error::Eof) => {
                                // The end of the file has been reached.
                                break;
                            }
                            Err(e) => {
                                eprintln!("Error decoding MP3 frame: {:?}", e);
                            }
                        }
                    }
                } else {
                    if let Ok(audio_data) = rx.recv() {
                        currently_playing = Some(audio_data);
                        warn!("Switching to '{currently_playing:?}'...");
                    }
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        });

        Self {}
    }
}
