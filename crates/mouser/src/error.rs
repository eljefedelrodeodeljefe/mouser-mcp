use crate::models::ErrorEntity;

/// Which Mouser API key an operation needs. Mouser issues them separately: the Search API key
/// for part search, the Cart and Order API key for carts, order history and orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    Search,
    Order,
}

impl std::fmt::Display for KeyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            KeyKind::Search => "Search API key",
            KeyKind::Order => "Cart and Order API key",
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation needs a key this client was not given.
    #[error("no {0} configured")]
    MissingKey(KeyKind),

    /// A caller argument Mouser would reject, or one this library refuses (see the message).
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// Mouser answered, and its `Errors` list is not empty.
    #[error("Mouser API error: {}", format_errors(.0))]
    Api(Vec<ErrorEntity>),

    /// Mouser refused the call for its rate: wait and retry.
    #[error("Mouser rate limit: {0}")]
    RateLimited(String),

    /// A non-success HTTP status. The body is kept as Mouser sent it (it never holds the key).
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    /// The request did not complete (DNS, TLS, timeout). The URL is stripped: it carries the key.
    #[error("transport: {0}")]
    Transport(String),

    /// Mouser's reply did not match the documented shape.
    #[error("unexpected response: {0}")]
    Decode(String),

    /// `submit_order` refused: the order Mouser would place now differs from the one confirmed.
    #[error("order not submitted: {0}")]
    OrderMismatch(String),
}

fn format_errors(errs: &[ErrorEntity]) -> String {
    errs.iter()
        .map(|e| {
            let code = e.code.as_deref().unwrap_or("?");
            let msg = e.message.as_deref().unwrap_or("");
            match e.property_name.as_deref() {
                Some(p) if !p.is_empty() => format!("{code} ({p}): {msg}"),
                _ => format!("{code}: {msg}"),
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        // reqwest includes the URL in its message, and Mouser takes the key in the query string.
        Error::Transport(e.without_url().to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
