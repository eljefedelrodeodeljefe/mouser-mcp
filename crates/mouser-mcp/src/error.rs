use rmcp::model::{ErrorCode, ErrorData};

/// Maps a tool failure onto an MCP error. Anything the caller can fix (a bad argument, an
/// argument Mouser rejected, an order that no longer matches what was confirmed) is
/// *invalid params*, so a model corrects the call instead of retrying it unchanged.
pub fn tool_error(e: anyhow::Error) -> ErrorData {
    if let Some(err) = e.downcast_ref::<mouser::Error>() {
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
