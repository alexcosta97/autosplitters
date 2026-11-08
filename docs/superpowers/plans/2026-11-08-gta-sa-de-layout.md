# gta-sa-de-layout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A second DE auto splitter, `games/gta-sa-de-layout`, identical to `games/gta-sa-de` except that globals are found through main.scm's layout (from the updated ASL), plus the ASL's Quarry 7, `finalec` and any% changes.

**Architecture:** Copy `games/gta-sa-de`, then add a host-testable `main_scm` module (header parsing, layout table, index shifts, base candidates) and route every global read through it. The data tables go back to the ASL's indexes; `VERSION_SHIFTED`, the hardcoded script base and `globals_shift` go.

**Tech Stack:** Rust, `no_std` + `alloc`, `asr` (git), `gta-sa-common`, wasm32.

**Spec:** `docs/superpowers/specs/2026-11-08-gta-sa-de-layout-design.md`

## Global Constraints

- Crate `games/gta-sa-de-layout`, package `gta-sa-de-layout-autosplitter`, `crate-type = ["cdylib"]`, process `SanAndreas.exe`.
- Log prefix `[GTASA:DE Layout Autosplitter]` (debug output and `settings::register`).
- Settings widgets byte-identical to `gta-sa-de`: `src/settings/widgets.snap` and `old_defaults.tsv` copied unchanged and their tests pass unchanged.
- `games/gta-sa-de` is not modified.
- Layouts: `49492 → []`, `49536 → [(450, 1), (4825, 5), (8729, 6), (9180, 7)]`.
- Fallbacks only on version `1.0.113.21181`: script base `0x51BEAE0`, tags `0x572B41C`.
- Checks (CONTRIBUTING.md): `cargo fmt --check`, `cargo clippy --release --locked -- -D warnings`, `cargo clippy --tests --locked --target x86_64-unknown-linux-gnu -- -D warnings`, `cargo test-host --locked`, `cargo build --release --locked`.
- Commits: Conventional, scope `gta-sa-de-layout`, signed, ending with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

- main.scm not loaded yet (title menu, bytes are zero or garbage): nothing splits and no header is accepted; start/reset still work. Test in Task 2 (`header_rejects_*`) and Task 4 (`should_split` returns false before a layout is known).
- Header found at the scanned base and the fallback both: the scanned one wins. Test in Task 2 (`first_candidate_with_a_header_wins`).
- An unknown main.scm size between or beyond the known ones: closest layout used, never a panic. Test in Task 2 (`unknown_size_uses_closest`).
- Highest global read (`MAX_GLOBAL` 9670 + 7) must be inside the buffer. Test in Task 2 (`buffer_covers_the_highest_shifted_index`).
- Quarry "All done" on a save loaded with Quarry already done: no split. Test in Task 3 (`quarry_all_done_on_the_flag`).

---

### Task 1: Copy the crate

**Files:**
- Create: `games/gta-sa-de-layout/` (copy of `games/gta-sa-de/` without `target/`)
- Modify: `games/gta-sa-de-layout/Cargo.toml` (package name), `src/lib.rs` (log prefix, crate doc line), `src/settings.rs` (prefix), `Cargo.lock` (via build)

- [ ] **Step 1:** `cp -r games/gta-sa-de games/gta-sa-de-layout`; set `name = "gta-sa-de-layout-autosplitter"`; replace the `[GTASA:DE Autosplitter]` prefix with `[GTASA:DE Layout Autosplitter]` in `lib.rs::debug` and `settings::register`. Leave `README.md` for Task 5.
- [ ] **Step 2:** Run `cargo test-host --locked -p gta-sa-de-layout-autosplitter` (after `cargo build -p gta-sa-de-layout-autosplitter` updates `Cargo.lock`). Expected: same tests pass as `gta-sa-de`.
- [ ] **Step 3:** Commit the spec and this plan first (`docs(gta-sa-de-layout): add the design and plan`), then `feat(gta-sa-de-layout): copy gta-sa-de as a starting point` (include `Cargo.lock`).

### Task 2: `main_scm` module

**Files:**
- Create: `games/gta-sa-de-layout/src/main_scm.rs` (no `asr`), `mod main_scm;` in `lib.rs`

**Interfaces:**
- Produces:
  - `pub const HEADER_LEN: usize = 7;`
  - `pub fn parse_header(bytes: &[u8]) -> Option<u32>`: `Some(size)` iff `bytes.len() >= 7`, `bytes[0..3] == [0x02, 0x00, 0x01]` and the LE `i32` at 3..7 is > 0.
  - `pub struct Layout { pub size: u32, pub shifts: &'static [(u32, u32)] }` and `pub static LAYOUTS: &[Layout]` (values from Global Constraints, comments naming the versions as in the ASL).
  - `pub fn layout_for(size: u32) -> (&'static Layout, bool)`: closest by absolute difference (first wins a tie), `bool` = exact.
  - `impl Layout { pub fn slot(&self, index: u32) -> usize }`: `index + shift`, shift of the last pair with first index `<= index`, else 0.
  - `pub const MAX_SHIFT: u32 = 7;` with a test that it equals the largest shift in `LAYOUTS`.
  - `pub fn script_candidates(scanned: Option<u64>, version: &str) -> Vec<u64>`: scanned first (if any), then `0x51BEAE0` when `version == "1.0.113.21181"`.
  - `pub fn first_with_header(candidates: &[u64], read: impl FnMut(u64) -> Option<[u8; 7]>) -> Option<(u64, u32)>`: the first candidate whose bytes parse, with the size.

- [ ] **Step 1: Write the failing tests**
  - `header_parses_size`: `[2, 0, 1]` followed by `49536i32.to_le_bytes()` → `Some(49536)`, the same with `49492` → `Some(49492)`; extra trailing bytes are ignored.
  - `header_rejects_wrong_opcode_short_and_zero`: `[0;7]`, `[2,0,2,..]`, 6 bytes, size 0 or negative → `None`.
  - `known_sizes_are_exact`: `layout_for(49492) == (.., true)` with no shifts; `layout_for(49536)` exact.
  - `unknown_size_uses_closest`: `layout_for(49500).0.size == 49492`, `layout_for(49530).0.size == 49536`, `layout_for(0).0.size == 49492`, `layout_for(u32::MAX).0.size == 49536`, all `false`.
  - `shifts_at_the_boundaries` (49536): `slot(449) == 449`, `slot(450) == 451`, `slot(4824) == 4825`, `slot(4825) == 4830`, `slot(8728) == 8733`, `slot(8729) == 8735`, `slot(9179) == 9185`, `slot(9180) == 9187`.
  - `matches_gta_sa_de_on_1_0_113` (49536; the real slots `gta-sa-de` reads at its base + 1): `slot(454) == 455`, `slot(91) == 91`, `slot(6690) == 6695`, `slot(9576) == 9583`, `slot(9581) == 9588`, `slot(9436) == 9443`.
  - `first_candidate_with_a_header_wins`: both candidates valid → first; first invalid → second; none → `None`.
  - `candidates_add_the_fallback_only_on_1_0_113`: `(Some(1), "1.0.113.21181") → [1, 0x51BEAE0]`, `(None, "1.0.113.21181") → [0x51BEAE0]`, `(Some(1), "1.0.0.14296") → [1]`, `(None, "<unknown>") → []`.
  - `buffer_covers_the_highest_shifted_index`: for every layout, `slot(data::MAX_GLOBAL) <= (data::MAX_GLOBAL + MAX_SHIFT) as usize`.
- [ ] **Step 2:** `cargo test-host --locked -p gta-sa-de-layout-autosplitter main_scm` → fails to compile (module missing).
- [ ] **Step 3:** Implement the interfaces above.
- [ ] **Step 4:** Same command → PASS.
- [ ] **Step 5:** Commit `feat(gta-sa-de-layout): find globals through main.scm's layout`.

### Task 3: The ASL's indexes, Quarry 7, `finalec`

**Files:**
- Modify: `src/data.rs`, `src/completion.rs`, `src/settings/layout.rs` (Quarry section only)

**Interfaces:**
- Consumes: nothing new.
- Produces (`data.rs`): `QUARRY_DONE: u32 = 2902`; `MISSIONS2` group `"Quarry"` = `[(QUARRY_DONE, "Quarry 7")]`; Quarry counter `1..=6`; `START_MISSIONS` gains `("finalec", "End of the Line Part 2")` (after `manson5`'s position in the ASL, i.e. first here); Kickstart `91`, Shooting Range `6690`, Driving School `8832`, Freight `9670`/`9669`; `VERSION_SHIFTED`, `FREIGHT_1`/`FREIGHT_2` consts if unused, removed. `MAX_GLOBAL` stays `9670`. Comments: replace the "ASL used X" notes with the ASL's own comments; Driving School gets a note that `gta-sa-de` reads 86 (ASL 87 under this layout) and 8832 is untested on 1.0.113.21181.
- Produces (`completion.rs`): `Count::StrandFlag { strand: u32, group: &'static str }`: `items()` = the group's flags with `Rule::Set`; `names()` = the strand's names then the group's. `SECTIONS` uses `("QuarryMissions", Count::StrandFlag { strand: QUARRY, group: "Quarry" })`.
- `settings/layout.rs`: the Quarry section adds the `"Quarry"` group (`add_missions2(b, "Quarry", true)`) after `add_missions(b, QUARRY)`.

- [ ] **Step 1: Write/adjust the failing tests**
  - In `completion.rs` `strands_count_up_to_their_last_mission`: Quarry line becomes `matches!(items("QuarryMissions")[..], [(QUARRY_DONE, Rule::Set)])`.
  - `quarry_all_done_on_the_flag`: with Quarry's items, `(0, 1)` → done, `(1, 1)` → not (loaded save), `(0, 0)` → not.
  - In `data.rs` (new `#[cfg(test)]`): `quarry_7_is_a_flag` (counter has no 7; `missions2("Quarry") == [(2902, "Quarry 7")]`), `asl_indexes` (Kickstart 91, Shooting Range 6690, Driving School Passed 8832), `finalec_splits_part_2`.
  - Unchanged and must still pass: `widgets_match_snapshot`, `all_done_sections_count_their_checkboxes`, `every_old_key_resolves_to_its_old_default`.
- [ ] **Step 2:** `cargo test-host --locked -p gta-sa-de-layout-autosplitter` → the new/changed tests fail.
- [ ] **Step 3:** Make the changes above. `lib.rs` may need `QUARRY_DONE`-free compile fixes only; leave `globals_shift` removal to Task 4 if it's simpler, but the crate must compile.
- [ ] **Step 4:** Same command → PASS, snapshot unchanged (`git diff --stat games/gta-sa-de-layout/src/settings/widgets.snap` empty).
- [ ] **Step 5:** Commit `feat(gta-sa-de-layout): use the updated ASL's indexes, Quarry 7 flag and finalec`.

### Task 4: Read globals through the layout

**Files:**
- Modify: `games/gta-sa-de-layout/src/lib.rs`

**Interfaces:**
- Consumes: Task 2's `parse_header`, `HEADER_LEN`, `layout_for`, `Layout::slot`, `MAX_SHIFT`, `script_candidates`, `first_with_header`.

- [ ] **Step 1:** `Addresses`: drop `globals_shift`, the `hardcoded` branch and `global_index`; `script_candidates: Vec<u64>` from `script_candidates(scan, version)`; tags = scan with the updated pattern `"8b 3d ?? ?? ?? ?? ?? 8d 15 ?? ?? ?? ?? 8b 1d ?? ?? ?? ?? 66 ?? 89 ?? ?? ?? ?? ?? e8"`, else `0x572B41C` on `1.0.113.21181`. `resolve` requires start, start offset, loading and play time, and a non-empty candidate list. Log the candidates and say when a fallback is used (`"Script base fallback: 0x51BEAE0"`, `"Tags fallback: 0x572B41C"`).
- [ ] **Step 2:** `Memory` gains `script: Option<(u64, &'static Layout)>`. In `update`, while `None`: `first_with_header` over the candidates, reading `HEADER_LEN` bytes at `base + candidate`; on success log `Script base: 0x..`, then `main.scm layout: <size>` or `Unknown main.scm layout: <size> (using <key>, some splits may not work)`, and read the globals once so old == current (the ASL's `UpdateAll` after adding the watchers). Globals buffer length `(MAX_GLOBAL + MAX_SHIFT + 1)`, read from `base + script_base`. `global(index)` reads `slot(index)`.
- [ ] **Step 3:** `Splitter::should_split` returns `false` before anything else while `mem.script.is_none()`. Load tracking, triggers, start and reset stay as they are.
- [ ] **Step 4:** any%: `eotl` 2 → 3 calls `try_split("any%")` directly, with the ASL's new comment; remove `Splitter::passed` if now unused.
- [ ] **Step 5:** Run all Global Constraints checks. Expected: all pass, `target/wasm32-unknown-unknown/release/gta_sa_de_layout_autosplitter.wasm` built.
- [ ] **Step 6:** Commit `feat(gta-sa-de-layout): wait for main.scm and split any% on eotl alone`.

### Task 5: Docs

**Files:**
- Create/replace: `games/gta-sa-de-layout/README.md`
- Modify: `README.md` (Games table), `.github/ISSUE_TEMPLATE/3-bug-report.yml` (game list)

- [ ] **Step 1:** README, based on `gta-sa-de`'s: title "GTA San Andreas DE auto splitter, main.scm layout variant (WebAssembly)"; why it exists (side-by-side test, #19, one approach goes after); download name `gta-sa-de-layout.wasm`, build/test commands with the new package name, log prefix; "Differences from gta-sa-de" (the spec's "What changes" list, including the Driving School 8832 vs 87 note and what to look for in the log: base, `main.scm layout: 49536`); "Differences from the ASL" = `gta-sa-de`'s list minus the items about version shifts, hardcoded addresses and the index fixes this crate reverts; "Fixes over the ASL" keeps only those still true (scan-range rounding, load tracking every tick, New Game once, races already upstream → drop); Credits as in `gta-sa-de`, plus the updated ASL's layout detection.
- [ ] **Step 2:** Root README row: "Grand Theft Auto: San Andreas – The Definitive Edition (main.scm layout variant, for testing)", `games/gta-sa-de-layout`, `gta-sa-de-layout.wasm` from `gta-sa-de-layout-v*`. Bug template: `- gta-sa-de-layout (Grand Theft Auto San Andreas – The Definitive Edition, main.scm layout variant)`.
- [ ] **Step 3:** `cargo fmt --check` and `cargo test-host --locked` pass.
- [ ] **Step 4:** Commit `docs(gta-sa-de-layout): describe the variant and list it`.
