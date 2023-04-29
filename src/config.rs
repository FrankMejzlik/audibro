//!
//! General static config file where you can tune the desired protocol paramters.
//!

use std::time::Duration;

// ---
use cfg_if::cfg_if;
use clap::Parser;
use rand_chacha::ChaCha20Rng;
// ---
use hab::{utils, HorstSigScheme};
// ---
use crate::config;

/// A directory where the identity files lie (e.g. `BlockSigner` with secret & public keys).
pub const ID_DIR: &str = ".identity/";
/// A name of the file where the state of `BlockSigner` is serialized.
pub const ID_FILENAME: &str = "id.bin";

/// A directory where we store the logs by default (e.g. when you run `cargo run`)
pub const LOGS_DIR: &str = "logs/";
/// A directory for output of signed blocks that the SENDER boradcasts.
pub const INPUT_DBG_DIR: &str = "logs/input/";
/// A directory for output of signed blocks that the RECEIVER receives.
pub const OUTPUT_DBG_DIR: &str = "logs/output/";

/// How long we will keep the subscriber alive without receiving another heartbeat.
pub const SUBSCRIBER_LIFETIME: Duration = Duration::from_secs(10);
/// Size of the datagram we send over the UDP prorocol.
//pub const DATAGRAM_SIZE: usize = 1500;
pub const DATAGRAM_SIZE: usize = 65507;
/// List of logging tags that we use throuought the program.
pub const USED_LOG_TAGS: &[&str] = &[
    "output",
    "sender",
    "registrator_task",
    "subscribers",
    "broadcasted",
    "block_signer",
    "receiver",
    "heartbeat_task",
    "received",
    "fragmented_blocks",
    "block_verifier",
    "delivery_queues",
];
/// A period in which the simulated STDIN input will be procuded.
#[cfg(feature = "simulate_stdin")]
//pub const SIM_INPUT_PERIOD: Option<Duration> = Some(Duration::from_millis(10));
pub const SIM_INPUT_PERIOD: Option<Duration> = None;

// ***************************************
//             PARAMETERS
// ***************************************
cfg_if! {
    // *** PRODUCTION ***
    if #[cfg(not(feature = "debug"))] {
        use sha3::Sha3_512;

        /// Size of the hashes in a Merkle tree
        const N: usize = 512 / 8;
        /// Number of SK segments in signature
        const K: usize = 32;
        /// Depth of the Merkle tree (without the root layer)
        const TAU: usize = 16;

        // --- Random generators ---
        /// A seedable CSPRNG used for number generation
        type CsPrng = ChaCha20Rng;

        // --- Hash function ---
        type TreeHashFn = Sha3_512;
    }
    // *** DEBUG ***
    else {
        use sha3::{Sha3_256};

        /// Size of the hashes in a Merkle tree
        const N: usize = 256 / 8;
        /// Number of SK segments in signature
        const K: usize = 64;
        /// Depth of the Merkle tree (without the root layer)
        const TAU: usize = 4;

        // --- Random generators ---
        /// A seedable CSPRNG used for number generation
        type CsPrng = ChaCha20Rng;

        // --- Hash functions ---
        type HashFn = Sha3_256;
    }
}

// ---
const T: usize = 2_usize.pow(TAU as u32);

pub type SignerInst = HorstSigScheme<N, K, TAU, { TAU + 1 }, T, CsPrng, HashFn>;

// ***
// The clap config for command line arguments.
// ***

/// Modes in which the progarm can operate.
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ProgramMode {
    /// The broadcaster of the data.
    Sender,
    /// The subscriber to the broadcasters.
    Receiver,
}

/// Define the CLI.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    // --- required ---
    /// What mode to launch the program in.
    #[clap(value_enum)]
    pub mode: ProgramMode,
    /// The address of the sender.
    #[clap()]
    pub addr: String,
    #[clap()]
    pub target_name: String,

    // --- optional ---
    /// If set, the sender runs in TUI mode.
    #[clap(short, long, action)]
    pub tui: bool,
    /// Seed used for the CSPRNG.
    #[clap(short, long, default_value_t = 42)]
    pub seed: u64,
    /// A desired number of key layers to use (for sender only)
    #[clap(long, default_value_t = 8)]
    pub layers: usize,
    /// A number of keys to certify forward (and backward).
    #[clap(long, default_value_t = 1)]
    pub cert_interval: usize,
    /// A number of signatures one keypair can generate.
    #[clap(long, default_value_t = 20)]
    pub key_lifetime: usize,
    /// Maximum delay between delivery of messages (in milliseconds)
    #[clap(long, default_value_t = 100)]
    pub delivery_deadline_ms: u64,
    /// Maximum size of one piece to be broadcasted.
    #[clap(long, default_value_t = 1024*1024)]
    pub max_piece_size: usize,
    #[clap(short, long, default_value = "../../config.toml")]
    pub config: String,
    /// If set, the receiver will also re-distribute the messages.
    #[clap(long)]
    pub distribute: Option<String>,
}

///
/// Setups the logger so it ignores the debug & trace logs in the third-party libs.
///
pub fn setup_logger() -> Result<(), fern::InitError> {
    std::fs::create_dir_all(config::LOGS_DIR).expect("The logs directory should be created.");

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                //chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                chrono::Local::now().format("%H:%M:%S"),
                record.level(),
                message
            ))
        })
        // Disable all by default
        .level(log::LevelFilter::Info) // TODO: This does now work properly
        // Allow for this module
        .level_for(utils::binary_name(), log::LevelFilter::Trace)
        //.chain(std::io::stdout())
        .chain(fern::log_file(format!("{}/output.log", config::LOGS_DIR))?)
        .apply()?;
    Ok(())
}
