//! The events that can start or reset the timer: a New Game and loading a
//! save.
//!
//! Doesn't touch `asr`, so it can be tested on the host.

/// Play time (ms) below which the game counts as new. A save has more play
/// time than this, so its start flag change can't look like a New Game, and a
/// New Game can't look like loading a save.
const NEW_GAME_PLAY_TIME: i32 = 5 * 1000;

/// What happened on a tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Events {
    pub new_game: bool,
    pub save_loaded: bool,
}

pub struct Triggers {
    /// Whether a New Game may still start/reset the timer. The start flag
    /// flips 1 -> 0 more than once during the intro (again ~5s in, still under
    /// the play time limit), which would reset and restart the timer, so only
    /// the first one counts until play time goes backwards again (New Game or
    /// loading a save).
    new_game_armed: bool,
}

impl Triggers {
    pub fn new() -> Self {
        Self { new_game_armed: true }
    }

    /// Takes each value's `(old, current)` pair for this tick.
    pub fn update(
        &mut self,
        loading: (u8, u8),
        start_flag: (u8, u8),
        play_time: (i32, i32),
    ) -> Events {
        let (old_play_time, play_time) = play_time;
        if play_time < old_play_time {
            self.new_game_armed = true;
        }
        // startFlag switches to 0 from 1 when the game begins to fade out to
        // the intro cutscene. The check for the playing time is there so the
        // timer doesn't start/reset when loading a save.
        let new_game =
            self.new_game_armed && start_flag == (1, 0) && play_time < NEW_GAME_PLAY_TIME;
        if new_game {
            self.new_game_armed = false;
        }
        // Triggered once the load is over, when the player gets control, so
        // the time spent on the loading screen isn't counted.
        let save_loaded = loading.0 != 0 && loading.1 == 0 && play_time >= NEW_GAME_PLAY_TIME;
        Events { new_game, save_loaded }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tick with nothing going on, 10 minutes into a save.
    const IDLE: ((u8, u8), (u8, u8), (i32, i32)) = ((0, 0), (0, 0), (600_000, 600_050));

    fn tick(t: &mut Triggers, values: ((u8, u8), (u8, u8), (i32, i32))) -> Events {
        t.update(values.0, values.1, values.2)
    }

    #[test]
    fn nothing_happens_while_playing() {
        let mut t = Triggers::new();
        assert_eq!(tick(&mut t, IDLE), Events::default());
    }

    #[test]
    fn new_game_on_start_flag_dropping() {
        let mut t = Triggers::new();
        let e = tick(&mut t, ((0, 0), (1, 0), (0, 0)));
        assert_eq!(e, Events { new_game: true, save_loaded: false });
    }

    #[test]
    fn new_game_only_once_until_play_time_goes_back() {
        let mut t = Triggers::new();
        assert!(tick(&mut t, ((0, 0), (1, 0), (0, 0))).new_game);
        // The start flag flips again during the intro.
        assert!(!tick(&mut t, ((0, 0), (1, 0), (4_000, 4_050))).new_game);
        // Play time going backwards (another New Game) arms it again.
        tick(&mut t, ((0, 0), (0, 0), (4_050, 0)));
        assert!(tick(&mut t, ((0, 0), (1, 0), (0, 0))).new_game);
    }

    #[test]
    fn save_loaded_when_loading_ends() {
        let mut t = Triggers::new();
        // Not while loading.
        assert!(!tick(&mut t, ((0, 1), (0, 0), (600_000, 600_000))).save_loaded);
        assert!(!tick(&mut t, ((1, 1), (0, 0), (600_000, 1_200_000))).save_loaded);
        // On the tick it ends.
        let e = tick(&mut t, ((1, 0), (0, 0), (1_200_000, 1_200_000)));
        assert_eq!(e, Events { new_game: false, save_loaded: true });
        // Not again after.
        assert!(!tick(&mut t, ((0, 0), (0, 0), (1_200_000, 1_200_050))).save_loaded);
    }

    #[test]
    fn new_game_load_is_not_a_save_load() {
        let mut t = Triggers::new();
        assert!(!tick(&mut t, ((1, 0), (1, 1), (0, 0))).save_loaded);
    }

    #[test]
    fn start_flag_dropping_in_a_save_is_not_a_new_game() {
        let mut t = Triggers::new();
        assert!(!tick(&mut t, ((0, 0), (1, 0), (600_000, 600_050))).new_game);
    }
}
