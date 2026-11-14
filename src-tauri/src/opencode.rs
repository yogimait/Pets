use reqwest::Client;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const PORT_START: u16 = 4097;
const PORT_END: u16 = 4197;

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
    password: String,
    port: u16,
    child: Mutex<Option<Child>>,
}

impl OpenCodeState {
    pub fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or(0);
        let password = std::env::var("OPENCODE_SERVER_PASSWORD")
            .unwrap_or_else(|_| format!("agent-bots-{}-{stamp}", std::process::id()));
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            password,
            port: find_free_port(),
            child: Mutex::new(None),
        }
    }

    fn base(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    async fn get_json(&self, path: &str) -> Result<Value, String> {
        let response = self
            .client
            .get(format!("{}{}", self.base(), path))
            .basic_auth("opencode", Some(&self.password))
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

    async fn post_json(&self, path: &str, body: Value) -> Result<Value, String> {
        let response = self
            .client
            .post(format!("{}{}", self.base(), path))
            .basic_auth("opencode", Some(&self.password))
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

    async fn post_empty(&self, path: &str) -> Result<Value, String> {
        let response = self
            .client
            .post(format!("{}{}", self.base(), path))
            .basic_auth("opencode", Some(&self.password))
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
        let value = self.get_json("/api/info").await?;
        Ok(RuntimeInfo {
            version: value
                .get("version")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            pid: value.get("pid").and_then(Value::as_u64).unwrap_or(0) as u32,
            port: self.port,
            connected: true,
        })
    }

    async fn ensure(&self, project: &str) -> Result<RuntimeInfo, String> {
        debug_log(&format!("ensure start project={project}"));
        if !Path::new(project).is_dir() {
            return Err(format!("Project directory does not exist: {project}"));
        }

        if let Ok(info) = self.info().await {
            debug_log("ensure found running service");
            return Ok(info);
        }
        debug_log("ensure service not running, spawning");

        {
            let mut guard = self.child.lock().map_err(|_| "Runtime lock poisoned".to_string())?;
            let alive = match guard.as_mut() {
                Some(child) => child.try_wait().ok().flatten().is_none(),
                None => false,
            };
            if !alive {
                if let Some(mut old) = guard.take() {
                    let _ = old.kill();
                }
                *guard = Some(spawn_server(project, &self.password, self.port)?);
            }
        }

        for _ in 0..60 {
            std::thread::sleep(std::time::Duration::from_millis(250));
            if let Ok(info) = self.info().await {
                debug_log("ensure service ready");
                return Ok(info);
            }
        }

        debug_log("ensure service not ready after timeout");
        Err(format!(
            "OpenCode did not become ready on port {}. Check the service log.",
            self.port
        ))
    }
}

fn find_free_port() -> u16 {
    for port in PORT_START..=PORT_END {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    PORT_START
}

fn kill_tree(child: &mut Child) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

impl Drop for OpenCodeState {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(mut child) = guard.take() {
                kill_tree(&mut child);
            }
        }
    }
}

fn debug_log(message: &str) {
    use std::io::Write;
    let path = std::env::temp_dir().join("agent-bots-runtime.log");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{message}");
    }
}

fn spawn_server(project: &str, password: &str, port: u16) -> Result<Child, String> {
    debug_log(&format!("spawn_server cwd={project} port={port}"));
    #[cfg(windows)]
    let mut command = {
        let mut command = Command::new("cmd");
        command.args(["/C", "opencode", "serve", "--port", &port.to_string()]);
        command
    };

    #[cfg(not(windows))]
    let mut command = {
        let mut command = Command::new("opencode");
        command.args(["serve", "--port", &port.to_string()]);
        command
    };

    command
        .args(["--hostname", "127.0.0.1"])
        .current_dir(project)
        .env("OPENCODE_SERVER_PASSWORD", password)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| {
            debug_log(&format!("spawn failed: {error}"));
            format!("Could not start OpenCode: {error}")
        })
        .map(|child| {
            debug_log(&format!("spawned pid={}", child.id()));
            child
        })
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
    let mut guard = state.child.lock().map_err(|_| "Runtime lock poisoned".to_string())?;
    if let Some(mut child) = guard.take() {
        kill_tree(&mut child);
    }
    Ok(())
}
