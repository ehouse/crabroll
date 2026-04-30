#!/bin/bash
set -e

BINARY=$(cargo test --no-run --message-format=json 2>/dev/null \
  | jq -r 'select(.executable != null) | .executable' \
  | tail -1)

ln -sf "$BINARY" target/debug/crabroll-test
