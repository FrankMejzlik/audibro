//!
//! The main module providing high-level API for the receiver of the data.
//!

use hashsig::{Receiver, ReceiverParams, ReceiverTrait};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
// ---
use xxhash_rust::xxh3::xxh3_64;
// ---
#[allow(unused_imports)]
use hashsig::{debug, error, info, trace, warn};

#[derive(Debug)]
pub struct AudiBroReceiverParams {
    pub addr: String,
    pub running: Arc<AtomicBool>,
}

pub struct AudiBroReceiver {
    params: AudiBroReceiverParams,
    receiver: Receiver,
}

impl AudiBroReceiver {
    pub fn new(params: AudiBroReceiverParams) -> Self {
        let receiver = Receiver::new(ReceiverParams {
            addr: params.addr.clone(),
            running: params.running.clone(),
        });

        AudiBroReceiver { params, receiver }
    }

    pub fn run(&mut self, output: &mut dyn Write, mut input: Option<impl Read>) {
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
