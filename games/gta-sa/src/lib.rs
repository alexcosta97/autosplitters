//! GTA San Andreas (2005, 32-bit PC) auto splitter for the LiveSplit auto
//! splitting runtime (LiveSplit One, and LiveSplit via the ASR component).
//!
//! Port of tduva's `GTASA.asl`.

// Tests run on the host, with std: `cargo test-host`.
#![cfg_attr(not(test), no_std)]
extern crate alloc;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

// Task 5 uses these.
#[allow(dead_code)]
mod data;
// Task 5 uses this.
#[allow(dead_code)]
mod version;
// Task 5 uses this.
#[allow(dead_code)]
mod settings;

use asr::future::next_tick;

asr::async_main!(stable);
asr::panic_handler!();

async fn main() {
    loop {
        next_tick().await;
    }
}
