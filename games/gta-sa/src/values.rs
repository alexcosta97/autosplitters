//! The memory values the ASL watches, as `(old, current)` pairs. Kept apart
//! from the reading itself (in `lib.rs`), so the splitting logic can be tested
//! without the runtime.

use alloc::{collections::BTreeMap, vec::Vec};

use gta_sa_common::pair::Pair;

/// What one tick read; `None` where a read failed.
pub struct Reading {
    /// Keyed by the ASL's 1.0 address.
    pub globals: Vec<(u32, Option<i32>)>,
    pub loading: Option<bool>,
    pub ped_status: Option<i32>,
    pub playing_time: Option<i32>,
    pub thread: Option<[u8; 10]>,
    /// Parallel to `data::COLLECTIBLES`.
    pub collectibles: [Option<i32>; 5],
}

#[derive(Default)]
pub struct Values {
    /// Script globals, keyed by the ASL's 1.0 address (before the offset).
    pub globals: BTreeMap<u32, Pair<i32>>,
    pub loading: Pair<bool>,
    pub ped_status: Pair<i32>,
    pub playing_time: Pair<i32>,
    pub thread: Pair<[u8; 10]>,
    /// Parallel to `data::COLLECTIBLES`.
    pub collectibles: [Pair<i32>; 5],
}

impl Values {
    /// The values after the first read: old and current are the same, so
    /// nothing looks changed.
    pub fn first(r: &Reading) -> Values {
        let seed = |value: Option<i32>| Pair::new(value.unwrap_or_default());
        Values {
            globals: r.globals.iter().map(|(addr, value)| (*addr, seed(*value))).collect(),
            loading: Pair::new(r.loading.unwrap_or_default()),
            ped_status: seed(r.ped_status),
            playing_time: seed(r.playing_time),
            thread: Pair::new(r.thread.unwrap_or_default()),
            collectibles: r.collectibles.map(seed),
        }
    }

    /// The next tick's read. A failed read keeps the previous value.
    pub fn update(&mut self, r: &Reading) {
        for (addr, value) in &r.globals {
            self.globals.entry(*addr).or_default().update(*value);
        }
        self.loading.update(r.loading);
        self.ped_status.update(r.ped_status);
        self.playing_time.update(r.playing_time);
        self.thread.update(r.thread);
        for (pair, value) in self.collectibles.iter_mut().zip(r.collectibles) {
            pair.update(value);
        }
    }

    /// The global at the ASL's 1.0 address `addr`.
    pub fn global(&self, addr: u32) -> Pair<i32> {
        self.globals.get(&addr).copied().unwrap_or_default()
    }

    /// The old and current name of the first thread, each up to the first
    /// NUL (the whole buffer if there's none).
    pub fn thread_name(&self) -> (&[u8], &[u8]) {
        fn trim(s: &[u8; 10]) -> &[u8] {
            &s[..s.iter().position(|&b| b == 0).unwrap_or(s.len())]
        }
        (trim(&self.thread.old), trim(&self.thread.current))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn reading() -> Reading {
        Reading {
            globals: vec![(0x64A060, Some(3)), (0x651698, Some(2))],
            loading: Some(true),
            ped_status: Some(55),
            playing_time: Some(1000),
            thread: Some(*b"intro\0xxxx"),
            collectibles: [Some(1), Some(2), Some(3), Some(4), Some(5)],
        }
    }

    #[test]
    fn thread_name_trims_at_nul() {
        let mut v = Values::first(&reading());
        assert_eq!(v.thread_name(), (&b"intro"[..], &b"intro"[..]));
        v.thread.update(Some(*b"abcdefghij"));
        assert_eq!(v.thread_name(), (&b"intro"[..], &b"abcdefghij"[..]));
    }

    #[test]
    fn first_values_have_no_changes() {
        let v = Values::first(&reading());
        assert_eq!(v.global(0x64A060), Pair::new(3));
        assert_eq!(v.global(0x651698), Pair::new(2));
        assert_eq!(v.loading, Pair::new(true));
        assert_eq!(v.ped_status, Pair::new(55));
        assert_eq!(v.playing_time, Pair::new(1000));
        assert_eq!(v.thread, Pair::new(*b"intro\0xxxx"));
        for (i, c) in v.collectibles.iter().enumerate() {
            assert_eq!(*c, Pair::new(i as i32 + 1));
        }
    }

    #[test]
    fn update_keeps_failed_reads() {
        let mut v = Values::first(&reading());
        v.update(&Reading {
            globals: vec![(0x64A060, Some(4)), (0x651698, None)],
            loading: None,
            ped_status: Some(63),
            playing_time: None,
            thread: None,
            collectibles: [None, Some(3), None, None, None],
        });
        assert_eq!(v.global(0x64A060).pair(), (3, 4));
        assert_eq!(v.global(0x651698).pair(), (2, 2));
        assert_eq!(v.loading.pair(), (true, true));
        assert_eq!(v.ped_status.pair(), (55, 63));
        assert_eq!(v.playing_time.pair(), (1000, 1000));
        assert_eq!(v.collectibles[1].pair(), (2, 3));
        assert_eq!(v.collectibles[0].pair(), (1, 1));
    }

    #[test]
    fn unknown_global_is_default() {
        let v = Values::first(&reading());
        assert_eq!(v.global(0x1234), Pair::default());
    }
}
