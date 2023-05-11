//!
//! The main module providing high-level API for the receiver of the data.
//!

use hab::common::MessageAuthentication;
use hab::{utils, Receiver, ReceiverParams, ReceiverTrait};
use rodio::Decoder as RodioDecoder;
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;
// ---
// ---
#[allow(unused_imports)]
use hab::{debug, error, info, trace, warn};

use crate::config::{self, SignerInst};
use crate::sliding_buffer::SlidingBuffer;
use crate::tui::TerminalUiReceiver;

#[derive(Debug)]
pub struct AudiBroReceiverParams {
    pub running: Arc<AtomicBool>,
    pub target_addr: String,
    pub target_name: String,
    pub delivery_deadline: Duration,
    pub distribute: Option<String>,
    pub heartbeat_period: Duration,
    pub frag_timeout: Duration,
    pub id_filepath: String,
    pub dgram_delay: Duration,
    pub receiver_lifetime: Duration,
    pub deliver: bool,
    pub tui: bool,
    pub alt_input: Option<std::sync::mpsc::Receiver<Vec<u8>>>,
}

pub struct AudiBroReceiver {
    params: AudiBroReceiverParams,
    receiver: Receiver<SignerInst>,
}

impl AudiBroReceiver {
    pub fn new(params: AudiBroReceiverParams) -> Self {
        let receiver = Receiver::new(ReceiverParams {
            running: params.running.clone(),
            target_addr: params.target_addr.clone(),
            target_name: params.target_name.clone(),
            id_filename: params.id_filepath.clone(),
            distribute: params.distribute.clone(),
            heartbeat_period: params.heartbeat_period,
            delivery_delay: params.delivery_deadline,
            frag_timeout: params.frag_timeout,
            dgram_delay: params.dgram_delay,
            receiver_lifetime: params.receiver_lifetime,
            deliver: params.deliver,
            alt_input: None,
        });

        AudiBroReceiver { params, receiver }
    }

    pub fn run(&mut self) {
        let my_buffer = SlidingBuffer::new();
        let my_buffer_clone = my_buffer.clone();

        let (tx, rx) = channel();
        let tx_clone = tx.clone();

        if self.params.tui {
            println!("Receiving the audio broadcast...");
            std::thread::spawn(move || {
                let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
                let sink = rodio::Sink::try_new(&handle).unwrap();
                loop {
                    let buffer_to_play = my_buffer.clone();

                    let source = match RodioDecoder::new(buffer_to_play) {
                        Ok(x) => x,
                        Err(_) => {
                            if let Err(_) = tx_clone.send(config::WAITING_FOR_DATA.to_string()) {}
                            std::thread::sleep(Duration::from_millis(500));
                            continue;
                        }
                    };

                    sink.append(source);
                    sink.sleep_until_end();
                }
            });
        }

        let is_distributor = self.params.distribute.is_some();
        let addr = self.params.target_addr.clone();
        let target_name = self.params.target_name.clone();

        // If should run with TUI
        if self.params.tui {
            std::thread::spawn(move || {
                // Run the UI
                let tui = TerminalUiReceiver::new(rx, addr, target_name, is_distributor);
                tui.run_tui();
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
                my_buffer_clone.append(&received_block.message);

                info!(tag:"receiver", "STATUS: {}", received_block.authentication);

                let state_str = match received_block.authentication {
                    MessageAuthentication::Authenticated(_) => "Authenticated",
                    MessageAuthentication::Certified(_) => "Certified",
                    MessageAuthentication::Unverified => "Unverified",
                };
                tx.send(state_str.to_string()).unwrap();
            } else {
                let mut handle = stdout().lock();

                let hash = utils::sha2_256_str(&received_block.message);

                let size = received_block.message.len();

                match &received_block.authentication {
                    MessageAuthentication::Authenticated(id) => {
                        writeln!(
                            handle,
                            "{};verified;{};{};{}",
                            received_block.seq,
                            id.petnames.join(","),
                            size,
                            hash
                        )
                        .unwrap();
                    }
                    MessageAuthentication::Certified(id) => {
                        writeln!(
                            handle,
                            "{};certified;{};{};{}",
                            received_block.seq,
                            id.petnames.join(","),
                            size,
                            hash
                        )
                        .unwrap();
                    }
                    MessageAuthentication::Unverified => {
                        writeln!(
                            handle,
                            "{};unverified;;{};{}",
                            received_block.seq, size, hash
                        )
                        .unwrap();
                    }
                }
            }
            debug!(tag: "received", "[{}][{:?}] {}", received_block.seq, received_block.authentication, &received_block.message.len());
        }
    }
}
