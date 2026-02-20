use serde::{Deserialize, Serialize};
use sqlx::{Type, FromRow};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Inbox,
    Assigned,
    InProgress,
    Review,
    Done,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Priority {
    Normal,
    Urgent,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentRole {
    Lead,
    Int,
    Spc,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentStatus {
    Working,
    Idle,
    Standby,
    Offline,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub status: AgentStatus,
    pub workspace: Option<String>,
    pub token: Option<String>,
    pub primary_model: Option<String>,
    pub fallback_model: Option<String>,
    pub current_model: Option<String>,
    pub model_failure_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub tags: Option<String>,
    pub assignee_id: Option<String>,
    pub reviewer: Option<String>,
    pub reviewer_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: String,
    pub task_id: String,
    pub agent_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Announcement {
    pub id: String,
    pub title: Option<String>,
    pub message: String,
    pub priority: Priority,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ActivityLog {
    pub id: String,
    pub activity_type: String,
    pub agent_id: Option<String>,
    pub task_id: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TaskActivity {
    pub id: String,
    pub task_id: String,
    pub agent_id: Option<String>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RecurringTask {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub assignee_id: Option<String>,
    pub schedule_type: String,
    pub schedule_value: Option<String>,
    pub schedule_time: String,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Deliverable {
    pub id: String,
    pub task_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub agent_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub agent: Option<Agent>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GatewayConfig {
    pub check_interval_seconds: u64,
    pub health_check_timeout: u64,
    pub max_restart_attempts: u32,
    pub notification_cooldown_minutes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GatewayStatus {
    pub health_status: String,
    pub uptime_seconds: u64,
    pub last_check_time: DateTime<Utc>,
    pub restart_count: u32,
    pub config: GatewayConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringConfig {
    pub normal_priority_limit_minutes: u64,
    pub urgent_priority_limit_minutes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StuckTaskStatus {
    pub total_notifications_sent: u32,
    pub currently_tracked_tasks: u32,
    pub last_run: DateTime<Utc>,
    pub config: MonitoringConfig,
}
