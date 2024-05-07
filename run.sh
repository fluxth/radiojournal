#!/bin/bash

set -e
set -o allexport

CWD="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"

. "$CWD/.env.dev"

if [[ "$1" == "logger" ]]; then
  (
    cd "$CWD/logger"
    cargo lambda watch -p 9001
  )
elif [[ "$1" == "logger-invoke" ]]; then
  cargo lambda invoke -p 9001 --data-ascii '{}' radiojournal-logger | jq
elif [[ "$1" == "api" ]]; then
  (
    cd "$CWD/api"
    cargo lambda watch
  )
elif [[ "$1" == "mock" ]]; then
  # TODO: pass argv into this when ready
  cargo run --bin radiojournal-cli
fi
