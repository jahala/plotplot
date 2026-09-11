//! plotplot, the stem of the garden.
//!
//! The library holds the pure parts: the error type every face returns, the three harnesses
//! and their payloads, the bed the rest of the crate consumes, the two contracts
//! (`garden.json` and `garden.lock`) and their schemas, the layout of a planted repository,
//! the renderers for what `init` writes into one, the nine questions `doctor` asks of a
//! planted repository, and the SARIF 2.1.0 every gate speaks. Every path is a function of a
//! repository root that the caller supplies; nothing here reads the current directory, and
//! nothing here panics on input.

pub mod bed;
pub mod bundle;
pub mod check;
pub mod cli;
pub mod deny;
pub mod doctor;
pub mod error;
pub mod fetch;
pub mod friction;
pub mod harness;
pub mod hook;
pub mod init;
pub mod install;
pub mod layout;
pub mod lock;
pub mod manifest;
pub mod plant;
pub mod platform;
pub mod receipt;
pub mod sarif;

/// The stem's own version, the first line of `plotplot version`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
