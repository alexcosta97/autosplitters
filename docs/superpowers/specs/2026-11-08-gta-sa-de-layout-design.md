# GTA San Andreas DE auto splitter, main.scm layout variant — design

Issue: [#19](https://github.com/alexcosta97/autosplitters/issues/19).
Source: the updated DE ASL, `LiveSplit.GTASA.Definitive.asl` (1,292 lines,
by tduva and contributors, hoxi and Parik, no license), compared with the
original DE ASL that `games/gta-sa-de` was ported from.

## Intent

`games/gta-sa-de` was ported from a local edit of the original DE ASL. That
edit hardcodes the script base `0x51BEAE4`, the tags address `0x572B41C` and
Trucking at 9587 for 1.0.113.21181, and later testing added a version-specific
+6 shift for some globals (`data::VERSION_SHIFTED`) and moved a few indexes
(Kickstart 90, Shooting Range 6694, Driving School 86).

The updated upstream ASL has no version checks. It scans for the script base,
reads the size of main.scm's globals from main.scm's first instruction, and
shifts every global index by range for the main.scm that is loaded:

```
49492 → no shift                                   (1.0, 1.01, 1.03, 1.04, 1.04.5)
49536 → +1 from 450, +5 from 4825, +6 from 8729, +7 from 9180   ("1.112")
```

Read against that table, `gta-sa-de`'s base looks 4 bytes late (the real one
would be `0x51BEAE0`). That accounts for its tested fixes (Kickstart,
Shooting Range, the +6) and predicts two bugs on 1.0.113.21181: the end of
any% reads slot 9437 instead of 9443, and Quarry 7 never splits.

The user wants to **test both side by side** on their game and then keep one
splitting and offset logic. So this adds a second crate that keeps every
feature of `gta-sa-de` and takes only its memory handling and split detection
from the updated ASL.

## Goals

- New crate `games/gta-sa-de-layout` (`gta-sa-de-layout-autosplitter`,
  `cdylib`), process `SanAndreas.exe`, built to its own `.wasm`. Release
  candidates follow automatically once merged (release tags
  `gta-sa-de-layout-v*`, which `gta-sa-de-v*` doesn't match).
- Start from a copy of `games/gta-sa-de` as it is on `main`. Everything not
  listed under "What changes" stays exactly as it is: the settings layout,
  setting keys and defaults, "All done", collectible dropdowns,
  territory-based `GT #1`/`GT #2`, save-load start/reset, New Game arming,
  cooldowns in ticks, the scan-range rounding, optional stats/thread/tags.
- The same settings widgets as `gta-sa-de`: its `widgets.snap` snapshot test
  must pass unchanged in the new crate, so one splits file works with both.
- Host tests for the new logic, which lives in modules without `asr`.
- A README with how it differs from `gta-sa-de` and from the updated ASL,
  how to tell the two apart in logs, and credits; the root README's Games
  table and the bug report template's list updated.

## What changes, from the updated ASL

1. **Script base and layout.**
   - The script base comes from the existing `SCRIPT_BASE_SIG` scan on every
     version. No hardcoded base, no `VERSION_SHIFTED`, no `globals_shift`.
   - Fallback (builder's call, made to keep the test meaningful): on
     1.0.113.21181 `0x51BEAE0` is also a candidate. Each candidate is checked
     for main.scm's header; the first that has it is used, and which one is
     logged.
   - The header is the first 7 bytes at the base: `02 00 01` followed by a
     little-endian `i32`, the size of the header and globals in bytes. Until
     a candidate has it, script globals aren't read and nothing splits (the
     ASL's `scriptReady`). The start and reset don't depend on it. Once found
     it's kept for as long as the process runs, as in the ASL.
   - Layouts: the table above. An unknown size uses the closest known one,
     as the ASL does. Log `main.scm layout: <size>` for a known one and
     `Unknown main.scm layout: <size> (using <key>, some splits may not
     work)` otherwise.
   - A global index `i` reads the 4 bytes at `base + (i + shift(i)) * 4`,
     where `shift(i)` is the shift of the last pair whose first index is
     `<= i` (0 if none).
2. **The ASL's indexes**, which the layout now shifts: Kickstart 91,
   Shooting Range 6690, Driving School Passed 8832, Trucking 9581, gyms 9575,
   9576 and 9580, Quarry 9593, Freight 9669 and 9670. The race indexes from
   Dirtbike Danger on (3730 to 3747) are the same in both already.
   - Driving School is the one index where the two disagree, and the user
     chose to keep the ASL's 8832. On 1.0.113.21181 it reads the real slot
     8838, which nobody has tested. `gta-sa-de`'s working 86 (`driving_test_
     passed`) is ASL index 87 under this layout. The README says so, so that
     a Driving School miss in this crate isn't blamed on the layout.
3. **Quarry 7** splits on flag `$2902` going from 0 to above 0, as a
   single-flag split (a `MISSIONS2` group "Quarry" holding only it), since
   the counter goes back to 0 in the same frame it reaches 7. The Quarry
   counter keeps 1 to 6. Quarry "All done" is done when `$2902` is set.
   The setting stays where it is (last in the Quarry section, key
   `Quarry 7`, default on), added explicitly after the counter's missions
   so the settings snapshot is unchanged.
4. **`finalec`**: starting the `finalec` thread splits
   `End of the Line Part 2`, for when the chase is started without the
   counter changing. Same split-once rule as the other thread splits.
5. **any%** splits when `eotl` (9436) goes from 2 to 3, without checking
   that End of the Line Part 2 was passed.
6. **Tags pattern**: the updated ASL's, with `89 ??` where `gta-sa-de` has
   `89 2d`. Fallback (builder's call): `0x572B41C` on 1.0.113.21181 if the
   scan fails.
7. **Log prefix** `[GTASA:DE Layout Autosplitter]`, so the two can be told
   apart in logs.

## Non-goals

- Changing `games/gta-sa-de`, including the bugs above. The comparison
  should be between the two approaches as they are.
- The cleanup that follows the test (removing the losing approach), which
  is a follow-up once the user has picked one.
- Sharing more code with `gta-sa-de` through `crates/gta-sa-common`. The
  copy is temporary, and one of the two goes in the cleanup.
- Upstream's GT #1 / GT #2 on thread starts (`grove2`, `manson5`): the WASM
  features stay.

## Anti-goals

- Settings that differ from `gta-sa-de` in keys, order or defaults. That
  would make side-by-side testing need two splits files.
- Splitting on globals read before main.scm is loaded (zeroes or old data
  from the menu).
- A silent fallback: whichever base and layout are used must be in the log.

## Constraints

- `no_std` + `alloc`, `asr` from git as in `gta-sa-de`, tick rate 20.
- The checks in `CONTRIBUTING.md`: fmt, both clippy runs, `cargo test-host
  --locked`, `cargo build --release --locked`.
- Conventional Commits, scope `gta-sa-de-layout`, signed.

## Left to the builder

- Module and type names, and how the header check fits in the main loop
  (for example, read the globals only once a layout is known).
- How many bytes to read for the globals: up to `MAX_GLOBAL` plus the
  largest shift.
- Tests beyond: header parsing (valid, wrong opcode, short), closest-layout
  choice, `shift` at and around each boundary, the Quarry 7 flag split and
  its "All done", `finalec`, and any% without the Part 2 check.

## How it will be tested

Host tests, then in game on 1.0.113.21181 under Proton, with each `.wasm`
loaded in turn: the log shows the base, the header size (49536 expected) and
the layout; then mission splits, Kickstart, Driving School, Shooting Range,
Quarry 7 and the end of any% are compared between the two.
