//! Code shared by the GTA San Andreas auto splitters.

// Tests run on the host, with std: `cargo test-host`.
#![cfg_attr(not(test), no_std)]
extern crate alloc;

pub mod settings;
