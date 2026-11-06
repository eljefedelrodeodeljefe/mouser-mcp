//! Tool arguments. Search and BOM tools take snake_case arguments; cart and order bodies are
//! Mouser's own request types (PascalCase, as in Mouser's API documentation), so nothing is
//! lost in translation.

use mouser::models::{OrderAddress, OrderRequestType, PaymentTypeRequest, ShippingMethod};
use mouser::{DateFilter, SearchOption};
use schemars::JsonSchema;
use serde::Deserialize;

// ── Search ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KeywordParams {
    #[schemars(description = "Keyword(s), e.g. \"resistive power splitter 4.2 GHz\"")]
    pub keyword: String,
    #[schemars(description = "Parts to return, 1 to 50 (Mouser's maximum per call). Default 10")]
    pub records: Option<i64>,
    #[schemars(description = "Offset into the result set, for paging (startingRecord). Default 0")]
    pub starting_record: Option<i64>,
    #[schemars(description = "searchOptions: None (default), Rohs, InStock or RohsAndInStock")]
    pub search_options: Option<SearchOption>,
    #[schemars(
        description = "Prices with Mouser paying import duty and customs (mouserPaysCustomsAndDuties)"
    )]
    pub mouser_pays_customs_and_duties: Option<bool>,
    #[schemars(
        description = "Search in the language chosen at Search API sign-up (searchWithYourSignUpLanguage)"
    )]
    pub search_with_sign_up_language: Option<bool>,
    #[schemars(
        description = "Return Mouser's complete part records instead of the compact summary. Default false"
    )]
    pub full: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KeywordMfrParams {
    #[schemars(description = "Keyword(s)")]
    pub keyword: String,
    #[schemars(
        description = "Manufacturer name exactly as list_manufacturers returns it, e.g. \"Mini-Circuits\""
    )]
    pub manufacturer_name: String,
    #[schemars(description = "Parts per page, 1 to 50. Default 10")]
    pub records: Option<i64>,
    #[schemars(
        description = "Page of the result set, counted in `records` (pageNumber). Default 1"
    )]
    pub page_number: Option<i64>,
    #[schemars(description = "searchOptions: None (default), Rohs, InStock or RohsAndInStock")]
    pub search_options: Option<SearchOption>,
    #[schemars(description = "Prices with Mouser paying import duty and customs")]
    pub mouser_pays_customs_and_duties: Option<bool>,
    #[schemars(description = "Search in the language chosen at Search API sign-up")]
    pub search_with_sign_up_language: Option<bool>,
    #[schemars(description = "Return complete part records instead of the compact summary")]
    pub full: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PartNumberParams {
    #[schemars(
        description = "1 to 10 part numbers, Mouser (\"139-ZFRSC-42-S+\") or manufacturer (\"ZFRSC-42-S+\"), 3 to 40 characters each"
    )]
    pub part_numbers: Vec<String>,
    #[schemars(
        description = "Exact matches only (partSearchOptions Exact). Default false: also partial matches"
    )]
    pub exact: Option<bool>,
    #[schemars(description = "Prices with Mouser paying import duty and customs")]
    pub mouser_pays_customs_and_duties: Option<bool>,
    #[schemars(description = "Return complete part records instead of the compact summary")]
    pub full: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PartNumberMfrParams {
    #[schemars(description = "1 to 10 part numbers, 3 to 40 characters each")]
    pub part_numbers: Vec<String>,
    #[schemars(description = "Manufacturer name exactly as list_manufacturers returns it")]
    pub manufacturer_name: String,
    #[schemars(description = "Exact matches only. Default false")]
    pub exact: Option<bool>,
    #[schemars(description = "Prices with Mouser paying import duty and customs")]
    pub mouser_pays_customs_and_duties: Option<bool>,
    #[schemars(description = "Return complete part records instead of the compact summary")]
    pub full: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ManufacturersParams {
    #[schemars(
        description = "Only names containing this text (case-insensitive). The full list has thousands of names"
    )]
    pub contains: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BomLine {
    #[schemars(description = "Manufacturer or Mouser part number")]
    pub part_number: String,
    #[schemars(description = "Quantity needed")]
    pub quantity: i64,
    #[schemars(description = "Your reference for the line, e.g. \"splitter\" (returned as is)")]
    pub reference: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BomParams {
    #[schemars(description = "BOM lines; any number (searched 10 part numbers per call)")]
    pub lines: Vec<BomLine>,
    #[schemars(description = "Prices with Mouser paying import duty and customs")]
    pub mouser_pays_customs_and_duties: Option<bool>,
}

// ── Cart ────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CartParams {
    #[schemars(description = "CartKey (a GUID) of the cart")]
    pub cart_key: String,
    #[schemars(
        description = "countryCode (ISO 3166-1 alpha-2) for shipping rules. Default: the account's"
    )]
    pub country_code: Option<String>,
    #[schemars(
        description = "currencyCode (ISO 4217) to price the cart in. Default: the account's"
    )]
    pub currency_code: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CartItem {
    #[schemars(description = "Mouser part number, e.g. \"139-ZFRSC-42-S+\" (from a search)")]
    pub mouser_part_number: String,
    #[schemars(description = "Quantity; a multiple of the part's Mult and at least its Min")]
    pub quantity: i64,
    #[schemars(description = "Your part number for this line (CustomerPartNumber)")]
    pub customer_part_number: Option<String>,
    #[schemars(description = "PackagingChoice, for parts offered on reels or cut tape")]
    pub packaging_choice: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CartItemsParams {
    #[schemars(description = "CartKey of the cart. Omit to have Mouser create a new cart")]
    pub cart_key: Option<String>,
    #[schemars(description = "Items")]
    pub items: Vec<CartItem>,
    #[schemars(description = "MouserPaysCustomsAndDuties for the cart")]
    pub mouser_pays_customs_and_duties: Option<bool>,
    #[schemars(description = "countryCode for shipping rules")]
    pub country_code: Option<String>,
    #[schemars(description = "currencyCode to price the cart in")]
    pub currency_code: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveCartItemParams {
    #[schemars(description = "CartKey of the cart")]
    pub cart_key: String,
    #[schemars(description = "Mouser part number to remove")]
    pub mouser_part_number: String,
    #[schemars(description = "countryCode")]
    pub country_code: Option<String>,
    #[schemars(description = "currencyCode")]
    pub currency_code: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScheduledRelease {
    #[schemars(
        description = "Release date as Mouser's cart schedule takes it (the key of the release pair)"
    )]
    pub date: String,
    #[schemars(description = "Quantity released on that date")]
    pub quantity: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScheduleItem {
    #[schemars(description = "Mouser part number in the cart")]
    pub mouser_part_number: String,
    #[schemars(description = "Scheduled deliveries")]
    pub releases: Vec<ScheduledRelease>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScheduleParams {
    #[schemars(description = "CartKey of the cart")]
    pub cart_key: String,
    #[schemars(description = "Items and their scheduled deliveries")]
    pub items: Vec<ScheduleItem>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CartKeyParams {
    #[schemars(description = "CartKey of the cart")]
    pub cart_key: String,
}

// ── Order history ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DateFilterParams {
    #[schemars(description = "dateFilter, e.g. ThisMonth, LastQuarter, YearToDate, All")]
    pub date_filter: DateFilter,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DateRangeParams {
    #[schemars(description = "Start date, yyyy-mm-dd (or Mouser's mm/dd/yyyy)")]
    pub start_date: String,
    #[schemars(description = "End date, yyyy-mm-dd (or mm/dd/yyyy)")]
    pub end_date: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SalesOrderParams {
    #[schemars(description = "Mouser sales order number")]
    pub sales_order_number: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WebOrderParams {
    #[schemars(description = "Mouser web order number")]
    pub web_order_number: i64,
}

// ── Orders ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OrderOptionsParams {
    #[schemars(description = "CartKey of the cart to be ordered")]
    pub cart_key: Option<String>,
    #[schemars(description = "CurrencyCode")]
    pub currency_code: Option<String>,
    #[schemars(description = "ShippingAddress, if not the account's default")]
    pub shipping_address: Option<OrderAddress>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CurrenciesParams {
    #[schemars(description = "shippingCountryCode. Default: the account's shipping address")]
    pub shipping_country_code: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CountriesParams {
    #[schemars(description = "Only this countryCode")]
    pub country_code: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OrderNumberParams {
    #[schemars(description = "Mouser web order number")]
    pub order_number: i64,
}

/// The choices an order of a cart needs, as query_order_options lists them.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct OrderChoice {
    #[schemars(description = "CartKey of the cart to order")]
    pub cart_key: String,
    #[schemars(description = "CurrencyCode, e.g. \"EUR\"")]
    pub currency_code: String,
    #[schemars(description = "Shipping method Code from query_order_options (PrimaryShipping)")]
    pub shipping_code: Option<i64>,
    #[schemars(description = "Payment Method from query_order_options, e.g. 1081 for PayPal")]
    pub payment_method: Option<i64>,
}

impl OrderChoice {
    pub fn request(&self) -> OrderRequestType {
        OrderRequestType {
            cart_key: self.cart_key.clone(),
            currency_code: self.currency_code.clone(),
            primary_shipping: self.shipping_code.map(|c| ShippingMethod {
                code: Some(c),
                ..Default::default()
            }),
            payment: self.payment_method.map(|m| PaymentTypeRequest {
                method: m,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OrderParams {
    #[serde(flatten)]
    pub order: OrderChoice,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SubmitOrderParams {
    #[serde(flatten)]
    pub order: OrderChoice,
    #[schemars(
        description = "The OrderTotal of preview_order, in currency_code. Mouser's fresh preview must match it, and the user then confirms every line in a form"
    )]
    pub total: f64,
}

/// Re-ordering an earlier order takes the same short choices as ordering a cart: ship-to,
/// freight accounts and the rest come from the Mouser account, never from the model, because
/// the user's form shows only name, city and country of the address.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FromOrderParams {
    #[schemars(description = "Web order number of the earlier order")]
    pub order_number: i64,
    #[serde(flatten)]
    pub order: OrderChoice,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SubmitFromOrderParams {
    #[schemars(description = "Web order number of the earlier order")]
    pub order_number: i64,
    #[serde(flatten)]
    pub order: OrderChoice,
    #[schemars(
        description = "The OrderTotal of preview_order_from_order, in currency_code. The fresh preview must match it, and the user then confirms every line in a form"
    )]
    pub total: f64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CartFromOrderParams {
    #[schemars(description = "Web order number of the earlier order")]
    pub order_number: i64,
    #[schemars(description = "countryCode")]
    pub country_code: Option<String>,
    #[schemars(description = "currencyCode")]
    pub currency_code: Option<String>,
}
