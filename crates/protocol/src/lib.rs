//! Nexum JSON-RPC 2.0 protocol primitives.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

pub const JSONRPC_VERSION: &str = "2.0";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

impl RpcRequest {
    pub fn new(id: impl Into<Value>, method: impl Into<String>, params: Option<Value>) -> Self {
        Self { jsonrpc: JSONRPC_VERSION.into(), id: Some(id.into()), method: method.into(), params }
    }

    pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self {
        Self { jsonrpc: JSONRPC_VERSION.into(), id: None, method: method.into(), params }
    }

    pub fn validate(&self) -> Result<(), RpcError> {
        if self.jsonrpc != JSONRPC_VERSION {
            return Err(RpcError::InvalidRequest("jsonrpc must be 2.0".into()));
        }
        if self.method.trim().is_empty() {
            return Err(RpcError::InvalidRequest("method must not be empty".into()));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcErrorObject>,
}

impl RpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self { jsonrpc: JSONRPC_VERSION.into(), id, result: Some(result), error: None }
    }

    pub fn error(id: Option<Value>, error: RpcErrorObject) -> Self {
        Self { jsonrpc: JSONRPC_VERSION.into(), id, result: None, error: Some(error) }
    }

    pub fn is_success(&self) -> bool { self.error.is_none() }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RpcErrorObject {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcErrorObject {
    pub fn parse_error(message: impl Into<String>) -> Self { Self { code: -32700, message: message.into(), data: None } }
    pub fn invalid_request(message: impl Into<String>) -> Self { Self { code: -32600, message: message.into(), data: None } }
    pub fn method_not_found(method: impl Into<String>) -> Self { Self { code: -32601, message: format!("method not found: {}", method.into()), data: None } }
    pub fn invalid_params(message: impl Into<String>) -> Self { Self { code: -32602, message: message.into(), data: None } }
    pub fn internal_error(message: impl Into<String>) -> Self { Self { code: -32603, message: message.into(), data: None } }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RpcError {
    Parse(String),
    InvalidRequest(String),
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(message) => write!(f, "JSON-RPC parse error: {message}"),
            Self::InvalidRequest(message) => write!(f, "invalid JSON-RPC request: {message}"),
        }
    }
}

impl std::error::Error for RpcError {}

pub fn parse_request(input: &str) -> Result<RpcRequest, RpcError> {
    let request: RpcRequest = serde_json::from_str(input)
        .map_err(|error| RpcError::Parse(error.to_string()))?;
    request.validate()?;
    Ok(request)
}

pub fn serialize_response(response: &RpcResponse) -> Result<String, RpcError> {
    serde_json::to_string(response).map_err(|error| RpcError::Parse(error.to_string()))
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EventEnvelope {
    pub event: String,
    pub data: Value,
}

impl EventEnvelope {
    pub fn new(event: impl Into<String>, data: Value) -> Self {
        Self { event: event.into(), data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_json_rpc_request() {
        let request = parse_request(r#"{"jsonrpc":"2.0","id":1,"method":"task.list","params":{"limit":10}}"#).unwrap();
        assert_eq!(request.method, "task.list");
        assert_eq!(request.id, Some(json!(1)));
    }

    #[test]
    fn rejects_wrong_protocol_version() {
        let error = parse_request(r#"{"jsonrpc":"1.0","id":1,"method":"task.list"}"#).unwrap_err();
        assert!(matches!(error, RpcError::InvalidRequest(_)));
    }

    #[test]
    fn serializes_success_response() {
        let response = RpcResponse::success(Some(json!(1)), json!({ "tasks": [] }));
        let encoded = serialize_response(&response).unwrap();
        assert!(encoded.contains(r#""jsonrpc":"2.0""#));
        assert!(encoded.contains(r#""result":{"tasks":[]}"#));
    }

    #[test]
    fn notifications_have_no_id() {
        let request = RpcRequest::notification("task.subscribe", None);
        assert_eq!(request.id, None);
    }

    #[test]
    fn event_envelope_is_transport_neutral() {
        let event = EventEnvelope::new("task.progress", json!({"downloaded_bytes": 42}));
        assert_eq!(event.event, "task.progress");
    }
}
