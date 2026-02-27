use crate::models::*;
use axum::{
    extract::{Path, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
    middleware,
};
use sqlx::SqlitePool;
use chrono::Utc;
use std::collections::HashMap;
use serde_json::Value;
use sha2::{Sha256, Digest};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use lru::LruCache;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use validator::Validate;
use regex::Regex;
use tracing::{info, warn, error, debug, instrument};
use metrics::{counter, histogram, gauge};
use tower::timeout::TimeoutLayer;
use tower::retry::RetryLayer;
use tower::limit::RateLimitLayer;

// Performance and Caching Infrastructure

/// Configuration cache with TTL and size limits
#[derive(Clone)]
pub struct ConfigCache {
    inner: Arc<RwLock<LruCache<String, CachedConfig>>>,
}

#[derive(Clone)]
struct CachedConfig {
    data: Value,
    hash: String,
    cached_at: chrono::DateTime<Utc>,
    ttl: Duration,
}

impl ConfigCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(capacity).unwrap()
            ))),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Value> {
        let cache = self.inner.read().await;
        if let Some(cached) = cache.get(key) {
            if cached.cached_at + cached.ttl > Utc::now() {
                return Some(cached.data.clone());
            }
        }
        None
    }

    pub async fn put(&self, key: String, data: Value, ttl: Duration) {
        let hash = format!("{:x}", Sha256::digest(data.to_string().as_bytes()));
        let cached = CachedConfig {
            data,
            hash,
            cached_at: Utc::now(),
            ttl,
        };
        
        let mut cache = self.inner.write().await;
        cache.put(key, cached);
    }

    pub async fn invalidate(&self, pattern: &str) {
        let mut cache = self.inner.write().await;
        let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new(r".*").unwrap());
        
        let keys_to_remove: Vec<String> = cache
            .iter()
            .map(|(k, _)| k.clone())
            .filter(|k| re.is_match(k))
            .collect();
            
        for key in keys_to_remove {
            cache.pop(&key);
        }
    }
}

/// Global configuration cache instance
static CONFIG_CACHE: Lazy<ConfigCache> = Lazy::new(|| ConfigCache::new(1000));

/// Metrics collector for OpenClaw integration
#[derive(Clone)]
pub struct OpenClawMetrics {
    pub config_reads_total: metrics::Counter,
    pub config_cache_hits_total: metrics::Counter,
    pub config_sync_duration: metrics::Histogram,
    pub agent_updates_total: metrics::Counter,
    pub active_agents: metrics::Gauge,
}

impl OpenClawMetrics {
    pub fn new() -> Self {
        Self {
            config_reads_total: metrics::counter!("openclaw_config_reads_total"),
            config_cache_hits_total: metrics::counter!("openclaw_config_cache_hits_total"),
            config_sync_duration: metrics::histogram!("openclaw_config_sync_duration_seconds"),
            agent_updates_total: metrics::counter!("openclaw_agent_updates_total"),
            active_agents: metrics::gauge!("openclaw_active_agents"),
        }
    }
}

/// Global metrics instance
static METRICS: Lazy<OpenClawMetrics> = Lazy::new(OpenClawMetrics::new);

/// Security validation utilities
pub struct SecurityValidator;

impl SecurityValidator {
    pub fn validate_agent_id(agent_id: &str) -> Result<(), String> {
        // Allow only alphanumeric, hyphens, and underscores
        let re = Regex::new(r"^[a-zA-Z0-9_-]+$").map_err(|e| format!("Invalid regex: {}", e))?;
        if !re.is_match(agent_id) {
            return Err("Invalid agent ID format".to_string());
        }
        
        // Length validation
        if agent_id.len() > 64 {
            return Err("Agent ID too long (max 64 chars)".to_string());
        }
        
        Ok(())
    }

    pub fn validate_file_path(path: &str) -> Result<(), String> {
        // Prevent path traversal attacks
        if path.contains("..") || path.contains('~') {
            return Err("Invalid path: potential traversal attack".to_string());
        }
        
        // Allow only specific file extensions
        let allowed_extensions = [".json", ".yaml", ".yml"];
        let has_valid_extension = allowed_extensions.iter().any(|&ext| path.ends_with(ext));
        
        if !has_valid_extension {
            return Err("Invalid file extension".to_string());
        }
        
        Ok(())
    }

    pub fn sanitize_json_input(input: &Value) -> Result<Value, String> {
        // Remove potentially dangerous fields
        match input {
            Value::Object(mut map) => {
                // Remove sensitive fields that shouldn't be stored
                map.remove("password");
                map.remove("token");
                map.remove("secret");
                map.remove("private_key");
                
                // Recursively sanitize nested objects
                for (_, value) in map.iter_mut() {
                    *value = Self::sanitize_json_input(value.clone())?;
                }
                
                Ok(Value::Object(map))
            }
            Value::Array(arr) => {
                let sanitized: Result<Vec<_>, _> = arr.iter()
                    .cloned()
                    .map(Self::sanitize_json_input)
                    .collect();
                Ok(Value::Array(sanitized?))
            }
            _ => Ok(input.clone()),
        }
    }
}

/// Resilience patterns for OpenClaw operations
#[derive(Clone)]
pub struct OpenClawResilience {
    pub max_retries: u32,
    pub timeout_duration: Duration,
    pub circuit_breaker_threshold: u32,
}

impl Default for OpenClawResilience {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout_duration: Duration::from_secs(30),
            circuit_breaker_threshold: 5,
        }
    }
}

impl OpenClawResilience {
    pub async fn execute_with_resilience<F, T>(&self, operation: F) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn() -> Result<T, Box<dyn std::error::Error + Send + Sync>> + Clone,
    {
        let mut last_error = None;
        
        for attempt in 1..=self.max_retries {
            debug!("OpenClaw operation attempt {}/{}", attempt, self.max_retries);
            
            match tokio::time::timeout(self.timeout_duration, async {
                operation()
            }).await {
                Ok(Ok(result)) => {
                    if attempt > 1 {
                        info!("OpenClaw operation succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Ok(Err(e)) => {
                    warn!("OpenClaw operation failed on attempt {}: {}", attempt, e);
                    last_error = Some(e);
                }
                Err(_) => {
                    warn!("OpenClaw operation timed out on attempt {}", attempt);
                    last_error = Some("Operation timed out".into());
                }
            }
            
            // Exponential backoff
            if attempt < self.max_retries {
                let backoff = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                tokio::time::sleep(backoff).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| "Unknown error".into()))
    }
}

/// Enhanced OpenClaw Configuration Management with Optimizations

/// Get comprehensive agent configurations from OpenClaw with caching
#[instrument(skip(state))]
pub async fn get_openclaw_agent_configs(
    State(state): State<crate::AppState>,
) -> Result<Json<Vec<OpenClawAgentConfig>>, (StatusCode, String)> {
    let start_time = std::time::Instant::now();
    METRICS.config_reads_total.increment();
    
    // Try cache first
    if let Some(cached_config) = CONFIG_CACHE.get("openclaw_config").await {
        METRICS.config_cache_hits_total.increment();
        debug!("Config retrieved from cache");
        
        let configs = parse_configs_from_value(&cached_config)?;
        histogram!("openclaw_config_parse_duration", start_time.elapsed().as_secs_f64());
        
        return Ok(Json(configs));
    }

    // Cache miss - read from file
    let resilience = OpenClawResilience::default();
    let configs = resilience.execute_with_resilience(|| {
        Box::pin(async {
            read_and_parse_openclaw_config().await
        })
    }).await.map_err(|e| {
        error!("Failed to read OpenClaw config: {}", e);
        (StatusCode::SERVICE_UNAVAILABLE, format!("Cannot read openclaw config: {}", e))
    })?;

    // Cache the result
    let config_value = serde_json::to_value(&configs).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialization error: {}", e))
    })?;
    
    CONFIG_CACHE.put("openclaw_config".to_string(), config_value.clone(), Duration::from_secs(300)).await;
    
    histogram!("openclaw_config_sync_duration", start_time.elapsed().as_secs_f64());
    info!("Successfully loaded {} agent configurations", configs.len());
    
    Ok(Json(configs))
}

/// Get specific agent configuration with validation and caching
#[instrument(skip(state))]
pub async fn get_openclaw_agent_config(
    Path(agent_id): Path<String>,
    State(state): State<crate::AppState>,
) -> Result<Json<OpenClawAgentConfig>, (StatusCode, String)> {
    // Validate agent ID
    SecurityValidator::validate_agent_id(&agent_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Try cache first
    let cache_key = format!("agent_config_{}", agent_id);
    if let Some(cached_config) = CONFIG_CACHE.get(&cache_key).await {
        METRICS.config_cache_hits_total.increment();
        
        let config: OpenClawAgentConfig = serde_json::from_value(cached_config)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Deserialization error: {}", e)))?;
        
        return Ok(Json(config));
    }

    // Cache miss - get from full config
    let configs = get_openclaw_agent_configs(State(state)).await?.0;
    
    let config = configs.into_iter()
        .find(|c| c.id == agent_id)
        .ok_or((StatusCode::NOT_FOUND, "Agent not found".to_string()))?;

    // Cache individual agent config
    let config_value = serde_json::to_value(&config)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialization error: {}", e)))?;
    
    CONFIG_CACHE.put(cache_key, config_value, Duration::from_secs(600)).await;

    Ok(Json(config))
}

/// Enhanced sync with batch operations and validation
#[instrument(skip(state))]
pub async fn sync_openclaw_configs(
    State(state): State<crate::AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let start_time = std::time::Instant::now();
    METRICS.config_sync_duration.record(start_time.elapsed().as_secs_f64());
    
    let openclaw_configs = get_openclaw_agent_configs(State(state.clone())).await?.0;
    let mut synced_count = 0;
    let mut errors = Vec::new();
    let mut updates = Vec::new();

    // Batch preparation
    for config in &openclaw_configs {
        // Validate configuration
        if let Err(e) = validate_agent_config_internal(config) {
            errors.push(format!("Agent {}: {}", config.id, e));
            continue;
        }
        
        // Prepare batch update
        updates.push(prepare_agent_update(config));
    }

    // Execute batch database operations
    match execute_batch_agent_updates(&state.pool, &updates).await {
        Ok(count) => synced_count = count,
        Err(e) => errors.push(format!("Batch update failed: {}", e)),
    }

    // Update metrics
    METRICS.active_agents.set(synced_count as f64);
    
    // Invalidate relevant cache entries
    CONFIG_CACHE.invalidate("agent_config_*").await;
    
    let duration = start_time.elapsed();
    info!("Configuration sync completed: {} agents synced in {:?} with {} errors", 
          synced_count, duration, errors.len());

    Ok(Json(serde_json::json!({
        "synced": synced_count,
        "errors": errors,
        "duration_ms": duration.as_millis(),
        "cache_invalidated": true
    })))
}

/// Apply configuration with comprehensive validation and audit
#[instrument(skip(state))]
pub async fn apply_agent_config(
    Path(agent_id): Path<String>,
    State(state): State<crate::AppState>,
    Json(mut config): Json<OpenClawAgentConfig>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Validate agent ID
    SecurityValidator::validate_agent_id(&agent_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Sanitize input
    let config_value = serde_json::to_value(&config)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)))?;
    
    let sanitized_value = SecurityValidator::sanitize_json_input(&config_value)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Security validation failed: {}", e)))?;
    
    config = serde_json::from_value(sanitized_value)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Deserialization error: {}", e)))?;

    // Validate configuration
    validate_agent_config_internal(&config)?;

    // Apply with resilience
    let resilience = OpenClawResilience::default();
    let result = resilience.execute_with_resilience(|| {
        Box::pin(async {
            apply_agent_config_to_db(&state.pool, &agent_id, &config).await
        })
    }).await.map_err(|e| {
        error!("Failed to apply config for agent {}: {}", agent_id, e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Application failed: {}", e))
    })?;

    // Update metrics and cache
    METRICS.agent_updates_total.increment();
    let cache_key = format!("agent_config_{}", agent_id);
    CONFIG_CACHE.invalidate(&cache_key).await;

    info!("Successfully applied configuration for agent {}", agent_id);

    Ok(Json(serde_json::json!({
        "status": "success",
        "agent_id": agent_id,
        "applied_at": Utc::now(),
        "config_hash": result
    })))
}

/// Enhanced agent list with capabilities and performance metrics
#[instrument(skip(state))]
pub async fn fetch_enhanced_openclaw_agents(
    State(state): State<crate::AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let start_time = std::time::Instant::now();
    
    // Parallel execution of database and OpenClaw config fetch
    let (db_agents, openclaw_configs) = tokio::try_join!(
        get_db_agents_optimized(&state.pool),
        get_openclaw_agent_configs(State(state.clone()))
    ).map_err(|e| {
        error!("Failed to fetch agent data: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Data fetch failed".to_string())
    })?;

    let config_map: HashMap<String, OpenClawAgentConfig> = openclaw_configs
        .into_iter()
        .map(|c| (c.id.clone(), c))
        .collect();

    // Enhanced agent information with performance data
    let enhanced_agents: Vec<Value> = db_agents.into_iter().map(|agent| {
        let openclaw_config = config_map.get(&agent.id);
        let capabilities = get_agent_capabilities(openclaw_config);
        
        serde_json::json!({
            "id": agent.id,
            "name": agent.name,
            "role": agent.role,
            "status": agent.status,
            "workspace": agent.workspace,
            "primary_model": agent.primary_model,
            "fallback_model": agent.fallback_model,
            "current_model": agent.current_model,
            "created_at": agent.created_at,
            "openclaw_config": openclaw_config,
            "sync_status": if agent.openclaw_config_hash.is_some() { "synced" } else { "pending" },
            "capabilities": capabilities,
            "performance": {
                "model_failure_count": agent.model_failure_count,
                "last_updated": agent.created_at, // This should be updated_at in real schema
                "health_score": calculate_agent_health_score(&agent, openclaw_config)
            }
        })
    }).collect();

    let duration = start_time.elapsed();
    histogram!("openclaw_enhanced_fetch_duration", duration.as_secs_f64());
    
    Ok(Json(serde_json::json!({
        "data": enhanced_agents,
        "total": enhanced_agents.len(),
        "fetch_duration_ms": duration.as_millis(),
        "cache_status": "active"
    })))
}

// Helper Functions with Optimizations

async fn read_and_parse_openclaw_config() -> Result<Vec<OpenClawAgentConfig>, Box<dyn std::error::Error + Send + Sync>> {
    let openclaw_dir = std::env::var("OPENCLAW_STATE_DIR")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.openclaw", h)))
        .unwrap_or_else(|_| "/root/.openclaw".to_string());
    
    let config_path = format!("{}/openclaw.json", openclaw_dir);
    
    // Validate file path for security
    SecurityValidator::validate_file_path(&config_path)?;

    let content = tokio::fs::read_to_string(&config_path).await?;
    let config: Value = serde_json::from_str(&content)?;

    let agents_config = config.get("agents")
        .ok_or("Missing agents config")?;
    
    let defaults = agents_config.get("defaults").unwrap_or(&Value::Object(Default::default()));
    let list = agents_config.get("list").and_then(|l| l.as_array()).unwrap_or(&[]);

    let mut enhanced_configs = Vec::with_capacity(list.len());

    for agent_entry in list {
        if let Some(agent_config) = parse_openclaw_agent_config(agent_entry, defaults) {
            enhanced_configs.push(agent_config);
        }
    }

    Ok(enhanced_configs)
}

fn parse_configs_from_value(config_value: &Value) -> Result<Vec<OpenClawAgentConfig>, (StatusCode, String)> {
    let agents_config = config_value.get("agents")
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Missing agents config".to_string()))?;
    
    let defaults = agents_config.get("defaults").unwrap_or(&Value::Object(Default::default()));
    let list = agents_config.get("list").and_then(|l| l.as_array()).unwrap_or(&[]);

    let mut configs = Vec::with_capacity(list.len());
    for agent_entry in list {
        if let Some(config) = parse_openclaw_agent_config(agent_entry, defaults) {
            configs.push(config);
        }
    }

    Ok(configs)
}

async fn get_db_agents_optimized(pool: &SqlitePool) -> Result<Vec<Agent>, sqlx::Error> {
    sqlx::query_as::<sqlx::Sqlite, Agent>(
        "SELECT * FROM agents ORDER BY name"
    )
    .fetch_all(pool)
    .await
}

fn prepare_agent_update(config: &OpenClawAgentConfig) -> AgentUpdateBatch {
    // Calculate config hash
    let config_json = serde_json::to_string(config).unwrap_or_default();
    let config_hash = format!("{:x}", Sha256::digest(config_json.as_bytes()));

    AgentUpdateBatch {
        agent_id: config.id.clone(),
        name: config.name.clone(),
        config_hash,
        config_data: config_json,
    }
}

struct AgentUpdateBatch {
    agent_id: String,
    name: Option<String>,
    config_hash: String,
    config_data: String,
}

async fn execute_batch_agent_updates(
    pool: &SqlitePool, 
    updates: &[AgentUpdateBatch]
) -> Result<usize, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut updated_count = 0;

    for update in updates {
        // Upsert agent with configuration
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO agents (
                id, name, role, status, openclaw_config_hash, created_at
            ) VALUES (?, ?, 'SPC', 'IDLE', ?, CURRENT_TIMESTAMP)
            "#
        )
        .bind(&update.agent_id)
        .bind(&update.name.as_deref().unwrap_or(&update.agent_id))
        .bind(&update.config_hash)
        .execute(&mut *tx)
        .await?;

        // Store configuration snapshot
        let snapshot_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO openclaw_config_snapshots (id, agent_id, config_hash, raw_config, is_active) VALUES (?, ?, ?, ?, 1)"
        )
        .bind(&snapshot_id)
        .bind(&update.agent_id)
        .bind(&update.config_hash)
        .bind(&update.config_data)
        .execute(&mut *tx)
        .await?;

        updated_count += 1;
    }

    tx.commit().await?;
    Ok(updated_count)
}

fn calculate_agent_health_score(agent: &Agent, openclaw_config: Option<&OpenClawAgentConfig>) -> f64 {
    let mut score = 100.0;
    
    // Deduct points for model failures
    score -= (agent.model_failure_count as f64) * 5.0;
    
    // Add points for having OpenClaw config
    if agent.openclaw_config_hash.is_some() {
        score += 10.0;
    }
    
    // Add points for advanced features
    if let Some(config) = openclaw_config {
        if config.heartbeat.as_ref().and_then(|h| h.enabled).unwrap_or(false) {
            score += 5.0;
        }
        if config.sandbox.as_ref().is_some() {
            score += 5.0;
        }
        if config.tools.as_ref().is_some() {
            score += 5.0;
        }
    }
    
    score.max(0.0).min(100.0)
}

// Re-export other functions from the original implementation
pub use super::openclaw_integration_helpers::*;
