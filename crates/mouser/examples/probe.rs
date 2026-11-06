//! Read-only check of a live Mouser key: one part-number search.
//!
//! ```sh
//! MOUSER_SEARCH_API_KEY_FILE=~/.config/mouser/search-api-key \
//!   cargo run -p mouser --example probe -- ZFRSC-42-S+ BW-S6W2+
//! ```
//!
//! Output goes through `tracing` (stderr), like everything in this workspace.

use mouser::models::SearchByPartRequest;
use mouser::{ApiKey, Client, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();
    let path = std::env::var("MOUSER_SEARCH_API_KEY_FILE")?;
    let parts: Vec<String> = std::env::args().skip(1).collect();
    let client = Client::new(Config {
        search_key: Some(ApiKey::from_file(path)?),
        ..Config::default()
    })?;
    let r = client
        .search_by_part_number(&SearchByPartRequest {
            mouser_part_number: Some(mouser::join_part_numbers(&parts)?),
            part_search_options: Some("Exact".into()),
            mouser_pays_customs_and_duties: None,
        })
        .await?;
    let res = r.search_results.unwrap_or_default();
    tracing::info!(results = res.number_of_result, "search ok");
    for p in res.parts.unwrap_or_default() {
        let price = p
            .price_breaks
            .as_deref()
            .and_then(|b| b.first())
            .map(|b| {
                format!(
                    "{} @ {}",
                    b.price.clone().unwrap_or_default(),
                    b.quantity.unwrap_or(0)
                )
            })
            .unwrap_or_default();
        tracing::info!(
            mouser = p.mouser_part_number.as_deref().unwrap_or(""),
            mpn = p.manufacturer_part_number.as_deref().unwrap_or(""),
            in_stock = p.availability_in_stock.as_deref().unwrap_or("0"),
            lead_time = p.lead_time.as_deref().unwrap_or(""),
            price,
            "part"
        );
    }
    Ok(())
}
