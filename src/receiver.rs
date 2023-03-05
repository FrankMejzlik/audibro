//!
//! The main module providing high-level API for the receiver of the data.
//!

use hashsig::{Receiver, ReceiverParams, ReceiverTrait};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
// ---
// ---
#[allow(unused_imports)]
use hashsig::{debug, error, info, trace, warn};

use crate::config::{self, BlockSignerInst};

#[derive(Debug)]
pub struct AudiBroReceiverParams {
    pub addr: String,
    pub running: Arc<AtomicBool>,
    pub target_name: String,
}

pub struct AudiBroReceiver {
    params: AudiBroReceiverParams,
    receiver: Receiver<BlockSignerInst>,
}

impl AudiBroReceiver {
    pub fn new(params: AudiBroReceiverParams) -> Self {
        let receiver = Receiver::new(ReceiverParams {
            running: params.running.clone(),
            target_addr: params.addr.clone(),
            target_name: params.target_name.clone(),
            id_dir: config::ID_DIR.into(),
            id_filename: config::ID_FILENAME.into(),
            datagram_size: config::DATAGRAM_SIZE,
            net_buffer_size: config::BUFFER_SIZE,
            pub_key_layer_limit: config::MAX_PKS,
        });

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
