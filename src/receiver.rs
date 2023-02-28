//!
//! The main module providing high-level API for the receiver of the data.
//!

use hashsig::{Receiver, ReceiverParams, ReceiverTrait};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
// ---
use hashsig::Config;
// ---
#[allow(unused_imports)]
use hashsig::{debug, error, info, trace, warn};

use crate::config::{self, BlockSignerInst};

#[derive(Debug)]
pub struct AudiBroReceiverParams {
    pub addr: String,
    pub running: Arc<AtomicBool>,
}

pub struct AudiBroReceiver {
    params: AudiBroReceiverParams,
    receiver: Receiver<BlockSignerInst>,
}

impl AudiBroReceiver {
    pub fn new(params: AudiBroReceiverParams) -> Self {
        let config = Config {
            id_dir: config::ID_DIR.into(),
            id_filename: config::ID_FILENAME.into(),
            logs_dir: config::LOGS_DIR.into(),
            subscriber_lifetime: config::SUBSCRIBER_LIFETIME,
            net_buffer_size: config::BUFFER_SIZE,
            datagram_size: config::DATAGRAM_SIZE,
            max_pks: config::MAX_PKS,
        };
        let receiver = Receiver::new(
            ReceiverParams {
                addr: params.addr.clone(),
                running: params.running.clone(),
            },
            config,
        );

        AudiBroReceiver { params, receiver }
    }

    pub fn run(&mut self, output: &mut dyn Write) {
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
            output
                .write_all(&received_block.data)
                .expect("The output should be writable!");
            output.flush().expect("Should be flushable!");
            debug!(tag: "received", "[{:?}] {}", received_block.sender, String::from_utf8_lossy(&received_block.data));
        }
    }
}
