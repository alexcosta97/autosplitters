# AGENTS.md

This repository holds LiveSplit auto splitters for the auto splitting runtime,
one Rust crate per game under `games/`. This file only says where to find
things; the information itself lives in the places below.

## Where to find what

| You need | Look in |
|---|---|
| What the repository is for, its layout, building | `README.md` |
| How a game's auto splitter works, what it changed from any original, credits | `games/<game>/README.md` |
| The work to do, its scope and acceptance criteria | GitHub issues, `alexcosta97/autosplitters` |
| Work in progress | Open pull requests |
| Conventions: commits, branches, pull requests, checks, releases | `CONTRIBUTING.md` |
| What a pull request must contain | `.github/pull_request_template.md` |
| Release history | GitHub Releases, tagged per game |

## Starting a session

1. Read `CONTRIBUTING.md`. Its conventions apply to every change.
2. Check open issues and pull requests with `gh`, to see what is in progress
   and what is next.
3. Work from an issue. Read it and the README of the game it concerns before
   changing code.

## How work is run

The main session coordinates. It holds the full context, gives subagents
self-contained tasks (each with the issue, the game's README, the files
involved and how to verify the result), and reviews their output before
anything is committed or merged. Subagents do not decide what is correct; the
coordinating session does.

Never push to `main`. Once the checks in `CONTRIBUTING.md` pass, open a pull
request for the work, filled in from `.github/pull_request_template.md`,
without waiting to be asked. Before handing a build over for testing, commit
the changes and push them to the pull request, so the build being tested is
what the pull request contains. If anything is left out of the pull request,
say so plainly when handing the build over.
