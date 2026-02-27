use crate::models::*;
use crate::db::SqlitePool;
use sqlx::{query, query_as};
use chrono::Utc;
use serde_json::Value;
use tracing::{info, warn, error};
use std::collections::HashMap;

pub struct AuditService;

impl AuditService {
    pub async fn log_entity_event(
        pool: &SqlitePool,
        entity_type: &str,
        entity_id: &str,
        action: &str,
        old_values: Option<&str>,
        new_values: Option<&str>,
        user_id: Option<&str>,
        user_role: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        session_id: Option<&str>,
        metadata: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let risk_score = calculate_risk_score(action, entity_type, user_role);
        let severity = determine_severity(risk_score);
        
        let metadata_json = metadata.map(|m| m.to_string()).unwrap_or_else("{}".to_string());
        
        query!(
            r#"
            INSERT INTO audit_log (
                id, entity_type, entity_id, action, old_values, new_values, user_id, user_role, 
                ip_address, user_agent, session_id, timestamp, success, error_message, 
                risk_score, compliance_flags, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(entity_type)
        .bind(entity_id)
        .bind(action)
        .bind(old_values)
        .bind(new_values)
        .bind(user_id)
        .bind(user_role)
        .bind(ip_address)
        .bind(user_agent)
        .bind(session_id)
        .bind(Utc::now())
        .bind(true)
        .bind(error_message)
        .bind(risk_score)
        .bind(serde_json::json!([]).to_string())
        .bind(metadata_json)
        .execute(pool)
        .await?;
        
        info!(
            "Audit: {} {} {} {} by {}",
            entity_type,
            action,
            entity_id,
            user_id.unwrap_or("system")
        );
        
        Ok(())
    }

    pub async fn log_security_event(
        &self,
        pool: &SqlitePool,
        event_type: &str,
        description: &str,
        source_ip: Option<&str>,
        user_id: Option<&str>,
        details: &str,
    ) -> Result<(), sqlx::Error> {
        let risk_score = calculate_risk_score(event_type, "user", None);
        let severity = determine_severity(risk_score);
        
        let details_json = serde_json::json!({
            "description": description,
            "source_ip": source_ip,
            "user_id": user_id,
        }).to_string();
        
        query!(
            r#"
            INSERT INTO security_events (
n                id, event_type, severity, description, source_ip, target_resource, user_id, details, created_at
n            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
n            "#
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(event_type)
        .bind(severity)
        .bind(description)
        .bind(source_ip)
        .bind(user_id)
        .bind(details_json)
        .execute(pool)
        .await?;
        
        warn!(
            "Security event: {} - {} - {}",
            event_type,
            description,
            user_id.unwrap_or("unknown")
        );
        
        Ok(())
    }

    pub async fn get_audit_trail(
        &self,
        pool: &SqlitePool,
        entity_type: Option<&str>,
        entity_id: Option<&str>,
        start_time: Option<chrono::DateTime<Utc>>,
        end_time: Option<chrono::DateTime<Utc>>,
        limit: Option<i64>,
        user_id: Option<&str>,
        action: Option<&str>,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        let mut query = "SELECT * FROM audit_log WHERE 1=1".to_string();
        
        if let Some(entity_type) = entity_type {
            query.push_str(&format!(" AND entity_type = '{}'", entity_type));
        }
        
        if let Some(entity_id) = entity_id {
            query.push_str(&format!(" AND entity_id = '{}'", entity_id));
        }
        
        if let Some(start_time) = start_time {
            query.push_str(&format!(" AND timestamp >= '{}'", start_time.to_rfc3339()));
        }
        
        if let Some(end_time) = end_time {
            query.push_str(&format!(" AND timestamp <= '{}'", end_time.to_rfc3339()));
        }
        
        if let Some(user_id) = user_id {
            query.push_str(&format!(" AND user_id = '{}'", user_id));
        }
        
        if let Some(action) = action {
            query.push_str(&format!(" AND action = '{}'", action));
        }
        
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        query.push_str(" ORDER BY timestamp DESC");
        
        sqlx::query_as(&query)
            .fetch_all(pool)
            .await
    }

    pub async fn get_security_events(
        &self,
        pool: &SqlitePool,
        severity: Option<&str>,
        start_time: Option<chrono::DateTime<Utc>>,
        end_time: Option<chrono::DateTime<Utc>>,
        limit: Option<i64>,
    ) -> Result<Vec<SecurityEvent>, sqlx::Error> {
        let mut query = "SELECT * FROM security_events WHERE 1=1".to_string();
        
        if let Some(severity) = severity {
            query.push_str(&format!(" AND severity = '{}'", severity));
        }
        
        if let Some(start_time) = start_time {
            query.push_str(&format!(" AND created_at >= '{}'", start_time.to_rfc3339()));
        }
        
        if let Some(end_time) = end_time {
            query.push_str(&format!(" AND created_at <= '{}'", end_time.to_rfc3339()));
        }
        
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        query.push_str(" ORDER BY created_at DESC");
        
        sqlx::query_as(&query)
            .fetch_all(pool)
            .await
    }

    pub async fn get_user_activity_log(
        &self,
        pool: &SqlitePool,
        user_id: &str,
        start_time: Option<chrono::DateTime<Utc>>,
        end_time: Option<chrono::DateTime<Utc>>,
        limit: Option<i64>,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        let mut query = "SELECT * FROM audit_log WHERE user_id = ?".to_string();
        
        if let Some(start_time) = start_time {
            query.push_str(&format!(" AND timestamp >= '{}'", start_time.to_rfc3339()));
        }
        
        if let Some(end_time) = end_time {
            query.push_str(&format!(" AND timestamp <= '{}'", end_time.to_rfc3339()));
        }
        
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        query.push_str(" ORDER BY timestamp DESC");
        
        sqlx::query_as(&query)
            .fetch_all(pool)
            .await
    }

    pub async fn cleanup_old_audit_logs(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let cutoff_date = Utc::now() - chrono::Duration::days(90); // Keep 90 days
        
        let result = query!(
            "DELETE FROM audit_log WHERE timestamp < ?"
        )
        .bind(cutoff_date.to_rfc3339())
        .execute(pool)
        .await?;
        
        let count = result.rows_affected();
        info!("Cleaned up {} old audit log entries", count);
        
        Ok(count)
    }

    pub async fn cleanup_old_security_events(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let cutoff_date = Utc::now() - chrono::Duration::days(30); // Keep 30 days
        
        let result = query!(
            "DELETE FROM security_events WHERE created_at < ?"
        )
        .bind(cutoff_date.to_rfc3339())
        .execute(pool)
        .await?;
        
        let count = result.rows_affected();
        info!("Cleaned up {} old security events", count);
        
        Ok(count)
    }

    pub async fn get_compliance_report(
        &self,
        pool: &SqlitePool,
        start_time: Option<chrono::DateTime<Utc>>,
        end_time: Option<chrono::DateTime<Utc>>,
    ) -> Result<serde_json::Value, sqlx::Error> {
        let mut report = serde_json::json!({
            "period": format!(
                "{} to {}",
                start_time.map(|t| t.format()).unwrap_or("1970-01-01".to_string()),
                end_time.map(|t| t.format()).unwrap_or("now".to_string())
            ),
            "summary": HashMap::new(),
            "violations": Vec::new(),
            "recommendations": Vec::new(),
            "metrics": HashMap::new(),
        });
        
        // Get compliance violations
        let violations = query_as!(
            AuditLog,
            "SELECT COUNT(*) as count, entity_type, action, risk_score FROM audit_log 
            WHERE timestamp >= ? AND timestamp <= ? AND risk_score >= 70 
            GROUP BY entity_type, action
        )
        .bind(start_time.map(|t| t.to_rfc3339()))
        .bind(end_time.map(|t| t.to_rfc3339()))
        .await?;
        
        let mut violation_counts = HashMap::new();
        for violation in violations {
            violation_counts.insert(
                (format!("{}:{}", violation.entity_type, violation.action)),
                violation.count,
            );
        }
        report["violations"] = serde_json::to_value(violation_counts)?;
        
        // Get high-risk users
        let high_risk_users = query_as!(
            User,
            "SELECT id, username, failed_login_attempts FROM users 
            WHERE failed_login_attempts >= 3 AND locked_until > CURRENT_TIMESTAMP"
        )
        .fetch_all(pool)
        .await?;
        
        report["high_risk_users"] = serde_json::to_value(
            high_risk_users.iter().map(|u| serde_json::json!({
                "id": u.id,
                "username": u.username,
                "failed_attempts": u.failed_login_attempts,
                "locked_until": u.locked_until.map(|t| t.to_rfc3339()),
            }).collect::<Vec<_>>()
        );
        
        // Get system metrics
        let metrics = query_as!(
            PerformanceMetric,
            "SELECT metric_name, AVG(value) as avg_value, COUNT(*) as count FROM performance_metrics 
            WHERE timestamp >= ? AND timestamp <= ? 
            GROUP BY metric_name
        )
        .bind(start_time.map(|t| t.to_rfc3339()))
        .bind(end_time.map(|t| t.to_rfc3339()))
        .await?;
        
        let mut metrics_map = HashMap::new();
        for metric in metrics {
            metrics_map.insert(
                metric.metric_name.to_string(),
                serde_json::json!({
                    "average": metric.avg_value,
                    "count": metric.count,
                }),
            );
        }
        report["metrics"] = serde_json::to_value(metrics_map);
        
        Ok(report)
    }
}

// Helper functions
fn calculate_risk_score(action: &str, entity_type: &str, user_role: Option<&str>) -> i32 {
    let base_score = match action {
        "delete" => 90,
        "unauthorized_access" => 85,
        "privilege_escalation" => 85,
        "data_breach" => 95,
        "malware_detected" => 100,
        _ => 50,
    };
    
    let role_multiplier = match user_role {
        Some("SUPER_ADMIN") => 0.5, // Reduce risk for admins
        Some("ADMIN") => 0.7,
        Some("AUDITOR") => 0.8,
        _ => 1.0, // Normal users
    };
    
    let entity_multiplier = match entity_type {
        "user" => 0.8,
        "agent" => 0.9,
        "task" => 0.7,
        "system" => 1.0,
        _ => 0.5,
    };
    
    let severity_multiplier = match action {
        "delete" => 1.2,
        "unauthorized_access" => 1.1,
        "privilege_escalation" => 1.1,
        "data_breach" => 1.3,
        "malware_detected" => 1.5,
        _ => 1.0,
    };
    
    let base_score = (base_score * role_multiplier) as i32;
    (base_score * entity_multiplier * severity_multiplier) as i32
}

fn determine_severity(risk_score: i32) -> String {
    match risk_score {
        0..=20 => "low",
        21..50 => "medium",
        51..80 => "high",
        81..100 => "critical",
        _ => "critical",
    }
}

fn validate_json_array_field(json_str: &str) -> Result<Vec<String>, String> {
    match serde_json::from_str::<Vec<Value>>(json_str) {
        Ok(strings) => {
            let mut valid_strings = Vec::new();
            for item in strings {
                if let Some(s) = item.as_str() {
                    if !s.is_empty() {
                        valid_strings.push(s.to_string());
                    }
                }
            }
            Ok(valid_strings)
        }
        Err(e) => Err(format!("Invalid JSON array: {}", e)),
    }
}

fn validate_json_object_field(json_str: &str) -> Result<HashMap<String, Value>, String> {
    match serde_json::from_str::<HashMap<String, Value>>(json_str) {
        Ok(map) => Ok(map),
        Err(e) => Err(format!("Invalid JSON object: {}", e)),
    }
}

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
    
    if !password.chars().any(|c| c.is_digit()) {
        issues.push("Password must contain at least one digit");
    }
    
    if !password.chars().all(|c| c.is_ascii()) {
        issues.push("Password must contain only ASCII characters");
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

fn validate_agent_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Agent name cannot be empty");
    }
    
    if name.len() > 255 {
        return Err("Agent name too long (max 255 characters)");
    }
    
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == ' ') {
        return Err("Agent name contains invalid characters");
    }
    
    Ok(())
}

fn sanitize_filename(filename: &str) -> String {
    let mut sanitized = String::new();
    
    for c in filename.chars() {
        match c {
            c if c.is_alphanumeric() || c.is_ascii_punctuation() || c == ' ' || c == '-' || c == '_' || c == '.' => {
                sanitized.push(c);
            } else {
                sanitized.push('_');
            }
        }
    }
    
    sanitized
}

fn validate_file_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Path cannot be empty");
    }
    
    if path.contains("..") {
        return Err("Path traversal not allowed");
    }
    
    let dangerous_patterns = vec![
        "/etc/", "/sys/", "/proc/", "/dev/", "/root/",
        "~/.ssh/", "~/.aws/", "~/.config/",
        "/etc/passwd", "/etc/shadow", "/etc/sudoers",
        "C:\\Windows\\System32\\", "C:\\Windows\\",
    ];
    
    for pattern in dangerous_patterns {
        if path.to_lowercase().contains(pattern) {
            return Err("Path contains dangerous pattern: {}", pattern);
        }
    }
    
    Ok(())
}

fn validate_file_upload(
    size: i64,
    mime_type: &str,
    max_size: i64,
    allowed_types: &[&str],
) -> Result<(), String> {
    if size > max_size {
        return Err(format!("File size {} exceeds maximum allowed size {}", size, max_size));
    }
    
    if !allowed_types.contains(&mime_type) {
        return Err(format!("MIME type {} not allowed", mime_type));
    }
    
    Ok(())
}

fn validate_file_size(size: i64, max_size: i64) -> Result<(), String> {
    if size > max_size {
        Err(format!("File size {} exceeds maximum allowed size {}", size, max_size));
    }
    
    Ok(())
}
