use crate::models::*;
use axum::{
    extract::{Path, State, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use sqlx::SqlitePool;
use chrono::{Utc, Duration};
use std::collections::HashMap;
use serde_json::Value;
use sha2::{Sha256, Digest};
use std::sync::Arc;
use tracing::{info, warn, error, debug, instrument};
use validator::Validate;
use serde::{Deserialize, Serialize};

// Advanced Agent Management System

/// Comprehensive agent management with enhanced UX features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManagementRequest {
    pub agent: AgentConfigRequest,
    pub preferences: AgentManagementPreferences,
    pub validation_options: ValidationOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigRequest {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub workspace: Option<String>,
    pub agent_dir: Option<String>,
    pub model_config: ModelConfigurationRequest,
    pub capabilities: AgentCapabilities,
    pub behavior_settings: AgentBehaviorSettings,
    pub resource_limits: ResourceLimits,
    pub security_settings: SecuritySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfigurationRequest {
    pub primary_model: String,
    pub fallback_models: Vec<String>,
    pub image_model: Option<String>,
    pub model_params: HashMap<String, Value>,
    pub thinking_level: ThinkingLevel,
    pub verbose_level: VerboseLevel,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub skills: Vec<String>,
    pub tools_enabled: ToolCapabilities,
    pub features_enabled: FeatureCapabilities,
    pub integrations: Vec<String>,
    pub custom_capabilities: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub exec_tools: bool,
    pub file_operations: bool,
    pub web_access: bool,
    pub api_calls: bool,
    pub database_access: bool,
    pub system_commands: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCapabilities {
    pub memory_search: bool,
    pub heartbeat: bool,
    pub human_delay: bool,
    pub subagents: bool,
    pub block_streaming: bool,
    pub context_pruning: bool,
    pub auto_save: bool,
    pub collaborative_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBehaviorSettings {
    pub personality: AgentPersonality,
    pub communication_style: CommunicationStyle,
    pub response_preferences: ResponsePreferences,
    pub working_hours: WorkingHours,
    pub interaction_patterns: InteractionPatterns,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPersonality {
    pub tone: String, // professional, casual, friendly, formal
    pub expertise_level: String, // beginner, intermediate, expert
    pub specialization: Vec<String>,
    pub language_preference: String,
    pub cultural_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    pub response_length: String, // concise, detailed, comprehensive
    pub technical_level: String, // simple, moderate, technical
    pub code_style: String, // minimal, documented, commented
    pub explanation_style: String, // step_by_step, conceptual, practical
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePreferences {
    pub include_confidence: bool,
    pub include_reasoning: bool,
    pub include_alternatives: bool,
    pub include_sources: bool,
    pub format_preference: String, // markdown, json, plain_text
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHours {
    pub timezone: String,
    pub active_hours: ActiveHoursConfig,
    pub break_schedule: Vec<BreakSchedule>,
    pub availability_calendar: HashMap<String, Vec<TimeSlot>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveHoursConfig {
    pub monday: DaySchedule,
    pub tuesday: DaySchedule,
    pub wednesday: DaySchedule,
    pub thursday: DaySchedule,
    pub friday: DaySchedule,
    pub saturday: DaySchedule,
    pub sunday: DaySchedule,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakSchedule {
    pub name: String,
    pub start_time: String,
    pub duration_minutes: u32,
    pub recurring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub start: String,
    pub end: String,
    pub available: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionPatterns {
    pub greeting_style: String,
    pub farewell_style: String,
    pub error_handling: ErrorHandlingStyle,
    pub clarification_preferences: ClarificationPreferences,
    pub feedback_requests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingStyle {
    pub approach: String, // apologetic, professional, casual
    pub offer_solutions: bool,
    pub request_clarification: bool,
    pub escalation_threshold: u8, // 0-10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationPreferences {
    pub ask_questions: bool,
    pub confirm_understanding: bool,
    pub provide_examples: bool,
    pub check_assumptions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_concurrent_tasks: u32,
    pub max_memory_mb: u32,
    pub max_execution_time_minutes: u32,
    pub max_file_size_mb: u32,
    pub max_api_calls_per_hour: u32,
    pub cost_limits: CostLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostLimits {
    pub daily_limit: Option<f64>,
    pub weekly_limit: Option<f64>,
    pub monthly_limit: Option<f64>,
    pub per_task_limit: Option<f64>,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub access_level: SecurityAccessLevel,
    pub data_permissions: DataPermissions,
    pub network_restrictions: NetworkRestrictions,
    pub audit_settings: AuditSettings,
    pub encryption_requirements: EncryptionRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAccessLevel {
    ReadOnly,
    ReadWrite,
    Administrator,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPermissions {
    pub can_read_sensitive: bool,
    pub can_write_sensitive: bool,
    pub can_delete_data: bool,
    pub can_share_data: bool,
    pub data_retention_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRestrictions {
    pub allowed_domains: Vec<String>,
    pub blocked_domains: Vec<String>,
    pub require_https: bool,
    pub max_requests_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSettings {
    pub log_all_interactions: bool,
    pub log_data_access: bool,
    pub log_tool_usage: bool,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionRequirements {
    pub encrypt_data_at_rest: bool,
    pub encrypt_data_in_transit: bool,
    pub encryption_algorithm: String,
    pub key_rotation_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManagementPreferences {
    pub auto_save: bool,
    pub validation_level: ValidationLevel,
    pub notification_preferences: NotificationPreferences,
    pub ui_preferences: UiPreferences,
    pub workflow_preferences: WorkflowPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationLevel {
    Strict,
    Standard,
    Relaxed,
    Custom(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_notifications: bool,
    pub webhook_notifications: bool,
    pub in_app_notifications: bool,
    pub notification_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub theme: String,
    pub language: String,
    pub timezone: String,
    pub dashboard_layout: String,
    pub default_view: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPreferences {
    pub auto_approve_changes: bool,
    pub require_review_for_critical: bool,
    pub parallel_processing: bool,
    pub batch_operations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOptions {
    pub check_syntax: bool,
    pub check_semantics: bool,
    pub check_security: bool,
    pub check_performance: bool,
    pub check_compatibility: bool,
    pub custom_validators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentComparisonRequest {
    pub agent_ids: Vec<String>,
    pub comparison_type: ComparisonType,
    pub metrics: Vec<String>,
    pub period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonType {
    Performance,
    Cost,
    Capabilities,
    Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentComparison {
    pub comparison_id: String,
    pub agents: Vec<AgentComparisonData>,
    pub rankings: ComparisonRankings,
    pub insights: Vec<ComparisonInsight>,
    pub recommendations: Vec<ComparisonRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentComparisonData {
    pub agent_id: String,
    pub metrics: HashMap<String, f64>,
    pub rank: u32,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonRankings {
    pub overall_ranking: Vec<String>,
    pub performance_ranking: Vec<String>,
    pub cost_ranking: Vec<String>,
    pub capability_ranking: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonInsight {
    pub title: String,
    pub description: String,
    pub agents_involved: Vec<String>,
    pub significance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonRecommendation {
    pub category: String,
    pub title: String,
    pub description: String,
    pub target_agents: Vec<String>,
    pub expected_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsageInsights {
    pub agent_id: String,
    pub usage_patterns: UsagePatterns,
    pub peak_times: Vec<TimeSlot>,
    pub common_tasks: Vec<TaskPattern>,
    pub efficiency_metrics: EfficiencyMetrics,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatterns {
    pub daily_pattern: Vec<f64>,
    pub weekly_pattern: Vec<f64>,
    pub monthly_pattern: Vec<f64>,
    pub seasonal_variations: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPattern {
    pub task_type: String,
    pub frequency: f64,
    pub average_duration: Duration,
    pub success_rate: f64,
    pub common_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub tasks_per_hour: f64,
    pub cost_per_task: f64,
    pub time_to_completion: Duration,
    pub resource_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub category: String,
    pub title: String,
    pub description: String,
    pub potential_savings: String,
    pub implementation_effort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAnalytics {
    pub agent_id: String,
    pub period: String,
    pub metrics: AnalyticsMetrics,
    pub trends: AnalyticsTrends,
    pub insights: Vec<AnalyticsInsight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMetrics {
    pub total_tasks: u64,
    pub success_rate: f64,
    pub average_duration: Duration,
    pub cost_analysis: CostAnalysis,
    pub resource_utilization: ResourceUtilization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysis {
    pub total_cost: f64,
    pub cost_per_task: f64,
    pub cost_trend: String,
    pub budget_usage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu_usage: Vec<f64>,
    pub memory_usage: Vec<f64>,
    pub token_usage: Vec<u64>,
    pub api_calls: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsTrends {
    pub performance_trend: TrendDirection,
    pub cost_trend: TrendDirection,
    pub usage_trend: TrendDirection,
    pub satisfaction_trend: TrendDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsInsight {
    pub category: String,
    pub title: String,
    pub description: String,
    pub impact: String,
    pub recommendation: String,
}

// Enhanced Agent Management Endpoints

/// Create or update agent with comprehensive configuration
#[instrument(skip(state))]
pub async fn create_or_update_agent_comprehensive(
    State(state): State<crate::AppState>,
    Json(request): Json<AgentManagementRequest>,
) -> Result<Json<AgentManagementResponse>, (StatusCode, String)> {
    // Validate the request
    request.validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)))?;

    // Check if agent exists
    let existing_agent = sqlx::query_as::<sqlx::Sqlite, Agent>(
        "SELECT * FROM agents WHERE id = ?"
    )
    .bind(&request.agent.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let is_update = existing_agent.is_some();

    // Process the agent configuration
    let result = if is_update {
        update_agent_comprehensive(&state.pool, &request).await?
    } else {
        create_agent_comprehensive(&state.pool, &request).await?
    };

    // Broadcast the change event
    crate::openclaw_monitoring::EVENT_BROADCASTER.broadcast(
        crate::openclaw_monitoring::ConfigSyncEvent {
            event_type: if is_update {
                crate::openclaw_monitoring::SyncEventType::ConfigChanged
            } else {
                crate::openclaw_monitoring::SyncEventType::AgentAdded
            },
            agent_id: request.agent.id.clone(),
            config_hash: result.config_hash.clone(),
            timestamp: Utc::now(),
            data: Some(serde_json::to_value(&request.agent).unwrap_or_default()),
        }
    ).await;

    Ok(Json(result))
}

/// Get comprehensive agent information with all settings
#[instrument(skip(state))]
pub async fn get_agent_comprehensive(
    Path(agent_id): Path<String>,
    State(state): State<crate::AppState>,
) -> Result<Json<ComprehensiveAgentInfo>, (StatusCode, String)> {
    // Validate agent ID
    crate::openclaw_integration::SecurityValidator::validate_agent_id(&agent_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Get basic agent info
    let agent = sqlx::query_as::<sqlx::Sqlite, Agent>(
        "SELECT * FROM agents WHERE id = ?"
    )
    .bind(&agent_id)
    .fetch_one(&state.pool)
    .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    // Get comprehensive configuration
    let config = get_agent_comprehensive_config(&state.pool, &agent_id).await?;
    
    // Get performance metrics
    let metrics = get_agent_performance_metrics(&state.pool, &agent_id).await?;
    
    // Get recent activity
    let activity = get_agent_recent_activity(&state.pool, &agent_id).await?;

    // Get capabilities analysis
    let capabilities = analyze_agent_capabilities(&agent, &config).await?;

    Ok(Json(ComprehensiveAgentInfo {
        basic_info: agent,
        configuration: config,
        performance_metrics: metrics,
        recent_activity: activity,
        capabilities_analysis: capabilities,
        recommendations: generate_agent_recommendations(&agent, &config, &metrics),
        health_status: calculate_agent_health_status(&agent, &config, &metrics),
    }))
}

/// Clone agent with customization options
#[instrument(skip(state))]
pub async fn clone_agent(
    Path(agent_id): Path<String>,
    State(state): State<crate::AppState>,
    Json(clone_options): Json<AgentCloneOptions>,
) -> Result<Json<AgentManagementResponse>, (StatusCode, String)> {
    // Validate source agent
    crate::openclaw_integration::SecurityValidator::validate_agent_id(&agent_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Get source agent configuration
    let source_config = get_agent_comprehensive_config(&state.pool, &agent_id).await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Source agent not found: {}", e)))?;

    // Create clone with modifications
    let clone_request = create_clone_request(&source_config, &clone_options)?;
    
    // Validate clone request
    clone_request.validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Clone validation error: {}", e)))?;

    // Create the cloned agent
    let result = create_agent_comprehensive(&state.pool, &clone_request).await?;

    // Broadcast clone event
    crate::openclaw_monitoring::EVENT_BROADCASTER.broadcast(
        crate::openclaw_monitoring::ConfigSyncEvent {
            event_type: crate::openclaw_monitoring::SyncEventType::AgentAdded,
            agent_id: clone_request.agent.id.clone(),
            config_hash: result.config_hash.clone(),
            timestamp: Utc::now(),
            data: Some(serde_json::json!({
                "action": "cloned_from",
                "source_agent": agent_id,
                "clone_options": clone_options
            })),
        }
    ).await;

    Ok(Json(result))
}

/// Get agent recommendations based on usage patterns
#[instrument(skip(state))]
pub async fn get_agent_recommendations(
    Path(agent_id): Path<String>,
    State(state): State<crate::AppState>,
) -> Result<Json<AgentRecommendations>, (StatusCode, String)> {
    // Validate agent ID
    crate::openclaw_integration::SecurityValidator::validate_agent_id(&agent_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Get agent info and metrics
    let agent = sqlx::query_as::<sqlx::Sqlite, Agent>(
        "SELECT * FROM agents WHERE id = ?"
    )
    .bind(&agent_id)
    .fetch_one(&state.pool)
    .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let config = get_agent_comprehensive_config(&state.pool, &agent_id).await?;
    let metrics = get_agent_performance_metrics(&state.pool, &agent_id).await?;

    let recommendations = generate_agent_recommendations(&agent, &config, &metrics);

    Ok(Json(recommendations))
}

/// Bulk agent operations
#[instrument(skip(state))]
pub async fn bulk_agent_operations(
    State(state): State<crate::AppState>,
    Json(operation): Json<BulkAgentOperation>,
) -> Result<Json<BulkOperationResult>, (StatusCode, String)> {
    let mut results = Vec::new();
    let mut success_count = 0;
    let mut error_count = 0;

    for agent_id in &operation.agent_ids {
        let result = match operation.operation_type.as_str() {
            "enable" => toggle_agent_status(&state.pool, agent_id, true).await,
            "disable" => toggle_agent_status(&state.pool, agent_id, false).await,
            "reset" => reset_agent_configuration(&state.pool, agent_id).await,
            "optimize" => optimize_agent_configuration(&state.pool, agent_id).await,
            "validate" => validate_agent_configuration(&state.pool, agent_id).await,
            _ => Err("Invalid operation type".to_string()),
        };

        match result {
            Ok(_) => {
                success_count += 1;
                results.push(BulkOperationItem {
                    agent_id: agent_id.clone(),
                    status: "success".to_string(),
                    message: None,
                });
            }
            Err(e) => {
                error_count += 1;
                results.push(BulkOperationItem {
                    agent_id: agent_id.clone(),
                    status: "error".to_string(),
                    message: Some(e),
                });
            }
        }
    }

    Ok(Json(BulkOperationResult {
        operation_id: uuid::Uuid::new_v4().to_string(),
        total_agents: operation.agent_ids.len(),
        success_count,
        error_count,
        results,
        duration_ms: 0, // Would be calculated in real implementation
    }))
}

/// Get agent templates
#[instrument(skip(state))]
pub async fn get_agent_templates(
    State(state): State<crate::AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<AgentTemplate>>, (StatusCode, String)> {
    let category = params.get("category").cloned();
    let role = params.get("role").cloned();

    let templates = get_agent_templates_filtered(&state.pool, category, role).await?;

    Ok(Json(templates))
}

/// Create agent from template
#[instrument(skip(state))]
pub async fn create_agent_from_template(
    State(state): State<crate::AppState>,
    Json(request): Json<CreateFromTemplateRequest>,
) -> Result<Json<AgentManagementResponse>, (StatusCode, String)> {
    // Get template
    let template = get_agent_template(&state.pool, &request.template_id).await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Template not found: {}", e)))?;

    // Create agent from template with customizations
    let agent_request = create_agent_request_from_template(&template, &request.customizations)?;
    
    // Validate and create
    agent_request.validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)))?;

    let result = create_agent_comprehensive(&state.pool, &agent_request).await?;

    // Broadcast creation event
    crate::openclaw_monitoring::EVENT_BROADCASTER.broadcast(
        crate::openclaw_monitoring::ConfigSyncEvent {
            event_type: crate::openclaw_monitoring::SyncEventType::AgentAdded,
            agent_id: agent_request.agent.id.clone(),
            config_hash: result.config_hash.clone(),
            timestamp: Utc::now(),
            data: Some(serde_json::json!({
                "action": "created_from_template",
                "template_id": request.template_id,
                "template_name": template.name
            })),
        }
    ).await;

    Ok(Json(result))
}

// Response Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManagementResponse {
    pub agent_id: String,
    pub status: String,
    pub message: String,
    pub config_hash: String,
    pub created_at: chrono::DateTime<Utc>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveAgentInfo {
    pub basic_info: Agent,
    pub configuration: AgentComprehensiveConfig,
    pub performance_metrics: AgentPerformanceMetrics,
    pub recent_activity: Vec<AgentActivity>,
    pub capabilities_analysis: AgentCapabilitiesAnalysis,
    pub recommendations: AgentRecommendations,
    pub health_status: AgentHealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentComprehensiveConfig {
    pub model_config: ModelConfigurationRequest,
    pub capabilities: AgentCapabilities,
    pub behavior_settings: AgentBehaviorSettings,
    pub resource_limits: ResourceLimits,
    pub security_settings: SecuritySettings,
    pub openclaw_integration: OpenClawIntegrationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawIntegrationConfig {
    pub sandbox_config: Option<crate::models::SandboxConfig>,
    pub tools_config: Option<crate::models::ToolsConfig>,
    pub memory_search_config: Option<crate::models::MemorySearchConfig>,
    pub heartbeat_config: Option<crate::models::HeartbeatConfig>,
    pub subagents_config: Option<crate::models::SubagentsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceMetrics {
    pub total_tasks_completed: u64,
    pub average_task_duration: Duration,
    pub success_rate: f64,
    pub error_rate: f64,
    pub resource_usage: ResourceUsageMetrics,
    pub model_performance: ModelPerformanceMetrics,
    pub user_satisfaction: Option<f64>,
    pub cost_efficiency: CostEfficiencyMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageMetrics {
    pub average_memory_usage_mb: f64,
    pub peak_memory_usage_mb: f64,
    pub average_cpu_usage_percent: f64,
    pub peak_cpu_usage_percent: f64,
    pub api_calls_made: u64,
    pub files_processed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    pub average_response_time_ms: f64,
    pub token_usage_total: u64,
    pub average_tokens_per_task: f64,
    pub model_switches: u64,
    pub fallback_usage_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEfficiencyMetrics {
    pub total_cost: f64,
    pub cost_per_task: f64,
    pub cost_per_token: f64,
    pub budget_utilization_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActivity {
    pub id: String,
    pub activity_type: String,
    pub description: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub duration: Option<Duration>,
    pub success: bool,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilitiesAnalysis {
    pub total_capabilities: u32,
    pub enabled_capabilities: u32,
    pub capability_usage_stats: HashMap<String, CapabilityUsage>,
    pub recommended_capabilities: Vec<String>,
    pub underutilized_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityUsage {
    pub capability: String,
    pub usage_count: u64,
    pub success_rate: f64,
    pub average_duration: Duration,
    pub last_used: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecommendations {
    pub performance_improvements: Vec<Recommendation>,
    pub configuration_optimizations: Vec<Recommendation>,
    pub capability_enhancements: Vec<Recommendation>,
    pub cost_optimizations: Vec<Recommendation>,
    pub security_improvements: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub estimated_impact: String,
    pub implementation_difficulty: ImplementationDifficulty,
    pub auto_applicable: bool,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationDifficulty {
    Trivial,
    Easy,
    Medium,
    Hard,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealthStatus {
    pub overall_health: f64, // 0-100
    pub performance_health: f64,
    pub configuration_health: f64,
    pub security_health: f64,
    pub resource_health: f64,
    pub last_check: chrono::DateTime<Utc>,
    pub health_trend: HealthTrend,
    pub issues: Vec<HealthIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthTrend {
    Improving,
    Stable,
    Declining,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub id: String,
    pub category: String,
    pub severity: IssueSeverity,
    pub title: String,
    pub description: String,
    pub detected_at: chrono::DateTime<Utc>,
    pub auto_resolvable: bool,
    pub resolution_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

// Additional request/response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCloneOptions {
    pub new_id: String,
    pub new_name: String,
    pub copy_configuration: bool,
    pub copy_performance_data: bool,
    pub reset_statistics: bool,
    pub modify_fields: HashMap<String, Value>,
    pub exclude_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkAgentOperation {
    pub operation_id: String,
    pub operation_type: String,
    pub agent_ids: Vec<String>,
    pub parameters: Option<HashMap<String, Value>>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResult {
    pub operation_id: String,
    pub total_agents: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub results: Vec<BulkOperationItem>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationItem {
    pub agent_id: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub role: AgentRole,
    pub configuration: AgentConfigRequest,
    pub usage_count: u64,
    pub rating: Option<f64>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFromTemplateRequest {
    pub template_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub customizations: HashMap<String, Value>,
    pub apply_recommendations: bool,
}

// Helper functions (implementations would go here)

pub async fn create_agent_comprehensive(
    pool: &SqlitePool,
    request: &AgentManagementRequest,
) -> Result<AgentManagementResponse, Box<dyn std::error::Error + Send + Sync>> {
    // Implementation would create agent with all comprehensive settings
    todo!("Implement comprehensive agent creation")
}

async fn update_agent_comprehensive(
    pool: &SqlitePool,
    request: &AgentManagementRequest,
) -> Result<AgentManagementResponse, Box<dyn std::error::Error + Send + Sync>> {
    // Implementation would update agent with all comprehensive settings
    todo!("Implement comprehensive agent update")
}

async fn get_agent_comprehensive_config(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<AgentComprehensiveConfig, Box<dyn std::error::Error + Send + Sync>> {
    // Implementation would retrieve comprehensive agent configuration
    todo!("Implement comprehensive config retrieval")
}

async fn get_agent_performance_metrics(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<AgentPerformanceMetrics, Box<dyn std::error::Error + Send + Sync>> {
    // Implementation would calculate performance metrics
    todo!("Implement performance metrics calculation")
}

async fn get_agent_recent_activity(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<Vec<AgentActivity>, Box<dyn std::error::Error + Send + Sync>> {
    // Implementation would retrieve recent activity
    todo!("Implement recent activity retrieval")
}

async fn analyze_agent_capabilities(
    agent: &Agent,
    config: &AgentComprehensiveConfig,
) -> Result<AgentCapabilitiesAnalysis, Box<dyn std::error::Error + Send + Sync>> {
    // Implementation would analyze agent capabilities
    todo!("Implement capabilities analysis")
}

fn generate_agent_recommendations(
    agent: &Agent,
    config: &AgentComprehensiveConfig,
    metrics: &AgentPerformanceMetrics,
) -> AgentRecommendations {
    // Implementation would generate intelligent recommendations
    todo!("Implement recommendation generation")
}

fn calculate_agent_health_status(
    agent: &Agent,
    config: &AgentComprehensiveConfig,
    metrics: &AgentPerformanceMetrics,
) -> AgentHealthStatus {
    // Implementation would calculate comprehensive health status
    todo!("Implement health status calculation")
}

fn create_clone_request(
    source_config: &AgentComprehensiveConfig,
    options: &AgentCloneOptions,
) -> Result<AgentManagementRequest, String> {
    // Implementation would create clone request with modifications
    todo!("Implement clone request creation")
}

async fn toggle_agent_status(
    pool: &SqlitePool,
    agent_id: &str,
    enabled: bool,
) -> Result<(), String> {
    // Implementation would toggle agent status
    todo!("Implement agent status toggle")
}

async fn reset_agent_configuration(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<(), String> {
    // Implementation would reset agent configuration
    todo!("Implement configuration reset")
}

async fn optimize_agent_configuration(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<(), String> {
    // Implementation would optimize agent configuration
    todo!("Implement configuration optimization")
}

async fn validate_agent_configuration(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<(), String> {
    // Implementation would validate agent configuration
    todo!("Implement configuration validation")
}

async fn get_agent_templates_filtered(
    pool: &SqlitePool,
    category: Option<String>,
    role: Option<String>,
) -> Result<Vec<AgentTemplate>, Box<dyn std::error::Error + Send + Sync>> {
    // Implementation would retrieve filtered templates
    todo!("Implement template retrieval")
}

async fn get_agent_template(
    pool: &SqlitePool,
    template_id: &str,
) -> Result<AgentTemplate, Box<dyn std::error::Error + Send + Sync>> {
    // Implementation would retrieve specific template
    todo!("Implement template retrieval")
}

fn create_agent_request_from_template(
    template: &AgentTemplate,
    customizations: &HashMap<String, Value>,
) -> Result<AgentManagementRequest, String> {
    // Implementation would create agent request from template
    todo!("Implement agent request creation from template")
}
