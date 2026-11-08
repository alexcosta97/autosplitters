//! Code shared by the auto splitters that is not tied to any one game.

// Tests run on the host, with std: `cargo test-host`.
#![cfg_attr(not(test), no_std)]
extern crate alloc;

pub mod attempt;
pub mod pair;
pub mod scan;
pub mod settings;
