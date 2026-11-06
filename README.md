# mouser-mcp

[![CI](https://github.com/eljefedelrodeodeljefe/mouser-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/eljefedelrodeodeljefe/mouser-mcp/actions/workflows/ci.yml)
[![Licence: MIT](https://img.shields.io/badge/licence-MIT-blue.svg)](LICENSE)
[![MCP](https://img.shields.io/badge/MCP-server-8A2BE2.svg)](https://modelcontextprotocol.io)

**Search, price, cart and order at Mouser from Claude Code, Cursor or any MCP client. The
agent finds the parts and prepares the order; the person places it in a confirmation form
that the model does not see.**

![Claude Code finds in-stock alternatives for a backordered RF BOM at Mouser](demo/search.gif)

A Rust SDK (`mouser`) and an [MCP](https://modelcontextprotocol.io) server (`mouser-mcp`) for
the whole [Mouser API](https://www.mouser.com/api-hub/): part search with live stock, lead
times and price breaks, `price_bom` for a BOM in one call, carts, order history and ordering.
Every endpoint of Mouser's Swagger documents is typed. Cart changes and orders are off until
each is switched on, every order has a ceiling in its currency, and the person at the MCP
client confirms each order in a form. Release binaries for Linux and macOS come with SBOMs.

For hardware engineers and makers who buy parts from an agent session, and for Rust programs
that talk to Mouser.

## What it does in practice

From a session with the live Mouser API on 2026-11-06, for a two-delay-line test at 2–3 GHz:

1. `price_bom` priced the first-choice BOM at €732.26. The splitter, Mini-Circuits
   ZFRSC-42-S+, had none in stock and a 217-day lead time; the 6 dB pads, the 1 m and 2 m
   cables and the terminations were out of stock as well, the 2 m cable until March. Only the
   jumpers could ship.
2. Asked "Are there no alternative parts?", the agent searched Mouser by keyword with the
   in-stock filter, checked each candidate's frequency range against 2–3 GHz and proposed
   ZX10R-2-183-S+ (resistive, DC–18 GHz, 2 in stock, €161.10), Fairview FMAT7467-6 pads
   (DC–12 GHz), Cinch 415-0033 cables in 1 m and 2 m, and Fairview ST0645 terminations:
   €493.78 without the jumpers, all in stock. It flagged that the splitter's stock equalled
   the quantity needed, and that the listings did not state two of the specs.
3. `add_cart_items` put the parts in a cart. Mouser's own insert endpoint was seen dropping
   lines already in a cart; the tool writes the whole cart and reads it back instead.
4. `query_order_options`, `preview_order` (€533.78 with €40.00 UPS, no VAT under reverse
   charge on a company VAT number), `submit_order`. Claude Code showed the confirmation form,
   and the order was placed after the person accepted it with every line selected.

The recording above replays this story against a stand-in Mouser API with the same parts and
prices ([`demo/`](demo/)); its order goes to an invented address and gets a `DEMO-` number.

## Safety

Searching and reading change nothing. Changing a cart and placing an order each have their
own gate, and every order needs the person's confirmation:

1. **Cart changes** need `MOUSER_ALLOW_CART_WRITES=true`.
2. **Orders** need `MOUSER_ALLOW_ORDERS=true` **and** a ceiling, `MOUSER_MAX_ORDER_TOTAL`,
   an amount and its currency (`"500 EUR"`). Without a ceiling no order is placed, and an
   order in another currency is refused: the caller chooses an order's currency, so a ceiling
   without one would bound nothing. The check is in the library, below the MCP layer.
3. `preview_order` always sends `SubmitOrder: false`, whatever the request says. Only
   `submit_order` sends `true`, and only after a **fresh preview** matches what the caller
   confirmed: currency, total and every line. If the price moved since the preview, nothing is
   placed and the caller gets an *invalid params* error explaining what changed.
4. **The person at the MCP client confirms every order in a form**
   ([elicitation](https://modelcontextprotocol.io/specification/draft/client/elicitation)).
   - The form has a one-line summary (total, shipping, tax, payment, ship-to), "Order
     lines", every line with quantity, part, unit price and line total, all selected, and
     "Place this order", unticked. Clients step through the list with the keyboard.
   - Only an accepted form with every line still selected and the box ticked places the
     order, and only if Mouser's final preview still has the lines confirmed. Deselecting a
     line, declining, cancelling or not answering places nothing. A client that cannot show
     forms cannot order.
   - A confirmation is single use, expires after ten minutes and is bound to the exact
     arguments of the call, so an answer cannot be replayed onto another order. On protocol
     `2026-07-28` the form travels as an `InputRequiredResult` (multi round-trip) and the
     pending confirmation stays on the server; on earlier protocols the server waits ten
     minutes for the answer, then refuses.
5. **A submission without an answer may have placed the order.** If the submission times out or the connection
   drops after it was sent, Mouser may have placed the order. The tool then fails with its
   own error (code `-32001`, "the order may have been placed") instead of a transport error,
   and says to look for the order (`order_history_by_date_filter` with `Today`, or
   `get_order`) before submitting again. The server never retries a submission.

The order from the recording, with the form as Claude Code shows it (stand-in API, invented
address, nothing bought). The model prepared the cart and the preview; the form goes to the
person, and the model cannot answer it:

![Claude Code carts the parts and orders them; the person confirms the order in a form](demo/order.gif)

```text
MCP server "mouser" requests your input
Real order at Mouser: 635.20 EUR · 5 lines · merchandise 493.78 · UPS Worldwide Express Saver 40.00 · tax 101.42 · PayPal · to Ada Lovelace Berlin DE
❯ ✔ Order lines: ▾
      ❯ ☒ "2 × ZX10R-2-183-S+  161.10 each  = 322.20 EUR"
        ☒ "2 × FMAT7467-6  53.10 each  = 106.20 EUR"
        ☒ "2 × ST0645  12.63 each  = 25.26 EUR"
        ☒ "1 × 415-0033-M1.0  17.44 each  = 17.44 EUR"
        ☒ "1 × 415-0033-M2.0  22.68 each  = 22.68 EUR"
      Everything in this order. Deselect any line to stop the order.
  ✔ Place this order: 635.20 EUR: ☐
      A real purchase, charged to the payment method above
  Accept    Decline
Esc to cancel · ↑/↓ to navigate · Backspace to unset · Space to toggle
```

An order takes four steps: `query_order_options` (shipping and payment codes),
`preview_order` (`cart_key`, `currency_code`, `shipping_code`, `payment_method`),
`submit_order` (the same, plus the preview's `total`), and the user's answer in the form.
A re-order of an earlier order takes the same arguments plus its `order_number`
(`preview_order_from_order`, `submit_order_from_order`). Neither takes an address or freight
accounts: those come from the Mouser account, so the model cannot send an order somewhere the
form does not show.

Keys travel in Mouser's query string, so the client never logs URLs, strips them from
transport errors, scrubs the key from any reply that echoes it, and redacts `ApiKey` in
`Debug`.

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

Mouser issues **two separate keys**, and each works only for its own API:

| Key | For | Where |
|---|---|---|
| **Search API** key | search, `price_bom` | <https://www.mouser.com/api-hub/>, under *Search API*: request it (it is not issued automatically) |
| **Cart and Order API** key | carts, order history, orders | the same page, *Manage APIs*: one key covers Order, Cart and Order History |

Both look like GUIDs (`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`). Put each in its own file,
readable only by you:

```sh
mkdir -p ~/.config/mouser && chmod 700 ~/.config/mouser
$EDITOR ~/.config/mouser/search-api-key        # paste the key, one line
$EDITOR ~/.config/mouser/order-api-key         # optional: carts and orders
chmod 600 ~/.config/mouser/*-api-key
```

### Pitfalls

- **`Invalid (API Key): Invalid unique identifier.`** is what Mouser answers to a key used for
  the wrong API, as well as to a mistyped one. If *Manage APIs* shows Order, Cart and Order
  History next to your key, that is the Cart and Order key: the Search API needs its own.
- **The server reads the key files when it starts.** After changing one, restart the server
  (in Claude Code: `/mcp`, then reconnect `mouser`). An empty key file stops it from starting.
- **Calls are rate limited per key.** The server spaces them to
  `MOUSER_MAX_CALLS_PER_MINUTE` (30 by default); `price_bom` looks up ten lines per call.
- **Prices and currency follow your Mouser account's region** (mouser.de prices in EUR, for
  example); `price_bom` reports them per currency.
- **Ordering uses the account's defaults.** Shipping address, payment methods (PayPal, card,
  invoice terms) and VAT details come from your Mouser account; `query_order_options` lists
  what it offers. With a validated EU VAT number the preview can show no tax (reverse
  charge). Mouser fills in the order type (for example `Rush`: ship what is in stock first).
- **`MOUSER_MAX_ORDER_TOTAL` needs its currency.** `"500 EUR"`, not `500`: a bare number
  stops the server from starting. Orders are placed only in the ceiling's
  currency; to order in another, change the ceiling.
- **Ordering needs a client that shows MCP forms** (elicitation), as Claude Code does: the
  person confirms the order there. Without one, `submit_order` refuses.
- **"The order may have been placed"** means the submission went out and no answer came
  back. Check the order history (or Mouser's website) before you submit again: a second
  submission can place a second order.
- **"The cart was written but did not verify"** (`add_cart_items`, code `-32002`): the cart
  changed, and reading it back did not show what was written. Look at it with `get_cart`
  before anything else: calling `add_cart_items` again adds the quantities a second time.
- **API carts have their own `CartKey`.** Keep the key the tools return; whether an API cart
  also shows up in the website's basket has not been verified.

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
| `MOUSER_MAX_ORDER_TOTAL` | — | Order ceiling with its currency, `"500 EUR"` (amount with a point, ISO 4217 code); required to order, and only orders in that currency are placed |
| `MOUSER_MAX_CALLS_PER_MINUTE` | `30` | Client-side spacing of calls (Mouser limits calls per key) |
| `MOUSER_TIMEOUT_MS` | `30000` | HTTP timeout |
| `MOUSER_API_BASE` | `https://api.mouser.com/api` | API root (tests and the demo) |

At least one key is required. The `MOUSER_ALLOW_*` switches take `true`/`false`, `1`/`0` or
`yes`/`no` in any case; any other value stops the server with an error naming the variable.
The server logs to stderr at `info` unless `RUST_LOG` says
otherwise: each submission, each order placed (its id, total and currency, never the address)
and each refusal. `RUST_LOG=mouser=debug` also logs each call (never the key).

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
parse is flagged as unreadable on its line.

Search results are compact by default (stock, lead time, Min/Mult, price breaks, lifecycle,
URLs); `full: true` returns Mouser's whole record. The library keeps every field Mouser sends,
including ones its documents do not describe.

## Demo

[`demo/`](demo/) holds the recordings above as a [VHS](https://github.com/charmbracelet/vhs)
script (`story.tape`) against `fake_mouser.py`, a stand-in Mouser API with the parts, prices
and lead times of the session described above. Anyone can run the server against it without a
Mouser account; see [demo/README.md](demo/README.md) to try it or to render the recordings.

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
