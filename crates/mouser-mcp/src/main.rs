use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;

mod confirm;
mod error;
mod params;
mod server;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    // Log to stderr — stdout is reserved for MCP JSON-RPC messages.
    tracing_subscriber::fmt()
        .with_env_filter(log_filter(std::env::var("RUST_LOG").ok().as_deref()))
        .with_writer(std::io::stderr)
        .init();

    let server = server::MouserServer::from_env()?;
    tracing::info!(
        search = server.has_search(),
        order = server.has_order(),
        "mouser-mcp starting"
    );

    let svc = server.serve(stdio()).await?;
    svc.waiting().await?;

    Ok(())
}

/// `RUST_LOG` if set, else `info`: the order trail (submissions, orders placed, refusals) is
/// logged at info and warn, and must reach the log without any configuration.
fn log_filter(rust_log: Option<&str>) -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(rust_log.unwrap_or(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_rust_log_the_order_trail_is_logged() {
        assert_eq!(log_filter(None).max_level_hint(), Some(LevelFilter::INFO));
        assert_eq!(
            log_filter(Some("")).max_level_hint(),
            Some(LevelFilter::INFO)
        );
        assert_eq!(
            log_filter(Some("mouser=debug")).max_level_hint(),
            Some(LevelFilter::DEBUG)
        );
        assert_eq!(
            log_filter(Some("warn")).max_level_hint(),
            Some(LevelFilter::WARN)
        );
    }
}
