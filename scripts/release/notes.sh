#!/usr/bin/env bash
# Release notes for one game at HEAD, on stdout.
#   notes.sh <game> rc [previous_rc_tag]  notes for a release candidate
#   notes.sh <game> full                  notes for a full release
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
# shellcheck source=scripts/release/lib.sh
source scripts/release/lib.sh

game=${1:?usage: notes.sh <game> rc [previous_rc_tag] | notes.sh <game> full}
last=$(last_full_tag "$game")
since=${last:-the first release}

section() { # heading, git-cliff arguments...
  local heading=$1 body
  shift
  body=$(cliff "$game" "$@" --strip all)
  printf '## %s\n\n' "$heading"
  if [[ -n "${body//[[:space:]]/}" ]]; then
    printf '%s\n\n' "$body"
  else
    printf 'No user-facing changes.\n\n'
  fi
}

case "${2:-}" in
  rc)
    if [[ -n "${3:-}" ]]; then
      section "New since $3" "$3..HEAD"
    fi
    section "All changes since $since" --unreleased
    ;;
  full)
    section "Changes since $since" --unreleased
    ;;
  *)
    echo "usage: notes.sh <game> rc [previous_rc_tag] | notes.sh <game> full" >&2
    exit 2
    ;;
esac
