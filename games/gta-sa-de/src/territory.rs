//! Splits on the gang territories owned.
//!
//! The game keeps the percentage of territories owned in a script global,
//! `player_territory_owned`, which is what these read. A split can also need a
//! number of missions of a strand passed, so it happens at the right point of
//! the story.
//!
//! Doesn't touch `asr`, so it can be tested on the host.

use crate::data::GROVE_PASSED;

/// `player_territory_owned`: the percentage of territories owned, 0 to 100.
///
/// Refreshed by the `TCASH` script every 20 s of game time (running from
/// passing the `mansio3` script for the last segment of the game) and by the
/// phone call check itself, so it can be up to 20 s behind.
pub const OWNED: u32 = 713;

/// A split that happens once enough territories are owned, and some missions
/// are passed if it needs them.
pub struct Gate {
    /// The setting this gate splits on.
    pub split_id: &'static str,
    /// Global variable counting the missions passed in a strand, and how many
    /// of them have to be passed.
    pub missions: Option<(u32, i32)>,
    /// Territories owned, in percent, needed.
    pub percent: i32,
}

/// The splits. Each territory is 1/53 of the total at this point of the game
/// (3 territories read 5.66% in game).
pub static GATES: &[Gate] = &[
    // 7 territories, which Grove 4 Life ends on: the 3 owned after Beat Down
    // on B Dup plus 4 more, 7/53 = 13.21%. The global is an integer, and reads
    // 13 whether the game truncates or rounds. 6 territories read 11.
    Gate { split_id: "GT #1", missions: Some((GROVE_PASSED, 1)), percent: 13 },
    // 35%, which takes 19 territories: 18 read 33 or 34. It's the share of
    // territories Sweet's call needs to unlock "End of the Line", which the
    // script holds back until more than 34% are owned (`player_territory_owned
    // > 34` in `cell_phone_LA2`).
    Gate { split_id: "GT #2", missions: None, percent: 35 },
];

impl Gate {
    fn met(&self, missions: i32, owned: i32) -> bool {
        let needed = self.missions.map_or(0, |(_, passed)| passed);
        missions >= needed && owned >= self.percent
    }

    /// Whether the gate was met on this tick. Takes each value's
    /// `(old, current)` pair (`missions` is ignored if the gate needs none),
    /// and works whichever of the two conditions comes last: passing the
    /// mission with the territories already taken, or taking them after.
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
    fn gt2_is_met_when_the_territories_reach_35_percent() {
        assert!(gate("GT #2").just_met((0, 0), (34, 35)));
    }

    #[test]
    fn gt2_needs_more_than_34_percent() {
        assert!(!gate("GT #2").just_met((0, 0), (33, 34)));
    }

    #[test]
    fn gt2_only_on_the_tick_it_happens() {
        let g = gate("GT #2");
        assert!(!g.just_met((0, 0), (35, 40)));
        assert!(!g.just_met((0, 0), (50, 50)));
    }

    #[test]
    fn gt2_does_not_depend_on_missions() {
        let g = gate("GT #2");
        assert!(g.just_met((0, 0), (20, 40)));
        assert!(g.just_met((2, 3), (34, 35)));
    }

    #[test]
    fn gt1_is_the_seventh_territory_after_beat_down_on_b_dup() {
        let g = gate("GT #1");
        assert!(g.just_met((1, 1), (11, 13)));
        // Beat Down on B Dup passed with the 7 territories already taken.
        assert!(g.just_met((0, 1), (13, 13)));
    }

    #[test]
    fn gt1_is_not_met_by_six_territories() {
        // 6/53 is 11.32%, which reads 11 whether it truncates or rounds.
        assert!(!gate("GT #1").just_met((0, 1), (11, 11)));
    }

    #[test]
    fn gt1_needs_beat_down_on_b_dup() {
        assert!(!gate("GT #1").just_met((0, 0), (11, 15)));
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
