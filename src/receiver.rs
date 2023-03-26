//!
//! The main module providing high-level API for the receiver of the data.
//!

use hashsig::common::MsgVerification;
use hashsig::{Receiver, ReceiverParams, ReceiverTrait};
use rodio::Decoder as RodioDecoder;
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
// ---
use sha2::{Digest, Sha256};
// ---
#[allow(unused_imports)]
use hashsig::{debug, error, info, trace, warn};

use crate::config::{self, BlockSignerInst};
use crate::sliding_buffer::SlidingBuffer;

#[derive(Debug)]
pub struct AudiBroReceiverParams {
    pub running: Arc<AtomicBool>,
    pub target_addr: String,
    pub target_name: String,
    /// A number of signatures one keypair can generate.
    pub key_lifetime: usize,
    pub cert_interval: usize,
    pub delivery_deadline: Duration,
    pub tui: bool,
}

pub struct AudiBroReceiver {
    params: AudiBroReceiverParams,
    receiver: Receiver<BlockSignerInst>,
}

impl AudiBroReceiver {
    pub fn new(params: AudiBroReceiverParams) -> Self {
        let receiver = Receiver::new(ReceiverParams {
            running: params.running.clone(),
            target_addr: params.target_addr.clone(),
            target_name: params.target_name.clone(),
            id_dir: config::ID_DIR.into(),
            id_filename: config::ID_FILENAME.into(),
            datagram_size: config::DATAGRAM_SIZE,
            net_buffer_size: config::BUFFER_SIZE,
            pub_key_layer_limit: config::MAX_PKS,
            key_lifetime: params.key_lifetime,
            cert_interval: params.cert_interval,
            delivery_deadline: params.delivery_deadline,
        });

        AudiBroReceiver { params, receiver }
    }

    pub fn run(&mut self) {
        let my_buffer = SlidingBuffer::new();
        let my_buffer_clone = my_buffer.clone();

        if self.params.tui {
            println!("Receiving the audio broadcast...");
            std::thread::spawn(move || loop {
                if my_buffer.len() < 100_000 {
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
                let sink = rodio::Sink::try_new(&handle).unwrap();

                let source = match RodioDecoder::new(my_buffer.clone()) {
                    Ok(x) => x,

                    Err(_) => {
                        println!("Waiting for data!");
                        std::thread::sleep(Duration::from_millis(1000));
                        continue;
                    }
                };

                sink.append(source);
                sink.sleep_until_end();
                std::thread::sleep(Duration::from_millis(100));
            });
        }

        // The main loop as long as the app should run
        while self.params.running.load(Ordering::Acquire) {
            let received_block = match self.receiver.receive() {
                Ok(x) => x,
                Err(e) => {
                    warn!("Unable to receive! ERROR: {e}");
                    continue;
                }
            };

            // OUTPUT
            if self.params.tui {
                my_buffer_clone.append(&received_block.data);
                println!("STATUS: {}", received_block.sender);
            } else {
                let mut handle = stdout().lock();

                let mut hasher = Sha256::new();
                hasher.update(&received_block.data);
                let result = hasher.finalize();
                let hash = format!("{:x}", result);

                let size = received_block.data.len();

                match &received_block.sender {
                    MsgVerification::Verified(id) => {
                        writeln!(
                            handle,
                            "{};verified;{};{};{}",
                            received_block.metadata.seq,
                            id.petnames.join(","),
                            size,
                            hash
                        )
                        .unwrap();
                    }
                    MsgVerification::Certified(id) => {
                        writeln!(
                            handle,
                            "{};certified;{};{};{}",
                            received_block.metadata.seq,
                            id.petnames.join(","),
                            size,
                            hash
                        )
                        .unwrap();
                    }
                    MsgVerification::Unverified => {
                        writeln!(
                            handle,
                            "{};unverified;;{};{}",
                            received_block.metadata.seq, size, hash
                        )
                        .unwrap();
                    }
                }
            }
            debug!(tag: "received", "[{}][{:?}] {}", received_block.metadata.seq, received_block.sender, &received_block.data.len());
        }
    }
}
