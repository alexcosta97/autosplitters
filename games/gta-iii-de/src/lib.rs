//! GTA III Definitive Edition auto splitter for the LiveSplit auto splitting
//! runtime (LiveSplit One, and LiveSplit via the ASR component).
//!
//! Port of `LiveSplit.GTA3.Definitive.asl`.
//! Original code by hoxi, initial version by illuminati7777, heavily based on
//! the original GTA III/VC auto splitters.

// Tests run on the host, with std: `cargo test-host`.
#![cfg_attr(not(test), no_std)]
extern crate alloc;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

// The rules and tables below aren't wired to the game yet.
#[allow(dead_code)]
mod completion;
#[allow(dead_code)]
mod data;
mod settings;
#[allow(dead_code)]
mod splits;
#[allow(dead_code)]
mod triggers;
#[allow(dead_code)]
mod version;

use asr::future::next_tick;

asr::async_main!(stable);
asr::panic_handler!();

// `refreshRate` in the ASL. There's no wall clock without WASI, so the
// cooldowns are expressed in ticks at this rate.
const TICK_RATE: u64 = 20;

async fn main() {
    asr::set_tick_rate(TICK_RATE as f64);

    let mut settings = settings::register();
    loop {
        settings.update();
        next_tick().await;
    }
}
