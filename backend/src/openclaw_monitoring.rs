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

// Real-time Event Synchronization and Monitoring

/// Event-driven configuration synchronization
#[derive(Clone, Debug)]
pub struct ConfigSyncEvent {
    pub event_type: SyncEventType,
    pub agent_id: String,
    pub config_hash: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub data: Option<Value>,
}

#[derive(Clone, Debug)]
pub enum SyncEventType {
    ConfigChanged,
    AgentAdded,
    AgentRemoved,
    SyncStarted,
    SyncCompleted,
    SyncFailed,
}

/// Real-time event broadcaster for OpenClaw configuration changes
#[derive(Clone)]
pub struct OpenClawEventBroadcaster {
    pub subscribers: Arc<DashMap<String, tokio::sync::broadcast::Sender<ConfigSyncEvent>>>,
    pub event_history: Arc<RwLock<Vec<ConfigSyncEvent>>>,
    pub metrics: Arc<OpenClawMetrics>,
}

impl OpenClawEventBroadcaster {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(DashMap::new()),
            event_history: Arc::new(RwLock::new(Vec::with_capacity(1000))),
            metrics: Arc::new(OpenClawMetrics::new()),
        }
    }

    pub async fn subscribe(&self, subscriber_id: String) -> tokio::sync::broadcast::Receiver<ConfigSyncEvent> {
        let (tx, rx) = tokio::sync::broadcast::channel(1000);
        self.subscribers.insert(subscriber_id.clone(), tx);
        
        info!("New event subscriber: {}", subscriber_id);
        
        // Send recent history to new subscriber
        let history = self.event_history.read().await;
        for event in history.iter().rev().take(50).rev() {
            let _ = tx.send(event.clone());
        }
        
        rx
    }

    pub async fn broadcast(&self, event: ConfigSyncEvent) {
        // Update metrics
        match event.event_type {
            SyncEventType::ConfigChanged => {
                self.metrics.config_reads_total.increment();
            }
            SyncEventType::SyncCompleted => {
                self.metrics.agent_updates_total.increment();
            }
            _ => {}
        }

        // Store in history
        {
            let mut history = self.event_history.write().await;
            history.push(event.clone());
            
            // Keep only last 1000 events
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        // Broadcast to all subscribers
        let mut active_subscribers = 0;
        for entry in self.subscribers.iter() {
            match entry.value().send(event.clone()) {
                Ok(count) => {
                    active_subscribers += 1;
                    debug!("Broadcasted event to {} subscribers", count);
                }
                Err(_) => {
                    // Remove disconnected subscriber
                    let subscriber_id = entry.key().clone();
                    drop(entry);
                    self.subscribers.remove(&subscriber_id);
                }
            }
        }

        self.metrics.active_agents.set(active_subscribers as f64);
        info!("Event broadcasted to {} active subscribers", active_subscribers);
    }

    pub async fn get_event_history(&self, limit: Option<usize>) -> Vec<ConfigSyncEvent> {
        let history = self.event_history.read().await;
        let limit = limit.unwrap_or(100);
        
        history.iter().rev().take(limit).cloned().collect()
    }
}

/// Global event broadcaster instance
static EVENT_BROADCASTER: Lazy<OpenClawEventBroadcaster> = Lazy::new(OpenClawEventBroadcaster::new);

/// Advanced monitoring and health checks
#[derive(Clone)]
pub struct OpenClawHealthMonitor {
    pub pool: SqlitePool,
    pub last_health_check: Arc<RwLock<chrono::DateTime<Utc>>>,
    pub health_status: Arc<RwLock<HealthStatus>>,
}

#[derive(Clone, Debug)]
pub struct HealthStatus {
    pub overall: HealthLevel,
    pub database: HealthLevel,
    pub openclaw_config: HealthLevel,
    pub cache_performance: HealthLevel,
    pub sync_status: HealthLevel,
    pub last_check: chrono::DateTime<Utc>,
    pub issues: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Unhealthy,
}

impl OpenClawHealthMonitor {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            last_health_check: Arc::new(RwLock::new(Utc::now())),
            health_status: Arc::new(RwLock::new(HealthStatus {
                overall: HealthLevel::Healthy,
                database: HealthLevel::Healthy,
                openclaw_config: HealthLevel::Healthy,
                cache_performance: HealthLevel::Healthy,
                sync_status: HealthLevel::Healthy,
                last_check: Utc::now(),
                issues: Vec::new(),
            })),
        }
    }

    #[instrument(skip(self))]
    pub async fn perform_health_check(&self) -> HealthStatus {
        let start_time = std::time::Instant::now();
        let mut status = HealthStatus {
            overall: HealthLevel::Healthy,
            database: HealthLevel::Healthy,
            openclaw_config: HealthLevel::Healthy,
            cache_performance: HealthLevel::Healthy,
            sync_status: HealthLevel::Healthy,
            last_check: Utc::now(),
            issues: Vec::new(),
        };

        // Database health check
        match self.check_database_health().await {
            Ok(_) => {
                debug!("Database health check passed");
            }
            Err(e) => {
                warn!("Database health check failed: {}", e);
                status.database = HealthLevel::Unhealthy;
                status.issues.push(format!("Database: {}", e));
            }
        }

        // OpenClaw configuration health check
        match self.check_openclaw_config_health().await {
            Ok(_) => {
                debug!("OpenClaw config health check passed");
            }
            Err(e) => {
                warn!("OpenClaw config health check failed: {}", e);
                status.openclaw_config = HealthLevel::Degraded;
                status.issues.push(format!("OpenClaw config: {}", e));
            }
        }

        // Cache performance check
        match self.check_cache_performance().await {
            Ok(_) => {
                debug!("Cache performance check passed");
            }
            Err(e) => {
                warn!("Cache performance check failed: {}", e);
                status.cache_performance = HealthLevel::Degraded;
                status.issues.push(format!("Cache: {}", e));
            }
        }

        // Sync status check
        match self.check_sync_status().await {
            Ok(_) => {
                debug!("Sync status check passed");
            }
            Err(e) => {
                warn!("Sync status check failed: {}", e);
                status.sync_status = HealthLevel::Degraded;
                status.issues.push(format!("Sync: {}", e));
            }
        }

        // Calculate overall health
        status.overall = self.calculate_overall_health(&status);

        // Update stored status
        {
            let mut stored_status = self.health_status.write().await;
            *stored_status = status.clone();
        }
        {
            let mut last_check = self.last_health_check.write().await;
            *last_check = Utc::now();
        }

        let duration = start_time.elapsed();
        histogram!("openclaw_health_check_duration", duration.as_secs_f64());
        
        info!("Health check completed in {:?} with overall status: {:?}", duration, status.overall);
        status
    }

    async fn check_database_health(&self) -> Result<(), String> {
        let start = std::time::Instant::now();
        
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => {
                let duration = start.elapsed();
                if duration > Duration::from_millis(100) {
                    return Err(format!("Database response slow: {:?}", duration));
                }
                Ok(())
            }
            Err(e) => Err(format!("Database connection failed: {}", e)),
        }
    }

    async fn check_openclaw_config_health(&self) -> Result<(), String> {
        let openclaw_dir = std::env::var("OPENCLAW_STATE_DIR")
            .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.openclaw", h)))
            .unwrap_or_else(|_| "/root/.openclaw".to_string());
        let config_path = format!("{}/openclaw.json", openclaw_dir);

        // Check file existence and readability
        match tokio::fs::metadata(&config_path).await {
            Ok(metadata) => {
                if metadata.len() > 10 * 1024 * 1024 { // 10MB limit
                    return Err("Configuration file too large".to_string());
                }
                Ok(())
            }
            Err(e) => Err(format!("Cannot access config file: {}", e)),
        }
    }

    async fn check_cache_performance(&self) -> Result<(), String> {
        let start = std::time::Instant::now();
        
        // Test cache performance
        let test_key = "health_check_test";
        let test_value = serde_json::json!({"test": true});
        
        CONFIG_CACHE.put(test_key.to_string(), test_value.clone(), Duration::from_secs(1)).await;
        
        let cached = CONFIG_CACHE.get(test_key).await;
        let duration = start.elapsed();
        
        if cached.is_none() {
            return Err("Cache retrieval failed".to_string());
        }
        
        if duration > Duration::from_millis(10) {
            return Err(format!("Cache response slow: {:?}", duration));
        }
        
        Ok(())
    }

    async fn check_sync_status(&self) -> Result<(), String> {
        // Check recent sync operations
        let recent_syncs = sqlx::query_as::<sqlx::Sqlite, (String, String)>(
            "SELECT agent_id, config_hash FROM agents WHERE openclaw_config_hash IS NOT NULL LIMIT 10"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to query sync status: {}", e))?;

        if recent_syncs.is_empty() {
            return Err("No agents have been synced".to_string());
        }

        Ok(())
    }

    fn calculate_overall_health(&self, status: &HealthStatus) -> HealthLevel {
        let health_levels = [
            &status.database,
            &status.openclaw_config,
            &status.cache_performance,
            &status.sync_status,
        ];

        let unhealthy_count = health_levels.iter().filter(|&&level| level == HealthLevel::Unhealthy).count();
        let degraded_count = health_levels.iter().filter(|&&level| level == HealthLevel::Degraded).count();

        if unhealthy_count > 0 {
            HealthLevel::Unhealthy
        } else if degraded_count > 1 {
            HealthLevel::Degraded
        } else {
            HealthLevel::Healthy
        }
    }

    pub async fn get_current_status(&self) -> HealthStatus {
        self.health_status.read().await.clone()
    }
}

/// Enhanced API endpoints with monitoring and events

/// Get real-time events via Server-Sent Events
pub async fn get_openclaw_events(
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let subscriber_id = params.get("subscriber_id")
        .unwrap_or(&uuid::Uuid::new_v4().to_string())
        .clone();

    let mut rx = EVENT_BROADCASTER.subscribe(subscriber_id).await;

    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            let event_json = serde_json::to_string(&event).unwrap_or_default();
            yield format!("data: {}\n\n", event_json);
        }
    };

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(axum::body::Body::from_stream(stream))
        .unwrap()
}

/// Get comprehensive health status
pub async fn get_openclaw_health(
    State(state): State<crate::AppState>,
) -> Result<Json<HealthStatus>, (StatusCode, String)> {
    let monitor = OpenClawHealthMonitor::new(state.pool.clone());
    let status = monitor.perform_health_check().await;
    Ok(Json(status))
}

/// Get performance metrics
pub async fn get_openclaw_metrics(
    State(state): State<crate::AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let monitor = OpenClawHealthMonitor::new(state.pool.clone());
    let health_status = monitor.get_current_status().await;
    
    // Get cache statistics
    let cache_stats = get_cache_statistics().await;
    
    // Get event statistics
    let event_stats = get_event_statistics().await;

    Ok(Json(serde_json::json!({
        "health": health_status,
        "cache": cache_stats,
        "events": event_stats,
        "timestamp": Utc::now()
    })))
}

/// Trigger configuration refresh with event broadcasting
pub async fn refresh_openclaw_config(
    State(state): State<crate::AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let start_time = std::time::Instant::now();
    
    // Broadcast sync start event
    EVENT_BROADCASTER.broadcast(ConfigSyncEvent {
        event_type: SyncEventType::SyncStarted,
        agent_id: "all".to_string(),
        config_hash: "".to_string(),
        timestamp: Utc::now(),
        data: None,
    }).await;

    // Perform the refresh
    match sync_openclaw_configs(State(state.clone())).await {
        Ok(result) => {
            let duration = start_time.elapsed();
            
            // Broadcast success event
            EVENT_BROADCASTER.broadcast(ConfigSyncEvent {
                event_type: SyncEventType::SyncCompleted,
                agent_id: "all".to_string(),
                config_hash: format!("{:x}", Sha256::digest(duration.as_nanos().to_string().as_bytes())),
                timestamp: Utc::now(),
                data: Some(serde_json::to_value(&result).unwrap_or_default()),
            }).await;

            Ok(Json(serde_json::json!({
                "status": "success",
                "duration_ms": duration.as_millis(),
                "synced": result["synced"],
                "errors": result["errors"]
            })))
        }
        Err(e) => {
            // Broadcast failure event
            EVENT_BROADCASTER.broadcast(ConfigSyncEvent {
                event_type: SyncEventType::SyncFailed,
                agent_id: "all".to_string(),
                config_hash: "".to_string(),
                timestamp: Utc::now(),
                data: Some(serde_json::json!({"error": e.1})),
            }).await;

            Err(e)
        }
    }
}

/// Enhanced agent update with event broadcasting
pub async fn update_agent_parameters_with_events(
    Path(agent_id): Path<String>,
    State(state): State<crate::AppState>,
    Json(params): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Validate agent ID
    SecurityValidator::validate_agent_id(&agent_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Get current state for comparison
    let current_agent = sqlx::query_as::<sqlx::Sqlite, Agent>(
        "SELECT * FROM agents WHERE id = ?"
    )
    .bind(&agent_id)
    .fetch_one(&state.pool)
    .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    // Broadcast update start event
    EVENT_BROADCASTER.broadcast(ConfigSyncEvent {
        event_type: SyncEventType::ConfigChanged,
        agent_id: agent_id.clone(),
        config_hash: current_agent.openclaw_config_hash.unwrap_or_default(),
        timestamp: Utc::now(),
        data: Some(serde_json::json!({
            "action": "update_started",
            "parameters": params
        })),
    }).await;

    // Perform the update
    match update_agent_parameters_original(Path(agent_id.clone()), State(state), Json(params)).await {
        Ok(result) => {
            // Get updated agent for new hash
            let updated_agent = sqlx::query_as::<sqlx::Sqlite, Agent>(
                "SELECT * FROM agents WHERE id = ?"
            )
            .bind(&agent_id)
            .fetch_one(&state.pool)
            .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            // Broadcast success event
            EVENT_BROADCASTER.broadcast(ConfigSyncEvent {
                event_type: SyncEventType::ConfigChanged,
                agent_id: agent_id.clone(),
                config_hash: updated_agent.openclaw_config_hash.unwrap_or_default(),
                timestamp: Utc::now(),
                data: Some(serde_json::json!({
                    "action": "update_completed",
                    "changes_made": result["changes_made"]
                })),
            }).await;

            Ok(result)
        }
        Err(e) => {
            // Broadcast failure event
            EVENT_BROADCASTER.broadcast(ConfigSyncEvent {
                event_type: SyncEventType::SyncFailed,
                agent_id: agent_id.clone(),
                config_hash: current_agent.openclaw_config_hash.unwrap_or_default(),
                timestamp: Utc::now(),
                data: Some(serde_json::json!({
                    "action": "update_failed",
                    "error": e.1
                })),
            }).await;

            Err(e)
        }
    }
}

// Helper functions for monitoring

async fn get_cache_statistics() -> serde_json::Value {
    // This would require access to cache internals
    // For now, return placeholder data
    serde_json::json!({
        "cache_size": "unknown",
        "hit_rate": "unknown",
        "evictions": "unknown"
    })
}

async fn get_event_statistics() -> serde_json::Value {
    let events = EVENT_BROADCASTER.get_event_history(Some(1000)).await;
    
    let mut event_counts = HashMap::new();
    for event in &events {
        let count = event_counts.entry(format!("{:?}", event.event_type)).or_insert(0);
        *count += 1;
    }

    serde_json::json!({
        "total_events": events.len(),
        "event_types": event_counts,
        "active_subscribers": EVENT_BROADCASTER.subscribers.len(),
        "latest_event": events.last()
    })
}

// Middleware for monitoring

pub fn monitoring_middleware() -> axum::middleware::FromFn<impl Fn(axum::extract::Request, axum::middleware::Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = axum::response::Response> + Send>>> {
    axum::middleware::from_fn(|request: axum::extract::Request, next: axum::middleware::Next| async move {
        let start = std::time::Instant::now();
        let method = request.method().clone();
        let uri = request.uri().clone();
        
        let response = next.run(request).await;
        
        let duration = start.elapsed();
        let status = response.status();
        
        // Record metrics
        let route = format!("{} {}", method, uri.path());
        histogram!("http_request_duration", duration.as_secs_f64(), "route" => route, "status" => status.as_u16().to_string());
        
        if status.is_server_error() {
            counter!("http_server_errors_total", "route" => route);
        }
        
        response
    })
}
