# Fix signature scans over ranges that aren't page aligned

> Draft, not submitted. Apply `fix.patch` (`git am fix.patch`) on top of
> LiveSplit/asr `main` @ 89d55ab.

Fixes #<issue number>.

`Signature::scan_iter` carries the last `N - 1` bytes of each chunk to the
front of the next one so matches across page boundaries are found, but it
indexed that tail with the length of the chunk about to be read instead of the
one just read. So:

- a range **ending** partway through a page panicked in `copy_from_slice` when
  the scan reached the short last chunk (whenever the signature isn't found
  earlier), which is common for `pe::read_size_of_image` ranges under
  Wine / Proton;
- a range **starting** partway through a page missed matches across the first
  page boundary;
- a first chunk shorter than `N - 1` bytes panicked too.

### Changes

- Remember the previous chunk's length and copy the carried tail from there,
  including bytes that were already carried in front of a chunk shorter than
  `N - 1`.
- Scan the carried bytes plus the current chunk, and adjust match addresses by
  the number of bytes actually carried rather than always `N - 1`.
- Add tests using the mock host: matches in and across full pages (unchanged
  behaviour), a range ending mid-page with and without a match across the
  boundary, a range starting mid-page, and a first chunk shorter than the
  signature.

The chunking, the syscall per page, and the buffer layout are unchanged.

### Testing

- `cargo test --all-features`: all pass. The 4 new edge case tests fail on
  `main` (3 panics, 1 missed match); the full-page test passes before and after.
- `cargo fmt --check` clean.
- End to end: a GTA San Andreas DE auto splitter scanning a 0x5CA4400 byte
  module under Proton 11 with a signature that isn't present. With `main` it
  panics on the first tick (`source slice length (3098) does not match
  destination slice length (26)`). With this patch it reports the signature as
  not found and every other signature resolves to the same address as with a
  page-rounded range.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
