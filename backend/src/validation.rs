use crate::models::*;
use crate::db::SqlitePool;
use sqlx::{query, query_as};
use serde_json::Value;
use validator::Validate;
use regex::Regex;
use tracing::{info, warn, error};
use std::collections::HashMap;
use chrono::Utc;
use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use crate::AppState;



// Validation functions for agents
pub fn validate_agent_creation(agent: &CreateAgentRequest) -> Result<(), Vec<LocalClawValidationError>> {
    let mut errors = Vec::new();
    
    // Validate name
    if agent.name.is_empty() {
        errors.push(LocalClawValidationError::new("Agent name cannot be empty"));
    }
    
    if agent.name.len() > 255 {
        errors.push(LocalClawValidationError::new("Agent name too long (max 255 characters)"));
    }
    
    // Validate role
    if agent.role.is_empty() {
        errors.push(LocalClawValidationError::new("Agent role cannot be empty"));
    }
    
    // Validate primary model
    if agent.primary_model.is_empty() {
        errors.push(LocalClawValidationError::new("Primary model cannot be empty"));
    }
    
    // Validate thinking level
    if let Some(thinking) = agent.thinking_default {
        if thinking < 1 || thinking > 6 {
            errors.push(LocalClawValidationError::new("Thinking level must be between 1 and 6"));
        }
    }
    
    // Validate temperature
    if let Some(temp) = agent.temperature {
        if temp < 0.0 || temp > 2.0 {
            errors.push(LocalClawValidationError::new("Temperature must be between 0.0 and 2.0"));
        }
    }
    
    // Validate max tokens
    if let Some(max_tokens) = agent.max_tokens {
        if max_tokens < 1 || max_tokens > 128000 {
            errors.push(LocalClawValidationError::new("Max tokens must be between 1 and 128000"));
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// Validation functions for tasks
pub fn validate_task_creation(task: &CreateTaskRequest) -> Result<(), Vec<LocalClawValidationError>> {
    let mut errors = Vec::new();
    
    // Validate title
    if task.title.is_empty() {
        errors.push(LocalClawValidationError::new("Task title cannot be empty"));
    }
    
    if task.title.len() > 500 {
        errors.push(LocalClawValidationError::new("Task title too long (max 500 characters)"));
    }
    
    // Validate priority
    match task.priority {
        Priority::Low | Priority::Normal | Priority::High | Priority::Critical => {},
        _ => errors.push(LocalClawValidationError::new("Invalid priority level")),
    }
    
    // Validate estimated hours
    if let Some(hours) = task.estimated_hours {
        if hours < 0.0 {
            errors.push(LocalClawValidationError::new("Estimated hours cannot be negative"));
        }
        
        if hours > 1000.0 {
            errors.push(LocalClawValidationError::new("Estimated hours too high (max 1000)"));
        }
    }
    
    // Validate actual hours
    if let Some(actual) = task.actual_hours {
        if actual < 0.0 {
            errors.push(LocalClawValidationError::new("Actual hours cannot be negative"));
        }
    }
    
    // Validate complexity score
    if let Some(complexity) = task.complexity_score {
        if complexity < 1 || complexity > 10 {
            errors.push(LocalClawValidationError::new("Complexity score must be between 1 and 10"));
        }
    }
    
    // Validate dependencies
    if let Some(dependencies) = &task.dependencies {
        if let Err(_) = validate_json_field(dependencies) {
            errors.push(LocalClawValidationError::new("Dependencies must be valid JSON array"));
        }
    }
    
    // Validate deliverables
    if let Some(deliverables) = &task.deliverables {
        if let Err(_) = validate_json_field(deliverables) {
            errors.push(LocalClawValidationError::new("Deliverables must be valid JSON array"));
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// Validation functions for comments
pub fn validate_comment_creation(comment: &CreateCommentRequest) -> Result<(), Vec<LocalClawValidationError>> {
    let mut errors = Vec::new();
    
    // Validate content
    if comment.content.is_empty() {
        errors.push(LocalClawValidationError::new("Comment content cannot be empty"));
    }
    
    if comment.content.len() > 10000 {
        errors.push(LocalClawValidationError::new("Comment too long (max 10000 characters)"));
    }
    
    // Validate agent ID
    if comment.agent_id.is_empty() {
        errors.push(LocalClawValidationError::new("Agent ID cannot be empty"));
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// Validation functions for users
pub fn validate_user_creation(user: &CreateUserRequest) -> Result<(), Vec<LocalClawValidationError>> {
    let mut errors = Vec::new();
    
    // Validate username
    if user.username.is_empty() {
        errors.push(LocalClawValidationError::new("Username cannot be empty"));
    }
    
    if user.username.len() < 3 {
        errors.push(LocalClawValidationError::new("Username too short (min 3 characters)"));
    }
    
    if user.username.len() > 50 {
        errors.push(LocalClawValidationError::new("Username too long (max 50 characters)"));
    }
    
    // Validate email
    if !validate_email(&user.email) {
        errors.push(LocalClawValidationError::new("Invalid email format"));
    }
    
    // Validate password
    let (is_valid, issues) = validate_password_strength(&user.password);
    if !is_valid {
        for issue in issues {
            errors.push(LocalClawValidationError::new(&issue));
        }
    }
    
    // Validate role
    match user.role.as_str() {
        "SUPER_ADMIN" | "ADMIN" | "USER" | "READ_ONLY" => {},
        _ => errors.push(LocalClawValidationError::new("Invalid role")),
    }
    
    // Validate access level
    match user.access_level {
        AccessLevel::SuperAdmin | AccessLevel::Admin | AccessLevel::ReadWrite | AccessLevel::ReadOnly => {},
        _ => errors.push(LocalClawValidationError::new("Invalid access level")),
    }
    
    // Validate security level
    match user.security_level {
        SecurityLevel::Public | SecurityLevel::Internal | SecurityLevel::Confidential | 
        SecurityLevel::Restricted | SecurityLevel::Secret => {},
        _ => errors.push(LocalClawValidationError::new("Invalid security level")),
    }
    
    // Validate access level vs security level
    match user.access_level {
        AccessLevel::SuperAdmin => {}
        AccessLevel::Admin => {
            if user.security_level < SecurityLevel::Internal {
                errors.push(LocalClawValidationError::new("Admin access level requires at least Internal security level"));
            }
        }
        AccessLevel::ReadWrite => {
            if user.security_level < SecurityLevel::Internal {
                errors.push(LocalClawValidationError::new("ReadWrite access level requires at least Internal security level"));
            }
        }
        AccessLevel::ReadOnly => {
            if user.security_level > SecurityLevel::Public {
                errors.push(LocalClawValidationError::new("ReadOnly access level cannot exceed Public security level"));
            }
        }
        _ => {}
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// General validation utilities
pub fn validate_json_field(json_str: &str) -> Result<(), String> {
    match serde_json::from_str::<Value>(json_str) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Invalid JSON: {}", e)),
    }
}

pub fn sanitize_filename(filename: &str) -> String {
    let mut sanitized = String::new();
    
    for c in filename.chars() {
        match c {
            c if c.is_alphanumeric() || c.is_ascii_punctuation() || c == ' ' || c == '-' || c == '_' || c == '.' => {
                sanitized.push(c);
            }
            _ => {
                sanitized.push('_');
            }
        }
    }
    
    sanitized
}

pub fn validate_file_upload(
    size: i64,
    mime_type: &str,
    max_size: i64,
    allowed_types: &[&str],
) -> Result<(), String> {
    // Check file size
    if size > max_size {
        return Err(format!("File size {} exceeds maximum allowed size {}", size, max_size));
    }
    
    // Check MIME type
    if !allowed_types.contains(&mime_type) {
        return Err(format!("MIME type {} not allowed", mime_type));
    }
    
    Ok(())
}

// Resource validation
pub async fn validate_resource_access(
    user: &User,
    resource_type: &str,
    resource_id: &str,
    action: &str,
    pool: &SqlitePool,
) -> Result<(), String> {
    // Check basic permissions
    if !crate::security::SecurityService::check_permissions(user, resource_type, action, pool).await {
        return Err("Insufficient permissions".to_string());
    }
    
    // Check entity ownership if applicable
    match resource_type {
        "agent" => {
            let agent = sqlx::query_as::<sqlx::Sqlite, 
                Agent,
                "SELECT * FROM agents WHERE id = ? AND is_deleted = 0",
                resource_id
            )
            .fetch_optional(pool)
            .await?;
            
            match (agent, user) {
                (Some(agent), _) => {
                    if user.id != agent.created_by && user.access_level != AccessLevel::SuperAdmin {
                        return Err("Can only modify agents you created or have SuperAdmin access".to_string());
                    }
                }
                (None, _) => return Err("Agent not found".to_string()),
            }
        }
        "task" => {
            let task = sqlx::query_as::<sqlx::Sqlite, 
                Task,
                "SELECT * FROM tasks WHERE id = ? AND is_deleted = 0",
                resource_id
            )
            .fetch_optional(pool)
            .await?;
            
            match (task, user) {
                (Some(task), _) => {
                    if user.id != task.created_by && user.access_level != AccessLevel::SuperAdmin {
                        return Err("Can only modify tasks you created or have SuperAdmin access".to_string());
                    }
                }
                (None, _) => return Err("Task not found".to_string()),
            }
        }
        _ => Ok(()),
    }

    Ok(())
}

pub async fn validate_multiple_resource_access(
    user: &User,
    resources: Vec<(&str, &str)>,
    action: &str,
    pool: &SqlitePool,
) -> Result<(), String> {
    for (res_type, res_id) in resources {
        validate_resource_access(user, res_type, res_id, action, pool).await?;
    }
    Ok(())
}

// Input sanitization
pub fn sanitize_user_input(input: &str) -> String {
    input.trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_ascii_whitespace() || c.is_ascii_punctuation())
        .collect()
}

pub async fn validate_agent_performance(agent_id: &str, pool: &SqlitePool) -> Result<(), String> {
    let metrics = sqlx::query_as::<sqlx::Sqlite, 
        PerformanceMetric,
        "SELECT * FROM performance_metrics 
        WHERE entity_type = 'agent' AND entity_id = ? AND timestamp > datetime('now', '-1 hour')
    ",
        agent_id
    )
    .fetch_optional(pool)
    .await?;
    
    if let Some(metric) = metrics {
        if metric.avg_response_time > 5000.0 {
            return Err(format!("Agent response time too high: {:.2}ms", metric.avg_response_time));
        }
        
        if metric.request_count > 1000 {
            return Err(format!("Too many requests: {}", metric.request_count));
        }
    }
    
    Ok(())
}

// Helper functions
fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

fn validate_password_strength(password: &str) -> (bool, Vec<String>) {
    let mut issues = Vec::new();
    
    if password.len() < 8 {
        issues.push("Password must be at least 8 characters long");
    }
    
    if !password.chars().any(|c| c.is_uppercase()) {
        issues.push("Password must contain at least one uppercase letter");
    }
    
    if !password.chars().any(|c| c.is_lowercase()) {
        issues.push("Password must contain at least one lowercase letter");
    }
    
    if !password.chars().any(|c| c.is_numeric()) {
        issues.push("Password must contain at least one number");
    }
    
    if !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
        issues.push("Password must contain at least one special character");
    }
    
    let common_passwords = vec![
        "password", "123456", "qwerty", "abc123", "password123", "admin", "letmein",
        "welcome", "monkey", "dragon", "password1", "123456789",
    ];
    
    if common_passwords.contains(&password.to_lowercase()) {
        issues.push("Password is too common");
    }
    
    (issues.is_empty(), issues)
}

// Error handling
#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum LocalLocalClawValidationError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    #[error("Business rule violation: {0}")]
    BusinessRule(String),
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    #[error("Data integrity error: {0}")]
    DataIntegrity(String),
}

impl From<Vec<LocalLocalClawValidationError>> for LocalLocalClawValidationError {
    fn from(error: Vec<LocalLocalClawValidationError>) -> Self {
        LocalLocalClawValidationError::InvalidFormat(format!("{:?}", error))
    }
}

// Axum Handlers
pub async fn validate_agent_creation_handler(
    State(_state): State<AppState>,
    Json(payload): Json<CreateAgentRequest>,
) -> impl IntoResponse {
    match validate_agent_creation(&payload) {
        Ok(_) => (StatusCode::OK, Json("Valid")),
        Err(e) => (StatusCode::BAD_REQUEST, Json(e)),
    }
}

pub async fn validate_task_creation_handler(
    State(_state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    match validate_task_creation(&payload) {
        Ok(_) => (StatusCode::OK, Json("Valid")),
        Err(e) => (StatusCode::BAD_REQUEST, Json(e)),
    }
}

pub async fn validate_comment_creation_handler(
    State(_state): State<AppState>,
    Json(payload): Json<CreateCommentRequest>,
) -> impl IntoResponse {
    match validate_comment_creation(&payload) {
        Ok(_) => (StatusCode::OK, Json("Valid")),
        Err(e) => (StatusCode::BAD_REQUEST, Json(e)),
    }
}

pub async fn validate_configuration_friendly_handler(
    State(_state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented")
}
