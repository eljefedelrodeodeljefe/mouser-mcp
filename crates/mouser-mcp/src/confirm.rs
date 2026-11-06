//! The user's confirmation of an order, asked through MCP elicitation.
//!
//! The confirmation comes from the person at the client, not from the model: the client
//! shows a form with the amount and a summary and returns what the user chose. A model can
//! neither see the form nor answer it.
//!
//! Two protocol paths:
//!
//! - **2026-07-28 and later** (multi round-trip, SEP-2322): the tool returns an
//!   `InputRequiredResult` holding the form; the client asks the user and calls the tool
//!   again with the answer and the echoed `requestState`. The state is only a handle to a
//!   pending confirmation kept here: single use, expiring, and bound to the exact arguments of
//!   the first call, so an answer cannot be replayed onto another order.
//! - **earlier versions**: the server sends `elicitation/create` while the tool call is open.
//!
//! A client that supports neither cannot confirm, and no order is placed.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use mouser::models::OrderRoot;
use rmcp::model::{ElicitationSchema, JsonObject};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

/// How long the user has to answer the form.
pub const TTL: Duration = Duration::from_secs(10 * 60);

/// Key of the form in `inputRequests` / `inputResponses`.
pub const INPUT_KEY: &str = "confirm_order";

/// What the form asks.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OrderConfirmation {
    /// Place this order now; it is a real purchase.
    #[schemars(title = "Place this order", description = "This is a real purchase")]
    pub place_order: bool,
}
rmcp::elicit_safe!(OrderConfirmation);

pub fn schema() -> ElicitationSchema {
    // The derive above yields an object schema with one boolean: it always converts.
    ElicitationSchema::from_type::<OrderConfirmation>().expect("confirmation schema")
}

fn money(v: Option<f64>) -> String {
    v.map_or_else(|| "?".into(), |v| format!("{v:.2}"))
}

/// The form's text: what is ordered, what it costs, where it goes and how it is paid.
pub fn summary(tool: &str, preview: &OrderRoot, ceiling: Option<f64>) -> String {
    let Some(o) = preview.order.as_ref() else {
        return format!("{tool}: Mouser returned a preview without an order.");
    };
    let cur = o.currency_code.as_deref().unwrap_or("?");
    let mut s = format!(
        "Place a real order at Mouser for {} {cur}?\n\n",
        money(o.order_total)
    );
    for l in o.order_lines.as_deref().unwrap_or(&[]) {
        s += &format!(
            "  {} × {}  {}  à {}  = {} {cur}\n",
            l.quantity.unwrap_or(0),
            l.mouser_part_number.as_deref().unwrap_or("?"),
            l.mfr_part_number.as_deref().unwrap_or(""),
            money(l.unit_price),
            money(l.extended_price),
        );
    }
    s += &format!("\nMerchandise {} {cur}", money(o.merchandise_total));
    if let Some(m) = o.shipping_method.as_ref() {
        s += &format!(
            "\nShipping {} ({})",
            money(m.primary_shipping_rate),
            m.primary_method.as_deref().unwrap_or("?")
        );
    }
    if o.additional_fees_total.is_some_and(|f| f != 0.0) {
        s += &format!("\nFees {}", money(o.additional_fees_total));
    }
    if o.tax_amount.is_some() {
        s += &format!("\nTax {}", money(o.tax_amount));
    }
    s += &format!("\nTotal {} {cur}", money(o.order_total));
    if let Some(c) = ceiling {
        s += &format!(" (ceiling {c:.2})");
    }
    if let Some(a) = o.shipping_address.as_ref() {
        let parts: Vec<&str> = [
            a.company.as_deref(),
            a.first_name.as_deref(),
            a.last_name.as_deref(),
            a.city.as_deref(),
            a.country_code.as_deref(),
        ]
        .into_iter()
        .flatten()
        .filter(|p| !p.is_empty())
        .collect();
        if !parts.is_empty() {
            s += &format!("\nShip to {}", parts.join(", "));
        }
    }
    if let Some(p) = o.payment_method.as_ref().and_then(|p| p.name.as_deref()) {
        s += &format!("\nPayment {p}");
    }
    s
}

/// What the user answered.
#[derive(Debug, PartialEq, Eq)]
pub enum Answer {
    Confirmed,
    Refused(String),
}

/// Reads an elicitation result (`{"action": …, "content": {…}}`): only an accepted form with
/// `place_order: true` confirms.
pub fn answer(response: Option<&Value>) -> Answer {
    let Some(r) = response else {
        return Answer::Refused("no answer to the confirmation form".into());
    };
    match r.get("action").and_then(Value::as_str) {
        Some("accept") => {}
        Some(other) => return Answer::Refused(format!("the user chose {other:?}")),
        None => return Answer::Refused("the confirmation answer has no action".into()),
    }
    match r.pointer("/content/place_order").and_then(Value::as_bool) {
        Some(true) => Answer::Confirmed,
        _ => Answer::Refused("the user did not tick \"Place this order\"".into()),
    }
}

struct Pending {
    tool: String,
    args: JsonObject,
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
    pub fn open(&self, tool: &str, args: JsonObject) -> String {
        let id = format!("order-{}", self.next.fetch_add(1, Ordering::Relaxed));
        let mut p = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        p.retain(|_, v| v.created.elapsed() < TTL);
        p.insert(
            id.clone(),
            Pending {
                tool: tool.into(),
                args,
                created: Instant::now(),
            },
        );
        id
    }

    /// Takes the confirmation `id` (single use); it must be for this tool and these exact
    /// arguments and not have expired.
    pub fn take(&self, id: &str, tool: &str, args: &JsonObject) -> Result<(), String> {
        let p = self
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(id)
            .ok_or("unknown or already used confirmation; preview and submit again")?;
        if p.created.elapsed() >= TTL {
            return Err("the confirmation expired; preview and submit again".into());
        }
        if p.tool != tool || &p.args != args {
            return Err("the confirmation was for another order; preview and submit again".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn args(total: f64) -> JsonObject {
        json!({"expected": {"order_total": total}})
            .as_object()
            .unwrap()
            .clone()
    }

    #[test]
    fn a_confirmation_is_single_use_and_bound_to_its_order() {
        let s = Store::default();
        let id = s.open("submit_order", args(548.21));
        assert!(s.take(&id, "submit_order", &args(999.0)).is_err());
        // a failed take consumed it: no second try with the right arguments either
        assert!(s.take(&id, "submit_order", &args(548.21)).is_err());
        let id = s.open("submit_order", args(548.21));
        assert!(
            s.take(&id, "submit_order_from_order", &args(548.21))
                .is_err()
        );
        let id = s.open("submit_order", args(548.21));
        assert!(s.take(&id, "submit_order", &args(548.21)).is_ok());
        assert!(s.take(&id, "submit_order", &args(548.21)).is_err());
        assert!(s.take("order-99", "submit_order", &args(548.21)).is_err());
    }

    #[test]
    fn only_an_accepted_ticked_form_confirms() {
        let a = |v: Value| answer(Some(&v));
        assert_eq!(
            a(json!({"action": "accept", "content": {"place_order": true}})),
            Answer::Confirmed
        );
        for v in [
            json!({"action": "accept", "content": {"place_order": false}}),
            json!({"action": "accept", "content": {}}),
            json!({"action": "accept"}),
            json!({"action": "decline"}),
            json!({"action": "cancel", "content": {"place_order": true}}),
            json!({"content": {"place_order": true}}),
        ] {
            assert!(matches!(a(v.clone()), Answer::Refused(_)), "{v}");
        }
        assert!(matches!(answer(None), Answer::Refused(_)));
    }

    #[test]
    fn the_summary_names_amount_lines_shipping_and_address() {
        let preview: OrderRoot = serde_json::from_value(json!({"Order": {
            "CurrencyCode": "EUR", "MerchandiseTotal": 493.78, "TaxAmount": 98.08, "OrderTotal": 610.36,
            "ShippingMethod": {"PrimaryMethod": "UPS Standard", "PrimaryShippingRate": 18.5},
            "ShippingAddress": {"FirstName": "Robert", "City": "Berlin", "CountryCode": "DE"},
            "PaymentMethod": {"Name": "Credit card"},
            "OrderLines": [{"MouserPartNumber": "139-ZX10R-2-183-S+", "MfrPartNumber": "ZX10R-2-183-S+",
                "Quantity": 2, "UnitPrice": 161.1, "ExtendedPrice": 322.2}]}}))
        .unwrap();
        let s = summary("submit_order", &preview, Some(700.0));
        for want in [
            "Place a real order at Mouser for 610.36 EUR?",
            "2 × 139-ZX10R-2-183-S+  ZX10R-2-183-S+  à 161.10  = 322.20 EUR",
            "Merchandise 493.78 EUR",
            "Shipping 18.50 (UPS Standard)",
            "Tax 98.08",
            "Total 610.36 EUR (ceiling 700.00)",
            "Ship to Robert, Berlin, DE",
            "Payment Credit card",
        ] {
            assert!(s.contains(want), "missing {want:?} in\n{s}");
        }
    }

    #[test]
    fn the_form_is_one_boolean() {
        let v = serde_json::to_value(schema()).unwrap();
        assert_eq!(v["type"], "object");
        assert_eq!(v["properties"]["place_order"]["type"], "boolean", "{v:#}");
    }
}
