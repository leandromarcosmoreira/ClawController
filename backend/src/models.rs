use serde::{Deserialize, Serialize};
use sqlx::{Type, FromRow};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use validator::{Validate, ValidationError as ValidatorError};

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
    Medium,
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

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum SecurityLevel {
    Public,
    Internal,
    Confidential,
    Restricted,
    Secret,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
    XHigh,
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
    #[validate(custom(function = "validate_agent_id"))]
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
    #[validate(custom(function = "validate_token_format"))]
    pub token: Option<String>,
    pub primary_model: Option<String>,
    pub fallback_model: Option<String>,
    pub current_model: Option<String>,
    #[validate(range(min = 0, max = 100))]
    pub model_failure_count: i32,
    pub access_level: AccessLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub created_ip: Option<String>,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<String>,
    pub last_active_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub version: i32,
    pub is_active: bool,
    pub is_deleted: bool,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub profile_picture: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub security_level: SecurityLevel,
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
    #[validate(custom(function = "validate_task_id"))]
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
    pub created_ip: Option<String>,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<String>,
    pub is_deleted: bool,
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
    pub created_by: Option<String>,
    pub created_ip: Option<String>,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<String>,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub profile_picture: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct PerformanceMetric {
    pub id: String,
    pub metric_name: String,
    pub metric_type: String, // 'counter', 'gauge', 'histogram', 'timer'
    pub value: f64,
    pub avg_response_time: Option<f64>,
    pub request_count: Option<i32>,
    pub labels: Option<String>, // JSON object
    pub timestamp: DateTime<Utc>,
    pub source: String, // 'agent', 'system', 'api', etc.
    pub entity_id: Option<String>,
    pub entity_type: Option<String>,
    pub unit: Option<String>,
    pub threshold_warning: Option<f64>,
    pub threshold_critical: Option<f64>,
}

// OpenClaw Integration Models
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawAgentConfig {
    pub id: String,
    pub name: Option<String>,
    pub workspace: Option<String>,
    pub agent_dir: Option<String>,
    pub model: Option<AgentModelConfig>,
    pub image_model: Option<AgentModelConfig>,
    pub skills: Option<Vec<String>>,
    pub memory_search: Option<MemorySearchConfig>,
    pub human_delay: Option<HumanDelayConfig>,
    pub heartbeat: Option<HeartbeatConfig>,
    pub identity: Option<IdentityConfig>,
    pub group_chat: Option<GroupChatConfig>,
    pub subagents: Option<SubagentsConfig>,
    pub sandbox: Option<SandboxConfig>,
    pub params: Option<serde_json::Value>,
    pub tools: Option<ToolsConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentModelConfig {
    pub primary: Option<String>,
    pub fallbacks: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemorySearchConfig {
    pub enabled: Option<bool>,
    pub max_results: Option<i32>,
    pub threshold: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HumanDelayConfig {
    pub enabled: Option<bool>,
    pub min_seconds: Option<f64>,
    pub max_seconds: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeartbeatConfig {
    pub enabled: Option<bool>,
    pub every: Option<String>,
    pub active_hours: Option<ActiveHoursConfig>,
    pub model: Option<String>,
    pub session: Option<String>,
    pub target: Option<String>,
    pub prompt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveHoursConfig {
    pub start: Option<String>,
    pub end: Option<String>,
    pub timezone: Option<String>,
    pub monday: Option<DaySchedule>,
    pub tuesday: Option<DaySchedule>,
    pub wednesday: Option<DaySchedule>,
    pub thursday: Option<DaySchedule>,
    pub friday: Option<DaySchedule>,
    pub saturday: Option<DaySchedule>,
    pub sunday: Option<DaySchedule>,
}

#[derive(Debug, Serialize, Deserialize, Clone, thiserror::Error)]
pub enum ClawValidationError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DaySchedule {
    pub enabled: bool,
    pub start_time: String,
    pub end_time: String,
    pub breaks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StrategyType {
    Speed,
    Quality,
    Cost,
    Balanced,
    Hybrid,
    Negotiation,
    Arbitration,
    Voting,
    Escalation,
    Consensus,
    Random,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub security_level: SecurityLevel,
    pub access_level: AccessLevel,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub email_verified: bool,
    pub language: Option<String>,
    pub profile_picture: Option<String>,
    pub timezone: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
    pub two_factor_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalClawValidationError {
    pub message: String,
    pub field: Option<String>,
}

impl LocalClawValidationError {
    pub fn new(message: &str) -> Self {
        Self { message: message.to_string(), field: None }
    }
    pub fn with_field(message: &str, field: &str) -> Self {
        Self { message: message.to_string(), field: Some(field.to_string()) }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdentityConfig {
    pub name: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupChatConfig {
    pub enabled: Option<bool>,
    pub mention_handling: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubagentsConfig {
    pub allow_agents: Option<Vec<String>>,
    pub model: Option<AgentModelConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SandboxConfig {
    pub mode: Option<String>,
    pub docker: Option<DockerSandboxConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerSandboxConfig {
    pub image: Option<String>,
    pub memory_mb: Option<i32>,
    pub cpu_cores: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolsConfig {
    pub exec: Option<ExecToolsConfig>,
    pub file_ops: Option<FileOpsConfig>,
    pub web: Option<WebToolsConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecToolsConfig {
    pub enabled: Option<bool>,
    pub host: Option<String>,
    pub safe_bins: Option<Vec<String>>,
    pub trusted_dirs: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileOpsConfig {
    pub enabled: Option<bool>,
    pub read_paths: Option<Vec<String>>,
    pub write_paths: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebToolsConfig {
    pub enabled: Option<bool>,
    pub allow_domains: Option<Vec<String>>,
    pub block_domains: Option<Vec<String>>,
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
pub struct GatewayConfig {
    pub check_interval_seconds: u64,
    pub health_check_timeout: u64,
    pub max_restart_attempts: u32,
    pub notification_cooldown_minutes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StuckTaskStatus {
    pub total_notifications_sent: u32,
    pub currently_tracked_tasks: u32,
    pub last_run: DateTime<Utc>,
    pub config: MonitoringConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringConfig {
    pub normal_priority_limit_minutes: u64,
    pub urgent_priority_limit_minutes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LearningMetrics {
    pub total_feedback_processed: u64,
    pub patterns_recognized: u64,
    pub adaptations_performed: u64,
    pub average_improvement: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdvancedFeaturesStatus {
    pub active_teams: usize,
    pub collaboration_metrics: CollaborationMetrics,
    pub adaptive_agents: usize,
    pub learning_metrics: LearningMetrics,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CollaborationMetrics {
    pub total_teams: usize,
    pub active_collaborations: usize,
    pub messages_exchanged: u64,
    pub tasks_delegated: u64,
    pub conflicts_resolved: u64,
    pub average_response_time: std::time::Duration,
    pub collaboration_efficiency: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub sender_id: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub role: String,
    pub primary_model: String,
    pub thinking_default: Option<i32>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<i32>,
    pub security_level: Option<i32>,
    pub skills: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub complexity_score: Option<i32>,
    pub dependencies: Option<Vec<String>>,
    pub deliverables: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<String>,
    pub security_level: Option<SecurityLevel>,
    pub access_level: Option<AccessLevel>,
    pub permissions: Option<Vec<String>>,
}

impl CreateUserRequest {
    pub fn access_level_or(&self, default: AccessLevel) -> AccessLevel {
        self.access_level.unwrap_or(default)
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct RecurringTask {
    pub id: String,
    pub title: String,
    pub cron_expression: String,
    pub last_run: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateTeamRequest {
    pub name: String,
    pub member_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Announcement {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ActivityLog {
    pub id: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub activity_type: String,
    pub description: String,
    pub metadata: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TaskActivity {
    pub id: String,
    pub task_id: String,
    pub agent_id: String,
    pub activity_type: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Deliverable {
    pub id: String,
    pub task_id: String,
    pub name: String,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TaskTag {
    pub id: String,
    pub name: String,
    pub color: String,
}

// Validation functions
fn validate_agent_id(id: &str) -> Result<(), ClawValidationError> {
    if !id.starts_with("agent_") {
        return Err(ClawValidationError::InvalidFormat("Agent ID must start with 'agent_'".to_string()));
    }
    if id.len() < 10 || id.len() > 100 {
        return Err(ClawValidationError::InvalidFormat("Agent ID must be between 10 and 100 characters".to_string()));
    }
    Ok(())
}

fn validate_task_id(id: &str) -> Result<(), ClawValidationError> {
    if !id.starts_with("task_") {
        return Err(ClawValidationError::InvalidFormat("Task ID must start with 'task_'".to_string()));
    }
    if id.len() < 10 || id.len() > 100 {
        return Err(ClawValidationError::InvalidFormat("Task ID must be between 10 and 100 characters".to_string()));
    }
    Ok(())
}

fn validate_token_format(token: &str) -> Result<(), ClawValidationError> {
    if token.len() < 32 {
        return Err(ClawValidationError::InvalidFormat("Token must be at least 32 characters".to_string()));
    }
    if !token.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(ClawValidationError::InvalidFormat("Token contains invalid characters".to_string()));
    }
    Ok(())
}
