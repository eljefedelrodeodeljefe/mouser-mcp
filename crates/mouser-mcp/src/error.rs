use rmcp::model::{ErrorCode, ErrorData};

/// An order submission whose outcome is unknown (`mouser::Error::SubmitOutcomeUnknown`): the
/// order may exist. A code of its own, in JSON-RPC's range for server errors, so a client
/// cannot take it for a failure that placed nothing.
pub const ORDER_OUTCOME_UNKNOWN: ErrorCode = ErrorCode(-32001);

/// Maps a tool failure onto an MCP error. Anything the caller can fix (a bad argument, an
/// argument Mouser rejected, an order that no longer matches what was confirmed) is
/// *invalid params*, so a model corrects the call instead of retrying it unchanged.
pub fn tool_error(e: anyhow::Error) -> ErrorData {
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
