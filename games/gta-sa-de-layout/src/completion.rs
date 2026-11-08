//! When a section counts as done, for the "All done" section option: none of
//! the section's own splits happen, only one split on the tick its last item
//! is done. The ASL never did this.
//!
//! Doesn't touch `asr`, so it can be tested on the host.

use alloc::vec::Vec;

use crate::data::{self, CHILIAD_DONE, QUARRY, TRUCKING};

/// What a global has to be for its item to count as done.
#[derive(Clone, Copy, Debug)]
pub enum Rule {
    /// Above 0, like the `data::MISSIONS2` splits.
    Set,
    /// At least this value, for mission counters.
    AtLeast(i32),
    /// Exactly this value.
    Is(i32),
}

impl Rule {
    fn done(self, value: i32) -> bool {
        match self {
            Rule::Set => value > 0,
            Rule::AtLeast(n) => value >= n,
            Rule::Is(n) => value == n,
        }
    }
}

/// What a section counts.
pub enum Count {
    /// Every global of these `data::MISSIONS2` groups.
    Groups(&'static [&'static str]),
    /// The `data::MISSIONS` counter of a mission strand, up to its last
    /// mission.
    Strand(u32),
    /// The `data::MISSIONS2` challenges, and the Chiliad Challenge being done
    /// (`CHILIAD_DONE`, set once all three races were won).
    Challenges,
}

/// The sections that offer "All done", and what they count. Settings in them
/// that aren't counted:
/// - Schools: "Driving School Started", a split on starting a mission.
/// - Properties: "Wang Cars (Showroom Bought)" and "Verdant Meadows
///   (Safehouse)", only seen when their mission starts (no global keeps
///   them).
/// - Challenges: "Chiliad Challenge #1" to "#3", counted through
///   `CHILIAD_DONE` instead.
///
/// Sections containing other sections don't offer it.
pub static SECTIONS: &[(&str, Count)] = &[
    ("HeistMissions", Count::Strand(602)),
    ("ZeroMissions", Count::Strand(544)),
    ("Wang CarsMissions", Count::Strand(546)),
    ("TruckingMissions", Count::Strand(TRUCKING)),
    ("QuarryMissions", Count::Strand(QUARRY)),
    ("Assets", Count::Groups(&["Assets"])),
    ("Schools", Count::Groups(&["Schools"])),
    ("Vehicle Submissions", Count::Groups(&["Vehicle Submissions", "Freight"])),
    ("LS Races", Count::Groups(&["LS Races"])),
    ("SF Races", Count::Groups(&["SF Races"])),
    ("LV Races", Count::Groups(&["LV Races"])),
    ("Air Races", Count::Groups(&["Air Races"])),
    ("Stadium Events", Count::Groups(&["Stadium Events"])),
    ("Challenges", Count::Challenges),
    ("Gym Moves", Count::Groups(&["Gym Moves"])),
    ("Properties", Count::Groups(&["Properties"])),
];

fn strand(index: u32) -> &'static [(i32, &'static str)] {
    data::MISSIONS.iter().find(|(i, _)| *i == index).unwrap().1
}

impl Count {
    /// The globals counted and when each is done.
    pub fn items(&self) -> Vec<(u32, Rule)> {
        match self {
            Count::Groups(groups) => groups
                .iter()
                .flat_map(|g| data::missions2(g))
                .map(|(index, _)| (*index, Rule::Set))
                .collect(),
            Count::Strand(index) => {
                let last = strand(*index).iter().map(|(v, _)| *v).max().unwrap();
                Vec::from([(*index, Rule::AtLeast(last))])
            }
            Count::Challenges => data::missions2("Challenges")
                .iter()
                .map(|(index, _)| (*index, Rule::Set))
                .chain([(CHILIAD_DONE, Rule::Is(1))])
                .collect(),
        }
    }

    /// The settings counted, by key.
    #[cfg(test)]
    pub fn names(&self) -> Vec<&'static str> {
        match self {
            Count::Groups(groups) => {
                groups.iter().flat_map(|g| data::missions2(g)).map(|(_, name)| *name).collect()
            }
            Count::Strand(index) => strand(*index).iter().map(|(_, name)| *name).collect(),
            Count::Challenges => {
                data::missions2("Challenges").iter().map(|(_, name)| *name).collect()
            }
        }
    }
}

/// Whether everything became done on this tick: every item is done now, and
/// at least one of them wasn't on the previous tick. Takes each item's rule
/// and its global's `(old, current)` values.
pub fn just_completed(items: impl IntoIterator<Item = (Rule, (i32, i32))>) -> bool {
    let mut changed = false;
    for (rule, (old, current)) in items {
        if !rule.done(current) {
            return false;
        }
        changed |= !rule.done(old);
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(values: &[(i32, i32)]) -> bool {
        just_completed(values.iter().map(|v| (Rule::Set, *v)))
    }

    #[test]
    fn flags_complete_on_the_last_one() {
        // Not before the last item.
        assert!(!flags(&[(0, 1), (0, 0), (0, 0)]));
        assert!(!flags(&[(1, 1), (0, 1), (0, 0)]));
        // On the tick the last one is set (some go straight to 2).
        assert!(flags(&[(1, 1), (1, 1), (0, 2)]));
        // Not again once everything is done.
        assert!(!flags(&[(1, 1), (1, 1), (2, 2)]));
    }

    #[test]
    fn counters_complete_on_the_last_mission() {
        let rule = Rule::AtLeast(6);
        assert!(!just_completed([(rule, (4, 5))]));
        assert!(just_completed([(rule, (5, 6))]));
        assert!(!just_completed([(rule, (6, 6))]));
        assert!(!just_completed([(rule, (6, 7))]));
    }

    #[test]
    fn chiliad_completes_on_the_done_flag() {
        let items = |done: (i32, i32)| {
            [(Rule::Set, (1, 1)), (Rule::Set, (1, 1)), (Rule::Set, (1, 1)), (Rule::Is(1), done)]
        };
        assert!(!just_completed(items((0, 0))));
        assert!(just_completed(items((0, 1))));
        assert!(!just_completed(items((1, 1))));
        // The challenges done last, after the Chiliad Challenge.
        let items =
            [(Rule::Set, (1, 1)), (Rule::Set, (0, 1)), (Rule::Set, (1, 1)), (Rule::Is(1), (1, 1))];
        assert!(just_completed(items));
    }

    #[test]
    fn already_complete_does_not_split() {
        // Loading a save with everything done: after the load guard, old and
        // current are the same.
        assert!(!flags(&[(1, 1), (2, 2)]));
        assert!(!just_completed([(Rule::AtLeast(7), (7, 7))]));
    }

    #[test]
    fn strands_count_up_to_their_last_mission() {
        let items = |key| SECTIONS.iter().find(|(k, _)| *k == key).unwrap().1.items();
        assert!(matches!(items("HeistMissions")[..], [(602, Rule::AtLeast(6))]));
        assert!(matches!(items("TruckingMissions")[..], [(TRUCKING, Rule::AtLeast(8))]));
        assert!(matches!(items("QuarryMissions")[..], [(QUARRY, Rule::AtLeast(7))]));
        assert_eq!(items("Vehicle Submissions").len(), 7);
        assert_eq!(items("Properties").len(), 30);
        assert_eq!(items("Challenges").len(), 4);
    }
}
