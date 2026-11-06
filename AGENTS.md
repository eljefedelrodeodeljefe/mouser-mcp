# AGENTS.md — conventions for `mouser-mcp`

Ported from `rigol-mcp` (itself from `mikrotik-mcp`) so muscle memory transfers.

## Branching and commits

`main` is the sole long-lived branch. Feature work branches off `main` and returns via pull
request. Branch prefixes, kebab-case: `feat/`, `fix/`, `bugfix/`, `docs/`, `chore/`,
`refactor/`.

Conventional commits drive release-please. `feat:` bumps minor, `fix:` and `perf:` bump
patch, `feat!:` or a `BREAKING CHANGE:` footer bumps major. One logical change per commit.

## Layout

```text
crates/mouser/          library — no MCP types cross into here
  src/models.rs         generated from Mouser's Swagger documents (tools/gen_models.py)
  src/client.rs         one method per endpoint; validation, key handling, order checks
  src/error.rs          typed errors
  examples/probe.rs     read-only check of a live key
  tests/api.rs          the client against a mock Mouser API (wiremock)
crates/mouser-mcp/      binary
  src/server.rs         MCP adapter — #[tool] stubs only; one router per key
  src/tools.rs          pure logic, returns anyhow::Result<Value>
  src/params.rs         Deserialize + JsonSchema argument structs
  src/error.rs          maps failures onto MCP error codes
tools/gen_models.py     Swagger -> models.rs
```

## Models

`models.rs` is generated: never edit it by hand. When Mouser changes its API, download both
documents (V1 and V2) and regenerate, as the script's docstring shows, then `cargo fmt`.
Mouser's own names are kept (serde renames), including its misspellings
(`OrderInializeRequestRoot`): they are the API.

## Adding a tool

1. A parameter struct in `params.rs`, deriving `Deserialize` and `JsonSchema`, a description
   on every field.
2. Logic in `tools.rs` returning `anyhow::Result<Value>`. No MCP types there.
3. A `#[tool(description = "…")]` stub in `server.rs` on the router of the key it needs
   (`search_router`, `cart_router`, `order_router`). Name the Mouser operation and its
   method and path in the description. Call `self.guard_cart()?` first if it changes a cart,
   `self.guard_orders()?` if it can place an order.

## Safety rules specific to this server

Ordering spends real money. These are not negotiable:

- Only `Client::submit_order` and `Client::submit_order_from_order` may send
  `SubmitOrder: true`. Every other path forces `false`.
- A submission previews again and goes ahead only if currency, total and every line match
  what the caller confirmed, and the total is within `Config::max_order_total`. No ceiling
  configured means no order. These checks live in the library, not the tool layer; do not add
  a parameter that relaxes them.
- Cart changes and orders are separate gates (`MOUSER_ALLOW_CART_WRITES`,
  `MOUSER_ALLOW_ORDERS`); neither implies the other.
- Never log a URL or a request: the key is in the query string. Transport errors are stripped
  of URLs (`reqwest::Error::without_url`) and replies are scrubbed of the key.
- Tests never call the real Mouser API. Live checks go through `examples/probe.rs`, read-only.

## Things to avoid

- Don't add AI attribution to commits or PRs. No `Co-Authored-By` trailers for AI tools:
  commits are authored by the human who reviews and accepts the change.
- Don't hand-roll parsing. Dates go through `time`, money through `rusty_money` (each
  currency's own separators) into exact `rust_decimal` amounts, JSON through `serde`. A value
  that does not parse is reported as unreadable, never guessed.
- Don't add dependencies casually; justify each in the commit message. `cargo deny check`
  (advisories, licences, sources: `deny.toml`) runs in CI.
- Don't disable lints to make CI pass.

## Gates

All commits must pass `prek run --all-files`: `cargo fmt`, `cargo clippy --all-targets -- -D
warnings` (**no `#[allow]` escapes**; fix the lint), `cargo test --all`, `typos`, TOML/YAML
validation, markdownlint.

## Runtime

stdout is the MCP protocol channel. All diagnostics go to stderr via `tracing`; the workspace
denies `print_stdout`, `print_stderr` and `dbg_macro`.
