//!
//! The main module providing high-level API for the sender of the data.
//!

use std::io::BufRead;
use std::io::{stdin, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver as MpscReceiver};
use std::sync::Arc;

// ---

// ---
#[allow(unused_imports)]
use hashsig::{debug, error, info, log_input, trace, warn};
use hashsig::{Sender, SenderParams, SenderTrait};
// ---
use crate::config::{self, BlockSignerInst};
use crate::tui::TerminalUi;

#[derive(Debug)]
pub struct AudiBroSenderParams {
    pub running: Arc<AtomicBool>,
    pub seed: u64,
    pub layers: usize,
    /// An address where the sender will listen for heartbeats.
    pub addr: String,
    /// A number of signatures one keypair can generate.
    pub key_lifetime: usize,
    pub cert_interval: usize,
    pub max_piece_size: usize,
    pub tui: bool,
}

pub struct AudiBroSender {
    params: AudiBroSenderParams,
    sender: Sender<BlockSignerInst>,
}

impl AudiBroSender {
    pub fn new(params: AudiBroSenderParams) -> Self {
        let sender = Sender::new(SenderParams {
            addr: params.addr.clone(),
            running: params.running.clone(),
            layers: params.layers,
            seed: params.seed,
            id_dir: config::ID_DIR.into(),
            id_filename: config::ID_FILENAME.into(),
            datagram_size: config::DATAGRAM_SIZE,
            net_buffer_size: config::BUFFER_SIZE,
            subscriber_lifetime: config::SUBSCRIBER_LIFETIME,
            key_lifetime: params.key_lifetime,
            cert_interval: params.cert_interval,
            max_piece_size: params.max_piece_size,
        });
        AudiBroSender { params, sender }
    }

    pub fn run(&mut self, input: &mut dyn Read) {
        let (tx, mut rx) = channel();

        if self.params.tui {
            std::thread::spawn(move || {
                let tui = TerminalUi::new(tx);
                tui.run_tui();
            });
        }

        // The main loop as long as the app should run
        while self.params.running.load(Ordering::Acquire) {
            // Get the data to broadcast from TUI mode
            let data = if self.params.tui {
                Self::read_input_tui(input, &mut rx)
            }
            // Else get data from stream mode
            else {
                Self::read_input(input)
            };

            if let Err(e) = self.sender.broadcast(data) {
                warn!("Failed to broadcast! ERROR: {e}");
            }
        }
    }
    // ---

    /// Reads the available chunk of data from the provided input.
    fn read_input(_input: &mut dyn Read) -> Vec<u8> {
        let input_bytes;
        #[cfg(feature = "simulate_stdin")]
        {
            use chrono::Local;
            use std::thread;

            if let Some(x) = config::SIM_INPUT_PERIOD {
                // We simulate periodic data coming via input
                thread::sleep(x);
            } else {
                let mut handle = stdin().lock();
                let mut input = String::new();
                handle.read_line(&mut input).expect("Failed to read line");
            }

            let msg = Local::now().format("%d-%m-%Y %H:%M:%S").to_string();
            input_bytes = msg.into_bytes();
        }

        #[cfg(not(feature = "simulate_stdin"))]
        {
            let mut buf = vec![];
            _input.read_to_end(&mut buf).expect("Fail!");
            input_bytes = buf;
        }

        debug!(tag: "broadcasted", "{}", String::from_utf8_lossy(&input_bytes));
        input_bytes
    }

    ///
    /// Runs the TUI and periodically sends the input data to broadcast.
    ///
    fn read_input_tui(_input: &mut dyn Read, rx: &mut MpscReceiver<Vec<u8>>) -> Vec<u8> {
        // Wait for the data
        let input_bytes = match rx.recv() {
            Ok(x) => x,
            Err(_e) => panic!("The input is dead!"),
        };

        debug!(tag: "broadcasted", "{}", String::from_utf8_lossy(&input_bytes));
        input_bytes
    }
}
