<!--
Title: use a Conventional Commit, for example `fix(gta-sa-de): split on Riot`.
Pull requests are squash-merged and the title becomes the commit on `main`,
so it decides the next version of the games it touches. See CONTRIBUTING.md.
-->

## Issue

<!-- Link the issue that describes this work. Every pull request needs one;
open an issue first if none exists. -->

Closes #

## What changed and why

## How it was tested

<!-- Host tests, and for split logic: which game version and timer you ran
it with, and what split. -->

## Checklist

- [ ] The pull request links the issue it resolves.
- [ ] The title follows Conventional Commits, scoped to the game it changes.
- [ ] Every commit is signed and follows Conventional Commits.
- [ ] `cargo fmt --check`, both `cargo clippy` runs, `cargo test-host --locked`
      and `cargo build --release --locked` pass locally (see CONTRIBUTING.md).
- [ ] Changes users notice are in the game's README.
- [ ] Code ported from another auto splitter is credited in the game's README.
