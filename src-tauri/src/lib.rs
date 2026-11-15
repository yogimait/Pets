mod opencode;
mod workforce;

use opencode::OpenCodeState;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PolicyFile {
    path: String,
    exists: bool,
    content: String,
    hash: String,
    bytes: usize,
    error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PolicyBundle {
    global: PolicyFile,
    project: Option<PolicyFile>,
    loaded_at: u64,
}

fn fnv1a(value: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

pub(crate) fn global_policy_path() -> PathBuf {
    if let Ok(path) = std::env::var("AGENT_BOTS_GLOBAL_POLICY") {
        return PathBuf::from(path);
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    PathBuf::from(home)
        .join(".config")
        .join("opencode")
        .join("AGENTS.md")
}

pub(crate) fn devvault_path() -> PathBuf {
    if let Ok(path) = std::env::var("DEVVAULT_PATH") {
        return PathBuf::from(path);
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    PathBuf::from(home).join("Obsidian").join("DevVault")
}

fn read_policy_file(path: &Path) -> PolicyFile {
    let display = path.display().to_string();
    match fs::read_to_string(path) {
        Ok(content) => PolicyFile {
            path: display,
            exists: true,
            hash: fnv1a(&content),
            bytes: content.len(),
            content,
            error: None,
        },
        Err(error) => PolicyFile {
            path: display,
            exists: false,
            content: String::new(),
            hash: String::new(),
            bytes: 0,
            error: Some(error.to_string()),
        },
    }
}

#[tauri::command]
fn read_policy(project_root: Option<String>) -> PolicyBundle {
    let global = read_policy_file(&global_policy_path());
    let project = project_root.map(|root| read_policy_file(&PathBuf::from(root).join("AGENTS.md")));
    PolicyBundle {
        global,
        project,
        loaded_at: now_ms(),
    }
}

fn show_orchestrator(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("orchestrator") {
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn open_orchestrator(app: tauri::AppHandle) {
    show_orchestrator(&app);
}

#[tauri::command]
fn hide_orchestrator(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("orchestrator") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn open_record(app: tauri::AppHandle, kind: String, id: String) {
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.unminimize();
        let _ = main.show();
        let _ = main.set_focus();
        let _ = app.emit_to("main", "navigate", format!("{kind}:{id}"));
    }
    if let Some(overlay) = app.get_webview_window("orchestrator") {
        let _ = overlay.hide();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(OpenCodeState::new())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
            let store = workforce::WorkforceStore::new(data_dir.join("workforce.db"))
                .map_err(|error| -> Box<dyn std::error::Error> {
                    Box::new(std::io::Error::other(error))
                })?;
            app.manage(store);

            let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Backquote);
            if let Err(error) = app.global_shortcut().on_shortcut(shortcut, |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    show_orchestrator(app);
                }
            }) {
                eprintln!("global shortcut registration failed: {error}");
            }

            let event_app = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                opencode::run_event_listener(event_app).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_orchestrator,
            hide_orchestrator,
            open_record,
            read_policy,
            opencode::runtime_info,
            opencode::runtime_agents,
            opencode::runtime_models,
            opencode::runtime_mcp,
            opencode::runtime_create_session,
            opencode::runtime_prompt,
            opencode::runtime_abort,
            opencode::runtime_messages,
            opencode::runtime_permissions,
            opencode::runtime_reply_permission,
            opencode::runtime_stop,
            workforce::workforce_snapshot,
            workforce::workforce_seed,
            workforce::workforce_inspect_project,
            workforce::workforce_create_project,
            workforce::workforce_plan,
            workforce::workforce_send,
            workforce::workforce_tick,
            workforce::workforce_poll,
            workforce::workforce_reply_task_permission,
            workforce::workforce_run_task,
            workforce::workforce_accept_task,
            workforce::workforce_abort_task,
            workforce::workforce_dreamer,
            workforce::workforce_decide_proposal
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
