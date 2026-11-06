//! Tool logic: plain functions over `mouser::Client`, returning JSON. No MCP types here.

use std::collections::BTreeMap;

use anyhow::Result;
use mouser::models::*;
use mouser::{Client, PartSearchOption, SearchOption};
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

/// Mouser formats prices for the account's locale: "$91.00", "87,10 €", "€1.234,56".
pub fn parse_price(s: &str) -> Option<f64> {
    let t: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .collect();
    if t.is_empty() {
        return None;
    }
    let (dot, comma) = (t.rfind('.'), t.rfind(','));
    let normal = match (dot, comma) {
        (Some(d), Some(c)) if c > d => t.replace('.', "").replace(',', "."),
        (Some(_), Some(_)) => t.replace(',', ""),
        // a lone comma with 1-3 decimals after it is a decimal comma ("87,10"); "1,234" too
        // ambiguous to be anything but thousands only with exactly three digits and no cents
        (None, Some(c)) if t.len() - c - 1 != 3 => t.replace(',', "."),
        (None, Some(_)) => t.replace(',', ""),
        _ => t,
    };
    normal.parse().ok()
}

fn int(s: &Option<String>) -> Option<i64> {
    s.as_deref()
        .and_then(|v| v.replace([',', '.'], "").trim().parse().ok())
}

/// Unit price at `qty`: the largest price break whose quantity is at most `qty`.
fn price_at(p: &MouserPart, qty: i64) -> Option<(f64, i64, String)> {
    let mut breaks: Vec<&Pricebreak> = p.price_breaks.as_deref().unwrap_or(&[]).iter().collect();
    breaks.sort_by_key(|b| b.quantity.unwrap_or(0));
    let b = breaks
        .iter()
        .rev()
        .find(|b| b.quantity.unwrap_or(0) <= qty)
        .or(breaks.first())?;
    Some((
        parse_price(b.price.as_deref()?)?,
        b.quantity.unwrap_or(0),
        b.currency.clone().unwrap_or_default(),
    ))
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
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut out = Vec::new();
    for l in &p.lines {
        let cands: Vec<&MouserPart> = found
            .iter()
            .filter(|f| matches(f, &l.part_number))
            .collect();
        // prefer an offer that covers the quantity from stock, then the lowest extended price
        let best = cands.iter().min_by(|a, b| {
            let key = |x: &MouserPart| {
                let short = int(&x.availability_in_stock).unwrap_or(0) < l.quantity;
                let ext = price_at(x, l.quantity)
                    .map(|(u, _, _)| u * l.quantity as f64)
                    .unwrap_or(f64::MAX);
                (short, ext)
            };
            let (ka, kb) = (key(a), key(b));
            ka.0.cmp(&kb.0).then(ka.1.total_cmp(&kb.1))
        });
        let line = match best {
            None => {
                json!({"reference": l.reference, "part_number": l.part_number, "quantity": l.quantity,
                "status": "not_found"})
            }
            Some(b) => {
                let stock = int(&b.availability_in_stock).unwrap_or(0);
                let price = price_at(b, l.quantity);
                let min = int(&b.min).unwrap_or(1);
                let mult = int(&b.mult).unwrap_or(1).max(1);
                let mut notes = Vec::new();
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
                let extended = price.as_ref().map(|(u, _, _)| u * l.quantity as f64);
                if let (Some(e), Some((_, _, cur))) = (extended, price.as_ref()) {
                    *totals.entry(cur.clone()).or_default() += e;
                }
                json!({
                    "reference": l.reference,
                    "part_number": l.part_number,
                    "quantity": l.quantity,
                    "status": if stock >= l.quantity { "in_stock" } else { "short" },
                    "MouserPartNumber": b.mouser_part_number,
                    "Manufacturer": b.manufacturer,
                    "ManufacturerPartNumber": b.manufacturer_part_number,
                    "Description": b.description,
                    "in_stock": stock,
                    "AvailabilityOnOrder": b.availability_on_order,
                    "LeadTime": b.lead_time,
                    "unit_price": price.as_ref().map(|p| p.0),
                    "price_break_quantity": price.as_ref().map(|p| p.1),
                    "currency": price.as_ref().map(|p| p.2.clone()),
                    "extended_price": extended,
                    "notes": notes,
                    "other_offers": cands.len() - 1,
                    "ProductDetailUrl": b.product_detail_url,
                })
            }
        };
        out.push(line);
    }
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

pub async fn add_cart_items(c: &Client, p: &CartItemsParams) -> Result<Value> {
    to_value(
        c.add_cart_items(
            &cart_request(p),
            p.country_code.as_deref(),
            p.currency_code.as_deref(),
        )
        .await?,
    )
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
    to_value(c.preview_order(&root(&p.order)).await?)
}

pub async fn submit_order(c: &Client, p: &SubmitOrderParams) -> Result<Value> {
    to_value(c.submit_order(&root(&p.order), &p.expected).await?)
}

pub async fn preview_order_from_order(c: &Client, p: &FromOrderParams) -> Result<Value> {
    to_value(
        c.preview_order_from_order(
            p.order_number,
            &root(&p.order),
            p.country_code.as_deref(),
            p.currency_code.as_deref(),
        )
        .await?,
    )
}

pub async fn submit_order_from_order(c: &Client, p: &SubmitFromOrderParams) -> Result<Value> {
    to_value(
        c.submit_order_from_order(
            p.order_number,
            &root(&p.order),
            p.country_code.as_deref(),
            p.currency_code.as_deref(),
            &p.expected,
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

    #[test]
    fn prices_parse_in_the_formats_mouser_uses() {
        for (s, v) in [
            ("$91.00", 91.0),
            ("87,10 €", 87.1),
            ("€1.234,56", 1234.56),
            ("$1,234.56", 1234.56),
            ("¥12,345", 12345.0),
            ("0,0459 €", 0.0459),
        ] {
            assert_eq!(parse_price(s), Some(v), "{s}");
        }
        assert_eq!(parse_price("call"), None);
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
        assert_eq!(price_at(&part, 1).unwrap().0, 90.0);
        assert_eq!(price_at(&part, 99).unwrap().0, 80.0);
        assert_eq!(price_at(&part, 500).unwrap().0, 70.0);
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
        assert_eq!(l[0]["extended_price"], 174.2);
        assert_eq!(l[0]["other_offers"], 1);
        assert_eq!(l[1]["status"], "not_found");
        assert_eq!(l[2]["MouserPartNumber"], "139-ZFRSC-42-S+A"); // one piece: the cheaper offer covers it
        let total = r["totals_by_currency"]["EUR"].as_f64().unwrap();
        assert!((total - 254.2).abs() < 1e-9, "{total}");
    }
}
