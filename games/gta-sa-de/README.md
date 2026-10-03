# GTA San Andreas DE auto splitter (WebAssembly)

Port of the GTA San Andreas Definitive Edition ASL auto splitter to the
LiveSplit auto splitting runtime, so it works in LiveSplit One and with games
running under Proton. See [Credits](#credits) for where it comes from.

## Download

Get `gta-sa-de.wasm` from the latest `gta-sa-de-v*`
[release](https://github.com/alexcosta97/autosplitters/releases).

## Build

From the repository root:

```sh
cargo build --release -p gta-sa-de-autosplitter
```

Output: `target/wasm32-unknown-unknown/release/gta_sa_de_autosplitter.wasm`

The settings logic has tests that run on the host:

```sh
cargo test-host -p gta-sa-de-autosplitter
```

## Use on Linux with Proton

1. Allow the timer to read the game's memory. Arch ships with
   `kernel.yama.ptrace_scope = 1`, which only lets a process read its own
   children. Pick one:
   - grant the timer binary the capability:
     `sudo setcap cap_sys_ptrace=eip /path/to/livesplit-one`
   - or relax it system-wide until reboot:
     `sudo sysctl kernel.yama.ptrace_scope=0`
2. Use a native desktop timer. The web version of LiveSplit One can't attach to
   processes.
3. Load the `.wasm` file as the auto splitter, then launch the game through
   Steam as usual. The runtime looks for the `SanAndreas.exe` process.

Log messages are prefixed with `[GTASA:DE Autosplitter]`.

## Differences from the ASL

- Settings are a flat list, since the runtime can't nest, hide or show
  settings conditionally, grouped by headings instead: General, Story
  missions, Asset missions, Schools and vehicle missions, Races and stadium,
  Challenges and gyms, Import/Export, Properties, Collectibles and Other.
  Under those, each section (Los Santos, Heist, Races, Los Santos Races,
  Import/Export List 1, Properties, ...) is a heading followed by a "Splits in
  this section" dropdown. Los Santos, San Fierro and Las Venturas are split up
  by mission giver, Return to Los Santos has its ending splits under "Ending",
  and Properties is split up into businesses, safehouses by area and hotel
  suites.
- Every split is decided by its own checkbox alone; there are no switches
  that disable the settings below them. A section's dropdown ticks every
  checkbox in it (All) or unticks them (None), sub-sections included, and
  shows Selection when only some are ticked. The auto splitter keeps it in
  sync: ticking or unticking a checkbox updates the dropdowns of its sections,
  and on startup every dropdown is corrected to match its checkboxes.
- Some sections also offer "All done" in their dropdown. It unticks every
  checkbox in the section and splits once instead, on the tick the section's
  last item is done (the ASL never split on a section being done). Ticking a
  checkbox in it switches the dropdown back to Selection or All. A section
  holding sub-sections on "All done" shows Selection even with none of its
  own checkboxes ticked. Offered by:
  - Heist, Zero, Wang Cars, Trucking and Quarry: done when the strand's last
    mission is passed.
  - Couriers and Valet, Schools, Vehicle Submissions (including Freight),
    Los Santos, San Fierro, Las Venturas and Air Races, Stadium Events, Gym
    Moves and Properties: done when all their completion flags are set.
    Not counted: "Driving School Started" (a split on starting the mission),
    and "Wang Cars (Showroom Bought)" and "Verdant Meadows (Safehouse)" (only
    seen when their mission starts; no global keeps them).
  - Challenges: done when the three challenges are and the Chiliad Challenge
    is (its "done" flag, rather than the "Chiliad Challenge #n" splits).
  - Import/Export List 1 to 3: replaces the ASL's "Export List n Complete"
    checkbox, splitting when the list's last vehicle is exported. Splitting on
    each vehicle and on the list being complete is no longer possible for the
    same list, since All done unticks the vehicles. Lists 2 and 3 default to
    All done, as their "Complete" setting was on and their vehicles off.
- Each collectible (Photos, Tags, ...) is one dropdown: Off, Each, All done,
  or Each and all done, instead of separate "(All Done)" and "(Each)"
  checkboxes.
- Setting keys are the ASL's wherever a setting is still a checkbox, so saved
  Windows values for those still apply. Saved values of the old section
  switches (LS, Missions, Races, Export Lists, ...), of the collectible
  checkboxes (`PhotosAll`, `PhotosEach`, ...) and of "Export List n Complete"
  no longer apply.
- LiveSplit's Start/Reset checkboxes are replaced by the `start` and `reset`
  settings.
- Defaults are for a 100% run: the ASL's defaults, overridden by the values
  from the Windows LiveSplit settings in the 100% splits file
  (`DEFAULTS_100` in `src/settings/layout.rs`), with GT #1 and GT #2 on.
  Settings saved in a splits file still take precedence.
- The 500 ms load guard and 2.5 s double-split guard are counted in ticks
  (20 per second), because there's no clock without WASI.
- The trucking global variable index uses the version-specific value from
  `init` (9587 on 1.0.113.21181, 9581 otherwise). The ASL always read 9581,
  because the missions dictionary was built before `init` changed the offset.
- The gym moves globals get the same +6 shift on 1.0.113.21181 (LS 9581,
  SF 9582, LV 9586 instead of 9575, 9576, 9580). SF was confirmed by watching
  $9582 change when learning the San Fierro gym moves; LS and LV follow the
  same spacing (`data::VERSION_SHIFTED`).
- Quarry (`$9593`) and the Freight levels (`$9669`, `$9670`) get the same +6
  shift on 1.0.113.21181. They're declared after the gym and Trucking globals
  in the leaked DE script source, so they move with them.
- The ASL's unused `startOnSaveLoad` setting now does something: "Start timer
  on loading a save" starts the timer once a save has finished loading, and
  "Reset timer on loading a save" (`resetOnSaveLoad`, new) resets it, starting
  it again if the start setting is on. They're for practising part of a run
  from a save, so both are off by default: reloading a save during a run would
  otherwise reset it. Saves with under 5 s of play time don't count, so a New
  Game doesn't trigger them.
- If the executable version can't be read, pattern scanning is still attempted
  instead of disabling the auto splitter.

## Fixes over the ASL

Found while testing on 1.0.113.21181 under Proton 11:

- The thread signature doesn't match this version. Only the start/reset and
  mission-counter addresses are required now; thread, stats and tags are
  optional, so a failed scan only disables the splits that rely on it.
- The scan range is rounded up to whole pages. asr's `Signature::scan_iter`
  panics when a scan reaches the end of a range that ends partway through a
  page (SizeOfImage is `0x5CA4400`), which crashed the auto splitter before it
  published its settings.
- Loads are tracked on every tick, not only while the timer runs. Previously
  the New Game load ended before the timer started, so the 500 ms guard didn't
  apply and `$6690` ("Shooting Range Complete"), which the game initialises to
  a non-zero value, split immediately.
- A New Game starts/resets the timer only once until play time goes backwards
  again. The start flag flips 1 -> 0 a second time during the intro, still
  under the 5 s play time limit, which reset and restarted the timer.
- "Shooting Range Complete" reads `$6694` instead of `$6690`. In DE, `$6690`
  is `car_gen_hover[5]`, a car generator handle set on New Game, so it could
  never work as a completion flag. `$6694` is `player_has_fast_reload`, which
  the Ammu-Nation challenge sets to 1 the first time its final round is
  passed. Found by lining up the leaked DE script source's declarations with
  the live globals (source slot + 37 in this area).
- "Kickstart" reads `$90` instead of `$91`, and "Driving School Passed" reads
  `$86` instead of `$8832`. In the leaked DE script source these are
  `flag_kickstart_passed_1stime` and `driving_test_passed`, which Driving
  School sets to 1 when it's passed. `$91` is `f1_the90_best_score`, the
  score of the seventh driving test, so Kickstart split halfway through
  Driving School, and `$8832` didn't change when the school was passed.
- Races from Dirtbike Danger on read `$3721` + their race index, like the Los
  Santos races. The ASL skipped the indexes of Badlands A and B (7 and 8), so
  it read every San Fierro, Las Venturas, air and stadium race two indexes
  early: winning Dirtbike Danger split "Go-Go Karting", and the first two San
  Fierro races read the Badlands races.

## Credits

This auto splitter is a port of the GTA San Andreas Definitive Edition ASL
auto splitter. Thanks to everyone who worked on it:

- **tduva and contributors**: the original GTA San Andreas auto splitter,
  including its settings, mission tables and split logic.
- **hoxi**: adapting it to the Definitive Edition.
- **Parik**: the patterns for signature scanning and the global variable
  indexes.

The notes in the source that name ASL variables (`$SWEET_TOTAL_PASSED_MISSIONS`
and so on) come from their work.
