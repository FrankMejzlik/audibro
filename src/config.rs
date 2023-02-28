//!
//! General static config file where you can tune the desired protocol paramters.
//!

use std::time::Duration;

// ---
use cfg_if::cfg_if;
use clap::Parser;
use rand_chacha::ChaCha20Rng;
use sha3::{Sha3_256, Sha3_512};
// ---
use hashsig::utils;
use hashsig::BlockSigner;
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
/// Size of the buffer used to receive UDP datagrams.
pub const BUFFER_SIZE: usize = 1024;
/// Size of the datagram we send over the UDP prorocol.
pub const DATAGRAM_SIZE: usize = 512;
/// A maximum number of keys per layer stored at the receiver.
pub const MAX_PKS: usize = 3;
pub const SIM_INPUT_PERIOD: Duration = Duration::from_millis(500);
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
];

// ***************************************
//             PARAMETERS
// ***************************************
cfg_if! {
    // *** PRODUCTION ***
    if #[cfg(not(feature = "debug"))] {
        /// Size of the hashes in a Merkle tree
        const N: usize = 256 / 8;
        /// Number of SK segments in signature
        const K: usize = 32;
        /// Depth of the Merkle tree (without the root layer)
        const TAU: usize = 16;

        // --- Random generators ---
        /// A seedable CSPRNG used for number generation
        type CsPrng = ChaCha20Rng;

        // --- Hash functions ---
        // Hash fn for message hashing. msg: * -> N
        type MsgHashFn = Sha3_512;
        // Hash fn for tree & secret hashing. sk: 2N -> N & tree: N -> N
        type TreeHashFn = Sha3_256;
    }
    // *** DEBUG ***
    else {
        /// Size of the hashes in a Merkle tree
        const N: usize = 256 / 8;
        /// Number of SK segments in signature
        const K: usize = 128;
        /// Depth of the Merkle tree (without the root layer)
        const TAU: usize = 4;

        // --- Random generators ---
        /// A seedable CSPRNG used for number generation
        type CsPrng = ChaCha20Rng;

        // --- Hash functions ---
        // Hash fn for message hashing. msg: * -> N
        type MsgHashFn = Sha3_512;
        // Hash fn for tree & secret hashing. sk: 2N -> N & tree: N -> N
        type TreeHashFn = Sha3_256;
    }
}

// ---
const T: usize = 2_usize.pow(TAU as u32);
const MSG_HASH_SIZE: usize = (K * TAU) / 8;
const TREE_HASH_SIZE: usize = N;

// Alias for the specific signer/verifier we'll be using.
pub type BlockSignerInst = BlockSigner<
    K,
    TAU,
    { TAU + 1 },
    T,
    MSG_HASH_SIZE,
    TREE_HASH_SIZE,
    CsPrng,
    MsgHashFn,
    TreeHashFn,
>;

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

    // --- optional ---
    /// Seed used for the CSPRNG.
    #[clap(short, long, default_value_t = 42)]
    pub seed: u64,
    /// The input source file (if none, STDIN for sender, network for receiver)
    #[clap(short, long)]
    pub input: Option<String>,
    /// The output source file (if none, STDOUT for receiver, network for sender)
    #[clap(short, long)]
    pub output: Option<String>,
    /// A desired number of key layers to use (for sender only)
    #[clap(long, default_value_t = 8)]
    pub layers: usize,
}

///
/// Setups the logger so it ignores the debug & trace logs in the third-party libs.
///
pub fn setup_logger() -> Result<(), fern::InitError> {
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
        .level(log::LevelFilter::Warn)
        // Allow for this module
        .level_for(utils::binary_name(), log::LevelFilter::Trace)
        //.chain(std::io::stdout())
        .chain(fern::log_file(format!("{}/output.log", config::LOGS_DIR))?)
        .apply()?;
    Ok(())
}
