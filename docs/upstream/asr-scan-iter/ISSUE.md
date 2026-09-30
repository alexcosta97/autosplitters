# `Signature::scan_iter` panics when the range doesn't end on a page boundary

> Draft, not submitted. Target: https://github.com/LiveSplit/asr/issues

## Summary

`Signature::scan_iter` (and so `scan_process_range`) panics with a
`copy_from_slice` length mismatch when the scanned range ends partway through a
4 KiB page and the scan reaches that last partial chunk, i.e. whenever the
signature isn't found earlier. In a release auto splitter the panic is a wasm
trap, so the auto splitter dies on that tick. If it happens on the first tick,
it dies before registering its settings.

A range that *starts* partway through a page doesn't panic but silently misses
matches that cross the first page boundary.

This is easy to hit under Wine / Proton: `pe::read_size_of_image` is the
recommended way to get a module's size there, and `SizeOfImage` isn't
necessarily a multiple of 0x1000.

## Minimal repro

With the crate's mock host (`src/runtime/mock.rs`):

```rust
static SIG: Signature<4> = Signature::new("DE AD ?? EF");

let bytes = std::vec![0u8; 0x1400]; // 1 full page + 0x400 bytes, no match
with_process(&[(0x10000, &bytes)], |process| {
    SIG.scan_iter(process, (Address::new(0x10000), 0x1400)).count()
});
```

```
panicked at src/signature.rs:295:23:
copy_from_slice: source slice length (3075) does not match destination slice length (3)
```

A match at offset 0xFFE (crossing into a 0x100 byte first chunk when scanning
`(0x10F00, 0x1100)`) isn't found at all.

## Observed in a real auto splitter

GTA San Andreas Definitive Edition 1.0.113.21181 under Proton 11 on Arch Linux,
scanning `SanAndreas.exe` with `(base, pe::read_size_of_image(..))` where
`SizeOfImage = 0x5CA4400`, with a 27 byte signature that isn't present in that
game version:

```
panicked at .../asr/src/signature.rs:295:23:
copy_from_slice: source slice length (3098) does not match destination slice length (26)
```

(3098 = 0x1000 - 0x400 + 26.)

## Root cause

`src/signature.rs`, `scan_iter`, at LiveSplit/asr@89d55ab07198da6fd75cab1ea1a6825b4240b343:

```rust
let end = ((addr.value() & !((4 << 10) - 1)) + (4 << 10)).min(overall_end);
let len = end.saturating_sub(addr.value()) as usize;

// If we have read the previous memory page successfully, then we can copy the last
// elements to the start of the buffer.
if last_page_success {
    let (start, end) = buffer.split_at_mut(N.saturating_sub(1));
    start.copy_from_slice(&end[len.saturating_sub(N).saturating_add(1)..]);
}
```

The last `N - 1` bytes of the **previous** chunk are indexed with `len`, the
length of the chunk **about to be read**. That's only right when both are full
pages:

- Short last chunk (range end not page aligned): the source slice is
  `0x1000 - len + N - 1` bytes long instead of `N - 1`, so `copy_from_slice`
  panics.
- Short first chunk (range start not page aligned): the lengths happen to match
  but the bytes copied are from past the end of what was read (stale / zeroed),
  so matches across the first page boundary are missed.
- A first chunk shorter than `N - 1` bytes also panics.

The address adjustment afterwards (`address.add_signed(-(N - 1))`) assumes
exactly `N - 1` bytes were carried, which is also only true for full pages.

## Workaround

Round the range size up (or down) to a multiple of 0x1000 and start at a page
aligned address, e.g. `size.next_multiple_of(0x1000)` for a module that's mapped
in whole pages.

## Environment

- asr: `main` @ 89d55ab07198da6fd75cab1ea1a6825b4240b343 (2026-09-19), `signature` feature
- Target: `wasm32-unknown-unknown`, stable Rust 1.98.1
- Runtime: livesplit-auto-splitting (wasmtime 45)
- Game: GTA San Andreas DE 1.0.113.21181, Proton 11, module base 0x140000000,
  `SizeOfImage` 0x5CA4400
