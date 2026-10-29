//! When to start, reset and split, ported from the `update`, `split`, `start`
//! and `reset` blocks of `GTASA.asl`. No runtime calls here except logging, so
//! it can be tested on the host.

use alloc::{format, string::String};

use crate::{data, values::Values};
use gta_sa_common::attempt::{Attempt, Outcome};
use gta_sa_common::settings::Toggles;
use gta_sa_common::{names, rules};

/// `refreshRate` in the ASL. There's no wall clock without WASI, so the
/// cooldowns are expressed in ticks at this rate.
pub const TICK_RATE: u64 = 30;
const LOAD_COOLDOWN_TICKS: u64 = TICK_RATE / 2; // 500 ms
const SPLIT_COOLDOWN_TICKS: u64 = TICK_RATE * 5 / 2; // 2.5 s

/// The timer's phase, as far as the splitter cares.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    NotRunning,
    Running,
    Paused,
    Ended,
}

/// The timer, as `run` drives it.
pub trait Timer {
    fn phase(&self) -> Phase;
    fn start(&mut self);
    fn split(&mut self);
    fn reset(&mut self);
}

pub fn debug(text: &str) {
    // asr's imports don't link on the host.
    #[cfg(not(test))]
    asr::print_message(&format!("[GTASA Autosplitter] {text}"));
    #[cfg(test)]
    let _ = text;
}

#[derive(Default)]
pub struct Splitter {
    /// Already split splits during this attempt (until the timer is reset).
    attempt: Attempt,
    prev_phase: Option<Phase>,
    tick: u64,
    /// Tick when the last load occurred (loading from a save and such, not
    /// load screens).
    last_load: Option<u64>,
    waiting: bool,
}

impl Splitter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Equivalent of the ASL's `update` block, minus reading memory. Call
    /// once per tick, before the others (`run` does).
    pub fn update(&mut self, phase: Phase) {
        self.tick += 1;
        // Clear list of already executed splits if timer is reset
        if Some(phase) != self.prev_phase {
            if phase == Phase::NotRunning {
                self.attempt.clear();
                debug("Cleared list of already executed splits");
            }
            self.prev_phase = Some(phase);
        }
    }

    /// The timer uses the playing time to start as soon as the game loads.
    /// The thread and time checks are there to make sure it doesn't start and
    /// reset on loading a save.
    fn new_game(v: &Values) -> bool {
        let (old, current) = v.playing_time.pair();
        let (_, thread) = v.thread_name();
        current > 0
            && old == 0
            && current < 60 * 1000
            && data::EARLY_GAME_THREADS.iter().any(|t| t.as_bytes() == thread)
    }

    pub fn start(&self, v: &Values, s: &impl Toggles) -> bool {
        // New Game
        if Self::new_game(v) {
            // The ASL returns true here either way, and LiveSplit only starts
            // the timer if starting is enabled; that's the `start` setting.
            if s.get("start") {
                debug("New Game");
                debug(&format!("Playing Time: {}", v.playing_time.current));
                let (_, thread) = v.thread_name();
                debug(&format!("Thread: {}", String::from_utf8_lossy(thread)));
                return true;
            }
            return false;
        }

        // Loaded Save
        //
        // Optional starting the timer when loading a save. The "loading" value
        // seems to only be true when loading a game initially (so not loading
        // screens during the game).
        // Like New Game, only if starting is enabled (`start`).
        if s.get("start") && s.get("startOnSaveLoad") && !v.loading.current && v.loading.old {
            debug("Loaded Save");
            return true;
        }
        false
    }

    /// Same logic as starting the timer to have both run in the same cycle.
    /// Whether the ASL's `reset` returns true; the `reset` setting (LiveSplit's
    /// Reset checkbox) only decides in `run` whether the timer is reset.
    pub fn reset(&self, v: &Values) -> bool {
        Self::new_game(v)
    }

    /// One tick, the way LiveSplit's ASL runner (`ASLScript.RunUpdate`)
    /// calls the blocks: reset or else split while running or paused, then
    /// start if the timer is (now) not running.
    pub fn run(&mut self, v: &Values, s: &impl Toggles, timer: &mut impl Timer) {
        let mut phase = timer.phase();
        self.update(phase);
        if matches!(phase, Phase::Running | Phase::Paused) {
            if self.reset(v) {
                if s.get("reset") {
                    // Only output when actually resetting.
                    debug("Reset");
                    // Not in the ASL: the list is cleared on the reset itself
                    // too, since a start in the same tick hides the phase
                    // change from `update`.
                    self.attempt.clear();
                    timer.reset();
                    // Not asked again: a remote timer (LiveSplit through
                    // the bridge) still reports Running for a moment, which
                    // would miss the start.
                    phase = Phase::NotRunning;
                }
            } else if self.should_split(v, s) {
                timer.split();
            }
        }
        if phase == Phase::NotRunning && self.start(v, s) {
            timer.start();
        }
    }

    /// Check if splitting should occur based on whether this split has
    /// already been split since the timer started.
    ///
    /// Returns true if the split should occur. See
    /// `gta_sa_common::attempt::Attempt::try_split`: the split is also added
    /// to the list of already split splits when the cooldown prevents it.
    fn try_split(&mut self, split_id: &str, s: &impl Toggles) -> bool {
        if !s.get(split_id) {
            return false;
        }
        // Double split prevention (mostly for duping). This is set to 2.5s so
        // that dupes should (hopefully) not split spice, whereas close-on
        // splits like the Deathwarp to Angel Pine after Body Harvest still do
        // get split.
        //
        // Always added to the already executed splits, so that
        // cooldown-prevented splits are not split if a savegame is loaded and
        // the dupe done again.
        let prevention = s.get("doubleSplitPrevention");
        match self.attempt.try_split(split_id, prevention, self.tick, SPLIT_COOLDOWN_TICKS) {
            Outcome::AlreadyDone => false,
            Outcome::Split => {
                debug(&format!("Split: {split_id}"));
                true
            }
            Outcome::Cooldown => {
                debug(&format!("Split Prevented (Cooldown): {split_id}"));
                false
            }
        }
    }

    /// Check if the given mission (the name has to be exact) has already been
    /// passed, based on the current memory value.
    fn passed(v: &Values, mission: &str) -> bool {
        match rules::passed(data::MISSIONS, mission, |addr| v.global(addr).current) {
            Some((required, current)) => {
                debug(&format!("Check: {mission} {required} >= {current}"));
                current >= required
            }
            None => {
                debug(&format!("Mission not found: {mission}"));
                false
            }
        }
    }

    pub fn should_split(&mut self, v: &Values, s: &impl Toggles) -> bool {
        //=====================================================================
        // Split prevention
        //=====================================================================
        if v.loading.current {
            debug("Loading");
            self.last_load = Some(self.tick);
            return false;
        }
        if self.last_load.is_some_and(|t| self.tick - t < LOAD_COOLDOWN_TICKS) {
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

        // Split missions
        for (addr, missions) in data::MISSIONS {
            if let Some(split_id) = rules::counter_split(missions, v.global(*addr).pair()) {
                if self.try_split(split_id, s) {
                    return true;
                }
            }
        }

        for (_, missions) in data::MISSIONS2 {
            for (addr, split_id) in missions.iter() {
                // Some values changes from 0 -> 2, so check for > 0
                if rules::flag_set(v.global(*addr).pair()) {
                    return self.try_split(split_id, s);
                }
            }
        }

        // Split collectibles
        for (value, c) in v.collectibles.iter().zip(&data::COLLECTIBLES) {
            if value.current > value.old {
                let kind = c.kind;
                let all = format!("{kind}All");
                if s.get(&all) {
                    let max = rules::collectible_max(kind);
                    if value.current == max && value.old == max - 1 {
                        return self.try_split(&all, s);
                    }
                }
                if s.get(&format!("{kind}Each")) {
                    // Need to keep track of already split splits separately
                    // from the setting
                    let split_name = format!("{kind} {}", value.current);
                    if self.attempt.first_time(&split_name) {
                        debug(&format!("Split: {split_name}"));
                        return true;
                    }
                }
            }
        }

        // Busted/Deathwarp
        let ped_status = v.ped_status;
        if ped_status.changed() {
            if ped_status.current == 63 {
                // Busted
                if Self::passed(v, "Badlands") && !Self::passed(v, "Tanker Commander") {
                    return self.try_split("Bustedwarp BL #1", s);
                } else if Self::passed(v, "King in Exile") && !Self::passed(v, "Small Town Bank") {
                    return self.try_split("Bustedwarp BL #2", s);
                }
            }
            if ped_status.current == 55 {
                // Wasted
                if Self::passed(v, "Tanker Commander") && !Self::passed(v, "Body Harvest") {
                    return self.try_split("Deathwarp BL #1", s);
                } else if Self::passed(v, "Body Harvest") && !Self::passed(v, "King in Exile") {
                    return self.try_split("Deathwarp BL #2", s);
                } else if Self::passed(v, "The Da Nang Thang")
                    && !Self::passed(v, "Yay Ka-Boom-Boom")
                {
                    return self.try_split("Deathwarp SF", s);
                }
            }
        }

        // End of any%
        let eotl = v.global(data::EOTL);
        if eotl.current == 3 && eotl.old == 2 {
            // This check is probably not necessary since the variable $8014
            // seems to be only used in EotL Part 3, but just to be safe.
            if Self::passed(v, "End of the Line Part 2") {
                return self.try_split("any%", s);
            }
        }

        // Starting a certain mission
        //
        // This requires the feature of splitting every split only once,
        // because it only checks the first thread, which can sometimes change.
        // This is relatively lazy and simply checks for the first thread in
        // the list, which probably is the thread that was last started.
        let (old_thread, thread) = v.thread_name();
        if thread != old_thread {
            for (name, split_id) in data::START_MISSIONS {
                if thread == name.as_bytes() && self.try_split(split_id, s) {
                    return true;
                }
            }
        }

        // Chiliad Challenge
        let race = v.global(data::CHILIAD_RACE).pair();
        let done = v.global(data::CHILIAD_DONE).pair();
        if let Some(race_done) = rules::chiliad_race_done(race, done) {
            return self.try_split(&format!("Chiliad Challenge #{race_done}"), s);
        }

        // Import/Export Lists
        let list = v.global(data::EXPORT_LIST).current;
        let vehicles = core::array::from_fn(|i| v.global(data::EXPORT_BASE + i as u32 * 4).pair());
        if let Some(exported) = rules::exported(list, vehicles) {
            let vehicle = names::EXPORT_LISTS[exported.list][exported.vehicle];
            if self.try_split(&format!("Export {vehicle}"), s) {
                return true;
            }
            if exported.all_done
                && self.try_split(&format!("Export List {} Complete", exported.list + 1), s)
            {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use alloc::{collections::BTreeMap, string::String, vec::Vec};

    use super::*;
    use crate::{data, settings};
    use gta_sa_common::pair::Pair;
    use gta_sa_common::settings::tree::{State, Tree, Value};

    /// The settings' defaults, with some overridden.
    struct Test {
        tree: Tree,
        state: State,
    }

    impl Toggles for Test {
        fn get(&self, key: &str) -> bool {
            self.tree.get(&self.state, key).unwrap_or_else(|| panic!("unknown setting {key}"))
        }
    }

    fn toggles(overrides: &[(&str, bool)]) -> Test {
        let tree = settings::build();
        let map: BTreeMap<String, Value> =
            overrides.iter().map(|(k, b)| ((*k).into(), Value::Bool(*b))).collect();
        let state = tree.read(&map);
        Test { tree, state }
    }

    /// A field and its old and current value.
    enum F {
        G(u32, i32, i32),
        Loading(bool, bool),
        Ped(i32, i32),
        Time(i32, i32),
        Thread(&'static str, &'static str),
        /// Index in `data::COLLECTIBLES`.
        C(usize, i32, i32),
    }

    fn thread(name: &str) -> [u8; 10] {
        let mut t = [0; 10];
        t[..name.len()].copy_from_slice(name.as_bytes());
        t
    }

    fn values(fields: &[F]) -> Values {
        let mut v = Values::default();
        for f in fields {
            match *f {
                F::G(addr, old, current) => {
                    v.globals.insert(addr, Pair { old, current });
                }
                F::Loading(old, current) => v.loading = Pair { old, current },
                F::Ped(old, current) => v.ped_status = Pair { old, current },
                F::Time(old, current) => v.playing_time = Pair { old, current },
                F::Thread(old, current) => {
                    v.thread = Pair { old: thread(old), current: thread(current) }
                }
                F::C(i, old, current) => v.collectibles[i] = Pair { old, current },
            }
        }
        v
    }

    /// One tick of a running timer.
    fn tick(sp: &mut Splitter, v: &Values, s: &Test) -> bool {
        sp.update(Phase::Running);
        sp.should_split(v, s)
    }

    /// Ticks with nothing changing, past the split cooldown.
    fn idle(sp: &mut Splitter, s: &Test) {
        for _ in 0..SPLIT_COOLDOWN_TICKS + 1 {
            assert!(!tick(sp, &values(&[]), s));
        }
    }

    const BIG_SMOKE: F = F::G(0x64A060, 0, 1);

    #[test]
    fn new_game_starts() {
        let sp = Splitter::new();
        let s = toggles(&[]);
        let start = |fields: &[F], s: &Test| sp.start(&values(fields), s);
        assert!(start(&[F::Time(0, 1000), F::Thread("intro", "intro")], &s));
        assert!(!start(&[F::Time(0, 1000), F::Thread("grove2", "grove2")], &s));
        assert!(!start(&[F::Time(0, 60_000), F::Thread("intro", "intro")], &s));
        assert!(!start(&[F::Time(500, 1000), F::Thread("intro", "intro")], &s));
        let off = toggles(&[("start", false)]);
        assert!(!start(&[F::Time(0, 1000), F::Thread("intro", "intro")], &off));
    }

    #[test]
    fn loaded_save_starts_only_with_the_setting() {
        let sp = Splitter::new();
        let v = values(&[F::Loading(true, false)]);
        assert!(!sp.start(&v, &toggles(&[])));
        assert!(sp.start(&v, &toggles(&[("startOnSaveLoad", true)])));
        // The start setting (LiveSplit's Start checkbox) gates every start.
        let no_start = toggles(&[("startOnSaveLoad", true), ("start", false)]);
        assert!(!sp.start(&v, &no_start));
        let still_loading = values(&[F::Loading(true, true)]);
        assert!(!sp.start(&still_loading, &toggles(&[("startOnSaveLoad", true)])));
    }

    /// A timer that changes phase like LiveSplit's.
    struct FakeTimer {
        phase: Phase,
        actions: Vec<&'static str>,
        /// Report the new phase only from the next tick, like a remote timer.
        lagging: bool,
        pending: Option<Phase>,
    }

    impl FakeTimer {
        fn new(phase: Phase) -> Self {
            Self { phase, actions: Vec::new(), lagging: false, pending: None }
        }

        fn lagging(phase: Phase) -> Self {
            Self { lagging: true, ..Self::new(phase) }
        }

        fn set(&mut self, phase: Phase) {
            if self.lagging {
                self.pending = Some(phase);
            } else {
                self.phase = phase;
            }
        }
    }

    impl Timer for FakeTimer {
        fn phase(&self) -> Phase {
            self.phase
        }
        fn start(&mut self) {
            self.set(Phase::Running);
            self.actions.push("start");
        }
        fn split(&mut self) {
            self.actions.push("split");
        }
        fn reset(&mut self) {
            self.set(Phase::NotRunning);
            self.actions.push("reset");
        }
    }

    /// One tick of the runner; what it did to the timer.
    fn run(sp: &mut Splitter, t: &mut FakeTimer, v: &Values, s: &Test) -> Vec<&'static str> {
        t.actions.clear();
        sp.run(v, s, t);
        if let Some(phase) = t.pending.take() {
            t.phase = phase;
        }
        t.actions.clone()
    }

    fn new_game_and(fields: &[F]) -> Values {
        let mut v = values(&[F::Time(0, 1000), F::Thread("intro", "intro")]);
        let more = values(fields);
        v.globals = more.globals;
        v
    }

    #[test]
    fn reset_is_the_new_game_condition() {
        let sp = Splitter::new();
        assert!(sp.reset(&values(&[F::Time(0, 1000), F::Thread("intro", "intro")])));
        assert!(!sp.reset(&values(&[F::Time(0, 1000), F::Thread("grove2", "grove2")])));
    }

    #[test]
    fn new_game_while_running_resets_clears_and_starts() {
        let mut sp = Splitter::new();
        let s = toggles(&[]);
        let mut t = FakeTimer::new(Phase::Running);
        assert_eq!(run(&mut sp, &mut t, &values(&[BIG_SMOKE]), &s), ["split"]);
        for _ in 0..SPLIT_COOLDOWN_TICKS + 1 {
            assert!(run(&mut sp, &mut t, &values(&[]), &s).is_empty());
        }
        // Reset and start in the same tick: the phase never shows NotRunning
        // to `update`, so only the reset itself clears the list.
        assert_eq!(run(&mut sp, &mut t, &new_game_and(&[]), &s), ["reset", "start"]);
        assert_eq!(t.phase, Phase::Running);
        for _ in 0..SPLIT_COOLDOWN_TICKS + 1 {
            assert!(run(&mut sp, &mut t, &values(&[]), &s).is_empty());
        }
        assert_eq!(run(&mut sp, &mut t, &values(&[BIG_SMOKE]), &s), ["split"]);
    }

    #[test]
    fn new_game_resets_and_starts_a_lagging_timer() {
        let mut sp = Splitter::new();
        let s = toggles(&[]);
        let mut t = FakeTimer::lagging(Phase::Running);
        assert_eq!(run(&mut sp, &mut t, &new_game_and(&[]), &s), ["reset", "start"]);
        assert_eq!(t.phase, Phase::Running);
    }

    #[test]
    fn new_game_with_reset_off_does_not_split() {
        let mut sp = Splitter::new();
        let s = toggles(&[("reset", false)]);
        let mut t = FakeTimer::new(Phase::Running);
        assert!(run(&mut sp, &mut t, &new_game_and(&[BIG_SMOKE]), &s).is_empty());
        assert_eq!(t.phase, Phase::Running);
        // Nothing recorded: the split happens on a later change.
        assert_eq!(run(&mut sp, &mut t, &values(&[BIG_SMOKE]), &s), ["split"]);
    }

    #[test]
    fn no_reset_or_split_while_ended() {
        let mut sp = Splitter::new();
        let s = toggles(&[]);
        let mut t = FakeTimer::new(Phase::Ended);
        assert!(run(&mut sp, &mut t, &new_game_and(&[]), &s).is_empty());
        assert!(run(&mut sp, &mut t, &values(&[BIG_SMOKE]), &s).is_empty());
        assert_eq!(t.phase, Phase::Ended);
    }

    #[test]
    fn splits_while_paused() {
        let mut sp = Splitter::new();
        let s = toggles(&[]);
        let mut t = FakeTimer::new(Phase::Paused);
        assert_eq!(run(&mut sp, &mut t, &values(&[BIG_SMOKE]), &s), ["split"]);
        assert_eq!(run(&mut sp, &mut t, &new_game_and(&[]), &s), ["reset", "start"]);
    }

    #[test]
    fn starts_only_when_not_running() {
        let mut sp = Splitter::new();
        let s = toggles(&[("reset", false)]);
        let mut t = FakeTimer::new(Phase::NotRunning);
        assert_eq!(run(&mut sp, &mut t, &new_game_and(&[]), &s), ["start"]);
        assert!(run(&mut sp, &mut t, &new_game_and(&[]), &s).is_empty());
    }

    #[test]
    fn phase_to_not_running_clears() {
        let mut sp = Splitter::new();
        let s = toggles(&[]);
        assert!(tick(&mut sp, &values(&[BIG_SMOKE]), &s));
        idle(&mut sp, &s);
        assert!(!tick(&mut sp, &values(&[BIG_SMOKE]), &s));
        sp.update(Phase::NotRunning);
        sp.update(Phase::Running);
        assert!(sp.should_split(&values(&[BIG_SMOKE]), &s));
    }

    #[test]
    fn counter_splits_once() {
        let mut sp = Splitter::new();
        let s = toggles(&[]);
        assert!(tick(&mut sp, &values(&[BIG_SMOKE]), &s));
        idle(&mut sp, &s);
        assert!(!tick(&mut sp, &values(&[BIG_SMOKE]), &s));
    }

    #[test]
    fn missions2_first_changed_flag_decides() {
        let both = values(&[F::G(0x649AB8, 0, 1), F::G(0x64B824, 0, 1)]);
        let mut sp = Splitter::new();
        assert!(!tick(&mut sp, &both, &toggles(&[("Driving School Passed", false)])));
        let mut sp = Splitter::new();
        assert!(tick(&mut sp, &both, &toggles(&[])));
    }

    #[test]
    fn load_guard() {
        let mut sp = Splitter::new();
        let s = toggles(&[]);
        assert!(!tick(&mut sp, &values(&[F::Loading(false, true), BIG_SMOKE]), &s));
        for _ in 1..LOAD_COOLDOWN_TICKS {
            assert!(!tick(&mut sp, &values(&[F::Loading(true, false), BIG_SMOKE]), &s));
        }
        assert!(tick(&mut sp, &values(&[BIG_SMOKE]), &s));
    }

    #[test]
    fn cooldown() {
        for (prevention, second) in [(true, false), (false, true)] {
            let mut sp = Splitter::new();
            let s = toggles(&[("doubleSplitPrevention", prevention)]);
            assert!(tick(&mut sp, &values(&[BIG_SMOKE]), &s));
            for _ in 0..9 {
                assert!(!tick(&mut sp, &values(&[]), &s));
            }
            assert_eq!(tick(&mut sp, &values(&[F::G(0x64A490, 0, 1)]), &s), second);
        }
    }

    #[test]
    fn collectibles() {
        let all = values(&[F::C(0, 49, 50)]);
        let each = values(&[F::C(0, 3, 4)]);

        let mut sp = Splitter::new();
        assert!(tick(&mut sp, &all, &toggles(&[("PhotosAll", true)])));

        let mut sp = Splitter::new();
        let s = toggles(&[("PhotosEach", true)]);
        assert!(tick(&mut sp, &each, &s));
        // Load an older save and collect the same photo again.
        assert!(!tick(&mut sp, &values(&[F::Loading(false, true)]), &s));
        for _ in 0..LOAD_COOLDOWN_TICKS + SPLIT_COOLDOWN_TICKS {
            assert!(!tick(&mut sp, &values(&[]), &s));
        }
        assert!(!tick(&mut sp, &each, &s));

        let mut sp = Splitter::new();
        let s = toggles(&[]);
        assert!(!tick(&mut sp, &all, &s));
        assert!(!tick(&mut sp, &each, &s));
    }

    #[test]
    fn warps() {
        let s = toggles(&[("Deathwarp BL #1", true)]);
        // Tanker Commander passed, Body Harvest not.
        let wasted = values(&[F::Ped(1, 55), F::G(0x64A490, 1, 1), F::G(0x64A10C, 0, 0)]);
        let mut sp = Splitter::new();
        assert!(tick(&mut sp, &wasted, &s));
        let mut sp = Splitter::new();
        let unchanged = values(&[F::Ped(55, 55), F::G(0x64A490, 1, 1), F::G(0x64A10C, 0, 0)]);
        assert!(!tick(&mut sp, &unchanged, &s));
    }

    #[test]
    fn second_warp_branches() {
        let mut sp = Splitter::new();
        // King in Exile passed, Small Town Bank not.
        let busted = values(&[F::Ped(1, 63), F::G(0x64BB2C, 1, 1), F::G(0x64A48C, 0, 0)]);
        assert!(tick(&mut sp, &busted, &toggles(&[("Bustedwarp BL #2", true)])));

        let mut sp = Splitter::new();
        // Body Harvest passed, King in Exile not.
        let wasted = values(&[F::Ped(1, 55), F::G(0x64A10C, 1, 1), F::G(0x64BB2C, 0, 0)]);
        assert!(tick(&mut sp, &wasted, &toggles(&[("Deathwarp BL #2", true)])));

        let mut sp = Splitter::new();
        // The Da Nang Thang passed, Yay Ka-Boom-Boom not.
        let sf = values(&[F::Ped(1, 55), F::G(0x64A1DC, 5, 5), F::G(0x64A1E4, 0, 0)]);
        assert!(tick(&mut sp, &sf, &toggles(&[])));
        let mut sp = Splitter::new();
        let done = values(&[F::Ped(1, 55), F::G(0x64A1DC, 5, 5), F::G(0x64A1E4, 10, 10)]);
        assert!(!tick(&mut sp, &done, &toggles(&[])));
    }

    #[test]
    fn unchanged_thread_does_not_split() {
        let s = toggles(&[("GT #1", true)]);
        let mut sp = Splitter::new();
        assert!(!tick(&mut sp, &values(&[F::Thread("grove2", "grove2")]), &s));
    }

    #[test]
    fn counter_with_setting_off_does_not_stop_the_loop() {
        // Big Smoke's counter rises first in the list; its setting is off, but
        // the later counter still splits.
        let s = toggles(&[("Big Smoke", false)]);
        let mut sp = Splitter::new();
        let v = values(&[BIG_SMOKE, F::G(0x64A490, 0, 1)]);
        assert!(tick(&mut sp, &v, &s));
    }

    #[test]
    fn chiliad_through_should_split() {
        let s = toggles(&[]);
        let mut sp = Splitter::new();
        let v = values(&[F::G(data::CHILIAD_RACE, 1, 2), F::G(data::CHILIAD_DONE, 0, 0)]);
        assert!(tick(&mut sp, &v, &s));
    }

    #[test]
    fn failed_read_does_not_split() {
        let s = toggles(&[("Deathwarp BL #1", true)]);
        let mut v = values(&[F::Ped(1, 55), F::G(0x64A490, 1, 1), F::G(0x64A10C, 0, 0)]);
        v.ped_status.update(None);
        assert_eq!(v.ped_status.pair(), (55, 55));
        let mut sp = Splitter::new();
        assert!(!tick(&mut sp, &v, &s));
    }

    #[test]
    fn any_percent() {
        let s = toggles(&[]);
        let mut sp = Splitter::new();
        assert!(tick(&mut sp, &values(&[F::G(data::EOTL, 2, 3), F::G(0x64A334, 4, 4)]), &s));
        let mut sp = Splitter::new();
        assert!(!tick(&mut sp, &values(&[F::G(data::EOTL, 2, 3), F::G(0x64A334, 3, 3)]), &s));
    }

    #[test]
    fn start_mission_needs_exact_name() {
        let s = toggles(&[("GT #1", true)]);
        let mut sp = Splitter::new();
        assert!(tick(&mut sp, &values(&[F::Thread("intro", "grove2")]), &s));
        let mut sp = Splitter::new();
        assert!(!tick(&mut sp, &values(&[F::Thread("intro", "grove20")]), &s));
    }

    #[test]
    fn export() {
        let s = toggles(&[("Export Buffalo", true)]);
        let mut sp = Splitter::new();
        let first = values(&[F::G(data::EXPORT_LIST, 0, 0), F::G(data::EXPORT_BASE, 0, 1)]);
        assert!(tick(&mut sp, &first, &s));

        let mut v = values(&[F::G(data::EXPORT_LIST, 0, 0)]);
        for i in 0..10 {
            let old = if i == 9 { 0 } else { 1 };
            v.globals.insert(data::EXPORT_BASE + i * 4, Pair { old, current: 1 });
        }
        let mut sp = Splitter::new();
        assert!(tick(&mut sp, &v, &toggles(&[("Export List 1 Complete", true)])));
        let mut sp = Splitter::new();
        assert!(!tick(&mut sp, &v, &toggles(&[])));
    }
}
