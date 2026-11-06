#!/usr/bin/env bash
# Starts the demo Mouser API (fixed data, no real orders) for the recordings in this directory.
# Run from the repository root; the tapes call it hidden before the first frame.
set -euo pipefail
cargo build --quiet --release -p mouser-mcp
if ! curl -fsS -o /dev/null "http://127.0.0.1:8787/api/v1/cart?apiKey=probe&cartKey=x" 2>/dev/null; then
  python3 -I demo/fake_mouser.py 8787 2>/tmp/fake-mouser.log &
  sleep 1
fi
