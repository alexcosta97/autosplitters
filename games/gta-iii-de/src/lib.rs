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

mod completion;
mod data;
mod settings;
mod splits;
mod triggers;
mod version;

use alloc::{format, vec, vec::Vec};
use asr::{
    future::{next_tick, retry},
    signature::Signature,
    timer::{self, TimerState},
    Address, Process,
};

use autosplitter_utils::attempt::Attempt;
use autosplitter_utils::pair::Pair;
use autosplitter_utils::scan;
use settings::Settings;
use version::{Group, Layout};

asr::async_main!(stable);
asr::panic_handler!();

// Under Wine / Proton the process and module names keep the `.exe` suffix.
const PROCESS_NAME: &str = "LibertyCity.exe";

// `refreshRate` in the ASL. There's no wall clock without WASI, so the 500 ms
// load guard is expressed in ticks at this rate.
const TICK_RATE: u64 = 20;
const LOAD_COOLDOWN_TICKS: u64 = TICK_RATE / 2; // 500 ms

/// Length of the current thread's name in the game, `StringWatcher(.., 10)`
/// in the ASL.
const THREAD_LEN: usize = 10;

fn debug(text: &str) {
    asr::print_message(&format!("[GTAIII:DE Autosplitter] {text}"));
}

//=============================================================================
// Signatures
//=============================================================================

static SCRIPT_SPACE_SIG: Signature<13> = Signature::new("8b 51 18 48 8d 3d ?? ?? ?? ?? 48 03 d7");
static MISSION_SCRIPT_SIG: Signature<14> =
    Signature::new("84 c0 75 ?? 48 8d 0d ?? ?? ?? ?? 48 2b cf");
static START_FLAG_SIG: Signature<13> = Signature::new("88 05 ?? ?? ?? ?? c7 47 40 03 00 00 00");
static LOADING_SIG: Signature<30> = Signature::new(
    "8b 15 ?? ?? ?? ?? e8 ?? ?? ?? ?? 8b 05 ?? ?? ?? ?? 89 05 ?? ?? ?? ?? c6 05 ?? ?? ?? ?? 01",
);

/// The loading flag, found through the instruction that writes it, which
/// depends on the build (see `version::loading_disp_offset`).
fn loading_address(process: &Process, module: (Address, u64)) -> Option<u64> {
    let ptr = LOADING_SIG.scan_process_range(process, module)?;
    let opcode: u8 = process.read(ptr + version::LOADING_OPCODE_OFFSET).ok()?;
    let Some(disp_offset) = version::loading_disp_offset(opcode) else {
        debug(&format!("Loading pattern found, but followed by unknown opcode 0x{opcode:02X}!"));
        return None;
    };
    let disp: i32 = process.read(ptr + disp_offset).ok()?;
    Some(version::loading_rva(ptr.value() - module.0.value(), disp))
}

//=============================================================================
// Addresses
//=============================================================================

/// Offsets from the module base, like the `RVA`s in the ASL.
struct Addresses {
    script_space: u64,
    mission_script: u64,
    start_flag: u64,
    loading: u64,
}

impl Addresses {
    /// Returns `None` if a signature wasn't found, the ASL disables itself
    /// then (`vars.ready`).
    fn resolve(process: &Process, module: (Address, u64)) -> Option<Self> {
        let script_space = scan::address_from_pattern(process, module, 6, &SCRIPT_SPACE_SIG);
        let mission_script = scan::address_from_pattern(process, module, 7, &MISSION_SCRIPT_SIG);
        // The flag is the byte after the one the instruction writes.
        let start_flag =
            scan::address_from_pattern(process, module, 2, &START_FLAG_SIG).map(|rva| rva + 1);
        let loading = loading_address(process, module);

        fn show(name: &str, addr: Option<u64>) {
            match addr {
                Some(addr) => debug(&format!("Found {name} at RVA 0x{addr:X}")),
                None => debug(&format!("{name} pattern scan failed!")),
            }
        }
        show("ScriptSpace", script_space);
        show("MissionScript", mission_script);
        show("StartFlag", start_flag);
        show("Loading", loading);

        Some(Self {
            script_space: script_space?,
            mission_script: mission_script?,
            start_flag: start_flag?,
            loading: loading?,
        })
    }
}

//=============================================================================
// Memory Watcher
//=============================================================================

struct Memory {
    base: Address,
    addrs: Addresses,
    layout: Layout,
    /// Script global variables are one contiguous array of 4 byte values, so
    /// read all of them at once instead of one watcher per variable.
    globals_old: Vec<i32>,
    globals: Vec<i32>,
    loading: Pair<u8>,
    start_flag: Pair<u8>,
    thread: Pair<[u8; THREAD_LEN]>,
}

/// The name of a thread, up to the first NUL. Empty if it isn't valid UTF-8.
fn thread_name(bytes: &[u8; THREAD_LEN]) -> &str {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    core::str::from_utf8(&bytes[..end]).unwrap_or("")
}

impl Memory {
    fn new(base: Address, addrs: Addresses, layout: Layout) -> Self {
        let len = splits::globals_len(&layout);
        Self {
            base,
            addrs,
            layout,
            globals_old: vec![0; len],
            globals: vec![0; len],
            loading: Pair::default(),
            start_flag: Pair::default(),
            thread: Pair::default(),
        }
    }

    fn update(&mut self, process: &Process) {
        let base = self.base;
        let a = &self.addrs;

        core::mem::swap(&mut self.globals_old, &mut self.globals);
        if process.read_into_slice(base + a.script_space, &mut self.globals).is_err() {
            self.globals.copy_from_slice(&self.globals_old);
        }

        self.loading.update(process.read(base + a.loading).ok());
        self.start_flag.update(process.read(base + a.start_flag).ok());
        self.thread.update(process.read(base + a.mission_script).ok());
    }

    /// The name of the thread running now.
    fn thread(&self) -> &str {
        thread_name(&self.thread.current)
    }

    fn thread_changed(&self) -> bool {
        thread_name(&self.thread.old) != self.thread()
    }
}

//=============================================================================
// State keeping
//=============================================================================

struct Splitter {
    settings: Settings,
    /// Already split splits during this attempt (until timer reset).
    attempt: Attempt,
    prev_phase: Option<TimerState>,
    tick: u64,
    /// Tick when the last load occurred.
    last_load: Option<u64>,
    waiting: bool,
    /// Whether "Loading" was logged for the load in progress.
    was_loading: bool,
}

impl Splitter {
    fn ticks_since(&self, t: Option<u64>) -> u64 {
        t.map_or(u64::MAX, |t| self.tick - t)
    }

    fn update(&mut self, mem: &Memory) {
        self.tick += 1;
        self.settings.update();

        // Track loads on every tick, not only while the timer runs, so a load
        // that ends just before the timer starts still starts the guard (as in
        // gta-sa-de). The ASL only did this in `split`.
        if mem.loading.current != 0 {
            self.last_load = Some(self.tick);
        }

        // Clear list of already executed splits if timer is reset
        let phase = timer::state();
        if Some(phase) != self.prev_phase {
            if phase == TimerState::NotRunning {
                self.attempt.clear();
                debug("Cleared list of already executed splits");
            }
            self.prev_phase = Some(phase);
        }
    }

    /// A New Game began on this tick.
    fn new_game(&self, mem: &Memory) -> bool {
        triggers::new_game(mem.start_flag.pair(), mem.thread())
    }

    fn start(&self, mem: &Memory) -> bool {
        if self.new_game(mem) && self.settings.get("start") {
            debug("New Game");
            return true;
        }
        false
    }

    fn reset(&mut self, mem: &Memory) -> bool {
        if self.new_game(mem) && self.settings.get("reset") {
            debug("Reset");
            self.attempt.clear();
            return true;
        }
        false
    }

    fn should_split(&mut self, mem: &Memory) -> bool {
        //=====================================================================
        // Split prevention
        //=====================================================================
        if mem.loading.current != 0 {
            if !self.was_loading {
                debug("Loading");
                self.was_loading = true;
            }
            return false;
        }
        self.was_loading = false;
        if self.ticks_since(self.last_load) < LOAD_COOLDOWN_TICKS {
            // Prevent splitting shortly after loading, since this can
            // sometimes occur because memory values change
            if !self.waiting {
                debug("Wait..");
                self.waiting = true;
            }
            return false;
        }
        if self.waiting {
            debug("Done waiting..");
            self.waiting = false;
        }

        let inputs = splits::Inputs {
            old: &mem.globals_old,
            current: &mem.globals,
            thread: mem.thread(),
            thread_changed: mem.thread_changed(),
            layout: mem.layout,
        };
        match splits::check(&self.settings, &mut self.attempt, &inputs) {
            Some(split) => {
                debug(&format!("{split}"));
                true
            }
            None => false,
        }
    }
}

//=============================================================================
// Main loop
//=============================================================================

/// Equivalent of the ASL `init` block. Retries until the game is far enough
/// along for the module to be mapped and all signatures to be found.
async fn init(process: &Process, settings: &mut Settings) -> Memory {
    loop {
        if let Some(mem) = try_init(process) {
            return mem;
        }
        for _ in 0..TICK_RATE {
            settings.update();
            next_tick().await;
        }
    }
}

fn try_init(process: &Process) -> Option<Memory> {
    let (base, size) = scan::scan_range(process, PROCESS_NAME)?;

    let group = match scan::version(process, base) {
        Some(v) => {
            debug(&format!("Detected executable version: {v}"));
            Group::from_version(v.build, v.private)
        }
        None => {
            // The ASL ends up with build and private parts of 0 here.
            debug("Could not read the executable's version, using Group 1");
            Group::from_version(0, 0)
        }
    };
    debug(&format!("SaveInfo variable layout: {}", group.description()));

    let Some(addrs) = Addresses::resolve(process, (base, size)) else {
        debug("One or more pattern scans failed, retrying");
        return None;
    };
    let mut mem = Memory::new(base, addrs, group.layout());
    mem.update(process);
    Some(mem)
}

async fn main() {
    asr::set_tick_rate(TICK_RATE as f64);

    let mut splitter = Splitter {
        settings: settings::register(),
        attempt: Attempt::new(),
        prev_phase: None,
        tick: 0,
        last_load: None,
        waiting: false,
        was_loading: false,
    };

    loop {
        // Keep syncing the settings while there's no game, so the section
        // dropdowns work before it's started.
        let process = retry(|| {
            splitter.settings.update();
            Process::attach(PROCESS_NAME)
        })
        .await;
        process
            .until_closes(async {
                let mut mem = init(&process, &mut splitter.settings).await;
                loop {
                    mem.update(&process);
                    splitter.update(&mem);

                    match timer::state() {
                        TimerState::NotRunning => {
                            if splitter.start(&mem) {
                                timer::start();
                            }
                        }
                        TimerState::Running | TimerState::Paused | TimerState::Ended => {
                            if splitter.reset(&mem) {
                                timer::reset();
                                // Reset and start in the same cycle, as the
                                // ASL's comment intends.
                                if splitter.start(&mem) {
                                    timer::start();
                                }
                            } else if splitter.should_split(&mem) {
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
