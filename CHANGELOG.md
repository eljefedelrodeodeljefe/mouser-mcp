# Changelog

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
