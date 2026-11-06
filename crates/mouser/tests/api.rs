//! The client against a mock Mouser API (wiremock): request shapes as Mouser's Swagger
//! documents define them, error mapping, key handling and the order safety checks.

use mouser::models::*;
use mouser::{ApiKey, Client, Config, DateFilter, Error, ExpectedOrder, KeyKind};
use serde_json::{Value, json};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SEARCH_KEY: &str = "search-key-0000-0000";
const ORDER_KEY: &str = "order-key-1111-11111";

fn client(server: &MockServer, ceiling: Option<f64>) -> Client {
    Client::new(Config {
        base_url: format!("{}/api", server.uri()),
        search_key: Some(ApiKey::new(SEARCH_KEY).unwrap()),
        order_key: Some(ApiKey::new(ORDER_KEY).unwrap()),
        max_calls_per_minute: 0,
        max_order_total: ceiling,
        ..Config::default()
    })
    .unwrap()
}

fn part(mpn: &str, stock: &str) -> Value {
    json!({
        "MouserPartNumber": format!("139-{mpn}"),
        "ManufacturerPartNumber": mpn,
        "Manufacturer": "Mini-Circuits",
        "Description": "RF power divider",
        "AvailabilityInStock": stock,
        "Availability": format!("{stock} In Stock"),
        "LeadTime": "98 Days",
        "LifecycleStatus": null,
        "Min": "1",
        "Mult": "1",
        "PriceBreaks": [{"Quantity": 1, "Price": "€87,10", "Currency": "EUR"}],
        "ProductDetailUrl": "https://www.mouser.de/ProductDetail/x",
        "SomethingNew": "kept in extra"
    })
}

#[tokio::test]
async fn part_number_search_sends_the_documented_body_and_parses_parts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/search/partnumber"))
        .and(query_param("apiKey", SEARCH_KEY))
        .and(body_json(json!({"SearchByPartRequest": {
            "mouserPartNumber": "ZFRSC-42-S+|BW-S6W2+", "partSearchOptions": "Exact"}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Errors": [],
            "SearchResults": {"NumberOfResult": 2, "Parts": [part("ZFRSC-42-S+", "283"), part("BW-S6W2+", "0")]}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server, None);
    let r = c
        .search_by_part_number(&SearchByPartRequest {
            mouser_part_number: Some(
                mouser::join_part_numbers(&["ZFRSC-42-S+".into(), "BW-S6W2+".into()]).unwrap(),
            ),
            part_search_options: Some("Exact".into()),
            mouser_pays_customs_and_duties: None,
        })
        .await
        .unwrap();
    let parts = r.search_results.unwrap().parts.unwrap();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].availability_in_stock.as_deref(), Some("283"));
    assert_eq!(
        parts[0].price_breaks.as_ref().unwrap()[0]
            .currency
            .as_deref(),
        Some("EUR")
    );
    assert_eq!(
        parts[0].extra.get("SomethingNew"),
        Some(&json!("kept in extra"))
    );
}

#[tokio::test]
async fn keyword_search_and_manufacturer_list_use_v2_where_mouser_deprecated_v1() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/search/keywordandmanufacturer"))
        .and(body_json(json!({"SearchByKeywordMfrNameRequest": {
            "manufacturerName": "Mini-Circuits", "keyword": "splitter", "records": 5, "searchOptions": "InStock"}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"Errors": [], "SearchResults": {"NumberOfResult": 0, "Parts": []}})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v2/search/manufacturerlist"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"Errors": [],
            "MouserManufacturerList": {"Count": 1, "ManufacturerList": [{"ManufacturerName": "Mini-Circuits"}]}})))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server, None);
    c.search_by_keyword_and_manufacturer(&SearchByKeywordMfrNameRequest {
        manufacturer_name: Some("Mini-Circuits".into()),
        keyword: "splitter".into(),
        records: Some(5),
        search_options: Some("InStock".into()),
        ..Default::default()
    })
    .await
    .unwrap();
    let m = c.manufacturer_list().await.unwrap();
    assert_eq!(m.mouser_manufacturer_list.unwrap().count, Some(1));
}

#[tokio::test]
async fn arguments_mouser_would_reject_fail_before_any_request() {
    let server = MockServer::start().await; // no mocks: any request would 404
    let c = client(&server, None);
    let many: Vec<String> = (0..11).map(|i| format!("PART-{i}")).collect();
    assert!(matches!(
        mouser::join_part_numbers(&many),
        Err(Error::InvalidArgument(_))
    ));
    assert!(matches!(
        mouser::join_part_numbers(&["ab".into()]),
        Err(Error::InvalidArgument(_))
    ));
    assert!(matches!(
        mouser::join_part_numbers(&["a|bcd".into()]),
        Err(Error::InvalidArgument(_))
    ));
    let r = c
        .search_by_keyword(&SearchByKeywordRequest {
            keyword: "x".into(),
            records: Some(51),
            ..Default::default()
        })
        .await;
    assert!(matches!(r, Err(Error::InvalidArgument(_))), "{r:?}");
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn a_missing_key_fails_without_a_request() {
    let server = MockServer::start().await;
    let c = Client::new(Config {
        base_url: format!("{}/api", server.uri()),
        search_key: Some(ApiKey::new(SEARCH_KEY).unwrap()),
        max_calls_per_minute: 0,
        ..Config::default()
    })
    .unwrap();
    assert!(c.has_key(KeyKind::Search) && !c.has_key(KeyKind::Order));
    let r = c.get_cart("cart-1", None, None).await;
    assert!(matches!(r, Err(Error::MissingKey(KeyKind::Order))), "{r:?}");
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn mouser_errors_in_a_200_reply_become_api_errors() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/search/keyword"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Errors": [{"Id": 0, "Code": "InvalidCharacters", "Message": "Keyword contains invalid characters", "PropertyName": "keyword"}],
            "SearchResults": null
        })))
        .mount(&server)
        .await;
    let r = client(&server, None)
        .search_by_keyword(&SearchByKeywordRequest {
            keyword: "<>".into(),
            ..Default::default()
        })
        .await;
    match r {
        Err(Error::Api(e)) => assert_eq!(e[0].code.as_deref(), Some("InvalidCharacters")),
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn rate_limits_and_http_errors_are_typed_and_never_leak_the_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/search/keyword"))
        .respond_with(ResponseTemplate::new(429).set_body_string("slow down"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/order/countries"))
        .respond_with(
            ResponseTemplate::new(403)
                .set_body_string(format!("bad key {ORDER_KEY} for /api/v1/order/countries")),
        )
        .mount(&server)
        .await;
    let c = client(&server, None);
    let r = c
        .search_by_keyword(&SearchByKeywordRequest {
            keyword: "x".into(),
            ..Default::default()
        })
        .await;
    assert!(matches!(r, Err(Error::RateLimited(_))), "{r:?}");
    let e = c.order_countries(None).await.unwrap_err();
    let msg = format!("{e} {e:?}");
    assert!(matches!(e, Error::Http { status: 403, .. }));
    assert!(!msg.contains(ORDER_KEY), "{msg}");
    // and a transport failure carries no URL (the key is in its query string)
    let dead = Client::new(Config {
        base_url: "http://127.0.0.1:9/api".into(),
        search_key: Some(ApiKey::new(SEARCH_KEY).unwrap()),
        max_calls_per_minute: 0,
        ..Config::default()
    })
    .unwrap();
    let e = dead.manufacturer_list().await.unwrap_err();
    assert!(matches!(e, Error::Transport(_)));
    assert!(!format!("{e} {e:?}").contains(SEARCH_KEY));
    assert_eq!(
        format!("{:?}", ApiKey::new(SEARCH_KEY).unwrap()),
        "ApiKey(***)"
    );
    assert!(ApiKey::new("short").is_err());
    assert!(ApiKey::new("with space in the key 123").is_err());
}

#[tokio::test]
async fn cart_calls_use_the_order_key_and_query_parameters() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/cart/items/insert"))
        .and(query_param("apiKey", ORDER_KEY))
        .and(query_param("currencyCode", "EUR"))
        .and(body_json(json!({"CartKey": "cart-1", "CartItems": [{"MouserPartNumber": "139-ZFRSC-42-S+", "Quantity": 2}]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"Errors": [], "CartKey": "cart-1",
            "CartItems": [{"MouserPartNumber": "139-ZFRSC-42-S+", "Quantity": 2, "UnitPrice": 87.1}], "MerchandiseTotal": 174.2})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/cart/item/remove"))
        .and(query_param("cartKey", "cart-1"))
        .and(query_param("mouserPartNumber", "139-ZFRSC-42-S+"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"Errors": [], "CartKey": "cart-1", "CartItems": []})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server, None);
    let r = c
        .add_cart_items(
            &CartItemRequestRoot {
                cart_key: Some("cart-1".into()),
                cart_items: Some(vec![CartItemRequest {
                    mouser_part_number: "139-ZFRSC-42-S+".into(),
                    quantity: 2,
                    ..Default::default()
                }]),
                ..Default::default()
            },
            None,
            Some("EUR"),
        )
        .await
        .unwrap();
    assert_eq!(r.merchandise_total, Some(174.2));
    c.remove_cart_item("cart-1", "139-ZFRSC-42-S+", None, None)
        .await
        .unwrap();
}

#[tokio::test]
async fn order_history_dates_are_sent_the_way_mouser_wants_them() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/orderhistory/ByDateRange"))
        .and(query_param("startDate", "1/30/2026"))
        .and(query_param("endDate", "11/6/2026"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"Errors": [], "NumberOfOrders": 0, "OrderHistoryItems": []})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/orderhistory/ByDateFilter"))
        .and(query_param("dateFilter", "ThisYear"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"Errors": [], "NumberOfOrders": 0})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server, None);
    c.order_history_by_date_range("2026-01-30", "11/6/2026")
        .await
        .unwrap();
    c.order_history_by_date_filter(DateFilter::ThisYear)
        .await
        .unwrap();
    assert!(mouser::mouser_date("2026-13-01").is_err() && mouser::mouser_date("tomorrow").is_err());
    assert!(
        mouser::mouser_date("2026-02-30").is_err(),
        "a date that does not exist"
    );
    assert_eq!(mouser::mouser_date("01/05/2026").unwrap(), "1/5/2026");
}

fn order_request() -> OrderRequestRootType {
    OrderRequestRootType {
        order: Some(OrderRequestType {
            cart_key: "cart-1".into(),
            currency_code: "EUR".into(),
            // a caller trying to submit through the preview path
            submit_order: Some(true),
            ..Default::default()
        }),
    }
}

fn order_reply(total: f64, qty: i64) -> Value {
    json!({"Order": {"Errors": [], "CurrencyCode": "EUR", "OrderTotal": total, "SubmitOrder": false,
        "OrderLines": [{"MouserPartNumber": "139-ZFRSC-42-S+", "Quantity": qty}]}})
}

fn expected(total: f64) -> ExpectedOrder {
    ExpectedOrder {
        currency_code: "EUR".into(),
        order_total: total,
        lines: vec![("139-ZFRSC-42-S+".into(), 2)],
    }
}

async fn mount_order(server: &MockServer, submit: bool, reply: Value, times: u64) {
    let mut body = json!({"Order": {"CartKey": "cart-1", "CurrencyCode": "EUR"}});
    body["Order"]["SubmitOrder"] = json!(submit);
    Mock::given(method("POST"))
        .and(path("/api/v1/order"))
        .and(body_json(body))
        .respond_with(ResponseTemplate::new(200).set_body_json(reply))
        .expect(times)
        .mount(server)
        .await;
}

#[tokio::test]
async fn preview_never_submits_even_if_the_request_says_so() {
    let server = MockServer::start().await;
    mount_order(&server, false, order_reply(200.0, 2), 1).await;
    mount_order(&server, true, order_reply(200.0, 2), 0).await;
    let r = client(&server, Some(500.0))
        .preview_order(&order_request())
        .await
        .unwrap();
    assert_eq!(r.order.unwrap().order_total, Some(200.0));
}

#[tokio::test]
async fn submit_needs_a_ceiling_and_a_matching_preview() {
    // no ceiling configured: refused after the preview, nothing submitted
    let server = MockServer::start().await;
    mount_order(&server, false, order_reply(200.0, 2), 1).await;
    mount_order(&server, true, order_reply(200.0, 2), 0).await;
    let e = client(&server, None)
        .submit_order(&order_request(), &expected(200.0))
        .await
        .unwrap_err();
    assert!(matches!(e, Error::OrderMismatch(_)), "{e:?}");

    // the total moved since the user confirmed it; a different quantity; above the ceiling
    for (reply, exp, ceiling) in [
        (order_reply(210.0, 2), expected(200.0), 500.0),
        (order_reply(200.0, 3), expected(200.0), 500.0),
        (order_reply(200.0, 2), expected(200.0), 150.0),
    ] {
        let server = MockServer::start().await;
        mount_order(&server, false, reply, 1).await;
        mount_order(&server, true, order_reply(0.0, 0), 0).await;
        let e = client(&server, Some(ceiling))
            .submit_order(&order_request(), &exp)
            .await
            .unwrap_err();
        assert!(matches!(e, Error::OrderMismatch(_)), "{e:?}");
    }

    // all of it matches: exactly one submission
    let server = MockServer::start().await;
    mount_order(&server, false, order_reply(200.0, 2), 1).await;
    mount_order(&server, true, order_reply(200.0, 2), 1).await;
    client(&server, Some(500.0))
        .submit_order(&order_request(), &expected(200.0))
        .await
        .unwrap();
}
