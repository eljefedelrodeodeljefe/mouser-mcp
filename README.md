# mouser-mcp

[![CI](https://github.com/eljefedelrodeodeljefe/mouser-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/eljefedelrodeodeljefe/mouser-mcp/actions/workflows/ci.yml)
[![Licence: MIT](https://img.shields.io/badge/licence-MIT-blue.svg)](LICENSE)
[![MCP](https://img.shields.io/badge/MCP-server-8A2BE2.svg)](https://modelcontextprotocol.io)

**Let an AI agent price your BOM at Mouser: live stock, lead times and price breaks, carts,
order history, and ordering that never happens by accident.**

A Rust SDK (`mouser`) and an [MCP](https://modelcontextprotocol.io) server (`mouser-mcp`) for
the complete [Mouser API](https://www.mouser.com/api-hub/): every endpoint of Mouser's Swagger
documents, typed. Point Claude Code, Cursor or any MCP client at it and ask "is everything on
this BOM in stock, and what does it cost at these quantities?".

## Tools

Only the tools for the keys you configure are listed.

| Key | Tools |
|---|---|
| Search API key | `search_by_keyword`, `search_by_keyword_and_manufacturer`, `search_by_part_number`, `search_by_part_number_and_manufacturer`, `list_manufacturers`, **`price_bom`** |
| Cart and Order API key | carts: `get_cart`, `update_cart`, `add_cart_items`, `update_cart_items`, `remove_cart_item`, `insert_schedule_cart_items`, `update_schedule_cart_items`, `delete_all_schedule_cart_items`, `create_cart_from_order` |
| | history: `order_history_by_date_filter`, `order_history_by_date_range`, `get_order_by_sales_order_number`, `get_order_by_web_order_number`, `get_order` |
| | ordering: `query_order_options`, `list_order_currencies`, `list_order_countries`, `preview_order`, `submit_order`, `preview_order_from_order`, `submit_order_from_order` |

`price_bom` takes BOM lines (part number, quantity, your reference), looks them up ten per
call by exact part number, picks the offer that covers the quantity from stock at the lowest
extended price, prices it at the line's quantity from the price breaks, flags shortages,
minimum and multiple violations and discontinued parts, and totals per currency. Amounts are
exact decimals, read with each currency's own separators; a price or stock figure that does not
parse is flagged on its line, never guessed.

Search results are compact by default (stock, lead time, Min/Mult, price breaks, lifecycle,
URLs); `full: true` returns Mouser's whole record. The library keeps every field Mouser sends,
including ones its documents do not describe.

## Safety

Searching and reading are free of side effects. Changing a cart and placing an order are not,
so they are gated separately, and an order is never a side effect:

1. **Cart changes** need `MOUSER_ALLOW_CART_WRITES=true`.
2. **Orders** need `MOUSER_ALLOW_ORDERS=true` **and** a ceiling, `MOUSER_MAX_ORDER_TOTAL`
   (in the order's currency). Without a ceiling no order is placed. The check is in the
   library, below the MCP layer.
3. `preview_order` always sends `SubmitOrder: false`, whatever the request says. Only
   `submit_order` sends `true`, and only after a **fresh preview** matches what the caller
   confirmed: currency, total and every line. If the price moved since the preview, nothing is
   placed and the caller gets an *invalid params* error explaining what changed.
4. **The user confirms in a form, not the model.** Before placing it, the server asks the
   person at the MCP client through [elicitation](https://modelcontextprotocol.io/specification/draft/client/elicitation).
   The form has a one-line summary (total, shipping, tax, payment, ship-to), one ticked box
   per order line (quantity, part, unit price, line total), and "Place this order", unticked.
   Clients step through form fields with the keyboard, so a long order stays readable. Only an
   accepted form with every box ticked places the order, and only if Mouser's final preview
   still has exactly the lines confirmed; unticking a line, declining, cancelling or not
   answering places nothing. A client that cannot show forms cannot order. On protocol
   `2026-07-28` the form travels as an `InputRequiredResult` (multi round-trip); the pending
   confirmation stays on the server, is single use, expires after ten minutes and is bound to
   the exact arguments of the call, so an answer cannot be replayed onto another order.

An order takes four short calls: `query_order_options` (shipping and payment codes),
`preview_order` (`cart_key`, `currency_code`, `shipping_code`, `payment_method`),
`submit_order` (the same, plus the preview's `total`), and the user's answer in the form.

Keys travel in Mouser's query string, so the client never logs URLs, strips them from
transport errors, scrubs the key from any reply that echoes it, and redacts `ApiKey` in
`Debug`.

## Demo

[`demo/`](demo/) has scripted terminal recordings ([VHS](https://github.com/charmbracelet/vhs))
of pricing a BOM and of an order confirmed in the form, against a stand-in Mouser API with
demo data, so anyone can try the server without a Mouser account.

## Install

**With cargo:**

```sh
cargo install --locked --git https://github.com/eljefedelrodeodeljefe/mouser-mcp mouser-mcp
```

**From source:**

```sh
git clone https://github.com/eljefedelrodeodeljefe/mouser-mcp.git
cd mouser-mcp
cargo install --locked --path crates/mouser-mcp   # or: cargo build --release
```

Release binaries (Linux and macOS, x86_64 and aarch64, with SBOMs) are attached to
[Releases](https://github.com/eljefedelrodeodeljefe/mouser-mcp/releases).

## Keys

Request keys in your Mouser account under **API Hub** (<https://www.mouser.com/api-hub/>):
the **Search API** key, and if you want carts and orders, the **Cart and Order API** key. Put
each in its own file, readable only by you:

```sh
mkdir -p ~/.config/mouser && chmod 700 ~/.config/mouser
$EDITOR ~/.config/mouser/search-api-key        # paste the key, one line
chmod 600 ~/.config/mouser/search-api-key
```

## Configure

**Claude Code:**

```sh
claude mcp add mouser --env MOUSER_SEARCH_API_KEY_FILE=$HOME/.config/mouser/search-api-key -- mouser-mcp
```

**Any MCP client (`.mcp.json` / `mcpServers`):**

```json
{
  "mcpServers": {
    "mouser": {
      "command": "mouser-mcp",
      "env": {
        "MOUSER_SEARCH_API_KEY_FILE": "/home/you/.config/mouser/search-api-key"
      }
    }
  }
}
```

| Variable | Default | Meaning |
|---|---|---|
| `MOUSER_SEARCH_API_KEY_FILE` / `MOUSER_SEARCH_API_KEY` | — | Search API key (file preferred) |
| `MOUSER_ORDER_API_KEY_FILE` / `MOUSER_ORDER_API_KEY` | — | Cart and Order API key |
| `MOUSER_ALLOW_CART_WRITES` | `false` | Enable cart changes |
| `MOUSER_ALLOW_ORDERS` | `false` | Enable `submit_order` / `submit_order_from_order` |
| `MOUSER_MAX_ORDER_TOTAL` | — | Order ceiling in the order's currency; required to order |
| `MOUSER_MAX_CALLS_PER_MINUTE` | `30` | Client-side spacing of calls (Mouser limits calls per key) |
| `MOUSER_TIMEOUT_MS` | `30000` | HTTP timeout |
| `MOUSER_API_BASE` | `https://api.mouser.com/api` | API root (tests) |

At least one key is required. `RUST_LOG=mouser=debug` logs each call (never the key) to stderr.

## Library

```rust
use mouser::{ApiKey, Client, Config, models::SearchByPartRequest};

let client = Client::new(Config {
    search_key: Some(ApiKey::from_file("/home/you/.config/mouser/search-api-key")?),
    ..Config::default()
})?;
let r = client
    .search_by_part_number(&SearchByPartRequest {
        mouser_part_number: Some(mouser::join_part_numbers(&["ZFRSC-42-S+".into()])?),
        part_search_options: Some("Exact".into()),
        ..Default::default()
    })
    .await?;
```

`cargo run -p mouser --example probe -- ZFRSC-42-S+` checks a live key with one read-only
search.

## Development

See [AGENTS.md](AGENTS.md). Gates: `prek run --all-files` (fmt, clippy `-D warnings`, tests,
typos, markdownlint). Tests run against a mock Mouser API (wiremock); nothing in the test
suite talks to Mouser.
