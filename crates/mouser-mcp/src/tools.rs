//! Tool logic: plain functions over `mouser::Client`, returning JSON. No MCP types here.

use std::collections::BTreeMap;

use anyhow::Result;
use mouser::models::*;
use mouser::{Client, ExpectedOrder, PartSearchOption, SearchOption};
use rust_decimal::Decimal;
use serde_json::{Value, json};

use crate::params::*;

fn to_value<T: serde::Serialize>(v: T) -> Result<Value> {
    Ok(serde_json::to_value(v)?)
}

// ── Search ──────────────────────────────────────────────────────────────────────────────

/// The fields that decide a purchase; `full` returns Mouser's whole record instead.
pub fn compact_part(p: &MouserPart) -> Value {
    json!({
        "MouserPartNumber": p.mouser_part_number,
        "ManufacturerPartNumber": p.manufacturer_part_number,
        "Manufacturer": p.manufacturer,
        "Description": p.description,
        "AvailabilityInStock": p.availability_in_stock,
        "Availability": p.availability,
        "AvailabilityOnOrder": p.availability_on_order,
        "LeadTime": p.lead_time,
        "LifecycleStatus": p.lifecycle_status,
        "IsDiscontinued": p.is_discontinued,
        "SuggestedReplacement": p.suggested_replacement,
        "Min": p.min,
        "Mult": p.mult,
        "PriceBreaks": p.price_breaks,
        "ROHSStatus": p.rohs_status,
        "RestrictionMessage": p.restriction_message,
        "ProductDetailUrl": p.product_detail_url,
        "DataSheetUrl": p.data_sheet_url,
    })
}

fn search_out(r: SearchResponseRoot, full: Option<bool>) -> Result<Value> {
    let res = r.search_results.unwrap_or_default();
    let parts = res.parts.unwrap_or_default();
    let parts: Vec<Value> = if full.unwrap_or(false) {
        parts
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()?
    } else {
        parts.iter().map(compact_part).collect()
    };
    Ok(json!({ "NumberOfResult": res.number_of_result, "Parts": parts }))
}

fn records(r: Option<i64>) -> Option<i64> {
    Some(r.unwrap_or(10))
}

fn opt_str(o: Option<SearchOption>) -> Option<String> {
    o.map(|o| o.as_str().to_string())
}

fn part_opt(exact: Option<bool>) -> Option<String> {
    Some(
        if exact.unwrap_or(false) {
            PartSearchOption::Exact
        } else {
            PartSearchOption::None
        }
        .as_str()
        .to_string(),
    )
}

fn lang(b: Option<bool>) -> Option<String> {
    b.filter(|b| *b).map(|_| "true".to_string())
}

pub async fn search_by_keyword(c: &Client, p: &KeywordParams) -> Result<Value> {
    let r = c
        .search_by_keyword(&SearchByKeywordRequest {
            keyword: p.keyword.clone(),
            records: records(p.records),
            starting_record: p.starting_record,
            search_options: opt_str(p.search_options),
            search_with_your_sign_up_language: lang(p.search_with_sign_up_language),
            mouser_pays_customs_and_duties: p.mouser_pays_customs_and_duties,
        })
        .await?;
    search_out(r, p.full)
}

pub async fn search_by_keyword_and_manufacturer(c: &Client, p: &KeywordMfrParams) -> Result<Value> {
    let r = c
        .search_by_keyword_and_manufacturer(&SearchByKeywordMfrNameRequest {
            manufacturer_name: Some(p.manufacturer_name.clone()),
            keyword: p.keyword.clone(),
            records: records(p.records),
            page_number: p.page_number,
            search_options: opt_str(p.search_options),
            search_with_your_sign_up_language: lang(p.search_with_sign_up_language),
            mouser_pays_customs_and_duties: p.mouser_pays_customs_and_duties,
        })
        .await?;
    search_out(r, p.full)
}

pub async fn search_by_part_number(c: &Client, p: &PartNumberParams) -> Result<Value> {
    let r = c
        .search_by_part_number(&SearchByPartRequest {
            mouser_part_number: Some(mouser::join_part_numbers(&p.part_numbers)?),
            part_search_options: part_opt(p.exact),
            mouser_pays_customs_and_duties: p.mouser_pays_customs_and_duties,
        })
        .await?;
    search_out(r, p.full)
}

pub async fn search_by_part_number_and_manufacturer(
    c: &Client,
    p: &PartNumberMfrParams,
) -> Result<Value> {
    let r = c
        .search_by_part_number_and_manufacturer(&SearchByPartMfrNameRequest {
            manufacturer_name: Some(p.manufacturer_name.clone()),
            mouser_part_number: Some(mouser::join_part_numbers(&p.part_numbers)?),
            part_search_options: part_opt(p.exact),
            mouser_pays_customs_and_duties: p.mouser_pays_customs_and_duties,
        })
        .await?;
    search_out(r, p.full)
}

pub async fn list_manufacturers(c: &Client, p: &ManufacturersParams) -> Result<Value> {
    let r = c.manufacturer_list().await?;
    let all: Vec<String> = r
        .mouser_manufacturer_list
        .and_then(|l| l.manufacturer_list)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|m| m.manufacturer_name)
        .collect();
    let total = all.len();
    let names: Vec<String> = match p.contains.as_deref().map(str::to_lowercase) {
        Some(f) => all
            .into_iter()
            .filter(|n| n.to_lowercase().contains(&f))
            .collect(),
        None => all,
    };
    Ok(json!({ "Total": total, "Matching": names.len(), "ManufacturerNames": names }))
}

// ── BOM pricing ─────────────────────────────────────────────────────────────────────────

/// A Mouser price string ("87,10 €", "$91.00") as an exact amount in `currency`. The amount is
/// read by `rusty_money` with that currency's own separators, after removing its symbol;
/// anything else (another symbol, another locale's separators) gives `None` rather than a
/// guess.
pub fn parse_price(raw: &str, currency: &str) -> Option<Decimal> {
    let cur = rusty_money::iso::find(currency)?;
    let amount = raw.replace(cur.symbol, "");
    let amount = amount.trim_matches(|c: char| c.is_whitespace());
    rusty_money::Money::from_str(amount, cur)
        .ok()
        .map(|m| *m.amount())
}

/// Mouser's counts ("283", "1") as integers; anything else is unknown, not zero.
fn count(s: &Option<String>) -> Option<i64> {
    s.as_deref().and_then(|v| v.trim().parse().ok())
}

struct Price {
    unit: Decimal,
    break_quantity: i64,
    currency: String,
}

/// Unit price at `qty`: the largest price break whose quantity is at most `qty` (the first
/// break below the minimum). `Err(raw)` if that break's price cannot be read.
fn price_at(p: &MouserPart, qty: i64) -> Option<Result<Price, String>> {
    let mut breaks: Vec<&Pricebreak> = p.price_breaks.as_deref().unwrap_or(&[]).iter().collect();
    breaks.sort_by_key(|b| b.quantity.unwrap_or(0));
    let b = breaks
        .iter()
        .rev()
        .find(|b| b.quantity.unwrap_or(0) <= qty)
        .or(breaks.first())?;
    let raw = b.price.clone().unwrap_or_default();
    let currency = b.currency.clone().unwrap_or_default();
    Some(match parse_price(&raw, &currency) {
        Some(unit) => Ok(Price {
            unit,
            break_quantity: b.quantity.unwrap_or(0),
            currency,
        }),
        None => Err(format!("{raw} ({currency})")),
    })
}

fn matches(p: &MouserPart, part_number: &str) -> bool {
    let eq = |s: &Option<String>| {
        s.as_deref()
            .is_some_and(|v| v.eq_ignore_ascii_case(part_number.trim()))
    };
    eq(&p.mouser_part_number) || eq(&p.manufacturer_part_number)
}

pub async fn price_bom(c: &Client, p: &BomParams) -> Result<Value> {
    anyhow::ensure!(!p.lines.is_empty(), "the BOM has no lines");
    for l in &p.lines {
        anyhow::ensure!(
            l.quantity > 0,
            "{}: quantity must be positive",
            l.part_number
        );
    }
    let mut unique: Vec<String> = Vec::new();
    for l in &p.lines {
        if !unique
            .iter()
            .any(|u| u.eq_ignore_ascii_case(l.part_number.trim()))
        {
            unique.push(l.part_number.trim().to_string());
        }
    }
    let mut found: Vec<MouserPart> = Vec::new();
    for chunk in unique.chunks(10) {
        let r = c
            .search_by_part_number(&SearchByPartRequest {
                mouser_part_number: Some(mouser::join_part_numbers(chunk)?),
                part_search_options: Some(PartSearchOption::Exact.as_str().into()),
                mouser_pays_customs_and_duties: p.mouser_pays_customs_and_duties,
            })
            .await?;
        found.extend(r.search_results.and_then(|s| s.parts).unwrap_or_default());
    }
    let mut totals: BTreeMap<String, Decimal> = BTreeMap::new();
    let mut out = Vec::new();
    for l in &p.lines {
        let qty = Decimal::from(l.quantity);
        let cands: Vec<&MouserPart> = found
            .iter()
            .filter(|f| matches(f, &l.part_number))
            .collect();
        // prefer an offer known to cover the quantity from stock, then the lowest extended price
        let best = cands.iter().min_by_key(|x| {
            let short = count(&x.availability_in_stock).is_none_or(|s| s < l.quantity);
            let ext = match price_at(x, l.quantity) {
                Some(Ok(pr)) => pr.unit * qty,
                _ => Decimal::MAX,
            };
            (short, ext)
        });
        let line = match best {
            None => json!({"reference": l.reference, "part_number": l.part_number,
                "quantity": l.quantity, "status": "not_found"}),
            Some(b) => {
                let stock = count(&b.availability_in_stock);
                let mut notes = Vec::new();
                let min = count(&b.min).unwrap_or(1);
                let mult = count(&b.mult).unwrap_or(1).max(1);
                if l.quantity < min {
                    notes.push(format!("below the minimum order quantity {min}"));
                }
                if l.quantity % mult != 0 {
                    notes.push(format!("not a multiple of {mult}"));
                }
                if b.is_discontinued
                    .as_deref()
                    .is_some_and(|d| d.eq_ignore_ascii_case("true"))
                {
                    notes.push("discontinued".into());
                }
                let price = match price_at(b, l.quantity) {
                    Some(Ok(pr)) => Some(pr),
                    Some(Err(raw)) => {
                        notes.push(format!("price not readable: {raw}"));
                        None
                    }
                    None => {
                        notes.push("no price breaks".into());
                        None
                    }
                };
                let extended = price.as_ref().map(|pr| pr.unit * qty);
                if let (Some(e), Some(pr)) = (extended, price.as_ref()) {
                    *totals.entry(pr.currency.clone()).or_default() += e;
                }
                let status = match stock {
                    Some(s) if s >= l.quantity => "in_stock",
                    Some(_) => "short",
                    None => "stock_unknown",
                };
                json!({
                    "reference": l.reference,
                    "part_number": l.part_number,
                    "quantity": l.quantity,
                    "status": status,
                    "MouserPartNumber": b.mouser_part_number,
                    "Manufacturer": b.manufacturer,
                    "ManufacturerPartNumber": b.manufacturer_part_number,
                    "Description": b.description,
                    "in_stock": stock,
                    "AvailabilityInStock": b.availability_in_stock,
                    "AvailabilityOnOrder": b.availability_on_order,
                    "LeadTime": b.lead_time,
                    "unit_price": price.as_ref().map(|pr| pr.unit.to_string()),
                    "price_break_quantity": price.as_ref().map(|pr| pr.break_quantity),
                    "currency": price.as_ref().map(|pr| pr.currency.clone()),
                    "extended_price": extended.map(|e| e.to_string()),
                    "notes": notes,
                    "other_offers": cands.len() - 1,
                    "ProductDetailUrl": b.product_detail_url,
                })
            }
        };
        out.push(line);
    }
    let totals: BTreeMap<String, String> = totals
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect();
    Ok(json!({ "lines": out, "totals_by_currency": totals }))
}

// ── Cart ────────────────────────────────────────────────────────────────────────────────

fn cart_request(p: &CartItemsParams) -> CartItemRequestRoot {
    CartItemRequestRoot {
        cart_key: p.cart_key.clone(),
        mouser_pays_customs_and_duties: p.mouser_pays_customs_and_duties,
        cart_items: Some(
            p.items
                .iter()
                .map(|i| CartItemRequest {
                    mouser_part_number: i.mouser_part_number.clone(),
                    quantity: i.quantity,
                    customer_part_number: i.customer_part_number.clone(),
                    packaging_choice: i.packaging_choice.clone(),
                })
                .collect(),
        ),
    }
}

pub async fn get_cart(c: &Client, p: &CartParams) -> Result<Value> {
    to_value(
        c.get_cart(
            &p.cart_key,
            p.country_code.as_deref(),
            p.currency_code.as_deref(),
        )
        .await?,
    )
}

pub async fn update_cart(c: &Client, p: &CartItemsParams) -> Result<Value> {
    to_value(
        c.update_cart(
            &cart_request(p),
            p.country_code.as_deref(),
            p.currency_code.as_deref(),
        )
        .await?,
    )
}

/// Adds to a cart without losing what is already in it.
///
/// Mouser's AddCartItems (`cart/items/insert`) has been seen replacing a cart's contents
/// instead of adding to them. With a cart key, the cart is read, the new items are merged in
/// (quantities of a part already in the cart add up), the whole cart is written with
/// UpdateCart, and the result is read back and checked line by line. Without a cart key, the
/// insert creates a new cart, which has nothing to lose.
pub async fn add_cart_items(c: &Client, p: &CartItemsParams) -> Result<Value> {
    let (country, currency) = (p.country_code.as_deref(), p.currency_code.as_deref());
    let Some(key) = p.cart_key.as_deref() else {
        return to_value(
            c.add_cart_items(&cart_request(p), country, currency)
                .await?,
        );
    };
    let before = c.get_cart(key, country, currency).await?;
    let merged = merge_cart(&before, &p.items);
    c.update_cart(
        &CartItemRequestRoot {
            cart_key: Some(key.into()),
            mouser_pays_customs_and_duties: p.mouser_pays_customs_and_duties,
            cart_items: Some(merged.clone()),
        },
        country,
        currency,
    )
    .await?;
    let after = c.get_cart(key, country, currency).await?;
    check_cart(&after, &merged)?;
    to_value(after)
}

fn same_part(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

/// The cart's lines plus `items`; a part already in the cart gets the quantities added and
/// keeps its line reference unless the new item brings one.
fn merge_cart(cart: &CartResponseRoot, items: &[CartItem]) -> Vec<CartItemRequest> {
    let mut out: Vec<CartItemRequest> = cart
        .cart_items
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter_map(|l| {
            Some(CartItemRequest {
                mouser_part_number: l.mouser_part_number.clone()?,
                quantity: l.quantity.unwrap_or(0),
                customer_part_number: l
                    .cart_item_cust_part_number
                    .clone()
                    .filter(|s| !s.is_empty()),
                // Mouser reports "None" for parts without a packaging choice
                packaging_choice: l
                    .packaging_choice
                    .clone()
                    .filter(|s| !s.is_empty() && s != "None"),
            })
        })
        .collect();
    for i in items {
        match out
            .iter_mut()
            .find(|o| same_part(&o.mouser_part_number, &i.mouser_part_number))
        {
            Some(o) => {
                o.quantity += i.quantity;
                if i.customer_part_number.is_some() {
                    o.customer_part_number.clone_from(&i.customer_part_number);
                }
                if i.packaging_choice.is_some() {
                    o.packaging_choice.clone_from(&i.packaging_choice);
                }
            }
            None => out.push(CartItemRequest {
                mouser_part_number: i.mouser_part_number.clone(),
                quantity: i.quantity,
                customer_part_number: i.customer_part_number.clone(),
                packaging_choice: i.packaging_choice.clone(),
            }),
        }
    }
    out
}

/// Every line written is in the cart with its quantity, and nothing else is.
fn check_cart(cart: &CartResponseRoot, want: &[CartItemRequest]) -> Result<()> {
    let got: Vec<(String, i64)> = cart
        .cart_items
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
    let mut problems = vec![];
    for w in want {
        match got
            .iter()
            .find(|(n, _)| same_part(n, &w.mouser_part_number))
        {
            None => problems.push(format!("{} is missing", w.mouser_part_number)),
            Some((_, q)) if *q != w.quantity => problems.push(format!(
                "{} has quantity {q}, expected {}",
                w.mouser_part_number, w.quantity
            )),
            Some(_) => {}
        }
    }
    for (n, _) in &got {
        if !want.iter().any(|w| same_part(n, &w.mouser_part_number)) {
            problems.push(format!("{n} is in the cart but was not written"));
        }
    }
    anyhow::ensure!(
        problems.is_empty(),
        "the cart does not hold what was written: {}",
        problems.join("; ")
    );
    Ok(())
}

pub async fn update_cart_items(c: &Client, p: &CartItemsParams) -> Result<Value> {
    to_value(
        c.update_cart_items(
            &cart_request(p),
            p.country_code.as_deref(),
            p.currency_code.as_deref(),
        )
        .await?,
    )
}

pub async fn remove_cart_item(c: &Client, p: &RemoveCartItemParams) -> Result<Value> {
    to_value(
        c.remove_cart_item(
            &p.cart_key,
            &p.mouser_part_number,
            p.country_code.as_deref(),
            p.currency_code.as_deref(),
        )
        .await?,
    )
}

fn schedule_request(p: &ScheduleParams) -> ScheduleCartItemsRequestRoot {
    ScheduleCartItemsRequestRoot {
        cart_key: Some(p.cart_key.clone()),
        schedule_cart_items: Some(
            p.items
                .iter()
                .map(|i| ScheduleReleaseRequest {
                    mouser_part_number: i.mouser_part_number.clone(),
                    scheduled_releases: Some(
                        i.releases
                            .iter()
                            .map(|r| KeyValuePairOfStringAndInt32 {
                                key: Some(r.date.clone()),
                                value: Some(r.quantity),
                                ..Default::default()
                            })
                            .collect(),
                    ),
                })
                .collect(),
        ),
    }
}

pub async fn insert_schedule_cart_items(c: &Client, p: &ScheduleParams) -> Result<Value> {
    to_value(c.insert_schedule_cart_items(&schedule_request(p)).await?)
}

pub async fn update_schedule_cart_items(c: &Client, p: &ScheduleParams) -> Result<Value> {
    to_value(c.update_schedule_cart_items(&schedule_request(p)).await?)
}

pub async fn delete_all_schedule_cart_items(c: &Client, p: &CartKeyParams) -> Result<Value> {
    to_value(c.delete_all_schedule_cart_items(&p.cart_key).await?)
}

// ── Order history ───────────────────────────────────────────────────────────────────────

pub async fn order_history_by_date_filter(c: &Client, p: &DateFilterParams) -> Result<Value> {
    to_value(c.order_history_by_date_filter(p.date_filter).await?)
}

pub async fn order_history_by_date_range(c: &Client, p: &DateRangeParams) -> Result<Value> {
    to_value(
        c.order_history_by_date_range(&p.start_date, &p.end_date)
            .await?,
    )
}

pub async fn order_by_sales_order_number(c: &Client, p: &SalesOrderParams) -> Result<Value> {
    to_value(c.order_by_sales_order_number(&p.sales_order_number).await?)
}

pub async fn order_by_web_order_number(c: &Client, p: &WebOrderParams) -> Result<Value> {
    to_value(c.order_by_web_order_number(p.web_order_number).await?)
}

// ── Orders ──────────────────────────────────────────────────────────────────────────────

pub async fn order_options_query(c: &Client, p: &OrderOptionsParams) -> Result<Value> {
    let r = OrderInializeRequestRoot {
        order_initialize: Some(OrderInitializeRequest {
            shipping_address: p.shipping_address.clone(),
            currency_code: p.currency_code.clone(),
            cart_key: p.cart_key.clone(),
        }),
    };
    to_value(c.order_options_query(&r).await?)
}

pub async fn order_currencies(c: &Client, p: &CurrenciesParams) -> Result<Value> {
    to_value(
        c.order_currencies(p.shipping_country_code.as_deref())
            .await?,
    )
}

pub async fn order_countries(c: &Client, p: &CountriesParams) -> Result<Value> {
    to_value(c.order_countries(p.country_code.as_deref()).await?)
}

pub async fn get_order(c: &Client, p: &OrderNumberParams) -> Result<Value> {
    to_value(c.get_order(p.order_number).await?)
}

fn root(o: &OrderRequestType) -> OrderRequestRootType {
    OrderRequestRootType {
        order: Some(o.clone()),
    }
}

pub async fn preview_order(c: &Client, p: &OrderParams) -> Result<Value> {
    to_value(c.preview_order(&root(&p.order.request())).await?)
}

/// What a submission must find again: the preview's lines, and the total and currency the
/// caller passed.
fn expected_from(preview: &OrderRoot, currency: &str, total: f64) -> ExpectedOrder {
    let lines = preview
        .order
        .as_ref()
        .and_then(|o| o.order_lines.as_deref())
        .unwrap_or(&[])
        .iter()
        .map(|l| {
            (
                l.mouser_part_number.clone().unwrap_or_default(),
                l.quantity.unwrap_or(0),
            )
        })
        .collect();
    ExpectedOrder {
        currency_code: currency.into(),
        order_total: total,
        lines,
    }
}

/// A fresh preview of the order `p` would place, checked against the total the caller passed
/// and the ceiling, before the user is asked; with what the submission must match.
pub async fn verified_preview(
    c: &Client,
    p: &SubmitOrderParams,
) -> Result<(OrderRoot, ExpectedOrder)> {
    let preview = c.preview_order(&root(&p.order.request())).await?;
    let expected = expected_from(&preview, &p.order.currency_code, p.total);
    c.verify_order(&preview, &expected)?;
    Ok((preview, expected))
}

pub async fn verified_preview_from_order(
    c: &Client,
    p: &SubmitFromOrderParams,
) -> Result<(OrderRoot, ExpectedOrder)> {
    let preview = c
        .preview_order_from_order(
            p.order_number,
            &root(&p.order.request()),
            None,
            Some(&p.order.currency_code),
        )
        .await?;
    let expected = expected_from(&preview, &p.order.currency_code, p.total);
    c.verify_order(&preview, &expected)?;
    Ok((preview, expected))
}

/// Places the order; Mouser's fresh preview must still match `expected` (the lines the user
/// confirmed, the total, the ceiling).
pub async fn submit_order(
    c: &Client,
    p: &SubmitOrderParams,
    expected: &ExpectedOrder,
) -> Result<Value> {
    to_value(c.submit_order(&root(&p.order.request()), expected).await?)
}

pub async fn preview_order_from_order(c: &Client, p: &FromOrderParams) -> Result<Value> {
    to_value(
        c.preview_order_from_order(
            p.order_number,
            &root(&p.order.request()),
            None,
            Some(&p.order.currency_code),
        )
        .await?,
    )
}

pub async fn submit_order_from_order(
    c: &Client,
    p: &SubmitFromOrderParams,
    expected: &ExpectedOrder,
) -> Result<Value> {
    to_value(
        c.submit_order_from_order(
            p.order_number,
            &root(&p.order.request()),
            None,
            Some(&p.order.currency_code),
            expected,
        )
        .await?,
    )
}

pub async fn create_cart_from_order(c: &Client, p: &CartFromOrderParams) -> Result<Value> {
    c.create_cart_from_order(
        p.order_number,
        p.country_code.as_deref(),
        p.currency_code.as_deref(),
    )
    .await
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order_client(server: &wiremock::MockServer) -> Client {
        Client::new(mouser::Config {
            base_url: format!("{}/api", server.uri()),
            order_key: Some(mouser::ApiKey::new("test-order-key-0000").unwrap()),
            max_calls_per_minute: 0,
            ..Default::default()
        })
        .unwrap()
    }

    fn cart(lines: Value) -> Value {
        json!({"Errors": [], "CartKey": "cart-1", "CurrencyCode": "EUR", "CartItems": lines})
    }

    /// GET /v1/cart answers `first` once, then `then`.
    async fn mount_carts(server: &wiremock::MockServer, first: Value, then: Value) {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, ResponseTemplate};
        Mock::given(method("GET"))
            .and(path("/api/v1/cart"))
            .and(query_param("cartKey", "cart-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(first))
            .up_to_n_times(1)
            .with_priority(1)
            .mount(server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/v1/cart"))
            .respond_with(ResponseTemplate::new(200).set_body_json(then))
            .with_priority(2)
            .mount(server)
            .await;
    }

    fn add(items: Vec<(&str, i64, Option<&str>)>) -> CartItemsParams {
        CartItemsParams {
            cart_key: Some("cart-1".into()),
            items: items
                .into_iter()
                .map(|(n, q, r)| CartItem {
                    mouser_part_number: n.into(),
                    quantity: q,
                    customer_part_number: r.map(Into::into),
                    packaging_choice: None,
                })
                .collect(),
            mouser_pays_customs_and_duties: None,
            country_code: None,
            currency_code: None,
        }
    }

    #[tokio::test]
    async fn adding_to_a_cart_keeps_its_lines_and_checks_the_result() {
        use wiremock::matchers::{body_json, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        let splitter = json!({"MouserPartNumber": "139-ZX10R-2-183-S+", "Quantity": 2,
            "CartItemCustPartNumber": "splitter", "PackagingChoice": "None"});
        let both = json!([splitter, {"MouserPartNumber": "67-ST0645", "Quantity": 3}]);
        mount_carts(&server, cart(json!([splitter])), cart(both.clone())).await;
        // the whole cart is written: the line already there, plus the new one
        Mock::given(method("POST"))
            .and(path("/api/v1/cart"))
            .and(body_json(json!({"CartKey": "cart-1", "CartItems": [
                {"MouserPartNumber": "139-ZX10R-2-183-S+", "Quantity": 2, "CustomerPartNumber": "splitter"},
                {"MouserPartNumber": "67-ST0645", "Quantity": 3}]})))
            .respond_with(ResponseTemplate::new(200).set_body_json(cart(both)))
            .expect(1)
            .mount(&server)
            .await;
        // the insert endpoint, which has dropped lines, is not used for an existing cart
        Mock::given(method("POST"))
            .and(path("/api/v1/cart/items/insert"))
            .respond_with(ResponseTemplate::new(500))
            .expect(0)
            .mount(&server)
            .await;
        let r = add_cart_items(
            &order_client(&server),
            &add(vec![("67-ST0645", 1, None), ("67-st0645", 2, None)]),
        )
        .await
        .unwrap();
        assert_eq!(r["CartItems"].as_array().unwrap().len(), 2, "{r:#}");
    }

    #[tokio::test]
    async fn a_cart_that_lost_a_line_is_an_error_not_a_success() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        let splitter = json!({"MouserPartNumber": "139-ZX10R-2-183-S+", "Quantity": 2});
        let only_new = cart(json!([{"MouserPartNumber": "67-ST0645", "Quantity": 2}]));
        mount_carts(&server, cart(json!([splitter])), only_new.clone()).await;
        Mock::given(method("POST"))
            .and(path("/api/v1/cart"))
            .respond_with(ResponseTemplate::new(200).set_body_json(only_new))
            .mount(&server)
            .await;
        let e = add_cart_items(&order_client(&server), &add(vec![("67-ST0645", 2, None)]))
            .await
            .unwrap_err()
            .to_string();
        assert!(e.contains("139-ZX10R-2-183-S+ is missing"), "{e}");
    }

    #[test]
    fn prices_are_read_with_the_currency_s_own_separators_or_not_at_all() {
        let d = |v: &str| v.parse::<Decimal>().unwrap();
        for (raw, cur, v) in [
            ("87,10 €", "EUR", "87.10"),
            ("€1.234,56", "EUR", "1234.56"),
            ("$91.00", "USD", "91.00"),
            ("$1,234.56", "USD", "1234.56"),
            ("0,0459 €", "EUR", "0.0459"),
            ("£12.50", "GBP", "12.50"),
        ] {
            assert_eq!(parse_price(raw, cur), Some(d(v)), "{raw} {cur}");
        }
        // another locale's separators, an unknown symbol, no number: refused, never guessed
        for (raw, cur) in [
            ("$91,00", "USD"),
            ("87.10 €", "EUR"),
            ("US$ 9.00", "USD"),
            ("call", "EUR"),
            ("1", "XXX9"),
        ] {
            assert_eq!(parse_price(raw, cur), None, "{raw} {cur}");
        }
    }

    #[test]
    fn the_price_break_is_the_largest_not_above_the_quantity() {
        let b = |q: i64, p: &str| Pricebreak {
            quantity: Some(q),
            price: Some(p.into()),
            currency: Some("EUR".into()),
            ..Default::default()
        };
        let part = MouserPart {
            price_breaks: Some(vec![b(10, "80,00 €"), b(1, "90,00 €"), b(100, "70,00 €")]),
            ..Default::default()
        };
        let unit = |q| price_at(&part, q).unwrap().unwrap().unit.to_string();
        assert_eq!(unit(1), "90.00");
        assert_eq!(unit(99), "80.00");
        assert_eq!(unit(500), "70.00");
    }

    #[tokio::test]
    async fn price_bom_picks_the_offer_that_covers_the_quantity_and_prices_it() {
        use wiremock::matchers::{body_json, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        let offer = |mpn: &str, stock: &str, price: &str| {
            json!({"MouserPartNumber": mpn, "ManufacturerPartNumber": "ZFRSC-42-S+", "Manufacturer": "Mini-Circuits",
                "AvailabilityInStock": stock, "Min": "1", "Mult": "1",
                "PriceBreaks": [{"Quantity": 1, "Price": price, "Currency": "EUR"}]})
        };
        Mock::given(method("POST"))
            .and(path("/api/v1/search/partnumber"))
            .and(body_json(json!({"SearchByPartRequest": {
                "mouserPartNumber": "ZFRSC-42-S+|NOPE-123", "partSearchOptions": "Exact"}})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"Errors": [], "SearchResults": {
                "NumberOfResult": 2,
                // the cheaper offer cannot cover the quantity: the other one wins
                "Parts": [offer("139-ZFRSC-42-S+A", "1", "80,00 €"), offer("139-ZFRSC-42-S+", "283", "87,10 €")]}})))
            .expect(1)
            .mount(&server)
            .await;
        let c = Client::new(mouser::Config {
            base_url: format!("{}/api", server.uri()),
            search_key: Some(mouser::ApiKey::new("test-search-key-0000").unwrap()),
            max_calls_per_minute: 0,
            ..Default::default()
        })
        .unwrap();
        let line = |number: &str, q: i64| BomLine {
            part_number: number.into(),
            quantity: q,
            reference: Some("x".into()),
        };
        let r = price_bom(
            &c,
            &BomParams {
                lines: vec![
                    line("ZFRSC-42-S+", 2),
                    line("NOPE-123", 1),
                    line("zfrsc-42-s+", 1),
                ],
                mouser_pays_customs_and_duties: None,
            },
        )
        .await
        .unwrap();
        let l = &r["lines"];
        assert_eq!(l[0]["MouserPartNumber"], "139-ZFRSC-42-S+", "{r:#}");
        assert_eq!(l[0]["status"], "in_stock");
        assert_eq!(l[0]["extended_price"], "174.20");
        assert_eq!(l[0]["other_offers"], 1);
        assert_eq!(l[1]["status"], "not_found");
        assert_eq!(l[2]["MouserPartNumber"], "139-ZFRSC-42-S+A"); // one piece: the cheaper offer covers it
        assert_eq!(r["totals_by_currency"]["EUR"], "254.20");
    }
}
