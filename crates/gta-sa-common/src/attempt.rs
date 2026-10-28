//! What has been split during this attempt, and double split prevention.

use alloc::{string::String, vec::Vec};

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Split now.
    Split,
    /// Recorded as done, but too soon after the last split to split now.
    Cooldown,
    /// Already split during this attempt.
    AlreadyDone,
}

#[derive(Default)]
pub struct Attempt {
    /// Already split splits during this attempt (until the timer is reset).
    split: Vec<String>,
    /// Tick when the last split was executed (to prevent double-splits).
    last_split: Option<u64>,
}

impl Attempt {
    pub fn new() -> Self {
        Self::default()
    }

    /// Forgets the splits, for when the timer is reset.
    pub fn clear(&mut self) {
        self.split.clear();
    }

    /// Check if splitting should occur based on whether this split has
    /// already been split since the timer started.
    ///
    /// The split is always added to the list of already split splits, even if
    /// the cooldown prevents it, so that cooldown-prevented splits are not
    /// split if a savegame is loaded and the dupe done again. The cooldown
    /// prevents double splits (mostly for duping): it is `cooldown_ticks`
    /// long, and a split with no earlier split is never prevented.
    pub fn try_split(
        &mut self,
        id: &str,
        prevention: bool,
        tick: u64,
        cooldown_ticks: u64,
    ) -> Outcome {
        if self.split.iter().any(|s| s == id) {
            return Outcome::AlreadyDone;
        }
        self.split.push(id.into());
        let since = self.last_split.map_or(u64::MAX, |t| tick - t);
        if !prevention || since > cooldown_ticks {
            self.last_split = Some(tick);
            Outcome::Split
        } else {
            Outcome::Cooldown
        }
    }

    /// Records `id` without looking at the cooldown (or moving it). `false`
    /// if it was already recorded.
    pub fn first_time(&mut self, id: &str) -> bool {
        if self.split.iter().any(|s| s == id) {
            return false;
        }
        self.split.push(id.into());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_once() {
        let mut a = Attempt::new();
        assert_eq!(a.try_split("a", true, 0, 75), Outcome::Split);
        assert_eq!(a.try_split("a", true, 1000, 75), Outcome::AlreadyDone);
    }

    #[test]
    fn cooldown_blocks_and_still_records() {
        let mut a = Attempt::new();
        assert_eq!(a.try_split("a", true, 100, 75), Outcome::Split);
        assert_eq!(a.try_split("b", true, 175, 75), Outcome::Cooldown);
        assert_eq!(a.try_split("b", true, 300, 75), Outcome::AlreadyDone);
        assert_eq!(a.try_split("c", true, 176, 75), Outcome::Split);
    }

    #[test]
    fn prevention_off_ignores_cooldown() {
        let mut a = Attempt::new();
        assert_eq!(a.try_split("a", false, 100, 75), Outcome::Split);
        assert_eq!(a.try_split("b", false, 101, 75), Outcome::Split);
    }

    #[test]
    fn first_split_ignores_cooldown() {
        let mut a = Attempt::new();
        assert_eq!(a.try_split("a", true, 0, 75), Outcome::Split);
    }

    #[test]
    fn clear_allows_again() {
        let mut a = Attempt::new();
        assert_eq!(a.try_split("a", true, 0, 75), Outcome::Split);
        a.clear();
        assert_eq!(a.try_split("a", true, 1000, 75), Outcome::Split);
    }

    #[test]
    fn first_time_records_without_cooldown() {
        let mut a = Attempt::new();
        assert_eq!(a.try_split("a", true, 100, 75), Outcome::Split);
        assert!(a.first_time("Photos 3"));
        assert!(!a.first_time("Photos 3"));
        // The cooldown still counts from tick 100.
        assert_eq!(a.try_split("b", true, 176, 75), Outcome::Split);
    }
}
