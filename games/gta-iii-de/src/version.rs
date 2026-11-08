//! What depends on the executable: where the script's globals sit, and how to
//! read the loading flag from the code the pattern found.
//!
//! Doesn't touch `asr`, so it can be tested on the host.

/// Which of the globals the layout moves, see `Layout`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shift {
    Mission,
    Usj,
    Rampage,
    Package,
    TeTimer,
    TeHelipad,
}

/// Which `main.scm` the executable shipped with, found from its version. The
/// ASL got these from analysing `GTA3SaveInfo.dat`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// 1.0, 1.01, 1.03: the standard 1.0 offsets.
    One,
    /// 1.04, 1.04.5, Steam, Epic: 10 variables inserted at offset 10644 shift
    /// the helipad.
    Two,
    /// 1.112, 1.113 and later: `FLAG_LUIGIS_REACHED` inserted at offset 28
    /// shifts all missions and collectibles.
    Three,
}

impl Group {
    /// From the numeric file version's build and private parts.
    pub fn from_version(build: u16, private: u16) -> Self {
        if build >= 112 {
            Group::Three
        } else if build >= 8 || private >= 15284 {
            Group::Two
        } else {
            Group::One
        }
    }

    pub fn layout(self) -> Layout {
        match self {
            Group::One => Layout::default(),
            Group::Two => Layout { te_helipad: 10, ..Layout::default() },
            // +4, +4, +8, +8, +4 and +88 bytes in the ASL.
            Group::Three => {
                Layout { mission: 1, usj: 1, rampage: 2, package: 2, te_timer: 1, te_helipad: 22 }
            }
        }
    }

    /// For the log.
    pub fn description(self) -> &'static str {
        match self {
            Group::One => "Group 1 (standard 1.0 offsets)",
            Group::Two => "Group 2 (+40 helipad shift)",
            Group::Three => "Group 3 (+4 mission shift, etc.)",
        }
    }
}

/// How many globals later than in 1.0 each kind of global is. The ASL counts
/// the shifts in bytes, these are in globals (4 bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Layout {
    pub mission: u32,
    pub usj: u32,
    pub rampage: u32,
    pub package: u32,
    pub te_timer: u32,
    pub te_helipad: u32,
}

impl Layout {
    pub fn shift(&self, shift: Shift) -> u32 {
        match shift {
            Shift::Mission => self.mission,
            Shift::Usj => self.usj,
            Shift::Rampage => self.rampage,
            Shift::Package => self.package,
            Shift::TeTimer => self.te_timer,
            Shift::TeHelipad => self.te_helipad,
        }
    }
}

/// Where the displacement of the instruction writing the loading flag is,
/// from the start of the loading pattern's match. Which instruction it is
/// depends on the build, found by the opcode at offset 30. `None` for an
/// opcode that isn't known.
pub fn loading_disp_offset(opcode: u8) -> Option<u64> {
    match opcode {
        // `c6 05 [disp32] 00`, earlier builds.
        0xC6 => Some(32),
        // `40 88 3d [disp32]`, later Title Updates (1.112+).
        0x40 => Some(33),
        _ => None,
    }
}

/// Offset of the opcode `loading_disp_offset` looks at.
pub const LOADING_OPCODE_OFFSET: u64 = 30;

/// Offset of the end of the instruction, which the displacement is relative
/// to.
const LOADING_INSTRUCTION_END: u64 = 37;

/// Offset from the module base of the loading flag, given where the loading
/// pattern matched (also from the base) and the displacement read.
pub fn loading_rva(pattern_rva: u64, disp: i32) -> u64 {
    (pattern_rva + LOADING_INSTRUCTION_END).wrapping_add_signed(i64::from(disp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_boundaries() {
        assert_eq!(Group::from_version(0, 0), Group::One);
        assert_eq!(Group::from_version(7, 15283), Group::One);
        assert_eq!(Group::from_version(8, 0), Group::Two);
        assert_eq!(Group::from_version(7, 15284), Group::Two);
        assert_eq!(Group::from_version(111, 0), Group::Two);
        assert_eq!(Group::from_version(112, 0), Group::Three);
        // 1.0.113.21181
        assert_eq!(Group::from_version(113, 21181), Group::Three);
        // The private part doesn't matter once the build is 112.
        assert_eq!(Group::from_version(112, 15284), Group::Three);
    }

    #[test]
    fn group_layouts_count_globals() {
        assert_eq!(Group::One.layout(), Layout::default());
        assert_eq!(Group::Two.layout(), Layout { te_helipad: 10, ..Layout::default() });
        let three = Group::Three.layout();
        assert_eq!(three.shift(Shift::Mission), 1);
        assert_eq!(three.shift(Shift::Usj), 1);
        assert_eq!(three.shift(Shift::Rampage), 2);
        assert_eq!(three.shift(Shift::Package), 2);
        assert_eq!(three.shift(Shift::TeTimer), 1);
        assert_eq!(three.shift(Shift::TeHelipad), 22);
    }

    #[test]
    fn loading_opcodes() {
        assert_eq!(loading_disp_offset(0xC6), Some(32));
        assert_eq!(loading_disp_offset(0x40), Some(33));
        assert_eq!(loading_disp_offset(0x00), None);
        assert_eq!(loading_disp_offset(0x88), None);
        assert_eq!(LOADING_OPCODE_OFFSET, 30);
    }

    #[test]
    fn loading_flag_is_relative_to_the_instruction_end() {
        assert_eq!(loading_rva(0x1000, 0x200), 0x1000 + 37 + 0x200);
        assert_eq!(loading_rva(0x1000, -0x100), 0x1000 + 37 - 0x100);
    }
}
