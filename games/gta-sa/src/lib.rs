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

mod data;
mod settings;
mod splitter;
mod values;
mod version;

use alloc::{format, vec::Vec};
use asr::{
    file_format::pe,
    future::{next_tick, retry},
    timer::{self, TimerState},
    Address, PointerSize, Process,
};

use gta_sa_common::settings::Settings;
use splitter::{debug, Phase, Splitter, TICK_RATE};
use values::{Reading, Values};
use version::{Addresses, Version, VersionValues, VERSION_VALUE_ADDRS};

asr::async_main!(stable);
asr::panic_handler!();

// Under Wine / Proton the process and module names keep the `.exe` suffix.
const PROCESS_NAMES: [&str; 2] = ["gta_sa.exe", "gta-sa.exe"];

/// Where to read what the ASL watches.
struct Memory {
    base: Address,
    addrs: Addresses,
    /// The ASL's 1.0 addresses of the script globals.
    globals: Vec<u32>,
}

impl Memory {
    fn read(&self, process: &Process) -> Reading {
        let base = self.base;
        let a = &self.addrs;
        let i32_at = |addr: u32| process.read::<i32>(base + addr as u64).ok();
        Reading {
            globals: self.globals.iter().map(|&addr| (addr, i32_at(a.global(addr)))).collect(),
            loading: process.read::<u8>(base + a.loading as u64).ok().map(|b| b != 0),
            ped_status: process
                .read_pointer_path(base, PointerSize::Bit32, &[a.player_ped as u64, 0x530])
                .ok(),
            playing_time: i32_at(a.playing_time),
            thread: process
                .read_pointer_path(base, PointerSize::Bit32, &[a.thread as u64, 0x8])
                .ok(),
            collectibles: core::array::from_fn(|i| i32_at(a.collectible(&data::COLLECTIBLES[i]))),
        }
    }
}

/// The module's base address and size, once the module is mapped.
fn module(process: &Process, name: &str) -> Option<(Address, u64)> {
    let base = process.get_module_address(name).ok()?;
    // Wine doesn't necessarily report the module size correctly, the PE
    // header does.
    let size = pe::read_size_of_image(process, base)
        .map(u64::from)
        .or_else(|| process.get_module_size(name).ok())?;
    Some((base, size))
}

/// Equivalent of the ASL `init` block's version detection. `None` if the
/// version is unknown, which disables the auto splitter.
async fn init(process: &Process, name: &str, settings: &mut Settings) -> Option<Memory> {
    let (base, size) = retry(|| {
        settings.update();
        module(process, name)
    })
    .await;
    let read = |i: usize| process.read::<i32>(base + VERSION_VALUE_ADDRS[i] as u64).ok();
    let values = VersionValues {
        v100_eu: read(0),
        v100_us: read(1),
        v101_eu: read(2),
        v101_us: read(3),
        v300_steam: read(4),
        v101_steam: read(5),
    };
    let Some(version) = Version::detect(size, &values) else {
        let all = [
            values.v100_eu,
            values.v100_us,
            values.v101_eu,
            values.v101_us,
            values.v300_steam,
            values.v101_steam,
        ];
        debug(&format!(
            "Unknown version (module size {size}, version values {all:?}), auto splitter disabled"
        ));
        return None;
    };
    debug(&format!("Version: {}", version.name()));
    Some(Memory { base, addrs: Addresses::for_version(version), globals: data::global_addresses() })
}

async fn main() {
    asr::set_tick_rate(TICK_RATE as f64);

    let mut settings = settings::register();
    let mut splitter = Splitter::new();

    loop {
        // Keep syncing the settings while there's no game, so the section
        // dropdowns work before it's started.
        let (process, name) = retry(|| {
            settings.update();
            PROCESS_NAMES.iter().find_map(|&name| Process::attach(name).map(|p| (p, name)))
        })
        .await;
        process
            .until_closes(async {
                let Some(mem) = init(&process, name, &mut settings).await else {
                    // Disable all timer control actions if the version was not
                    // detected.
                    loop {
                        settings.update();
                        next_tick().await;
                    }
                };
                let mut values = Values::first(&mem.read(&process));
                loop {
                    settings.update();
                    values.update(&mem.read(&process));

                    let state = timer::state();
                    let phase = match state {
                        TimerState::NotRunning => Phase::NotRunning,
                        _ => Phase::Running,
                    };
                    splitter.update(phase);

                    match state {
                        TimerState::NotRunning => {
                            if splitter.start(&values, &settings) {
                                timer::start();
                            }
                        }
                        TimerState::Running | TimerState::Paused | TimerState::Ended => {
                            if splitter.reset(&values, &settings) {
                                timer::reset();
                                // Reset and start in the same cycle, as the
                                // ASL's comment intends.
                                if splitter.start(&values, &settings) {
                                    timer::start();
                                }
                            } else if splitter.should_split(&values, &settings) {
                                timer::split();
                            }
                        }
                        _ => {}
                    }

                    next_tick().await;
                }
            })
            .await;
    }
}
