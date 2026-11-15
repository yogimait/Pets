use crate::opencode::OpenCodeState;
use crate::{devvault_path, global_policy_path, now_ms};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::State;
use tokio::sync::Mutex as AsyncMutex;

const CURRENT_SCHEMA: u64 = 3;
const DEFAULT_MODEL: &str = "opencode-go/deepseek-v4.1-flash";
const ORCHESTRATOR_AGENT: &str = "orchestrator-workforce";
const IDLE_TIMEOUT_SECS: u64 = 180;
const RUNNING_TIMEOUT_MS: u64 = 30 * 60 * 1000;
const PROMPT_BUDGET: usize = 20_000;
const MAX_AUTO_PERMISSIONS: usize = 16;

pub struct WorkforceStore {
    conn: Mutex<Connection>,
    sessions: Mutex<HashMap<String, String>>,
    run_lock: AsyncMutex<()>,
    session_lock: AsyncMutex<()>,
    worktrees_root: PathBuf,
}

impl WorkforceStore {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create app data directory: {error}"))?;
        }
        let worktrees_root = path
            .parent()
            .map(|parent| parent.join("worktrees"))
            .unwrap_or_else(|| PathBuf::from("worktrees"));
        fs::create_dir_all(&worktrees_root)
            .map_err(|error| format!("Could not create worktree directory: {error}"))?;
        let conn = Connection::open(&path)
            .map_err(|error| format!("Could not open workforce database: {error}"))?;
        conn.busy_timeout(Duration::from_secs(10))
            .map_err(|error| format!("Could not configure workforce database: {error}"))?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS state (
                 id INTEGER PRIMARY KEY CHECK (id = 1),
                 json TEXT NOT NULL,
                 updated_at INTEGER NOT NULL
             );",
        )
        .map_err(|error| format!("Could not prepare workforce database: {error}"))?;
        Ok(Self {
            conn: Mutex::new(conn),
            sessions: Mutex::new(HashMap::new()),
            run_lock: AsyncMutex::new(()),
            session_lock: AsyncMutex::new(()),
            worktrees_root,
        })
    }

    fn read_locked(conn: &Connection) -> Result<Option<Value>, String> {
        let raw: Option<String> = conn
            .query_row("SELECT json FROM state WHERE id = 1", [], |row| row.get(0))
            .optional()
            .map_err(|error| format!("Could not read workforce snapshot: {error}"))?;
        raw.map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(|error| format!("Stored workforce snapshot is invalid JSON: {error}"))
    }

    fn write_locked(conn: &Connection, value: &Value) -> Result<(), String> {
        let json = serde_json::to_string(value)
            .map_err(|error| format!("Could not serialize workforce snapshot: {error}"))?;
        conn.execute(
            "INSERT INTO state (id, json, updated_at) VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET json = excluded.json, updated_at = excluded.updated_at",
            params![json, now_ms() as i64],
        )
        .map_err(|error| format!("Could not save workforce snapshot: {error}"))?;
        Ok(())
    }

    fn read_current_locked(conn: &Connection) -> Result<Option<Value>, String> {
        let Some(mut value) = Self::read_locked(conn)? else {
            return Ok(None);
        };
        if migrate_snapshot(&mut value) {
            Self::write_locked(conn, &value)?;
        }
        Ok(Some(value))
    }

    pub fn snapshot(&self) -> Result<Option<Value>, String> {
        let guard = self.conn.lock().map_err(|_| "Workforce store lock poisoned")?;
        Self::read_current_locked(&guard)
    }

    pub fn seed(&self, mut snapshot: Value) -> Result<Value, String> {
        if !snapshot.is_object() {
            return Err("Workforce snapshot must be a JSON object".to_string());
        }
        let guard = self.conn.lock().map_err(|_| "Workforce store lock poisoned")?;
        if let Some(existing) = Self::read_current_locked(&guard)? {
            return Ok(existing);
        }
        normalize_seed(&mut snapshot);
        Self::write_locked(&guard, &snapshot)?;
        Ok(snapshot)
    }

    pub fn update<F>(&self, mutate: F) -> Result<Value, String>
    where
        F: FnOnce(&mut Value) -> Result<(), String>,
    {
        let guard = self.conn.lock().map_err(|_| "Workforce store lock poisoned")?;
        let mut value = Self::read_current_locked(&guard)?
            .ok_or_else(|| "Workforce snapshot is not seeded".to_string())?;
        mutate(&mut value)?;
        Self::write_locked(&guard, &value)?;
        Ok(value)
    }

    fn session_for(&self, project_id: &str) -> Option<String> {
        self.sessions
            .lock()
            .ok()
            .and_then(|map| map.get(project_id).cloned())
    }

    fn remember_session(&self, project_id: &str, session_id: &str) {
        if let Ok(mut map) = self.sessions.lock() {
            map.insert(project_id.to_string(), session_id.to_string());
        }
    }

    fn forget_session(&self, project_id: &str) {
        if let Ok(mut map) = self.sessions.lock() {
            map.remove(project_id);
        }
    }
}

#[tauri::command]
pub async fn workforce_snapshot(store: State<'_, WorkforceStore>) -> Result<Option<Value>, String> {
    store.inner().snapshot()
}

#[tauri::command]
pub async fn workforce_seed(
    store: State<'_, WorkforceStore>,
    snapshot: Value,
) -> Result<Value, String> {
    store.inner().seed(snapshot)
}

#[tauri::command]
pub fn workforce_inspect_project(path: String) -> Result<Value, String> {
    let path = absolute_project_path(&path)?;
    let exists = path.exists();
    let is_directory = path.is_dir();
    let has_agents = path.join("AGENTS.md").is_file();
    let has_readme = path.join("README.md").is_file();
    let is_git = path.join(".git").exists();
    let entry_count = if is_directory {
        fs::read_dir(&path)
            .map(|entries| entries.filter_map(Result::ok).count())
            .unwrap_or(0)
    } else {
        0
    };
    Ok(json!({
        "path": path.display().to_string(),
        "exists": exists,
        "isDirectory": is_directory,
        "hasAgents": has_agents,
        "hasReadme": has_readme,
        "isGit": is_git,
        "entryCount": entry_count
    }))
}

#[tauri::command]
pub fn workforce_create_project(
    store: State<'_, WorkforceStore>,
    name: String,
    key: String,
    repo_path: String,
    description: Option<String>,
    context: Option<String>,
    instructions: Option<String>,
    create_folder: Option<bool>,
    create_policy: Option<bool>,
    create_readme: Option<bool>,
    init_git: Option<bool>,
) -> Result<Value, String> {
    let store = store.inner();
    let name = name.trim();
    if name.is_empty() {
        return Err("Project name is required.".to_string());
    }
    if name.chars().count() > 80 {
        return Err("Project name must be 80 characters or fewer.".to_string());
    }
    let key = validate_project_key(&key)?;
    let path = absolute_project_path(&repo_path)?;
    let snapshot = snapshot_value(store)?;

    if snapshot
        .get("projects")
        .and_then(Value::as_array)
        .map(|projects| {
            projects.iter().any(|project| {
                project
                    .get("key")
                    .and_then(Value::as_str)
                    .map(|existing| existing.eq_ignore_ascii_case(&key))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
    {
        return Err(format!("Project key {key} is already in use."));
    }

    let path_key = path_identity(&path);
    if snapshot
        .get("projects")
        .and_then(Value::as_array)
        .map(|projects| {
            projects.iter().any(|project| {
                project
                    .get("repoPath")
                    .and_then(Value::as_str)
                    .map(|existing| path_identity(Path::new(existing)) == path_key)
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
    {
        return Err(format!(
            "A project already exists for this folder: {}",
            path.display()
        ));
    }

    let exists = path.exists();
    if exists && !path.is_dir() {
        return Err(format!(
            "The path exists but is not a folder: {}",
            path.display()
        ));
    }
    if !exists {
        if !create_folder.unwrap_or(false) {
            return Err(format!(
                "Folder does not exist: {}. Enable folder creation or choose an existing folder.",
                path.display()
            ));
        }
        fs::create_dir_all(&path)
            .map_err(|error| format!("Could not create project folder: {error}"))?;
    }

    let description = description.unwrap_or_default();
    let created_policy = if create_policy.unwrap_or(true) {
        ensure_starter_file(&path.join("AGENTS.md"), &starter_project_policy(name))?
    } else {
        false
    };
    let created_readme = if create_readme.unwrap_or(true) {
        ensure_starter_file(&path.join("README.md"), &starter_project_readme(name, &description))?
    } else {
        false
    };
    let created_git = if init_git.unwrap_or(false) && !path.join(".git").exists() {
        let status = Command::new("git")
            .args(["init"])
            .current_dir(&path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|error| format!("Could not run git init: {error}"))?;
        if !status.success() {
            return Err("git init failed. Create the project without Git, then initialize it manually.".to_string());
        }
        true
    } else {
        false
    };
    let policy_state = if created_policy {
        "created"
    } else if path.join("AGENTS.md").is_file() {
        "present"
    } else {
        "missing"
    };
    let readme_state = if created_readme {
        "created"
    } else if path.join("README.md").is_file() {
        "present"
    } else {
        "missing"
    };
    let git_state = if created_git {
        "initialized"
    } else if path.join(".git").exists() {
        "present"
    } else {
        "not requested"
    };

    let project_id = project_id_for_key(&key);
    if snapshot
        .get("projects")
        .and_then(Value::as_array)
        .map(|projects| {
            projects
                .iter()
                .any(|project| project.get("id").and_then(Value::as_str) == Some(&project_id))
        })
        .unwrap_or(false)
    {
        return Err(format!("Project id {project_id} is already in use."));
    }

    let project = json!({
        "id": project_id.clone(),
        "key": key,
        "name": name,
        "description": description,
        "status": "active",
        "context": context.unwrap_or_default(),
        "instructions": instructions.unwrap_or_default(),
        "repoPath": path.display().to_string(),
        "createdAt": now_ms()
    });
    let created_path = path.display().to_string();
    let summary = format!(
        "created project {name} ({project_id}) at {created_path}; policy {policy_state} readme {readme_state} git {git_state}"
    );
    let project_for_write = project.clone();
    let project_id_for_activity = project_id.clone();
    let updated = store.update(|value| {
        if let Some(projects) = value.get_mut("projects").and_then(Value::as_array_mut) {
            projects.push(project_for_write.clone());
        } else {
            value["projects"] = json!([project_for_write.clone()]);
        }
        if let Some(agents) = value.get_mut("agents").and_then(Value::as_array_mut) {
            for agent in agents {
                if let Some(project_ids) = agent.get_mut("projectIds").and_then(Value::as_array_mut) {
                    if !project_ids
                        .iter()
                        .any(|id| id.as_str() == Some(&project_id_for_activity))
                    {
                        project_ids.push(json!(project_id_for_activity.clone()));
                    }
                }
            }
        }
        push_activity(
            value,
            "system",
            "orchestrator",
            Some(&project_id_for_activity),
            None,
            &summary,
            None,
        );
        Ok(())
    })?;
    Ok(updated)
}

#[tauri::command]
pub async fn workforce_plan(
    store: State<'_, WorkforceStore>,
    runtime: State<'_, OpenCodeState>,
    project_id: String,
    goal: String,
) -> Result<Value, String> {
    let store = store.inner();
    let runtime = runtime.inner();
    let snapshot = snapshot_value(store)?;
    let project = get_project(&snapshot, &project_id)?.clone();
    let repo = repo_path(&project)?;
    let prompt = plan_prompt(&snapshot, &project, &goal);
    let title = format!("{} · planning", project_label(&project));
    let (_session_id, text) = run_once(
        runtime,
        &repo,
        &title,
        ORCHESTRATOR_AGENT,
        &prompt,
        None,
    )
    .await?;

    let parsed = extract_json_object(&text)
        .ok_or_else(|| "Planner did not return a JSON object.".to_string())?;
    let summary = parsed
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let raw_tasks: Vec<Value> = parsed
        .get("tasks")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let owners = available_owners(&snapshot);
    let at = now_ms();
    let ids: Vec<String> = (0..raw_tasks.len())
        .map(|index| format!("t-{at}-{index}"))
        .collect();

    let mut planned: Vec<Value> = Vec::new();
    for (index, item) in raw_tasks.iter().enumerate().take(50) {
        let title = item
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let title = if title.is_empty() {
            format!("Task {}", index + 1)
        } else {
            title
        };
        let owner = item
            .get("ownerId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|owner| !owner.is_empty() && owners.iter().any(|known| known == owner))
            .map(str::to_string)
            .or_else(|| owners.first().cloned())
            .unwrap_or_else(|| "orchestrator".to_string());
        let priority = match item.get("priority").and_then(Value::as_str) {
            Some("low") => "low",
            Some("high") => "high",
            Some("critical") => "critical",
            _ => "normal",
        };
        let mut depends_on: Vec<String> = Vec::new();
        if let Some(list) = item.get("dependsOn").and_then(Value::as_array) {
            for dependency in list {
                if let Some(dep_index) = dependency.as_u64().map(|value| value as usize) {
                    if dep_index < index {
                        if let Some(id) = ids.get(dep_index) {
                            if !depends_on.contains(id) {
                                depends_on.push(id.clone());
                            }
                        }
                    }
                }
            }
        }
        planned.push(json!({
            "id": ids[index],
            "title": title,
            "description": item.get("description").and_then(Value::as_str).unwrap_or(""),
            "status": "queued",
            "priority": priority,
            "projectId": project_id,
            "ownerId": owner,
            "notes": "",
            "tags": ["planned"],
            "createdAt": at,
            "updatedAt": at,
            "steps": [],
            "messages": [],
            "acceptance": item.get("acceptance").and_then(Value::as_str).unwrap_or(""),
            "dependsOn": depends_on,
            "autoStart": true,
            "sessionId": null,
            "result": null
        }));
    }

    let summary = if summary.is_empty() {
        format!("Planned {} tasks.", planned.len())
    } else {
        summary
    };
    let project_key = project
        .get("key")
        .and_then(Value::as_str)
        .unwrap_or(&project_id)
        .to_string();
    let chat_text = if planned.is_empty() {
        summary.clone()
    } else {
        let count = planned.len();
        format!(
            "{summary}\n\nCreated {count} {}.",
            if count == 1 { "task" } else { "tasks" }
        )
    };
    let planned_for_write = planned.clone();
    let summary_for_write = summary.clone();
    store.update(|value| {
        if let Some(tasks) = value.get_mut("tasks").and_then(Value::as_array_mut) {
            for task in &planned_for_write {
                tasks.push(task.clone());
            }
        }
        for task in &planned_for_write {
            let task_id = task.get("id").and_then(Value::as_str).unwrap_or("");
            let title = task.get("title").and_then(Value::as_str).unwrap_or("");
            let owner = task.get("ownerId").and_then(Value::as_str).unwrap_or("orchestrator");
            push_activity(
                value,
                "delegation",
                "orchestrator",
                Some(&project_id),
                Some(task_id),
                &format!("delegated \"{title}\" to {owner}"),
                task.get("description").and_then(Value::as_str),
            );
        }
        let refs: Vec<Value> = planned_for_write
            .iter()
            .map(|task| {
                let id = task.get("id").and_then(Value::as_str).unwrap_or("");
                json!({
                    "kind": "task",
                    "id": id,
                    "label": format!("{project_key} {id}")
                })
            })
            .collect();
        if !goal.trim().is_empty() {
            let goal_ref = vec![json!({
                "kind": "project",
                "id": project_id,
                "label": project_key
            })];
            push_chat(value, "user", goal.trim(), goal_ref);
        }
        push_chat(value, "orchestrator", &chat_text, refs);
        Ok(())
    })?;

    Ok(json!({ "summary": summary_for_write, "tasks": planned }))
}

#[tauri::command]
pub async fn workforce_send(
    store: State<'_, WorkforceStore>,
    runtime: State<'_, OpenCodeState>,
    project_id: String,
    text: String,
) -> Result<Value, String> {
    let store = store.inner();
    let runtime = runtime.inner();
    let _session_guard = store.session_lock.lock().await;
    let snapshot = snapshot_value(store)?;
    let project = get_project(&snapshot, &project_id)?.clone();
    let repo = repo_path(&project)?;
    require_policy(&repo)?;
    runtime.ensure(&repo).await?;
    let (provider_id, model_id) = model_config()?;
    let prompt = send_prompt(&snapshot, &project, &text);

    let session_id = match store.session_for(&project_id) {
        Some(session_id) => session_id,
        None => {
            let title = format!("{} · Orchestrator", project_label(&project));
            let session = create_session(
                runtime,
                &repo,
                &title,
                ORCHESTRATOR_AGENT,
                &provider_id,
                &model_id,
                None,
            )
            .await
            .map_err(|error| {
                format!("Could not start model {provider_id}/{model_id}: {error}")
            })?;
            let session_id = session_id_of(&session)
                .ok_or_else(|| "OpenCode did not return a session id.".to_string())?;
            store.remember_session(&project_id, &session_id);
            session_id
        }
    };

    if let Err(error) = prompt_and_wait(runtime, &session_id, &prompt).await {
        store.forget_session(&project_id);
        return Err(error);
    }
    let messages = runtime
        .get_json(&format!(
            "/api/session/{session_id}/message?limit=200&order=asc"
        ))
        .await?;
    let reply = last_assistant_text(&messages)
        .unwrap_or_else(|| "No assistant text returned.".to_string());

    store.update(|value| {
        let refs = vec![json!({
            "kind": "project",
            "id": project_id,
            "label": project_label(&project)
        })];
        push_chat(value, "user", &text, Vec::new());
        push_chat(value, "orchestrator", &reply, refs);
        Ok(())
    })?;

    Ok(json!({ "text": reply }))
}

#[tauri::command]
pub async fn workforce_tick(
    store: State<'_, WorkforceStore>,
    runtime: State<'_, OpenCodeState>,
) -> Result<Value, String> {
    let store = store.inner();
    let runtime = runtime.inner();
    let mut started = 0usize;
    for task_id in {
        let snapshot = snapshot_value(store)?;
        ready_tasks(&snapshot)
    } {
        if started >= 2 {
            break;
        }
        match start_task(store, runtime, &task_id).await {
            Ok(_) => started += 1,
            Err(error)
                if !error.contains("already has a running task")
                    && !error.contains("Concurrency limit")
                    && !error.contains("is not queued or retryable")
                    && !error.contains("was claimed by another run")
                    && !error.contains("has unfinished dependencies") =>
            {
                store.update(|value| {
                    mark_blocked(
                        value,
                        &task_id,
                        &format!("Could not start the task: {error}"),
                    );
                    Ok(())
                })?;
            }
            Err(_) => {}
        }
    }
    snapshot_value(store)
}

#[tauri::command]
pub async fn workforce_poll(
    store: State<'_, WorkforceStore>,
    runtime: State<'_, OpenCodeState>,
) -> Result<Value, String> {
    let store = store.inner();
    let runtime = runtime.inner();
    let snapshot = snapshot_value(store)?;
    let running: Vec<(String, Option<String>, u64, String)> = snapshot
        .get("tasks")
        .and_then(Value::as_array)
        .map(|tasks| {
            tasks
                .iter()
                .filter_map(|task| {
                    if task.get("status").and_then(Value::as_str) != Some("running") {
                        return None;
                    }
                    let id = task.get("id").and_then(Value::as_str)?.to_string();
                    let project_id = task.get("projectId").and_then(Value::as_str)?.to_string();
                    let session_id = task
                        .get("sessionId")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    let started = task
                        .get("startedAt")
                        .and_then(Value::as_u64)
                        .or_else(|| task.get("updatedAt").and_then(Value::as_u64))
                        .unwrap_or_else(now_ms);
                    Some((id, session_id, started, project_id))
                })
                .collect()
        })
        .unwrap_or_default();

    for (task_id, session_id, started, project_id) in running {
        let session_id = match session_id {
            Some(session_id) => session_id,
            None => {
                if now_ms().saturating_sub(started) > 60_000 {
                    store.update(|value| {
                        mark_blocked(value, &task_id, "Task session was not created.");
                        Ok(())
                    })?;
                }
                continue;
            }
        };

        let repo = get_project(&snapshot, &project_id)
            .and_then(repo_path)
            .map_err(|error| format!("Could not resolve the task project: {error}"));
        let repo = match repo {
            Ok(repo) => repo,
            Err(error) => {
                store.update(|value| {
                    mark_blocked(value, &task_id, &error);
                    Ok(())
                })?;
                continue;
            }
        };
        if let Err(error) = runtime.ensure(&repo).await {
            store.update(|value| {
                mark_blocked(
                    value,
                    &task_id,
                    &format!("OpenCode unavailable while checking the session: {error}"),
                );
                Ok(())
            })?;
            continue;
        }

        let session = match runtime
            .get_json(&format!("/api/session/{session_id}"))
            .await
        {
            Ok(value) => unwrap_data(value),
            Err(error) => {
                store.update(|value| {
                    mark_blocked(value, &task_id, &format!("Session check failed: {error}"));
                    Ok(())
                })?;
                continue;
            }
        };

        let permissions = match runtime
            .get_json(&format!("/api/session/{session_id}/permission"))
            .await
        {
            Ok(value) => value,
            Err(error) => {
                store.update(|value| {
                    mark_blocked(value, &task_id, &format!("Permission check failed: {error}"));
                    Ok(())
                })?;
                continue;
            }
        };
        if let Some(permission) = first_permission(&permissions) {
            store.update(|value| {
                let (_owner, project_id, title) = task_meta(value, &task_id);
                let permission_id = permission
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if find_task(value, &task_id)
                    .and_then(|task| task.pointer("/permission/id"))
                    .and_then(Value::as_str)
                    != Some(&permission_id)
                {
                    if let Some(task) = find_task_mut(value, &task_id) {
                        task["permission"] = permission.clone();
                        task["updatedAt"] = json!(now_ms());
                    }
                    let action = permission
                        .get("action")
                        .and_then(Value::as_str)
                        .unwrap_or("tool");
                    let resources = permission
                        .get("resources")
                        .and_then(Value::as_array)
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(Value::as_str)
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    push_activity(
                        value,
                        "status",
                        "orchestrator",
                        Some(&project_id),
                        Some(&task_id),
                        &format!("permission requested for {title}: {action}"),
                        Some(&resources),
                    );
                }
                Ok(())
            })?;
            continue;
        }
        store.update(|value| {
            if let Some(task) = find_task_mut(value, &task_id) {
                if task
                    .get("permission")
                    .map(|permission| !permission.is_null())
                    .unwrap_or(false)
                {
                    task["permission"] = Value::Null;
                }
            }
            Ok(())
        })?;

        let idle = session_idle(&session)
            .map(|idle_at| idle_at >= started)
            .unwrap_or(false);
        let has_outcome = session_outcome(&session).is_some();
        if !idle && !has_outcome {
            if now_ms().saturating_sub(started) > RUNNING_TIMEOUT_MS {
                store.update(|value| {
                    mark_blocked(value, &task_id, "Task exceeded the 30 minute run limit.");
                    Ok(())
                })?;
            }
            continue;
        }

        if let Some(error) = session_error(&session) {
            store.update(|value| {
                mark_blocked(value, &task_id, &format!("Session ended with an error: {error}"));
                Ok(())
            })?;
            continue;
        }

        let messages = match runtime
            .get_json(&format!(
                "/api/session/{session_id}/message?limit=200&order=asc"
            ))
            .await
        {
            Ok(value) => value,
            Err(error) => {
                store.update(|value| {
                    mark_blocked(value, &task_id, &format!("Could not read session messages: {error}"));
                    Ok(())
                })?;
                continue;
            }
        };
        let text = last_assistant_text(&messages)
            .unwrap_or_else(|| "No assistant text returned.".to_string());
        store.update(|value| {
            mark_review(value, &task_id, &text);
            Ok(())
        })?;
    }

    snapshot_value(store)
}

#[tauri::command]
pub async fn workforce_reply_task_permission(
    store: State<'_, WorkforceStore>,
    runtime: State<'_, OpenCodeState>,
    task_id: String,
    request_id: String,
    decision: String,
) -> Result<Value, String> {
    let store = store.inner();
    let runtime = runtime.inner();
    let decision = match decision.trim().to_ascii_lowercase().as_str() {
        "once" | "always" | "reject" => decision.trim().to_ascii_lowercase(),
        other => {
            return Err(format!(
                "Unknown permission decision {other:?}. Expected once, always, or reject."
            ))
        }
    };
    let snapshot = snapshot_value(store)?;
    let task = find_task(&snapshot, &task_id)
        .ok_or_else(|| format!("Unknown task: {task_id}"))?;
    let session_id = task
        .get("sessionId")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Task {task_id} has no OpenCode session."))?
        .to_string();
    runtime
        .post_json(
            &format!("/api/session/{session_id}/permission/{request_id}/reply"),
            json!({ "decision": decision.clone() }),
        )
        .await?;

    if decision == "reject" {
        let _ = runtime
            .post_empty(&format!("/api/session/{session_id}/interrupt"))
            .await;
    }

    store.update(|value| {
        let (owner, project_id, title) = task_meta(value, &task_id);
        if decision == "reject" {
            if let Some(task) = find_task_mut(value, &task_id) {
                task["status"] = json!("blocked");
                task["permission"] = Value::Null;
                task["updatedAt"] = json!(now_ms());
                append_note(task, "Permission denied by operator.");
            }
            clear_agent_current_task(value, &owner, &task_id);
        } else if let Some(task) = find_task_mut(value, &task_id) {
            task["permission"] = Value::Null;
            task["updatedAt"] = json!(now_ms());
        }
        push_activity(
            value,
            "status",
            "orchestrator",
            Some(&project_id),
            Some(&task_id),
            &format!("permission {decision} for {title}"),
            None,
        );
        Ok(())
    })
}

#[tauri::command]
pub async fn workforce_run_task(
    store: State<'_, WorkforceStore>,
    runtime: State<'_, OpenCodeState>,
    task_id: String,
) -> Result<Value, String> {
    let store = store.inner();
    start_task(store, runtime.inner(), &task_id).await
}

#[tauri::command]
pub async fn workforce_accept_task(
    store: State<'_, WorkforceStore>,
    task_id: String,
) -> Result<Value, String> {
    let store = store.inner();
    let snapshot = snapshot_value(store)?;
    let current = find_task(&snapshot, &task_id)
        .ok_or_else(|| format!("Unknown task: {task_id}"))?;
    let status = current.get("status").and_then(Value::as_str);
    if status == Some("done") {
        return Ok(snapshot);
    }
    if status != Some("review") {
        return Err(format!("Task {task_id} is not in review."));
    }
    if current
        .get("sessionId")
        .and_then(Value::as_str)
        .is_none()
    {
        return Err(format!(
            "Task {task_id} has no linked OpenCode session; run it again before accepting."
        ));
    }
    let project_id = current
        .get("projectId")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Task {task_id} has no projectId."))?;
    let base_repo = repo_path(get_project(&snapshot, project_id)?)?;
    let merged_branch = merge_isolated_worktree(&base_repo, current)?;
    store.update(|value| {
        let current_status = find_task(value, &task_id)
            .and_then(|task| task.get("status").and_then(Value::as_str));
        if current_status != Some("review") {
            return Err(format!("Task {task_id} is no longer in review."));
        }
        let (owner, project_id, title) = task_meta(value, &task_id);
        if let Some(task) = find_task_mut(value, &task_id) {
            task["status"] = json!("done");
            task["updatedAt"] = json!(now_ms());
            task["finishedAt"] = json!(now_ms());
            task["worktreePath"] = Value::Null;
            task["worktreeBranch"] = Value::Null;
            if let Some(branch) = &merged_branch {
                append_note(task, &format!("Merged isolated worktree branch {branch}."));
            }
        }
        clear_agent_current_task(value, &owner, &task_id);
        push_activity(
            value,
            "result",
            &owner,
            Some(&project_id),
            Some(&task_id),
            &format!("accepted {task_id}: {title}"),
            None,
        );
        Ok(())
    })
}

#[tauri::command]
pub async fn workforce_abort_task(
    store: State<'_, WorkforceStore>,
    runtime: State<'_, OpenCodeState>,
    task_id: String,
) -> Result<Value, String> {
    let store = store.inner();
    let snapshot = snapshot_value(store)?;
    let task = find_task(&snapshot, &task_id)
        .ok_or_else(|| format!("Unknown task: {task_id}"))?;
    if task.get("status").and_then(Value::as_str) == Some("done") {
        return Ok(snapshot);
    }
    let session_id = task
        .get("sessionId")
        .and_then(Value::as_str)
        .map(str::to_string);
    let mut interrupt_error = None;
    if let Some(session_id) = session_id {
        if let Err(error) = runtime
            .post_empty(&format!("/api/session/{session_id}/interrupt"))
            .await
        {
            interrupt_error = Some(error);
        }
    }
    store.update(|value| {
        let (owner, project_id, title) = task_meta(value, &task_id);
        if let Some(task) = find_task_mut(value, &task_id) {
            task["status"] = json!("blocked");
            task["updatedAt"] = json!(now_ms());
            append_note(task, "Aborted by operator.");
        }
        clear_agent_current_task(value, &owner, &task_id);
        let detail = interrupt_error
            .as_deref()
            .map(|error| format!("Interrupt request failed: {error}"));
        push_activity(
            value,
            "status",
            "orchestrator",
            Some(&project_id),
            Some(&task_id),
            &format!("aborted {task_id}: {title}"),
            detail.as_deref(),
        );
        Ok(())
    })
}

#[tauri::command]
pub async fn workforce_dreamer(
    store: State<'_, WorkforceStore>,
    runtime: State<'_, OpenCodeState>,
    project_id: String,
) -> Result<Value, String> {
    let store = store.inner();
    let runtime = runtime.inner();
    let snapshot = snapshot_value(store)?;
    let project = get_project(&snapshot, &project_id)?.clone();
    let repo = repo_path(&project)?;
    let prompt = dreamer_prompt(&snapshot, &project);
    let title = format!("{} · Dreamer", project_label(&project));
    let (_session_id, text) = run_once(runtime, &repo, &title, "dreamer", &prompt, None).await?;
    let parsed = extract_json_object(&text)
        .ok_or_else(|| "Dreamer did not return a JSON object.".to_string())?;

    let at = now_ms();
    let findings: Vec<String> = parsed
        .get("findings")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| item.to_string())
                })
                .collect()
        })
        .unwrap_or_default();
    let raw_proposals = parsed
        .get("proposals")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let proposals: Vec<Value> = raw_proposals
        .iter()
        .take(20)
        .enumerate()
        .map(|(index, proposal)| normalize_proposal(proposal, &project_id, at, index))
        .collect();
    let proposal_ids: Vec<String> = proposals
        .iter()
        .filter_map(|proposal| proposal.get("id").and_then(Value::as_str))
        .map(str::to_string)
        .collect();
    let run_id = unique_id(
        snapshot.get("dreamerRuns").and_then(Value::as_array),
        "run",
        at,
    );
    let run = json!({
        "id": run_id,
        "trigger": "manual",
        "scope": "project",
        "projectId": project_id,
        "startedAt": at,
        "finishedAt": at,
        "sourcesRead": ["snapshot", "project", "tasks", "memory"],
        "sourcesSkipped": [],
        "findings": findings,
        "proposalIds": proposal_ids,
        "status": "done",
        "parentUpdate": "none"
    });

    store.update(|value| {
        if let Some(memory) = value.get_mut("proposals").and_then(Value::as_array_mut) {
            for proposal in &proposals {
                memory.push(proposal.clone());
            }
        }
        if let Some(runs) = value.get_mut("dreamerRuns").and_then(Value::as_array_mut) {
            runs.push(run.clone());
        }
        push_activity(
            value,
            "memory",
            "dreamer",
            Some(&project_id),
            None,
            &format!(
                "proposed {} memory {}",
                proposals.len(),
                if proposals.len() == 1 { "entry" } else { "entries" }
            ),
            findings.first().map(String::as_str),
        );
        Ok(())
    })
}

#[tauri::command]
pub async fn workforce_decide_proposal(
    store: State<'_, WorkforceStore>,
    proposal_id: String,
    decision: String,
) -> Result<Value, String> {
    let store = store.inner();
    let status = decision_status(&decision)?;
    let snapshot = snapshot_value(store)?;
    if !snapshot
        .get("proposals")
        .and_then(Value::as_array)
        .map(|proposals| {
            proposals
                .iter()
                .any(|proposal| proposal.get("id").and_then(Value::as_str) == Some(&proposal_id))
        })
        .unwrap_or(false)
    {
        return Err(format!("Unknown memory proposal: {proposal_id}"));
    }
    let pending = snapshot
        .get("proposals")
        .and_then(Value::as_array)
        .and_then(|proposals| {
            proposals
                .iter()
                .find(|proposal| proposal.get("id").and_then(Value::as_str) == Some(&proposal_id))
        })
        .and_then(|proposal| proposal.get("status").and_then(Value::as_str))
        == Some("pending");
    if !pending {
        return Err(format!("Memory proposal {proposal_id} is already decided."));
    }

    store.update(|value| {
        let at = now_ms();
        let proposal = value
            .get("proposals")
            .and_then(Value::as_array)
            .and_then(|proposals| {
                proposals
                    .iter()
                    .find(|proposal| proposal.get("id").and_then(Value::as_str) == Some(&proposal_id))
            })
            .cloned()
            .ok_or_else(|| format!("Unknown memory proposal: {proposal_id}"))?;
        if proposal.get("status").and_then(Value::as_str) != Some("pending") {
            return Err(format!("Memory proposal {proposal_id} is already decided."));
        }

        if let Some(proposals) = value.get_mut("proposals").and_then(Value::as_array_mut) {
            if let Some(target) = proposals
                .iter_mut()
                .find(|entry| entry.get("id").and_then(Value::as_str) == Some(&proposal_id))
            {
                target["status"] = json!(status);
                target["decidedAt"] = json!(at);
            }
        }

        if status == "approved" {
            apply_memory(value, &proposal, at);
        }

        let title = proposal.get("title").and_then(Value::as_str).unwrap_or("");
        let project_id = proposal
            .get("projectId")
            .and_then(Value::as_str)
            .unwrap_or("");
        let summary = format!("{status} memory proposal: {title}");
        push_activity(
            value,
            "memory",
            "orchestrator",
            Some(project_id),
            None,
            &summary,
            proposal.get("reason").and_then(Value::as_str),
        );
        Ok(())
    })
}

async fn run_once(
    runtime: &OpenCodeState,
    project: &str,
    title: &str,
    agent: &str,
    prompt: &str,
    permissions: Option<Value>,
) -> Result<(String, String), String> {
    require_policy(project)?;
    runtime.ensure(project).await?;
    let (provider_id, model_id) = model_config()?;
    let session = create_session(
        runtime,
        project,
        title,
        agent,
        &provider_id,
        &model_id,
        permissions,
    )
    .await
    .map_err(|error| format!("Could not start model {provider_id}/{model_id}: {error}"))?;
    let session_id = session_id_of(&session)
        .ok_or_else(|| "OpenCode did not return a session id.".to_string())?;
    prompt_and_wait(runtime, &session_id, prompt).await?;
    let messages = runtime
        .get_json(&format!(
            "/api/session/{session_id}/message?limit=200&order=asc"
        ))
        .await?;
    let text = last_assistant_text(&messages).unwrap_or_default();
    Ok((session_id, text))
}

async fn create_session(
    runtime: &OpenCodeState,
    project: &str,
    title: &str,
    agent: &str,
    provider_id: &str,
    model_id: &str,
    permissions: Option<Value>,
) -> Result<Value, String> {
    let mut body = json!({
        "title": title,
        "agent": agent,
        "model": { "id": model_id, "providerID": provider_id },
        "location": { "directory": project }
    });
    if let Some(permissions) = permissions {
        body["permissions"] = permissions;
    }
    let value = runtime.post_json("/api/session", body).await?;
    Ok(unwrap_data(value))
}

async fn prompt_and_wait(
    runtime: &OpenCodeState,
    session_id: &str,
    prompt: &str,
) -> Result<(), String> {
    let started = now_ms();
    runtime
        .post_json(
            &format!("/api/session/{session_id}/prompt"),
            json!({ "text": prompt }),
        )
        .await?;
    let deadline = Instant::now() + Duration::from_secs(IDLE_TIMEOUT_SECS);
    let mut approvals = 0usize;
    loop {
        let session = unwrap_data(
            runtime
                .get_json(&format!("/api/session/{session_id}"))
                .await?,
        );
        if session_outcome(&session).is_some() {
            return Ok(());
        }
        if session_idle(&session)
            .map(|idle_at| idle_at >= started)
            .unwrap_or(false)
        {
            return Ok(());
        }
        let permissions = runtime
            .get_json(&format!("/api/session/{session_id}/permission"))
            .await?;
        if has_permissions(&permissions) {
            if approvals >= MAX_AUTO_PERMISSIONS {
                return Err(format!(
                    "OpenCode session {session_id} requested more than {MAX_AUTO_PERMISSIONS} permissions in one turn."
                ));
            }
            let permission = first_permission(&permissions)
                .ok_or_else(|| "OpenCode returned an empty permission request.".to_string())?;
            let request_id = permission
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| "OpenCode permission request has no id.".to_string())?;
            runtime
                .post_json(
                    &format!("/api/session/{session_id}/permission/{request_id}/reply"),
                    json!({ "decision": "once" }),
                )
                .await?;
            approvals += 1;
            continue;
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "OpenCode session {session_id} did not become idle within {IDLE_TIMEOUT_SECS}s."
            ));
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn start_task(
    store: &WorkforceStore,
    runtime: &OpenCodeState,
    task_id: &str,
) -> Result<Value, String> {
    let _run_guard = store.run_lock.lock().await;
    let snapshot = snapshot_value(store)?;
    let task = find_task(&snapshot, task_id)
        .ok_or_else(|| format!("Unknown task: {task_id}"))?
        .clone();
    let status = task.get("status").and_then(Value::as_str);
    if !matches!(status, Some("queued") | Some("blocked")) {
        return Err(format!("Task {task_id} is not queued or retryable."));
    }
    if !dependencies_done(&snapshot, &task) {
        return Err(format!("Task {task_id} has unfinished dependencies."));
    }
    let owner = task
        .get("ownerId")
        .and_then(Value::as_str)
        .unwrap_or("orchestrator")
        .to_string();
    if !agent_exists(&snapshot, &owner) {
        return Err(format!("Task {task_id} has unknown owner: {owner}"));
    }
    let running_count = snapshot
        .get("tasks")
        .and_then(Value::as_array)
        .map(|tasks| {
            tasks
                .iter()
                .filter(|other| other.get("status").and_then(Value::as_str) == Some("running"))
                .count()
        })
        .unwrap_or(0);
    if running_count >= 2 {
        return Err("Concurrency limit reached: two tasks are already running.".to_string());
    }
    if snapshot
        .get("tasks")
        .and_then(Value::as_array)
        .map(|tasks| {
            tasks.iter().any(|other| {
                other.get("status").and_then(Value::as_str) == Some("running")
                    && other.get("ownerId").and_then(Value::as_str) == Some(owner.as_str())
            })
        })
        .unwrap_or(false)
    {
        return Err(format!("Agent {owner} already has a running task."));
    }
    let project_id = task
        .get("projectId")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Task {task_id} has no projectId."))?
        .to_string();
    let project = get_project(&snapshot, &project_id)?.clone();
    let repo = repo_path(&project)?;
    if !Path::new(&repo).is_dir() {
        return Err(format!("Project directory does not exist: {repo}"));
    }
    let worktree = create_isolated_worktree(store, &repo, task_id)?;
    let work_dir = worktree
        .as_ref()
        .map(|(path, _)| path.as_str())
        .unwrap_or(&repo)
        .to_string();
    require_policy(&work_dir)?;
    let (provider_id, model_id) = model_config()?;
    let agent_id = opencode_agent_id(&snapshot, &owner);
    let prompt_project = if worktree.is_some() {
        let mut project = project.clone();
        project["repoPath"] = json!(work_dir);
        project
    } else {
        project.clone()
    };
    let prompt = task_prompt(&snapshot, &prompt_project, &task, &agent_id);
    let started_at = now_ms();
    store.update(|value| {
        let current = find_task(value, task_id)
            .ok_or_else(|| format!("Unknown task: {task_id}"))?;
        let current_status = current.get("status").and_then(Value::as_str);
        if !matches!(current_status, Some("queued") | Some("blocked")) {
            return Err(format!("Task {task_id} was claimed by another run."));
        }
        if let Some(task) = find_task_mut(value, task_id) {
            task["status"] = json!("running");
            task["sessionId"] = Value::Null;
            task["result"] = Value::Null;
            task["finishedAt"] = Value::Null;
            task["startedAt"] = json!(started_at);
            task["updatedAt"] = json!(started_at);
            if let Some((path, branch)) = &worktree {
                task["worktreePath"] = json!(path);
                task["worktreeBranch"] = json!(branch);
            } else {
                task["worktreePath"] = Value::Null;
                task["worktreeBranch"] = Value::Null;
            }
        }
        set_agent_current_task(value, &owner, task_id);
        Ok(())
    })?;
    let title = format!(
        "{} {task_id} · {}",
        project_label(&project),
        task.get("title").and_then(Value::as_str).unwrap_or("task")
    );
    if let Err(error) = runtime.ensure(&work_dir).await {
        let note = format!("Could not start OpenCode: {error}");
        let _ = store.update(|value| {
            mark_blocked(value, task_id, &note);
            Ok(())
        });
        return Err(note);
    }
    let session = match create_session(
        runtime,
        &work_dir,
        &title,
        &agent_id,
        &provider_id,
        &model_id,
        None,
    )
    .await
    {
        Ok(session) => session,
        Err(error) => {
            let note = format!("Could not start model {provider_id}/{model_id}: {error}");
            let _ = store.update(|value| {
                mark_blocked(value, task_id, &note);
                Ok(())
            });
            return Err(note);
        }
    };
    let session_id = match session_id_of(&session) {
        Some(session_id) => session_id,
        None => {
            let note = "OpenCode did not return a session id.".to_string();
            let _ = store.update(|value| {
                mark_blocked(value, task_id, &note);
                Ok(())
            });
            return Err(note);
        }
    };
    store.update(|value| {
        if let Some(task) = find_task_mut(value, task_id) {
            task["sessionId"] = json!(session_id);
        }
        Ok(())
    })?;
    if let Err(error) = runtime
        .post_json(
            &format!("/api/session/{session_id}/prompt"),
            json!({ "text": prompt }),
        )
        .await
    {
        let note = format!("Session {session_id} failed to start: {error}");
        let _ = store.update(|value| {
            mark_blocked(value, task_id, &note);
            Ok(())
        });
        return Err(note);
    }

    store.update(|value| {
        let at = now_ms();
        if let Some(task) = find_task_mut(value, task_id) {
            task["status"] = json!("running");
            task["startedAt"] = json!(at);
            task["updatedAt"] = json!(at);
        }
        set_agent_current_task(value, &owner, task_id);
        let title = task.get("title").and_then(Value::as_str).unwrap_or("");
        push_activity(
            value,
            "status",
            &owner,
            Some(&project_id),
            Some(task_id),
            &format!("started {task_id}: {title}"),
            None,
        );
        Ok(())
    })
}

fn absolute_project_path(raw: &str) -> Result<PathBuf, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err("Folder path is required.".to_string());
    }
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(format!(
            "Folder path must be absolute, for example D:\\Projects\\my-app. Got {value:?}."
        ));
    }
    Ok(path)
}

fn validate_project_key(raw: &str) -> Result<String, String> {
    let key = raw.trim().to_ascii_uppercase();
    if key.len() < 2 || key.len() > 8 {
        return Err("Project key must be 2 to 8 characters.".to_string());
    }
    if !key
        .chars()
        .all(|character| character.is_ascii_uppercase() || character.is_ascii_digit())
    {
        return Err("Project key may contain only letters and digits.".to_string());
    }
    Ok(key)
}

fn project_id_for_key(key: &str) -> String {
    key.to_ascii_lowercase()
}

fn path_identity(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

fn ensure_starter_file(path: &Path, contents: &str) -> Result<bool, String> {
    if path.exists() {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create starter file folder: {error}"))?;
    }
    fs::write(path, contents).map_err(|error| {
        format!(
            "Could not create starter file {}: {error}",
            path.display()
        )
    })?;
    Ok(true)
}

fn starter_project_policy(name: &str) -> String {
    format!(
        "# {name} project rules\n\n\
This project is managed by Agent Bots. The global OpenCode policy still applies.\n\n\
- Work only inside this repository.\n\
- Read this file and the project README before changing code.\n\
- Keep changes inside the requested scope.\n\
- Run the project checks before reporting done.\n\
- Never add secrets, credentials, or private keys.\n\
- Ask for approval before destructive, external, or spending actions.\n\n\
## Checks\n\n\
Add the exact build, test, and lint commands for this project here.\n"
    )
}

fn starter_project_readme(name: &str, description: &str) -> String {
    let description = if description.trim().is_empty() {
        "New Agent Bots project."
    } else {
        description.trim()
    };
    format!(
        "# {name}\n\n{description}\n\n\
## Getting started\n\n\
Add setup and run instructions here.\n\n\
## Checks\n\n\
Add the build, test, and lint commands here.\n"
    )
}

fn is_seed_task_id(task_id: &str) -> bool {
    task_id
        .strip_prefix("t-")
        .map(|rest| !rest.contains('-'))
        .unwrap_or(false)
}

fn migrate_snapshot(snapshot: &mut Value) -> bool {
    let version = snapshot
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .unwrap_or(1);
    if version >= CURRENT_SCHEMA {
        return false;
    }

    if version < 3 {
        if let Some(tasks) = snapshot.get_mut("tasks").and_then(Value::as_array_mut) {
        for task in tasks {
            let id = task.get("id").and_then(Value::as_str).unwrap_or("");
            if !is_seed_task_id(id) {
                continue;
            }
            task["autoStart"] = json!(false);
            task["sessionId"] = Value::Null;
            let status = task.get("status").and_then(Value::as_str);
            if matches!(status, Some("running") | Some("review")) {
                task["status"] = json!("queued");
                task["sessionId"] = Value::Null;
                task["result"] = Value::Null;
                task["startedAt"] = Value::Null;
                task["finishedAt"] = Value::Null;
                append_note(
                    task,
                    "Migrated from the pre-runtime seed; no live OpenCode session exists.",
                );
            }
        }
    }

    if let Some(agents) = snapshot.get_mut("agents").and_then(Value::as_array_mut) {
        for agent in agents {
            let current = agent.get("currentTaskId").and_then(Value::as_str);
            let stale = match current {
                None => true,
                Some(task_id) => is_seed_task_id(task_id),
            };
            if stale {
                agent["currentTaskId"] = Value::Null;
                if agent.get("status").and_then(Value::as_str) == Some("active") {
                    agent["status"] = json!("idle");
                }
            }
        }
    }

    }

    if version < 2 {
        // The pre-runtime database contained a scripted transcript and sample
        // Dreamer data. Those are not valid live state and must not survive.
        snapshot["chat"] = json!([]);
        snapshot["activity"] = json!([]);
        snapshot["proposals"] = json!([]);
        snapshot["dreamerRuns"] = json!([]);
    }
    snapshot["schemaVersion"] = json!(CURRENT_SCHEMA);
    true
}

fn normalize_seed(snapshot: &mut Value) {
    if let Some(tasks) = snapshot.get_mut("tasks").and_then(Value::as_array_mut) {
        for task in tasks {
            // Seed tasks never start on their own. Only Plan-created tasks may
            // carry autoStart, and only after the user explicitly asks for a plan.
            task["autoStart"] = json!(false);
            task["sessionId"] = Value::Null;
            let status = task.get("status").and_then(Value::as_str);
            if matches!(status, Some("running") | Some("review")) {
                task["status"] = json!("queued");
                task["sessionId"] = Value::Null;
                task["result"] = Value::Null;
                task["startedAt"] = Value::Null;
                task["finishedAt"] = Value::Null;
                append_note(
                    task,
                    "Seeded as queued; no live OpenCode session exists for this task yet.",
                );
            }
        }
    }
    if let Some(agents) = snapshot.get_mut("agents").and_then(Value::as_array_mut) {
        for agent in agents {
            agent["currentTaskId"] = Value::Null;
            if agent.get("status").and_then(Value::as_str) == Some("active") {
                agent["status"] = json!("idle");
            }
        }
    }
    // Seed fixtures contain a scripted thread and sample proposals. The desktop
    // app starts with an empty live thread and no pending proposals.
    snapshot["chat"] = json!([]);
    snapshot["activity"] = json!([]);
    snapshot["proposals"] = json!([]);
    snapshot["dreamerRuns"] = json!([]);
    snapshot["schemaVersion"] = json!(CURRENT_SCHEMA);
}

fn ready_tasks(snapshot: &Value) -> Vec<String> {
    let mut tasks: Vec<&Value> = snapshot
        .get("tasks")
        .and_then(Value::as_array)
        .map(|tasks| {
            tasks
                .iter()
                .filter(|task| {
                    task.get("status").and_then(Value::as_str) == Some("queued")
                        && task.get("autoStart").and_then(Value::as_bool) == Some(true)
                        && dependencies_done(snapshot, task)
                })
                .collect()
        })
        .unwrap_or_default();
    tasks.sort_by_key(|task| task.get("createdAt").and_then(Value::as_u64).unwrap_or(0));
    tasks
        .into_iter()
        .filter_map(|task| task.get("id").and_then(Value::as_str).map(str::to_string))
        .take(20)
        .collect()
}

fn mark_review(snapshot: &mut Value, task_id: &str, text: &str) {
    if find_task(snapshot, task_id)
        .and_then(|task| task.get("status").and_then(Value::as_str))
        != Some("running")
    {
        return;
    }
    let (owner, project_id, title) = task_meta(snapshot, task_id);
    let at = now_ms();
    if let Some(task) = find_task_mut(snapshot, task_id) {
        task["status"] = json!("review");
        task["result"] = json!(text);
        task["updatedAt"] = json!(at);
        task["finishedAt"] = json!(at);
    }
    clear_agent_current_task(snapshot, &owner, task_id);
    push_activity(
        snapshot,
        "result",
        &owner,
        Some(&project_id),
        Some(task_id),
        &format!("finished {task_id} for review: {title}"),
        Some(text),
    );
}

fn mark_blocked(snapshot: &mut Value, task_id: &str, note: &str) {
    match find_task(snapshot, task_id).and_then(|task| task.get("status").and_then(Value::as_str)) {
        Some("blocked") | Some("done") => return,
        _ => {}
    }
    let (owner, project_id, title) = task_meta(snapshot, task_id);
    if let Some(task) = find_task_mut(snapshot, task_id) {
        task["status"] = json!("blocked");
        task["updatedAt"] = json!(now_ms());
        append_note(task, note);
    }
    clear_agent_current_task(snapshot, &owner, task_id);
    push_activity(
        snapshot,
        "status",
        &owner,
        Some(&project_id),
        Some(task_id),
        &format!("blocked {task_id}: {title}"),
        Some(note),
    );
}

fn task_meta(snapshot: &Value, task_id: &str) -> (String, String, String) {
    find_task(snapshot, task_id)
        .map(|task| {
            (
                task.get("ownerId")
                    .and_then(Value::as_str)
                    .unwrap_or("orchestrator")
                    .to_string(),
                task.get("projectId")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                task.get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            )
        })
        .unwrap_or_else(|| ("orchestrator".to_string(), String::new(), String::new()))
}

fn append_note(task: &mut Value, note: &str) {
    let existing = task
        .get("notes")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    task["notes"] = json!(if existing.is_empty() {
        note.to_string()
    } else {
        format!("{existing}\n{note}")
    });
}

fn apply_memory(snapshot: &mut Value, proposal: &Value, at: u64) {
    let action = proposal
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("add");
    let target = proposal.get("targetEntryId").and_then(Value::as_str);
    match action {
        "delete" => {
            if let Some(target) = target {
                if let Some(memory) = snapshot.get_mut("memory").and_then(Value::as_array_mut) {
                    memory.retain(|entry| entry.get("id").and_then(Value::as_str) != Some(target));
                }
            }
        }
        "update" => {
            let mut updated = false;
            if let Some(target) = target {
                if let Some(memory) = snapshot.get_mut("memory").and_then(Value::as_array_mut) {
                    if let Some(entry) = memory
                        .iter_mut()
                        .find(|entry| entry.get("id").and_then(Value::as_str) == Some(target))
                    {
                        copy_memory_fields(entry, proposal, at);
                        updated = true;
                    }
                }
            }
            if !updated {
                add_memory(snapshot, proposal, at);
            }
        }
        _ => add_memory(snapshot, proposal, at),
    }
}

fn copy_memory_fields(entry: &mut Value, proposal: &Value, at: u64) {
    if let Some(title) = proposal.get("title").and_then(Value::as_str) {
        entry["title"] = json!(title);
    }
    if let Some(kind) = proposal.get("kind").and_then(Value::as_str) {
        entry["kind"] = json!(kind);
    }
    if let Some(content) = proposal.get("content").and_then(Value::as_str) {
        entry["content"] = json!(content);
    }
    if let Some(confidence) = proposal.get("confidence").and_then(Value::as_f64) {
        entry["confidence"] = json!(confidence);
    }
    entry["updatedAt"] = json!(at);
}

fn add_memory(snapshot: &mut Value, proposal: &Value, at: u64) {
    let id = unique_id(
        snapshot.get("memory").and_then(Value::as_array),
        "mem",
        at,
    );
    let entry = json!({
        "id": id,
        "title": proposal.get("title").and_then(Value::as_str).unwrap_or(""),
        "kind": proposal.get("kind").and_then(Value::as_str).unwrap_or("reference"),
        "projectId": proposal.get("projectId").and_then(Value::as_str).unwrap_or(""),
        "content": proposal.get("content").and_then(Value::as_str).unwrap_or(""),
        "tags": [],
        "source": format!("Proposal {}", proposal.get("id").and_then(Value::as_str).unwrap_or("")),
        "confidence": proposal.get("confidence").and_then(Value::as_f64).unwrap_or(0.5),
        "createdAt": at,
        "updatedAt": at,
        "linkedTaskId": null
    });
    if let Some(memory) = snapshot.get_mut("memory").and_then(Value::as_array_mut) {
        memory.push(entry);
    }
}

fn normalize_proposal(value: &Value, project_id: &str, at: u64, index: usize) -> Value {
    let action = match value.get("action").and_then(Value::as_str) {
        Some("update") => "update",
        Some("delete") => "delete",
        Some("merge") => "merge",
        _ => "add",
    };
    let scope = match value.get("scope").and_then(Value::as_str) {
        Some("global") => "global",
        Some("agent") => "agent",
        _ => "project",
    };
    let kind = match value.get("kind").and_then(Value::as_str) {
        Some("decision") => "decision",
        Some("doc") => "doc",
        Some("result") => "result",
        Some("convention") => "convention",
        _ => "reference",
    };
    let confidence = value
        .get("confidence")
        .and_then(Value::as_f64)
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);
    let mut evidence: Vec<String> = value
        .get("evidence")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .take(20)
                .map(|item| {
                    item.as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| item.to_string())
                })
                .collect()
        })
        .unwrap_or_default();
    if evidence.is_empty() {
        evidence.push("Dreamer run".to_string());
    }
    json!({
        "id": format!("prop-{at}-{index}"),
        "action": action,
        "scope": scope,
        "projectId": project_id,
        "agentId": "dreamer",
        "targetEntryId": value.get("targetEntryId").and_then(Value::as_str),
        "kind": kind,
        "title": value.get("title").and_then(Value::as_str).unwrap_or(""),
        "content": value.get("content").and_then(Value::as_str).unwrap_or(""),
        "reason": value.get("reason").and_then(Value::as_str).unwrap_or(""),
        "evidence": evidence,
        "confidence": confidence,
        "conflictsWith": null,
        "status": "pending",
        "createdAt": at,
        "decidedAt": null
    })
}

fn decision_status(raw: &str) -> Result<&'static str, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "approved" | "approve" | "accepted" | "accept" => Ok("approved"),
        "rejected" | "reject" => Ok("rejected"),
        other => Err(format!(
            "Unknown proposal decision {other:?}. Expected approved or rejected."
        )),
    }
}

fn plan_prompt(snapshot: &Value, project: &Value, goal: &str) -> String {
    let owners = available_owners(snapshot).join(", ");
    let titles = project_task_titles(snapshot, project);
    let goal = if goal.trim().is_empty() {
        "(not provided; plan the next useful step from the project context)"
    } else {
        goal.trim()
    };
    format!(
        "You are the workforce orchestrator planning work for one project.\n\n{context}\n\nGoal from the user:\n{goal}\n\nExisting tasks:\n{titles}\n\nAvailable owner ids: {owners}\n\nReturn ONLY a JSON object. No prose, no markdown fences.\n{{\"summary\":\"one or two sentences\",\"tasks\":[{{\"title\":\"short title\",\"description\":\"one deliverable\",\"ownerId\":\"pixel\",\"priority\":\"normal\",\"acceptance\":\"how to verify\",\"dependsOn\":[0]}}]}}\n\nRules:\n- One deliverable per task.\n- ownerId must be one of the available owner ids.\n- priority is low, normal, high, or critical.\n- dependsOn contains zero-based indices of earlier tasks in this array, or an empty array.\n- acceptance states the check that proves the task is done.",
        context = project_context(project)
    )
}

fn send_prompt(snapshot: &Value, project: &Value, text: &str) -> String {
    let project_id = project.get("id").and_then(Value::as_str).unwrap_or("");
    let mut lines = Vec::new();
    if let Some(chat) = snapshot.get("chat").and_then(Value::as_array) {
        for message in chat
            .iter()
            .filter(|message| chat_message_in_project(snapshot, message, project_id))
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let role = message.get("role").and_then(Value::as_str).unwrap_or("user");
            let body = message.get("text").and_then(Value::as_str).unwrap_or("");
            lines.push(format!("{role}: {}", truncate(body, 500)));
        }
    }
    if lines.is_empty() {
        lines.push("(no earlier messages)".to_string());
    }
    format!(
        "{}\n\nRecent conversation:\n{}\n\nUser:\n{}\n\nReply as the orchestrator. Be concrete, short, and honest about what is simulated or not run.",
        project_context(project),
        lines.join("\n"),
        text.trim()
    )
}

fn task_prompt(snapshot: &Value, project: &Value, task: &Value, agent_id: &str) -> String {
    let repo = project.get("repoPath").and_then(Value::as_str).unwrap_or("");
    let global_policy = read_capped(&global_policy_path(), PROMPT_BUDGET);
    let project_policy = read_capped(&Path::new(repo).join("AGENTS.md"), PROMPT_BUDGET);
    let project_agent = Path::new(repo)
        .join(".opencode")
        .join("agents")
        .join(format!("{agent_id}.md"));
    let agent_policy = read_capped(&project_agent, PROMPT_BUDGET);
    let agent_policy = if agent_policy.trim().is_empty() {
        let global_agent = global_policy_path()
            .parent()
            .map(|path| path.join("agent").join(format!("{agent_id}.md")))
            .unwrap_or_else(|| PathBuf::from(format!("{agent_id}.md")));
        read_capped(&global_agent, PROMPT_BUDGET)
    } else {
        agent_policy
    };
    let agent_policy = strip_frontmatter(&agent_policy);
    let project_id = task.get("projectId").and_then(Value::as_str).unwrap_or("");
    let project_name = project
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(project_id);
    let vault = devvault_path();
    let hub_note = vault
        .join("Projects")
        .join(format!("{}.md", project_name.replace(' ', "-")));
    let memory = memory_context(snapshot, project_id, 8);
    let handoff = handoff_contract(repo);
    let acceptance = task
        .get("acceptance")
        .and_then(Value::as_str)
        .unwrap_or("No acceptance criteria recorded.");
    format!(
        "# Global policy\n\n{global}\n\n# Project policy\n\n{project_policy}\n\n# Agent policy\n\n{agent_policy}\n\n# Task\n\nid: {id}\ntitle: {title}\ndescription: {description}\nacceptance: {acceptance}\n\n# Approved project memory\n\n{memory}\n\n# DevVault\n\nVault: {vault}\nHome: {home}\nProject hub: {hub}\n\nThe global policy requires DevVault updates after meaningful work. If this task changes architecture, behavior, APIs, data, configuration, verified commands, or reusable components, update the project hub note, its row in DevVault/Home.md, and the matching project docs before finishing. Read the existing note first; link to project docs instead of copying them. Do not touch DevVault for read-only or no-change work.\n\n# Handoff contract\n\n{handoff}\n\nWork only inside {repo}. Report what changed, the exact checks run, and what was not checked.",
        global = none_if_empty(&global_policy),
        project_policy = none_if_empty(&project_policy),
        agent_policy = none_if_empty(&agent_policy),
        id = task.get("id").and_then(Value::as_str).unwrap_or(""),
        title = task.get("title").and_then(Value::as_str).unwrap_or(""),
        description = task.get("description").and_then(Value::as_str).unwrap_or(""),
        memory = none_if_empty(&memory),
        vault = vault.display(),
        home = vault.join("Home.md").display(),
        hub = hub_note.display(),
        handoff = none_if_empty(&handoff)
    )
}

fn dreamer_prompt(snapshot: &Value, project: &Value) -> String {
    let project_id = project.get("id").and_then(Value::as_str).unwrap_or("");
    let mut tasks = Vec::new();
    if let Some(list) = snapshot.get("tasks").and_then(Value::as_array) {
        for task in list.iter().filter(|task| {
            task.get("projectId").and_then(Value::as_str) == Some(project_id)
                && matches!(
                    task.get("status").and_then(Value::as_str),
                    Some("done") | Some("review")
                )
        }) {
            tasks.push(format!(
                "- {} [{}] {}: {}",
                task.get("id").and_then(Value::as_str).unwrap_or(""),
                task.get("status").and_then(Value::as_str).unwrap_or(""),
                task.get("title").and_then(Value::as_str).unwrap_or(""),
                truncate(
                    task.get("result").and_then(Value::as_str).unwrap_or(""),
                    1_000
                )
            ));
            if tasks.len() >= 40 {
                break;
            }
        }
    }
    let memory = all_memory_context(snapshot, project_id, 40);
    format!(
        "{}\n\nCompleted or review tasks:\n{}\n\nExisting memory:\n{}\n\nReturn ONLY a JSON object. No prose, no markdown fences.\n{{\"findings\":[\"fact\"],\"proposals\":[{{\"action\":\"add\",\"scope\":\"project\",\"kind\":\"decision\",\"title\":\"short title\",\"content\":\"durable fact\",\"reason\":\"why it matters\",\"evidence\":[\"task id or file\"],\"confidence\":0.8,\"targetEntryId\":null}}]}}\n\nRules:\n- One proposal per durable fact.\n- Evidence is required.\n- Allowed kinds: decision, reference, doc, result, convention.\n- Allowed actions: add, update, delete, merge.\n- Never include secrets, credentials, or raw logs.\n- If nothing durable was found, return empty arrays.",
        project_context(project),
        if tasks.is_empty() { "(none)".to_string() } else { tasks.join("\n") },
        if memory.is_empty() { "(none)".to_string() } else { memory.join("\n") }
    )
}

fn chat_message_in_project(snapshot: &Value, message: &Value, project_id: &str) -> bool {
    let refs = message.get("refs").and_then(Value::as_array);
    let Some(refs) = refs else {
        return true;
    };
    if refs.is_empty() {
        return true;
    }
    refs.iter().any(|reference| {
        match reference.get("kind").and_then(Value::as_str) {
            Some("project") => {
                reference.get("id").and_then(Value::as_str) == Some(project_id)
            }
            Some("task") => reference
                .get("id")
                .and_then(Value::as_str)
                .and_then(|task_id| find_task(snapshot, task_id))
                .and_then(|task| task.get("projectId").and_then(Value::as_str))
                == Some(project_id),
            Some("agent") => true,
            _ => false,
        }
    })
}

fn project_context(project: &Value) -> String {
    let field = |key: &str| {
        project
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    format!(
        "Project: {} ({})\nRepo: {}\nDescription: {}\nContext: {}\nInstructions: {}",
        field("name"),
        field("key"),
        field("repoPath"),
        field("description"),
        field("context"),
        field("instructions")
    )
}

fn project_task_titles(snapshot: &Value, project: &Value) -> String {
    let project_id = project.get("id").and_then(Value::as_str).unwrap_or("");
    let mut lines = Vec::new();
    if let Some(tasks) = snapshot.get("tasks").and_then(Value::as_array) {
        for task in tasks.iter().filter(|task| {
            task.get("projectId").and_then(Value::as_str) == Some(project_id)
        }) {
            lines.push(format!(
                "- [{}] {}",
                task.get("status").and_then(Value::as_str).unwrap_or(""),
                task.get("title").and_then(Value::as_str).unwrap_or("")
            ));
        }
    }
    if lines.is_empty() {
        "(none)".to_string()
    } else {
        lines.join("\n")
    }
}

fn memory_context(snapshot: &Value, project_id: &str, limit: usize) -> String {
    all_memory_context(snapshot, project_id, limit).join("\n")
}

fn all_memory_context(snapshot: &Value, project_id: &str, limit: usize) -> Vec<String> {
    snapshot
        .get("memory")
        .and_then(Value::as_array)
        .map(|memory| {
            memory
                .iter()
                .rev()
                .filter(|entry| entry.get("projectId").and_then(Value::as_str) == Some(project_id))
                .take(limit)
                .map(|entry| {
                    format!(
                        "- {}: {}",
                        entry.get("title").and_then(Value::as_str).unwrap_or(""),
                        truncate(
                            entry.get("content").and_then(Value::as_str).unwrap_or(""),
                            600
                        )
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn handoff_contract(repo: &str) -> String {
    let project_path = Path::new(repo).join("prompts").join("shared").join("handoff.md");
    let fallback = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("prompts")
        .join("shared")
        .join("handoff.md");
    let text = read_capped(&project_path, PROMPT_BUDGET);
    if text.is_empty() {
        read_capped(&fallback, PROMPT_BUDGET)
    } else {
        text
    }
}

fn read_capped(path: &Path, cap: usize) -> String {
    match fs::read_to_string(path) {
        Ok(text) if text.chars().count() > cap => {
            let mut clipped: String = text.chars().take(cap).collect();
            clipped.push_str("\n[truncated]");
            clipped
        }
        Ok(text) => text,
        Err(_) => String::new(),
    }
}

fn strip_frontmatter(text: &str) -> String {
    if let Some(rest) = text.strip_prefix("---") {
        if let Some(end) = rest.find("\n---") {
            return rest[end + 4..].trim_start().to_string();
        }
    }
    text.trim_start().to_string()
}

fn none_if_empty(value: &str) -> &str {
    if value.trim().is_empty() {
        "(none)"
    } else {
        value
    }
}

fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        value.to_string()
    } else {
        value.chars().take(limit).collect()
    }
}

fn require_policy(repo: &str) -> Result<(), String> {
    let global = global_policy_path();
    if !global.is_file() {
        return Err(format!(
            "Global policy is missing: {}. A real run cannot start.",
            global.display()
        ));
    }
    let project = Path::new(repo).join("AGENTS.md");
    if !project.is_file() {
        return Err(format!(
            "Project policy is missing: {}. Add AGENTS.md before running work.",
            project.display()
        ));
    }
    Ok(())
}

fn repo_path(project: &Value) -> Result<String, String> {
    let repo = project
        .get("repoPath")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if repo.is_empty() {
        return Err("Project has no repoPath.".to_string());
    }
    Ok(repo.to_string())
}

fn git_output(repo: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|error| format!("Could not run git: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            stderr.trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn is_git_repo(repo: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn worktree_name(task_id: &str) -> String {
    task_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn create_isolated_worktree(
    store: &WorkforceStore,
    repo: &str,
    task_id: &str,
) -> Result<Option<(String, String)>, String> {
    if !is_git_repo(repo) {
        return Ok(None);
    }
    if !git_output(repo, &["status", "--porcelain"])?.trim().is_empty() {
        return Err(format!(
            "Project has uncommitted changes. Commit or stash them before an isolated task runs in {repo}."
        ));
    }

    let name = worktree_name(task_id);
    let branch = format!("agent-bots/{name}");
    let path = store.worktrees_root.join(&name);
    let path_text = path.to_string_lossy().to_string();

    let _ = git_output(repo, &["worktree", "remove", "--force", &path_text]);
    let _ = git_output(repo, &["branch", "-D", &branch]);
    if path.exists() {
        fs::remove_dir_all(&path)
            .map_err(|error| format!("Could not clear stale worktree {}: {error}", path.display()))?;
    }

    git_output(repo, &["worktree", "add", "-b", &branch, &path_text, "HEAD"])?;
    Ok(Some((path_text, branch)))
}

fn merge_isolated_worktree(repo: &str, task: &Value) -> Result<Option<String>, String> {
    let Some(path) = task.get("worktreePath").and_then(Value::as_str) else {
        return Ok(None);
    };
    let Some(branch) = task.get("worktreeBranch").and_then(Value::as_str) else {
        return Ok(None);
    };
    if !Path::new(path).is_dir() {
        return Err(format!("Isolated worktree is missing: {path}"));
    }

    if !git_output(path, &["status", "--porcelain"])?.trim().is_empty() {
        git_output(path, &["add", "-A"])?;
        let message = format!(
            "agent-bots: {}",
            task.get("title").and_then(Value::as_str).unwrap_or("task result")
        );
        git_output(
            path,
            &[
                "-c",
                "user.name=Agent Bots",
                "-c",
                "user.email=agent-bots@local",
                "commit",
                "-m",
                &message,
            ],
        )?;
    }

    git_output(repo, &["merge", "--ff-only", branch])?;
    let _ = git_output(repo, &["worktree", "remove", "--force", path]);
    let _ = git_output(repo, &["branch", "-D", branch]);
    let _ = fs::remove_dir(path);
    Ok(Some(branch.to_string()))
}

fn project_label(project: &Value) -> String {
    project
        .get("key")
        .and_then(Value::as_str)
        .filter(|key| !key.is_empty())
        .or_else(|| project.get("name").and_then(Value::as_str))
        .unwrap_or("Project")
        .to_string()
}

fn available_owners(snapshot: &Value) -> Vec<String> {
    snapshot
        .get("agents")
        .and_then(Value::as_array)
        .map(|agents| {
            agents
                .iter()
                .filter_map(|agent| agent.get("id").and_then(Value::as_str))
                .filter(|id| !matches!(*id, "orchestrator" | "dreamer"))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn opencode_agent_id(snapshot: &Value, agent_id: &str) -> String {
    snapshot
        .get("agents")
        .and_then(Value::as_array)
        .and_then(|agents| {
            agents
                .iter()
                .find(|agent| agent.get("id").and_then(Value::as_str) == Some(agent_id))
        })
        .and_then(|agent| agent.get("opencodeAgentId").and_then(Value::as_str))
        .unwrap_or(agent_id)
        .to_string()
}

fn agent_exists(snapshot: &Value, agent_id: &str) -> bool {
    snapshot
        .get("agents")
        .and_then(Value::as_array)
        .map(|agents| {
            agents
                .iter()
                .any(|agent| agent.get("id").and_then(Value::as_str) == Some(agent_id))
        })
        .unwrap_or(false)
}

fn set_agent_current_task(snapshot: &mut Value, agent_id: &str, task_id: &str) {
    if let Some(agents) = snapshot.get_mut("agents").and_then(Value::as_array_mut) {
        if let Some(agent) = agents
            .iter_mut()
            .find(|agent| agent.get("id").and_then(Value::as_str) == Some(agent_id))
        {
            agent["currentTaskId"] = json!(task_id);
            agent["status"] = json!("active");
            agent["lastActiveAt"] = json!(now_ms());
        }
    }
}

fn clear_agent_current_task(snapshot: &mut Value, agent_id: &str, task_id: &str) {
    if let Some(agents) = snapshot.get_mut("agents").and_then(Value::as_array_mut) {
        if let Some(agent) = agents
            .iter_mut()
            .find(|agent| agent.get("id").and_then(Value::as_str) == Some(agent_id))
        {
            if agent.get("currentTaskId").and_then(Value::as_str) == Some(task_id) {
                agent["currentTaskId"] = Value::Null;
                agent["status"] = json!("idle");
            }
            agent["lastActiveAt"] = json!(now_ms());
        }
    }
}

fn push_activity(
    snapshot: &mut Value,
    kind: &str,
    actor_id: &str,
    project_id: Option<&str>,
    task_id: Option<&str>,
    summary: &str,
    detail: Option<&str>,
) {
    let at = now_ms();
    let id = unique_id(
        snapshot.get("activity").and_then(Value::as_array),
        "ev",
        at,
    );
    let event = json!({
        "id": id,
        "at": at,
        "kind": kind,
        "actorId": actor_id,
        "projectId": project_id,
        "taskId": task_id,
        "summary": summary,
        "detail": detail
    });
    if let Some(activity) = snapshot.get_mut("activity").and_then(Value::as_array_mut) {
        activity.insert(0, event);
    } else {
        snapshot["activity"] = json!([event]);
    }
}

fn push_chat(snapshot: &mut Value, role: &str, text: &str, refs: Vec<Value>) {
    let at = now_ms();
    let id = unique_id(snapshot.get("chat").and_then(Value::as_array), "chat", at);
    let message = json!({
        "id": id,
        "role": role,
        "text": text,
        "at": at,
        "refs": refs
    });
    if let Some(chat) = snapshot.get_mut("chat").and_then(Value::as_array_mut) {
        chat.push(message);
    } else {
        snapshot["chat"] = json!([message]);
    }
}

fn unique_id(existing: Option<&Vec<Value>>, prefix: &str, at: u64) -> String {
    for index in 1..10_000usize {
        let candidate = format!("{prefix}-{at}-{index}");
        let taken = existing
            .map(|items| {
                items
                    .iter()
                    .any(|item| item.get("id").and_then(Value::as_str) == Some(&candidate))
            })
            .unwrap_or(false);
        if !taken {
            return candidate;
        }
    }
    format!("{prefix}-{at}-overflow")
}

fn snapshot_value(store: &WorkforceStore) -> Result<Value, String> {
    store
        .snapshot()?
        .ok_or_else(|| "Workforce snapshot is not seeded".to_string())
}

fn find_task<'a>(snapshot: &'a Value, task_id: &str) -> Option<&'a Value> {
    snapshot
        .get("tasks")
        .and_then(Value::as_array)
        .and_then(|tasks| {
            tasks
                .iter()
                .find(|task| task.get("id").and_then(Value::as_str) == Some(task_id))
        })
}

fn find_task_mut<'a>(snapshot: &'a mut Value, task_id: &str) -> Option<&'a mut Value> {
    snapshot
        .get_mut("tasks")
        .and_then(Value::as_array_mut)
        .and_then(|tasks| {
            tasks
                .iter_mut()
                .find(|task| task.get("id").and_then(Value::as_str) == Some(task_id))
        })
}

fn get_project<'a>(snapshot: &'a Value, project_id: &str) -> Result<&'a Value, String> {
    snapshot
        .get("projects")
        .and_then(Value::as_array)
        .and_then(|projects| {
            projects
                .iter()
                .find(|project| project.get("id").and_then(Value::as_str) == Some(project_id))
        })
        .ok_or_else(|| format!("Unknown project: {project_id}"))
}

pub(crate) fn dependencies_done(snapshot: &Value, task: &Value) -> bool {
    task.get("dependsOn")
        .and_then(Value::as_array)
        .map(|dependencies| {
            dependencies.iter().all(|dependency| {
                dependency
                    .as_str()
                    .and_then(|id| task_status(snapshot, id))
                    == Some("done")
            })
        })
        .unwrap_or(true)
}

fn task_status<'a>(snapshot: &'a Value, task_id: &str) -> Option<&'a str> {
    find_task(snapshot, task_id).and_then(|task| task.get("status").and_then(Value::as_str))
}

pub(crate) fn extract_json_object(text: &str) -> Option<Value> {
    let start = text.find('{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, character) in text[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let end = start + offset + character.len_utf8();
                    return serde_json::from_str(&text[start..end]).ok();
                }
            }
            _ => {}
        }
    }
    None
}

fn unwrap_data(value: Value) -> Value {
    match value {
        Value::Object(mut map) => map.remove("data").unwrap_or(Value::Object(map)),
        other => other,
    }
}

fn session_id_of(session: &Value) -> Option<String> {
    session.get("id").and_then(Value::as_str).map(str::to_string)
}

fn session_idle(session: &Value) -> Option<u64> {
    session.get("time")?.get("idle")?.as_u64()
}

fn session_outcome(session: &Value) -> Option<&Value> {
    session.get("outcome").filter(|outcome| !outcome.is_null())
}

fn session_error(session: &Value) -> Option<String> {
    if let Some(error) = session.get("error").filter(|error| !error.is_null()) {
        return Some(error.to_string());
    }
    let outcome = session_outcome(session)?;
    if outcome.get("error").is_some() {
        return Some(outcome.to_string());
    }
    let text = outcome.to_string().to_ascii_lowercase();
    if text.contains("\"error\"")
        || text.contains("\"failed\"")
        || text.contains("\"failure\"")
        || outcome
            .as_str()
            .map(|value| matches!(value.to_ascii_lowercase().as_str(), "error" | "failed" | "failure"))
            .unwrap_or(false)
    {
        Some(outcome.to_string())
    } else {
        None
    }
}

fn has_permissions(value: &Value) -> bool {
    unwrap_data(value.clone())
        .as_array()
        .map(|items| !items.is_empty())
        .unwrap_or(false)
}

fn first_permission(value: &Value) -> Option<Value> {
    unwrap_data(value.clone()).as_array()?.first().cloned()
}

fn last_assistant_text(value: &Value) -> Option<String> {
    let messages = unwrap_data(value.clone());
    let messages = messages.as_array()?;
    for message in messages.iter().rev() {
        let is_assistant = message.get("type").and_then(Value::as_str) == Some("assistant")
            || message.pointer("/info/role").and_then(Value::as_str) == Some("assistant");
        if !is_assistant {
            continue;
        }
        let mut parts = Vec::new();
        let content = message
            .get("content")
            .and_then(Value::as_array)
            .or_else(|| message.get("parts").and_then(Value::as_array));
        if let Some(content) = content {
            for part in content {
                if part.get("type").and_then(Value::as_str) == Some("text") {
                    if let Some(text) = part.get("text").and_then(Value::as_str) {
                        if !text.is_empty() {
                            parts.push(text.to_string());
                        }
                    }
                }
            }
        }
        if !parts.is_empty() {
            return Some(parts.join("\n"));
        }
        if let Some(text) = message.get("text").and_then(Value::as_str) {
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }
    None
}

fn model_config() -> Result<(String, String), String> {
    let raw = std::env::var("AGENT_BOTS_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    split_model(&raw)
}

fn split_model(raw: &str) -> Result<(String, String), String> {
    let trimmed = raw.trim();
    let (provider, model) = trimmed
        .split_once('/')
        .ok_or_else(|| format!("AGENT_BOTS_MODEL must be provider/model, got {trimmed:?}."))?;
    if provider.trim().is_empty() || model.trim().is_empty() {
        return Err(format!(
            "AGENT_BOTS_MODEL must be provider/model with both parts set, got {trimmed:?}."
        ));
    }
    Ok((provider.trim().to_string(), model.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worktree_names_are_git_safe() {
        assert_eq!(worktree_name("t-1794-0"), "t-1794-0");
        assert_eq!(worktree_name("task/with spaces"), "task-with-spaces");
    }

    #[test]
    fn extracts_first_json_object_from_prose() {
        let text = "Here is the plan:\n```json\n{\"summary\":\"ok\",\"tasks\":[]}\n```";
        let value = extract_json_object(text).expect("object");
        assert_eq!(value["summary"], "ok");
    }

    #[test]
    fn ignores_braces_inside_strings() {
        let text = "noise {\"summary\":\"a } b\",\"tasks\":[]} tail";
        let value = extract_json_object(text).expect("object");
        assert_eq!(value["summary"], "a } b");
    }

    #[test]
    fn returns_none_without_json_object() {
        assert!(extract_json_object("no json here").is_none());
        assert!(extract_json_object("{\"unfinished\": true").is_none());
    }

    #[test]
    fn dependency_readiness_requires_all_done() {
        let snapshot = json!({
            "tasks": [
                { "id": "t-1", "status": "done" },
                { "id": "t-2", "status": "queued" },
                { "id": "t-3", "status": "queued", "dependsOn": ["t-1"] },
                { "id": "t-4", "status": "queued", "dependsOn": ["t-2"] },
                { "id": "t-5", "status": "queued", "dependsOn": ["t-9"] },
                { "id": "t-6", "status": "queued", "dependsOn": [] }
            ]
        });
        let task = |id: &str| {
            snapshot["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|task| task["id"] == id)
                .unwrap()
                .clone()
        };
        assert!(dependencies_done(&snapshot, &task("t-3")));
        assert!(!dependencies_done(&snapshot, &task("t-4")));
        assert!(!dependencies_done(&snapshot, &task("t-5")));
        assert!(dependencies_done(&snapshot, &task("t-6")));
        assert!(dependencies_done(&snapshot, &task("t-1")));
    }

    #[test]
    fn store_seeds_once_and_persists_updates() {
        let path = std::env::temp_dir().join(format!(
            "agent-bots-workforce-test-{}-{}.db",
            std::process::id(),
            now_ms()
        ));
        {
            let store = WorkforceStore::new(path.clone()).expect("store");
            assert!(store.snapshot().unwrap().is_none());
            let seeded = store.seed(json!({ "counter": 1 })).expect("seed");
            assert_eq!(seeded["counter"], 1);
            let second = store.seed(json!({ "counter": 2 })).expect("second seed");
            assert_eq!(second["counter"], 1);
            let updated = store
                .update(|value| {
                    value["counter"] = json!(3);
                    Ok(())
                })
                .expect("update");
            assert_eq!(updated["counter"], 3);
            assert_eq!(store.snapshot().unwrap().unwrap()["counter"], 3);
        }
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
    }

    #[test]
    fn readiness_respects_auto_start_and_dependencies() {
        let snapshot = json!({
            "tasks": [
                { "id": "t-done", "status": "done" },
                { "id": "t-ready", "status": "queued", "autoStart": true, "createdAt": 1 },
                { "id": "t-manual", "status": "queued", "autoStart": false, "createdAt": 2 },
                { "id": "t-waiting", "status": "queued", "autoStart": true, "dependsOn": ["t-done"], "createdAt": 3 },
                { "id": "t-blocked", "status": "blocked", "autoStart": true, "createdAt": 4 }
            ]
        });
        assert_eq!(
            ready_tasks(&snapshot),
            vec!["t-ready".to_string(), "t-waiting".to_string()]
        );
    }

    #[test]
    fn seed_normalization_removes_fake_sessions() {
        let mut snapshot = json!({
            "tasks": [
                { "id": "t-1", "status": "running", "sessionId": "ses-fake", "result": "fake", "autoStart": true },
                { "id": "t-2", "status": "review", "sessionId": null, "result": "fake", "autoStart": false },
                { "id": "t-3", "status": "done" }
            ],
            "agents": [
                { "id": "pixel", "status": "active", "currentTaskId": "t-1" }
            ],
            "chat": [{ "id": "chat-old", "text": "scripted" }],
            "proposals": [{ "id": "prop-old" }],
            "dreamerRuns": [{ "id": "run-old" }]
        });
        normalize_seed(&mut snapshot);
        assert_eq!(snapshot["tasks"][0]["status"], "queued");
        assert_eq!(snapshot["tasks"][1]["status"], "queued");
        assert_eq!(snapshot["tasks"][0]["sessionId"], serde_json::Value::Null);
        assert_eq!(snapshot["tasks"][1]["result"], serde_json::Value::Null);
        assert_eq!(snapshot["tasks"][0]["autoStart"], false);
        assert_eq!(snapshot["tasks"][1]["autoStart"], false);
        assert_eq!(snapshot["tasks"][2]["status"], "done");
        assert_eq!(snapshot["agents"][0]["status"], "idle");
        assert_eq!(snapshot["agents"][0]["currentTaskId"], serde_json::Value::Null);
        assert_eq!(snapshot["chat"].as_array().unwrap().len(), 0);
        assert_eq!(snapshot["activity"].as_array().unwrap().len(), 0);
        assert_eq!(snapshot["proposals"].as_array().unwrap().len(), 0);
        assert_eq!(snapshot["dreamerRuns"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn migrates_old_seed_without_touching_plan_tasks() {
        let mut snapshot = json!({
            "tasks": [
                { "id": "t-102", "status": "review", "autoStart": true, "sessionId": "ses-fake", "result": "fake" },
                { "id": "t-205", "status": "blocked", "autoStart": false, "sessionId": "ses-fake-blocked" },
                { "id": "t-1700000000000-0", "status": "queued", "autoStart": true }
            ],
            "agents": [
                { "id": "pixel", "status": "active", "currentTaskId": "t-102" }
            ],
            "chat": [{ "id": "chat-old" }],
            "activity": [{ "id": "ev-old" }],
            "proposals": [{ "id": "prop-old" }],
            "dreamerRuns": [{ "id": "run-old" }]
        });
        assert!(migrate_snapshot(&mut snapshot));
        assert_eq!(snapshot["tasks"][0]["status"], "queued");
        assert_eq!(snapshot["tasks"][0]["autoStart"], false);
        assert_eq!(snapshot["tasks"][0]["sessionId"], serde_json::Value::Null);
        assert_eq!(snapshot["tasks"][1]["status"], "blocked");
        assert_eq!(snapshot["tasks"][1]["sessionId"], serde_json::Value::Null);
        assert_eq!(snapshot["tasks"][2]["autoStart"], true);
        assert_eq!(snapshot["agents"][0]["status"], "idle");
        assert_eq!(snapshot["chat"].as_array().unwrap().len(), 0);
        assert_eq!(snapshot["schemaVersion"], json!(CURRENT_SCHEMA));
        assert!(!migrate_snapshot(&mut snapshot));
    }

    #[test]
    fn migration_v2_keeps_chat_but_clears_seed_session() {
        let mut snapshot = json!({
            "schemaVersion": 2,
            "tasks": [
                { "id": "t-205", "status": "blocked", "sessionId": "ses-fake" }
            ],
            "chat": [{ "id": "real-chat", "text": "keep" }]
        });
        assert!(migrate_snapshot(&mut snapshot));
        assert_eq!(snapshot["tasks"][0]["sessionId"], serde_json::Value::Null);
        assert_eq!(snapshot["chat"].as_array().unwrap().len(), 1);
        assert_eq!(snapshot["schemaVersion"], json!(CURRENT_SCHEMA));
    }

    #[test]
    fn review_and_blocked_transitions_do_not_overwrite_terminal_states() {
        let mut snapshot = json!({
            "tasks": [{ "id": "t-1", "status": "done" }],
            "activity": [],
            "agents": []
        });
        mark_review(&mut snapshot, "t-1", "late result");
        mark_blocked(&mut snapshot, "t-1", "late failure");
        assert_eq!(snapshot["tasks"][0]["status"], "done");
        assert_eq!(snapshot["tasks"][0]["result"], serde_json::Value::Null);
        assert_eq!(snapshot["activity"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn owner_list_excludes_control_agents() {
        let snapshot = json!({
            "agents": [
                { "id": "orchestrator" },
                { "id": "dreamer" },
                { "id": "pixel" }
            ]
        });
        assert_eq!(available_owners(&snapshot), vec!["pixel".to_string()]);
    }

    #[test]
    fn extracts_first_permission_request() {
        let value = json!({
            "data": [
                { "id": "p1", "action": "playwright_browser_navigate", "resources": ["https://example.com"] },
                { "id": "p2", "action": "shell", "resources": ["npm test"] }
            ]
        });
        let permission = first_permission(&value).expect("permission");
        assert_eq!(permission["id"], "p1");
        assert_eq!(permission["action"], "playwright_browser_navigate");
        assert!(first_permission(&json!({ "data": [] })).is_none());
    }

    #[test]
    fn validates_project_keys_and_path_identity() {
        assert_eq!(validate_project_key(" mnap ").unwrap(), "MNAP");
        assert!(validate_project_key("A").is_err());
        assert!(validate_project_key("BAD KEY").is_err());
        assert!(validate_project_key("TOOLONGKEY").is_err());
        assert_eq!(project_id_for_key("MNAP"), "mnap");
        assert_eq!(
            path_identity(Path::new("D:\\Projects\\My-App\\")),
            "d:/projects/my-app"
        );
        assert_eq!(
            path_identity(Path::new("d:/projects/my-app")),
            "d:/projects/my-app"
        );
    }

    #[test]
    fn starter_files_are_created_once() {
        let dir = std::env::temp_dir().join(format!("agent-bots-test-{}", now_ms()));
        fs::create_dir_all(&dir).unwrap();
        let policy = dir.join("AGENTS.md");
        assert!(ensure_starter_file(&policy, "policy").unwrap());
        assert!(!ensure_starter_file(&policy, "other").unwrap());
        assert_eq!(fs::read_to_string(&policy).unwrap(), "policy");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn starter_policy_does_not_copy_global_policy() {
        let policy = starter_project_policy("Example");
        assert!(policy.contains("# Example project rules"));
        assert!(policy.contains("## Checks"));
        assert!(!policy.contains("Caveman mode"));
        assert!(!policy.contains("API key"));
    }

    #[test]
    fn maps_app_agent_to_opencode_agent() {
        let snapshot = json!({
            "agents": [
                { "id": "orchestrator", "opencodeAgentId": "orchestrator-workforce" },
                { "id": "pixel" }
            ]
        });
        assert_eq!(opencode_agent_id(&snapshot, "orchestrator"), "orchestrator-workforce");
        assert_eq!(opencode_agent_id(&snapshot, "pixel"), "pixel");
    }

    #[test]
    fn parses_provider_model_pair() {
        assert_eq!(
            split_model("opencode-go/deepseek-v4.1-flash").unwrap(),
            ("opencode-go".to_string(), "deepseek-v4.1-flash".to_string())
        );
        assert!(split_model("deepseek-v4.1-flash").is_err());
        assert!(split_model("/deepseek").is_err());
        assert!(split_model("opencode-go/").is_err());
    }
}
