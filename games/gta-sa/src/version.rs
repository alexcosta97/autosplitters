//! Version detection and the per-version addresses, ported from the `init`
//! block of `GTASA.asl`.

use crate::data::Collectible;

/// What the ASL reads at `VERSION_VALUE_ADDRS` to recognise a version.
const VERSION_VALUE: i32 = 38079;

/// The values at the version addresses, `None` where unreadable.
#[derive(Default)]
pub struct VersionValues {
    pub v100_eu: Option<i32>,
    pub v100_us: Option<i32>,
    pub v101_eu: Option<i32>,
    pub v101_us: Option<i32>,
    pub v300_steam: Option<i32>,
    pub v101_steam: Option<i32>,
}

/// Module-relative addresses of the version values, in field order.
pub const VERSION_VALUE_ADDRS: [u32; 6] =
    [0x4245BC, 0x42457C, 0x42533C, 0x4252FC, 0x45EC4A, 0x45DEDA];

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Version {
    V1_0,
    V1_01,
    V2_00,
    V3_00Steam,
    V1_01Steam,
    Steam,
}

impl Version {
    pub fn name(self) -> &'static str {
        match self {
            Version::V1_0 => "1.0",
            Version::V1_01 => "1.01",
            Version::V2_00 => "2.00",
            Version::V3_00Steam => "3.00 Steam",
            Version::V1_01Steam => "1.01 Steam",
            Version::Steam => "Steam",
        }
    }

    /// Follows the ASL's sequence of `if`s: later matches overwrite earlier
    /// ones.
    pub fn detect(module_size: u64, values: &VersionValues) -> Option<Version> {
        let is = |v: Option<i32>| v == Some(VERSION_VALUE);
        let mut version = None;
        if is(values.v100_eu) || is(values.v100_us) || module_size == 18313216 {
            version = Some(Version::V1_0);
        }
        if is(values.v101_eu) || is(values.v101_us) || module_size == 34471936 {
            version = Some(Version::V1_01);
        }
        if module_size == 17985536 {
            version = Some(Version::V2_00);
        }
        if is(values.v300_steam) || module_size == 9691136 {
            version = Some(Version::V3_00Steam);
        }
        if is(values.v101_steam) {
            version = Some(Version::V1_01Steam);
        }
        if module_size == 9981952 {
            version = Some(Version::Steam);
        }
        version
    }
}

/// The addresses that depend on the version.
pub struct Addresses {
    /// Added to every module-relative 1.0 address.
    pub offset: u32,
    pub playing_time: u32,
    pub thread: u32,
    pub loading: u32,
    pub player_ped: u32,
    steam: bool,
}

impl Addresses {
    pub fn for_version(v: Version) -> Addresses {
        let offset = match v {
            Version::V1_0 => 0,
            Version::V1_01 | Version::V2_00 => 0x2680,
            Version::V3_00Steam => 0x75130,
            Version::V1_01Steam => 0x75770,
            Version::Steam => 0x77970,
        };
        if v == Version::Steam {
            // These are absolute for the recent Steam version.
            Addresses {
                offset,
                playing_time: 0x706660,
                thread: 0x702D98,
                loading: 0x833995,
                player_ped: 0x8100D0,
                steam: true,
            }
        } else {
            Addresses {
                offset,
                playing_time: 0x68ECF0 + offset,
                thread: 0x68B42C + offset,
                loading: 0x7A67A5 + offset,
                player_ped: 0x77CD98 + offset,
                steam: false,
            }
        }
    }

    /// A script global's address; the offset applies on Steam too.
    pub fn global(&self, addr: u32) -> u32 {
        addr + self.offset
    }

    pub fn collectible(&self, c: &Collectible) -> u32 {
        if self.steam {
            c.steam_addr
        } else {
            c.addr + self.offset
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::COLLECTIBLES;

    fn none() -> VersionValues {
        VersionValues::default()
    }

    #[test]
    fn detects_by_module_size() {
        for (size, v) in [
            (18313216, Version::V1_0),
            (34471936, Version::V1_01),
            (17985536, Version::V2_00),
            (9691136, Version::V3_00Steam),
            (9981952, Version::Steam),
        ] {
            assert_eq!(Version::detect(size, &none()), Some(v));
        }
    }

    #[test]
    fn detects_by_version_value() {
        let s = Some(38079);
        let d = |values| Version::detect(0, &values);
        assert_eq!(d(VersionValues { v100_eu: s, ..none() }), Some(Version::V1_0));
        assert_eq!(d(VersionValues { v100_us: s, ..none() }), Some(Version::V1_0));
        assert_eq!(d(VersionValues { v101_eu: s, ..none() }), Some(Version::V1_01));
        assert_eq!(d(VersionValues { v101_us: s, ..none() }), Some(Version::V1_01));
        assert_eq!(d(VersionValues { v300_steam: s, ..none() }), Some(Version::V3_00Steam));
        assert_eq!(d(VersionValues { v101_steam: s, ..none() }), Some(Version::V1_01Steam));
    }

    #[test]
    fn later_checks_win() {
        let eu = VersionValues { v100_eu: Some(38079), ..none() };
        assert_eq!(Version::detect(9981952, &eu), Some(Version::Steam));
        let st = VersionValues { v101_steam: Some(38079), ..none() };
        assert_eq!(Version::detect(9691136, &st), Some(Version::V1_01Steam));
    }

    #[test]
    fn unknown_is_none() {
        let zero = VersionValues {
            v100_eu: Some(0),
            v100_us: Some(0),
            v101_eu: Some(0),
            v101_us: Some(0),
            v300_steam: Some(0),
            v101_steam: Some(0),
        };
        assert_eq!(Version::detect(1234, &zero), None);
    }

    #[test]
    fn names() {
        assert_eq!(Version::V3_00Steam.name(), "3.00 Steam");
        assert_eq!(Version::V1_0.name(), "1.0");
    }

    #[test]
    fn offsets() {
        let o = |v| Addresses::for_version(v).offset;
        assert_eq!(o(Version::V1_0), 0);
        assert_eq!(o(Version::V1_01), 0x2680);
        assert_eq!(o(Version::V2_00), 0x2680);
        assert_eq!(o(Version::V3_00Steam), 0x75130);
        assert_eq!(o(Version::V1_01Steam), 0x75770);
        assert_eq!(o(Version::Steam), 0x77970);
    }

    #[test]
    fn fixed_addresses() {
        let a = Addresses::for_version(Version::V1_0);
        assert_eq!(
            (a.playing_time, a.thread, a.loading, a.player_ped),
            (0x68ECF0, 0x68B42C, 0x7A67A5, 0x77CD98)
        );
        let a = Addresses::for_version(Version::V1_01);
        assert_eq!(
            (a.playing_time, a.thread, a.loading, a.player_ped),
            (0x68ECF0 + 0x2680, 0x68B42C + 0x2680, 0x7A67A5 + 0x2680, 0x77CD98 + 0x2680)
        );
        let a = Addresses::for_version(Version::Steam);
        assert_eq!(
            (a.playing_time, a.thread, a.loading, a.player_ped),
            (0x706660, 0x702D98, 0x833995, 0x8100D0)
        );
    }

    #[test]
    fn globals_get_the_offset_on_steam_too() {
        let a = Addresses::for_version(Version::Steam);
        assert_eq!(a.global(0x64A060), 0x64A060 + 0x77970);
    }

    #[test]
    fn collectibles_use_steam_addresses() {
        let photos = &COLLECTIBLES[0];
        assert_eq!(Addresses::for_version(Version::V1_0).collectible(photos), 0x7791BC);
        assert_eq!(Addresses::for_version(Version::V1_01).collectible(photos), 0x7791BC + 0x2680);
        assert_eq!(Addresses::for_version(Version::Steam).collectible(photos), 0x80C3E4);
    }
}
