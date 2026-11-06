use rmcp::model::{ErrorCode, ErrorData};

/// An order submission whose outcome is unknown (`mouser::Error::SubmitOutcomeUnknown`): the
/// order may exist. A code of its own, in JSON-RPC's range for server errors, so a client
/// cannot take it for a failure that placed nothing.
pub const ORDER_OUTCOME_UNKNOWN: ErrorCode = ErrorCode(-32001);

/// `add_cart_items` wrote the cart and could not confirm the result
/// ([`crate::tools::CartUnverified`]): not *invalid params*, because the same call again would
/// add the quantities twice.
pub const CART_UNVERIFIED: ErrorCode = ErrorCode(-32002);

/// Maps a tool failure onto an MCP error. Anything the caller can fix (a bad argument, an
/// argument Mouser rejected, an order that no longer matches what was confirmed) is
/// *invalid params*, so a model corrects the call instead of retrying it unchanged.
pub fn tool_error(e: anyhow::Error) -> ErrorData {
    if let Some(err) = e.downcast_ref::<crate::tools::CartUnverified>() {
        tracing::warn!(error = %err, "cart written but not verified");
        return ErrorData::new(
            CART_UNVERIFIED,
            format!(
                "{err}. Do not call add_cart_items again: that adds the quantities a second time. Read the cart with get_cart and correct it with update_cart_items or update_cart"
            ),
            None,
        );
    }
    if let Some(err) = e.downcast_ref::<mouser::Error>() {
        if let mouser::Error::SubmitOutcomeUnknown(source) = err {
            return ErrorData::new(
                ORDER_OUTCOME_UNKNOWN,
                format!(
                    "the order may have been placed: the submission was sent and no answer came back ({source}). Do not submit again before checking: look for the order with order_history_by_date_filter (Today) or get_order, and tell the user"
                ),
                None,
            );
        }
        let code = match err {
            mouser::Error::InvalidArgument(_)
            | mouser::Error::Api(_)
            | mouser::Error::OrderMismatch(_) => ErrorCode::INVALID_PARAMS,
            mouser::Error::MissingKey(_) => ErrorCode::INVALID_REQUEST,
            _ => ErrorCode::INTERNAL_ERROR,
        };
        return ErrorData::new(code, err.to_string(), None);
    }
    ErrorData::new(ErrorCode::INVALID_PARAMS, e.to_string(), None)
}
