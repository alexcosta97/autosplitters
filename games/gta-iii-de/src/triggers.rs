//! The event that can start or reset the timer: a New Game.
//!
//! Doesn't touch `asr`, so it can be tested on the host.

use crate::data::INTRO_THREAD;

/// Whether a New Game began on this tick. `start_flag` is its `(old,
/// current)` pair and `thread` the name of the thread running now.
///
/// The flag switches to 0 from 1 when the game begins to fade out to the
/// intro cutscene. The check for the thread is there just for safety. The
/// ASL uses the same check for the start and the reset, so the timer resets
/// and starts in the same cycle.
pub fn new_game(start_flag: (u8, u8), thread: &str) -> bool {
    start_flag == (1, 0) && thread == INTRO_THREAD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_on_the_start_flag_dropping_in_the_intro() {
        assert!(new_game((1, 0), "intro"));
    }

    #[test]
    fn other_flag_changes_are_not_a_new_game() {
        assert!(!new_game((0, 1), "intro"));
        assert!(!new_game((0, 0), "intro"));
        assert!(!new_game((1, 1), "intro"));
        assert!(!new_game((2, 0), "intro"));
    }

    #[test]
    fn other_threads_are_not_a_new_game() {
        assert!(!new_game((1, 0), ""));
        assert!(!new_game((1, 0), "luigi2"));
    }
}
