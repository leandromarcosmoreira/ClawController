use crate::models::*;
use crate::db::SqlitePool;
use sqlx::{query, query_as};
use chrono::{Utc, Duration};
use anyhow::Result;
use tracing::{info, warn, error};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, Encoding, Header as JwtHeader};
use jsonwebtoken::algorithm::Algorithm;
use jsonwebtoken::token::{Header as JwtTokenHeader, Claims as JwtClaims};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
    pub iss: String, // Issuer
    pub aud: String, // Audience
    pub role: String, // User role
    pub permissions: Vec<String>, // User permissions
    pub security_level: SecurityLevel,
    pub access_level: AccessLevel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
    pub two_factor_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
    pub expires_at: String,
    pub permissions: Vec<String>,
    pub two_factor_required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub security_level: SecurityLevel,
    pub access_level: AccessLevel,
    pub is_active: bool,
    pub last_login: Option<String>,
    pub two_factor_enabled: bool,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub profile_picture: Option<String>,
    pub timezone: String,
    pub language: String,
    pub created_at: String,
    pub updated_at: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub role: Option<String>,
    pub security_level: Option<SecurityLevel>,
    pub access_level: Option<AccessLevel>,
    pub permissions: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub profile_picture: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub reset_token: String,
    pub new_password: String,
    pub confirm_password: String,
}

pub struct SecurityService {
    jwt_secret: String,
    token_expiry: Duration,
    password_min_length: usize,
    max_failed_attempts: u32,
    lockout_duration: Duration,
    two_factor_issuer: String,
}

impl SecurityService {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            token_expiry: Duration::hours(24),
            password_min_length: 8,
            max_failed_attempts: 5,
            lockdown_duration: Duration::minutes(30),
            two_factor_issuer: "ClawController".to_string(),
        }
    }

    pub async fn authenticate_user(
        &self,
        pool: &SqlitePool,
        username: &str,
        password: &str,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<Option<(User, String)>, anyhow::Error> {
        // Find user by username
        let user = query_as!(
            User,
            "SELECT * FROM users WHERE username = ? AND is_active = 1"
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        if let Some(user) = user {
            // Check if account is locked
            if let Some(locked_until) = user.locked_until {
                if locked_until > Utc::now() {
                    warn!("User {} is locked until {}", username, locked_until);
                    return Ok(None);
                }
            }

            // Check failed attempts
            if user.failed_login_attempts >= self.max_failed_attempts {
                warn!("User {} has exceeded max failed attempts", username);
                // Lock the account
                let locked_until = Utc::now() + self.lockout_duration;
                query!(
                    "UPDATE users SET locked_until = ?, failed_login_attempts = 0 WHERE id = ?"
                )
                .bind(locked_until)
                .bind(&user.id)
                .execute(pool)
                .await?;
                return Ok(None);
            }

            // Verify password
            if let Some(hash) = &user.password_hash {
                if verify(password, hash).await.is_ok() {
                    // Reset failed attempts on successful login
                    if user.failed_login_attempts > 0 {
                        query!(
                            "UPDATE users SET failed_login_attempts = 0, last_login = CURRENT_TIMESTAMP WHERE id = ?"
                        )
                        .bind(&user.id)
                        .execute(pool)
                        .await?;
                    }

                    // Log successful login
                    self.log_security_event(
                        pool,
                        "login_success",
                        "User logged in successfully",
                        Some(ip_address.to_string()),
                        Some(&user.id),
                        serde_json::json!({
                            "username": username,
                            "user_agent": user_agent
                        }).to_string(),
                    ).await?;

                    return Ok((user, self.generate_token(&user)?));
                } else {
                    // Increment failed attempts
                    let new_attempts = user.failed_login_attempts + 1;
                    query!(
                        "UPDATE users SET failed_login_attempts = ? WHERE id = ?"
                    )
                    .bind(new_attempts)
                    .bind(&user.id)
                    .execute(pool)
                    .await?;

                    // Log failed login attempt
                    self.log_security_event(
                        pool,
                        "login_failure",
                        "Invalid password attempt",
                        Some(ip_address.to_string()),
                        Some(&user.id),
                        serde_json::json!({
                            "username": username,
                            "failed_attempts": new_attempts,
                            "user_agent": user_agent
                        }).to_string(),
                    ).await?;

                    if new_attempts >= self.max_failed_attempts {
                        // Lock the account
                        let locked_until = Utc::now() + self.lockout_duration;
                        query!(
                            "UPDATE users SET locked_until = ? WHERE id = ?"
                        )
                        .bind(locked_until)
                        .bind(&user.id)
                        .execute(pool)
                        .await?;
                    }

                    return Ok(None);
                }
            }
        }

        Ok(None)
    }

    pub async fn create_user(
        &self,
        pool: &SqlitePool,
        request: CreateUserRequest,
        created_by: &str,
        ip_address: &str,
    ) -> Result<User, anyhow::Error> {
        // Check if username already exists
        let existing = query_as!(
            User,
            "SELECT id FROM users WHERE username = ? OR email = ?"
        )
        .bind(&request.username)
        .bind(&request.email)
        .fetch_optional(pool)
        .await?;

        if existing.is_some() {
            return Err(anyhow::anyhow!("Username or email already exists"));
        }

        // Validate password
        if request.password.len() < self.password_min_length {
            return Err(anyhow::anyhow!("Password too short"));
        }

        // Hash password
        let password_hash = hash(&request.password, DEFAULT_COST).await?;

        // Create user
        let user_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let user = User {
            id: user_id.clone(),
            username: request.username.clone(),
            email: request.email.clone(),
            password_hash,
            role: request.role.unwrap_or_else(|| "USER".to_string()),
            is_active: true,
            last_login: None,
            failed_login_attempts: 0,
            locked_until: None,
            security_level: request.security_level.unwrap_or(SecurityLevel::Internal),
            access_level: request.access_level_or(AccessLevel::ReadOnly),
            permissions: request.permissions.unwrap_or_default(),
            created_at: now,
            updated_at: now,
            last_password_change: now,
            two_factor_enabled: false,
            two_factor_secret: None,
            email_verified: false,
            phone_verified: false,
            profile_picture: None,
            timezone: "UTC".to_string(),
            language: "en".to_string(),
        };

        // Insert user
        query!(
            r#"
            INSERT INTO users (
                id, username, email, password_hash, role, is_active, 
                security_level, access_level, permissions, created_at, updated_at, 
                last_password_change, two_factor_enabled, 
                email_verified, phone_verified, timezone, language
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.role)
        .bind(&user.is_active)
        .bind(format!("{:?}", user.security_level))
        .bind(format!("{:?}", user.access_level))
        .bind(serde_json::to_string(&user.permissions)?)
        .bind(&user.created_at)
        .bind(&user.updated_at)
        .bind(&user.last_password_change)
        .bind(&user.two_factor_enabled)
        .bind(&user.email_verified)
        .bind(&user.phone_verified)
        .bind(&user.timezone)
        .bind(&user.language)
        .execute(pool)
        .await?;

        // Log user creation
        self.log_security_event(
            pool,
            "user_created",
            "New user account created",
            Some(ip_address.to_string()),
            Some(&user.id),
            serde_json::json!({
                "username": user.username,
                "email": user.email,
                "role": user.role,
                "created_by": created_by
            }).to_string(),
        ).await?;

        info!("Created user: {}", user.username);

        Ok(user)
    }

    pub async fn update_user(
        &self,
        pool: &SqlitePool,
        user_id: &str,
        request: UpdateUserRequest,
        modified_by: &str,
    ) -> Result<User, anyhow::Error> {
        let mut user = query_as!(
            User,
            "SELECT * FROM users WHERE id = ? AND is_active = 1"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let mut updated_fields = Vec::new();
        
        if let Some(email) = &request.email {
            updated_fields.push(("email", email));
        }
        if let Some(role) = &request.role {
            updated_fields.push(("role", role));
        }
        if let Some(security_level) = &request.security_level {
            updated_fields.push(("security_level", format!("{:?}", security_level)));
        }
        if let Some(access_level) = &request.access_level {
            updated_fields.push(("access_level", format!("{:?}", access_level)));
        }
        if let Some(permissions) = &request.permissions {
            updated_fields.push(("permissions", serde_json::to_string(&permissions)?));
        }
        if let Some(is_active) = request.is_active {
            updated_fields.push(("is_active", format!("{}", is_active)));
        }
        if let Some(profile_picture) = &request.profile_picture {
            updated_fields.push(("profile_picture", profile_picture.clone()));
        }
        if let Some(timezone) = &request.timezone {
            updated_fields(("timezone", timezone.clone()));
        }
        if let Some(language) = &request.language {
            updated_fields.push(("language", language.clone()));
        }

        if updated_fields.is_empty() {
            return Ok(user);
        }

        // Build dynamic UPDATE query
        let mut query = "UPDATE users SET updated_at = CURRENT_TIMESTAMP".to_string();
        let mut params = Vec::new();
        
        for (field, value) in updated_fields {
            query.push_str(&format!(", {} = ?", field));
            params.push(value);
        }
        query.push_str(" WHERE id = ?");
        params.push(user_id.to_string());

        sqlx::query(&query)
            .bind_all(&params)
            .execute(pool)
            .await?;

        // Fetch updated user
        let updated_user = query_as!(
            User,
            "SELECT * FROM users WHERE id = ?"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        // Log update
        self.log_security_event(
            pool,
            "user_updated",
            "User profile updated",
            None,
            Some(user_id),
            serde_json::json!({
                "modified_by": modified_by,
                "updated_fields": updated_fields.iter().map(|(field, _)| field.to_string()).collect::<Vec<_>>()
            }).to_string(),
        ).await?;

        Ok(updated_user)
    }

    pub async fn change_password(
        &self,
        pool: &SqlitePool,
        user_id: &str,
        request: ChangePasswordRequest,
        ip_address: &str,
    ) -> Result<(), anyhow::Error> {
        let user = query_as!(
            User,
            "SELECT * FROM users WHERE id = ? AND is_active = 1"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        // Verify current password
        if let Some(hash) = &user.password_hash {
            if !verify(&request.current_password, hash).await.is_ok() {
                self.log_security_event(
                    pool,
                    "password_change_failed",
                    "Invalid current password",
                    Some(ip_address.to_string()),
                    Some(user_id),
                    serde_json::json!({
                        "reason": "invalid_current_password"
                    }).to_string(),
                ).await?;
                return Err(anyhow::anyhow!("Invalid current password"));
            }
        } else {
            return Err(anyhow::anyhow!("No password hash found"));
        }

        // Validate new password
        if request.new_password != request.confirm_password {
            return Err(anyhow::anyhow!("Passwords do not match"));
        }

        if request.new_password.len() < self.password_min_length {
            return Err(anyhow::anyhow!("Password too short"));
        }

        // Hash new password
        let new_hash = hash(&request.new_password, DEFAULT_COST).await?;

        // Update password and reset failed attempts
        query!(
            "UPDATE users SET password_hash = ?, last_password_change = CURRENT_TIMESTAMP, failed_login_attempts = 0 WHERE id = ?"
        )
        .bind(new_hash)
        .bind(user_id)
        .execute(pool)
        .await?;

        // Log password change
        self.log_security_event(
            pool,
            "password_changed",
            "User changed password",
            Some(ip_address.to_string()),
            Some(user_id),
            serde_json::json!({
                "changed_by": "user"
            }).to_string(),
        ).await?;

        info!("Password changed for user: {}", user.username);

        Ok(())
    }

    pub async fn reset_password(
        &self,
        pool: &SqlitePool,
        request: ResetPasswordRequest,
        ip_address: &str,
    ) -> Result<(), anyhow::Error> {
        // Find user by email
        let user = query_as!(
            User,
            "SELECT * FROM users WHERE email = ? AND is_active = 1"
        )
        .bind(&request.email)
        .fetch_optional(pool)
        .await?;

        if let Some(user) = user {
            // Verify reset token (in production, this would be sent via email)
            // For now, we'll accept the token directly
            if request.reset_token.len() < 32 {
                return Err(anyhow::anyhow!("Invalid reset token"));
            }

            // Validate new password
            if request.new_password != request.confirm_password {
                return Err(anyhow::anyhow!("Passwords do not match"));
            }

            if request.new_password.len() < self.password_min_length {
                return Err(anyhow::anyhow!("Password too short"));
            }

            // Hash new password
            let new_hash = hash(&request.new_password, DEFAULT_COST).await?;

            // Update password and reset failed attempts
            query!(
                "UPDATE users SET password_hash = ?, last_password_change = CURRENT_TIMESTAMP, failed_login_attempts = 0, locked_until = NULL WHERE id = ?"
            )
            .bind(new_hash)
            .bind(&user.id)
            .execute(pool)
            .await?;

            // Log password reset
            self.log_security_event(
                pool,
                "password_reset",
                "Password reset completed",
                Some(ip_address.to_string()),
                Some(&user.id),
                serde_json::json!({
                    "email": user.email,
                    "reset_by": "user"
                }).to_string(),
            ).await?;

            info!("Password reset completed for: {}", user.email);
        } else {
            return Err(anyhow::anyhow!("Email not found"));
        }

        Ok(())
    }

    pub async fn create_session(
        &self,
        pool: &SqlitePool,
        user_id: &str,
        ip_address: &str,
        user_agent: &str,
        remember_me: bool,
    ) -> Result<Session, anyhow::Error> {
        let expires_at = if remember_me {
            Utc::now() + Duration::days(30)
        } else {
            Utc::now() + self.token_expiry
        };

        let session_id = uuid::Uuid::new_v4().to_string();
        let token = self.generate_token(&query_as!(
            User,
            "SELECT * FROM users WHERE id = ?"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?)?;

        let device_fingerprint = self.generate_device_fingerprint(user_agent, ip_address);

        let session = Session {
            id: session_id.clone(),
            user_id: user_id.to_string(),
            token,
            expires_at,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
            is_active: true,
            device_fingerprint: Some(device_fingerprint),
        };

        // Clean up expired sessions
        query!(
            "DELETE FROM sessions WHERE expires_at < CURRENT_TIMESTAMP"
        )
        .execute(pool)
        .await?;

        // Insert new session
        query!(
            r#"
            INSERT INTO sessions (
n                id, user_id, token, expires_at, created_at, last_accessed, ip_address, 
n                user_agent, is_active, device_fingerprint
n            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
n            "#
        )
        .bind(&session.id)
        .bind(&session.user_id)
        .bind(&session.token)
        .bind(&session.expires_at)
        .bind(&session.created_at)
        .bind(&session.last_accessed)
        .bind(&session.ip_address)
        .bind(&session.user_agent)
        .bind(&session.is_active)
        .bind(&session.device_fingerprint)
        .execute(pool)
        .await?;

        // Log session creation
        self.log_security_event(
            pool,
            "session_created",
            "New session created",
            Some(ip_address.to_string()),
            Some(user_id),
            serde_json::json!({
                "session_id": session_id,
                "remember_me": remember_me,
                "device_fingerprint": device_fingerprint
            }).to_string(),
        ).await?;

        Ok(session)
    }

    pub async fn validate_session(
        &self,
        pool: &SqlitePool,
        token: &str,
        ip_address: &str,
    ) -> Result<Option<User>, anyhow::Error> {
        // Find session by token
        let session = query_as!(
            Session,
            r#"
            SELECT s.*, u.* FROM sessions s
            JOIN users u ON s.user_id = u.id
            WHERE s.token = ? AND s.is_active = 1 AND (s.expires_at > CURRENT_TIMESTAMP)
n            "#
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        if let Some(session) = session {
            // Update last_accessed
            query!(
                "UPDATE sessions SET last_accessed = CURRENT_TIMESTAMP WHERE id = ?"
            )
            .bind(&session.id)
            .execute(pool)
            .await?;

            // Check if IP address has changed significantly
            if session.ip_address != ip_address {
                // Log potential session hijacking
                self.log_security_event(
                    pool,
                    "suspicious_activity",
                    "Session IP address changed",
                    Some(ip_address.to_string()),
                    Some(&session.user_id),
                    serde_json::json!({
                        "old_ip": session.ip_address,
                        "new_ip": ip_address,
                        "user_agent": session.user_agent
                    }).to_string(),
                ).await?;
            }

            // Return the associated user
            let user = query_as!(
                User,
                "SELECT * FROM users WHERE id = ? AND is_active = 1"
            )
            .bind(&session.user_id)
            .fetch_one(pool)
            .await?;

            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    pub async fn revoke_session(&self, pool: &SqlitePool, token: &str) -> Result<(), anyhow::Error> {
        query!(
            "UPDATE sessions SET is_active = 0 WHERE token = ?"
        )
        .bind(token)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn revoke_all_user_sessions(&self, pool: &SqlitePool, user_id: &str) -> Result<(), anyhow::Error> {
        query!(
            "UPDATE sessions SET is_active = 0 WHERE user_id = ?"
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub fn generate_token(&self, user: &User) -> Result<String, anyhow::Error> {
        let expiration = Utc::now() + self.token_expiry;
        
        let claims = Claims {
            sub: user.id.clone(),
            exp: expiration.timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            iss: "ClawController".to_string(),
            aud: "clawcontroller-api".to_string(),
            role: user.role.clone(),
            permissions: user.permissions.clone().unwrap_or_default(),
            security_level: user.security_level,
            access_level: user.access_level,
        };

        let header = JwtHeader::new(
            Algorithm::HS512,
            JwtHeader::default(),
        );

        let token = encode(&header, &self.jwt_secret, &claims)?;
        
        Ok(token)
    }

    pub fn generate_device_fingerprint(&self, user_agent: &str, ip_address: &str) -> String {
        use std::collections::hash_map::Default;
        
        let mut hasher = Default::default();
        hasher.hash(user_agent);
        hasher.hash(ip_address);
        format!("{:x}", hasher.finish())
    }

    pub async fn log_security_event(
        &self,
        pool: &SqlitePool,
        event_type: &str,
        description: &str,
        ip_address: Option<String>,
        user_id: Option<&str>,
        details: String,
    ) -> Result<(), sqlx::Error> {
        let risk_score = match event_type {
            "login_failure" => 80,
            "unauthorized_access" => 90,
            "privilege_escalation" => 85,
            "data_breach" => 95,
            "suspicious_activity" => 70,
            "malware_detected" => 100,
            _ => 50,
        };

        let severity = match risk_score {
            0..=20 => "low",
            21..50 => "medium",
            51..80 => "high",
            _ => "critical",
        };

        query!(
            r#"
            INSERT INTO security_events (
n                id, event_type, severity, description, source_ip, target_resource, user_id, details, created_at
n            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
n            "#
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(event_type)
        .bind(severity)
        .bind(description)
        .bind(ip_address)
        .bind(user_id)
        .bind(details)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn check_permissions(
        &self,
        user: &User,
        resource: &str,
        action: &str,
    ) -> bool {
        // Check if user has super admin access
        if user.access_level == AccessLevel::SuperAdmin {
            return true;
        }

        // Check specific permissions
        if let Some(permissions) = &user.permissions {
            let required_permission = format!("{}:{}", resource, action);
            permissions.contains(&required_permission)
        } else {
            false
        }
    }

    pub async fn get_user_permissions(&self, user: &User) -> Vec<String> {
        // Get user's direct permissions
        let mut permissions = user.permissions.clone().unwrap_or_default();
        
        // Add role-based permissions
        match user.role.as_str() {
            "SUPER_ADMIN" => {
                permissions.extend(vec![
                    "system:read", "system:write", "system:delete", "system:admin",
                    "users:read", "users:write", "users:delete", "users:admin",
                    "agents:read", "agents:write", "agents:delete", "agents:admin",
                    "tasks:read", "tasks:write", "tasks:delete", "tasks:admin",
                    "audit:read", "security:read", "monitoring:read",
                ]);
            },
            "ADMIN" => {
                permissions.extend(vec![
                    "agents:read", "agents:write", "agents:delete", "agents:admin",
                    "tasks:read", "tasks:write", "tasks:delete",
                    "users:read", "audit:read",
                    "monitoring:read",
                ]);
            },
            "USER" => {
                permissions.extend(vec![
                    "agents:read", "tasks:read", "monitoring:read",
                ]);
            },
            "READ_ONLY" => {
                permissions.extend(vec![
                    "agents:read", "tasks:read",
                ]);
            },
            _ => {}
        }

        permissions
    }

    pub async fn validate_access_level(
        &self,
        user: &User,
        required_level: AccessLevel,
    ) -> bool {
        user.access_level >= required_level
    }

    pub async fn validate_security_level(
        &self,
        user: &User,
        required_level: SecurityLevel,
    ) -> bool {
        user.security_level >= required_level
    }
}

// Utility functions for security
pub async fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST).await
}

pub async fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash).await
}

pub fn generate_secure_token() -> String {
    use rand::Rng;
    use rand::distributions::Alphanumeric;
    
    let rng = rand::thread_rng();
    (0..128)
        .map(|_| rng.sample(&Alphanumeric()))
        .collect()
}

pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

pub fn validate_password_strength(password: &str) -> (bool, Vec<String>) {
    let mut issues = Vec::new();
    
    if password.len() < 8 {
        issues.push("Password must be at least 8 characters long".to_string());
    }
    
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        issues.push("Password must contain at least one uppercase letter".to_string());
    }
    
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        issues.push("Password must contain at least one lowercase letter".to_string());
    }
    
    if !password.chars().any(|c| c.is_ascii_digit()) {
        issues.push("Password must contain at least one digit".to_string());
    }
    
    if password.chars().any(|c| !c.is_ascii()) {
        issues.push("Password must contain only ASCII characters".to_string());
    }
    
    let common_passwords = vec![
        "password", "123456", "qwerty", "abc123", "password123", "admin", "letmein",
        "welcome", "monkey", "dragon", "password1", "123456789",
    ];
    
    if common_passwords.contains(&password.to_lowercase()) {
        issues.push("Password is too common".to_string());
    }
    
    (issues.is_empty(), issues)
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

// Input validation
pub fn validate_agent_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Agent name cannot be empty".to_string());
    }
    
    if name.len() > 255 {
        return Err("Agent name too long (max 255 characters)".to_string());
    }
    
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == ' ') {
        return Err("Agent name contains invalid characters".to_string());
    }
    
    Ok(())
}

pub fn sanitize_input(input: &str) -> String {
    input.chars()
        .filter(|c| c.is_ascii() && !c.is_control())
        .collect()
}

pub fn validate_json_field(json_str: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))
}

// Security middleware utilities
pub fn extract_bearer_token(auth_header: &str) -> Option<String> {
    if auth_header.starts_with("Bearer ") {
        Some(auth_header[7..].to_string())
    } else {
        None
    }
}

pub fn is_safe_path(path: &str) -> bool {
    // Basic path traversal protection
    if path.contains("..") {
        return false;
    }
    
    // Check for dangerous file extensions
    let dangerous_extensions = vec![
        ".exe", ".bat", ".cmd", ".com", ".scr", ".vbs", ".bat", ".sh",
        ".ps1", ".ps2", ".pyc", ".rb", ".php", ".jsp", ".asp", ".aspx",
        ".js", ".ts", ".coffee", ".css", ".html", ".htm",
    ];
    
    let path_lower = path.to_lowercase();
    dangerous_extensions.iter().any(|ext| path_lower.ends_with(ext))
}

pub fn validate_file_size(size: i64, max_size: i64) -> Result<(), String> {
    if size > max_size {
        return Err(format!("File size {} exceeds maximum allowed size {}", size, max_size));
    }
    Ok(())
}

// Error handling
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    InvalidCredentials,
    AccountLocked,
    AccountNotFound,
    PermissionDenied,
    TokenExpired,
    InvalidToken,
    DatabaseError(String),
    ValidationError(String),
    RateLimited,
    Unauthorized,
    InternalServerError,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SecurityError::InvalidCredentials => write!(f, "Invalid credentials"),
            SecurityError::AccountLocked => write!(f, "Account locked"),
            SecurityError::AccountNotFound => write!(f, "Account not found"),
            SecurityError::PermissionDenied => write!(f, "Permission denied"),
            SecurityError::TokenExpired => write!(f, "Token expired"),
            SecurityError::InvalidToken => write!(f, "Invalid token"),
            SecurityError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            SecurityError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            SecurityError::RateLimited => write!(f, "Rate limited"),
            SecurityError::Unauthorized => write!(f, "Unauthorized"),
            SecurityError::InternalServerError => write!(f, "Internal server error"),
        }
    }
}

impl std::error::Error for SecurityError {
    fn from(error: sqlx::Error) -> Self {
        SecurityError::DatabaseError(error.to_string())
    }
}

impl std::error::Error for bcrypt::BcryptError {
    fn from(error: bcrypt::BcryptError) -> Self {
        SecurityError::InvalidCredentials
    }
}

impl std::error::Error for serde_json::Error {
    fn from(error: serde_json::Error) -> Self {
        SecurityError::ValidationError(error.to_string())
    }
}
