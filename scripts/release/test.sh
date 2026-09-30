#!/usr/bin/env bash
# Tests plan.sh and notes.sh in a throwaway repository.
set -euo pipefail

root=$(git rev-parse --show-toplevel)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"
git init -q -b main
git config user.name test
git config user.email test@example.com
git config commit.gpgSign false
git config tag.gpgSign false
mkdir -p scripts/release
cp "$root"/scripts/release/{lib,plan,notes}.sh scripts/release/
cp "$root/cliff.toml" .
git add -A
git commit -qm "chore: release scripts"

failures=0
commit() { # file, message
  mkdir -p "$(dirname "$1")"
  echo "$RANDOM" >>"$1"
  git add -A
  git commit -qm "$2"
}
tag() { git tag -a -m "$1" "$1"; }
planned() { # game -> "version rc_tag previous_rc_tag", or nothing
  scripts/release/plan.sh | jq -r --arg g "$1" '.[] | select(.game == $g) | "\(.version) \(.rc_tag) \(.previous_rc_tag)"'
}
expect() { # description, expected, actual
  if [[ "$2" == "$3" ]]; then
    echo "ok: $1"
  else
    echo "FAIL: $1: expected '$2', got '$3'"
    failures=$((failures + 1))
  fi
}

expect "no games, no releases" "[]" "$(scripts/release/plan.sh)"

commit games/a/src "Add a"
expect "a new game gets 0.1.0 without a conventional commit" "0.1.0 a-v0.1.0-rc.1 " "$(planned a)"

tag a-v0.1.0-rc.1
expect "a re-run reuses the candidate at HEAD" "0.1.0 a-v0.1.0-rc.1 " "$(planned a)"

commit games/a/src "chore: tidy"
expect "a new commit before approval makes the next candidate" "0.1.0 a-v0.1.0-rc.2 a-v0.1.0-rc.1" "$(planned a)"

tag a-v0.1.0-rc.2
tag a-v0.1.0
expect "nothing to release after the full release" "" "$(planned a)"

commit games/a/src "docs: explain"
expect "commits that aren't feat, fix or perf don't release" "" "$(planned a)"

commit games/b/src "feat: b thing"
expect "a commit in another game doesn't release this one" "" "$(planned a)"
expect "the other game gets its first release" "0.1.0 b-v0.1.0-rc.1 " "$(planned b)"

commit games/a/src "fix: bug"
expect "fix bumps patch" "0.1.1 a-v0.1.1-rc.1 " "$(planned a)"

commit games/a/src "feat(settings): option"
expect "feat bumps minor" "0.2.0 a-v0.2.0-rc.1 " "$(planned a)"

commit games/a/src "feat!: new keys"
expect "breaking bumps minor before 1.0.0" "0.2.0 a-v0.2.0-rc.1 " "$(planned a)"

commit games/c/src "Add c"
tag c-v1.2.3
commit games/c/src "fix!: breaking fix"
expect "breaking bumps major from 1.0.0" "2.0.0 c-v2.0.0-rc.1 " "$(planned c)"

commit games/c-d/src "Add c-d"
tag c-d-v5.0.0
commit games/c-d/src "feat: c-d thing"
expect "a game's tags don't count for a game named after its prefix" "2.0.0 c-v2.0.0-rc.1 " "$(planned c)"
expect "and the other way round" "5.1.0 c-d-v5.1.0-rc.1 " "$(planned c-d)"

notes=$(scripts/release/notes.sh a full)
expect "full notes start from the last full release" "## Changes since a-v0.1.0" "$(head -n1 <<<"$notes")"
if grep -q -- "- Bug" <<<"$notes" && grep -q -- "- New keys" <<<"$notes" && ! grep -q "b thing" <<<"$notes"; then
  expect "full notes list this game's changes only" ok ok
else
  expect "full notes list this game's changes only" ok "$notes"
fi

git tag -a -m rc a-v0.2.0-rc.1 HEAD~4
notes=$(scripts/release/notes.sh a rc a-v0.2.0-rc.1)
expect "candidate notes start with what's new since the previous candidate" "## New since a-v0.2.0-rc.1" "$(head -n1 <<<"$notes")"

if ((failures > 0)); then
  echo "$failures failed"
  exit 1
fi
echo "all passed"
