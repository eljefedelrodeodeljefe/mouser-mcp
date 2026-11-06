use anyhow::Context;
use mouser::models::OrderRoot;
use mouser::{ApiKey, Client, Config, KeyKind};
use rmcp::{
    RoleServer, ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        tool::{InputResponses as ToolInputResponses, RequestState},
        wrapper::Parameters,
    },
    model::{
        CallToolResponse, CallToolResult, ContentBlock, ElicitRequest, ElicitRequestParams,
        ErrorCode, ErrorData, Implementation, InputRequest, InputRequests, InputRequiredResult,
        InputResponses, JsonObject, ProtocolVersion, ServerCapabilities, ServerConfig,
    },
    service::{ElicitationError, ElicitationMode, RequestContext},
    tool, tool_handler, tool_router,
};
use serde_json::Value;
use std::time::Duration;

use crate::confirm::{self, Answer, OrderConfirmation};
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
    /// Order confirmations asked for and not yet answered (multi round-trip path).
    confirmations: confirm::Store,
}

/// Whether an order goes ahead.
enum Gate {
    /// The user confirmed it.
    Go,
    /// Ask the user first: return this to the client.
    Ask(CallToolResponse),
}

fn refused(why: impl std::fmt::Display) -> ErrorData {
    ErrorData::new(
        ErrorCode::INVALID_REQUEST,
        format!("nothing was ordered: {why}"),
        None,
    )
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
            confirmations: confirm::Store::default(),
        })
    }

    /// Asks the user, through the MCP client, to confirm the order in `preview` (see
    /// [`confirm`]). The preview has already passed `Client::verify_order`.
    async fn confirm_order(
        &self,
        tool: &str,
        args: JsonObject,
        preview: &OrderRoot,
        state: Option<String>,
        responses: Option<InputResponses>,
        ctx: &RequestContext<RoleServer>,
    ) -> Result<Gate, ErrorData> {
        // Only a client that declared form elicitation shows the form to its user.
        if !ctx
            .peer
            .supported_elicitation_modes()
            .contains(&ElicitationMode::Form)
        {
            return Err(refused(
                "this MCP client cannot show a confirmation form (MCP elicitation), and orders need one",
            ));
        }
        let text = confirm::summary(tool, preview, self.client.config().max_order_total);
        let round_trip = ctx
            .protocol_version()
            .is_some_and(|v| v >= ProtocolVersion::V_2026_07_28);
        if round_trip {
            let Some(id) = state else {
                let id = self.confirmations.open(tool, args);
                let mut requests = InputRequests::new();
                requests.insert(
                    confirm::INPUT_KEY.into(),
                    InputRequest::Elicitation(ElicitRequest::new(
                        ElicitRequestParams::FormElicitationParams {
                            meta: None,
                            message: text,
                            requested_schema: confirm::schema(),
                        },
                    )),
                );
                tracing::info!(tool, "asking the user to confirm an order");
                return Ok(Gate::Ask(
                    InputRequiredResult::new(Some(requests), Some(id)).into(),
                ));
            };
            self.confirmations.take(&id, tool, &args).map_err(refused)?;
            let response = responses.as_ref().and_then(|r| r.get(confirm::INPUT_KEY));
            return match confirm::answer(response) {
                Answer::Confirmed => Ok(Gate::Go),
                Answer::Refused(why) => Err(refused(why)),
            };
        }
        match ctx.peer.elicit::<OrderConfirmation>(text).await {
            Ok(Some(c)) if c.place_order => Ok(Gate::Go),
            Ok(_) => Err(refused("the user did not tick \"Place this order\"")),
            Err(ElicitationError::UserDeclined) => Err(refused("the user declined")),
            Err(ElicitationError::UserCancelled) => Err(refused("the user cancelled")),
            Err(ElicitationError::CapabilityNotSupported) => Err(refused(
                "this MCP client cannot show a confirmation form (MCP elicitation), and orders need one",
            )),
            Err(e) => Err(refused(format!("the confirmation failed: {e}"))),
        }
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
        description = "Add parts to a cart, keeping what is in it. Without cart_key: a new cart (MouserCart AddCartItems, POST /api/v1/cart/items/insert). With cart_key: reads the cart (GET /api/v1/cart), adds the items (quantities of a part already there add up), writes the whole cart (MouserCart UpdateCart, POST /api/v1/cart) and reads it back; fails if any line is missing or wrong, because Mouser's insert endpoint has been seen dropping existing lines. Requires MOUSER_ALLOW_CART_WRITES"
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
        description = "PLACE AN ORDER at Mouser — a real purchase (POST /api/v1/order with SubmitOrder true). Call after preview_order. Previews again; if CurrencyCode, OrderTotal and every line match `expected` and the total is within MOUSER_MAX_ORDER_TOTAL, the user is asked in a confirmation form from the MCP client (elicitation) showing the amount and a summary, and the order is placed only if they confirm there. Clients without elicitation cannot order. Requires MOUSER_ALLOW_ORDERS"
    )]
    async fn submit_order(
        &self,
        Parameters(p): Parameters<SubmitOrderParams>,
        args: JsonObject,
        RequestState(state): RequestState,
        ToolInputResponses(responses): ToolInputResponses,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        self.guard_orders()?;
        tracing::warn!(currency = %p.expected.currency_code, total = p.expected.order_total, "submit_order requested");
        let preview = tools::verified_preview(&self.client, &p)
            .await
            .map_err(tool_error)?;
        match self
            .confirm_order("submit_order", args, &preview, state, responses, &ctx)
            .await?
        {
            Gate::Ask(r) => Ok(r),
            Gate::Go => Self::done(tools::submit_order(&self.client, &p).await).map(Into::into),
        }
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
        description = "PLACE A RE-ORDER of an earlier order — a real purchase (POST /api/v1/order/CreateFromOrder with SubmitOrder true), with the same fresh-preview check and user confirmation form as submit_order. Requires MOUSER_ALLOW_ORDERS"
    )]
    async fn submit_order_from_order(
        &self,
        Parameters(p): Parameters<SubmitFromOrderParams>,
        args: JsonObject,
        RequestState(state): RequestState,
        ToolInputResponses(responses): ToolInputResponses,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        self.guard_orders()?;
        tracing::warn!(
            order_number = p.order_number,
            total = p.expected.order_total,
            "submit_order_from_order requested"
        );
        let preview = tools::verified_preview_from_order(&self.client, &p)
            .await
            .map_err(tool_error)?;
        match self
            .confirm_order(
                "submit_order_from_order",
                args,
                &preview,
                state,
                responses,
                &ctx,
            )
            .await?
        {
            Gate::Ask(r) => Ok(r),
            Gate::Go => {
                Self::done(tools::submit_order_from_order(&self.client, &p).await).map(Into::into)
            }
        }
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
    "preview_order. submit_order re-previews, refuses if anything changed, and then asks the user ",
    "directly in a confirmation form (MCP elicitation) with the amount and a summary; only their ",
    "answer there places the order. Mouser limits calls per key; the server spaces calls to at ",
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

    // ── Order confirmation, end to end over MCP ─────────────────────────────────────────

    use rmcp::ClientHandler;
    use rmcp::model::{
        CallToolRequestParams, ClientCapabilities, ElicitResult, ElicitationAction,
        InitializeRequestParams, InitializeResult,
    };
    use rmcp::service::{RoleClient, ServiceError, serve_directly};
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// A client whose user answers the confirmation form with `answer` (`None`: the client
    /// cannot show forms), and remembers the text it was shown.
    #[derive(Clone)]
    struct User {
        answer: Option<(ElicitationAction, bool)>,
        shown: Arc<Mutex<Vec<String>>>,
    }

    impl ClientHandler for User {
        async fn create_elicitation(
            &self,
            request: ElicitRequestParams,
            _: RequestContext<RoleClient>,
        ) -> Result<rmcp::model::ElicitResult, ErrorData> {
            if let ElicitRequestParams::FormElicitationParams { message, .. } = &request {
                self.shown.lock().unwrap().push(message.clone());
            }
            let (action, tick) = self
                .answer
                .clone()
                .expect("no form for a client without elicitation");
            Ok(ElicitResult::new(action).with_content(json!({"place_order": tick})))
        }
    }

    async fn mount_order(server: &MockServer, submit: bool, times: impl Into<wiremock::Times>) {
        let reply = json!({"Order": {"Errors": [], "CurrencyCode": "EUR", "OrderTotal": 200.0,
            "MerchandiseTotal": 174.2, "SubmitOrder": submit, "OrderId": "placed-1",
            "OrderLines": [{"MouserPartNumber": "139-ZFRSC-42-S+", "Quantity": 2, "UnitPrice": 87.1, "ExtendedPrice": 174.2}]}});
        Mock::given(method("POST"))
            .and(path("/api/v1/order"))
            .and(body_json(json!({"Order": {"CartKey": "cart-1", "CurrencyCode": "EUR", "SubmitOrder": submit}})))
            .respond_with(ResponseTemplate::new(200).set_body_json(reply))
            .expect(times)
            .mount(server)
            .await;
    }

    /// Calls submit_order through an MCP client speaking `protocol`; returns the tool's text
    /// or the error, and what the user was shown.
    async fn order_through_mcp(
        mouser: &MockServer,
        protocol: ProtocolVersion,
        answer: Option<(ElicitationAction, bool)>,
    ) -> (Result<String, String>, Vec<String>) {
        let server = MouserServer::new(
            Config {
                base_url: format!("{}/api", mouser.uri()),
                order_key: Some(ApiKey::new("test-order-key-00000").unwrap()),
                max_calls_per_minute: 0,
                max_order_total: Some(500.0),
                ..Config::default()
            },
            false,
            true,
        )
        .unwrap();
        let shown = Arc::new(Mutex::new(vec![]));
        let can_ask = answer.is_some();
        let user = User {
            answer,
            shown: shown.clone(),
        };
        let caps = if can_ask {
            ClientCapabilities::builder().enable_elicitation().build()
        } else {
            ClientCapabilities::default()
        };
        let client_info = InitializeRequestParams::new(caps, Implementation::new("test", "0"))
            .with_protocol_version(protocol.clone());
        let mut server_info =
            InitializeResult::new(ServerCapabilities::builder().enable_tools().build());
        server_info.protocol_version = protocol;
        let result = tokio::task::LocalSet::new()
            .run_until(async move {
                let (st, ct) = tokio::io::duplex(1 << 16);
                let srv = tokio::task::spawn_local(async move {
                    let running =
                        serve_directly::<RoleServer, _, _, _, _>(server, st, Some(client_info));
                    let _ = running.waiting().await;
                });
                let client =
                    serve_directly::<RoleClient, _, _, _, _>(user, ct, Some(server_info.into()));
                let args = json!({
                    "order": {"CartKey": "cart-1", "CurrencyCode": "EUR"},
                    "expected": {"currency_code": "EUR", "order_total": 200.0,
                        "lines": [["139-ZFRSC-42-S+", 2]]}
                });
                let r = client
                    .call_tool(
                        CallToolRequestParams::new("submit_order")
                            .with_arguments(args.as_object().unwrap().clone()),
                    )
                    .await;
                srv.abort();
                match r {
                    Ok(r) => Ok(r.content[0].as_text().unwrap().text.clone()),
                    Err(ServiceError::McpError(e)) => Err(e.message.to_string()),
                    Err(e) => Err(e.to_string()),
                }
            })
            .await;
        let shown = shown.lock().unwrap().clone();
        (result, shown)
    }

    #[tokio::test(flavor = "current_thread")]
    async fn an_order_is_placed_only_after_the_user_confirms_it_in_a_form() {
        for protocol in [ProtocolVersion::V_2026_07_28, ProtocolVersion::V_2025_06_18] {
            let mouser = MockServer::start().await;
            mount_order(&mouser, true, 1).await;
            mount_order(&mouser, false, 1..).await;
            let (r, shown) = order_through_mcp(
                &mouser,
                protocol.clone(),
                Some((ElicitationAction::Accept, true)),
            )
            .await;
            assert!(
                r.as_ref().unwrap().contains("placed-1"),
                "{protocol:?}: {r:?}"
            );
            assert_eq!(shown.len(), 1, "{protocol:?}");
            assert!(
                shown[0].contains("Place a real order at Mouser for 200.00 EUR?")
                    && shown[0].contains("2 × 139-ZFRSC-42-S+"),
                "{}",
                shown[0]
            );
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn no_order_without_a_yes_from_the_user() {
        for protocol in [ProtocolVersion::V_2026_07_28, ProtocolVersion::V_2025_06_18] {
            for answer in [
                Some((ElicitationAction::Decline, false)),
                Some((ElicitationAction::Cancel, true)),
                Some((ElicitationAction::Accept, false)),
                None, // a client that cannot show the form
            ] {
                let mouser = MockServer::start().await;
                mount_order(&mouser, true, 0).await;
                mount_order(&mouser, false, 0..).await;
                let (r, _) = order_through_mcp(&mouser, protocol.clone(), answer.clone()).await;
                let e = r.expect_err("an order without confirmation");
                assert!(
                    e.contains("nothing was ordered"),
                    "{protocol:?} {answer:?}: {e}"
                );
            }
        }
    }
}
