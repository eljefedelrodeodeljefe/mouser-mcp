//! The user's confirmation of an order, asked through MCP elicitation.
//!
//! The confirmation comes from the person at the client, not from the model: the client
//! shows a form and returns what the user chose. A model can neither see the form nor answer
//! it.
//!
//! The form has a one-line summary (total, shipping, payment, ship-to) and one checkbox per
//! order line, ticked, followed by "Place this order", unticked. Clients step through form
//! fields with the keyboard, so every line is in reach however long the order is. Only an
//! accepted form with every box ticked confirms; unticking a line refuses the order.
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
use std::time::{Duration, Instant};

use mouser::ExpectedOrder;
use mouser::models::OrderRoot;
use rmcp::model::{ElicitationSchema, JsonObject};
use serde_json::{Value, json};

/// How long the user has to answer the form.
pub const TTL: Duration = Duration::from_secs(10 * 60);

/// Key of the form in `inputRequests` / `inputResponses`.
pub const INPUT_KEY: &str = "confirm_order";

/// The final checkbox. Sorts after the `line_…` fields (clients list fields in key order).
pub const PLACE: &str = "place_order";

fn money(v: Option<f64>) -> String {
    v.map_or_else(|| "?".into(), |v| format!("{v:.2}"))
}

/// The form for the order in `preview`: the message and the schema.
pub struct Form {
    pub message: String,
    pub schema: ElicitationSchema,
    /// The checkbox keys, lines first, then [`PLACE`].
    pub keys: Vec<String>,
}

/// Builds the form: a one-line message and one ticked checkbox per line.
pub fn form(preview: &OrderRoot, ceiling: Option<f64>) -> Form {
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
    let mut message = format!(
        "Real order at Mouser: {} {cur} ({}).",
        money(o.order_total),
        facts.join(" · ")
    );
    if let Some(c) = ceiling {
        message += &format!(" Ceiling {c:.2}.");
    }
    message += " Every line is ticked; untick one to stop. Tick \"Place this order\" to buy.";

    let width = lines.len().max(1).to_string().len().max(2);
    let mut props = serde_json::Map::new();
    let mut keys = vec![];
    for (i, l) in lines.iter().enumerate() {
        let key = format!("line_{:0width$}", i + 1);
        let title = format!(
            "{} × {}  {} each  = {} {cur}",
            l.quantity.unwrap_or(0),
            l.mfr_part_number
                .as_deref()
                .or(l.mouser_part_number.as_deref())
                .unwrap_or("?"),
            money(l.unit_price),
            money(l.extended_price),
        );
        let description = [
            l.mouser_part_number.as_deref(),
            l.description.as_deref(),
            l.cart_item_cust_part_number.as_deref(),
        ]
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
        props.insert(
            key.clone(),
            json!({"type": "boolean", "title": title, "description": description, "default": true}),
        );
        keys.push(key);
    }
    props.insert(
        PLACE.into(),
        json!({"type": "boolean",
            "title": format!("Place this order: {} {cur}", money(o.order_total)),
            "description": "A real purchase, charged to the payment method above",
            "default": false}),
    );
    keys.push(PLACE.into());
    let schema = serde_json::from_value(json!({
        "type": "object", "properties": props, "required": keys,
    }))
    // Only boolean fields with a title, a description and a default: always a valid schema.
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
/// every one of `keys` true confirms.
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
    let unticked: Vec<&str> = keys
        .iter()
        .filter(|k| {
            content
                .and_then(|c| c.get(k.as_str()))
                .and_then(Value::as_bool)
                != Some(true)
        })
        .map(String::as_str)
        .collect();
    if unticked.is_empty() {
        Answer::Confirmed
    } else if unticked == [PLACE] {
        Answer::Refused("the user did not tick \"Place this order\"".into())
    } else {
        Answer::Refused(format!("the user unticked {}", unticked.join(", ")))
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
            keys: vec!["line_01".into(), PLACE.into()],
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

    #[test]
    fn only_an_accepted_form_with_every_box_ticked_confirms() {
        let keys: Vec<String> = vec!["line_01".into(), "line_02".into(), PLACE.into()];
        let a = |v: Value| answer(Some(&v), &keys);
        let all = json!({"line_01": true, "line_02": true, "place_order": true});
        assert_eq!(
            a(json!({"action": "accept", "content": all})),
            Answer::Confirmed
        );
        assert_eq!(
            a(
                json!({"action": "accept", "content": {"line_01": true, "line_02": false, "place_order": true}})
            ),
            Answer::Refused("the user unticked line_02".into())
        );
        assert_eq!(
            a(
                json!({"action": "accept", "content": {"line_01": true, "line_02": true, "place_order": false}})
            ),
            Answer::Refused("the user did not tick \"Place this order\"".into())
        );
        for v in [
            json!({"action": "accept", "content": {"place_order": true}}),
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
    fn the_form_has_a_one_line_summary_and_a_ticked_box_per_line() {
        let f = form(&preview(2), Some(650.0));
        assert!(!f.message.contains('\n'), "{}", f.message);
        for want in [
            "Real order at Mouser: 533.78 EUR",
            "2 lines",
            "UPS Worldwide Express Saver 40.00",
            "tax 0.00",
            "PayPal",
            "to Robert L Berlin DE",
            "Ceiling 650.00",
        ] {
            assert!(
                f.message.contains(want),
                "missing {want:?} in {}",
                f.message
            );
        }
        assert_eq!(f.keys, ["line_01", "line_02", "place_order"]);
        let v = serde_json::to_value(&f.schema).unwrap();
        let line = &v["properties"]["line_01"];
        assert_eq!(line["type"], "boolean");
        assert_eq!(line["default"], true);
        assert_eq!(line["title"], "2 × PART-1  161.10 each  = 322.20 EUR");
        assert_eq!(line["description"], "139-PART-1 · RF splitter · WP6");
        assert_eq!(v["properties"]["place_order"]["default"], false);
        assert_eq!(v["required"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn line_keys_sort_in_order_however_many_lines() {
        let f = form(&preview(120), None);
        assert_eq!(f.keys[0], "line_001");
        assert_eq!(f.keys[119], "line_120");
        let mut sorted = f.keys.clone();
        sorted.sort();
        assert_eq!(sorted, f.keys, "clients list fields in key order");
    }
}
