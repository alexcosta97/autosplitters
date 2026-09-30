#!/usr/bin/env bash
# Works out which games need a release at HEAD, from the commits that touched
# games/<game> since that game's last full release, and prints them as a JSON
# array of {game, version, rc_tag, previous_rc_tag}, also written to
# $GITHUB_OUTPUT as `games` when set. Run with all tags fetched.
#
# The largest change wins: a breaking change bumps major (minor before 1.0.0),
# otherwise a feat bumps minor, otherwise a fix or perf bumps patch. A game
# that was never released gets 0.1.0 as soon as it has any commit.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
# shellcheck source=scripts/release/lib.sh
source scripts/release/lib.sh

plan='[]'
for dir in games/*/; do
  game=$(basename "$dir")
  last=$(last_full_tag "$game")
  version=""
  if [[ -z "$last" ]]; then
    if [[ -n "$(git log --oneline -1 -- "games/$game")" ]]; then
      version=0.1.0
    fi
  else
    IFS=. read -r major minor patch <<<"${last##*-v}"
    groups=$(cliff "$game" --unreleased --context | jq -r '[.[].commits[] | .group] | .[]')
    if grep -q 'Breaking changes' <<<"$groups"; then
      if ((major == 0)); then version="0.$((minor + 1)).0"; else version="$((major + 1)).0.0"; fi
    elif grep -q 'Features' <<<"$groups"; then
      version="$major.$((minor + 1)).0"
    elif grep -qE 'Bug fixes|Performance' <<<"$groups"; then
      version="$major.$minor.$((patch + 1))"
    fi
  fi
  [[ -z "$version" ]] && continue

  prefix="$game-v$version-rc."
  numbers=$(git tag --list "${prefix}*" | sed "s/^${prefix//./\\.}//" | sort -n)
  existing=$(git tag --points-at HEAD --list "${prefix}*" | sed "s/^${prefix//./\\.}//" | sort -n | tail -n1)
  if [[ -n "$existing" ]]; then
    # A re-run for a commit that already has a candidate reuses it.
    number=$existing
  else
    number=$(($(tail -n1 <<<"${numbers:-0}") + 1))
  fi
  previous=$(awk -v n="$number" '$1 < n' <<<"$numbers" | tail -n1)
  plan=$(jq -c --arg game "$game" --arg version "$version" --arg rc "$prefix$number" \
    --arg prev "${previous:+$prefix$previous}" \
    '. + [{game: $game, version: $version, rc_tag: $rc, previous_rc_tag: $prev}]' <<<"$plan")
done

echo "$plan"
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "games=$plan" >>"$GITHUB_OUTPUT"
fi
