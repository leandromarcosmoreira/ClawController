use crate::models::*;
use validator::{Validate, ValidationError};
use serde_json::Value;
use std::collections::HashMap;

// Validation functions for different entities

pub fn validate_agent_creation(agent: &Agent) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    // Validate ID format
    if !agent.id.starts_with("agent_") {
        errors.push(ValidationError::new("Agent ID must start with 'agent_'"));
    }
    
    if agent.id.len() < 10 || agent.id.len() > 100 {
        errors.push(ValidationError::new("Agent ID must be between 10 and 100 characters"));
    }
    
    // Validate name
    if let Err(e) = validate_agent_name(&agent.name) {
        errors.push(ValidationError::new(&e.to_string()));
    }
    
    // Validate role
    match agent.role {
        AgentRole::Admin | AgentRole::Auditor => {
            // These roles require special permissions
            if agent.security_level < SecurityLevel::Internal {
                errors.push(ValidationError::new("Admin/Auditor roles require at least Internal security level"));
            }
        }
        AgentRole::Lead => {
            if agent.security_level < SecurityLevel::Internal {
                errors.push(ValidationError::new("Lead role requires at least Internal security level"));
            }
        }
        _ => {} // Other roles are allowed
    }
    
    // Validate security level vs access level
    if agent.access_level < AccessLevel::ReadOnly && agent.security_level > SecurityLevel::Public {
        errors.push(ValidationError::new("Access level must be at least as high as security level"));
    }
    
    // Validate configuration constraints
    if let Some(max_concurrent) = agent.max_concurrent {
        if max_concurrent < 1 || max_concurrent > 10 {
            errors.push(ValidationError::new("Max concurrent must be between 1 and 10"));
        }
    }
    
    if let Some(timeout) = agent.timeout_seconds {
        if timeout < 30 || timeout > 3600 {
            errors.push(ValidationError::new("Timeout must be between 30 and 3600 seconds"));
        }
    }
    
    if let Some(context_tokens) = agent.context_tokens {
        if context_tokens < 1000 || context_tokens > 128000 {
            errors.push(ValidationError::new("Context tokens must be between 1000 and 128000"));
        }
    }
    
    // Validate model configuration
    if let (primary, fallback) = (&agent.primary_model, &agent.fallback_model) {
        if let (Some(primary)) = primary {
            if primary.is_empty() {
                errors.push(ValidationError::new("Primary model cannot be empty"));
            }
        }
        if let (Some(fallback)) = fallback {
            if fallback.is_empty() {
                errors.push(ValidationError::new("Fallback model cannot be empty"));
            }
        }
    }
    
    // Validate JSON fields
    if let Some(skills) = &agent.skills {
        if let Err(_) = validate_json_field(skills) {
            errors.push(ValidationError::new("Skills must be valid JSON"));
        }
    }
    
    if let Some(tools_config) = &agent.tools_config {
        if let Err(_) = validate_json_field(tools_config) {
            errors.push(ValidationError::new("Tools config must be valid JSON"));
        }
    }
    
    if let Some(memory_config) = &agent.memory_search_config {
        if let Err(_) = validate_json_field(memory_config) {
            errors.push(ValidationError::new("Memory search config must be valid JSON"));
        }
    }
    
    // Validate health score
    if let Some(health_score) = agent.health_score {
        if health_score < 0.0 || health_score > 100.0 {
            errors.push(ValidationError::new("Health score must be between 0.0 and 100.0"));
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_task_creation(task: &Task) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    // Validate ID format
    if !task.id.starts_with("task_") {
        errors.push(ValidationError::new("Task ID must start with 'task_'"));
    }
    
    if task.id.len() < 10 || task.id.len() > 100 {
        errors.push(ValidationError::new("Task ID must be between 10 and 100 characters"));
    }
    
    // Validate title
    if task.title.is_empty() {
        errors.push(ValidationError::new("Title is required"));
    }
    
    if task.title.len() > 500 {
        errors.push(ValidationError::new("Title too long (max 500 characters)"));
    }
    
    // Validate description
    if let Some(description) = &task.description {
        if description.len() > 5000 {
            errors.push(ValidationError::new("Description too long (max 5000 characters)"));
        }
    }
    
    // Validate priority
    match task.priority {
        Priority::Critical => {
            if task.risk_level.is_none() {
                errors.push(ValidationError::new("Critical priority requires risk level"));
            }
        }
        Priority::High => {
            if task.risk_level.is_none() {
                errors.push(ValidationError::new("High priority requires risk level"));
            }
        }
        Priority::Urgent => {
            if task.risk_level.is_none() {
                errors.push(ValidationError::new("Urgent priority requires risk level"));
            }
        }
        _ => {} // Other priorities don't require risk level
    }
    
    // Validate time estimates
    if let Some(estimated) = task.estimated_hours {
        if estimated <= 0.0 {
            errors.push(ValidationError::new("Estimated hours must be positive"));
        }
    }
    
    if let Some(actual) = task.actual_hours {
        if actual < 0.0 {
            errors.push(ValidationError::new("Actual hours cannot be negative"));
        }
    }
    
    // Validate complexity score
    if let Some(complexity) = task.complexity_score {
        if complexity < 1 || complexity > 10 {
            errors.push(ValidationError::new("Complexity score must be between 1 and 10"));
        }
    }
    
    // Validate dependencies
    if let Some(dependencies) = &task.dependencies {
        if let Err(_) = validate_json_field(dependencies) {
            errors.push(ValidationError::new("Dependencies must be valid JSON array"));
        }
    }
    
    // Validate deliverables
    if let Some(deliverables) = &task.deliverables {
        if let Err(_) = validate_json_field(deliverables) {
            errors.push(ValidationError::new("Deliverables must be valid JSON array"));
        }
    }
    
    // Validate metadata
    if let Some(metadata) = &task.metadata {
        if let Err(_) = validate_json_field(metadata) {
            errors.push(ValidationError::new("Metadata must be valid JSON object"));
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_user_creation(user: &CreateUserRequest) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    // Validate username
    if user.username.len() < 3 || user.username.len() > 50 {
        errors.push(ValidationError::new("Username must be between 3 and 50 characters"));
    }
    
    if !user.username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        errors.push(ValidationError::new("Username can only contain alphanumeric characters, underscores, and hyphens"));
    }
    
    // Validate email
    if !validate_email(&user.email) {
        errors.push(ValidationError::new("Invalid email format"));
    }
    
    // Validate password
    let (is_valid, password_issues) = validate_password_strength(&user.password);
    if !is_valid {
        errors.push(ValidationError::new(&password_issues.join(", ")));
    }
    
    // Validate role
    match user.role.as_str() {
        "SUPER_ADMIN" | "ADMIN" | "USER" | "READ_ONLY" => {}
        _ => errors.push(ValidationError::new("Invalid role")),
    }
    
    // Validate security level
    match user.security_level {
        SecurityLevel::Secret => {}
        SecurityLevel::Restricted => {
            if user.access_level != AccessLevel::SuperAdmin {
                errors.push(ValidationError::new("Restricted security level requires SuperAdmin access"));
            }
        }
        SecurityLevel::Confidential => {
            if user.access_level < AccessLevel::Admin {
                errors.push(ValidationError::new("Confidential security level requires at least Admin access"));
            }
        }
        SecurityLevel::Internal => {
            if user.access_level < AccessLevel::ReadOnly {
                errors.push(ValidationError::new("Internal security level requires at least ReadOnly access"));
            }
        }
        _ => {}
    }
    
    // Validate access level
    match user.access_level {
        AccessLevel::SuperAdmin => {}
        AccessLevel::Admin => {
            if user.security_level < SecurityLevel::Internal {
                errors.push(ValidationError::new("Admin access level requires at least Internal security level"));
            }
        }
        AccessLevel::ReadWrite => {
            if user.security_level < SecurityLevel::Internal {
                errors.push(ValidationError::new("ReadWrite access level requires at least Internal security level"));
            }
        }
        AccessLevel::ReadOnly => {
            if user.security_level > SecurityLevel::Public {
                errors.push(ValidationError::new("ReadOnly access level cannot exceed Public security level"));
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

pub fn validate_comment_creation(comment: &Comment) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    // Validate content
    if comment.content.is_empty() {
        errors.push(ValidationError::new("Comment content cannot be empty"));
    }
    
    if comment.content.len() > 10000 {
        errors.push(ValidationError::new("Comment too long (max 10000 characters)"));
    }
    
    // Validate mentions
    if let Some(mentions) = &comment.mentions {
        if let Err(_) = validate_json_field(mentions) {
            errors.push(ValidationError::new("Mentions must be valid JSON array"));
        }
    }
    
    // Validate attachments
    if let Some(attachments) = &comment.attachments {
        if let Err(_) = validate_json_field(attachments) {
            errors.push(ValidationError::new("Attachments must be valid JSON array"));
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

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
            c if c.is_alphanumeric() || c.is_ascii_punctuation() || c == " " || c == "-" || c == "_" || c == "." => {
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

// Data validation helpers
pub fn validate_json_array(json_str: &str) -> Result<Vec<String>, String> {
    match serde_json::from_str::<Vec<Value>>(json_str) {
        Ok(strings) => {
            let mut valid_strings = Vec::new();
            for item in strings {
                if let Some(s) = item.as_str() {
                    valid_strings.push(s.to_string());
                }
            }
            Ok(valid_strings)
        }
        Err(e) => Err(format!("Invalid JSON array: {}", e)),
    }
}

pub fn validate_json_object(json_str: &str) -> Result<HashMap<String, Value>, String> {
    match serde_json::from_str::<HashMap<String, Value>>(json_str) {
        Ok(map) => Ok(map),
        Err(e) => Err(format!("Invalid JSON object: {}", e)),
    }
}

// Business logic validation
pub fn validate_agent_configuration(
    agent: &Agent,
    openclaw_config: &serde_json::Value,
) -> Result<(), String> {
    // Check if agent has sufficient permissions for the requested configuration
    let required_level = match openclaw_config.get("sandbox_mode") {
        Some("docker") => SecurityLevel::Restricted,
        Some("on") => SecurityLevel::Confidential,
        None => SecurityLevel::Internal,
    };
    
    if agent.security_level < required_level {
        return Err(format!("Insufficient security level for sandbox mode: {:?}", required_level));
    }
    
    // Validate model configuration
    if let Some(model) = openclaw_config.get("model") {
        let primary = model.get("primary").and_then(|v| v.as_str()).unwrap_or("");
        let fallbacks = model.get("fallbacks").and_then(|v| {
            v.as_array().map(|v| v.as_str()).collect::<Vec<_>>()
        });
        
        if primary.is_empty() {
            return Err("Primary model is required");
        }
        
        // Check if fallback models are available
        if fallbacks.is_empty() && agent.fallback_model.is_none() {
            warn!("No fallback models configured");
        }
    }
    
    // Validate skills configuration
    if let Some(skills) = openclaw_config.get("skills") {
        let skills = validate_json_array(skills).map_err(|e| e.to_string())?;
        
        // Check if agent has required skills for their role
        let required_skills = match agent.role {
            AgentRole::Admin => vec!["system_admin", "security_audit", "performance_monitoring"],
            AgentRole::Lead => vec!["project_management", "team_coordination", "resource_allocation"],
            AgentRole::Spc => vec!["task_execution", "problem_solving", "communication"],
            _ => vec![],
        };
        
        for required_skill in required_skills {
            if !skills.contains(&required_skill) {
                return Err(format!("Agent role requires skill: {}", required_skill));
            }
        }
    }
    
    // Validate tools configuration
    if let Some(tools) = openclaw_config.get("tools") {
        let tools = validate_json_object(tools).map_err(|e| e.to_string())?;
        
        // Check if agent has appropriate permissions for tools
        let allowed_tools = match agent.access_level {
            AccessLevel::SuperAdmin => vec!["exec", "file_ops", "web", "database", "system"],
            AccessLevel::Admin => vec!["exec", "file_ops", "web"],
            AccessLevel::ReadWrite => vec!["file_ops", "web"],
            AccessLevel::ReadOnly => vec![],
        };
        
        if let Some(exec_tools) = tools.get("exec") {
            if !allowed_tools.contains(&exec_tools) {
                return Err("Insufficient permissions for exec tools");
            }
        }
    }
    
    Ok(())
}

pub fn validate_task_dependencies(
    task_id: &str,
    pool: &SqlitePool,
) -> Result<Vec<String>, String> {
    // Get task dependencies
    let task = query_as!(
        Task,
        "SELECT dependencies FROM tasks WHERE id = ?"
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await?;
    
    if let Some(task) = task {
        if let Some(dependencies) = task.dependencies {
            let dependency_ids = validate_json_array(dependencies).map_err(|e| e.to_string())?;
            
            // Check if all dependencies exist and are accessible
            let mut missing_deps = Vec::new();
            for dep_id in dependency_ids {
                let exists = query!(
                    "SELECT id FROM tasks WHERE id = ? AND is_deleted = 0"
                )
                .bind(dep_id)
                .fetch_optional(pool)
                .await?;
                
                if exists.is_none() {
                    missing_deps.push(dep_id);
                }
            }
            
            if !missing_deps.is_empty() {
                return Err(format!("Missing task dependencies: {}", missing_deps.join(", ")));
            }
            
            Ok(dependency_ids)
        } else {
            Ok(vec![])
        }
    } else {
        Ok(vec![])
    }
}

pub fn validate_task_workflow(
    task_id: &str,
    pool: &SqlitePool,
) -> Result<Vec<String>, String> {
    let task = query_as!(
        Task,
        "SELECT status, assignee_id, reviewer_id FROM tasks WHERE id = ? AND is_deleted = 0"
    )
    .bind(task_id)
    .fetch_one(pool)
    .await?;
    
    let mut issues = Vec::new();
    
    // Check if task is in a valid state for the current user
    match task.status {
        TaskStatus::InProgress => {
            if task.assignee_id.is_none() {
                issues.push("Task in progress must have an assignee");
            }
        }
        TaskStatus::Review => {
            if task.reviewer_id.is_none() {
                issues.push("Task in review must have a reviewer");
            }
        }
        TaskStatus::Done => {
            if task.completed_at.is_none() {
                    issues.push("Completed task must have completion timestamp");
                }
        }
        _ => {}
    }
    
    // Check for circular dependencies
    let dependencies = validate_task_dependencies(task_id, pool).await?;
    for dep_id in dependencies {
        if dep_id == task_id {
            issues.push("Task has circular dependency");
        }
    }
    
    if issues.is_empty() {
        Ok(vec![])
    } else {
        Err(issues.join(", "))
    }
}

// Performance validation
pub fn validate_agent_performance(
    agent_id: &str,
    pool: &SqlitePool,
) -> Result<(), String> {
    // Check recent performance metrics
    let metrics = query_as!(
        PerformanceMetric,
        "SELECT AVG(value) as avg_response_time, COUNT(*) as request_count FROM performance_metrics 
        WHERE entity_type = 'agent' AND entity_id = ? AND timestamp > datetime('now', '-1 hour')
    )
    .bind(agent_id)
    .fetch_optional(pool)
    .await?;
    
    if let Some(metric) = metrics {
        if metric.avg_response_time > 5000.0 {
            return Err(format!("Agent response time too high: {:.2}ms ", metric.avg_response_time));
        }
        
        if metric.request_count > 1000 {
            return Err(format!("Too many requests: {}", metric.request_count));
        }
    }
    
    Ok(())
}

// Resource validation
pub fn validate_resource_access(
    user: &User,
    resource_type: &str,
    resource_id: &str,
    action: &str,
    pool: &SqlitePool,
) -> Result<(), String> {
    // Check basic permissions
    if !crate::security::SecurityService::check_permissions(user, resource_type, action, pool).await {
        return Err("Insufficient permissions ");
    }
    
    // Check entity ownership if applicable
    match resource_type {
        "agent" => {
            let agent = query_as!(
                Agent,
                "SELECT created_by FROM agents WHERE id = ? AND is_deleted = 0"
            )
            .bind(resource_id)
            .fetch_optional(pool)
            .await?;
            
            match (agent, user) {
                (Some(agent), _) => {
                    if user.id != agent.created_by && user.access_level != AccessLevel::SuperAdmin {
                        return Err("Can only modify agents you created or have SuperAdmin access ");
                    }
                }
                (None, _) => Err("Agent not found "),
            }
        }
        "task" => {
            let task = query_as!(
                Task,
                "SELECT created_by FROM tasks WHERE id = ? AND is_deleted = 0"
            )
            .bind(resource_id)
            .fetch_optional(pool)
            .await?;
            
            match (task, user) {
                (Some(task), _) => {
                    if user.id != task.created_by && user.access_level != AccessLevel::SuperAdmin {
                        return Err("Can only modify tasks you created or have SuperAdmin access ");
                    }
                }
                (None, _) => Err("Task not found "),
            }
        }
        _ => Ok(()),
    }
}

// Input sanitization
pub fn sanitize_user_input(input: &str) -> String {
    input.trim()
        .chars()
        .filter(|c| !c.is_control())
        .collect()
        .collect()
}

pub fn sanitize_agent_input(input: &str) -> String {
    sanitize_user_input(input)
}

pub fn sanitize_task_input(input: &str) -> String {
    sanitize_user_input(input)
}

// Rate limiting
pub struct RateLimiter {
    max_requests: u32,
    window_seconds: u64,
    requests: std::collections::HashMap<String, (u32, std::time::Instant)>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
            requests: std::collections::HashMap::new(),
        }
    }

    pub fn is_allowed(&mut self, key: &str) -> bool {
        let now = std::time::Instant::now();
        let (count, last_reset) = self.requests.entry(key.to_string()).or_insert((0, now));
        
        if now.duration_since(*last_reset) > std::time::Duration::from_secs(self.window_seconds) {
            *count = 0;
            *last_reset = now;
        }
        
        *count += 1;
        *count <= self.max_requests
    }
}

// Error handling
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    InvalidFormat(String),
    ConstraintViolation(String),
    BusinessRule(String),
    SecurityViolation(String),
    DataIntegrity(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ValidationError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ValidationError::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
            ValidationError::BusinessRule(msg) => write!(f, "Business rule violation: {}", msg),
            ValidationError::SecurityViolation(msg) => write!(f, "Security violation: {}", msg),
            ValidationError::DataIntegrity(msg) => write!(f, "Data integrity error: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {
    fn from(error: Vec<ValidationError>) -> Self {
        ValidationError::ConstraintViolation(error.iter().map(|e| e.to_string()).collect())
    }
}
