# Contributing to autosplitters

Thanks for your interest in contributing. This guide covers how to set up a
development environment and the conventions every change follows.

## Before you start

- All work is tracked in the
  [issues](https://github.com/alexcosta97/autosplitters/issues), and every
  pull request must link one, except Renovate's dependency updates. If there's
  no issue for what you want to do, open one first with the matching template:
  **Task** for a well-defined piece of work, **Feature request** for a new
  idea, or **Bug report** for an auto splitter that doesn't work.
- For anything beyond a small fix, comment on the issue before starting, so
  the approach can be agreed first.
- Each game's README describes how its auto splitter works and how it differs
  from any auto splitter it was ported from. Changes that alter that update
  the README in the same pull request.
- Code ported from, or based on, someone else's auto splitter is credited in
  the game's README.

## Development setup

1. Install [rustup](https://rustup.rs/). The toolchain, the
   `wasm32-unknown-unknown` target, clippy and rustfmt come from
   `rust-toolchain.toml`.
2. Clone the repository and build every game:

   ```sh
   cargo build --release
   ```

3. Before pushing, run the same checks CI runs:

   ```sh
   cargo fmt --check
   cargo clippy --release --locked -- -D warnings
   cargo clippy --tests --locked --target x86_64-unknown-linux-gnu -- -D warnings
   cargo test-host --locked
   cargo build --release --locked
   ```

   `cargo test-host` runs the tests on the host, since the default target is
   WebAssembly. Keep logic that doesn't need the runtime (settings, split
   rules) in modules that don't use `asr`, so it can be tested there.

4. For the release scripts and workflow linting, install the pinned tools with
   [mise](https://mise.jdx.dev/) (`mise install`, see `mise.toml`), then run
   `scripts/release/test.sh` and `actionlint`. The same checks run on pull
   requests that change release files.

## Commits

### Conventional Commits

Every commit message follows
[Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

| Type | Use for | Effect on the next version |
|---|---|---|
| `feat` | A new feature for users | Minor |
| `fix` | A bug fix for users | Patch |
| `perf` | A performance improvement | Patch |
| `refactor` | A code change that neither fixes a bug nor adds a feature | None |
| `docs` | Documentation only | None |
| `test` | Adding or changing tests | None |
| `build` | Build system or dependencies | None |
| `ci` | CI configuration and workflows | None |
| `style` | Formatting only, no code change | None |
| `chore` | Anything else that doesn't affect users | None |

- The description is in the imperative mood and lowercase, with no full stop:
  `fix(gta-sa-de): split on riot`, not `Fixed Riot split.`
- The scope names the game a change is for (`feat(gta-sa-de): …`). Changes
  that aren't about one game leave it out or name the area (`ci: …`).
- Which games are released is decided by the files a change touches, not by
  the scope: see [Releases](#releases).
- A **breaking change** is marked with `!` after the type or scope
  (`feat!: …`), or a `BREAKING CHANGE:` footer describing it. For an auto
  splitter, that includes setting keys that change or stop applying, so saved
  settings are lost. Until version 1.0.0, breaking changes bump the minor
  version.

### Signed commits

All commits must be signed, and `main` rejects unsigned ones. Signing with an
SSH key is the simplest option:

```sh
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true
```

Then add the same public key to your GitHub account as a **signing key**
(Settings → SSH and GPG keys → New SSH key → Key type: Signing Key). GitHub's
documentation on
[commit signature verification](https://docs.github.com/en/authentication/managing-commit-signature-verification)
covers GPG and other options.

## Branches

Name branches `<type>/<short-description>`, using the commit types above, for
example `feat/gta-sa-de-all-done` or `fix/gta-sa-de-riot-split`.

## Pull requests

- **Title:** a Conventional Commit, like a commit message. Pull requests are
  squash-merged, and the title becomes the commit on `main`, so it determines
  the next version of the games it touches.
- **Description:** fill in the pull request template. It asks for the issue
  the pull request resolves (`Closes #123`), what changed and why, how it was
  tested, and a short checklist.
- **Scope:** one logical change per pull request, and normally one game.
- **Requirements to merge:**
  - all CI checks pass: formatting, clippy, host tests, a build of every
    game, and the Conventional Commits check on the title and every commit;
  - all review conversations are resolved;
  - all commits are signed.
- **Merging:** only maintainers can merge into `main`, using squash merge.
  The branch is deleted after merging.

## Adding a game

1. Create `games/<game>/` with a crate named `<game>-autosplitter`
   (`crate-type = ["cdylib"]`). `<game>` is short, lowercase and hyphenated,
   like `gta-sa-de`: it becomes the release tag prefix and the download's
   name.
2. Give it a README: how to use it, how it differs from any auto splitter it
   was ported from, and credits.
3. Add it to the Games table in the README, and to the list in the bug report
   template.

A new game gets its first release candidate, 0.1.0, as soon as it is merged.

## Dependency updates

[Renovate](https://docs.renovatebot.com/) opens pull requests every week to
update dependencies, configured in `renovate.json`. Their titles follow the
conventions above, and the type decides whether the update is released:

- `fix(deps)`: crates compiled into the auto splitters, including the
  LiveSplit `asr` crate and `Cargo.lock` refreshes. These produce a release of
  every game whose folder they change.
- `ci(deps)`: GitHub Actions. No release.
- `chore(deps)`: tools pinned in `mise.toml` and development-only crates. No
  release.

Renovate's pull requests are the one exception to the linked-issue rule. They
go through the same checks and are merged by a maintainer like any other pull
request. The Dependency Dashboard issue lists pending updates.

## Releases

Releases are automated, and each game is released on its own. There is no
manual version bump and no `CHANGELOG.md`: the version and release notes come
from the commits. The `version` in each game's `Cargo.toml` isn't used.

- A game's version follows [Semantic Versioning](https://semver.org/) and is
  calculated from the commits that touched `games/<game>/` since its last full
  release. The largest change wins: a breaking change bumps major (minor
  before 1.0.0), otherwise a `feat` bumps minor, otherwise a `fix` or `perf`
  bumps patch. Commits of the other types alone don't produce a release. A
  game that was never released gets 0.1.0.
- Every merge to `main` that produces a version for a game publishes a
  **release candidate** for it as a GitHub pre-release, tagged
  `<game>-vX.Y.Z-rc.N`, with `<game>.wasm` attached.
- A maintainer promotes a release candidate by approving its pending release
  job. That publishes the full release `<game>-vX.Y.Z` from the same commit,
  with the candidate's `<game>.wasm`, marked **Latest**.
- Release notes list every change to the game since its previous full
  release, grouped by type. They are the game's changelog.

## License

By contributing, you agree that your contributions are dual licensed under
the MIT and Apache-2.0 licenses, as described in the [README](README.md#license),
without any additional terms or conditions.
