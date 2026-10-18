//! Splits on the gang territories owned.
//!
//! The game keeps the percentage of territories owned in a script global,
//! `player_territory_owned`, which is what these read. Each split also needs a
//! number of missions of a strand passed, so it happens at the right point of
//! the story.
//!
//! Doesn't touch `asr`, so it can be tested on the host.

use crate::data::{GROVE_PASSED, RIOT_PASSED};

/// `player_territory_owned`: the percentage of territories owned, 0 to 100.
///
/// Refreshed by the `TCASH` script every 20 s of game time (running from
/// passing the `mansio3` script for the last segment of the game) and by the
/// phone call check itself, so it can be up to 20 s behind.
pub const OWNED: u32 = 713;

/// A split that happens once some missions are passed and enough territories
/// are owned.
pub struct Gate {
    /// The setting this gate splits on.
    pub split_id: &'static str,
    /// Global variable counting the missions passed in a strand.
    pub missions: u32,
    /// Missions of that strand that have to be passed.
    pub missions_passed: i32,
    /// Territories owned, in percent, needed.
    pub percent: i32,
}

/// The splits. Each territory is 1/53 of the total at this point of the game
/// (3 territories read 5.66% in game).
pub static GATES: &[Gate] = &[
    // 4 territories, after Beat Down on B Dup and before Grove 4 Life: 7.55%.
    // The global is an integer, so it reads 7 or 8 depending on whether the
    // game truncates or rounds, and 3 territories read 5 or 6. 7 tells the
    // 4th territory apart either way.
    Gate { split_id: "GT #1", missions: GROVE_PASSED, missions_passed: 1, percent: 7 },
    // "End of the Line" is unlocked once Los Desperados is passed and Sweet's
    // call comes, which the script holds back until more than 34% are owned
    // (`player_territory_owned > 34` in `cell_phone_LA2`).
    Gate { split_id: "Territories Unlock", missions: RIOT_PASSED, missions_passed: 2, percent: 35 },
];

impl Gate {
    fn met(&self, missions: i32, owned: i32) -> bool {
        missions >= self.missions_passed && owned >= self.percent
    }

    /// Whether the gate was met on this tick. Takes each value's
    /// `(old, current)` pair, and works whichever of the two conditions comes
    /// last: passing the mission with the territories already taken, or
    /// taking them after.
    pub fn just_met(&self, missions: (i32, i32), owned: (i32, i32)) -> bool {
        self.met(missions.1, owned.1) && !self.met(missions.0, owned.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gate(split_id: &str) -> &'static Gate {
        GATES.iter().find(|g| g.split_id == split_id).unwrap()
    }

    #[test]
    fn unlocks_when_the_territories_reach_the_minimum_after_los_desperados() {
        assert!(gate("Territories Unlock").just_met((2, 2), (34, 35)));
    }

    #[test]
    fn unlocks_when_los_desperados_is_passed_with_the_territories_taken() {
        assert!(gate("Territories Unlock").just_met((1, 2), (60, 60)));
    }

    #[test]
    fn needs_more_than_34_percent() {
        assert!(!gate("Territories Unlock").just_met((2, 2), (33, 34)));
    }

    #[test]
    fn does_not_unlock_before_los_desperados() {
        assert!(!gate("Territories Unlock").just_met((1, 1), (50, 100)));
    }

    #[test]
    fn only_on_the_tick_it_happens() {
        let g = gate("Territories Unlock");
        assert!(!g.just_met((2, 2), (35, 40)));
        assert!(!g.just_met((2, 3), (50, 50)));
    }

    #[test]
    fn gt1_is_the_fourth_territory_after_beat_down_on_b_dup() {
        let g = gate("GT #1");
        // 4th territory, whether the percentage truncates (7) or rounds (8).
        assert!(g.just_met((1, 1), (5, 7)));
        assert!(g.just_met((1, 1), (6, 8)));
        // Beat Down on B Dup passed with the 4th territory already taken.
        assert!(g.just_met((0, 1), (7, 7)));
    }

    #[test]
    fn gt1_is_not_met_by_three_territories() {
        let g = gate("GT #1");
        // 3 territories read 5.66%, so 5 or 6.
        assert!(!g.just_met((0, 1), (5, 5)));
        assert!(!g.just_met((0, 1), (6, 6)));
    }

    #[test]
    fn gt1_needs_beat_down_on_b_dup() {
        assert!(!gate("GT #1").just_met((0, 0), (5, 9)));
    }

    #[test]
    fn four_territories_of_53_read_7_or_8() {
        let exact: f64 = 4.0 * 100.0 / 53.0;
        assert_eq!(exact as i32, 7);
        assert_eq!(exact.round() as i32, 8);
    }
}
