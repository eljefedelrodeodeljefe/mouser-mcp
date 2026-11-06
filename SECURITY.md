# Security policy

## Reporting a vulnerability

Please report vulnerabilities privately through
[GitHub's private vulnerability reporting](https://github.com/eljefedelrodeodeljefe/mouser-mcp/security/advisories/new),
not in a public issue. You will get an answer within a week.

Especially relevant here: anything that could expose a Mouser API key (logs, error messages,
replies), or place or change an order without the checks described in the README (the
`MOUSER_ALLOW_ORDERS` gate, the `MOUSER_MAX_ORDER_TOTAL` ceiling, the fresh-preview match).

## Supported versions

Fixes go into the latest release only.

## What this project does to stay safe

- Dependencies: `cargo deny check` (RustSec advisories, licences, crates.io only) in CI;
  Dependabot security and version updates; a weekly SBOM scan with Grype.
- Code: CodeQL; secret scanning with push protection.
- Workflows: actions pinned to commit SHAs, read-only default token.
- `main` is protected: required CI checks, no force pushes or deletions; release tags cannot be
  moved or deleted.
