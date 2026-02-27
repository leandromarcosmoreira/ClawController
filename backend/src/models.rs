use serde::{Deserialize, Serialize};
use sqlx::{Type, FromRow};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Inbox,
    Assigned,
    InProgress,
    Review,
    Done,
    Blocked,
    Cancelled,
    Archived,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
    Critical,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentRole {
    Lead,
    Int,
    Spc,
    Admin,
    Auditor,
    Observer,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentStatus {
    Working,
    Idle,
    Standby,
    Offline,
    Maintenance,
    Suspended,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SecurityLevel {
    Public,
    Internal,
    Confidential,
    Restricted,
    Secret,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccessLevel {
    ReadOnly,
    ReadWrite,
    Admin,
    SuperAdmin,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SandboxMode {
    Off,
    On,
    Docker,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThinkingLevel {
    Off,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerboseLevel {
    Off,
    On,
    Full,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Validate)]
pub struct Agent {
    #[validate(length(min = 1, max = 255))]
    #[validate(custom = "validate_agent_id")]
    pub id: String,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub role: AgentRole,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub status: AgentStatus,
    pub workspace: Option<String>,
    pub agent_dir: Option<String>,
    #[validate(custom = "validate_token_format")]
    pub token: Option<String>,
    pub primary_model: Option<String>,
    pub fallback_model: Option<String>,
    pub current_model: Option<String>,
    #[validate(range(min = 0, max = 100))]
    pub model_failure_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub last_active_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub is_active: bool,
    pub security_level: SecurityLevel,
    pub access_level: AccessLevel,
    // OpenClaw Advanced Configuration
    pub image_model: Option<String>,
    pub sandbox_mode: Option<SandboxMode>,
    pub thinking_default: Option<ThinkingLevel>,
    pub verbose_default: Option<VerboseLevel>,
    #[validate(range(min = 1, max = 10))]
    pub max_concurrent: Option<i32>,
    #[validate(range(min = 30, max = 3600))]
    pub timeout_seconds: Option<i32>,
    #[validate(range(min = 1000, max = 128000))]
    pub context_tokens: Option<i32>,
    pub skills: Option<String>, // JSON array
    pub tools_config: Option<String>, // JSON object
    pub memory_search_config: Option<String>, // JSON object
    pub heartbeat_enabled: Option<bool>,
    pub subagents_enabled: Option<bool>,
    pub human_delay_enabled: Option<bool>,
    pub block_streaming_enabled: Option<bool>,
    pub context_pruning_enabled: Option<bool>,
    pub openclaw_config_hash: Option<String>, // For sync tracking
    // Enhanced fields
    pub tags: Option<String>, // JSON array
    pub metadata: Option<String>, // JSON object
    pub performance_metrics: Option<String>, // JSON object
    pub health_score: Option<f64>,
    pub last_health_check: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Validate)]
pub struct Task {
    #[validate(length(min = 1, max = 255))]
    #[validate(custom = "validate_task_id")]
    pub id: String,
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    #[validate(length(max = 5000))]
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub tags: Option<String>, // JSON array
    pub assignee_id: Option<String>,
    pub reviewer: Option<String>,
    pub reviewer_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub complexity_score: Option<i32>, // 1-10
    pub risk_level: Option<String>,
    pub dependencies: Option<String>, // JSON array
    pub deliverables: Option<String>, // JSON array
    pub metadata: Option<String>, // JSON object
    pub version: i32,
    pub is_template: bool,
    pub template_usage_count: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Validate)]
pub struct Comment {
    pub id: String,
    pub task_id: String,
    pub agent_id: String,
    #[validate(length(min = 1, max = 10000))]
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_edited: bool,
    pub parent_id: Option<String>, // For threaded comments
    pub mentions: Option<String>, // JSON array
    pub attachments: Option<String>, // JSON array
    pub reaction_count: i32,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<String>,
}

// Enhanced audit and security structures

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: String,
    pub entity_type: String, // 'agent', 'task', 'user', etc.
    pub entity_id: String,
    pub action: String, // 'create', 'update', 'delete', 'access', etc.
    pub old_values: Option<String>, // JSON object
    pub new_values: Option<String>, // JSON object
    pub user_id: Option<String>,
    pub user_role: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
    pub risk_score: Option<i32>, // 0-100
    pub compliance_flags: Option<String>, // JSON array
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SecurityEvent {
    pub id: String,
    pub event_type: String, // 'login_failure', 'unauthorized_access', etc.
    pub severity: String, // 'low', 'medium', 'high', 'critical'
    pub description: String,
    pub source_ip: Option<String>,
    pub target_resource: Option<String>,
    pub user_id: Option<String>,
    pub details: Option<String>, // JSON object
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub security_level: SecurityLevel,
    pub access_level: AccessLevel,
    pub permissions: Option<String>, // JSON array
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_password_change: DateTime<Utc>,
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
    pub is_active: bool,
    pub device_fingerprint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub description: String,
    pub resource: String,
    pub action: String,
    pub conditions: Option<String>, // JSON object
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: Option<String>, // JSON array
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SystemConfiguration {
    pub id: String,
    pub key: String,
    pub value: String,
    pub data_type: String, // 'string', 'number', 'boolean', 'json'
    pub description: String,
    pub category: String,
    pub is_sensitive: bool,
    pub requires_restart: bool,
    pub validation_rules: Option<String>, // JSON object
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
    pub version: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BackupRecord {
    pub id: String,
    pub backup_type: String, // 'full', 'incremental', 'differential'
    pub location: String,
    pub size_bytes: i64,
    pub status: String, // 'in_progress', 'completed', 'failed'
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub checksum: Option<String>,
    pub retention_days: i32,
    pub created_by: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PerformanceMetric {
    pub id: String,
    pub metric_name: String,
    pub metric_type: String, // 'counter', 'gauge', 'histogram', 'timer'
    pub value: f64,
    pub labels: Option<String>, // JSON object
    pub timestamp: DateTime<Utc>,
    pub source: String, // 'agent', 'system', 'api', etc.
    pub entity_id: Option<String>,
    pub entity_type: Option<String>,
    pub unit: Option<String>,
    pub threshold_warning: Option<f64>,
    pub threshold_critical: Option<f64>,
}

// Validation functions
fn validate_agent_id(id: &str) -> Result<(), ValidationError> {
    if !id.starts_with("agent_") {
        return Err(ValidationError::new("Agent ID must start with 'agent_'"));
    }
    if id.len() < 10 || id.len() > 100 {
        return Err(ValidationError::new("Agent ID must be between 10 and 100 characters"));
    }
    Ok(())
}

fn validate_task_id(id: &str) -> Result<(), ValidationError> {
    if !id.starts_with("task_") {
        return Err(ValidationError::new("Task ID must start with 'task_'"));
    }
    if id.len() < 10 || id.len() > 100 {
        return Err(ValidationError::new("Task ID must be between 10 and 100 characters"));
    }
    Ok(())
}

fn validate_token_format(token: &str) -> Result<(), ValidationError> {
    if let Some(token) = token {
        if token.len() < 32 {
            return Err(ValidationError::new("Token must be at least 32 characters"));
        }
        // Basic token format validation
        if !token.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ValidationError::new("Token contains invalid characters"));
        }
    }
    Ok(())
}
