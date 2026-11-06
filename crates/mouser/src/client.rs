//! The Mouser API client: one method per endpoint of Mouser's Swagger documents
//! (<https://api.mouser.com/api/docs/ui/index>), named after the operation.

use std::collections::VecDeque;
use std::path::Path;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::error::{Error, KeyKind, Result};
use crate::models::*;

pub const DEFAULT_BASE_URL: &str = "https://api.mouser.com/api";

/// An API key. Its `Debug` output is redacted: keys travel in the query string, so the client
/// also strips URLs from transport errors.
#[derive(Clone)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(key: impl Into<String>) -> Result<Self> {
        let k = key.into().trim().to_string();
        // Mouser keys are GUIDs (36 characters). Requiring a realistic length also keeps the
        // scrubbing of replies (the key replaced by *** wherever an error page echoes it) from
        // ever matching ordinary text.
        if k.len() < 16 || k.chars().any(char::is_whitespace) {
            return Err(Error::InvalidArgument(
                "API key must be at least 16 characters without whitespace (Mouser keys are GUIDs)"
                    .into(),
            ));
        }
        Ok(Self(k))
    }

    /// Read a key from a file (the first line, trimmed), as for `MOUSER_SEARCH_API_KEY_FILE`.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let p = path.as_ref();
        let s = std::fs::read_to_string(p).map_err(|e| {
            Error::InvalidArgument(format!("cannot read key file {}: {e}", p.display()))
        })?;
        Self::new(s.lines().next().unwrap_or(""))
    }

    fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey(***)")
    }
}

/// Client configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// `https://api.mouser.com/api`; a test server in tests.
    pub base_url: String,
    pub search_key: Option<ApiKey>,
    pub order_key: Option<ApiKey>,
    pub timeout: Duration,
    /// Client-side cap on calls per rolling minute, over both keys. Mouser enforces its own
    /// limits per key (see your API hub page); this keeps a busy agent well clear of them.
    pub max_calls_per_minute: u32,
    /// Ceiling for `submit_order` / `submit_order_from_order`, in the order's currency. `None`
    /// refuses every submission: placing an order needs an explicit ceiling.
    pub max_order_total: Option<f64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.into(),
            search_key: None,
            order_key: None,
            timeout: Duration::from_secs(30),
            max_calls_per_minute: 30,
            max_order_total: None,
        }
    }
}

/// `searchOptions` of the keyword searches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum SearchOption {
    #[default]
    None,
    Rohs,
    InStock,
    RohsAndInStock,
}

impl SearchOption {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Rohs => "Rohs",
            Self::InStock => "InStock",
            Self::RohsAndInStock => "RohsAndInStock",
        }
    }
}

/// `partSearchOptions` of the part-number searches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum PartSearchOption {
    #[default]
    None,
    Exact,
}

impl PartSearchOption {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Exact => "Exact",
        }
    }
}

/// `dateFilter` of the order-history query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum DateFilter {
    None,
    All,
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    ThisMonth,
    LastMonth,
    ThisQuarter,
    LastQuarter,
    ThisYear,
    LastYear,
    YearToDate,
}

impl DateFilter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::All => "All",
            Self::Today => "Today",
            Self::Yesterday => "Yesterday",
            Self::ThisWeek => "ThisWeek",
            Self::LastWeek => "LastWeek",
            Self::ThisMonth => "ThisMonth",
            Self::LastMonth => "LastMonth",
            Self::ThisQuarter => "ThisQuarter",
            Self::LastQuarter => "LastQuarter",
            Self::ThisYear => "ThisYear",
            Self::LastYear => "LastYear",
            Self::YearToDate => "YearToDate",
        }
    }
}

/// What the caller confirmed before an order is placed: `submit_order` previews the order
/// again and submits only if Mouser's preview still matches all of it.
#[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ExpectedOrder {
    /// The preview's `CurrencyCode`, e.g. "EUR".
    pub currency_code: String,
    /// The preview's `OrderTotal`, in that currency.
    pub order_total: f64,
    /// Every line as (Mouser part number, quantity), in any order.
    pub lines: Vec<(String, i64)>,
}

/// Joins part numbers for `mouserPartNumber`: 1 to 10, each 3 to 40 characters, no `|`.
pub fn join_part_numbers(parts: &[String]) -> Result<String> {
    if parts.is_empty() || parts.len() > 10 {
        return Err(Error::InvalidArgument(format!(
            "between 1 and 10 part numbers per search, got {}",
            parts.len()
        )));
    }
    for p in parts {
        let n = p.trim().chars().count();
        if !(3..=40).contains(&n) || p.contains('|') {
            return Err(Error::InvalidArgument(format!(
                "part number {p:?}: 3 to 40 characters, without '|'"
            )));
        }
    }
    Ok(parts.iter().map(|p| p.trim()).collect::<Vec<_>>().join("|"))
}

/// Mouser's order-history dates are m/d/yyyy (its example: `1/30/2019`). Accepts ISO
/// `yyyy-mm-dd` or that form, and rejects dates that do not exist.
pub fn mouser_date(d: &str) -> Result<String> {
    use time::macros::format_description;
    let iso = format_description!("[year]-[month]-[day]");
    let us = format_description!("[month padding:none]/[day padding:none]/[year]");
    let us_padded = format_description!("[month]/[day]/[year]");
    let date = time::Date::parse(d.trim(), &iso)
        .or_else(|_| time::Date::parse(d.trim(), &us))
        .or_else(|_| time::Date::parse(d.trim(), &us_padded))
        .map_err(|_| Error::InvalidArgument(format!("date {d:?}: use yyyy-mm-dd or mm/dd/yyyy")))?;
    date.format(&us)
        .map_err(|e| Error::InvalidArgument(format!("date {d:?}: {e}")))
}

fn check_records(records: Option<i64>) -> Result<()> {
    match records {
        Some(r) if !(1..=50).contains(&r) => Err(Error::InvalidArgument(format!(
            "records: 1 to 50 per call (Mouser's maximum), got {r}"
        ))),
        _ => Ok(()),
    }
}

fn require(s: &str, what: &str) -> Result<()> {
    if s.trim().is_empty() {
        Err(Error::InvalidArgument(format!("{what} is required")))
    } else {
        Ok(())
    }
}

/// Rolling one-minute window of call times.
struct Limiter {
    per_minute: u32,
    calls: Mutex<VecDeque<Instant>>,
}

impl Limiter {
    async fn acquire(&self) {
        if self.per_minute == 0 {
            return;
        }
        loop {
            let wait = {
                let mut q = self.calls.lock().await;
                let now = Instant::now();
                while q
                    .front()
                    .is_some_and(|t| now.duration_since(*t) >= Duration::from_secs(60))
                {
                    q.pop_front();
                }
                if (q.len() as u32) < self.per_minute {
                    q.push_back(now);
                    return;
                }
                Duration::from_secs(60) - now.duration_since(*q.front().expect("non-empty"))
            };
            tracing::info!(
                wait_ms = wait.as_millis() as u64,
                "client-side rate limit: waiting"
            );
            tokio::time::sleep(wait).await;
        }
    }
}

pub struct Client {
    http: reqwest::Client,
    cfg: Config,
    limiter: Limiter,
}

type Query<'a> = Vec<(&'a str, String)>;

fn opt<'a>(q: &mut Query<'a>, name: &'a str, v: Option<&str>) {
    if let Some(v) = v {
        q.push((name, v.to_string()));
    }
}

impl Client {
    pub fn new(cfg: Config) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(cfg.timeout)
            .user_agent(concat!("mouser-rs/", env!("CARGO_PKG_VERSION")))
            .build()?;
        let limiter = Limiter {
            per_minute: cfg.max_calls_per_minute,
            calls: Mutex::new(VecDeque::new()),
        };
        Ok(Self { http, cfg, limiter })
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    pub fn has_key(&self, kind: KeyKind) -> bool {
        match kind {
            KeyKind::Search => self.cfg.search_key.is_some(),
            KeyKind::Order => self.cfg.order_key.is_some(),
        }
    }

    async fn call<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        key: KeyKind,
        mut query: Query<'_>,
        body: Option<&(impl Serialize + ?Sized)>,
    ) -> Result<T> {
        let k = match key {
            KeyKind::Search => self.cfg.search_key.as_ref(),
            KeyKind::Order => self.cfg.order_key.as_ref(),
        }
        .ok_or(Error::MissingKey(key))?;
        query.push(("apiKey", k.expose().to_string()));
        self.limiter.acquire().await;
        tracing::debug!(%method, path, "Mouser API call");
        let bodiless_write = body.is_none() && method != reqwest::Method::GET;
        let mut req = self
            .http
            .request(
                method,
                format!("{}{path}", self.cfg.base_url.trim_end_matches('/')),
            )
            .query(&query)
            .header("Accept", "application/json");
        if let Some(b) = body {
            req = req.json(b);
        } else if bodiless_write {
            // Mouser's CDN answers a POST without a body with 411 Length Required, and hyper
            // leaves the header off an empty body: state the length.
            req = req
                .header(reqwest::header::CONTENT_LENGTH, "0")
                .body(Vec::<u8>::new());
        }
        let resp = req.send().await?;
        let status = resp.status();
        // An error page may echo the request; the key must not travel further than this.
        let text = resp.text().await?.replace(k.expose(), "***");
        if status.as_u16() == 429 {
            return Err(Error::RateLimited(truncate(&text)));
        }
        if !status.is_success() {
            return Err(Error::Http {
                status: status.as_u16(),
                body: truncate(&text),
            });
        }
        let v: Value = serde_json::from_str(&text)
            .map_err(|e| Error::Decode(format!("{e}: {}", truncate(&text))))?;
        check_errors(&v)?;
        serde_json::from_value(v).map_err(|e| Error::Decode(e.to_string()))
    }

    // ── Search API (Search API key) ─────────────────────────────────────────────────────

    /// `POST /api/v1/search/keyword` — SearchApi_SearchByKeyword: up to 50 parts per call.
    pub async fn search_by_keyword(
        &self,
        r: &SearchByKeywordRequest,
    ) -> Result<SearchResponseRoot> {
        require(&r.keyword, "keyword")?;
        check_records(r.records)?;
        let body = SearchByKeywordRequestRoot {
            search_by_keyword_request: Some(r.clone()),
        };
        self.call(
            reqwest::Method::POST,
            "/v1/search/keyword",
            KeyKind::Search,
            vec![],
            Some(&body),
        )
        .await
    }

    /// `POST /api/v2/search/keywordandmanufacturer` — keyword search within one manufacturer
    /// (its name as `list_manufacturers` returns it).
    pub async fn search_by_keyword_and_manufacturer(
        &self,
        r: &SearchByKeywordMfrNameRequest,
    ) -> Result<SearchResponseRoot> {
        require(&r.keyword, "keyword")?;
        check_records(r.records)?;
        let body = SearchByKeywordMfrNameRequestRoot {
            search_by_keyword_mfr_name_request: Some(r.clone()),
        };
        self.call(
            reqwest::Method::POST,
            "/v2/search/keywordandmanufacturer",
            KeyKind::Search,
            vec![],
            Some(&body),
        )
        .await
    }

    /// `POST /api/v1/search/partnumber` — SearchApi_SearchByPartNumber: up to 10 part numbers
    /// joined by `|` (see [`join_part_numbers`]). Matches Mouser and manufacturer part numbers.
    pub async fn search_by_part_number(
        &self,
        r: &SearchByPartRequest,
    ) -> Result<SearchResponseRoot> {
        check_part_field(r.mouser_part_number.as_deref())?;
        let body = SearchByPartRequestRoot {
            search_by_part_request: Some(r.clone()),
        };
        self.call(
            reqwest::Method::POST,
            "/v1/search/partnumber",
            KeyKind::Search,
            vec![],
            Some(&body),
        )
        .await
    }

    /// `POST /api/v2/search/partnumberandmanufacturer` — part-number search within one
    /// manufacturer.
    pub async fn search_by_part_number_and_manufacturer(
        &self,
        r: &SearchByPartMfrNameRequest,
    ) -> Result<SearchResponseRoot> {
        check_part_field(r.mouser_part_number.as_deref())?;
        let body = SearchByPartMfrNameRequestRoot {
            search_by_part_mfr_name_request: Some(r.clone()),
        };
        self.call(
            reqwest::Method::POST,
            "/v2/search/partnumberandmanufacturer",
            KeyKind::Search,
            vec![],
            Some(&body),
        )
        .await
    }

    /// `GET /api/v2/search/manufacturerlist` — every manufacturer name Mouser carries.
    pub async fn manufacturer_list(&self) -> Result<MouserManufacturersNameRoot> {
        self.call(
            reqwest::Method::GET,
            "/v2/search/manufacturerlist",
            KeyKind::Search,
            vec![],
            None::<&()>,
        )
        .await
    }

    /// `POST /api/v1/search/keywordandmanufacturer` — deprecated by Mouser in favour of
    /// [`Self::search_by_keyword_and_manufacturer`]; takes a manufacturer ID. Untyped reply.
    pub async fn search_by_keyword_and_manufacturer_id_v1(
        &self,
        r: &SearchByKeywordMfrRequest,
    ) -> Result<Value> {
        require(&r.keyword, "keyword")?;
        check_records(r.records)?;
        let body = SearchByKeywordMfrRequestRoot {
            search_by_keyword_mfr_request: Some(r.clone()),
        };
        self.call(
            reqwest::Method::POST,
            "/v1/search/keywordandmanufacturer",
            KeyKind::Search,
            vec![],
            Some(&body),
        )
        .await
    }

    /// `POST /api/v1/search/partnumberandmanufacturer` — deprecated by Mouser in favour of
    /// [`Self::search_by_part_number_and_manufacturer`]; takes a manufacturer ID.
    pub async fn search_by_part_number_and_manufacturer_id_v1(
        &self,
        r: &SearchByPartMfrRequest,
    ) -> Result<Value> {
        check_part_field(r.mouser_part_number.as_deref())?;
        let body = SearchByPartMfrRequestRoot {
            search_by_part_mfr_request: Some(r.clone()),
        };
        self.call(
            reqwest::Method::POST,
            "/v1/search/partnumberandmanufacturer",
            KeyKind::Search,
            vec![],
            Some(&body),
        )
        .await
    }

    /// `GET /api/v1/search/manufacturerlist` — deprecated by Mouser; names and IDs.
    pub async fn manufacturer_list_with_ids_v1(&self) -> Result<Value> {
        self.call(
            reqwest::Method::GET,
            "/v1/search/manufacturerlist",
            KeyKind::Search,
            vec![],
            None::<&()>,
        )
        .await
    }

    // ── Cart (Cart and Order API key) ───────────────────────────────────────────────────

    /// `GET /api/v1/cart` — MouserCart_GetCart.
    pub async fn get_cart(
        &self,
        cart_key: &str,
        country: Option<&str>,
        currency: Option<&str>,
    ) -> Result<CartResponseRoot> {
        require(cart_key, "cartKey")?;
        let mut q = vec![("cartKey", cart_key.to_string())];
        opt(&mut q, "countryCode", country);
        opt(&mut q, "currencyCode", currency);
        self.call(
            reqwest::Method::GET,
            "/v1/cart",
            KeyKind::Order,
            q,
            None::<&()>,
        )
        .await
    }

    /// `POST /api/v1/cart` — MouserCart_UpdateCart: replaces the whole cart. Parts not in the
    /// request are removed. Without a `CartKey`, Mouser creates a new cart.
    pub async fn update_cart(
        &self,
        r: &CartItemRequestRoot,
        country: Option<&str>,
        currency: Option<&str>,
    ) -> Result<CartResponseRoot> {
        self.cart_post("/v1/cart", r, country, currency).await
    }

    /// `POST /api/v1/cart/items/insert` — MouserCart_AddCartItems.
    pub async fn add_cart_items(
        &self,
        r: &CartItemRequestRoot,
        country: Option<&str>,
        currency: Option<&str>,
    ) -> Result<CartResponseRoot> {
        self.cart_post("/v1/cart/items/insert", r, country, currency)
            .await
    }

    /// `POST /api/v1/cart/items/update` — MouserCart_UpdateCartItems: only the items sent.
    pub async fn update_cart_items(
        &self,
        r: &CartItemRequestRoot,
        country: Option<&str>,
        currency: Option<&str>,
    ) -> Result<CartResponseRoot> {
        self.cart_post("/v1/cart/items/update", r, country, currency)
            .await
    }

    async fn cart_post(
        &self,
        path: &str,
        r: &CartItemRequestRoot,
        country: Option<&str>,
        currency: Option<&str>,
    ) -> Result<CartResponseRoot> {
        let items = r.cart_items.as_deref().unwrap_or(&[]);
        if items.is_empty() && path != "/v1/cart" {
            return Err(Error::InvalidArgument("CartItems is empty".into()));
        }
        for i in items {
            require(&i.mouser_part_number, "MouserPartNumber")?;
            if i.quantity < 0 {
                return Err(Error::InvalidArgument(format!(
                    "{}: negative quantity",
                    i.mouser_part_number
                )));
            }
        }
        let mut q = vec![];
        opt(&mut q, "countryCode", country);
        opt(&mut q, "currencyCode", currency);
        self.call(reqwest::Method::POST, path, KeyKind::Order, q, Some(r))
            .await
    }

    /// `POST /api/v1/cart/item/remove` — MouserCart_RemoveCartItem.
    pub async fn remove_cart_item(
        &self,
        cart_key: &str,
        mouser_part_number: &str,
        country: Option<&str>,
        currency: Option<&str>,
    ) -> Result<CartResponseRoot> {
        require(cart_key, "cartKey")?;
        require(mouser_part_number, "mouserPartNumber")?;
        let mut q = vec![
            ("cartKey", cart_key.to_string()),
            ("mouserPartNumber", mouser_part_number.to_string()),
        ];
        opt(&mut q, "countryCode", country);
        opt(&mut q, "currencyCode", currency);
        self.call(
            reqwest::Method::POST,
            "/v1/cart/item/remove",
            KeyKind::Order,
            q,
            None::<&()>,
        )
        .await
    }

    /// `POST /api/v1/cart/insert/schedule` — MouserCart_InsertScheduleCartItems.
    pub async fn insert_schedule_cart_items(
        &self,
        r: &ScheduleCartItemsRequestRoot,
    ) -> Result<CartResponseRoot> {
        self.schedule("/v1/cart/insert/schedule", r).await
    }

    /// `POST /api/v1/cart/update/schedule` — MouserCart_UpdateScheduleCartItems.
    pub async fn update_schedule_cart_items(
        &self,
        r: &ScheduleCartItemsRequestRoot,
    ) -> Result<CartResponseRoot> {
        self.schedule("/v1/cart/update/schedule", r).await
    }

    async fn schedule(
        &self,
        path: &str,
        r: &ScheduleCartItemsRequestRoot,
    ) -> Result<CartResponseRoot> {
        require(r.cart_key.as_deref().unwrap_or(""), "CartKey")?;
        self.call(reqwest::Method::POST, path, KeyKind::Order, vec![], Some(r))
            .await
    }

    /// `POST /api/v1/cart/deleteall/schedule` — MouserCart_DeleteAllScheduleCartItems.
    pub async fn delete_all_schedule_cart_items(&self, cart_key: &str) -> Result<CartResponseRoot> {
        require(cart_key, "cartKey")?;
        let q = vec![("cartKey", cart_key.to_string())];
        self.call(
            reqwest::Method::POST,
            "/v1/cart/deleteall/schedule",
            KeyKind::Order,
            q,
            None::<&()>,
        )
        .await
    }

    // ── Order history (Cart and Order API key) ──────────────────────────────────────────

    /// `GET /api/v1/orderhistory/ByDateFilter` — MouserOrderHistory_OrderHistory.
    pub async fn order_history_by_date_filter(
        &self,
        filter: DateFilter,
    ) -> Result<OrderHistoryResponseRoot> {
        let q = vec![("dateFilter", filter.as_str().to_string())];
        self.call(
            reqwest::Method::GET,
            "/v1/orderhistory/ByDateFilter",
            KeyKind::Order,
            q,
            None::<&()>,
        )
        .await
    }

    /// `GET /api/v1/orderhistory/ByDateRange`. Dates as yyyy-mm-dd or mm/dd/yyyy.
    pub async fn order_history_by_date_range(
        &self,
        start: &str,
        end: &str,
    ) -> Result<OrderHistoryResponseRoot> {
        let q = vec![
            ("startDate", mouser_date(start)?),
            ("endDate", mouser_date(end)?),
        ];
        self.call(
            reqwest::Method::GET,
            "/v1/orderhistory/ByDateRange",
            KeyKind::Order,
            q,
            None::<&()>,
        )
        .await
    }

    /// `GET /api/v1/orderhistory/salesOrderNumber` — MouserOrderHistory_GetOrderAsync.
    pub async fn order_by_sales_order_number(
        &self,
        sales_order_number: &str,
    ) -> Result<OrderDetail> {
        require(sales_order_number, "salesOrderNumber")?;
        let q = vec![("salesOrderNumber", sales_order_number.to_string())];
        self.call(
            reqwest::Method::GET,
            "/v1/orderhistory/salesOrderNumber",
            KeyKind::Order,
            q,
            None::<&()>,
        )
        .await
    }

    /// `GET /api/v1/orderhistory/webOrderNumber` — MouserOrderHistory_GetWebOrderAsync.
    pub async fn order_by_web_order_number(&self, web_order_number: i64) -> Result<OrderDetail> {
        let q = vec![("webOrderNumber", web_order_number.to_string())];
        self.call(
            reqwest::Method::GET,
            "/v1/orderhistory/webOrderNumber",
            KeyKind::Order,
            q,
            None::<&()>,
        )
        .await
    }

    // ── Order (Cart and Order API key) ──────────────────────────────────────────────────

    /// `POST /api/v1/order/options/query` — Order_OptionsQuery: shipping and payment methods,
    /// tax certificates, VAT accounts and saved cards for the account.
    pub async fn order_options_query(
        &self,
        r: &OrderInializeRequestRoot,
    ) -> Result<OrderInitializeResponseRoot> {
        self.call(
            reqwest::Method::POST,
            "/v1/order/options/query",
            KeyKind::Order,
            vec![],
            Some(r),
        )
        .await
    }

    /// `GET /api/v1/order/currencies` — Order_GetCurrencies.
    pub async fn order_currencies(
        &self,
        shipping_country: Option<&str>,
    ) -> Result<OrderCurrencyRoot> {
        let mut q = vec![];
        opt(&mut q, "shippingCountryCode", shipping_country);
        self.call(
            reqwest::Method::GET,
            "/v1/order/currencies",
            KeyKind::Order,
            q,
            None::<&()>,
        )
        .await
    }

    /// `GET /api/v1/order/countries` — Order_GetCountries, with their states and provinces.
    pub async fn order_countries(&self, country: Option<&str>) -> Result<CountryRoot> {
        let mut q = vec![];
        opt(&mut q, "countryCode", country);
        self.call(
            reqwest::Method::GET,
            "/v1/order/countries",
            KeyKind::Order,
            q,
            None::<&()>,
        )
        .await
    }

    /// `GET /api/v1/order/{orderNumber}` — Order_GetOrder.
    pub async fn get_order(&self, order_number: i64) -> Result<Order> {
        self.call(
            reqwest::Method::GET,
            &format!("/v1/order/{order_number}"),
            KeyKind::Order,
            vec![],
            None::<&()>,
        )
        .await
    }

    /// `POST /api/v1/order` with `SubmitOrder` false — Order_Create as a summary only: prices,
    /// shipping and tax of the order as it would be placed. Never places an order.
    pub async fn preview_order(&self, r: &OrderRequestRootType) -> Result<OrderRoot> {
        let body = with_submit(r, false)?;
        self.call(
            reqwest::Method::POST,
            "/v1/order",
            KeyKind::Order,
            vec![],
            Some(&body),
        )
        .await
    }

    /// `POST /api/v1/order` with `SubmitOrder` true — **places an order**. Previews first and
    /// submits only if the preview matches `expected` (currency, total, every line) and the
    /// total is within [`Config::max_order_total`].
    pub async fn submit_order(
        &self,
        r: &OrderRequestRootType,
        expected: &ExpectedOrder,
    ) -> Result<OrderRoot> {
        let preview = self.preview_order(r).await?;
        self.verify_order(&preview, expected)?;
        let body = with_submit(r, true)?;
        tracing::warn!(currency = %expected.currency_code, total = expected.order_total, "submitting a Mouser order");
        self.call(
            reqwest::Method::POST,
            "/v1/order",
            KeyKind::Order,
            vec![],
            Some(&body),
        )
        .await
    }

    /// `POST /api/v1/order/CreateFromOrder` with `SubmitOrder` false: summary of re-ordering
    /// an earlier order.
    pub async fn preview_order_from_order(
        &self,
        order_number: i64,
        r: &OrderRequestRootType,
        country: Option<&str>,
        currency: Option<&str>,
    ) -> Result<OrderRoot> {
        let body = with_submit(r, false)?;
        let q = from_order_query(order_number, country, currency);
        self.call(
            reqwest::Method::POST,
            "/v1/order/CreateFromOrder",
            KeyKind::Order,
            q,
            Some(&body),
        )
        .await
    }

    /// `POST /api/v1/order/CreateFromOrder` with `SubmitOrder` true — **places an order**,
    /// with the same preview check as [`Self::submit_order`].
    pub async fn submit_order_from_order(
        &self,
        order_number: i64,
        r: &OrderRequestRootType,
        country: Option<&str>,
        currency: Option<&str>,
        expected: &ExpectedOrder,
    ) -> Result<OrderRoot> {
        let preview = self
            .preview_order_from_order(order_number, r, country, currency)
            .await?;
        self.verify_order(&preview, expected)?;
        let body = with_submit(r, true)?;
        let q = from_order_query(order_number, country, currency);
        tracing::warn!(
            order_number,
            total = expected.order_total,
            "submitting a Mouser re-order"
        );
        self.call(
            reqwest::Method::POST,
            "/v1/order/CreateFromOrder",
            KeyKind::Order,
            q,
            Some(&body),
        )
        .await
    }

    /// `POST /api/v1/order/item/CreateCartFromOrder` — a new cart holding an earlier order's
    /// items. Untyped reply.
    pub async fn create_cart_from_order(
        &self,
        order_number: i64,
        country: Option<&str>,
        currency: Option<&str>,
    ) -> Result<Value> {
        let q = from_order_query(order_number, country, currency);
        self.call(
            reqwest::Method::POST,
            "/v1/order/item/CreateCartFromOrder",
            KeyKind::Order,
            q,
            None::<&()>,
        )
        .await
    }

    /// The checks [`Self::submit_order`] runs on its fresh preview: a ceiling is configured,
    /// and currency, total and every line match `expected`, the total within the ceiling.
    /// Public so a caller can run them on a preview before asking a user to confirm it.
    pub fn verify_order(&self, preview: &OrderRoot, expected: &ExpectedOrder) -> Result<()> {
        let max = self.cfg.max_order_total.ok_or_else(|| {
            Error::OrderMismatch("no order ceiling configured (MOUSER_MAX_ORDER_TOTAL)".into())
        })?;
        let o = preview
            .order
            .as_ref()
            .ok_or_else(|| Error::Decode("preview without Order".into()))?;
        let currency = o.currency_code.as_deref().unwrap_or("");
        let total = o
            .order_total
            .ok_or_else(|| Error::Decode("preview without OrderTotal".into()))?;
        if !currency.eq_ignore_ascii_case(&expected.currency_code) {
            return Err(Error::OrderMismatch(format!(
                "currency is {currency}, confirmed {}",
                expected.currency_code
            )));
        }
        if (total - expected.order_total).abs() > 0.005 {
            return Err(Error::OrderMismatch(format!(
                "total is {total} {currency}, confirmed {}",
                expected.order_total
            )));
        }
        if total > max {
            return Err(Error::OrderMismatch(format!(
                "total {total} {currency} is above the ceiling {max}"
            )));
        }
        let mut got: Vec<(String, i64)> = o
            .order_lines
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(|l| {
                (
                    l.mouser_part_number.clone().unwrap_or_default(),
                    l.quantity.unwrap_or(0),
                )
            })
            .collect();
        let mut want = expected.lines.clone();
        got.sort();
        want.sort();
        if got != want {
            return Err(Error::OrderMismatch(format!(
                "lines are {got:?}, confirmed {want:?}"
            )));
        }
        Ok(())
    }
}

fn from_order_query<'a>(
    order_number: i64,
    country: Option<&str>,
    currency: Option<&str>,
) -> Query<'a> {
    let mut q = vec![("orderNumber", order_number.to_string())];
    opt(&mut q, "countryCode", country);
    opt(&mut q, "currencyCode", currency);
    q
}

fn with_submit(r: &OrderRequestRootType, submit: bool) -> Result<OrderRequestRootType> {
    let mut body = r.clone();
    let o = body
        .order
        .as_mut()
        .ok_or_else(|| Error::InvalidArgument("Order is required".into()))?;
    require(&o.cart_key, "CartKey")?;
    require(&o.currency_code, "CurrencyCode")?;
    o.submit_order = Some(submit);
    Ok(body)
}

fn check_part_field(v: Option<&str>) -> Result<()> {
    let v = v.unwrap_or("");
    join_part_numbers(&v.split('|').map(str::to_string).collect::<Vec<_>>()).map(|_| ())
}

/// Mouser reports failures with HTTP 200 and an `Errors` list, at the top level or inside
/// `Order` / `OrderInitialize`.
fn check_errors(v: &Value) -> Result<()> {
    let lists = [
        v.get("Errors"),
        v.pointer("/Order/Errors"),
        v.pointer("/OrderInitialize/Errors"),
    ];
    for l in lists.into_iter().flatten() {
        if let Some(a) = l.as_array().filter(|a| !a.is_empty()) {
            let errs: Vec<ErrorEntity> =
                serde_json::from_value(Value::Array(a.clone())).unwrap_or_default();
            let rate = errs.iter().any(|e| {
                let c = e.code.as_deref().unwrap_or("").to_ascii_lowercase();
                let m = e.message.as_deref().unwrap_or("").to_ascii_lowercase();
                c.contains("toomanyrequests")
                    || c.contains("ratelimit")
                    || m.contains("too many requests")
            });
            if rate {
                return Err(Error::RateLimited(format!("{errs:?}")));
            }
            return Err(Error::Api(errs));
        }
    }
    Ok(())
}

fn truncate(s: &str) -> String {
    const MAX: usize = 2000;
    if s.len() <= MAX {
        s.to_string()
    } else {
        let mut end = MAX;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}
