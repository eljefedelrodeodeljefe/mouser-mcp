//! Client for the Mouser Electronics APIs (<https://www.mouser.com/api-hub/>): part search,
//! cart, order history and ordering, one method per endpoint of Mouser's Swagger documents.
//!
//! Mouser issues two keys: the **Search API key** (part search) and the **Cart and Order API
//! key** (everything else). A [`Client`] holds either or both; calls that need a missing key
//! fail with [`Error::MissingKey`] before anything is sent.
//!
//! Placing an order is never a side effect. [`Client::preview_order`] always sends
//! `SubmitOrder: false`; only [`Client::submit_order`] and [`Client::submit_order_from_order`]
//! send `true`, and only after a fresh preview matches what the caller confirmed and stays
//! under [`Config::max_order_total`] (no ceiling configured: no order).

mod client;
mod error;
pub mod models;

pub use client::{
    ApiKey, Client, Config, DEFAULT_BASE_URL, DateFilter, ExpectedOrder, PartSearchOption,
    SearchOption, join_part_numbers, mouser_date,
};
pub use error::{Error, KeyKind, Result};
