# Auto splitters

Auto splitters for the [LiveSplit auto splitting
runtime](https://github.com/LiveSplit/livesplit-core/tree/master/crates/livesplit-auto-splitting),
written in Rust with the [`asr`](https://github.com/LiveSplit/asr) crate and
compiled to WebAssembly. They run in LiveSplit One, in LiveSplit through its
auto splitting runtime component, and in
[livesplit-asr-bridge](https://github.com/alexcosta97/livesplit-asr-bridge).

## Games

| Game | Folder | Download |
|---|---|---|
| Grand Theft Auto: San Andreas – The Definitive Edition | [`games/gta-sa-de`](games/gta-sa-de) | `gta-sa-de.wasm` from the latest `gta-sa-de-v*` [release](https://github.com/alexcosta97/autosplitters/releases) |

Each game's README explains how to use it and credits the people whose work it
builds on.

## Layout

```
games/<game>/          one crate per game, named <game>-autosplitter
  Cargo.toml           the game's dependencies (releases set the version)
  README.md            usage, differences from any original, credits
  src/
docs/                  notes that aren't about one game
scripts/release/       works out each game's next version and release notes
Cargo.toml             the workspace, and the release profile for every game
.cargo/config.toml     builds for wasm32 by default, with the runtime's features
rust-toolchain.toml    the toolchain and the wasm32 target
```

Everything shares one `Cargo.lock` and one `target/` folder. Build output is
never committed.

## Building

With [rustup](https://rustup.rs/) installed, the toolchain and the wasm32
target come from `rust-toolchain.toml`.

```sh
cargo build --release                           # every game
cargo build --release -p gta-sa-de-autosplitter # one game
```

The `.wasm` files are in `target/wasm32-unknown-unknown/release/`.

Logic that doesn't need the runtime is tested on the host:

```sh
cargo test-host                                 # every game
cargo test-host -p gta-sa-de-autosplitter       # one game
```

Before pushing, run the same checks as CI:

```sh
cargo fmt --check
cargo clippy --release --locked -- -D warnings
cargo clippy --tests --locked --target x86_64-unknown-linux-gnu -- -D warnings
cargo test-host --locked
cargo build --release --locked
```

## Contributing and releases

See [CONTRIBUTING.md](CONTRIBUTING.md) for the conventions, adding a game, and
how releases work: each game is released on its own, as a release candidate
first and a full release once a maintainer approves it.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
