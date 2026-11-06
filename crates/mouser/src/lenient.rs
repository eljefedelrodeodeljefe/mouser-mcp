//! Deserializers for fields where Mouser's replies differ from its Swagger documents.

use serde::{Deserialize, Deserializer};

/// A string or a number, as Mouser sends them.
#[derive(Deserialize)]
#[serde(untagged)]
enum Text {
    S(String),
    I(i64),
}

/// An enum field: documented as a string ("Residential", …), sent by the API as the enum's
/// number (`1`). Either is kept as text.
pub fn enum_text<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<Text>::deserialize(d)?.map(|t| match t {
        Text::S(s) => s,
        Text::I(i) => i.to_string(),
    }))
}
