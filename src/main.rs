//!
//! <PROJECT_NAME> is an implementation of the hash-based authentication protocol for streamed data.
//!
mod config;
mod receiver;
mod sender;
// ---
use std::fs::File;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
// ---
use clap::Parser;
#[allow(unused_imports)]
use hashsig::{debug, error, info, log_input, trace, warn};
// ---
use crate::config::{Args, ProgramMode};
use crate::receiver::{AudiBroReceiver, AudiBroReceiverParams};
use crate::sender::{AudiBroSender, AudiBroSenderParams};

fn run_sender(args: Args, running: Arc<AtomicBool>) {
    let sender_params = AudiBroSenderParams {
        running,
        seed: args.seed,
        layers: args.layers,
        addr: args.addr,
        key_lifetime: args.key_lifetime,
        cert_interval: args.cert_interval,
    };
    info!("Running a sender with {sender_params:#?}");

    let mut sender = AudiBroSender::new(sender_params);

    // Use the desired input (STDIN or the provided file)
    match args.input {
        Some(input_file) => {
            info!("Getting input from the file '{}'...", input_file);
            let mut file = match File::open(input_file) {
                Ok(file) => file,
                Err(e) => {
                    panic!("Failed to open file: {:?}", e);
                }
            };
            sender.run(&mut file)
        }
        None => {
            info!("Getting input from STDIN...");
            sender.run(&mut std::io::stdin())
        }
    }
}

fn run_receiver(args: Args, running: Arc<AtomicBool>) {
    let recv_params = AudiBroReceiverParams {
        running,
        target_addr: args.addr,
        target_name: args.target_name,
        key_lifetime: args.key_lifetime,
        cert_interval: args.cert_interval,
        delivery_deadline: Duration::from_millis(args.delivery_deadline_ms),
    };
    info!("Running a receiver with {recv_params:#?}");

    let mut receiver = AudiBroReceiver::new(recv_params);

    // Use the desired input (STDOUT or the provided file)
    match args.output {
        Some(output_file) => {
            info!("Putting output to the file '{}'...", output_file);
            let mut file = match File::create(output_file) {
                Ok(file) => file,
                Err(e) => {
                    panic!("Failed to open file: {:?}", e);
                }
            };
            receiver.run(&mut file)
        }
        None => {
            info!("Putting output to STDOUT...");
            receiver.run(&mut std::io::stdout())
        }
    }
}

fn init_application() -> Arc<AtomicBool> {
    // Clear the directories before every launch
    _ = std::fs::remove_dir_all(config::INPUT_DBG_DIR);
    _ = std::fs::remove_dir_all(config::OUTPUT_DBG_DIR);

    // Create the directory for logs
    std::fs::create_dir_all(config::LOGS_DIR).expect("The logs directory should be created.");

    // Create the directories for debugging input/output
    std::fs::create_dir_all(config::INPUT_DBG_DIR).expect("The directory should be created.");
    std::fs::create_dir_all(config::OUTPUT_DBG_DIR).expect("The directory should be created.");

    // Flag that indicates if the app shoul still run
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::Release);
        thread::sleep(std::time::Duration::from_millis(100));
        std::process::exit(0x01);
    })
    .expect("Error setting Ctrl-C handler");

    for t in config::USED_LOG_TAGS {
        info!(tag: t, "+++++++++++++++++++++++++++++++++");
        info!(tag: t, "+++++++++ PROGRAM START +++++++++");
        info!(tag: t, "+++++++++++++++++++++++++++++++++");
    }

    running
}

fn main() {
    if let Err(e) = config::setup_logger() {
        info!("Unable to initialize the logger!\nERROR: {}", e);
    }
    let args = Args::parse();
    let running = init_application();

    // Sender mode
    match args.mode {
        ProgramMode::Sender => run_sender(args, running),
        ProgramMode::Receiver => run_receiver(args, running),
    }
}
