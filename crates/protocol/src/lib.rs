//! Nexum JSON-RPC 2.0 protocol primitives.

use nexum_core::Core;
use nexum_domain::{Destination, DownloadSource, TaskId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

pub const JSONRPC_VERSION: &str = "2.0";
pub const ERR_TASK_NOT_FOUND: i32 = -32004;
pub const ERR_INTERNAL: i32 = -32603;

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
        if self.jsonrpc != JSONRPC_VERSION { return Err(RpcError::InvalidRequest("jsonrpc must be 2.0".into())); }
        if self.method.trim().is_empty() { return Err(RpcError::InvalidRequest("method must not be empty".into())); }
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
    pub fn success(id: Option<Value>, result: Value) -> Self { Self { jsonrpc: JSONRPC_VERSION.into(), id, result: Some(result), error: None } }
    pub fn error(id: Option<Value>, error: RpcErrorObject) -> Self { Self { jsonrpc: JSONRPC_VERSION.into(), id, result: None, error: Some(error) } }
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
    pub fn task_not_found(message: impl Into<String>) -> Self { Self { code: ERR_TASK_NOT_FOUND, message: message.into(), data: None } }
    pub fn internal_error(message: impl Into<String>) -> Self { Self { code: ERR_INTERNAL, message: message.into(), data: None } }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RpcError { Parse(String), InvalidRequest(String) }
impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self { Self::Parse(message) => write!(f, "JSON-RPC parse error: {message}"), Self::InvalidRequest(message) => write!(f, "invalid JSON-RPC request: {message}") }
    }
}
impl std::error::Error for RpcError {}
pub fn parse_request(input: &str) -> Result<RpcRequest, RpcError> {
    let request: RpcRequest = serde_json::from_str(input).map_err(|error| RpcError::Parse(error.to_string()))?;
    request.validate()?;
    Ok(request)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TaskView {
    pub id: String, pub source: String, pub destination: String, pub state: String,
    pub downloaded_bytes: u64, pub total_bytes: Option<u64>,
}
impl From<&nexum_task::DownloadTask> for TaskView {
    fn from(task: &nexum_task::DownloadTask) -> Self {
        Self { id: task.id.to_string(), source: task.source.as_str().to_owned(), destination: task.destination.as_str().to_owned(),
            state: format!("{:?}", task.state), downloaded_bytes: task.progress.downloaded_bytes, total_bytes: task.progress.total_bytes }
    }
}

pub struct RpcDispatcher;
impl RpcDispatcher {
    pub fn dispatch<R: nexum_core::nexum_storage::TaskRepository>(core: &mut Core<R>, request: &RpcRequest) -> Option<RpcResponse> {
        let id = request.id.clone();
        let params = request.params.clone().unwrap_or(Value::Null);
        let result = match request.method.as_str() {
            "task.get" => Self::task_get(core, &params),
            "task.list" => Self::task_list(core),
            "task.create" => Self::task_create(core, &params),
            "task.queue" => Self::task_queue(core, &params),
            "task.start" => Self::task_start(core),
            "task.pause" => Self::task_pause(core, &params),
            "task.resume" => Self::task_resume(core, &params),
            "task.remove" => Self::task_remove(core, &params),
            _ => Err(DispatchError::MethodNotFound),
        };
        if id.is_none() { return None; }
        Some(match result {
            Ok(value) => RpcResponse::success(id, value),
            Err(DispatchError::InvalidParams(message)) => RpcResponse::error(id, RpcErrorObject::invalid_params(message)),
            Err(DispatchError::TaskNotFound(message)) => RpcResponse::error(id, RpcErrorObject::task_not_found(message)),
            Err(DispatchError::Internal(message)) => RpcResponse::error(id, RpcErrorObject::internal_error(message)),
            Err(DispatchError::MethodNotFound) => RpcResponse::error(id, RpcErrorObject::method_not_found(&request.method)),
        })
    }

    fn task_get<R: nexum_core::nexum_storage::TaskRepository>(core: &Core<R>, params: &Value) -> Result<Value, DispatchError> {
        let id = params.get("id").and_then(Value::as_str).ok_or_else(|| DispatchError::InvalidParams("missing id".into()))?;
        let task = core.tasks.get(&TaskId::from(id)).ok_or_else(|| DispatchError::TaskNotFound("task not found".into()))?;
        serde_json::to_value(TaskView::from(task)).map_err(|e| DispatchError::Internal(e.to_string()))
    }
    fn task_list<R: nexum_core::nexum_storage::TaskRepository>(core: &Core<R>) -> Result<Value, DispatchError> {
        let tasks: Vec<TaskView> = core.tasks.list().map(TaskView::from).collect();
        serde_json::to_value(tasks).map_err(|e| DispatchError::Internal(e.to_string()))
    }
    fn task_create<R: nexum_core::nexum_storage::TaskRepository>(core: &mut Core<R>, params: &Value) -> Result<Value, DispatchError> {
        let id = params.get("id").and_then(Value::as_str).ok_or_else(|| DispatchError::InvalidParams("missing id".into()))?;
        let source = params.get("source").and_then(Value::as_str).ok_or_else(|| DispatchError::InvalidParams("missing source".into()))?;
        let destination = params.get("destination").and_then(Value::as_str).ok_or_else(|| DispatchError::InvalidParams("missing destination".into()))?;
        let task = core.create_task(TaskId::from(id), DownloadSource::new(source), Destination::new(destination))
            .map_err(|e| DispatchError::Internal(format!("{e:?}")))?;
        serde_json::to_value(TaskView::from(&task)).map_err(|e| DispatchError::Internal(e.to_string()))
    }
    fn task_queue<R: nexum_core::nexum_storage::TaskRepository>(core: &mut Core<R>, params: &Value) -> Result<Value, DispatchError> {
        let id = params.get("id").and_then(Value::as_str).ok_or_else(|| DispatchError::InvalidParams("missing id".into()))?;
        core.queue_task(&TaskId::from(id), nexum_core::nexum_scheduler::Priority::NORMAL)
            .map_err(|e| match e { nexum_core::nexum_scheduler::SchedulerError::Task(nexum_core::nexum_task::TaskServiceError::NotFound(_)) => DispatchError::TaskNotFound("task not found".into()), _ => DispatchError::Internal(format!("{e:?}")) })?;
        Ok(Value::Bool(true))
    }
    fn task_start<R: nexum_core::nexum_storage::TaskRepository>(core: &mut Core<R>) -> Result<Value, DispatchError> {
        let id = core.start_next().map_err(|e| DispatchError::Internal(format!("{e:?}")))?
            .ok_or_else(|| DispatchError::TaskNotFound("no queued task".into()))?;
        Ok(Value::String(id.to_string()))
    }
    fn task_pause<R: nexum_core::nexum_storage::TaskRepository>(core: &mut Core<R>, params: &Value) -> Result<Value, DispatchError> {
        let id = params.get("id").and_then(Value::as_str).ok_or_else(|| DispatchError::InvalidParams("missing id".into()))?;
        core.pause_task(&TaskId::from(id)).map_err(|e| DispatchError::Internal(format!("{e:?}")))?;
        Ok(Value::Bool(true))
    }
    fn task_resume<R: nexum_core::nexum_storage::TaskRepository>(core: &mut Core<R>, params: &Value) -> Result<Value, DispatchError> {
        let id = params.get("id").and_then(Value::as_str).ok_or_else(|| DispatchError::InvalidParams("missing id".into()))?;
        Ok(Value::Bool(core.resume_task(&TaskId::from(id)).map_err(|e| DispatchError::Internal(format!("{e:?}")))?))
    }
    fn task_remove<R: nexum_core::nexum_storage::TaskRepository>(core: &mut Core<R>, params: &Value) -> Result<Value, DispatchError> {
        let id = params.get("id").and_then(Value::as_str).ok_or_else(|| DispatchError::InvalidParams("missing id".into()))?;
        core.remove_task(&TaskId::from(id)).map_err(|e| DispatchError::Internal(format!("{e:?}")))?;
        Ok(Value::Bool(true))
    }
}

enum DispatchError { InvalidParams(String), TaskNotFound(String), Internal(String), MethodNotFound }

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EventEnvelope { pub event: String, pub data: Value }
impl EventEnvelope { pub fn new(event: impl Into<String>, data: Value) -> Self { Self { event: event.into(), data } } }

pub fn serialize_response(response: &RpcResponse) -> Result<String, RpcError> {
    serde_json::to_string(response).map_err(|error| RpcError::Parse(error.to_string()))
}

pub fn task_event_to_envelope(event: &nexum_task::TaskEvent) -> EventEnvelope {
    use nexum_task::TaskEvent;
    match event {
        TaskEvent::Created { task_id } => EventEnvelope::new("task.created", serde_json::json!({"task_id": task_id.to_string()})),
        TaskEvent::StateChanged { task_id, from, to } => EventEnvelope::new("task.state_changed", serde_json::json!({"task_id": task_id.to_string(), "from": format!("{from:?}"), "to": format!("{to:?}")})),
        TaskEvent::ProgressChanged { task_id, progress } => EventEnvelope::new("task.progress", serde_json::json!({"task_id": task_id.to_string(), "downloaded_bytes": progress.downloaded_bytes, "total_bytes": progress.total_bytes, "speed_bytes_per_second": progress.speed_bytes_per_second, "eta_seconds": progress.eta_seconds})),
        TaskEvent::Removed { task_id } => EventEnvelope::new("task.removed", serde_json::json!({"task_id": task_id.to_string()})),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexum_core::Core;
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
        let response = RpcResponse::success(Some(json!(1)), json!({"tasks":[]}));
        let encoded = serialize_response(&response).unwrap();
        assert!(encoded.contains(r#""jsonrpc":"2.0""#));
        assert!(encoded.contains(r#""result":{"tasks":[]}"#));
    }
    #[test]
    fn notifications_have_no_response() {
        let request = RpcRequest::notification("task.list", None);
        let mut core = Core::default();
        assert!(RpcDispatcher::dispatch(&mut core, &request).is_none());
    }
    #[test]
    fn unknown_method_is_reported() {
        let request = RpcRequest::new(1, "task.unknown", None);
        let mut core = Core::default();
        let response = RpcDispatcher::dispatch(&mut core, &request).unwrap();
        assert_eq!(response.error.unwrap().code, -32601);
    }
    #[test]
    fn task_event_maps_to_stable_envelope() {
        let event = nexum_task::TaskEvent::Created { task_id: TaskId::from("t1") };
        let envelope = task_event_to_envelope(&event);
        assert_eq!(envelope.event, "task.created");
        assert_eq!(envelope.data["task_id"], "t1");
    }
}
