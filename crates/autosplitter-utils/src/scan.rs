//! Finding a module in a process and scanning it for signatures, like the
//! helpers of the same name in the ASLs.

use core::fmt;

use asr::{file_format::pe, signature::Signature, Address, Process};

/// The module's base address and size, once the module is mapped. The size is
/// not rounded.
pub fn module_range(process: &Process, name: &str) -> Option<(Address, u64)> {
    let base = process.get_module_address(name).ok()?;
    // Wine doesn't necessarily report the module size correctly, the PE
    // header does.
    let size = pe::read_size_of_image(process, base)
        .map(u64::from)
        .or_else(|| process.get_module_size(name).ok())?;
    Some((base, size))
}

/// The module's base address and size, to scan for signatures: `module_range`
/// with the size rounded up to whole pages.
pub fn scan_range(process: &Process, name: &str) -> Option<(Address, u64)> {
    let (base, size) = module_range(process, name)?;
    // Round up to whole pages. asr's `Signature::scan_iter` panics on a range
    // that ends partway through a page when the scan reaches the end, i.e.
    // whenever a signature isn't found. The module is mapped in whole pages,
    // so this is safe.
    Some((base, size.next_multiple_of(0x1000)))
}

/// `getAddressFromPattern`: resolves a RIP-relative operand to an offset from
/// the module base.
pub fn address_from_pattern<const N: usize>(
    process: &Process,
    module: (Address, u64),
    offset: u64,
    sig: &Signature<N>,
) -> Option<u64> {
    let ptr = sig.scan_process_range(process, module)? + offset;
    let disp: i32 = process.read(ptr).ok()?;
    Some((ptr.value() - module.0.value()).wrapping_add_signed(disp as i64 + 4))
}

/// `getValueFromPattern`: reads an immediate operand.
pub fn value_from_pattern<const N: usize>(
    process: &Process,
    module: (Address, u64),
    offset: u64,
    sig: &Signature<N>,
) -> Option<i32> {
    let ptr = sig.scan_process_range(process, module)? + offset;
    process.read(ptr).ok()
}

/// An executable's numeric file version, `major.minor.build.private`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub build: u16,
    pub private: u16,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.major, self.minor, self.build, self.private)
    }
}

/// The file version of the PE module mapped at `base`, `None` if it has no
/// version resource.
pub fn version(process: &Process, base: Address) -> Option<Version> {
    let v = pe::FileVersion::read(process, base)?;
    Some(Version {
        major: v.major_version,
        minor: v.minor_version,
        build: v.build_part,
        private: v.private_part,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn version_displays_as_four_dotted_parts() {
        let v = Version { major: 1, minor: 0, build: 113, private: 21181 };
        assert_eq!(v.to_string(), "1.0.113.21181");
    }
}
