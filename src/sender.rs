//!
//! The main module providing high-level API for the sender of the data.
//!

use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
// ---
use chrono::Local;
// ---
#[allow(unused_imports)]
use hashsig::{debug, error, info, log_input, trace, warn};
use hashsig::{Config, Sender, SenderParams, SenderTrait};
// ---
use crate::config::{self, BlockSignerInst};

#[derive(Debug)]
pub struct AudiBroSenderParams {
    pub seed: u64,
    pub layers: usize,
    pub addr: String,
    pub running: Arc<AtomicBool>,
}

pub struct AudiBroSender {
    params: AudiBroSenderParams,
    sender: Sender<BlockSignerInst>,
}

impl AudiBroSender {
    pub fn new(params: AudiBroSenderParams) -> Self {
        let config = Config {
            id_dir: config::ID_DIR.into(),
            id_filename: config::ID_FILENAME.into(),
            logs_dir: config::LOGS_DIR.into(),
            subscriber_lifetime: config::SUBSCRIBER_LIFETIME,
            net_buffer_size: config::BUFFER_SIZE,
            datagram_size: config::DATAGRAM_SIZE,
            max_pks: config::MAX_PKS,
        };
        let sender = Sender::new(
            SenderParams {
                addr: params.addr.clone(),
                running: params.running.clone(),
                layers: params.layers,
                seed: params.seed,
            },
            config,
        );
        AudiBroSender { params, sender }
    }

    pub fn run(&mut self, input: &mut dyn Read) {
        // The main loop as long as the app should run
        while self.params.running.load(Ordering::Acquire) {
            let data = Self::read_input(input);

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
            // We simulate periodic data coming via input
            thread::sleep(config::SIM_INPUT_PERIOD);
            let msg = Local::now().format("%d-%m-%Y %H:%M:%S").to_string();
            input_bytes = msg.into_bytes();
        }

        #[cfg(not(feature = "simulate_stdin"))]
        {
            let buf = vec![];
            _input.read_to_end(&mut buf).expect("Fail!");
            input_bytes = buf;
        }

        debug!(tag: "broadcasted", "{}", String::from_utf8_lossy(&input_bytes));
        input_bytes
    }
}
