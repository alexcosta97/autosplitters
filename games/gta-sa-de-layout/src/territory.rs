//! Splits on the gang territories owned.
//!
//! The game keeps the percentage of territories owned in a script global,
//! `player_territory_owned`, which is what these read. They only count once
//! the territories side quest has been enabled again on the way back to Los
//! Santos, whatever the route to get there.
//!
//! Doesn't touch `asr`, so it can be tested on the host.

use crate::data::MANSION_PASSED;

/// `player_territory_owned`: the percentage of territories owned, 0 to 100.
///
/// Refreshed by the `TCASH` script every 20 s of game time (running again from
/// the second enablement on) and by the phone call check itself, so it can be
/// up to 20 s behind.
pub const OWNED: u32 = 713;

/// Global variable that tells the territories side quest is enabled again on
/// Return to Los Santos, and the value it has from then on.
///
/// The first enablement is the tutorial gang war, which is switched off again
/// before leaving Los Santos. The second one is in `mission_mansion3_passed`,
/// which in one go sets `SET_GANG_WARS_ACTIVE TRUE`, starts the territory
/// income script and raises the mansion missions passed
/// (`flag_mansion_mission_counter`) to 3. The script has no flag of its own for
/// it: the one global the tutorial sets, `flag_gang_war_active`, isn't set
/// again.
pub const ENABLED: (u32, i32) = (MANSION_PASSED, 3);

/// A split that happens once the territories side quest is enabled and enough
/// territories are owned.
pub struct Gate {
    /// The setting this gate splits on.
    pub split_id: &'static str,
    /// Territories owned, in percent, needed.
    pub percent: i32,
}

/// The splits. Each territory is 1/53 of the total at this point of the game
/// (3 territories read 5.66% in game).
pub static GATES: &[Gate] = &[
    // 7 territories: 7/53 = 13.21%. The global is an integer, and reads 13
    // whether the game truncates or rounds. 6 territories read 11.
    Gate { split_id: "GT #1", percent: 13 },
    // 35%, which takes 19 territories: 18 read 33 or 34. It's the share of
    // territories Sweet's call needs to unlock "End of the Line", which the
    // script holds back until more than 34% are owned (`player_territory_owned
    // > 34` in `cell_phone_LA2`).
    Gate { split_id: "GT #2", percent: 35 },
];

impl Gate {
    fn met(&self, enabled: i32, owned: i32) -> bool {
        enabled >= ENABLED.1 && owned >= self.percent
    }

    /// Whether the gate was met on this tick. Takes the `(old, current)` pair
    /// of the `ENABLED` global and of `OWNED`, and works whichever of the two
    /// conditions comes last: the side quest being enabled with the
    /// territories already taken, or taking them after.
    pub fn just_met(&self, enabled: (i32, i32), owned: (i32, i32)) -> bool {
        self.met(enabled.1, owned.1) && !self.met(enabled.0, owned.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The side quest being enabled, on the tick before and during.
    const ON: (i32, i32) = (3, 3);
    const OFF: (i32, i32) = (2, 2);

    fn gate(split_id: &str) -> &'static Gate {
        GATES.iter().find(|g| g.split_id == split_id).unwrap()
    }

    #[test]
    fn gt1_is_met_by_the_seventh_territory() {
        assert!(gate("GT #1").just_met(ON, (11, 13)));
    }

    #[test]
    fn gt1_is_not_met_by_six_territories() {
        // 6/53 is 11.32%, which reads 11 whether it truncates or rounds.
        assert!(!gate("GT #1").just_met(ON, (10, 11)));
    }

    #[test]
    fn gt2_is_met_at_35_percent() {
        assert!(gate("GT #2").just_met(ON, (34, 35)));
    }

    #[test]
    fn gt2_needs_more_than_34_percent() {
        assert!(!gate("GT #2").just_met(ON, (33, 34)));
    }

    #[test]
    fn only_on_the_tick_it_happens() {
        let g = gate("GT #2");
        assert!(!g.just_met(ON, (35, 40)));
        assert!(!g.just_met((3, 4), (50, 50)));
    }

    #[test]
    fn not_before_the_side_quest_is_enabled() {
        assert!(!gate("GT #1").just_met(OFF, (5, 20)));
    }

    #[test]
    fn met_when_the_side_quest_is_enabled_with_the_territories_taken() {
        assert!(gate("GT #1").just_met((2, 3), (15, 15)));
    }

    #[test]
    fn territory_counts_read_the_same_truncated_or_rounded_where_it_matters() {
        let percent = |n: f64| n * 100.0 / 53.0;
        // GT #1 (7) and 6 territories before it.
        assert_eq!((percent(7.0) as i32, percent(7.0).round() as i32), (13, 13));
        assert_eq!((percent(6.0) as i32, percent(6.0).round() as i32), (11, 11));
        // GT #2 (35%) takes 19: 18 stay under 35.
        assert!(percent(18.0).round() < 35.0);
        assert!(percent(19.0) as i32 >= 35);
    }
}
