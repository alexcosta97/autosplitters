# shellcheck shell=bash
# Shared by the release scripts. Source it; it defines functions only.

# The last full release of a game, e.g. gta-sa-de-v0.2.0, or nothing.
last_full_tag() { # game
  git describe --tags --abbrev=0 --match "$1-v[0-9]*.[0-9]*.[0-9]*" --exclude '*-rc.*' 2>/dev/null || true
}

# git-cliff limited to one game's folder and its full release tags.
cliff() { # game, git-cliff arguments...
  local game=$1
  shift
  git cliff --tag-pattern "^${game}-v[0-9]+\\.[0-9]+\\.[0-9]+\$" --include-path "games/${game}/**" "$@" 2>/dev/null
}
