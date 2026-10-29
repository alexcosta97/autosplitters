# GTA San Andreas (original PC release) auto splitter — design

Issue: [#5](https://github.com/alexcosta97/autosplitters/issues/5).
Source: [`GTASA.asl`](https://github.com/tduva/LiveSplit-ASL/blob/master/GTASA.asl)
by tduva and contributors (1,304 lines, no license).

## Intent

Runners of the original PC release use tduva's ASL in Windows LiveSplit. A
WASM port lets them use LiveSplit One, livesplit-asr-bridge and Proton, as
`games/gta-sa-de` already does for the Definitive Edition.

This first pass is a **faithful port**: it starts, splits and resets the way
the ASL does today, so the community can adopt it without first agreeing to
any behaviour change. The improvements gta-sa-de made on top of its ASL go in
a separate follow-up issue (possibly a separate variant), not here.

## Goals

- New crate `games/gta-sa` (`gta-sa-autosplitter`, `cdylib`), process names
  `gta_sa.exe` and `gta-sa.exe` (the ASL's `state("gta_sa")` and
  `state("gta-sa")`).
- Every ASL behaviour ported: version detection by the version values and
  module size, the per-version offset and the separate "Steam" addresses,
  mission counters (`vars.missions`), single-flag missions (`vars.missions2`),
  collectibles (All Done / Each), busted/death warps, any% end, thread-start
  splits (including GT #1 on `grove2` and GT #2 on `manson5`), Chiliad
  Challenge, Import/Export (each vehicle and "Export List n Complete"),
  start on New Game, reset on New Game, `startOnSaveLoad`, the 500 ms
  load guard and the 2.5 s `doubleSplitPrevention` cooldown, split-once-per-
  attempt, and the "disabled when the version is unknown" behaviour.
- The same setting keys as the ASL, which are also gta-sa-de's where the
  splits are the same.
- Code shared with gta-sa-de through a new crate, in this same PR.
- Host tests (`cargo test-host`) for the logic that doesn't need `asr`.
- A README for the game, the root README's Games table, and the bug report
  template's list updated (CONTRIBUTING.md, "Adding a game").

## Non-goals (deferred to a follow-up issue, which this PR's work opens)

- "All done" in the section dropdowns.
- Collectible dropdowns (Off / Each / All done / Each and all done).
- GT #1 / GT #2 on territories owned.
- gta-sa-de's reworked save-load start/reset (`resetOnSaveLoad`, the 5 s
  play-time rule, "only once until play time goes back").
- gta-sa-de's 100% defaults (`DEFAULTS_100`), Quarry 1–7 mission splits, and
  any other gta-sa-de-only addition.
- gta-sa-de's split up of big sections by mission giver and Properties by
  area is a **layout** choice, not a behaviour change, and may be reused
  (builder's call).

## Anti-goals

- **Do not fix suspected ASL bugs.** Anything that looks wrong in the ASL
  (example: "Dirt Track" reads `0xA4BDB8`, where its neighbours suggest
  `0x64BDB8`) stays exactly as the ASL has it. Each one goes on a
  "Suspected ASL issues to check in-game" list in the game's README and in
  the PR's testing notes, for manual in-game testing. Fixes come later,
  after testing, in their own change.
- Do not carry over gta-sa-de's DE-specific fixes (Kickstart `$90`, Shooting
  Range `$6694`, the race index gap, the thread-signature workaround). They
  are about DE's script and memory. The original ASL's addresses are taken as
  correct until testing says otherwise. (For example the race gap at indexes
  7 and 8 is present in the original ASL's addresses, so that DE bug doesn't
  exist here.)
- No behaviour change to gta-sa-de. The shared-crate refactor must keep its
  setting keys, defaults, splits and log output identical; its existing host
  tests must pass unchanged (moved with the code is fine).

## Constraints

- The auto splitting runtime can't nest settings, so the ASL's
  parent-gates-children rule can't be reproduced. The user decided: use
  gta-sa-de's layout — headings, and per-section "Splits in this section"
  dropdowns (None / All / Selection, **no "All done"**) kept in sync with
  their checkboxes both ways and corrected on startup. Every split is decided
  by its own checkbox. The ASL's parent keys (`LS`, `Missions`, `Races`,
  `Export Lists`, `Collectibles`, ...) no longer apply; the README says so.
- Collectibles keep the ASL's two checkboxes per type (`PhotosAll`,
  `PhotosEach`, ...).
- Defaults are the ASL's own defaults, as the ASL computes them for each
  leaf checkbox. Where an ASL parent defaulted off (Races, Export Lists,
  Other, Collectibles), the children's effective default in the ASL was
  "off", so the port defaults them off and their dropdown shows None.
- LiveSplit's Start/Reset checkboxes become `start` and `reset` settings, as
  in gta-sa-de (the runtime has no equivalent). Both default on.
- No clock without WASI: time guards are counted in ticks, as gta-sa-de does
  (500 ms and 2.5 s at its tick rate). The ASL's `refreshRate = 30`; the
  builder picks a tick rate and counts the guards to match the same
  durations.
- `asr` scan/read caveats found in gta-sa-de apply (scan ranges rounded to
  whole pages, if any scanning is used). The ASL uses fixed addresses, no
  signatures, so none is expected.
- Log messages prefixed `[GTASA Autosplitter]`, the ASL's prefix.
- CONTRIBUTING.md: Conventional Commits, signed commits, branch
  `feat/gta-sa`, the five checks, PR from the template. PR title
  `feat(gta-sa): add GTA San Andreas auto splitter`.
- Releases: a squash-merged `feat(gta-sa)` commit that touches
  `games/gta-sa-de/` publishes a gta-sa-de minor release. The user accepted
  that; the PR description says the gta-sa-de part is a refactor with no
  behaviour change. Note: `scripts/release` only looks at `games/<game>/**`,
  so a later change to the shared crate alone wouldn't release either game
  (a follow-up issue, see below).

## Decided by the user

- Port the ASL as is; improvements are a separate issue, maybe a separate
  variant.
- Suspected bugs: report for manual testing, don't fix.
- Settings: gta-sa-de's dropdowns, since the ASL's parent check can't be
  matched.
- Shared code: a new crate in this PR, accepting the extra gta-sa-de
  release.
- Reset: the split-once list is cleared when the auto splitter resets the
  timer on New Game, as gta-sa-de does. The ASL only cleared it on seeing the
  timer not running, which an automatic reset followed by a start in the same
  cycle never shows, so splits from the previous attempt wouldn't fire again.
  This is the one ASL bug fixed in this pass; the README lists it as a
  difference.
- Release scripts: not changed here; a follow-up issue.
- Testing: a downgraded 1.0 under Proton first, then the current Steam
  release. 1.01, 2.00, 3.00 Steam and 1.01 Steam are ported but untested,
  and the README says so.

## Shared crate (shape left to the builder, within these lines)

A new workspace member outside `games/`, for example
`crates/gta-sa-common` (add it to `[workspace] members`). `no_std` + `alloc`
like the games. Candidates:

- The settings dropdown machinery (`games/gta-sa-de/src/settings/tree.rs`
  and the runtime glue in `settings.rs`), with "All done" support staying
  available to gta-sa-de but unused by gta-sa.
- Mission names and their values per counter, export lists, collectible
  names, thread-start names, the story mission order lists — the parts both
  ASLs share. Global indexes / addresses stay per game: the two scripts
  number their globals differently (original `$INTRO_TOTAL_PASSED_MISSIONS`
  is at `0x64A060` = global 448 from `0x649960`; DE uses 450), and gta-sa-de's
  tables have since diverged (Quarry 1–7, no GT thread splits, Quarry
  Complete moved). The builder decides whether tables are shared as names
  with per-game indexes, or stay per game where they diverge.
- Split-rule logic that both games run the same way (split once per attempt
  plus cooldown, `Passed`, warps, Chiliad, export detection), if it can be
  shared without changing gta-sa-de's behaviour. Where gta-sa-de differs from
  the ASL, gta-sa keeps the ASL's behaviour.

`asr`-free logic goes in modules that host tests can run.

## gta-sa memory model (from the ASL)

- Version detection in `init`, in the ASL's order (later matches override
  earlier ones): version value `38079` at the six version addresses, and the
  main module size (18313216 → 1.0, 34471936 → 1.01, 17985536 → 2.00,
  9691136 → 3.00 Steam, 9981952 → Steam; 1.01 Steam by version value only).
  Offsets: 1.0 0, 1.01 0x2680, 2.00 0x2680, 3.00 Steam 0x75130, 1.01 Steam
  0x75770, Steam 0x77970. The offset applies to every global (missions,
  `vars.missions2`, eotl, chiliad, export) on every version, Steam included.
  Only on Steam do play time, start flag, thread, loading, player ped and the
  collectibles use separate fixed addresses instead. Unknown version →
  disabled (no start, split or reset).
- If the current Steam release isn't one of the builds the ASL recognises,
  the port stays disabled on it like the ASL would (no new detection here).
  Log the module size and the version values read, so testing can tell, and
  record it in the README and the follow-up issue. *(Builder's call, made by
  the coordinating session.)*
- All addresses are module-relative. Use the main module's base and size
  (the ASL's `modules.First()`).
- Watched values: every `vars.missions` key and `vars.missions2` address
  (+offset), `eotl`, `intro_state`, `chiliadRace`, `chiliadDone`, `loading`
  (bool), `pedStatus` (`[playerPed] + 0x530`), `playingTime`, `started`
  (byte), `thread` (string at `[thread] + 0x8`, 10 bytes), collectibles,
  `exportList` and `export0..9`. Old/current pairs per tick.
- Keep the ASL's order of checks within `split`, since it returns on the
  first split (one split per tick).

## Testing

- Host tests for settings (keys, defaults, dropdown sync) and split rules.
- In game (the user, on 1.0 under Proton, then current Steam): a New Game
  starts the timer, and a few splits of each kind fire once: a mission
  counter, a single-flag mission, a thread-start split, a collectible, a
  warp if practical, export, and reset on New Game. Plus every item on the
  suspected issues list.
- Before handing the build over: all five checks pass, the work is
  committed and pushed to the PR.

## Left to the builder

Crate and module layout, how tables are expressed, tick rate, the exact
heading/section grouping (following gta-sa-de's), test design, commit split.

## Follow-up issues this work opens

- The gta-sa-de improvements listed under Non-goals, for gta-sa (possibly as
  a separate variant).
- `scripts/release` and the changelogs only look at `games/<game>/**`, so a
  change to the shared crate alone releases neither game. Fixing that is out
  of scope here (user's decision).
- Anything testing finds on the suspected issues list, and the current Steam
  build if the ASL doesn't recognise it.
