//! GTA San Andreas Definitive Edition auto splitter for the LiveSplit auto
//! splitting runtime (LiveSplit One, and LiveSplit via the ASR component).
//!
//! Port of `GTA_SA.asl`.
//! Original code by tduva and contributors, adapted for DE by hoxi.
//! Patterns for signature scanning and global variable indexes provided by Parik.

// Tests run on the host, with std: `cargo test --target x86_64-unknown-linux-gnu`.
#![cfg_attr(not(test), no_std)]
extern crate alloc;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

mod completion;
mod data;
mod settings;
mod triggers;

use alloc::{format, string::String, vec, vec::Vec};
use asr::{
    file_format::pe,
    future::{next_tick, retry},
    signature::Signature,
    timer::{self, TimerState},
    Address, PointerSize, Process,
};

use settings::Settings;
use triggers::{Events, Triggers};

asr::async_main!(stable);
asr::panic_handler!();

// Under Wine / Proton the process and module names keep the `.exe` suffix.
const PROCESS_NAME: &str = "SanAndreas.exe";

// `refreshRate` in the ASL. There's no wall clock without WASI, so the
// cooldowns are expressed in ticks at this rate.
const TICK_RATE: u64 = 20;
const LOAD_COOLDOWN_TICKS: u64 = TICK_RATE / 2; // 500 ms
const SPLIT_COOLDOWN_TICKS: u64 = TICK_RATE * 5 / 2; // 2.5 s

fn debug(text: &str) {
    asr::print_message(&format!("[GTASA:DE Autosplitter] {text}"));
}

//=============================================================================
// Signatures
//=============================================================================

static STAT_BASE_SIG: Signature<25> =
    Signature::new("?? 8b c3 ff ?? ?? ?? ?? ?? eb ?? 8b d3 ?? 8d 0d ?? ?? ?? ?? e8 ?? ?? ?? ??");
static SCRIPT_BASE_SIG: Signature<16> =
    Signature::new("74 ?? ?? 8d 05 ?? ?? ?? ?? ?? 83 3c ?? 01 74 ??");
static START_SIG: Signature<26> =
    Signature::new("8b 05 ?? ?? ?? ?? ?? 8b ?? ?? ?? ?? ?? 83 f8 08 73 ?? ?? c6 ?? ?? ?? ?? ?? 01");
// The ASL's pattern without its leading `?? 53 ?? 83 ec ??` (function
// prologue), which no longer precedes this code on 1.0.113.21181. Matches
// wherever the old pattern did. Resolves to the active scripts list head.
static THREAD_SIG: Signature<21> =
    Signature::new("?? 8b 15 ?? ?? ?? ?? ?? 85 d2 74 ?? 33 c9 ?? 8d 05 ?? ?? ?? ??");
static LOADING_SIG: Signature<30> = Signature::new(
    "0f b6 ?? ?? ?? ?? ?? ?? 88 ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? 66 89 ?? ?? ?? ?? ?? ?? 88",
);
static PLAY_TIME_SIG: Signature<22> =
    Signature::new("8b 15 ?? ?? ?? ?? ?? 85 c9 74 ?? 8b 05 ?? ?? ?? ?? 05 20 bf 02 00");
static TAGS_SIG: Signature<28> = Signature::new(
    "8b 3d ?? ?? ?? ?? ?? 8d 15 ?? ?? ?? ?? 8b 1d ?? ?? ?? ?? 66 ?? 89 2d ?? ?? ?? ?? e8",
);

/// `getAddressFromPattern`: resolves a RIP-relative operand to an offset from
/// the module base.
fn address_from_pattern<const N: usize>(
    process: &Process,
    module: (Address, u64),
    offset: u64,
    sig: &Signature<N>,
) -> Option<u64> {
    let ptr = sig.scan_process_range(process, module)? + offset;
    let disp: i32 = process.read(ptr).ok()?;
    Some((ptr.value() - module.0.value()).wrapping_add_signed(disp as i64 + 4))
}

/// `getValueFromPattern`: reads an immediate operand.
fn value_from_pattern<const N: usize>(
    process: &Process,
    module: (Address, u64),
    offset: u64,
    sig: &Signature<N>,
) -> Option<i32> {
    let ptr = sig.scan_process_range(process, module)? + offset;
    process.read(ptr).ok()
}

//=============================================================================
// Version Detection
//=============================================================================

/// Offsets from the module base, like the `int` addresses in the ASL.
struct Addresses {
    script_base: u64,
    start: u64,
    start_offset: u64,
    loading: u64,
    play_time: u64,
    /// Added to the indexes in `data::VERSION_SHIFTED`.
    globals_shift: u32,
    // Optional: the ASL kept going when these scans failed, only the splits
    // relying on them stopped working. The thread pattern doesn't match on
    // 1.0.113.21181, for example.
    stat_base: Option<u64>,
    thread: Option<u64>,
    tags: Option<u64>,
}

impl Addresses {
    /// Returns `None` if one of the addresses needed for starting the timer
    /// and splitting on missions wasn't found yet.
    fn resolve(process: &Process, module: (Address, u64), version: &str) -> Option<Self> {
        let hardcoded = version == "1.0.113.21181";
        let (script_base, tags, globals_shift) = if hardcoded {
            debug("Using hardcoded addresses for version 1.0.113.21181");
            (Some(0x51BEAE4), Some(0x572B41C), 6)
        } else {
            debug(&format!("Using pattern scanning for version {version}"));
            (
                address_from_pattern(process, module, 5, &SCRIPT_BASE_SIG),
                address_from_pattern(process, module, 2, &TAGS_SIG),
                0,
            )
        };
        let start = address_from_pattern(process, module, 9, &START_SIG);
        let start_offset = value_from_pattern(process, module, 21, &START_SIG);
        let loading = address_from_pattern(process, module, 3, &LOADING_SIG);
        let play_time = address_from_pattern(process, module, 2, &PLAY_TIME_SIG);
        let stat_base = address_from_pattern(process, module, 16, &STAT_BASE_SIG);
        let thread = address_from_pattern(process, module, 3, &THREAD_SIG);

        fn show(name: &str, addr: Option<u64>) {
            match addr {
                Some(addr) => debug(&format!("{name}: 0x{addr:X}")),
                None => debug(&format!("{name}: NOT FOUND")),
            }
        }
        show("Stat Base Address", stat_base);
        show("Script Base Address", script_base);
        show("Start Address", start);
        show("Start Offset", start_offset.map(|o| o as u64));
        show("Thread Address", thread);
        show("Loading Address", loading);
        show("Play Time Address", play_time);
        show("Tags Address", tags);
        debug(&format!("Globals Shift: {globals_shift}"));

        Some(Self {
            script_base: script_base?,
            start: start?,
            start_offset: start_offset? as u64,
            loading: loading?,
            play_time: play_time?,
            globals_shift,
            stat_base,
            thread,
            tags,
        })
    }

    fn global_index(&self, index: u32) -> usize {
        let shifted = data::VERSION_SHIFTED.contains(&index);
        (if shifted { index + self.globals_shift } else { index }) as usize
    }
}

//=============================================================================
// Memory Watcher
//=============================================================================

#[derive(Copy, Clone, Default)]
struct Pair<T> {
    old: T,
    current: T,
}

impl<T: Copy> Pair<T> {
    /// Keeps the previous value if the read failed, so a failed read can't
    /// look like a value change.
    fn update(&mut self, value: Option<T>) {
        self.old = self.current;
        if let Some(value) = value {
            self.current = value;
        }
    }

    fn pair(self) -> (T, T) {
        (self.old, self.current)
    }
}

struct Memory {
    base: Address,
    addrs: Addresses,
    /// Script global variables are one contiguous array of 4 byte values, so
    /// read all of them at once instead of one watcher per variable.
    globals_old: Vec<i32>,
    globals: Vec<i32>,
    loading: Pair<u8>,
    start_flag: Pair<u8>,
    thread: Pair<[u8; 8]>,
    play_time: Pair<i32>,
    /// Parallel to `data::COLLECTIBLES`.
    collectibles: Vec<Pair<i32>>,
}

impl Memory {
    fn new(base: Address, addrs: Addresses) -> Self {
        let len = (data::MAX_GLOBAL + addrs.globals_shift) as usize + 1;
        Self {
            base,
            addrs,
            globals_old: vec![0; len],
            globals: vec![0; len],
            loading: Pair::default(),
            start_flag: Pair::default(),
            thread: Pair::default(),
            play_time: Pair::default(),
            collectibles: vec![Pair::default(); data::COLLECTIBLES.len()],
        }
    }

    fn update(&mut self, process: &Process) {
        let base = self.base;
        let a = &self.addrs;

        core::mem::swap(&mut self.globals_old, &mut self.globals);
        if process.read_into_slice(base + a.script_base, &mut self.globals).is_err() {
            self.globals.copy_from_slice(&self.globals_old);
        }

        self.loading.update(process.read(base + a.loading).ok());
        self.start_flag.update(
            process.read_pointer_path(base, PointerSize::Bit64, &[a.start, a.start_offset]).ok(),
        );
        self.thread.update(a.thread.and_then(|thread| {
            process.read_pointer_path(base, PointerSize::Bit64, &[thread, 0x10]).ok()
        }));
        self.play_time.update(process.read(base + a.play_time).ok());

        for (pair, (kind, stat_id)) in self.collectibles.iter_mut().zip(data::COLLECTIBLES) {
            let addr = if *kind == "Tags" {
                a.tags
            } else {
                a.stat_base.map(|s| s + (*stat_id as u64 - 120) * 4)
            };
            pair.update(addr.and_then(|addr| process.read(base + addr).ok()));
        }
    }

    fn global(&self, index: u32) -> Pair<i32> {
        let i = self.addrs.global_index(index);
        Pair { old: self.globals_old[i], current: self.globals[i] }
    }

    fn thread_name(&self) -> (&[u8], &[u8]) {
        fn trim(s: &[u8; 8]) -> &[u8] {
            &s[..s.iter().position(|&b| b == 0).unwrap_or(s.len())]
        }
        (trim(&self.thread.old), trim(&self.thread.current))
    }
}

//=============================================================================
// State keeping
//=============================================================================

struct Splitter {
    settings: Settings,
    /// Already split splits during this attempt (until timer reset).
    split: Vec<String>,
    prev_phase: Option<TimerState>,
    tick: u64,
    /// Tick when the last load occurred (loading from a save and such, not
    /// load screens).
    last_load: Option<u64>,
    /// Tick when the last split was executed (to prevent double-splits).
    last_split: Option<u64>,
    waiting: bool,
    triggers: Triggers,
    /// What can start or reset the timer that happened on this tick.
    events: Events,
}

impl Splitter {
    fn ticks_since(&self, t: Option<u64>) -> u64 {
        t.map_or(u64::MAX, |t| self.tick - t)
    }

    /// Check if splitting should occur based on whether this split has
    /// already been split since the timer started.
    ///
    /// If this returns true (the split should occur), the split is also added
    /// to the list of already split splits.
    fn try_split(&mut self, split_id: &str) -> bool {
        if !self.settings.get(split_id) {
            return false;
        }
        if self.split.iter().any(|s| s == split_id) {
            return false;
        }
        self.split.push(split_id.into());
        // Double split prevention (mostly for duping). This is set to 2.5s so
        // that dupes should (hopefully) not split twice, whereas close-on
        // splits like the Deathwarp to Angel Pine after Body Harvest still do
        // get split.
        //
        // Always add this to the already executed splits, so that
        // cooldown-prevented splits are not split if a savegame is loaded and
        // the dupe done again.
        if !self.settings.get("doubleSplitPrevention")
            || self.ticks_since(self.last_split) > SPLIT_COOLDOWN_TICKS
        {
            debug(&format!("Split: {split_id}"));
            self.last_split = Some(self.tick);
            true
        } else {
            debug(&format!("Split Prevented (Cooldown): {split_id}"));
            false
        }
    }

    /// Check if the given mission (the name has to be exact) has already been
    /// passed, based on the current memory value.
    fn passed(&self, mem: &Memory, mission: &str) -> bool {
        for (index, missions) in data::MISSIONS {
            if let Some((value, _)) = missions.iter().find(|(_, m)| *m == mission) {
                let current = mem.global(*index).current;
                debug(&format!("Check: {mission} {value} >= {current}"));
                return current >= *value;
            }
        }
        debug(&format!("Mission not found: {mission}"));
        false
    }

    fn update(&mut self, mem: &Memory) {
        self.tick += 1;
        self.settings.update();

        // Track loads on every tick, not just while the timer is running. The
        // ASL only did this in `split`, so the load at New Game (which ends
        // before the timer starts) didn't trigger the cooldown, and globals the
        // game initialises right after it could split immediately.
        if mem.loading.current != 0 {
            self.last_load = Some(self.tick);
        }

        self.events =
            self.triggers.update(mem.loading.pair(), mem.start_flag.pair(), mem.play_time.pair());

        // Clear list of already executed splits if timer is reset
        let phase = timer::state();
        if Some(phase) != self.prev_phase {
            if phase == TimerState::NotRunning {
                self.split.clear();
                debug("Cleared list of already executed splits");
            }
            self.prev_phase = Some(phase);
        }
    }

    fn start(&self) -> bool {
        if self.events.new_game && self.settings.get("start") {
            debug("New Game");
            return true;
        }
        if self.events.save_loaded && self.settings.get("startOnSaveLoad") {
            debug("Save Loaded");
            return true;
        }
        false
    }

    fn reset(&mut self) -> bool {
        // Only downside is that accidental new game will reset the timer (but
        // who would do that with the way DE menu is laid out?)
        let reset = (self.events.new_game && self.settings.get("reset"))
            || (self.events.save_loaded && self.settings.get("resetOnSaveLoad"));
        if reset {
            debug("Reset");
            self.split.clear();
            return true;
        }
        false
    }

    fn should_split(&mut self, mem: &Memory) -> bool {
        //=====================================================================
        // Split prevention
        //=====================================================================
        if mem.loading.current != 0 {
            debug("Loading");
            return false;
        }
        if self.ticks_since(self.last_load) < LOAD_COOLDOWN_TICKS {
            // Prevent splitting shortly after loading from a save, since this
            // can sometimes occur because memory values change
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

        //=====================================================================
        // Splits
        //=====================================================================

        // Sections set to "All done" split once, when their last item is
        // done. Before the loops below, which return on the first change.
        for (section, count) in completion::SECTIONS {
            let split_id = format!("{section}#allDone");
            if self.settings.get(&split_id) {
                let items = count.items().into_iter().map(|(index, rule)| {
                    let value = mem.global(index);
                    (rule, (value.old, value.current))
                });
                if completion::just_completed(items) && self.try_split(&split_id) {
                    return true;
                }
            }
        }

        // Split missions
        for (index, missions) in data::MISSIONS {
            let value = mem.global(*index);
            if value.current > value.old {
                if let Some((_, split_id)) = missions.iter().find(|(v, _)| *v == value.current) {
                    if self.try_split(split_id) {
                        return true;
                    }
                }
            }
        }

        for (_, missions) in data::MISSIONS2 {
            for (index, split_id) in missions.iter() {
                let value = mem.global(*index);
                // Some values changes from 0 -> 2, so check for > 0
                if value.current > 0 && value.old == 0 {
                    return self.try_split(split_id);
                }
            }
        }

        // Split collectibles
        for (value, (kind, _)) in mem.collectibles.iter().zip(data::COLLECTIBLES) {
            if value.current > value.old {
                let all = format!("{kind}All");
                if self.settings.get(&all) {
                    let max = match *kind {
                        "Tags" => 100,
                        "Stunts (Completed)" => 70,
                        _ => 50,
                    };
                    if value.current == max && value.old == max - 1 {
                        return self.try_split(&all);
                    }
                }
                if self.settings.get(&format!("{kind}Each")) {
                    // Need to keep track of already split splits separately
                    // from the setting
                    let split_name = format!("{kind} {}", value.current);
                    if !self.split.contains(&split_name) {
                        debug(&format!("Split: {split_name}"));
                        self.split.push(split_name);
                        return true;
                    }
                }
            }
        }

        // End of any%
        let eotl = mem.global(data::EOTL);
        if eotl.current == 3 && eotl.old == 2 {
            // This check is probably not necessary since the variable $8014
            // seems to be only used in EotL Part 3, but just to be safe.
            if self.passed(mem, "End of the Line Part 2") {
                return self.try_split("any%");
            }
        }

        // Starting a certain mission
        //
        // This requires the feature of splitting every split only once,
        // because it only checks the first thread, which can sometimes change.
        // This is relatively lazy and simply checks for the first thread in
        // the list, which probably is the thread that was last started.
        let (old_thread, thread) = mem.thread_name();
        if thread != old_thread {
            for (name, split_id) in data::START_MISSIONS {
                if thread == name.as_bytes() && self.try_split(split_id) {
                    return true;
                }
            }
        }

        // Chiliad Challenge
        //
        // "chiliadRace" contains the next race to be started (1-3), but also
        // repeats when you do the races again (changes to 1 on finishing the
        // last race). "chiliadDone" changes from 0 to 1 when all races have
        // been done.
        let chiliad_race = mem.global(data::CHILIAD_RACE);
        let chiliad_done = mem.global(data::CHILIAD_DONE);
        if (chiliad_race.current > chiliad_race.old
            && chiliad_race.current > 1
            && chiliad_done.current == 0)
            || chiliad_done.current > chiliad_done.old
        {
            let race_done = if chiliad_done.current == 1 { 3 } else { chiliad_race.current - 1 };
            return self.try_split(&format!("Chiliad Challenge #{race_done}"));
        }

        // Import/Export Lists
        //
        // The three lists all contain 10 vehicles, which have their exported
        // state stored in an array, so basically 10 values that change from 0
        // to 1 when that car is exported. This is per list, so which vehicles
        // the values refer to changes based on which list is active.
        let export_list = mem.global(data::EXPORT_LIST).current;
        if (0..=2).contains(&export_list) {
            let mut all_done = true;
            let mut exported = None;
            for i in 0..10 {
                let vehicle = mem.global(data::EXPORT_BASE + i);
                if vehicle.current == 1 && vehicle.old == 0 {
                    exported = Some(i as usize);
                }
                if vehicle.current == 0 {
                    all_done = false;
                }
            }
            if let Some(i) = exported {
                let list = export_list as usize;
                if self.try_split(&format!("Export {}", data::EXPORT_LISTS[list][i])) {
                    return true;
                }
                // The list's "All done" option, which replaced the ASL's
                // "Export List n Complete" (and unticks the vehicles).
                if all_done && self.try_split(&format!("Export List {}#allDone", list + 1)) {
                    return true;
                }
            }
        }

        false
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
    let base = process.get_module_address(PROCESS_NAME).ok()?;
    // Wine doesn't necessarily report the module size correctly, the PE
    // header does.
    let size = pe::read_size_of_image(process, base)
        .map(u64::from)
        .or_else(|| process.get_module_size(PROCESS_NAME).ok())?;
    // Round up to whole pages. asr's `Signature::scan_iter` panics on a range
    // that ends partway through a page when the scan reaches the end, i.e.
    // whenever a signature isn't found (SizeOfImage is 0x5CA4400 on
    // 1.0.113.21181). The module is mapped in whole pages, so this is safe.
    let size = size.next_multiple_of(0x1000);

    let version = match pe::FileVersion::read(process, base) {
        Some(v) => {
            format!("{}.{}.{}.{}", v.major_version, v.minor_version, v.build_part, v.private_part)
        }
        None => {
            debug("Could not read the executable's version, trying pattern scanning");
            String::from("<unknown>")
        }
    };

    let addrs = Addresses::resolve(process, (base, size), &version);
    if addrs.is_none() {
        debug("Signature scan failed, retrying");
    }
    let mut mem = Memory::new(base, addrs?);
    mem.update(process);
    Some(mem)
}

async fn main() {
    asr::set_tick_rate(TICK_RATE as f64);

    let mut splitter = Splitter {
        settings: settings::register(),
        split: Vec::new(),
        prev_phase: None,
        tick: 0,
        last_load: None,
        last_split: None,
        waiting: false,
        triggers: Triggers::new(),
        events: Events::default(),
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
                            if splitter.start() {
                                timer::start();
                            }
                        }
                        TimerState::Running | TimerState::Paused | TimerState::Ended => {
                            if splitter.reset() {
                                timer::reset();
                                // Reset and start in the same cycle, as the
                                // ASL's comment intends.
                                if splitter.start() {
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
