#!/bin/bash -e

sedcmd="sed"
case "$(uname)" in
  Darwin) sedcmd="gsed";;
esac

CWD="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"

bump_type="$1"
if [ -z "$bump_type" ] ; then
  printf 'Version bump type required\n' 1>&2
  exit 1
fi

case "$bump_type" in
  patch|major|minor)
    version="$bump_type"
    ;;
  version)
    version="$2"
    if [ -z "$version" ] ; then
      printf 'Target version required\n' 1>&2
      exit 1
    fi
    ;;
  *)
    printf 'Version bump parameter must be one of: patch, minor, major, version\n' 1>&2
    exit 2
    ;;
esac

(
  cd "$CWD"
  NEXT_VERSION="$(poetry version -s "$version")"

  find .. -name Cargo.toml -exec  "$sedcmd" -i 's/^version = .*/version = "'"$NEXT_VERSION"'"/' {} \;
  cargo check

  cd ../frontend
  pnpm version --no-commit-hooks "$version"

  if [ -n "$GITHUB_ENV" ] ; then
    printf 'NEXT_VERSION=%s\n' "$NEXT_VERSION" >> "$GITHUB_ENV"
  fi
)
