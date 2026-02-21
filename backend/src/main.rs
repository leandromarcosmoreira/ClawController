mod db;
mod models;

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, State},
    routing::{get, post, patch, put},
    Router,
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use models::*;
use tokio::process::Command;
use chrono::Utc;

// Connection Manager for WebSockets
pub struct ConnectionManager {
    tx: broadcast::Sender<String>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn broadcast(&self, message: &str) {
        let _ = self.tx.send(message.to_string());
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}

struct AppState {
    pool: SqlitePool,
    manager: Arc<ConnectionManager>,
    gateway_status: RwLock<GatewayStatus>,
    stuck_task_status: RwLock<StuckTaskStatus>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = db::setup_db().await?;
    let manager = Arc::new(ConnectionManager::new());
    
    let gateway_status = RwLock::new(GatewayStatus {
        health_status: "unknown".to_string(),
        uptime_seconds: 0,
        last_check_time: Utc::now(),
        restart_count: 0,
        config: GatewayConfig {
            check_interval_seconds: 60,
            health_check_timeout: 5,
            max_restart_attempts: 3,
            notification_cooldown_minutes: 30,
        },
    });

    let stuck_task_status = RwLock::new(StuckTaskStatus {
        total_notifications_sent: 0,
        currently_tracked_tasks: 0,
        last_run: Utc::now(),
        config: MonitoringConfig {
            normal_priority_limit_minutes: 120,
            urgent_priority_limit_minutes: 30,
        },
    });

    let state = Arc::new(AppState { pool, manager, gateway_status, stuck_task_status });

    let api_routes = Router::new()
        .route("/agents", get(get_agents).post(create_agent))
        .route("/agents/:id", get(get_agent).patch(update_agent).delete(delete_agent))
        .route("/tasks", get(get_tasks).post(create_task))
        .route("/tasks/:id", get(get_task).patch(update_task).delete(delete_task))
        .route("/tasks/:id/comments", get(get_comments).post(create_comment))
        .route("/announcements", get(get_announcements).post(create_announcement))
        .route("/activity", get(get_activity))
        .route("/tasks/:id/activity", get(get_task_activity).post(add_task_activity))
        .route("/tasks/:id/deliverables", get(get_deliverables).post(create_deliverable))
        .route("/tasks/:id/route", post(route_task))
        .route("/recurring", get(list_recurring_tasks).post(create_recurring_task))
        .route("/recurring/:id/trigger", post(trigger_recurring_task))
        .route("/recurring/:id/runs", get(get_recurring_task_runs))
        .route("/stats", get(get_stats))
        .route("/chat", get(get_chat_messages).post(send_chat_message))
        .route("/chat/send-to-agent", post(send_chat_message_to_agent))
        .route("/models", get(get_models))
        .route("/agents/generate", post(generate_agent_config))
        .route("/agents/:id/files", get(get_agent_files).put(update_agent_files))
        .route("/tasks/:id/review", post(review_task))
        .route("/deliverables/:id/complete", patch(complete_deliverable))
        .route("/openclaw/status", get(check_openclaw_status))
        .route("/openclaw/agents", get(fetch_openclaw_agents))
        .route("/openclaw/import", post(import_openclaw_agents))
        .route("/monitoring/gateway/status", get(get_gateway_status))
        .route("/monitoring/gateway/restart", post(restart_gateway))
        .route("/monitoring/stuck-tasks/status", get(get_stuck_task_status))
        .route("/monitoring/stuck-tasks/check", post(run_stuck_task_check));

    let app = Router::new()
        .route("/", get(root))
        .route("/ws", get(ws_handler))
        .nest("/api", api_routes)
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    // Spawn background tasks
    let state_task = state.clone();
    tokio::spawn(async move {
        loop {
            // Run checks immediately on start and then every minute
            tracing::info!("Running background checks...");
            
            // 1. Check Gateway Health
            let gateway_active = check_gateway_connectivity().await;
            {
                let mut status = state_task.gateway_status.write().await;
                status.health_status = if gateway_active { "healthy".to_string() } else { "crashed".to_string() };
                status.last_check_time = Utc::now();
                if gateway_active {
                    status.uptime_seconds += 60;
                } else {
                    status.uptime_seconds = 0;
                }
            }

            // 2. Monitor Stuck Tasks
            if let Ok(stuck_count) = perform_stuck_task_check(&state_task.pool).await {
                let mut status = state_task.stuck_task_status.write().await;
                status.currently_tracked_tasks = stuck_count as u32;
                status.last_run = Utc::now();
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "ClawController API (Rust) is running"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.manager.subscribe();
    
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if socket.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // We don't expect messages from client for now, but we need to stay alive
    tokio::select! {
        _ = &mut send_task => {},
    }
}

async fn get_agents(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Agent>>, (StatusCode, String)> {
    let agents = sqlx::query_as::<sqlx::Sqlite, Agent>(
        "SELECT id, name, role, description, avatar, status, workspace, token, primary_model, fallback_model, current_model, model_failure_count, created_at FROM agents"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(agents))
}

async fn get_agent(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let agent = sqlx::query_as::<sqlx::Sqlite, Agent>(
        "SELECT id, name, role, description, avatar, status, workspace, token, primary_model, fallback_model, current_model, model_failure_count, created_at FROM agents WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(agent))
}

async fn create_agent(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    // Basic implementation for now
    let id = uuid::Uuid::new_v4().to_string();
    let name = payload["name"].as_str().unwrap_or("Unknown");
    
    sqlx::query("INSERT INTO agents (id, name, role, status, created_at) VALUES (?, ?, 'SPC', 'IDLE', CURRENT_TIMESTAMP)")
        .bind(&id)
        .bind(name)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    get_agent(Path(id), State(state)).await
}

async fn update_agent(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    if let Some(status) = payload["status"].as_str() {
        sqlx::query("UPDATE agents SET status = ? WHERE id = ?")
            .bind(status)
            .bind(&id)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    
    get_agent(Path(id), State(state)).await
}

async fn delete_agent(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM agents WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_tasks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    let tasks = sqlx::query_as::<sqlx::Sqlite, Task>(
        "SELECT id, title, description, status, priority, tags, assignee_id, reviewer, reviewer_id, created_at, updated_at, due_at FROM tasks"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(tasks))
}

async fn get_task(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let task = sqlx::query_as::<sqlx::Sqlite, Task>(
        "SELECT id, title, description, status, priority, tags, assignee_id, reviewer, reviewer_id, created_at, updated_at, due_at FROM tasks WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(task))
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let id = uuid::Uuid::new_v4().to_string();
    let title = payload["title"].as_str().ok_or((StatusCode::BAD_REQUEST, "Title required".to_string()))?;
    
    sqlx::query("INSERT INTO tasks (id, title, status, priority, created_at, updated_at) VALUES (?, ?, 'INBOX', 'NORMAL', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)")
        .bind(&id)
        .bind(title)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.manager.broadcast(&format!(r#"{{"type": "task_created", "task_id": "{}"}}"#, id));

    get_task(Path(id), State(state)).await
}

async fn update_task(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Task>, (StatusCode, String)> {
    if let Some(status) = payload["status"].as_str() {
        sqlx::query("UPDATE tasks SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(status)
            .bind(&id)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            
        state.manager.broadcast(&format!(r#"{{"type": "status_changed", "task_id": "{}", "status": "{}"}}"#, id, status));
    }
    
    get_task(Path(id), State(state)).await
}

async fn delete_task(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_comments(
    Path(task_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Comment>>, (StatusCode, String)> {
    let comments = sqlx::query_as::<sqlx::Sqlite, Comment>(
        "SELECT id, task_id, agent_id, content, created_at FROM comments WHERE task_id = ?"
    )
    .bind(task_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(comments))
}

async fn create_comment(
    Path(task_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Comment>, (StatusCode, String)> {
    let id = uuid::Uuid::new_v4().to_string();
    let agent_id = payload["agent_id"].as_str().ok_or((StatusCode::BAD_REQUEST, "agent_id required".to_string()))?;
    let content = payload["content"].as_str().ok_or((StatusCode::BAD_REQUEST, "content required".to_string()))?;

    sqlx::query("INSERT INTO comments (id, task_id, agent_id, content, created_at) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)")
        .bind(&id)
        .bind(&task_id)
        .bind(agent_id)
        .bind(content)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let comment = sqlx::query_as::<sqlx::Sqlite, Comment>(
        "SELECT id, task_id, agent_id, content, created_at FROM comments WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.manager.broadcast(&format!(r#"{{"type": "comment_added", "task_id": "{}"}}"#, task_id));

    Ok(Json(comment))
}

async fn get_announcements(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Announcement>>, (StatusCode, String)> {
    let announcements = sqlx::query_as::<sqlx::Sqlite, Announcement>(
        "SELECT id, title, message, priority, created_at, created_by FROM announcements ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(announcements))
}

async fn create_announcement(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Announcement>, (StatusCode, String)> {
    let id = uuid::Uuid::new_v4().to_string();
    let message = payload["message"].as_str().ok_or((StatusCode::BAD_REQUEST, "message required".to_string()))?;
    let title = payload["title"].as_str();
    let priority = payload["priority"].as_str().unwrap_or("NORMAL");

    sqlx::query("INSERT INTO announcements (id, title, message, priority, created_at, created_by) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, 'human')")
        .bind(&id)
        .bind(title)
        .bind(message)
        .bind(priority)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let announcement = sqlx::query_as::<sqlx::Sqlite, Announcement>(
        "SELECT id, title, message, priority, created_at, created_by FROM announcements WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.manager.broadcast(r#"{"type": "announcement_created"}"#);

    Ok(Json(announcement))
}

async fn get_activity(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ActivityLog>>, (StatusCode, String)> {
    let activity = sqlx::query_as::<sqlx::Sqlite, ActivityLog>(
        "SELECT id, activity_type, agent_id, task_id, description, created_at FROM activity_log ORDER BY created_at DESC LIMIT 50"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(activity))
}

async fn get_task_activity(
    Path(task_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TaskActivity>>, (StatusCode, String)> {
    let activity = sqlx::query_as::<sqlx::Sqlite, TaskActivity>(
        "SELECT id, task_id, agent_id, message, timestamp FROM task_activity WHERE task_id = ? ORDER BY timestamp DESC"
    )
    .bind(task_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(activity))
}

async fn add_task_activity(
    Path(task_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TaskActivity>, (StatusCode, String)> {
    let id = uuid::Uuid::new_v4().to_string();
    let agent_id = payload["agent_id"].as_str();
    let message = payload["message"].as_str().ok_or((StatusCode::BAD_REQUEST, "message required".to_string()))?;

    sqlx::query("INSERT INTO task_activity (id, task_id, agent_id, message, timestamp) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)")
        .bind(&id)
        .bind(&task_id)
        .bind(agent_id)
        .bind(message)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let activity = sqlx::query_as::<sqlx::Sqlite, TaskActivity>(
        "SELECT id, task_id, agent_id, message, timestamp FROM task_activity WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.manager.broadcast(&format!(r#"{{"type": "task_activity_added", "task_id": "{}"}}"#, task_id));

    Ok(Json(activity))
}

async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let task_count: i32 = sqlx::query_scalar::<sqlx::Sqlite, i32>("SELECT COUNT(*) FROM tasks")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let agent_count: i32 = sqlx::query_scalar::<sqlx::Sqlite, i32>("SELECT COUNT(*) FROM agents")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let done_count: i32 = sqlx::query_scalar::<sqlx::Sqlite, i32>("SELECT COUNT(*) FROM tasks WHERE status = 'DONE'")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "total_tasks": task_count,
        "total_agents": agent_count,
        "tasks_completed": done_count,
    })))
}

async fn get_deliverables(
    Path(task_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Deliverable>>, (StatusCode, String)> {
    let deliverables = sqlx::query_as::<sqlx::Sqlite, Deliverable>(
        "SELECT id, task_id, title, description, status, created_at FROM deliverables WHERE task_id = ?"
    )
    .bind(task_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(deliverables))
}

async fn create_deliverable(
    Path(task_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Deliverable>, (StatusCode, String)> {
    let id = uuid::Uuid::new_v4().to_string();
    let title = payload["title"].as_str().ok_or((StatusCode::BAD_REQUEST, "title required".to_string()))?;
    let description = payload["description"].as_str();

    sqlx::query("INSERT INTO deliverables (id, task_id, title, description, status, created_at) VALUES (?, ?, ?, ?, 'PENDING', CURRENT_TIMESTAMP)")
        .bind(&id)
        .bind(&task_id)
        .bind(title)
        .bind(description)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let deliverable = sqlx::query_as::<sqlx::Sqlite, Deliverable>(
        "SELECT id, task_id, title, description, status, created_at FROM deliverables WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.manager.broadcast(&format!(r#"{{"type": "deliverable_added", "task_id": "{}"}}"#, task_id));

    Ok(Json(deliverable))
}

async fn route_task(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let task = get_task(Path(id.clone()), State(state.clone())).await?;
    let assignee_id = task.assignee_id.clone().ok_or((StatusCode::BAD_REQUEST, "Task has no assignee".to_string()))?;

    let output = Command::new("openclaw")
        .arg("sessions")
        .arg("spawn")
        .arg("--agent")
        .arg(&assignee_id)
        .arg("--label")
        .arg(format!("task:{}", id))
        .output()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to execute openclaw: {}", e)))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("OpenClaw error: {}", error)));
    }

    state.manager.broadcast(&format!(r#"{{"type": "task_routed", "task_id": "{}"}}"#, id));

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Task routed to agent session",
        "output": String::from_utf8_lossy(&output.stdout)
    })))
}

async fn list_recurring_tasks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<RecurringTask>>, (StatusCode, String)> {
    let tasks = sqlx::query_as::<sqlx::Sqlite, RecurringTask>(
        "SELECT id, title, description, assignee_id, schedule_type, schedule_value, schedule_time, last_run, next_run, is_active, created_at FROM recurring_tasks"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(tasks))
}

async fn create_recurring_task(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<RecurringTask>, (StatusCode, String)> {
    let id = uuid::Uuid::new_v4().to_string();
    let title = payload["title"].as_str().ok_or((StatusCode::BAD_REQUEST, "title required".to_string()))?;
    let schedule_type = payload["schedule_type"].as_str().ok_or((StatusCode::BAD_REQUEST, "schedule_type required".to_string()))?;
    let schedule_time = payload["schedule_time"].as_str().ok_or((StatusCode::BAD_REQUEST, "schedule_time required".to_string()))?;
    
    // Simplistic next_run calculation
    sqlx::query("INSERT INTO recurring_tasks (id, title, schedule_type, schedule_time, next_run, is_active, created_at) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, 1, CURRENT_TIMESTAMP)")
        .bind(&id)
        .bind(title)
        .bind(schedule_type)
        .bind(schedule_time)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let task = sqlx::query_as::<sqlx::Sqlite, RecurringTask>(
        "SELECT id, title, description, assignee_id, schedule_type, schedule_value, schedule_time, last_run, next_run, is_active, created_at FROM recurring_tasks WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(task))
}

async fn check_openclaw_status() -> impl IntoResponse {
    // Verifica se o arquivo de configuração do openclaw está acessível
    let openclaw_dir = std::env::var("OPENCLAW_STATE_DIR")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.openclaw", h)))
        .unwrap_or_else(|_| "/root/.openclaw".to_string());
    let config_path = format!("{}/openclaw.json", openclaw_dir);
    let available = tokio::fs::metadata(&config_path).await.is_ok();
    Json(serde_json::json!({ 
        "available": available,
        "status": if available { "ok" } else { "unavailable" }, 
        "config_path": config_path
    }))
}

async fn fetch_openclaw_agents() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Lê os agentes diretamente do arquivo openclaw.json montado via volume compartilhado
    let openclaw_dir = std::env::var("OPENCLAW_STATE_DIR")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.openclaw", h)))
        .unwrap_or_else(|_| "/root/.openclaw".to_string());
    let config_path = format!("{}/openclaw.json", openclaw_dir);

    tracing::info!("Reading Openclaw config from: {}", config_path);

    let content = tokio::fs::read_to_string(&config_path).await.map_err(|e| {
        tracing::error!("Failed to read openclaw.json at {}: {}", config_path, e);
        (StatusCode::SERVICE_UNAVAILABLE, format!("Cannot read openclaw config at {}: {}", config_path, e))
    })?;

    let config: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        tracing::error!("Failed to parse openclaw.json: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid openclaw.json: {}", e))
    })?;

    // Extrai a lista de agentes da configuração
    let agent_list = config.get("agents")
        .and_then(|a| a.get("list"))
        .and_then(|l| l.as_array())
        .cloned()
        .unwrap_or_default();

    // Transforma no formato esperado pelo frontend
    let agents: Vec<serde_json::Value> = agent_list.iter().map(|entry| {
        let id = entry.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
        let name = entry.get("name").and_then(|v| v.as_str()).unwrap_or(id);
        let workspace = entry.get("workspace").and_then(|v| v.as_str());
        serde_json::json!({
            "id": id,
            "name": name,
            "role": "SPC",
            "status": "IDLE",
            "workspace": workspace,
            "identity": { "name": name }
        })
    }).collect();

    Ok(Json(serde_json::json!({ "data": agents })))
}

async fn import_openclaw_agents(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // 1. Fetch from OpenClaw
    let gateway_agents = fetch_openclaw_agents().await?.0;
    
    // 2. Iterate and upsert into local DB
    let mut imported = 0;
    
    if let Some(agents_array) = gateway_agents.get("data").and_then(|v| v.as_array()) {
        for agent in agents_array {
            if let Some(id) = agent.get("id").and_then(|i| i.as_str()) {
                let name = agent.get("identity").and_then(|i| i.get("name")).and_then(|n| n.as_str()).unwrap_or(id);
                let workspace = agent.get("workspace").and_then(|w| w.as_str());
                let role = agent.get("role").and_then(|r| r.as_str()).unwrap_or("SPC");
                
                // Keep it simple: insert or update
                sqlx::query("INSERT INTO agents (id, name, role, workspace, status, created_at) VALUES (?, ?, ?, ?, 'IDLE', CURRENT_TIMESTAMP) ON CONFLICT(id) DO UPDATE SET name = excluded.name, role = excluded.role, workspace = excluded.workspace, status = 'IDLE'")
                    .bind(id)
                    .bind(name)
                    .bind(role.to_uppercase())
                    .bind(workspace)
                    .execute(&state.pool)
                    .await
                    .map_err(|e| {
                        tracing::error!("DB error importing agent {}: {}", id, e);
                        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                    })?;
                    
                imported += 1;
            }
        }
    }

    Ok(Json(serde_json::json!({ "status": "success", "imported": imported })))
}

async fn get_chat_messages() -> impl IntoResponse {
    Json(Vec::<ChatMessage>::new())
}

async fn send_chat_message() -> impl IntoResponse {
    StatusCode::OK
}

async fn send_chat_message_to_agent() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "success",
        "reply": "I am a mock response from the backend sync.",
        "replyId": uuid::Uuid::new_v4().to_string()
    }))
}

async fn get_models() -> impl IntoResponse {
    Json(serde_json::json!(["gpt-4", "gpt-3.5-turbo", "claude-3-opus"]))
}

async fn generate_agent_config() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "Generated Agent",
        "role": "SPC",
        "description": "Auto-generated from description"
    }))
}

async fn get_agent_files() -> impl IntoResponse {
    Json(serde_json::json!([]))
}

async fn update_agent_files() -> impl IntoResponse {
    StatusCode::OK
}

async fn review_task() -> impl IntoResponse {
    StatusCode::OK
}

async fn complete_deliverable() -> impl IntoResponse {
    StatusCode::OK
}

async fn trigger_recurring_task() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_recurring_task_runs() -> impl IntoResponse {
    Json(serde_json::json!([]))
}

async fn get_gateway_status(
    State(state): State<Arc<AppState>>,
) -> Json<GatewayStatus> {
    let status = state.gateway_status.read().await;
    Json(status.clone())
}

async fn restart_gateway(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    tracing::info!("Restarting gateway...");
    let mut status = state.gateway_status.write().await;
    status.restart_count += 1;
    status.health_status = "unknown".to_string();
    
    // In a real scenario, we would trigger a process restart here.
    // For now, we'll just mock the success signal.
    Json(serde_json::json!({
        "success": true,
        "message": "Gateway restart initiated"
    }))
}

async fn get_stuck_task_status(
    State(state): State<Arc<AppState>>,
) -> Json<StuckTaskStatus> {
    let status = state.stuck_task_status.read().await;
    Json(status.clone())
}

async fn run_stuck_task_check(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    match perform_stuck_task_check(&state.pool).await {
        Ok(count) => {
            let mut status = state.stuck_task_status.write().await;
            status.currently_tracked_tasks = count as u32;
            status.last_run = Utc::now();
            Json(serde_json::json!({ "success": true, "stuck_count": count }))
        },
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() }))
    }
}

// Monitoring Helper Functions

async fn check_gateway_connectivity() -> bool {
    let host = std::env::var("GATEWAY_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{}:18789", host);
    tracing::debug!("Checking gateway connectivity to {}", addr);
    tokio::net::TcpStream::connect(addr).await.is_ok()
}

async fn perform_stuck_task_check(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    // A task is "stuck" if it hasn't been updated for more than 2 hours (default)
    // For demonstration, we'll just count tasks in INBOX or ASSIGNED that haven't been updated recently
    let count = sqlx::query_scalar::<sqlx::Sqlite, i64>(
        "SELECT COUNT(*) FROM tasks WHERE status IN ('INBOX', 'ASSIGNED') AND updated_at < datetime('now', '-2 hours')"
    )
    .fetch_one(pool)
    .await?;
    
    Ok(count)
}
