//! The pieces `init` writes into a planted repository, as pure renderers.
//!
//! Nothing here reads the current directory, opens a harness, or decides what to plant: the
//! repository root, the season and the beds arrive as arguments. Three of the four
//! functions produce bytes and touch nothing; the fourth, [`gitconfig::apply`], is the one
//! edge in the module, and it is the only one that runs a program.
//!
//! `init` itself lands with a later node. What it will write is here, so it can be checked
//! before anything is written.

pub mod garden_block;
pub mod gitconfig;
pub mod githooks;
pub mod region;
