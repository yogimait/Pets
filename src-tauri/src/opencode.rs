use futures_util::StreamExt;
use reqwest::Client;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Clone)]
struct SharedTarget {
    base: String,
    password: String,
    port: u16,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInfo {
    version: String,
    pid: u32,
    port: u16,
    connected: bool,
}

pub struct OpenCodeState {
    client: Client,
    target: Mutex<Option<SharedTarget>>,
}

impl OpenCodeState {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            target: Mutex::new(None),
        }
    }

    fn target(&self) -> Result<SharedTarget, String> {
        self.target
            .lock()
            .map_err(|_| "Runtime lock poisoned".to_string())?
            .clone()
            .ok_or_else(|| "OpenCode service is not connected.".to_string())
    }

    fn set_target(&self, target: Option<SharedTarget>) -> Result<(), String> {
        let mut guard = self.target.lock().map_err(|_| "Runtime lock poisoned".to_string())?;
        *guard = target;
        Ok(())
    }

    pub(crate) async fn get_json(&self, path: &str) -> Result<Value, String> {
        let target = self.target()?;
        let response = self
            .client
            .get(format!("{}{}", target.base, path))
            .basic_auth("opencode", Some(&target.password))
            .send()
            .await
            .map_err(|error| format!("OpenCode request failed: {error}"))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| format!("OpenCode response failed: {error}"))?;
        if !status.is_success() {
            return Err(format!("OpenCode {}: {}", status, text));
        }
        serde_json::from_str(&text).map_err(|error| format!("OpenCode JSON failed: {error}"))
    }

    pub(crate) async fn post_json(&self, path: &str, body: Value) -> Result<Value, String> {
        let target = self.target()?;
        let response = self
            .client
            .post(format!("{}{}", target.base, path))
            .basic_auth("opencode", Some(&target.password))
            .json(&body)
            .send()
            .await
            .map_err(|error| format!("OpenCode request failed: {error}"))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| format!("OpenCode response failed: {error}"))?;
        if !status.is_success() {
            return Err(format!("OpenCode {}: {}", status, text));
        }
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&text).map_err(|error| format!("OpenCode JSON failed: {error}"))
    }

    pub(crate) async fn post_empty(&self, path: &str) -> Result<Value, String> {
        let target = self.target()?;
        let response = self
            .client
            .post(format!("{}{}", target.base, path))
            .basic_auth("opencode", Some(&target.password))
            .send()
            .await
            .map_err(|error| format!("OpenCode request failed: {error}"))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| format!("OpenCode response failed: {error}"))?;
        if !status.is_success() {
            return Err(format!("OpenCode {}: {}", status, text));
        }
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&text).map_err(|error| format!("OpenCode JSON failed: {error}"))
    }

    async fn info(&self) -> Result<RuntimeInfo, String> {
        let target = self.target()?;
        let value = self.get_json("/api/info").await?;
        Ok(RuntimeInfo {
            version: value
                .get("version")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            pid: value.get("pid").and_then(Value::as_u64).unwrap_or(0) as u32,
            port: target.port,
            connected: true,
        })
    }

    pub(crate) async fn ensure(&self, project: &str) -> Result<RuntimeInfo, String> {
        debug_log(&format!("ensure start project={project}"));
        if !Path::new(project).is_dir() {
            return Err(format!("Project directory does not exist: {project}"));
        }

        if let Ok(info) = self.info().await {
            debug_log("ensure found connected service");
            return Ok(info);
        }

        self.set_target(None)?;
        let target = connect_shared_service()?;
        debug_log(&format!(
            "ensure connected shared service port={}",
            target.port
        ));
        self.set_target(Some(target))?;

        for _ in 0..60 {
            if let Ok(info) = self.info().await {
                debug_log("ensure service ready");
                return Ok(info);
            }
            std::thread::sleep(Duration::from_millis(250));
        }

        self.set_target(None)?;
        Err(
            "OpenCode background service did not become ready. Run `opencode` once or `opencode service start`, then retry."
                .to_string(),
        )
    }
}

fn debug_log(message: &str) {
    use std::io::Write;
    let path = std::env::temp_dir().join("agent-bots-runtime.log");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{message}");
    }
}

fn config_home() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".config")
        .join("opencode")
}

fn service_status_url() -> Option<String> {
    let output = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", "opencode service status"])
            .output()
            .ok()?
    } else {
        Command::new("opencode")
            .args(["service", "status"])
            .output()
            .ok()?
    };
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .map(str::trim)
        .find(|line| line.starts_with("http://"))
        .map(|line| line.trim_end_matches('/').to_string())
}

fn service_password() -> Option<String> {
    let text = std::fs::read_to_string(config_home().join("service.json")).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    value
        .get("password")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn port_from_base(base: &str) -> u16 {
    base.rsplit(':')
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0)
}

fn discover_shared_service() -> Option<SharedTarget> {
    let base = service_status_url()?;
    let password = service_password()?;
    Some(SharedTarget {
        port: port_from_base(&base),
        base,
        password,
    })
}

fn start_shared_service() {
    let mut command = if cfg!(windows) {
        let mut command = Command::new("cmd");
        command.args(["/C", "opencode service start"]);
        command
    } else {
        let mut command = Command::new("opencode");
        command.args(["service", "start"]);
        command
    };
    let _ = command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn connect_shared_service() -> Result<SharedTarget, String> {
    if let Some(target) = discover_shared_service() {
        return Ok(target);
    }
    debug_log("shared service not found; starting it");
    start_shared_service();
    for _ in 0..60 {
        std::thread::sleep(Duration::from_millis(250));
        if let Some(target) = discover_shared_service() {
            return Ok(target);
        }
    }
    Err(
        "OpenCode background service is not available. Run `opencode` once or `opencode service start`, then retry."
            .to_string(),
    )
}

const SYNC_EVENT_TYPES: &[&str] = &[
    "server.connected",
    "session.created",
    "session.updated",
    "session.deleted",
    "session.idle",
    "session.error",
    "session.status",
    "session.permissions",
    "permission.asked",
    "permission.updated",
    "message.updated",
];

fn is_sync_event(event_type: &str) -> bool {
    SYNC_EVENT_TYPES.contains(&event_type)
}

fn event_session_id(value: &Value) -> Option<String> {
    value
        .pointer("/data/sessionID")
        .and_then(Value::as_str)
        .or_else(|| value.pointer("/data/info/sessionID").and_then(Value::as_str))
        .or_else(|| value.pointer("/data/session/id").and_then(Value::as_str))
        .map(str::to_string)
}

fn drain_sse_frames(buffer: &mut String) -> Vec<String> {
    let mut frames = Vec::new();
    while let Some(end) = buffer.find("\n\n") {
        let frame = buffer[..end].to_string();
        buffer.drain(..end + 2);
        frames.push(frame);
    }
    frames
}

fn sse_data(frame: &str) -> Option<String> {
    let data = frame
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n");
    if data.is_empty() { None } else { Some(data) }
}

async fn event_stream_once(app: &AppHandle, target: &SharedTarget) -> Result<(), String> {
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|error| format!("SSE client failed: {error}"))?;
    let response = client
        .get(format!("{}/api/event", target.base))
        .basic_auth("opencode", Some(&target.password))
        .header(reqwest::header::ACCEPT, "text/event-stream")
        .send()
        .await
        .map_err(|error| format!("SSE request failed: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("SSE endpoint returned {status}"));
    }

    debug_log("SSE connected");
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|error| format!("SSE stream failed: {error}"))?;
        buffer.push_str(&String::from_utf8_lossy(&bytes).replace("\r\n", "\n").replace('\r', "\n"));
        for frame in drain_sse_frames(&mut buffer) {
            let Some(data) = sse_data(&frame) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<Value>(&data) else {
                continue;
            };
            let Some(event_type) = value.get("type").and_then(Value::as_str) else {
                continue;
            };
            if !is_sync_event(event_type) {
                continue;
            }
            let payload = json!({
                "type": event_type,
                "sessionId": event_session_id(&value),
            });
            let _ = app.emit("workforce-event", payload);
        }
    }
    Ok(())
}

pub(crate) async fn run_event_listener(app: AppHandle) {
    loop {
        let Some(target) = discover_shared_service() else {
            tokio::time::sleep(Duration::from_secs(2)).await;
            continue;
        };
        if let Err(error) = event_stream_once(&app, &target).await {
            debug_log(&format!("SSE disconnected: {error}"));
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn location_query(project: &str) -> String {
    format!("/api/agent?location%5Bdirectory%5D={}", url_encode(project))
}

fn url_encode(value: &str) -> String {
    let mut result = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char)
            }
            _ => result.push_str(&format!("%{byte:02X}")),
        }
    }
    result
}

#[tauri::command]
pub async fn runtime_info(
    state: tauri::State<'_, OpenCodeState>,
    project: String,
) -> Result<RuntimeInfo, String> {
    state.ensure(&project).await
}

#[tauri::command]
pub async fn runtime_agents(
    state: tauri::State<'_, OpenCodeState>,
    project: String,
) -> Result<Value, String> {
    state.ensure(&project).await?;
    let path = location_query(&project);
    state.get_json(&path).await
}

#[tauri::command]
pub async fn runtime_models(
    state: tauri::State<'_, OpenCodeState>,
    project: String,
) -> Result<Value, String> {
    state.ensure(&project).await?;
    state
        .get_json(&format!(
            "/api/model?location%5Bdirectory%5D={}",
            url_encode(&project)
        ))
        .await
}

#[tauri::command]
pub async fn runtime_mcp(
    state: tauri::State<'_, OpenCodeState>,
    project: String,
) -> Result<Value, String> {
    state.ensure(&project).await?;
    state
        .get_json(&format!(
            "/api/mcp?location%5Bdirectory%5D={}",
            url_encode(&project)
        ))
        .await
}

#[tauri::command]
pub async fn runtime_create_session(
    state: tauri::State<'_, OpenCodeState>,
    project: String,
    title: String,
    agent: String,
    provider_id: String,
    model_id: String,
    permissions: Option<Value>,
) -> Result<Value, String> {
    state.ensure(&project).await?;
    let mut body = json!({
        "title": title,
        "agent": agent,
        "model": { "id": model_id, "providerID": provider_id },
        "location": { "directory": project }
    });
    if let Some(permissions) = permissions {
        body["permissions"] = permissions;
    }
    state.post_json("/api/session", body).await
}

#[tauri::command]
pub async fn runtime_prompt(
    state: tauri::State<'_, OpenCodeState>,
    session_id: String,
    text: String,
) -> Result<Value, String> {
    state
        .post_json(
            &format!("/api/session/{session_id}/prompt"),
            json!({ "text": text }),
        )
        .await
}

#[tauri::command]
pub async fn runtime_abort(
    state: tauri::State<'_, OpenCodeState>,
    session_id: String,
) -> Result<Value, String> {
    state
        .post_empty(&format!("/api/session/{session_id}/interrupt"))
        .await
}

#[tauri::command]
pub async fn runtime_messages(
    state: tauri::State<'_, OpenCodeState>,
    session_id: String,
) -> Result<Value, String> {
    state
        .get_json(&format!("/api/session/{session_id}/message?limit=200&order=asc"))
        .await
}

#[tauri::command]
pub async fn runtime_permissions(
    state: tauri::State<'_, OpenCodeState>,
    session_id: String,
) -> Result<Value, String> {
    state
        .get_json(&format!("/api/session/{session_id}/permission"))
        .await
}

#[tauri::command]
pub async fn runtime_reply_permission(
    state: tauri::State<'_, OpenCodeState>,
    session_id: String,
    request_id: String,
    decision: String,
) -> Result<Value, String> {
    state
        .post_json(
            &format!("/api/session/{session_id}/permission/{request_id}/reply"),
            json!({ "decision": decision }),
        )
        .await
}

#[tauri::command]
pub fn runtime_stop(state: tauri::State<'_, OpenCodeState>) -> Result<(), String> {
    // The app uses the shared OpenCode background service. Disconnect the
    // adapter without stopping a service the user may still be using.
    state.set_target(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drains_complete_sse_frames_and_keeps_partial_data() {
        let mut buffer = String::from("data: {\"type\":\"session.idle\"}\n\ndata: {\"type\":\"session.updated\"}\n\npartial");
        let frames = drain_sse_frames(&mut buffer);
        assert_eq!(frames.len(), 2);
        assert_eq!(sse_data(&frames[0]).as_deref(), Some("{\"type\":\"session.idle\"}"));
        assert_eq!(buffer, "partial");
    }

    #[test]
    fn filters_only_sync_events_and_extracts_session_id() {
        assert!(is_sync_event("session.idle"));
        assert!(!is_sync_event("message.part.updated"));
        let value = json!({"type":"session.idle","data":{"sessionID":"ses_123"}});
        assert_eq!(event_session_id(&value).as_deref(), Some("ses_123"));
    }
}
