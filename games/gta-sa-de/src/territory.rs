//! When the gang territories unlock "End of the Line".
//!
//! After Los Desperados, Sweet's phone call only comes once the player owns
//! more than 34% of the territories, and "End of the Line" can't be started
//! before that call has been answered. In the leaked DE script source this is
//! `flag_riot_mission_counter > 1` and `player_territory_owned > 34` in
//! `cell_phone_LA2`.
//!
//! Doesn't touch `asr`, so it can be tested on the host.

/// `player_territory_owned`: the percentage of territories owned, 0 to 100.
///
/// Refreshed by the `TCASH` script every 20 s of game time (running from
/// passing the `mansio3` script for the last segment of the game) and by the
/// phone call check itself, so it can be up to 20 s behind.
pub const OWNED: u32 = 713;

/// Missions of the Riot strand that have to be passed, as counted by
/// `RIOT_PASSED`: Riot (1) and Los Desperados (2).
pub const RIOT_MISSIONS_PASSED: i32 = 2;

/// Territories owned, in percent, needed to unlock "End of the Line". The
/// script checks `> 34`.
pub const MIN_PERCENT: i32 = 35;

fn unlocked(riot_passed: i32, owned: i32) -> bool {
    riot_passed >= RIOT_MISSIONS_PASSED && owned >= MIN_PERCENT
}

/// Whether "End of the Line" became unlocked on this tick. Takes each value's
/// `(old, current)` pair, and works whichever of the two conditions comes
/// last: finishing Los Desperados with the territories already taken, or
/// taking them after.
pub fn just_unlocked(riot_passed: (i32, i32), owned: (i32, i32)) -> bool {
    unlocked(riot_passed.1, owned.1) && !unlocked(riot_passed.0, owned.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlocks_when_the_territories_reach_the_minimum_after_los_desperados() {
        assert!(just_unlocked((2, 2), (34, 35)));
    }

    #[test]
    fn unlocks_when_los_desperados_is_passed_with_the_territories_taken() {
        assert!(just_unlocked((1, 2), (60, 60)));
    }

    #[test]
    fn needs_more_than_34_percent() {
        assert!(!just_unlocked((2, 2), (33, 34)));
    }

    #[test]
    fn does_not_unlock_before_los_desperados() {
        assert!(!just_unlocked((1, 1), (50, 100)));
    }

    #[test]
    fn only_on_the_tick_it_happens() {
        assert!(!just_unlocked((2, 2), (35, 40)));
        assert!(!just_unlocked((2, 3), (50, 50)));
    }
}
