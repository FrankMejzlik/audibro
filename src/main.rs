//!
//! <PROJECT_NAME> is an implementation of the hash-based authentication protocol for streamed data.
//!
mod audio_source;
mod config;
mod receiver;
mod sender;
mod sliding_buffer;
mod tui;
// ---
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
// ---
use clap::Parser;
#[allow(unused_imports)]
use hab::{debug, error, info, log_input, trace, warn};
use serde::{Deserialize, Serialize};
// ---
use crate::config::{Args, ProgramMode};
use crate::receiver::{AudiBroReceiver, AudiBroReceiverParams};
use crate::sender::{AudiBroSender, AudiBroSenderParams};

fn run_sender(args: Args, running: Arc<AtomicBool>, file_config: FileConfig) {
    let sender_params = AudiBroSenderParams {
        running,
        seed: args.seed,
        layers: args.layers,
        addr: args.addr,
        key_charges: args.key_charges,
        cert_interval: args.cert_interval,
        max_piece_size: args.max_piece_size,
        id_filepath: args.id_filepath,
        dgram_size: args.dgram_size,
        receiver_lifetime: Duration::from_secs(args.receiver_lifetime_s),
        key_dist: file_config.key_dist,
        dgram_delay: Duration::from_micros(args.dgram_delay_us),
        tui: args.tui,
        data_dir: args.data_dir,
    };
    info!("Running a sender with {sender_params:#?}");

    let mut sender = AudiBroSender::new(sender_params);

    sender.run();
}

fn run_receiver(args: Args, running: Arc<AtomicBool>) {
    let recv_params = AudiBroReceiverParams {
        running,
        target_addr: args.addr,
        target_name: args.target_name,
        delivery_deadline: Duration::from_millis(args.delivery_deadline_ms),
        heartbeat_period: Duration::from_secs(args.heartbeat_period_s),
        frag_timeout: Duration::from_secs(args.frag_timeout_s),
        id_filepath: args.id_filepath,
        receiver_lifetime: Duration::from_secs(args.receiver_lifetime_s),
        deliver: args.deliver,
        dgram_delay: Duration::from_micros(args.dgram_delay_us),
        tui: args.tui,
        distribute: args.distribute,
        alt_input: None,
    };
    info!("Running a receiver with {recv_params:#?}");

    let mut receiver = AudiBroReceiver::new(recv_params);

    receiver.run();
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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileConfig {
    key_dist: Vec<Vec<usize>>,
}

fn main() {
    if let Err(e) = config::setup_logger() {
        panic!("Unable to initialize the logger!\nERROR: {}", e);
    }

    // Override with cmd args
    // TODO
    let args = Args::parse();
    let running = init_application();

    let config_str = std::fs::read_to_string(&args.config).expect("Failed to read config file");
    let config: FileConfig = toml::from_str(&config_str).expect("Failed to parse config file");

    // Sender mode
    match args.mode {
        ProgramMode::Sender => run_sender(args, running, config),
        ProgramMode::Receiver => run_receiver(args, running),
    }
}
