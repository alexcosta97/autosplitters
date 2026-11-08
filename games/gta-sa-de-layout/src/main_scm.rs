//! The layout of `main.scm` in memory, which decides where each global lives.
//!
//! The script base points at a 7 byte header (`02 00 01`, then the size in
//! bytes of the header and the globals as a little-endian i32). Game updates
//! insert globals, so the same global index sits at a different offset
//! depending on that size. The header size picks a known layout, and each
//! layout says how far to shift a global index.

use alloc::vec::Vec;

/// Bytes read at the script base: the opcode `02 00 01` and the i32 size.
pub const HEADER_LEN: usize = 7;

/// The script base on game version 1.0.113.21181, used when the signature
/// scan finds nothing.
const FALLBACK_SCRIPT_BASE: u64 = 0x51BEAE0;

/// The largest shift of any layout, to size the read buffer.
pub const MAX_SHIFT: u32 = 7;

/// One known layout of `main.scm`.
pub struct Layout {
    /// Size in bytes of the header and the globals.
    pub size: u32,
    /// `(first global index, shift)` pairs in ascending index order.
    pub shifts: &'static [(u32, u32)],
}

impl Layout {
    /// The slot a global index sits in: the index plus the shift of the last
    /// pair whose first index is at or below it, or none before the first.
    pub fn slot(&self, index: u32) -> usize {
        let shift = self
            .shifts
            .iter()
            .take_while(|&&(first, _)| first <= index)
            .last()
            .map_or(0, |&(_, shift)| shift);
        (index + shift) as usize
    }
}

/// The known layouts.
pub static LAYOUTS: &[Layout] = &[
    // 1.0, 1.01, 1.03, 1.04, 1.04.5.
    Layout { size: 49492, shifts: &[] },
    // "1.112", game version 1.0.113.21181.
    Layout { size: 49536, shifts: &[(450, 1), (4825, 5), (8729, 6), (9180, 7)] },
];

/// The size from a script header, or `None` if the bytes are not one.
pub fn parse_header(bytes: &[u8]) -> Option<u32> {
    if bytes.len() < HEADER_LEN || bytes[0..3] != [0x02, 0x00, 0x01] {
        return None;
    }
    let size = i32::from_le_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]);
    u32::try_from(size).ok().filter(|&size| size > 0)
}

/// The layout closest in size to `size` (the first wins a tie), and whether
/// it matches exactly.
pub fn layout_for(size: u32) -> (&'static Layout, bool) {
    let mut best = &LAYOUTS[0];
    for layout in &LAYOUTS[1..] {
        if layout.size.abs_diff(size) < best.size.abs_diff(size) {
            best = layout;
        }
    }
    (best, best.size == size)
}

/// The script base addresses to try: the scanned one, then the fixed one on
/// the version it is known for.
pub fn script_candidates(scanned: Option<u64>, version: &str) -> Vec<u64> {
    let mut candidates: Vec<u64> = scanned.into_iter().collect();
    if version == "1.0.113.21181" {
        candidates.push(FALLBACK_SCRIPT_BASE);
    }
    candidates
}

/// The first candidate whose header bytes parse, with the size they give.
/// `read` returns the header bytes at an address, or `None` if unreadable.
pub fn first_with_header(
    candidates: &[u64],
    mut read: impl FnMut(u64) -> Option<[u8; HEADER_LEN]>,
) -> Option<(u64, u32)> {
    candidates.iter().find_map(|&address| parse_header(&read(address)?).map(|size| (address, size)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;

    fn header(size: i32) -> [u8; 7] {
        let mut bytes = [2, 0, 1, 0, 0, 0, 0];
        bytes[3..].copy_from_slice(&size.to_le_bytes());
        bytes
    }

    #[test]
    fn header_parses_size() {
        assert_eq!(parse_header(&header(49536)), Some(49536));
        assert_eq!(parse_header(&header(49492)), Some(49492));
        let mut longer = header(49536).to_vec();
        longer.extend_from_slice(&[9, 9, 9]);
        assert_eq!(parse_header(&longer), Some(49536));
    }

    #[test]
    fn header_rejects_wrong_opcode_short_and_zero() {
        assert_eq!(parse_header(&[0; 7]), None);
        let mut wrong = header(49536);
        wrong[2] = 2;
        assert_eq!(parse_header(&wrong), None);
        assert_eq!(parse_header(&header(49536)[..6]), None);
        assert_eq!(parse_header(&header(0)), None);
        assert_eq!(parse_header(&header(-1)), None);
    }

    #[test]
    fn known_sizes_are_exact() {
        let (layout, exact) = layout_for(49492);
        assert!(exact);
        assert_eq!(layout.size, 49492);
        assert!(layout.shifts.is_empty());
        let (layout, exact) = layout_for(49536);
        assert!(exact);
        assert_eq!(layout.size, 49536);
    }

    #[test]
    fn unknown_size_uses_closest() {
        for (size, expected) in [(49500, 49492), (49530, 49536), (0, 49492), (u32::MAX, 49536)] {
            let (layout, exact) = layout_for(size);
            assert_eq!(layout.size, expected, "size {size}");
            assert!(!exact, "size {size}");
        }
    }

    #[test]
    fn shifts_at_the_boundaries() {
        let (layout, _) = layout_for(49536);
        for (index, slot) in [
            (449, 449),
            (450, 451),
            (4824, 4825),
            (4825, 4830),
            (8728, 8733),
            (8729, 8735),
            (9179, 9185),
            (9180, 9187),
        ] {
            assert_eq!(layout.slot(index), slot, "index {index}");
        }
    }

    #[test]
    fn matches_gta_sa_de_on_1_0_113() {
        let (layout, _) = layout_for(49536);
        for (index, slot) in
            [(454, 455), (91, 91), (6690, 6695), (9576, 9583), (9581, 9588), (9436, 9443)]
        {
            assert_eq!(layout.slot(index), slot, "index {index}");
        }
    }

    #[test]
    fn first_candidate_with_a_header_wins() {
        let good = |address: u64| Some(header(address as i32));
        assert_eq!(first_with_header(&[100, 200], good), Some((100, 100)));
        let only_second = |address: u64| (address == 200).then(|| header(200));
        assert_eq!(first_with_header(&[100, 200], only_second), Some((200, 200)));
        assert_eq!(first_with_header(&[100, 200], |_| None), None);
        assert_eq!(first_with_header(&[], good), None);
    }

    #[test]
    fn candidates_add_the_fallback_only_on_1_0_113() {
        assert_eq!(script_candidates(Some(1), "1.0.113.21181"), [1, 0x51BEAE0]);
        assert_eq!(script_candidates(None, "1.0.113.21181"), [0x51BEAE0]);
        assert_eq!(script_candidates(Some(1), "1.0.0.14296"), [1]);
        assert!(script_candidates(None, "<unknown>").is_empty());
    }

    #[test]
    fn buffer_covers_the_highest_shifted_index() {
        for layout in LAYOUTS {
            assert!(layout.slot(data::MAX_GLOBAL) <= (data::MAX_GLOBAL + MAX_SHIFT) as usize);
        }
    }

    #[test]
    fn max_shift_is_the_largest_in_layouts() {
        let largest = LAYOUTS.iter().flat_map(|l| l.shifts.iter().map(|&(_, s)| s)).max();
        assert_eq!(largest, Some(MAX_SHIFT));
    }
}
