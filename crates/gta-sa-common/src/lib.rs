//! Code shared by the GTA San Andreas auto splitters: the names and the split
//! rules.

// Tests run on the host, with std: `cargo test-host`.
#![cfg_attr(not(test), no_std)]
extern crate alloc;

pub mod names;
pub mod rules;
