use anyhow::Context;
use mouser::{ApiKey, Client, Config, KeyKind};
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, ContentBlock, ErrorCode, ErrorData, Implementation, ServerCapabilities,
        ServerConfig,
    },
    tool, tool_handler, tool_router,
};
use serde_json::Value;
use std::time::Duration;

use crate::error::tool_error;
use crate::params::*;
use crate::tools;

pub struct MouserServer {
    client: Client,
    /// Only the tools whose key is configured: search tools with the Search API key, cart,
    /// history and order tools with the Cart and Order API key.
    tool_router: ToolRouter<Self>,
    cart_writes: bool,
    orders: bool,
}

/// `NAME` or the first line of the file `NAME_FILE` (preferred: keeps the key out of
/// process listings and of the MCP client's JSON config).
fn key_from_env(name: &str) -> anyhow::Result<Option<ApiKey>> {
    let file = format!("{name}_FILE");
    match (std::env::var(name), std::env::var(&file)) {
        (Ok(_), Ok(_)) => anyhow::bail!("set {name} or {file}, not both"),
        (Ok(v), Err(_)) => Ok(Some(ApiKey::new(v).with_context(|| name.to_string())?)),
        (Err(_), Ok(path)) => Ok(Some(
            ApiKey::from_file(&path).with_context(|| file.clone())?,
        )),
        (Err(_), Err(_)) => Ok(None),
    }
}

fn env_bool(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.as_str(), "true" | "1" | "yes"))
        .unwrap_or(false)
}

fn env_parse<T: std::str::FromStr>(name: &str) -> anyhow::Result<Option<T>> {
    match std::env::var(name) {
        Ok(v) => {
            Ok(Some(v.parse().map_err(|_| {
                anyhow::anyhow!("{name}: cannot parse {v:?}")
            })?))
        }
        Err(_) => Ok(None),
    }
}

impl MouserServer {
    pub fn from_env() -> anyhow::Result<Self> {
        let search_key = key_from_env("MOUSER_SEARCH_API_KEY")?;
        let order_key = key_from_env("MOUSER_ORDER_API_KEY")?;
        if search_key.is_none() && order_key.is_none() {
            anyhow::bail!(
                "no Mouser API key: set MOUSER_SEARCH_API_KEY_FILE (or MOUSER_SEARCH_API_KEY) and/or MOUSER_ORDER_API_KEY_FILE"
            );
        }
        let max_order_total: Option<f64> = env_parse("MOUSER_MAX_ORDER_TOTAL")?;
        if let Some(m) = max_order_total {
            anyhow::ensure!(
                m.is_finite() && m > 0.0,
                "MOUSER_MAX_ORDER_TOTAL must be a positive amount"
            );
        }
        let cfg = Config {
            base_url: std::env::var("MOUSER_API_BASE")
                .unwrap_or_else(|_| mouser::DEFAULT_BASE_URL.into()),
            search_key,
            order_key,
            timeout: Duration::from_millis(env_parse("MOUSER_TIMEOUT_MS")?.unwrap_or(30_000)),
            max_calls_per_minute: env_parse("MOUSER_MAX_CALLS_PER_MINUTE")?.unwrap_or(30),
            max_order_total,
        };
        Self::new(
            cfg,
            env_bool("MOUSER_ALLOW_CART_WRITES"),
            env_bool("MOUSER_ALLOW_ORDERS"),
        )
    }

    pub fn new(cfg: Config, cart_writes: bool, orders: bool) -> anyhow::Result<Self> {
        let client = Client::new(cfg)?;
        let mut tool_router = ToolRouter::new();
        if client.has_key(KeyKind::Search) {
            tool_router += Self::search_router();
        }
        if client.has_key(KeyKind::Order) {
            tool_router += Self::cart_router();
            tool_router += Self::order_router();
        }
        Ok(Self {
            client,
            tool_router,
            cart_writes,
            orders,
        })
    }

    pub fn has_search(&self) -> bool {
        self.client.has_key(KeyKind::Search)
    }

    pub fn has_order(&self) -> bool {
        self.client.has_key(KeyKind::Order)
    }

    fn ok(value: &Value) -> CallToolResult {
        CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()),
        )])
    }

    fn done(r: anyhow::Result<Value>) -> Result<CallToolResult, ErrorData> {
        r.map(|v| Self::ok(&v)).map_err(tool_error)
    }

    fn guard_cart(&self) -> Result<(), ErrorData> {
        if self.cart_writes {
            Ok(())
        } else {
            Err(ErrorData::new(
                ErrorCode::INVALID_REQUEST,
                "cart changes are disabled — set MOUSER_ALLOW_CART_WRITES=true to enable",
                None,
            ))
        }
    }

    fn guard_orders(&self) -> Result<(), ErrorData> {
        if self.orders {
            Ok(())
        } else {
            Err(ErrorData::new(
                ErrorCode::INVALID_REQUEST,
                "placing orders is disabled — set MOUSER_ALLOW_ORDERS=true and MOUSER_MAX_ORDER_TOTAL to enable",
                None,
            ))
        }
    }

    #[cfg(test)]
    pub fn tool_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .tool_router
            .list_all()
            .into_iter()
            .map(|t| t.name.to_string())
            .collect();
        v.sort();
        v
    }
}

#[tool_router(router = search_router)]
impl MouserServer {
    #[tool(
        description = "Search Mouser by keyword (Search API SearchByKeyword, POST /api/v1/search/keyword): up to 50 parts per call with stock, lead time and price breaks. Page with starting_record"
    )]
    async fn search_by_keyword(
        &self,
        Parameters(p): Parameters<KeywordParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::search_by_keyword(&self.client, &p).await)
    }

    #[tool(
        description = "Search Mouser by keyword within one manufacturer (SearchByKeywordAndManufacturer, POST /api/v2/search/keywordandmanufacturer). The manufacturer name must match list_manufacturers"
    )]
    async fn search_by_keyword_and_manufacturer(
        &self,
        Parameters(p): Parameters<KeywordMfrParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::search_by_keyword_and_manufacturer(&self.client, &p).await)
    }

    #[tool(
        description = "Look up 1 to 10 Mouser or manufacturer part numbers (SearchByPartNumber, POST /api/v1/search/partnumber): stock, on-order quantities, lead time, Min/Mult, price breaks, lifecycle, product URL"
    )]
    async fn search_by_part_number(
        &self,
        Parameters(p): Parameters<PartNumberParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::search_by_part_number(&self.client, &p).await)
    }

    #[tool(
        description = "Look up part numbers within one manufacturer (SearchByPartNumberAndManufacturer, POST /api/v2/search/partnumberandmanufacturer)"
    )]
    async fn search_by_part_number_and_manufacturer(
        &self,
        Parameters(p): Parameters<PartNumberMfrParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::search_by_part_number_and_manufacturer(&self.client, &p).await)
    }

    #[tool(
        description = "List the manufacturer names Mouser carries (GetManufacturerList, GET /api/v2/search/manufacturerlist), optionally filtered by a substring"
    )]
    async fn list_manufacturers(
        &self,
        Parameters(p): Parameters<ManufacturersParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::list_manufacturers(&self.client, &p).await)
    }

    #[tool(
        description = "Price a BOM at Mouser: every line looked up by exact part number (10 per SearchByPartNumber call), the best offer chosen (covers the quantity from stock, then lowest extended price), unit price at the line's quantity from the price breaks, stock status, Min/Mult notes, totals per currency. Read-only"
    )]
    async fn price_bom(
        &self,
        Parameters(p): Parameters<BomParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::price_bom(&self.client, &p).await)
    }
}

#[tool_router(router = cart_router)]
impl MouserServer {
    #[tool(
        description = "Get a cart with its lines and totals (MouserCart GetCart, GET /api/v1/cart)"
    )]
    async fn get_cart(
        &self,
        Parameters(p): Parameters<CartParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::get_cart(&self.client, &p).await)
    }

    #[tool(
        description = "Replace a cart's contents (MouserCart UpdateCart, POST /api/v1/cart): parts not listed are REMOVED. Without cart_key Mouser creates a new cart and returns its CartKey. Requires MOUSER_ALLOW_CART_WRITES"
    )]
    async fn update_cart(
        &self,
        Parameters(p): Parameters<CartItemsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_cart()?;
        Self::done(tools::update_cart(&self.client, &p).await)
    }

    #[tool(
        description = "Add parts to a cart (MouserCart AddCartItems, POST /api/v1/cart/items/insert). Without cart_key a new cart is created. Requires MOUSER_ALLOW_CART_WRITES"
    )]
    async fn add_cart_items(
        &self,
        Parameters(p): Parameters<CartItemsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_cart()?;
        Self::done(tools::add_cart_items(&self.client, &p).await)
    }

    #[tool(
        description = "Change quantities of the listed cart lines only (MouserCart UpdateCartItems, POST /api/v1/cart/items/update). Requires MOUSER_ALLOW_CART_WRITES"
    )]
    async fn update_cart_items(
        &self,
        Parameters(p): Parameters<CartItemsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_cart()?;
        Self::done(tools::update_cart_items(&self.client, &p).await)
    }

    #[tool(
        description = "Remove one part from a cart (MouserCart RemoveCartItem, POST /api/v1/cart/item/remove). Requires MOUSER_ALLOW_CART_WRITES"
    )]
    async fn remove_cart_item(
        &self,
        Parameters(p): Parameters<RemoveCartItemParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_cart()?;
        Self::done(tools::remove_cart_item(&self.client, &p).await)
    }

    #[tool(
        description = "Add scheduled deliveries to cart lines (MouserCart InsertScheduleCartItems, POST /api/v1/cart/insert/schedule). Requires MOUSER_ALLOW_CART_WRITES"
    )]
    async fn insert_schedule_cart_items(
        &self,
        Parameters(p): Parameters<ScheduleParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_cart()?;
        Self::done(tools::insert_schedule_cart_items(&self.client, &p).await)
    }

    #[tool(
        description = "Change scheduled deliveries of cart lines (MouserCart UpdateScheduleCartItems, POST /api/v1/cart/update/schedule). Requires MOUSER_ALLOW_CART_WRITES"
    )]
    async fn update_schedule_cart_items(
        &self,
        Parameters(p): Parameters<ScheduleParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_cart()?;
        Self::done(tools::update_schedule_cart_items(&self.client, &p).await)
    }

    #[tool(
        description = "Delete all scheduled deliveries of a cart (MouserCart DeleteAllScheduleCartItems, POST /api/v1/cart/deleteall/schedule). Requires MOUSER_ALLOW_CART_WRITES"
    )]
    async fn delete_all_schedule_cart_items(
        &self,
        Parameters(p): Parameters<CartKeyParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_cart()?;
        Self::done(tools::delete_all_schedule_cart_items(&self.client, &p).await)
    }
}

#[tool_router(router = order_router)]
impl MouserServer {
    #[tool(
        description = "List the account's orders for a date filter such as ThisMonth or YearToDate (MouserOrderHistory, GET /api/v1/orderhistory/ByDateFilter)"
    )]
    async fn order_history_by_date_filter(
        &self,
        Parameters(p): Parameters<DateFilterParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::order_history_by_date_filter(&self.client, &p).await)
    }

    #[tool(
        description = "List the account's orders between two dates (GET /api/v1/orderhistory/ByDateRange)"
    )]
    async fn order_history_by_date_range(
        &self,
        Parameters(p): Parameters<DateRangeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::order_history_by_date_range(&self.client, &p).await)
    }

    #[tool(
        description = "Order details with lines, status, tracking and invoices, by sales order number (GET /api/v1/orderhistory/salesOrderNumber)"
    )]
    async fn get_order_by_sales_order_number(
        &self,
        Parameters(p): Parameters<SalesOrderParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::order_by_sales_order_number(&self.client, &p).await)
    }

    #[tool(
        description = "Order details by web order number (GET /api/v1/orderhistory/webOrderNumber)"
    )]
    async fn get_order_by_web_order_number(
        &self,
        Parameters(p): Parameters<WebOrderParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::order_by_web_order_number(&self.client, &p).await)
    }

    #[tool(
        description = "An order as placed through the Order API, by web order number (Order GetOrder, GET /api/v1/order/{orderNumber})"
    )]
    async fn get_order(
        &self,
        Parameters(p): Parameters<OrderNumberParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::get_order(&self.client, &p).await)
    }

    #[tool(
        description = "Shipping methods, payment methods, tax certificates, VAT accounts and saved cards available for ordering a cart (Order OptionsQuery, POST /api/v1/order/options/query). Read-only"
    )]
    async fn query_order_options(
        &self,
        Parameters(p): Parameters<OrderOptionsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::order_options_query(&self.client, &p).await)
    }

    #[tool(
        description = "Currencies available for the account's addresses (GET /api/v1/order/currencies)"
    )]
    async fn list_order_currencies(
        &self,
        Parameters(p): Parameters<CurrenciesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::order_currencies(&self.client, &p).await)
    }

    #[tool(
        description = "Countries and their states/provinces for ordering (GET /api/v1/order/countries)"
    )]
    async fn list_order_countries(
        &self,
        Parameters(p): Parameters<CountriesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::order_countries(&self.client, &p).await)
    }

    #[tool(
        description = "Preview an order of a cart: lines, shipping, tax and OrderTotal, as Mouser would place it (POST /api/v1/order with SubmitOrder false — never places an order). Show the result to the user before any submit_order"
    )]
    async fn preview_order(
        &self,
        Parameters(p): Parameters<OrderParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::preview_order(&self.client, &p).await)
    }

    #[tool(
        description = "PLACE AN ORDER at Mouser — a real purchase (POST /api/v1/order with SubmitOrder true). Only after the user has seen preview_order and explicitly confirmed it. Previews again and submits only if CurrencyCode, OrderTotal and every line still match `expected` and the total is within MOUSER_MAX_ORDER_TOTAL. Requires MOUSER_ALLOW_ORDERS"
    )]
    async fn submit_order(
        &self,
        Parameters(p): Parameters<SubmitOrderParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_orders()?;
        tracing::warn!(currency = %p.expected.currency_code, total = p.expected.order_total, "submit_order requested");
        Self::done(tools::submit_order(&self.client, &p).await)
    }

    #[tool(
        description = "Preview re-ordering an earlier order (POST /api/v1/order/CreateFromOrder with SubmitOrder false — never places an order)"
    )]
    async fn preview_order_from_order(
        &self,
        Parameters(p): Parameters<FromOrderParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::done(tools::preview_order_from_order(&self.client, &p).await)
    }

    #[tool(
        description = "PLACE A RE-ORDER of an earlier order — a real purchase (POST /api/v1/order/CreateFromOrder with SubmitOrder true), with the same fresh-preview check as submit_order. Requires MOUSER_ALLOW_ORDERS"
    )]
    async fn submit_order_from_order(
        &self,
        Parameters(p): Parameters<SubmitFromOrderParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_orders()?;
        tracing::warn!(
            order_number = p.order_number,
            total = p.expected.order_total,
            "submit_order_from_order requested"
        );
        Self::done(tools::submit_order_from_order(&self.client, &p).await)
    }

    #[tool(
        description = "Create a new cart holding an earlier order's items (POST /api/v1/order/item/CreateCartFromOrder). Requires MOUSER_ALLOW_CART_WRITES"
    )]
    async fn create_cart_from_order(
        &self,
        Parameters(p): Parameters<CartFromOrderParams>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_cart()?;
        Self::done(tools::create_cart_from_order(&self.client, &p).await)
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for MouserServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("mouser-mcp", env!("CARGO_PKG_VERSION"))
                    .with_title("Mouser Electronics"),
            )
            .with_instructions(INSTRUCTIONS)
    }
}

const INSTRUCTIONS: &str = concat!(
    "Mouser Electronics APIs: part search with live stock, lead times and price breaks; carts; ",
    "order history; ordering. Configure MOUSER_SEARCH_API_KEY_FILE (search tools) and/or ",
    "MOUSER_ORDER_API_KEY_FILE (cart, history and order tools); only the tools for the configured ",
    "keys are listed. For a BOM use price_bom. Cart changes need MOUSER_ALLOW_CART_WRITES=true. ",
    "Placing an order needs MOUSER_ALLOW_ORDERS=true and MOUSER_MAX_ORDER_TOTAL, and must follow ",
    "preview_order and the user's explicit confirmation of that preview: submit_order re-previews ",
    "and refuses if anything changed. Mouser limits calls per key; the server spaces calls to at ",
    "most MOUSER_MAX_CALLS_PER_MINUTE (default 30).",
);

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(search: bool, order: bool) -> Config {
        Config {
            search_key: search.then(|| ApiKey::new("test-search-key-0000").unwrap()),
            order_key: order.then(|| ApiKey::new("test-order-key-00000").unwrap()),
            ..Config::default()
        }
    }

    #[test]
    fn only_the_tools_of_the_configured_keys_are_listed() {
        let s = MouserServer::new(cfg(true, false), false, false).unwrap();
        assert_eq!(
            s.tool_names(),
            [
                "list_manufacturers",
                "price_bom",
                "search_by_keyword",
                "search_by_keyword_and_manufacturer",
                "search_by_part_number",
                "search_by_part_number_and_manufacturer"
            ]
        );
        let all = MouserServer::new(cfg(true, true), false, false)
            .unwrap()
            .tool_names();
        assert_eq!(all.len(), 6 + 8 + 13, "{all:?}");
        assert!(all.contains(&"submit_order".to_string()) && all.contains(&"get_cart".to_string()));
    }

    #[test]
    fn writes_and_orders_are_gated_separately() {
        let s = MouserServer::new(cfg(false, true), true, false).unwrap();
        assert!(s.guard_cart().is_ok());
        assert!(s.guard_orders().is_err());
        let s = MouserServer::new(cfg(false, true), false, true).unwrap();
        assert!(s.guard_cart().is_err());
        assert!(s.guard_orders().is_ok());
    }
}
