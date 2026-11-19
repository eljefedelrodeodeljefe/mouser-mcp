#!/usr/bin/env bash
# Refuse private data in committed files and commit messages (AGENTS.md, "Private data").
#
# Two checks:
#   1. Email addresses. Only placeholders pass: example.com/.org/.net, *.invalid and
#      GitHub noreply addresses.
#   2. A private denylist kept OUT of the repo: one term per line (case-insensitive, fixed
#      string; blank lines and lines starting with # are ignored). Put your name, address,
#      customer, order and invoice numbers, card digits, VAT number in it. Path:
#      $MOUSER_MCP_PRIVATE_TERMS, else ~/.config/mouser-mcp/private-terms. Missing file:
#      check 2 is skipped (CI has none).
#
# Usage: check_private_data.sh FILE...
set -euo pipefail

email_re='[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'
allowed_re='@(([A-Za-z0-9-]+\.)*example\.(com|org|net)|([A-Za-z0-9.-]+\.)?invalid|users\.noreply\.github\.com)$|^noreply@'
terms_file="${MOUSER_MCP_PRIVATE_TERMS:-$HOME/.config/mouser-mcp/private-terms}"

terms=()
if [[ -r "$terms_file" ]]; then
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ -z "${line// /}" || "$line" == \#* ]] && continue
    terms+=("$line")
  done <"$terms_file"
fi

status=0
for f in "$@"; do
  [[ -f "$f" ]] || continue
  grep -Iq . "$f" 2>/dev/null || continue # skip binary and empty files

  while IFS=: read -r n addr; do
    if ! grep -Eqi "$allowed_re" <<<"$addr"; then
      echo "$f:$n: email address: $addr (use a placeholder such as buyer@example.com)"
      status=1
    fi
  done < <(grep -Eon "$email_re" "$f" || true)

  for t in "${terms[@]}"; do
    if n=$(grep -Fin -- "$t" "$f" | cut -d: -f1 | paste -sd, -); [[ -n "$n" ]]; then
      echo "$f:$n: a term from your private denylist ($terms_file)"
      status=1
    fi
  done
done

if [[ $status -ne 0 ]]; then
  echo "Private data found. Replace it with invented values; see AGENTS.md, \"Private data\"." >&2
fi
exit $status
