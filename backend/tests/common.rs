use axum::{Router, extract::State, response::IntoResponse};
use std::sync::Arc;
use crate::AppState;
use crate::db::SqlitePool;
use crate::models::*;
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

pub struct TestApp {
    pub app: Router<Arc<AppState>>,
    pub pool: Arc<SqlitePool>,
}

impl TestApp {
    pub async fn new() -> Self {
        let pool = create_test_pool().await;
        let state = AppState {
            pool: pool.clone(),
            manager: Arc::new(tokio::sync::broadcast::channel("global")),
            gateway_status: Arc::new(tokio::sync::RwLock::new(crate::GatewayStatus::default())),
            stuck_task_status: Arc::new(tokio::sync::RwLock::new(crate::StuckTaskStatus::default())),
        };
        
        let app = create_app_with_state(state).await;
        
        Self { app, pool }
    }
}

pub async fn create_test_pool() -> Arc<SqlitePool> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|| "sqlite:::memory:".to_string());
    
    let pool = sqlx::SqlitePool::connect(&database_url)
        .await
        .expect("Failed to create database pool");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .await
        .expect("Failed to run migrations");
    
    Arc::new(pool)
}

pub async fn create_app_with_state(state: Arc<AppState>) -> Router<Arc<AppState>> {
    crate::create_app().with_state(state)
}

pub fn create_test_agent() -> Agent {
    Agent {
        id: format!("test-agent-{}", Uuid::new_v4()),
        name: "Test Agent".to_string(),
        role: "SPC".to_string(),
        status: AgentStatus::Working,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_active: true,
        is_deleted: false,
        created_by: None,
        description: Some("Test agent description".to_string()),
        avatar: None,
        workspace: Some("test-workspace".to_string()),
        token: None,
        primary_model: Some("claude-3-sonnet".to_string()),
        fallback_model: Some("gpt-4".to_string()),
        current_model: None,
        model_failure_count: 0,
        skills: Some(serde_json::to_string(&vec!["research", "analysis", "writing"])),
        max_concurrent: Some(5),
        max_memory_mb: Some(4096),
        max_execution_time_minutes: Some(60),
        thinking_default: Some(3),
        temperature: Some(0.7),
        max_tokens: Some(4000),
        security_level: SecurityLevel::Internal,
        access_level: AccessLevel::ReadWrite,
        sandbox_mode: SandboxMode::Enabled,
    }
}

pub fn create_test_agent_data() -> Value {
    json!({
        "id": format!("test-agent-{}", Uuid::new_v4()),
        "name": "Test Agent",
        "role": "SPC",
        "description": "Test agent description",
        "workspace": "test-workspace",
        "primary_model": "claude-3-sonnet",
        "max_concurrent": 5,
        "max_memory_mb": 4096,
        "max_execution_time_minutes": 60,
        "thinking_default": 3,
        "temperature": 0.7,
        "max_tokens": 4000,
        "security_level": "Internal",
        "access_level": "ReadWrite",
        "sandbox_mode": "Enabled",
        "skills": ["research", "analysis", "writing"]
    })
}

pub fn create_test_task() -> Task {
    Task {
        id: format!("test-task-{}", Uuid::new_v4()),
        title: "Test Task".to_string(),
        description: Some("Test task description".to_string()),
        status: TaskStatus::Inbox,
        priority: Priority::Medium,
        tags: Some(serde_json::to_string(&vec!["test", "sample"])),
        assignee_id: None,
        reviewer: None,
        reviewer_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_at: None,
        estimated_hours: Some(2.0),
        actual_hours: None,
        complexity_score: Some(5),
        dependencies: Some(serde_json::to_string(&vec![])),
        deliverables: Some(serde_json::to_string(&vec![])),
        is_deleted: false,
        created_by: Some("test-user".to_string()),
    }
}

pub fn create_test_task_data() -> Value {
    json!({
        "title": "Test Task",
        "description": "Test task description",
        "priority": "Medium",
        "tags": ["test", "sample"],
        "estimated_hours": 2.0,
        "complexity_score": 5
    })
}

pub fn create_test_user() -> User {
    User {
        id: format!("user-{}", Uuid::new_v4()),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        role: "USER".to_string(),
        access_level: AccessLevel::ReadWrite,
        security_level: SecurityLevel::Internal,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        profile_picture: None,
        timezone: "UTC".to_string(),
        preferences: None,
        last_login: None,
        failed_login_attempts: 0,
        locked_until: None,
    }
}
