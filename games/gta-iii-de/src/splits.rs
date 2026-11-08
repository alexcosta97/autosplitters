//! When to split, ported from the `split` block of the ASL: a mission is
//! passed or started, a collectible is found, or the Any% run ends. Also
//! "All done" for the missions of a giver, see `completion`.
//!
//! Doesn't touch `asr`, so it can be tested on the host.

use alloc::{format, string::String};
use core::fmt;

use crate::completion;
use crate::data::{self, COLLECTIBLES, EXCHANGE_THREAD, MISSIONS, TE_HELIPAD, TE_TIMER};
use crate::version::{Layout, Shift};
use gta_sa_common::attempt::Attempt;
use gta_sa_common::pair::Pair;
use gta_sa_common::settings::Toggles;

/// What the rules look at on a tick.
pub struct Inputs<'a> {
    /// The script's globals read on the previous tick and on this one,
    /// indexed like `data` does, both `globals_len` long.
    pub old: &'a [i32],
    pub current: &'a [i32],
    /// The name of the thread running now, and whether it changed.
    pub thread: &'a str,
    pub thread_changed: bool,
    pub layout: Layout,
}

impl Inputs<'_> {
    /// The pair of a global, from its index in `data` and how the executable
    /// moves it. Reads 0 outside the slices.
    fn pair(&self, index: u32, shift: Shift) -> Pair<i32> {
        let i = (index + self.layout.shift(shift)) as usize;
        Pair {
            old: self.old.get(i).copied().unwrap_or(0),
            current: self.current.get(i).copied().unwrap_or(0),
        }
    }
}

/// How many globals to read, from the first, for `Inputs`.
pub fn globals_len(layout: &Layout) -> usize {
    let missions = MISSIONS.iter().flat_map(|(_, ms)| ms.iter().map(|(i, _)| (*i, Shift::Mission)));
    let collectibles = COLLECTIBLES.iter().map(|c| (c.index, c.shift));
    let others = [(TE_TIMER, Shift::TeTimer), (TE_HELIPAD, Shift::TeHelipad)];
    missions
        .chain(collectibles)
        .chain(others)
        .map(|(index, shift)| (index + layout.shift(shift)) as usize + 1)
        .max()
        .unwrap()
}

/// What to split on, for the log. Displays like the ASL's messages.
#[derive(Debug, PartialEq, Eq)]
pub enum Split {
    /// All of a mission giver's missions are passed. Holds the section's key.
    AllDone(String),
    MissionEnd(String),
    MissionStart(String),
    /// The last collectible of a kind, holds what's split on (`"Rampages 20"`).
    AllCollectibles(String),
    Collectible(String),
    AnyEnd,
}

impl fmt::Display for Split {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Split::AllDone(key) => write!(f, "All Done Split: {key}"),
            Split::MissionEnd(key) => write!(f, "Mission End Split: {key}"),
            Split::MissionStart(key) => write!(f, "Mission Start Split: {key}"),
            Split::AllCollectibles(name) => write!(f, "All Collectibles Split: {name}"),
            Split::Collectible(name) => write!(f, "Collectible Split: {name}"),
            Split::AnyEnd => write!(f, "Any% Final Split"),
        }
    }
}

/// The setting for splitting when a mission is passed.
pub fn end_key(mission: &str) -> String {
    format!("{mission} (end)")
}

/// The setting for splitting when a mission starts.
pub fn start_key(mission: &str) -> String {
    format!("{mission} (start)")
}

/// The setting for splitting when all of a mission giver's missions are
/// passed.
pub fn all_done_key(giver: &str) -> String {
    format!("{giver} (end)#allDone")
}

/// The first of the splits that should happen on this tick, in the ASL's
/// order. Records it in `attempt`, so it happens once. Call every tick, once
/// the loading guard allows it.
pub fn check(toggles: &impl Toggles, attempt: &mut Attempt, inputs: &Inputs) -> Option<Split> {
    // Missions passed
    for (giver, missions) in MISSIONS {
        let passed = missions.iter().map(|(index, _)| inputs.pair(*index, Shift::Mission).pair());
        if completion::just_completed(passed) {
            let key = all_done_key(giver);
            if toggles.get(&key) && attempt.first_time(&key) {
                return Some(Split::AllDone(key));
            }
        }
    }
    for (_, missions) in MISSIONS {
        for (index, mission) in *missions {
            let flag = inputs.pair(*index, Shift::Mission);
            if flag.current == flag.old.wrapping_add(1) {
                let key = end_key(mission);
                if toggles.get(&key) && attempt.first_time(&key) {
                    return Some(Split::MissionEnd(key));
                }
            }
        }
    }

    // Missions started
    if inputs.thread_changed {
        if let Some(mission) = data::script_mission(inputs.thread) {
            let key = start_key(mission);
            if toggles.get(&key) && attempt.first_time(&key) {
                return Some(Split::MissionStart(key));
            }
        }
    }

    // Collectibles
    for c in COLLECTIBLES {
        let count = inputs.pair(c.index, c.shift);
        if count.current > count.old {
            let name = format!("{} {}", c.kind, count.current);
            if toggles.get(&format!("{}All", c.kind))
                && count.current == c.max
                && count.old == c.max - 1
                && attempt.first_time(&name)
            {
                return Some(Split::AllCollectibles(name));
            }
            if toggles.get(&format!("{}Each", c.kind)) && attempt.first_time(&name) {
                return Some(Split::Collectible(name));
            }
        }
    }

    // Any% final split
    if inputs.thread == EXCHANGE_THREAD
        && inputs.pair(TE_HELIPAD, Shift::TeHelipad).current == 1
        && inputs.pair(TE_TIMER, Shift::TeTimer).changed()
        && toggles.get("any_end")
        && attempt.first_time("any_end")
    {
        return Some(Split::AnyEnd);
    }
    None
}

#[cfg(test)]
mod tests {
    use alloc::{collections::BTreeSet, vec, vec::Vec};

    use super::*;
    use crate::version::Group;

    /// Settings by the keys that are on.
    struct On(BTreeSet<String>);

    impl On {
        fn new(keys: &[&str]) -> Self {
            On(keys.iter().map(|k| String::from(*k)).collect())
        }
    }

    impl Toggles for On {
        fn get(&self, key: &str) -> bool {
            self.0.contains(key)
        }
    }

    /// Globals of the version `layout`, and what's running.
    struct Game {
        layout: Layout,
        old: Vec<i32>,
        current: Vec<i32>,
        thread: &'static str,
        thread_changed: bool,
    }

    impl Game {
        fn new(group: Group) -> Self {
            let layout = group.layout();
            let n = globals_len(&layout);
            Game { layout, old: vec![0; n], current: vec![0; n], thread: "", thread_changed: false }
        }

        /// Sets a global, on this tick only.
        fn set(&mut self, index: u32, shift: Shift, old: i32, current: i32) {
            let i = (index + self.layout.shift(shift)) as usize;
            self.old[i] = old;
            self.current[i] = current;
        }

        fn mission(&mut self, name: &str, old: i32, current: i32) {
            let (index, _) =
                MISSIONS.iter().flat_map(|(_, ms)| ms.iter()).find(|(_, m)| *m == name).unwrap();
            self.set(*index, Shift::Mission, old, current);
        }

        fn collectible(&mut self, kind: &str, old: i32, current: i32) {
            let c = COLLECTIBLES.iter().find(|c| c.kind == kind).unwrap();
            self.set(c.index, c.shift, old, current);
        }

        fn inputs(&self) -> Inputs<'_> {
            Inputs {
                old: &self.old,
                current: &self.current,
                thread: self.thread,
                thread_changed: self.thread_changed,
                layout: self.layout,
            }
        }

        fn check(&self, on: &On, attempt: &mut Attempt) -> Option<Split> {
            check(on, attempt, &self.inputs())
        }
    }

    fn game() -> Game {
        Game::new(Group::One)
    }

    #[test]
    fn nothing_happens_when_nothing_changes() {
        let on = On::new(&["The Getaway (end)", "any_end", "RampagesEach"]);
        assert_eq!(game().check(&on, &mut Attempt::new()), None);
    }

    #[test]
    fn mission_end_on_the_flag_going_up_by_one() {
        let on = On::new(&["The Crook (end)"]);
        let mut a = Attempt::new();
        let mut g = game();
        g.mission("The Crook", 0, 1);
        assert_eq!(g.check(&on, &mut a), Some(Split::MissionEnd("The Crook (end)".into())));
        // Only once.
        assert_eq!(g.check(&on, &mut a), None);
    }

    #[test]
    fn mission_end_needs_exactly_one_more() {
        let on = On::new(&["The Crook (end)", "The Thieves (end)"]);
        let mut g = game();
        g.mission("The Crook", 0, 2);
        g.mission("The Thieves", 1, 1);
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
        g.mission("The Crook", 2, 1);
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
    }

    #[test]
    fn mission_end_needs_its_setting() {
        let mut g = game();
        g.mission("The Crook", 0, 1);
        assert_eq!(g.check(&On::new(&["The Thieves (end)"]), &mut Attempt::new()), None);
        // Start is a different setting.
        assert_eq!(g.check(&On::new(&["The Crook (start)"]), &mut Attempt::new()), None);
    }

    #[test]
    fn mission_end_follows_the_layout() {
        let on = On::new(&["Turismo (end)"]);
        let mut g = Game::new(Group::Three);
        g.mission("Turismo", 0, 1);
        assert!(g.check(&on, &mut Attempt::new()).is_some());
        // Its unshifted index is another global.
        let mut g = Game::new(Group::Three);
        let (index, _) =
            MISSIONS.iter().flat_map(|(_, ms)| ms.iter()).find(|(_, m)| *m == "Turismo").unwrap();
        g.current[*index as usize] = 1;
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
    }

    #[test]
    fn mission_start_on_a_known_thread_changing() {
        let on = On::new(&["Van Heist (start)"]);
        let mut a = Attempt::new();
        let mut g = game();
        g.thread = "joey3";
        g.thread_changed = true;
        assert_eq!(g.check(&on, &mut a), Some(Split::MissionStart("Van Heist (start)".into())));
        assert_eq!(g.check(&on, &mut a), None);
    }

    #[test]
    fn mission_start_needs_a_change_a_known_thread_and_its_setting() {
        let on = On::new(&["Van Heist (start)"]);
        let mut g = game();
        g.thread = "joey3";
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
        g.thread_changed = true;
        g.thread = "joey4";
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
        g.thread = "unknown";
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
        g.thread = "";
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
    }

    #[test]
    fn each_collectible_splits_once_per_count() {
        let on = On::new(&["Hidden PackagesEach"]);
        let mut a = Attempt::new();
        let mut g = game();
        g.collectible("Hidden Packages", 4, 5);
        assert_eq!(g.check(&on, &mut a), Some(Split::Collectible("Hidden Packages 5".into())));
        // Collecting it again after loading a save doesn't split.
        assert_eq!(g.check(&on, &mut a), None);
        g.collectible("Hidden Packages", 5, 6);
        assert_eq!(g.check(&on, &mut a), Some(Split::Collectible("Hidden Packages 6".into())));
        // Going down is nothing.
        g.collectible("Hidden Packages", 6, 5);
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
    }

    #[test]
    fn collectibles_need_their_setting() {
        let mut g = game();
        g.collectible("Rampages", 4, 5);
        assert_eq!(g.check(&On::new(&["Rampages"]), &mut Attempt::new()), None);
        assert_eq!(g.check(&On::new(&["RampagesAll"]), &mut Attempt::new()), None);
        assert_eq!(g.check(&On::new(&["Unique Stunt JumpsEach"]), &mut Attempt::new()), None);
    }

    #[test]
    fn all_collectibles_split_on_the_last_one() {
        for (kind, max) in [("Hidden Packages", 100), ("Unique Stunt Jumps", 20), ("Rampages", 20)]
        {
            let on = On::new(&[&format!("{kind}All")]);
            let mut a = Attempt::new();
            let mut g = game();
            g.collectible(kind, max - 2, max - 1);
            assert_eq!(g.check(&on, &mut a), None, "{kind}");
            g.collectible(kind, max - 1, max);
            assert_eq!(
                g.check(&on, &mut a),
                Some(Split::AllCollectibles(format!("{kind} {max}"))),
                "{kind}"
            );
            assert_eq!(g.check(&on, &mut a), None, "{kind}");
        }
    }

    #[test]
    fn all_collectibles_are_not_the_max_of_another_kind() {
        // 20 packages out of 100.
        let on = On::new(&["Hidden PackagesAll"]);
        let mut g = game();
        g.collectible("Hidden Packages", 19, 20);
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
    }

    #[test]
    fn all_and_each_share_the_dedupe_name() {
        let on = On::new(&["RampagesAll", "RampagesEach"]);
        let mut a = Attempt::new();
        let mut g = game();
        g.collectible("Rampages", 19, 20);
        assert_eq!(g.check(&on, &mut a), Some(Split::AllCollectibles("Rampages 20".into())));
        assert_eq!(g.check(&on, &mut a), None);
    }

    #[test]
    fn any_end_needs_the_thread_the_helipad_and_the_timer_changing() {
        let on = On::new(&["any_end"]);
        let mut g = game();
        g.thread = "cat1";
        g.set(TE_HELIPAD, Shift::TeHelipad, 1, 1);
        // The timer hasn't changed.
        g.set(TE_TIMER, Shift::TeTimer, 5, 5);
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
        // The helipad isn't 1.
        g.set(TE_HELIPAD, Shift::TeHelipad, 0, 0);
        g.set(TE_TIMER, Shift::TeTimer, 5, 6);
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
        // Another thread.
        g.set(TE_HELIPAD, Shift::TeHelipad, 1, 1);
        g.thread = "cat2";
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
        // Without the setting.
        g.thread = "cat1";
        assert_eq!(g.check(&On::new(&[]), &mut Attempt::new()), None);
        // All of it.
        let mut a = Attempt::new();
        assert_eq!(g.check(&on, &mut a), Some(Split::AnyEnd));
        assert_eq!(g.check(&on, &mut a), None);
    }

    #[test]
    fn any_end_follows_the_layout() {
        let on = On::new(&["any_end"]);
        let mut g = Game::new(Group::Three);
        g.thread = "cat1";
        g.set(TE_HELIPAD, Shift::TeHelipad, 1, 1);
        g.set(TE_TIMER, Shift::TeTimer, 5, 6);
        assert_eq!(g.check(&on, &mut Attempt::new()), Some(Split::AnyEnd));
    }

    #[test]
    fn all_done_on_the_last_mission_of_a_giver() {
        let on = On::new(&["Joey Leone (end)#allDone"]);
        let mut a = Attempt::new();
        let mut g = game();
        let missions = [
            "Mike Lips Last Lunch",
            "Farewell 'Chunky' Lee Chong",
            "Van Heist",
            "Cipriani's Chauffeur",
            "Dead Skunk in The Trunk",
            "The Getaway",
        ];
        for m in missions {
            g.mission(m, 1, 1);
        }
        g.mission("Mike Lips Last Lunch", 0, 1);
        assert_eq!(g.check(&on, &mut a), Some(Split::AllDone("Joey Leone (end)#allDone".into())));
        assert_eq!(g.check(&on, &mut a), None);
    }

    #[test]
    fn all_done_waits_for_every_mission() {
        let on = On::new(&["Marty Chonks (end)#allDone"]);
        let mut g = game();
        g.mission("The Crook", 0, 1);
        assert_eq!(g.check(&on, &mut Attempt::new()), None);
        for m in ["The Thieves", "The Wife", "Her Lover"] {
            g.mission(m, 1, 1);
        }
        // And needs its setting.
        let mut a = Attempt::new();
        assert_eq!(g.check(&On::new(&[]), &mut a), None);
        // Not recorded as done, so it still splits once it's on.
        assert!(g.check(&on, &mut a).is_some());
    }

    #[test]
    fn all_done_comes_before_the_mission_split() {
        // The Exchange is the only mission of Catalina.
        let on = On::new(&["Catalina (end)#allDone", "The Exchange (end)"]);
        let mut a = Attempt::new();
        let mut g = game();
        g.mission("The Exchange", 0, 1);
        assert_eq!(g.check(&on, &mut a), Some(Split::AllDone("Catalina (end)#allDone".into())));
        assert_eq!(g.check(&on, &mut a), Some(Split::MissionEnd("The Exchange (end)".into())));
    }

    #[test]
    fn splits_in_the_asl_order_one_per_tick() {
        let on = On::new(&["The Crook (end)", "Van Heist (start)", "RampagesEach", "any_end"]);
        let mut a = Attempt::new();
        let mut g = game();
        g.mission("The Crook", 0, 1);
        g.thread = "cat1";
        g.thread_changed = true;
        g.collectible("Rampages", 0, 1);
        g.set(TE_HELIPAD, Shift::TeHelipad, 1, 1);
        g.set(TE_TIMER, Shift::TeTimer, 1, 2);
        assert!(matches!(g.check(&on, &mut a), Some(Split::MissionEnd(_))));
        g.thread = "joey3";
        assert!(matches!(g.check(&on, &mut a), Some(Split::MissionStart(_))));
        assert!(matches!(g.check(&on, &mut a), Some(Split::Collectible(_))));
        g.thread = "cat1";
        assert_eq!(g.check(&on, &mut a), Some(Split::AnyEnd));
        assert_eq!(g.check(&on, &mut a), None);
    }

    #[test]
    fn clearing_the_attempt_splits_again() {
        let on = On::new(&["The Crook (end)"]);
        let mut a = Attempt::new();
        let mut g = game();
        g.mission("The Crook", 0, 1);
        assert!(g.check(&on, &mut a).is_some());
        assert_eq!(g.check(&on, &mut a), None);
        a.clear();
        assert!(g.check(&on, &mut a).is_some());
    }

    #[test]
    fn globals_len_covers_every_global() {
        for group in [Group::One, Group::Two, Group::Three] {
            let layout = group.layout();
            let len = globals_len(&layout);
            let mut g = Game::new(group);
            // Setting every global would panic on one beyond the end.
            for (_, ms) in MISSIONS {
                for (index, _) in *ms {
                    g.set(*index, Shift::Mission, 0, 0);
                }
            }
            for c in COLLECTIBLES {
                g.set(c.index, c.shift, 0, 0);
            }
            g.set(TE_TIMER, Shift::TeTimer, 0, 0);
            g.set(TE_HELIPAD, Shift::TeHelipad, 0, 0);
            assert_eq!(g.old.len(), len);
        }
        // The last global is the Group 3 helipad.
        assert_eq!(globals_len(&Group::Three.layout()), (TE_HELIPAD + 22) as usize + 1);
    }

    #[test]
    fn keys_are_the_asl_settings() {
        assert_eq!(end_key("Turismo"), "Turismo (end)");
        assert_eq!(start_key("Turismo"), "Turismo (start)");
        assert_eq!(all_done_key("El Burro"), "El Burro (end)#allDone");
    }
}
