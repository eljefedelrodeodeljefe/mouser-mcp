//! The user's confirmation of an order, asked through MCP elicitation.
//!
//! The confirmation comes from the person at the client, not from the model: the client
//! shows a form and returns what the user chose. A model can neither see the form nor answer
//! it.
//!
//! The form has a one-line summary (total, shipping, payment, ship-to) and two fields: "Order
//! lines", a multi-select list with one option per line, all selected, and "Place this
//! order", a checkbox, unticked. Clients step through the list with the keyboard, so every
//! line is in reach however long the order is. Only an accepted form with every line still
//! selected and the box ticked confirms; deselecting a line refuses the order.
//!
//! Two protocol paths:
//!
//! - **2026-07-28 and later** (multi round-trip, SEP-2322): the tool returns an
//!   `InputRequiredResult` holding the form; the client asks the user and calls the tool
//!   again with the answer and the echoed `requestState`. The state is only a handle to a
//!   pending confirmation kept here: single use, expiring, bound to the exact arguments of the
//!   first call, and holding the lines and total the user was shown.
//! - **earlier versions**: the server sends `elicitation/create` while the tool call is open.
//!
//! A client that supports neither cannot confirm, and no order is placed.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use mouser::ExpectedOrder;
use mouser::models::OrderRoot;
use rmcp::model::{ElicitationSchema, JsonObject};
use serde_json::{Value, json};
use tokio::time::Instant;

/// How long the user has to answer the form.
pub const TTL: Duration = Duration::from_secs(10 * 60);

/// Key of the form in `inputRequests` / `inputResponses`.
pub const INPUT_KEY: &str = "confirm_order";

/// The multi-select list of order lines. Sorts before [`PLACE`] (clients list fields in key
/// order).
pub const LINES: &str = "lines";

/// The checkbox that places the order.
pub const PLACE: &str = "place_order";

fn money(v: Option<f64>) -> String {
    v.map_or_else(|| "?".into(), |v| format!("{v:.2}"))
}

/// The form for the order in `preview`: the message and the schema.
pub struct Form {
    pub message: String,
    pub schema: ElicitationSchema,
    /// The values of the [`LINES`] options, one per order line.
    pub keys: Vec<String>,
}

/// Builds the form: a one-line message, the lines as a multi-select list, all selected, and
/// an unticked "Place this order".
pub fn form(preview: &OrderRoot) -> Form {
    let o = preview.order.clone().unwrap_or_default();
    let cur = o.currency_code.as_deref().unwrap_or("?");
    let lines = o.order_lines.as_deref().unwrap_or(&[]);

    let mut facts = vec![format!(
        "{} line{}",
        lines.len(),
        if lines.len() == 1 { "" } else { "s" }
    )];
    facts.push(format!("merchandise {}", money(o.merchandise_total)));
    if let Some(m) = o.shipping_method.as_ref() {
        facts.push(format!(
            "{} {}",
            m.primary_method.as_deref().unwrap_or("shipping"),
            money(m.primary_shipping_rate)
        ));
    }
    if o.additional_fees_total.is_some_and(|f| f != 0.0) {
        facts.push(format!("fees {}", money(o.additional_fees_total)));
    }
    if o.tax_amount.is_some() {
        facts.push(format!("tax {}", money(o.tax_amount)));
    }
    if let Some(p) = o.payment_method.as_ref().and_then(|p| p.name.as_deref()) {
        facts.push(p.to_string());
    }
    if let Some(a) = o.shipping_address.as_ref() {
        let who: Vec<&str> = [
            a.first_name.as_deref(),
            a.last_name.as_deref(),
            a.city.as_deref(),
            a.country_code.as_deref(),
        ]
        .into_iter()
        .flatten()
        .filter(|p| !p.is_empty())
        .collect();
        if !who.is_empty() {
            facts.push(format!("to {}", who.join(" ")));
        }
    }
    // Claude Code shows the message on one line, cut at about 150 characters: facts only, the
    // fields explain themselves. The ceiling is enforced, not shown.
    let message = format!(
        "Real order at Mouser: {} {cur} · {}",
        money(o.order_total),
        facts.join(" · ")
    );

    let width = lines.len().max(1).to_string().len().max(2);
    let mut options = vec![];
    let mut keys = vec![];
    for (i, l) in lines.iter().enumerate() {
        let key = format!("line_{:0width$}", i + 1);
        let reference = l
            .cart_item_cust_part_number
            .as_deref()
            .filter(|r| !r.is_empty())
            .map(|r| format!("  ({r})"))
            .unwrap_or_default();
        let title = format!(
            "{} × {}  {} each  = {} {cur}{reference}",
            l.quantity.unwrap_or(0),
            l.mfr_part_number
                .as_deref()
                .or(l.mouser_part_number.as_deref())
                .unwrap_or("?"),
            money(l.unit_price),
            money(l.extended_price),
        );
        options.push(json!({"const": key, "title": title}));
        keys.push(key);
    }
    let mut props = serde_json::Map::new();
    props.insert(
        LINES.into(),
        json!({"type": "array", "title": "Order lines",
            "description": "Everything in this order. Deselect any line to stop the order.",
            "items": {"anyOf": options}, "default": keys}),
    );
    props.insert(
        PLACE.into(),
        json!({"type": "boolean",
            "title": format!("Place this order: {} {cur}", money(o.order_total)),
            "description": "A real purchase, charged to the payment method above",
            "default": false}),
    );
    let schema = serde_json::from_value(json!({
        "type": "object", "properties": props, "required": [LINES, PLACE],
    }))
    // A titled multi-select and a boolean, built from strings: always a valid schema.
    .expect("order form schema");
    Form {
        message,
        schema,
        keys,
    }
}

/// What the user answered.
#[derive(Debug, PartialEq, Eq)]
pub enum Answer {
    Confirmed,
    Refused(String),
}

/// Reads an elicitation result (`{"action": …, "content": {…}}`): only an accepted form with
/// every one of `keys` still selected in [`LINES`] and [`PLACE`] ticked confirms.
pub fn answer(response: Option<&Value>, keys: &[String]) -> Answer {
    let Some(r) = response else {
        return Answer::Refused("no answer to the confirmation form".into());
    };
    match r.get("action").and_then(Value::as_str) {
        Some("accept") => {}
        Some(other) => return Answer::Refused(format!("the user chose {other:?}")),
        None => return Answer::Refused("the confirmation answer has no action".into()),
    }
    let content = r.get("content");
    let selected: Vec<&str> = content
        .and_then(|c| c.get(LINES))
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let dropped: Vec<&str> = keys
        .iter()
        .map(String::as_str)
        .filter(|k| !selected.contains(k))
        .collect();
    if !dropped.is_empty() {
        return Answer::Refused(format!("the user deselected {}", dropped.join(", ")));
    }
    match content.and_then(|c| c.get(PLACE)).and_then(Value::as_bool) {
        Some(true) => Answer::Confirmed,
        _ => Answer::Refused("the user did not tick \"Place this order\"".into()),
    }
}

/// A confirmation asked for: what the user is shown and must find again at submission.
pub struct Confirmed {
    pub expected: ExpectedOrder,
    pub keys: Vec<String>,
}

struct Pending {
    tool: String,
    args: JsonObject,
    confirmed: Confirmed,
    created: Instant,
}

/// Confirmations asked for and not yet answered.
#[derive(Default)]
pub struct Store {
    next: AtomicU64,
    pending: Mutex<HashMap<String, Pending>>,
}

impl Store {
    /// Records a confirmation for `tool` called with `args`; returns its handle.
    pub fn open(&self, tool: &str, args: JsonObject, confirmed: Confirmed) -> String {
        let id = format!("order-{}", self.next.fetch_add(1, Ordering::Relaxed));
        let mut p = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        p.retain(|_, v| v.created.elapsed() < TTL);
        p.insert(
            id.clone(),
            Pending {
                tool: tool.into(),
                args,
                confirmed,
                created: Instant::now(),
            },
        );
        id
    }

    /// Takes the confirmation `id` (single use); it must be for this tool and these exact
    /// arguments and not have expired.
    pub fn take(&self, id: &str, tool: &str, args: &JsonObject) -> Result<Confirmed, String> {
        let p = self
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(id)
            .ok_or("unknown or already used confirmation; submit again")?;
        if p.created.elapsed() >= TTL {
            return Err("the confirmation expired; submit again".into());
        }
        if p.tool != tool || &p.args != args {
            return Err("the confirmation was for another order; submit again".into());
        }
        Ok(p.confirmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(total: f64) -> JsonObject {
        json!({"total": total}).as_object().unwrap().clone()
    }

    fn confirmed() -> Confirmed {
        Confirmed {
            expected: ExpectedOrder {
                currency_code: "EUR".into(),
                order_total: 533.78,
                lines: vec![("139-ZX10R-2-183-S+".into(), 2)],
            },
            keys: vec!["line_01".into()],
        }
    }

    #[test]
    fn a_confirmation_is_single_use_and_bound_to_its_order() {
        let s = Store::default();
        let id = s.open("submit_order", args(533.78), confirmed());
        assert!(s.take(&id, "submit_order", &args(999.0)).is_err());
        // a failed take consumed it: no second try with the right arguments either
        assert!(s.take(&id, "submit_order", &args(533.78)).is_err());
        let id = s.open("submit_order", args(533.78), confirmed());
        assert!(
            s.take(&id, "submit_order_from_order", &args(533.78))
                .is_err()
        );
        let id = s.open("submit_order", args(533.78), confirmed());
        let c = s.take(&id, "submit_order", &args(533.78)).unwrap();
        assert_eq!(c.expected.order_total, 533.78);
        assert!(s.take(&id, "submit_order", &args(533.78)).is_err());
        assert!(s.take("order-99", "submit_order", &args(533.78)).is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn a_confirmation_expires_after_ten_minutes() {
        let s = Store::default();
        let id = s.open("submit_order", args(533.78), confirmed());
        tokio::time::advance(TTL - Duration::from_secs(1)).await;
        assert!(s.take(&id, "submit_order", &args(533.78)).is_ok());

        let id = s.open("submit_order", args(533.78), confirmed());
        tokio::time::advance(TTL).await;
        let e = s.take(&id, "submit_order", &args(533.78)).err();
        assert_eq!(e.as_deref(), Some("the confirmation expired; submit again"));

        // an expired confirmation is dropped when the next one is asked for
        let old = s.open("submit_order", args(533.78), confirmed());
        tokio::time::advance(TTL).await;
        s.open("submit_order", args(533.78), confirmed());
        let e = s.take(&old, "submit_order", &args(533.78)).err();
        assert_eq!(
            e.as_deref(),
            Some("unknown or already used confirmation; submit again")
        );
    }

    #[test]
    fn only_an_accepted_form_with_every_line_and_the_box_confirms() {
        let keys: Vec<String> = vec!["line_01".into(), "line_02".into()];
        let a = |v: Value| answer(Some(&v), &keys);
        let all = json!({"lines": ["line_02", "line_01"], "place_order": true});
        assert_eq!(
            a(json!({"action": "accept", "content": all})),
            Answer::Confirmed
        );
        assert_eq!(
            a(json!({"action": "accept", "content": {"lines": ["line_01"], "place_order": true}})),
            Answer::Refused("the user deselected line_02".into())
        );
        assert_eq!(
            a(
                json!({"action": "accept", "content": {"lines": ["line_01", "line_02"], "place_order": false}})
            ),
            Answer::Refused("the user did not tick \"Place this order\"".into())
        );
        for v in [
            json!({"action": "accept", "content": {"place_order": true}}),
            json!({"action": "accept", "content": {"lines": "line_01,line_02", "place_order": true}}),
            json!({"action": "accept"}),
            json!({"action": "decline"}),
            json!({"action": "cancel", "content": all}),
            json!({"content": all}),
        ] {
            assert!(matches!(a(v.clone()), Answer::Refused(_)), "{v}");
        }
        assert!(matches!(answer(None, &keys), Answer::Refused(_)));
    }

    fn preview(lines: usize) -> OrderRoot {
        let line = |i: usize| {
            json!({"MouserPartNumber": format!("139-PART-{i}"), "MfrPartNumber": format!("PART-{i}"),
                "Description": "RF splitter", "CartItemCustPartNumber": "WP6",
                "Quantity": 2, "UnitPrice": 161.1, "ExtendedPrice": 322.2})
        };
        serde_json::from_value(json!({"Order": {
            "CurrencyCode": "EUR", "MerchandiseTotal": 493.78, "TaxAmount": 0.0, "OrderTotal": 533.78,
            "ShippingMethod": {"PrimaryMethod": "UPS Worldwide Express Saver", "PrimaryShippingRate": 40.0},
            "ShippingAddress": {"FirstName": "Robert", "LastName": "L", "City": "Berlin", "CountryCode": "DE"},
            "PaymentMethod": {"Name": "PayPal"},
            "OrderLines": (1..=lines).map(line).collect::<Vec<_>>()}}))
        .unwrap()
    }

    #[test]
    fn the_form_has_a_one_line_summary_a_list_of_lines_and_one_box() {
        let f = form(&preview(2));
        assert!(!f.message.contains('\n'), "{}", f.message);
        for want in [
            "Real order at Mouser: 533.78 EUR",
            "2 lines",
            "UPS Worldwide Express Saver 40.00",
            "tax 0.00",
            "PayPal",
            "to Robert L Berlin DE",
        ] {
            assert!(
                f.message.contains(want),
                "missing {want:?} in {}",
                f.message
            );
        }
        assert_eq!(f.keys, ["line_01", "line_02"]);
        let v = serde_json::to_value(&f.schema).unwrap();
        let props = v["properties"].as_object().unwrap();
        assert_eq!(props.keys().collect::<Vec<_>>(), ["lines", "place_order"]);
        let lines = &v["properties"]["lines"];
        assert_eq!(lines["type"], "array");
        assert_eq!(lines["default"], json!(["line_01", "line_02"]));
        assert_eq!(
            lines["items"]["anyOf"][0],
            json!({"const": "line_01", "title": "2 × PART-1  161.10 each  = 322.20 EUR  (WP6)"})
        );
        assert_eq!(v["properties"]["place_order"]["type"], "boolean");
        assert_eq!(v["properties"]["place_order"]["default"], false);
        assert_eq!(v["required"], json!(["lines", "place_order"]));
    }

    #[test]
    fn line_keys_sort_in_order_however_many_lines() {
        let f = form(&preview(120));
        assert_eq!(f.keys[0], "line_001");
        assert_eq!(f.keys[119], "line_120");
        let mut sorted = f.keys.clone();
        sorted.sort();
        assert_eq!(sorted, f.keys, "clients list fields in key order");
    }
}
