use crate::agent_management::*;
use axum::{
    extract::{Path, State, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// User-friendly API improvements

#[derive(Debug, Serialize, Deserialize)]
pub struct QuickAgentRequest {
    pub name: String,
    pub purpose: String,
    pub expertise: Option<String>, // "coding", "writing", "analysis", etc.
    pub complexity: Option<String>, // "simple", "standard", "advanced"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuickAgentResponse {
    pub agent_id: String,
    pub name: String,
    pub status: String,
    pub quick_summary: String,
    pub next_steps: Vec<String>,
    pub estimated_setup_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationWizardRequest {
    pub step: u32,
    pub agent_id: Option<String>,
    pub answers: HashMap<String, serde_json::Value>,
    pub previous_data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationWizardResponse {
    pub step: u32,
    pub total_steps: u32,
    pub question: WizardQuestion,
    pub progress: f32,
    pub next_action: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WizardQuestion {
    pub id: String,
    pub title: String,
    pub description: String,
    pub question_type: String, // "text", "choice", "multichoice", "boolean", "number"
    pub options: Option<Vec<WizardOption>>,
    pub default_value: Option<serde_json::Value>,
    pub validation: Option<ValidationRule>,
    pub help_text: Option<String>,
    pub examples: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WizardOption {
    pub value: serde_json::Value,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub recommended: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationRule {
    pub required: bool,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    pub pattern: Option<String>,
    pub custom_validator: Option<String>,
}

// Simplified endpoints for better UX

/// Quick agent creation with smart defaults
pub async fn create_agent_quick(
    State(state): State<crate::AppState>,
    Json(request): Json<QuickAgentRequest>,
) -> Result<Json<QuickAgentResponse>, (StatusCode, String)> {
    let agent_id = format!("agent-{}", uuid::Uuid::new_v4().to_string()[..8]);
    
    // Generate smart configuration based on purpose
    let config = generate_smart_config(&request);
    
    // Create agent with generated configuration
    let full_request = AgentManagementRequest {
        agent: config,
        preferences: AgentManagementPreferences {
            auto_save: true,
            validation_level: ValidationLevel::Standard,
            notification_preferences: NotificationPreferences {
                email_notifications: false,
                webhook_notifications: false,
                in_app_notifications: true,
                notification_events: vec!["created".to_string(), "updated".to_string()],
            },
            ui_preferences: UiPreferences {
                theme: "light".to_string(),
                language: "en".to_string(),
                timezone: "UTC".to_string(),
                dashboard_layout: "grid".to_string(),
                default_view: "overview".to_string(),
            },
            workflow_preferences: WorkflowPreferences {
                auto_approve_changes: false,
                require_review_for_critical: true,
                parallel_processing: true,
                batch_operations: false,
            },
        },
        validation_options: ValidationOptions {
            check_syntax: true,
            check_semantics: true,
            check_security: true,
            check_performance: false,
            check_compatibility: true,
            custom_validators: vec![],
        },
    };

    // Create the agent
    let result = crate::agent_management_impl::create_or_update_agent_comprehensive(
        State(state), 
        Json(full_request)
    ).await?;

    Ok(Json(QuickAgentResponse {
        agent_id: result.agent_id.clone(),
        name: request.name,
        status: "created".to_string(),
        quick_summary: format!("{} agent created successfully for {}", request.purpose, request.name),
        next_steps: vec![
            "Test your agent with a simple task".to_string(),
            "Review configuration in dashboard".to_string(),
            "Customize advanced settings if needed".to_string(),
        ],
        estimated_setup_time: "2 minutes".to_string(),
    }))
}

/// Configuration wizard for step-by-step setup
pub async fn configuration_wizard(
    State(state): State<crate::AppState>,
    Json(request): Json<ConfigurationWizardRequest>,
) -> Result<Json<ConfigurationWizardResponse>, (StatusCode, String)> {
    let wizard_steps = get_wizard_steps();
    
    if request.step == 0 {
        // Start wizard
        return Ok(Json(ConfigurationWizardResponse {
            step: 1,
            total_steps: wizard_steps.len() as u32,
            question: wizard_steps[0].clone(),
            progress: 0.0,
            next_action: "answer_question".to_string(),
            data: serde_json::json!({}),
        }));
    }

    if request.step > wizard_steps.len() as u32 {
        return Err((StatusCode::BAD_REQUEST, "Invalid step".to_string()));
    }

    // Process current step answer
    let mut data = request.previous_data.unwrap_or(serde_json::json!({}));
    data[&wizard_steps[request.step as usize - 1].id] = request.answers;

    // Validate answer
    if let Some(validation) = &wizard_steps[request.step as usize - 1].validation {
        if !validate_answer(&request.answers, validation) {
            return Err((StatusCode::BAD_REQUEST, "Invalid answer".to_string()));
        }
    }

    // Generate next step or complete
    if request.step == wizard_steps.len() as u32 {
        // Complete wizard and create agent
        let agent_config = build_config_from_wizard_data(&data);
        
        let full_request = AgentManagementRequest {
            agent: agent_config,
            preferences: AgentManagementPreferences {
                auto_save: true,
                validation_level: ValidationLevel::Standard,
                notification_preferences: NotificationPreferences {
                    email_notifications: false,
                    webhook_notifications: false,
                    in_app_notifications: true,
                    notification_events: vec!["created".to_string()],
                },
                ui_preferences: UiPreferences {
                    theme: "light".to_string(),
                    language: "en".to_string(),
                    timezone: "UTC".to_string(),
                    dashboard_layout: "grid".to_string(),
                    default_view: "overview".to_string(),
                },
                workflow_preferences: WorkflowPreferences {
                    auto_approve_changes: false,
                    require_review_for_critical: true,
                    parallel_processing: true,
                    batch_operations: false,
                },
            },
            validation_options: ValidationOptions {
                check_syntax: true,
                check_semantics: true,
                check_security: true,
                check_performance: false,
                check_compatibility: true,
                custom_validators: vec![],
            },
        };

        let result = crate::agent_management_impl::create_or_update_agent_comprehensive(
            State(state), 
            Json(full_request)
        ).await?;

        return Ok(Json(ConfigurationWizardResponse {
            step: request.step,
            total_steps: wizard_steps.len() as u32,
            question: WizardQuestion {
                id: "complete".to_string(),
                title: "Setup Complete!".to_string(),
                description: "Your agent has been created successfully.".to_string(),
                question_type: "complete".to_string(),
                options: None,
                default_value: None,
                validation: None,
                help_text: None,
                examples: None,
            },
            progress: 1.0,
            next_action: "view_agent".to_string(),
            data: serde_json::json!({
                "agent_id": result.agent_id,
                "agent_name": result.agent_id,
                "summary": "Agent created successfully"
            }),
        }));
    }

    // Next step
    let next_step = request.step + 1;
    Ok(Json(ConfigurationWizardResponse {
        step: next_step,
        total_steps: wizard_steps.len() as u32,
        question: wizard_steps[next_step as usize - 1].clone(),
        progress: (next_step as f32) / (wizard_steps.len() as f32),
        next_action: "answer_question".to_string(),
        data,
    }))
}

/// Get contextual help for agent configuration
pub async fn get_configuration_help(
    Path(topic): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<HelpResponse>, (StatusCode, String)> {
    let context = params.get("context").cloned().unwrap_or_default();
    let help = generate_help_for_topic(&topic, &context);
    
    Ok(Json(help))
}

/// Get smart suggestions based on current configuration
pub async fn get_smart_suggestions(
    Path(agent_id): Path<String>,
    State(state): State<crate::AppState>,
) -> Result<Json<Vec<SmartSuggestion>>, (StatusCode, String)> {
    // Get current agent configuration
    let agent = sqlx::query_as::<sqlx::Sqlite, crate::models::Agent>(
        "SELECT * FROM agents WHERE id = ?"
    )
    .bind(&agent_id)
    .fetch_one(&state.pool)
    .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let suggestions = generate_smart_suggestions_for_agent(&agent);
    
    Ok(Json(suggestions))
}

/// Validate configuration with user-friendly feedback
pub async fn validate_configuration_friendly(
    State(state): State<crate::AppState>,
    Json(config): Json<serde_json::Value>,
) -> Result<Json<ValidationResult>, (StatusCode, String)> {
    let validation_result = validate_configuration_with_detailed_feedback(&config);
    
    Ok(Json(validation_result))
}

/// Get configuration templates with filtering and search
pub async fn get_templates_user_friendly(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<TemplatesResponse>, (StatusCode, String)> {
    let search = params.get("search").cloned();
    let category = params.get("category").cloned();
    let sort_by = params.get("sort_by").cloned().unwrap_or_else(|| "popular".to_string());
    
    let templates = get_templates_with_filters(search, category, &sort_by);
    
    Ok(Json(TemplatesResponse {
        templates,
        total: templates.len(),
        filters_used: vec![
            search.as_ref().map(|s| format!("search: {}", s)).unwrap_or_default(),
            category.as_ref().map(|c| format!("category: {}", c)).unwrap_or_default(),
            format!("sort: {}", sort_by),
        ],
    }))
}

/// Get agent comparison with visual insights
pub async fn compare_agents_visual(
    State(state): State<crate::AppState>,
    Json(request): Json<AgentComparisonRequest>,
) -> Result<Json<VisualComparison>, (StatusCode, String)> {
    let comparison = crate::agent_management_impl::perform_agent_comparison(&state.pool, &request).await?;
    
    let visual = create_visual_comparison(&comparison);
    
    Ok(Json(visual))
}

// Response types

#[derive(Debug, Serialize, Deserialize)]
pub struct HelpResponse {
    pub topic: String,
    pub title: String,
    pub content: String,
    pub examples: Vec<String>,
    pub related_topics: Vec<String>,
    pub difficulty: String,
    pub estimated_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmartSuggestion {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: SuggestionCategory,
    pub impact: String,
    pub effort: String,
    pub auto_applicable: bool,
    pub steps: Vec<String>,
    pub why_important: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Performance,
    Cost,
    Security,
    Usability,
    Capability,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub score: f64,
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
    pub suggestions: Vec<String>,
    pub auto_fixable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub field: String,
    pub severity: String,
    pub message: String,
    pub suggestion: String,
    pub auto_fixable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplatesResponse {
    pub templates: Vec<TemplateCard>,
    pub total: usize,
    pub filters_used: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateCard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub popularity_score: f64,
    pub setup_difficulty: String,
    pub estimated_time: String,
    pub preview: TemplatePreview,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplatePreview {
    pub model_config: String,
    pub key_features: Vec<String>,
    pub use_cases: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisualComparison {
    pub comparison_id: String,
    pub agents: Vec<VisualAgentData>,
    pub insights: Vec<VisualInsight>,
    pub recommendations: Vec<VisualRecommendation>,
    pub charts: Vec<Chart>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisualAgentData {
    pub agent_id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub scores: AgentScores,
    pub key_metrics: HashMap<String, f64>,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentScores {
    pub overall: f64,
    pub performance: f64,
    pub cost_efficiency: f64,
    pub capabilities: f64,
    pub reliability: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisualInsight {
    pub title: String,
    pub description: String,
    pub type_: String,
    pub importance: String,
    pub affected_agents: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisualRecommendation {
    pub title: String,
    pub description: String,
    pub category: String,
    pub target_agents: Vec<String>,
    pub expected_improvement: String,
    pub implementation_steps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chart {
    pub type_: String,
    pub title: String,
    pub data: Vec<ChartPoint>,
    pub x_axis: String,
    pub y_axis: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChartPoint {
    pub x: f64,
    pub y: f64,
    pub label: Option<String>,
}

// Helper functions

fn generate_smart_config(request: &QuickAgentRequest) -> AgentConfigRequest {
    let expertise = request.expertise.as_deref().unwrap_or("general");
    let complexity = request.complexity.as_deref().unwrap_or("standard");
    
    // Smart defaults based on purpose and expertise
    let (model, thinking, verbose) = match (expertise, complexity.as_str()) {
        ("coding", "simple") => ("anthropic/claude-3-haiku", ThinkingLevel::Low, VerboseLevel::On),
        ("coding", "standard") => ("anthropic/claude-3-sonnet", ThinkingLevel::Medium, VerboseLevel::On),
        ("coding", "advanced") => ("anthropic/claude-3-opus", ThinkingLevel::High, VerboseLevel::Full),
        ("writing", "simple") => ("anthropic/claude-3-sonnet", ThinkingLevel::Low, VerboseLevel::Full),
        ("writing", "standard") => ("anthropic/claude-3-sonnet", ThinkingLevel::Medium, VerboseLevel::Full),
        ("writing", "advanced") => ("anthropic/claude-3-opus", ThinkingLevel::High, VerboseLevel::Full),
        _ => ("anthropic/claw-3-sonnet", ThinkingLevel::Medium, VerboseLevel::On),
    };

    AgentConfigRequest {
        id: format!("agent-{}", uuid::Uuid::new_v4().to_string()[..8]),
        name: request.name.clone(),
        role: AgentRole::Spc,
        description: Some(format!("{} agent for {}", request.name, request.purpose)),
        avatar: None,
        workspace: None,
        agent_dir: None,
        model_config: ModelConfigurationRequest {
            primary_model: model.to_string(),
            fallback_models: vec!["openai/gpt-4".to_string()],
            image_model: None,
            model_params: HashMap::new(),
            thinking_level,
            verbose_level,
            temperature: Some(0.3),
            max_tokens: Some(2000),
        },
        capabilities: AgentCapabilities {
            skills: get_skills_for_expertise(expertise),
            tools_enabled: get_tools_for_complexity(complexity),
            features_enabled: FeatureCapabilities {
                memory_search: true,
                heartbeat: false,
                human_delay: false,
                subagents: false,
                block_streaming: true,
                context_pruning: true,
                auto_save: true,
                collaborative_mode: false,
            },
            integrations: vec![],
            custom_capabilities: HashMap::new(),
        },
        behavior_settings: AgentBehaviorSettings {
            personality: AgentPersonality {
                tone: "professional".to_string(),
                expertise_level: "intermediate".to_string(),
                specialization: vec![expertise.to_string()],
                language_preference: "english".to_string(),
                cultural_context: None,
            },
            communication_style: CommunicationStyle {
                response_length: "detailed".to_string(),
                technical_level: "moderate".to_string(),
                code_style: "documented".to_string(),
                explanation_style: "step_by_step".to_string(),
            },
            response_preferences: ResponsePreferences {
                include_confidence: true,
                include_reasoning: true,
                include_alternatives: false,
                include_sources: false,
                format_preference: "markdown".to_string(),
            },
            working_hours: WorkingHours {
                timezone: "UTC".to_string(),
                active_hours: ActiveHoursConfig {
                    monday: DaySchedule { enabled: true, start_time: "09:00".to_string(), end_time: "18:00".to_string(), breaks: vec!["12:00-13:00".to_string()] },
                    tuesday: DaySchedule { enabled: true, start_time: "09:00".to_string(), end_time: "18:00".to_string(), breaks: vec!["12:00-13:00".to_string()] },
                    wednesday: DaySchedule { enabled: true, start_time: "09:00".to_string(), end_time: "18:00".to_string(), breaks: vec!["12:00-13:00".to_string()] },
                    thursday: DaySchedule { enabled: true, start_time: "09:00".to_string(), end_time: "18:00".to_string(), breaks: vec!["12:00-13:00".to_string()] },
                    friday: DaySchedule { enabled: true, start_time: "09:00".to_string(), end_time: "18:00".to_string(), breaks: vec!["12:00-13:00".to_string()] },
                    saturday: DaySchedule { enabled: false, start_time: "09:00".to_string(), end_time: "13:00".to_string(), breaks: vec![] },
                    sunday: DaySchedule { enabled: false, start_time: "09:00".to_string(), end_time: "13:00".to_string(), breaks: vec![] },
                },
                break_schedule: vec![],
                availability_calendar: HashMap::new(),
            },
            interaction_patterns: InteractionPatterns {
                greeting_style: "professional".to_string(),
                farewell_style: "professional".to_string(),
                error_handling: ErrorHandlingStyle {
                    approach: "professional".to_string(),
                    offer_solutions: true,
                    request_clarification: true,
                    escalation_threshold: 7,
                },
                clarification_preferences: ClarificationPreferences {
                    ask_questions: true,
                    confirm_understanding: true,
                    provide_examples: true,
                    check_assumptions: true,
                },
                feedback_requests: true,
            },
        },
        resource_limits: ResourceLimits {
            max_concurrent_tasks: 3,
            max_memory_mb: 2048,
            max_execution_time_minutes: 30,
            max_file_size_mb: 100,
            max_api_calls_per_hour: 100,
            cost_limits: CostLimits {
                daily_limit: Some(10.0),
                weekly_limit: Some(50.0),
                monthly_limit: Some(200.0),
                per_task_limit: Some(5.0),
                currency: "USD".to_string(),
            },
        },
        security_settings: SecuritySettings {
            access_level: SecurityAccessLevel::ReadWrite,
            data_permissions: DataPermissions {
                can_read_sensitive: false,
                can_write_sensitive: false,
                can_delete_data: false,
                can_share_data: true,
                data_retention_days: Some(30),
            },
            network_restrictions: NetworkRestrictions {
                allowed_domains: vec![],
                blocked_domains: vec![],
                require_https: true,
                max_requests_per_minute: 60,
            },
            audit_settings: AuditSettings {
                log_all_interactions: true,
                log_data_access: false,
                log_tool_usage: true,
                retention_days: 90,
            },
            encryption_requirements: EncryptionRequirements {
                encrypt_data_at_rest: true,
                encrypt_data_in_transit: true,
                encryption_algorithm: "AES-256".to_string(),
                key_rotation_days: Some(90),
            },
        },
    }
}

fn get_skills_for_expertise(expertise: &str) -> Vec<String> {
    match expertise {
        "coding" => vec!["coding", "debugging", "code_review", "documentation"],
        "writing" => vec!["writing", "editing", "content_creation", "copywriting"],
        "analysis" => vec!["data_analysis", "research", "critical_thinking", "synthesis"],
        "management" => vec!["project_management", "planning", "coordination", "leadership"],
        _ => vec!["general", "communication", "problem_solving"],
    }
}

fn get_tools_for_complexity(complexity: &str) -> ToolCapabilities {
    match complexity {
        "simple" => ToolCapabilities {
            exec_tools: false,
            file_operations: true,
            web_access: false,
            api_calls: false,
            database_access: false,
            system_commands: false,
        },
        "standard" => ToolCapabilities {
            exec_tools: true,
            file_operations: true,
            web_access: true,
            api_calls: true,
            database_access: false,
            system_commands: false,
        },
        "advanced" => ToolCapabilities {
            exec_tools: true,
            file_operations: true,
            web_access: true,
            api_calls: true,
            database_access: true,
            system_commands: true,
        },
        _ => ToolCapabilities {
            exec_tools: true,
            file_operations: true,
            web_access: true,
            api_calls: true,
            database_access: false,
            system_commands: false,
        },
    }
}

fn get_wizard_steps() -> Vec<WizardQuestion> {
    vec![
        WizardQuestion {
            id: "basic_info".to_string(),
            title: "Basic Information".to_string(),
            description: "Let's start with some basic information about your agent.".to_string(),
            question_type: "text".to_string(),
            options: None,
            default_value: None,
            validation: Some(ValidationRule {
                required: true,
                min_length: Some(2),
                max_length: Some(50),
                pattern: None,
                custom_validator: None,
            }),
            help_text: Some("Choose a descriptive name for your agent".to_string()),
            examples: vec!["Data Analyst Pro".to_string(), "Creative Writer".to_string(), "Debug Assistant".to_string()],
        },
        WizardQuestion {
            id: "purpose".to_string(),
            title: "Agent Purpose".to_string(),
            description: "What will this agent primarily do?".to_string(),
            question_type: "choice".to_string(),
            options: Some(vec![
                WizardOption {
                    value: serde_json::json!("coding"),
                    label: "Programming & Development".to_string(),
                    description: Some("Code, debug, review, and test software".to_string()),
                    icon: Some("💻".to_string()),
                    recommended: true,
                },
                WizardOption {
                    value: serde_json::json!("writing"),
                    label: "Writing & Content".to_string(),
                    description: Some("Create, edit, and review written content".to_string()),
                    icon: Some("✍️".to_string()),
                    recommended: false,
                },
                WizardOption {
                    value: serde_json::json!("analysis"),
                    label: "Data Analysis".to_string(),
                    description: Some("Analyze data, create reports, and insights".to_string()),
                    icon: Some("📊".to_string()),
                    recommended: false,
                },
                WizardOption {
                    value: serde_json::json!("research"),
                    label: "Research & Learning".to_string(),
                    description: some("Research topics and synthesize information".to_string()),
                    icon: Some("🔍".to_string()),
                    recommended: false,
                },
                ]),
            default_value: None,
            validation: Some(ValidationRule {
                required: true,
                min_length: None,
                max_length: None,
                pattern: None,
                custom_validator: None,
            }),
            help_text: Some("This determines the default configuration and skills".to_string()),
            examples: vec![],
        },
        WizardQuestion {
            id: "expertise".to_string(),
            title: "Expertise Level".to_string(),
            description: "How experienced should this agent be?".to_string(),
            question_type: "choice".to_string(),
            options: Some(vec![
                WizardOption {
                    value: serde_json::json!("beginner"),
                    label: "Beginner".to_string(),
                    description: Some("Simple tasks, clear explanations".to_string()),
                    icon: Some("🌱".to_string()),
                    recommended: false,
                },
                WizardOption {
                    value: serde_json::json!("intermediate"),
                    label: "Intermediate".to_string(),
                    description: Some("Balanced approach with some complexity".to_string()),
                    icon: Some("🚀".to_string()),
                    recommended: true,
                },
                WizardOption {
                    value: serde_json::json!("expert"),
                    label: "Expert".to_string(),
                    description: some("Complex tasks, minimal guidance needed".to_string()),
                    icon: Some("🏆".to_string()),
                    recommended: false,
                },
            ]),
            default_value: None,
            validation: Some(ValidationRule {
                required: true,
                min_length: None,
                max_length: None,
                pattern: None,
                custom_validator: None,
            }),
            help_text: Some("Affects model choice and response style".to_string()),
            examples: vec![],
        },
        WizardQuestion {
            id: "working_hours".to_string(),
            title: "Working Hours".to_string(),
            description: "When should this agent be available?".to_string(),
            question_type: "choice".to_string(),
            options: Some(vec![
                WizardOption {
                    value: serde_json::json!("business_hours"),
                    label: "Business Hours (9-5)".to_string(),
                    description: Some("Monday-Friday, 9AM-5PM".to_string()),
                    icon: Some("💼".to_string()),
                    recommended: true,
                },
                WizardOption {
                    value: serde_json::json!("extended"),
                    label: "Extended Hours".to_string(),
                    description: Some("Monday-Friday, 9AM-8PM".to_string()),
                    icon: Some("🕐".to_string()),
                    recommended: false,
                },
                WizardOption {
                    value: serde_json::json!("24_7"),
                    label: "24/7".to_string(),
                    description: Some("Always available".to_string()),
                    icon: Some("🌍".to_string()),
                    recommended: false,
                },
            ]),
            default_value: None,
            validation: Some(ValidationRule {
                required: true,
                min_length: None,
                max_length: None,
                pattern: None,
                custom_validator: None,
            }),
            help_text: Some("Consider when you'll need this agent".to_string()),
            examples: vec![],
        },
        WizardQuestion {
            id: "review".to_string(),
            title: "Review Configuration".to_string(),
            description: "Review your agent configuration before creation.".to_string(),
            question_type: "boolean".to_string(),
            options: None,
            default_value: Some(serde_json::json!(false)),
            validation: None,
            help_text: Some("Take a moment to review your choices".to_string()),
            examples: vec![],
        },
    ]
}

fn validate_answer(answers: &HashMap<String, serde_json::Value>, rule: &ValidationRule) -> bool {
    if rule.required && !answers.contains_key(&rule.id) {
        return false;
    }
    
    if let Some(answer) = answers.get(&rule.id) {
        if let Some(min_length) = rule.min_length {
            if let Some(s) = answer.as_str() {
                if s.len() < min_length {
                    return false;
                }
            }
        }
        
        if let Some(max_length) = rule.max_length {
            if let Some(s) = answer.as_str() {
                if s.len() > max_length {
                    return false;
                }
            }
        }
        
        if let Some(pattern) = &rule.pattern {
            // Simple regex validation
            // In production, use proper regex library
            if let Some(s) = answer.as_str() {
                // This is a simplified validation - in production use regex crate
                if pattern == "^[a-zA-Z0-9_-]+$" && !s.chars().all(|c| c.is_alphanumeric() || *c == '_' || *c == '-') {
                    return false;
                }
            }
        }
    }
    
    true
}

fn build_config_from_wizard_data(data: &serde_json::Value) -> AgentConfigRequest {
    // Extract answers from wizard data
    let name = data.get("basic_info").and_then(|v| v.as_str()).unwrap_or("Unconfigured Agent");
    let purpose = data.get("purpose").and_then(|v| v.as_str()).unwrap_or("general");
    let expertise = data.get("expertise").and_then(|v| v.as_str()).unwrap_or("intermediate");
    let working_hours = data.get("working_hours").and_then(|v| v.as_str()).unwrap_or("business_hours");
    
    // Build configuration based on wizard answers
    generate_smart_config(&QuickAgentRequest {
        name: name.to_string(),
        purpose: purpose.to_string(),
        expertise: Some(expertise.to_string()),
        complexity: Some("standard".to_string()),
    })
}

fn generate_help_for_topic(topic: &str, context: &str) -> HelpResponse {
    let help_content = match topic {
        "model_selection" => HelpResponse {
            topic: "model_selection".to_string(),
            title: "Choosing the Right Model".to_string(),
            content: "Select the appropriate AI model based on your task complexity and requirements. Claude Sonnet offers a good balance of performance and cost for most tasks, while Claude Opus provides superior reasoning for complex problems.".to_string(),
            examples: vec![
                "Use Sonnet for coding and debugging".to_string(),
                "Use Opus for research and analysis".to_string(),
                "Use Haiku for simple tasks".to_string(),
            ],
            related_topics: vec!["cost_optimization".to_string(), "performance_tuning".to_string()],
            difficulty: "Easy".to_string(),
            estimated_time: "5 minutes".to_string(),
        },
        "security_settings" => HelpResponse {
            topic: "security_settings".to_string(),
            title: "Security Configuration".to_string(),
            content: "Configure access levels, data permissions, and network restrictions to ensure your agent operates safely. Start with restrictive settings and gradually relax as needed.".to_string(),
            examples: vec![
                "Set access level to ReadOnly for data analysis agents".to_string(),
                "Enable HTTPS requirement for all web access".to_string(),
                "Configure audit logging for compliance".to_string(),
            ],
            related_topics: vec!["data_permissions".to_string(), "network_restrictions".to_string()],
            difficulty: "Medium".to_string(),
            estimated_time: "10 minutes".to_string(),
        },
        _ => HelpResponse {
            topic: topic.to_string(),
            title: "Help Topic".to_string(),
            content: "Help content for this topic is not yet available.".to_string(),
            examples: vec![],
            related_topics: vec![],
            difficulty: "Unknown".to_string(),
            estimated_time: "Unknown".to_string(),
        },
    };
    
    help_content
}

fn generate_smart_suggestions_for_agent(agent: &crate::models::Agent) -> Vec<SmartSuggestion> {
    let mut suggestions = Vec::new();
    
    // Model performance suggestions
    if agent.model_failure_count > 5 {
        suggestions.push(SmartSuggestion {
            id: "model_fallback".to_string(),
            title: "Improve Model Reliability".to_string(),
            description: format!("Your agent has {} model failures. Consider adding more fallback models or improving error handling.", agent.model_failure_count),
            category: SuggestionCategory::Performance,
            impact: "High".to_string(),
            effort: "Medium".to_string(),
            auto_applicable: false,
            steps: vec![
                "Add additional fallback models".to_string(),
                "Review error handling patterns".to_string(),
                "Consider model retraining".to_string(),
            ],
            why_important: "Model failures reduce agent reliability and user satisfaction".to_string(),
        });
    }
    
    // Resource optimization suggestions
    if let Some(max_concurrent) = agent.max_concurrent {
        if max_concurrent < 3 {
            suggestions.push(SmartSuggestion {
                id: "increase_concurrency".to_string(),
                title: "Increase Concurrent Tasks".to_string(),
                description: "Your agent can handle more concurrent tasks for better efficiency. Consider increasing from {} to 5.".to_string(),
                category: SuggestionCategory::Performance,
                impact: "Medium".to_string(),
                effort: "Easy".to_string(),
                auto_applicable: true,
                steps: vec![
                    "Update max_concurrent_tasks to 5".to_string(),
                    "Monitor performance after change".to_string(),
                ],
                why_important: "Higher concurrency improves throughput and reduces wait times".to_string(),
            });
        }
    }
    
    // Security recommendations
    suggestions.push(SmartSuggestion {
        id: "security_audit".to_string(),
        title: "Security Audit Recommended".to_string(),
        description: "Regular security audits help maintain compliance and identify potential vulnerabilities.".to_string(),
        category: SuggestionCategory::Security,
        impact: "Medium".to_string(),
        effort: "Low".to_string(),
        auto_applicable: false,
        steps: vec![
            "Review access permissions".to_string(),
            "Check network restrictions".to_string(),
            "Validate audit settings".to_string(),
        ],
        why_important: "Security audits prevent data breaches and compliance issues".to_string(),
    });
    
    suggestions
}

fn validate_configuration_with_detailed_feedback(config: &serde_json::Value) -> ValidationResult {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    let mut score = 100.0;
    
    // Basic validation
    if !config.get("agent").and_then(|a| a.get("name")).and_then(|n| n.as_str()).map_or(false, |s| s.len() < 2) {
        issues.push(ValidationIssue {
            field: "agent.name".to_string(),
            severity: "error".to_string(),
            message: "Agent name must be at least 2 characters".to_string(),
            suggestion: "Choose a more descriptive name".to_string(),
            auto_fixable: false,
        });
        score -= 20.0;
    }
    
    // Model configuration validation
    if let Some(model_config) = config.get("agent").and_then(|a| a.get("model_config")) {
        if !model_config.get("primary_model").and_then(|m| m.as_str()).map_or(false, |s| !s.is_empty()) {
            issues.push(ValidationIssue {
                field: "model_config.primary_model".to_string(),
                severity: "error".to_string(),
                message: "Primary model is required".to_string(),
                suggestion: "Select a primary model from the available options".to_string(),
                auto_fixable: false,
            });
            score -= 30.0;
        }
        
        if model_config.get("temperature").and_then(|t| t.as_f64()).map_or(false, |temp| temp < 0.0 || temp > 2.0) {
            warnings.push(ValidationIssue {
                field: "model_config.temperature".to_string(),
                severity: "warning".to_string(),
                message: "Temperature should be between 0.0 and 2.0".to_string(),
                suggestion: "Consider using a more moderate temperature".to_string(),
                auto_fixable: true,
            });
            score -= 10.0;
        }
    }
    
    // Resource limits validation
    if let Some(resource_limits) = config.get("agent").and_then(|a| a.get("resource_limits")) {
        if resource_limits.get("max_concurrent_tasks").and_then(|c| c.as_u64()).map_or(false, |c| c == 0) {
            issues.push(ValidationIssue {
                field: "resource_limits.max_concurrent_tasks".to_string(),
                severity: "warning".to_string(),
                message: "Max concurrent tasks should be at least 1".to_string(),
                suggestion: "Set to at least 3 for better performance".to_string(),
                auto_fixable: true,
            });
            score -= 15.0;
        }
        
        if resource_limits.get("max_execution_time_minutes").and_then(|t| t.as_u64()).map_or(false, |t| t > 120) {
            warnings.push(ValidationIssue {
                field: "resource_limits.max_execution_time_minutes".to_string(),
                severity: "warning".to_string(),
                message: "Long execution times may impact performance".to_string(),
                suggestion: "Consider reducing to 60 minutes or less".to_string(),
                auto_fixable: false,
            });
            score -= 5.0;
        }
    }
    
    let suggestions = vec![
        "Review agent configuration in dashboard".to_string(),
        "Test agent with sample tasks".to_string(),
        "Monitor performance metrics".to_string(),
    ];
    
    ValidationResult {
        valid: issues.is_empty(),
        score: score.max(0.0),
        issues,
        warnings,
        suggestions,
        auto_fixable: warnings.iter().any(|w| w.auto_fixable),
    }
}

fn get_templates_with_filters(
    search: Option<String>,
    category: Option<String>,
    sort_by: &str,
) -> Vec<TemplateCard> {
    // This would query the database for templates
    // For now, return mock data
    vec![
        TemplateCard {
            id: "developer-assistant".to_string(),
            name: "Developer Assistant".to_string(),
            description: "Expert in coding, debugging, and technical tasks".to_string(),
            category: "development".to_string(),
            tags: vec!["coding".to_string(), "debugging".to_string(), "technical".to_string()],
            popularity_score: 0.9,
            setup_difficulty: "Medium".to_string(),
            estimated_time: "5 minutes".to_string(),
            preview: TemplatePreview {
                model_config: "Claude Sonnet with coding focus".to_string(),
                key_features: vec!["Code generation".to_string(), "Debugging".to_string(), "Code review".to_string()],
                use_cases: vec!["Software development".to_string(), "Bug fixing".to_string(), "Code review".to_string()],
            },
        },
        TemplateCard {
            id: "data-analyst".to_string(),
            name: "Data Analyst".to_string(),
            description: "Specialized in data analysis and visualization".to_string(),
            category: "analytics".to_string(),
            tags: vec!["data".to_string(), "analytics".to_string(), "visualization".to_string()],
            popularity_score: 0.8,
            setup_difficulty: "Easy".to_string(),
            estimated_time: "3 minutes".to_string(),
            preview: TemplatePreview {
                model_config: "Claude Sonnet with analysis focus".to_string(),
                key_features: vec!["Data analysis".to_string(), "Visualization".to_string(), "Reporting".to_string()],
                use_cases: vec!["Business intelligence".to_string(), "Data insights".to_string(), "Report generation".to_string()],
            },
        },
    ]
}

fn create_visual_comparison(comparison: &crate::agent_management::AgentComparison) -> VisualComparison {
    // Convert comparison data to visual format
    let agents: Vec<VisualAgentData> = comparison.agents.iter().map(|agent_data| {
        VisualAgentData {
            agent_id: agent_data.agent_id.clone(),
            name: agent_data.agent_id.clone(), // Would use actual name
            avatar: None,
            scores: AgentScores {
                overall: agent_data.score,
                performance: 0.8, // Would calculate actual scores
                cost_efficiency: 0.7,
                capabilities: 0.9,
                reliability: 0.95,
            },
            key_metrics: agent_data.metrics.clone(),
            strengths: vec!["High success rate".to_string(), "Fast response time".to_string()],
            weaknesses: vec!["High cost".to_string(), "Limited memory".to_string()],
        }
    }).collect();
    
    VisualComparison {
        comparison_id: comparison.comparison_id.clone(),
        agents,
        insights: vec![
            VisualInsight {
                title: "Performance vs Cost Trade-off".to_string(),
                description: "Agent 1 has better performance but higher costs".to_string(),
                type_: "analysis".to_string(),
                importance: "High".to_string(),
                affected_agents: vec!["agent1".to_string(), "agent2".to_string()],
            },
        ],
        recommendations: vec![
            VisualRecommendation {
                title: "Optimize Agent 2 Configuration".to_string(),
                description: "Adjust resource limits for better cost efficiency".to_string(),
                category: "cost".to_string(),
                target_agents: vec!["agent2".to_string()],
                expected_improvement: "20% cost reduction".to_string(),
                implementation_steps: vec![
                    "Reduce max_concurrent_tasks".to_string(),
                    "Optimize model selection".to_string(),
                    "Enable context pruning".to_string(),
                ],
            },
        ],
        charts: vec![
            Chart {
                type_: "bar".to_string(),
                title: "Performance Comparison".to_string(),
                data: vec![
                    ChartPoint { x: 1.0, y: 0.85, label: Some("Agent 1".to_string()) },
                    ChartPoint { x: 2.0, y: 0.92, label: Some("Agent 2".to_string()) },
                ],
                x_axis: "Agent".to_string(),
                y_axis: "Performance Score".to_string(),
            },
            Chart {
                type_: "radar".to_string(),
                title: "Capability Overview".to_string(),
                data: vec![
                    ChartPoint { x: 0.9, y: 0.8, label: Some("Coding".to_string()) },
                    ChartPoint { x: 0.7, y: 0.95, label: Some("Analysis".to_string()) },
                    ChartPoint { x: 0.8, y: 0.85, label: Some("Writing".to_string()) },
                    ChartPoint { x: 0.95, y: 0.6, label: Some("Research".to_string()) },
                ],
                x_axis: "Capability".to_string(),
                y_axis: "Proficiency".to_string(),
            },
        ],
    }
}

// Add these endpoints to main.rs routing
pub fn add_user_friendly_endpoints(router: axum::Router<crate::AppState>) -> axum::Router<crate::AppState> {
    router
        .route("/agents/quick", axum::routing::post(create_agent_quick))
        .route("/agents/wizard", axum::routing::post(configuration_wizard))
        .route("/agents/help/:topic", axum::routing::get(get_configuration_help))
        .route("/agents/:id/suggestions", axum::routing::get(get_smart_suggestions))
        .route("/agents/validate", axum::routing::post(validate_configuration_friendly))
        .route("/agents/templates/user-friendly", axum::routing::get(get_templates_user_friendly))
        .route("/agents/compare/visual", axum::routing::post(compare_agents_visual))
}
