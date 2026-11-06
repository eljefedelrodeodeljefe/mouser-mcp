# Changelog

## [0.6.1](https://github.com/eljefedelrodeodeljefe/mouser-mcp/compare/v0.6.0...v0.6.1) (2026-11-06)


### Documentation

* **demo:** one recording that follows the session, cut into two clips ([42b4b4f](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/42b4b4f816c22ad075267bd6c8517f9ecd22f08d))
* **demo:** the stand-in Mouser API answers like the session it replays ([0f74227](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/0f74227f5ef0006f97007ca4f7123910ae1c0c02))
* **demo:** the stand-in searches the way Mouser does ([d1a8964](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/d1a89643c31f3fd92d94b81bfe8f9be8b8f18197))
* lead the README with a real session; one demo recording that follows it ([58e65d1](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/58e65d1ab675f66f4c42dfc0265b6e3947363ba6))
* lead the README with what the server did in a real session ([7aff17b](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/7aff17b52a5128e83f6157e5b3eb26de895e70b4))

## [0.6.0](https://github.com/eljefedelrodeodeljefe/mouser-mcp/compare/v0.5.0...v0.6.0) (2026-11-06)


### ⚠ BREAKING CHANGES

* MOUSER_MAX_ORDER_TOTAL takes "<amount> <currency>", e.g. MOUSER_MAX_ORDER_TOTAL="500 EUR". A bare amount stops the server from starting, and orders in a currency other than the ceiling's are refused. In the library, Config::max_order_total is Option<OrderCeiling> (build one with "500 EUR".parse() or OrderCeiling::new(amount, "EUR")).
* preview_order_from_order and submit_order_from_order no longer take `order` (an OrderRequestType), `country_code` or a separate `currency_code` query argument. Pass `order_number`, `cart_key`, `currency_code`, optional `shipping_code` and `payment_method` (as for preview_order), and `total` for the submission. Ship-to and freight accounts come from the Mouser account.

### Bug Fixes

* a cart written but not verified is not an argument error ([ab7235d](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/ab7235daadd4a2d66ae7a8d4d1f09ac4b44df852))
* a confirmation form nobody answers ends after ten minutes ([164507c](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/164507c5f8caf1ddbc391b18871d236a8490e10e))
* a submission without an answer says the order may have been placed ([2668db5](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/2668db5df13a2b1bec7e8b04547d360f70f4572f))
* log the order trail at info by default ([24a09c9](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/24a09c9642c07ae2576746457d24e431798ecdf9))
* re-orders take the short order choices, not a whole order body ([ca1160c](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/ca1160c095545cd6cdb5a5065f53f75d2b524e70))
* the order ceiling names its currency ([6fa36c5](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/6fa36c5592886aa9d1dc883e2b0298489a26aaac))
* the rate limiter and pending confirmations run on tokio's clock ([1afb2d1](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/1afb2d12cb1d41e90125db87535935cc2c4b0a27))
* yes/no settings accept six words, any case, and refuse the rest ([7577eef](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/7577eefe8adfb5e8cd3844a35d374506498a6418))


### Documentation

* **agents:** #[expect] with a reason where a lint is wrong for one site ([6b4227e](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/6b4227ea6ea62f7943556fdd029a7bf4f6c5e849))
* demo GIFs in the README ([5924123](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/5924123d4a05702901ebdaa419b14454651f6657))
* demo GIFs in the README ([93527c0](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/93527c0023b7f2c20926f8684a2e8ea90ecc01a5))
* getting the two Mouser keys, the pitfalls, and the order form ([2c768f2](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/2c768f2f9585511044f6220d0c72d35c3cb809ef))
* prose sweep against the airdress prose rules ([0df4286](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/0df4286cf2cc3001cd8b65140d55b980356671d2))
* prose sweep against the airdress prose rules ([db77611](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/db77611c01b1ca5d8ecb97ca97a5341f4ee5cdd1))

## [0.5.0](https://github.com/eljefedelrodeodeljefe/mouser-mcp/compare/v0.4.0...v0.5.0) (2026-11-06)


### Features

* the order form lists the lines in one multi-select field ([#11](https://github.com/eljefedelrodeodeljefe/mouser-mcp/issues/11)) ([373b0f2](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/373b0f268f3a2792f5006f53f10875593aa88f9f))

## [0.4.0](https://github.com/eljefedelrodeodeljefe/mouser-mcp/compare/v0.3.0...v0.4.0) (2026-11-06)


### ⚠ BREAKING CHANGES

* preview_order and submit_order take cart_key, currency_code, shipping_code and payment_method instead of Mouser's OrderRequestType, and submit_order takes the preview's total instead of an expected block with every line. The lines a submission must match are the ones the user confirmed in the form. submit_order_from_order takes total instead of expected likewise.

### Features

* an order form with a box per line; submit_order takes the total, not the lines ([#9](https://github.com/eljefedelrodeodeljefe/mouser-mcp/issues/9)) ([b42531d](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/b42531dcab9eb27e8a06227b89643d768ef080cc))


### Bug Fixes

* read the client's capabilities from each request on protocol 2026-07-28 ([#8](https://github.com/eljefedelrodeodeljefe/mouser-mcp/issues/8)) ([7b4234d](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/7b4234d910203a887556ff4e5151ee871728064a))

## [0.3.0](https://github.com/eljefedelrodeodeljefe/mouser-mcp/compare/v0.2.0...v0.3.0) (2026-11-06)


### Features

* the user confirms every order in an MCP elicitation form ([b01300b](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/b01300be66e680d5fb74435368331818a05fa9bc))
* the user confirms every order in an MCP elicitation form ([f0ff06d](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/f0ff06d91305a674c189c713b72ae9490bb6b21d))


### Bug Fixes

* add_cart_items keeps the lines already in a cart ([fe92b26](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/fe92b26d2b39be1493a4acc5b4c7425929e6d47e))
* cart writes against the live API ([4b62528](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/4b6252896be5aecaa9948e17fdd027d0507d14c5))
* decode enum fields Mouser sends as numbers ([ab6435b](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/ab6435b08dc7a28e1795a94a3d0b9cc861ac7cc3))
* decode enum fields Mouser sends as numbers ([d8cd356](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/d8cd356188d1d024e90328918bd613777ab73ffd))
* send Content-Length: 0 on POSTs without a body ([32d048c](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/32d048ce1a637194d6f78daef1db55b79eff8727))

## [0.2.0](https://github.com/eljefedelrodeodeljefe/mouser-mcp/compare/v0.1.0...v0.2.0) (2026-11-06)


### Features

* Mouser API client and MCP server ([dd82744](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/dd8274422d4cce02f0c953ee7fb5fa0b759729ca))


### Bug Fixes

* identify as mouser-mcp in the MCP handshake ([d0dc645](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/d0dc6457971564a56ca3d354c49a6e1a47bbb068))
* parse dates and prices with established crates; cargo-deny in CI ([d71945a](https://github.com/eljefedelrodeodeljefe/mouser-mcp/commit/d71945a1263912ef329a88f0f0fc1201c1908ea2))
