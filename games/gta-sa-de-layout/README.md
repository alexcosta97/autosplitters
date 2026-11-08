# GTA San Andreas DE auto splitter, main.scm layout variant (WebAssembly)

Port of the GTA San Andreas Definitive Edition ASL auto splitter to the
LiveSplit auto splitting runtime, so it works in LiveSplit One and with games
running under Proton. See [Credits](#credits) for where it comes from.

This is a copy of [`gta-sa-de`](../gta-sa-de) that finds the script's globals
the way the updated DE ASL does, so the two approaches can be tested side by
side on the same game (issue #19). After testing, one approach is kept and
the other goes. Settings, setting keys and defaults are the same as in
`gta-sa-de`, so one splits file works with both.

## Download

Get `gta-sa-de-layout.wasm` from the latest `gta-sa-de-layout-v*`
[release](https://github.com/alexcosta97/autosplitters/releases).

## Build

From the repository root:

```sh
cargo build --release -p gta-sa-de-layout-autosplitter
```

Output: `target/wasm32-unknown-unknown/release/gta_sa_de_layout_autosplitter.wasm`

The settings logic has tests that run on the host:

```sh
cargo test-host -p gta-sa-de-layout-autosplitter
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

Log messages are prefixed with `[GTASA:DE Layout Autosplitter]`.

## Differences from gta-sa-de

`gta-sa-de` reads the script's globals from a hardcoded base, `0x51BEAE4`,
and shifts some of them by +6 on 1.0.113.21181. This one finds the base and
the shift from `main.scm`, as the updated DE ASL does. Everything else is the
same. What changes:

- The script base comes from the signature scan on every version. On
  1.0.113.21181, `0x51BEAE0` is also tried if the scan finds nothing. Each
  candidate must start with main.scm's header (`02 00 01`, then the size in
  bytes of the header and globals). Until one does, "Waiting for main.scm" is
  logged once, script globals aren't read and nothing splits. Start and reset
  don't depend on it. Once found, the base is kept while the process runs.
- The header size picks the layout, which shifts the global indexes: 49492
  has no shift, and 49536 (1.0.113.21181) shifts by +1 from index 450, +5 from
  4825, +6 from 8729 and +7 from 9180. An unknown size uses the closest known
  layout and logs `Unknown main.scm layout: <size> (using <key>, some splits
  may not work)`.
- `gta-sa-de`'s +6 shift for some globals is gone. The globals use the ASL's
  indexes instead, which the layout shifts: Kickstart 91, Shooting Range
  Complete 6690, Driving School Passed 8832, Trucking 9581, the gym moves
  9575, 9576 and 9580, Quarry 9593 and Freight 9669 and 9670.
- Quarry 7 splits on the flag `$2902` going from 0 to above 0, because the
  Quarry counter goes back to 0 in the same frame it reaches 7. Quarry "All
  done" is done when that flag is set. The setting stays where it was, last in
  the Quarry section.
- Starting the `finalec` thread splits "End of the Line Part 2", for when the
  chase starts without the counter changing.
- The end of any% splits when `eotl` (9436) goes from 2 to 3, without checking
  that End of the Line Part 2 was passed.
- The tags pattern is the updated ASL's (`89 ??` where `gta-sa-de` has
  `89 2d`). On 1.0.113.21181, `0x572B41C` is used if the scan fails.
- Log lines are prefixed `[GTASA:DE Layout Autosplitter]`.

Driving School is the one index where this and `gta-sa-de` disagree.
`gta-sa-de` reads `$86`, which is index 87 under this layout. This one keeps
the ASL's 8832, which on 1.0.113.21181 reads a slot nobody has tested. If
Driving School doesn't split here, suspect that index before the layout.

### Comparing the two in game

Load each `.wasm` in turn on 1.0.113.21181 and check the log. Expected here:

```
[GTASA:DE Layout Autosplitter] Script base: 0x51BEAE0
[GTASA:DE Layout Autosplitter] main.scm layout: 49536
```

The base is `0x51BEAE0` whether it was scanned or came from the fallback. If
the scan finds nothing, `Script base fallback: 0x51BEAE0` is logged (the scan
is then not the source of the base), and `Tags fallback: 0x572B41C` if the tags
scan fails. Then compare mission splits, Kickstart, Driving School, Shooting
Range, Quarry 7 and the end of any%. Quarry 7 (and Quarry "All done") and the
end of any% are expected to work here and not in `gta-sa-de` on
1.0.113.21181, because its base is 4 bytes later than this one's.

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
- The ASL's unused `startOnSaveLoad` setting now does something: "Start timer
  on loading a save" starts the timer once a save has finished loading, and
  "Reset timer on loading a save" (`resetOnSaveLoad`, new) resets it, starting
  it again if the start setting is on. They're for practising part of a run
  from a save, so both are off by default: reloading a save during a run would
  otherwise reset it. Saves with under 5 s of play time don't count, so a New
  Game doesn't trigger them.
- "GT #1" and "GT #2" split on the territories owned instead of on starting
  Grove 4 Life and Cut Throat Business. The percentage is a script global
  ($713) that the game refreshes every 20 s, so they can come up to 20 s after
  the territory is taken, and it is logged whenever it changes. At this point
  each territory is 1/53 (3 territories read 5.66%).
  - They only count once the territories side quest is enabled again on the
    way back to Los Santos, whichever missions were done to get there. The
    script does that when it passes the third Mansion mission, and has no flag
    of its own for it, so the auto splitter looks at the Mansion missions
    passed ($628) being at least 3, which the script raises in the same step.
    Nothing else about the route is checked.
  - GT #1: the territories owned reach 13%, which is 7 territories (the 3
    owned after the side quest is enabled and 4 more). 7 read 13.21% and 6
    read 11.32%. The global is a whole number, and reads 13 and 11 whether the
    game truncates or rounds.
  - GT #2: the territories owned reach 35%, which takes 19 territories (18
    read 33 or 34). That is what Sweet's phone call needs to unlock "End of the
    Line": the script only makes it once more than 34% are owned.
- If the executable version can't be read, pattern scanning is still attempted
  instead of disabling the auto splitter. Only the fallback addresses need the
  version.

## Fixes over the ASL

Found while testing `gta-sa-de` on 1.0.113.21181 under Proton 11, and kept
here:

- The thread signature doesn't match this version. Only the start/reset and
  mission-counter addresses are required now; thread, stats and tags are
  optional, so a failed scan only disables the splits that rely on it.
- The scan range is rounded up to whole pages. asr's `Signature::scan_iter`
  panics when a scan reaches the end of a range that ends partway through a
  page (SizeOfImage is `0x5CA4400`), which crashed the auto splitter before it
  published its settings.
- Loads are tracked on every tick, not only while the timer runs. Previously
  the New Game load ended before the timer started, so the 500 ms guard didn't
  apply and "Shooting Range Complete", which the game initialises to a
  non-zero value, split immediately.
- A New Game starts/resets the timer only once until play time goes backwards
  again. The start flag flips 1 -> 0 a second time during the intro, still
  under the 5 s play time limit, which reset and restarted the timer.

## Credits

This auto splitter is a port of the GTA San Andreas Definitive Edition ASL
auto splitter. Thanks to everyone who worked on it:

- **tduva and contributors**: the original GTA San Andreas auto splitter,
  including its settings, mission tables and split logic.
- **hoxi**: adapting it to the Definitive Edition.
- **Parik**: the patterns for signature scanning and the global variable
  indexes.

The main.scm layout detection (header size, shifts by index range, the Quarry
7 flag, `finalec` and the any% condition) comes from the updated DE ASL.

The global variable indexes were checked against the leaked DE main script
([JuniorDjjr's gist](https://gist.github.com/JuniorDjjr/2129e1e7640f7969acdfb1c56c263155)).

The notes in the source that name ASL variables (`$SWEET_TOTAL_PASSED_MISSIONS`
and so on) come from their work.
