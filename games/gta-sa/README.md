# GTA San Andreas auto splitter (WebAssembly)

Port of the GTA San Andreas (original PC release) ASL auto splitter to the
LiveSplit auto splitting runtime, so it works in LiveSplit One and with games
running under Proton. It behaves like the ASL; the
[differences](#differences-from-the-asl) are about how settings and timing
work in the runtime. See [Credits](#credits) for where it comes from.

## Download

Get `gta-sa.wasm` from the latest `gta-sa-v*`
[release](https://github.com/alexcosta97/autosplitters/releases).

## Build

From the repository root:

```sh
cargo build --release -p gta-sa-autosplitter
```

Output: `target/wasm32-unknown-unknown/release/gta_sa_autosplitter.wasm`

The settings and split logic have tests that run on the host:

```sh
cargo test-host -p gta-sa-autosplitter
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
3. Load the `.wasm` file as the auto splitter, then launch the game as usual.
   The runtime looks for the `gta_sa.exe` process, then `gta-sa.exe`.

Log messages are prefixed with `[GTASA Autosplitter]`.

## Versions

Detected, as in the ASL: 1.0, 1.01, 2.00, 3.00 Steam, 1.01 Steam and Steam
(the newer Steam release). The version is recognised from values in the game's
memory and from the module size. An unrecognised build disables the auto
splitter, as in the ASL, and logs its module size and the version values it
read.

Tested: not yet tested in-game; 1.0 under Proton and the current Steam release
will be tested before release.

## Differences from the ASL

- Settings are a flat list, since the runtime can't nest settings, grouped by
  headings instead. Each section (its key is the ASL's parent key) has a
  "Splits in this section" dropdown with None, All and Selection, which the
  auto splitter keeps in sync with its checkboxes: All ticks every checkbox in
  the section, None unticks them, and Selection shows when only some are
  ticked.
- There's no gating: every split is decided by its own checkbox. The ASL's
  parent switches (`Missions`, `Missions2`, `LS`, `Races`, `Export Lists`,
  `Other`, `Collectibles`, ...) no longer apply as saved values. Every other
  key is the ASL's, so saved Windows values for those still apply.
- The defaults are the ASL's effective defaults: everything under Races,
  Import/Export, Properties, Other and Collectibles starts off, as it did with
  the ASL's parents off.
- LiveSplit's Start and Reset checkboxes are replaced by the `start` and
  `reset` settings. `start` gates every start: New Game, and loading a save
  (with `startOnSaveLoad`). There's no equivalent of LiveSplit's Split
  checkbox; untick the splits instead.
- Start, reset and split run in the same order as LiveSplit's ASL runner:
  reset or split only while the timer is running or paused, and start after a
  reset in the same tick.
- The 500 ms load guard and 2.5 s double-split guard are counted in ticks (30
  per second, `refreshRate = 30`), because there's no clock without WASI.
- The list of splits already done is also cleared when the auto splitter
  resets the timer on New Game. The ASL only cleared it on seeing the timer
  not running, which never happens when a reset is followed by a start in the
  same tick, so splits from the previous attempt wouldn't fire again.
- The version is detected from the PE header's image size (falling back to the
  module size), the ASL's `ModuleMemorySize`, plus the same version values. An
  unknown version disables the auto splitter, as in the ASL, and logs its
  module size and version values.

## Suspected ASL issues to check in-game

Kept as the ASL has them, not fixed, until they are checked in-game:

- "Dirt Track" reads `0xA4BDB8`, where `0x64BDB8` would follow "8-Track"
  (`0x64BDB4`).
- On the Steam version, the ASL's comment says the `loading` address couldn't
  be found for Steam (an address is defined since). It affects the load guard
  and "Start timer when loading save (experimental)" there.

## Credits

This auto splitter is a port of the GTA San Andreas ASL auto splitter
([`GTASA.asl`](https://github.com/tduva/LiveSplit-ASL/blob/master/GTASA.asl)).
Thanks to everyone who worked on it:

- **tduva and contributors**: the original auto splitter, including its
  settings, mission tables and split logic.

The source repository has no license.
