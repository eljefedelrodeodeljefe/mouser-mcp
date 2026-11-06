#!/usr/bin/env bash
# Starts the demo Mouser API (fixed data, no real orders) for the recordings in this directory.
# Run from the repository root; the tapes call it hidden before the first frame.
set -euo pipefail
cargo build --quiet --release -p mouser-mcp
if ! curl -fsS -o /dev/null "http://127.0.0.1:8787/api/v1/cart?apiKey=probe&cartKey=x" 2>/dev/null; then
  python3 -I demo/fake_mouser.py 8787 2>/tmp/fake-mouser.log &
  sleep 1
fi
# Claude Code runs in an empty bench folder, so the recording shows a user's session, not
# this repository. The MCP config there points at this checkout's release build.
bench="${MOUSER_DEMO_BENCH:-$HOME/.cache/mouser-mcp-demo/bench}"
mkdir -p "$bench"
mkdir -p "$bench/.claude"
sed "s#\./target/release/mouser-mcp#$PWD/target/release/mouser-mcp#" demo/mcp.json > "$bench/.mcp.json"
rm -f "$bench/mcp.json"
# Approve the project's server and its tools, so the session asks nothing.
cat > "$bench/.claude/settings.local.json" <<'JSON'
{
  "enabledMcpjsonServers": ["mouser"],
  "permissions": { "allow": ["mcp__mouser"] }
}
JSON
# Ask the agent to name each tool it calls, so a viewer can follow the session.
printf '%s\n' "Before each Mouser tool call, say in one short line which tool you call and why." > "$bench/CLAUDE.md"
