//! When a mission giver's missions count as done, for the "All done" option of
//! each of the "Missions (end)" sections: none of the section's own splits
//! happen, only one split on the tick its last mission is passed. The ASL
//! never did this.
//!
//! Doesn't touch `asr`, so it can be tested on the host.

/// Whether everything became done on this tick: every mission's global is at
/// least 1 now, and at least one wasn't on the previous tick. Takes each
/// global's `(old, current)` values.
pub fn just_completed(missions: impl IntoIterator<Item = (i32, i32)>) -> bool {
    let mut changed = false;
    for (old, current) in missions {
        if current < 1 {
            return false;
        }
        changed |= old < 1;
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completes_on_the_last_mission() {
        // Not before the last mission.
        assert!(!just_completed([(0, 1), (0, 0), (0, 0)]));
        assert!(!just_completed([(1, 1), (0, 1), (0, 0)]));
        // On the tick it's passed.
        assert!(just_completed([(1, 1), (1, 1), (0, 1)]));
        // Not again once everything is done.
        assert!(!just_completed([(1, 1), (1, 1), (1, 1)]));
    }

    #[test]
    fn single_mission_givers_complete_with_their_mission() {
        assert!(just_completed([(0, 1)]));
        assert!(!just_completed([(1, 1)]));
    }

    #[test]
    fn already_complete_does_not_split() {
        // Loading a save with everything done: after the load guard, old and
        // current are the same.
        assert!(!just_completed([(1, 1), (2, 2)]));
    }

    #[test]
    fn nothing_to_complete_does_not_split() {
        assert!(!just_completed([]));
    }
}
