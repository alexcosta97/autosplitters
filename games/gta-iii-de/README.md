# GTA III DE auto splitter (WebAssembly)

Port of the GTA III Definitive Edition ASL auto splitter to the LiveSplit auto
splitting runtime, so it works in LiveSplit One and with games running under
Proton. See [Credits](#credits) for where it comes from.

## Download

Get `gta-iii-de.wasm` from the latest `gta-iii-de-v*`
[release](https://github.com/alexcosta97/autosplitters/releases).

## Build

From the repository root:

```sh
cargo build --release -p gta-iii-de-autosplitter
```

Output: `target/wasm32-unknown-unknown/release/gta_iii_de_autosplitter.wasm`

The split rules and settings logic have tests that run on the host:

```sh
cargo test-host -p gta-iii-de-autosplitter
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
   Steam as usual. The runtime looks for the `LibertyCity.exe` process.

Log messages are prefixed with `[GTAIII:DE Autosplitter]`.

## Differences from the ASL

- Settings are a flat list, since the runtime can't nest, hide or show
  settings conditionally, grouped by headings instead: General, Missions
  (end), Missions (start), Collectibles and Other. Under Missions, each
  mission giver is a heading followed by a "Splits in this section" dropdown.
- Every split is decided by its own checkbox alone; there are no switches
  that disable the settings below them. A section's dropdown ticks every
  checkbox in it (All) or unticks them (None), and shows Selection when only
  some are ticked. The auto splitter keeps it in sync: ticking or unticking a
  checkbox updates the dropdown of its section, and on startup every dropdown
  is corrected to match its checkboxes.
- Each mission giver under Missions (end) also offers "All done" in its
  dropdown. It unticks every mission checkbox in the section and splits once
  instead, on the tick the giver's last mission is passed (the ASL never split
  on a giver being done). It is checked before the individual mission splits.
- Each collectible (Hidden Packages, Unique Stunt Jumps, Rampages) is one
  dropdown: Off, Each, All done, or Each and all done, instead of separate
  "(All Done)" and "(Each)" checkboxes.
- Setting keys are the ASL's wherever a setting is still a checkbox (the
  mission end and start checkboxes and the Any% final split), so saved Windows
  values for those still apply. Saved values of the collectible checkboxes
  (`Hidden PackagesAll`, `Hidden PackagesEach`, ...) no longer apply, as the
  dropdown replaces them.
- LiveSplit's Start/Reset checkboxes are replaced by the `start` and `reset`
  settings, both on by default.
- The 500 ms load guard is counted in ticks (20 per second), because there's
  no clock without WASI. Loads are tracked on every tick, not only while the
  timer runs, so a load that ends just before the timer starts still starts the
  guard (as in gta-sa-de). The ASL only did it while splitting.
- If the executable version can't be read, the auto splitter logs it and uses
  the layout of Group 1 (the standard 1.0 offsets), which is what the ASL ends
  up with. The group in use is logged.
- The scan range is the size from SizeOfImage in the PE header instead of the
  ASL's `ModuleMemorySize`, because Wine doesn't always report the module size
  correctly, rounded up to whole pages.
  asr's `Signature::scan_iter` panics when a scan reaches the end of a range
  that ends partway through a page, i.e. whenever a signature isn't found.
- The ASL's defaults are kept: mission end checkboxes, the Any% final split,
  `start` and `reset` are on; mission start checkboxes and collectibles are
  off. All four signature scans are required, as in the ASL; if one fails, the
  log says which.
- If a scan fails, it tries again every second instead of staying off until
  the game restarts, so attaching before the game has finished starting up
  still works.

## Credits

This auto splitter is a port of the GTA III Definitive Edition ASL auto
splitter, [`LiveSplit.GTA3.Definitive.asl`](https://github.com/hoXyy/Livesplit.Autosplitters/blob/5c26dad5cf33a9e22974814f70de9be26a15ab73/LiveSplit.GTA3.Definitive.asl).
Thanks to everyone who worked on it:

- **hoxi**: the Definitive Edition auto splitter, its pattern scans and layout
  groups.
- **illuminati7777**: the initial version.
- The authors of the original GTA III and Vice City auto splitters it was
  based on.
