use crate::models::*;
use sqlx::SqlitePool;
use chrono::Utc;
use serde_json::Value;

// Database schema extensions for comprehensive agent management

pub async fn setup_agent_management_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Comprehensive agent configurations table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_comprehensive_configs (
            agent_id TEXT PRIMARY KEY,
            config_json TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE
        );
        "#
    )
    .execute(pool)
    .await?;

    // Agent clone relationships table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_clone_relationships (
            id TEXT PRIMARY KEY,
            source_agent_id TEXT NOT NULL,
            cloned_agent_id TEXT NOT NULL,
            cloned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(source_agent_id) REFERENCES agents(id),
            FOREIGN KEY(cloned_agent_id) REFERENCES agents(id)
        );
        "#
    )
    .execute(pool)
    .await?;

    // Agent templates table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            category TEXT,
            role TEXT NOT NULL,
            configuration TEXT NOT NULL,
            usage_count INTEGER DEFAULT 0,
            rating REAL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            tags TEXT -- JSON array
        );
        "#
    )
    .execute(pool)
    .await?;

    // Agent template relationships table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_template_relationships (
            id TEXT PRIMARY KEY,
            template_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(template_id) REFERENCES agent_templates(id),
            FOREIGN KEY(agent_id) REFERENCES agents(id)
        );
        "#
    )
    .execute(pool)
    .await?;

    // Agent performance metrics table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_performance_metrics (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            metric_date DATE NOT NULL,
            total_tasks_completed INTEGER DEFAULT 0,
            average_task_duration_seconds REAL,
            success_rate REAL,
            error_rate REAL,
            total_cost REAL,
            total_tokens_used INTEGER,
            total_api_calls INTEGER,
            memory_usage_mb REAL,
            cpu_usage_percent REAL,
            user_satisfaction_score REAL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(agent_id) REFERENCES agents(id)
        );
        "#
    )
    .execute(pool)
    .await?;

    // Agent activity detailed table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_activity_detailed (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            activity_type TEXT NOT NULL,
            description TEXT,
            task_id TEXT,
            duration_seconds REAL,
            success BOOLEAN,
            error_message TEXT,
            tokens_used INTEGER,
            cost REAL,
            metadata TEXT, -- JSON object
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(agent_id) REFERENCES agents(id)
        );
        "#
    )
    .execute(pool)
    .await?;

    // Agent recommendations table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_recommendations (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            category TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            priority TEXT,
            estimated_impact TEXT,
            implementation_difficulty TEXT,
            auto_applicable BOOLEAN DEFAULT FALSE,
            status TEXT DEFAULT 'pending', -- pending, applied, dismissed
            steps TEXT, -- JSON array
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            applied_at DATETIME,
            FOREIGN KEY(agent_id) REFERENCES agents(id)
        );
        "#
    )
    .execute(pool)
    .await?;

    // Agent health status table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_health_status (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            overall_health REAL,
            performance_health REAL,
            configuration_health REAL,
            security_health REAL,
            resource_health REAL,
            health_trend TEXT,
            last_check DATETIME DEFAULT CURRENT_TIMESTAMP,
            issues TEXT, -- JSON array
            FOREIGN KEY(agent_id) REFERENCES agents(id)
        );
        "#
    )
    .execute(pool)
    .await?;

    // Agent analytics cache table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_analytics_cache (
            agent_id TEXT,
            period TEXT,
            metrics_type TEXT,
            analytics_data TEXT NOT NULL,
            cached_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            expires_at DATETIME,
            PRIMARY KEY (agent_id, period, metrics_type),
            FOREIGN KEY(agent_id) REFERENCES agents(id)
        );
        "#
    )
    .execute(pool)
    .await?;

    // Agent usage patterns table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_usage_patterns (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            pattern_type TEXT NOT NULL, -- daily, weekly, monthly
            pattern_data TEXT NOT NULL, -- JSON array of usage data
            calculated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            valid_until DATETIME,
            FOREIGN KEY(agent_id) REFERENCES agents(id)
        );
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes for performance
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_agent_performance_agent_date ON agent_performance_metrics(agent_id, metric_date)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_agent_activity_agent_timestamp ON agent_activity_detailed(agent_id, timestamp)")
        .execute(pool)
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_agent_recommendations_agent_status ON agent_recommendations(agent_id, status)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_agent_templates_category_role ON agent_templates(category, role)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_analytics_cache_expires ON agent_analytics_cache(expires_at)")
        .execute(pool)
        .await?;

    Ok(())
}

// Insert default agent templates
pub async fn insert_default_agent_templates(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let templates = vec![
        AgentTemplateData {
            id: "developer-assistant".to_string(),
            name: "Developer Assistant".to_string(),
            description: "General-purpose developer assistant for coding, debugging, and technical tasks".to_string(),
            category: "development".to_string(),
            role: AgentRole::Spc,
            configuration: create_developer_template_config(),
            tags: vec!["coding".to_string(), "debugging".to_string(), "technical".to_string()],
        },
        AgentTemplateData {
            id: "data-analyst".to_string(),
            name: "Data Analyst".to_string(),
            description: "Specialized agent for data analysis, visualization, and reporting".to_string(),
            category: "analytics".to_string(),
            role: AgentRole::Spc,
            configuration: create_analyst_template_config(),
            tags: vec!["data".to_string(), "analytics".to_string(), "visualization".to_string()],
        },
        AgentTemplateData {
            id: "content-creator".to_string(),
            name: "Content Creator".to_string(),
            description: "Creative agent for writing, editing, and content generation".to_string(),
            category: "content".to_string(),
            role: AgentRole::Spc,
            configuration: create_content_template_config(),
            tags: vec!["writing".to_string(), "creative".to_string(), "editing".to_string()],
        },
        AgentTemplateData {
            id: "research-assistant".to_string(),
            name: "Research Assistant".to_string(),
            description: "Research-focused agent for information gathering, analysis, and synthesis".to_string(),
            category: "research".to_string(),
            role: AgentRole::Spc,
            configuration: create_research_template_config(),
            tags: vec!["research".to_string(), "analysis".to_string(), "synthesis".to_string()],
        },
        AgentTemplateData {
            id: "project-manager".to_string(),
            name: "Project Manager".to_string(),
            description: "Management agent for project planning, coordination, and tracking".to_string(),
            category: "management".to_string(),
            role: AgentRole::Lead,
            configuration: create_manager_template_config(),
            tags: vec!["management".to_string(), "planning".to_string(), "coordination".to_string()],
        },
    ];

    for template in templates {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO agent_templates (
                id, name, description, category, role, configuration, tags
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&template.id)
        .bind(&template.name)
        .bind(&template.description)
        .bind(&template.category)
        .bind(format!("{:?}", template.role))
        .bind(&template.configuration)
        .bind(serde_json::to_string(&template.tags)?)
        .execute(pool)
        .await?;
    }

    Ok(())
}

// Helper structures
struct AgentTemplateData {
    id: String,
    name: String,
    description: String,
    category: String,
    role: AgentRole,
    configuration: String,
    tags: Vec<String>,
}

// Template configuration generators
fn create_developer_template_config() -> String {
    serde_json::json!({
        "model_config": {
            "primary_model": "anthropic/claude-3-sonnet",
            "fallback_models": ["openai/gpt-4", "google/gemini-pro"],
            "image_model": null,
            "thinking_level": "medium",
            "verbose_level": "on",
            "temperature": 0.1,
            "max_tokens": 4000
        },
        "capabilities": {
            "skills": ["coding", "debugging", "code_review", "documentation", "testing"],
            "tools_enabled": {
                "exec_tools": true,
                "file_operations": true,
                "web_access": true,
                "api_calls": true,
                "database_access": false,
                "system_commands": false
            },
            "features_enabled": {
                "memory_search": true,
                "heartbeat": false,
                "human_delay": false,
                "subagents": false,
                "block_streaming": true,
                "context_pruning": true,
                "auto_save": true,
                "collaborative_mode": false
            },
            "integrations": ["github", "gitlab", "jira", "slack"],
            "custom_capabilities": {}
        },
        "behavior_settings": {
            "personality": {
                "tone": "professional",
                "expertise_level": "expert",
                "specialization": ["software_development", "system_architecture"],
                "language_preference": "english",
                "cultural_context": null
            },
            "communication_style": {
                "response_length": "detailed",
                "technical_level": "technical",
                "code_style": "documented",
                "explanation_style": "step_by_step"
            },
            "response_preferences": {
                "include_confidence": true,
                "include_reasoning": true,
                "include_alternatives": true,
                "include_sources": false,
                "format_preference": "markdown"
            },
            "working_hours": {
                "timezone": "UTC",
                "active_hours": {
                    "monday": {"enabled": true, "start_time": "09:00", "end_time": "18:00", "breaks": ["12:00-13:00"]},
                    "tuesday": {"enabled": true, "start_time": "09:00", "end_time": "18:00", "breaks": ["12:00-13:00"]},
                    "wednesday": {"enabled": true, "start_time": "09:00", "end_time": "18:00", "breaks": ["12:00-13:00"]},
                    "thursday": {"enabled": true, "start_time": "09:00", "end_time": "18:00", "breaks": ["12:00-13:00"]},
                    "friday": {"enabled": true, "start_time": "09:00", "end_time": "18:00", "breaks": ["12:00-13:00"]},
                    "saturday": {"enabled": false, "start_time": "09:00", "end_time": "13:00", "breaks": []},
                    "sunday": {"enabled": false, "start_time": "09:00", "end_time": "13:00", "breaks": []}
                },
                "break_schedule": [],
                "availability_calendar": {}
            },
            "interaction_patterns": {
                "greeting_style": "professional",
                "farewell_style": "professional",
                "error_handling": {
                    "approach": "professional",
                    "offer_solutions": true,
                    "request_clarification": true,
                    "escalation_threshold": 7
                },
                "clarification_preferences": {
                    "ask_questions": true,
                    "confirm_understanding": true,
                    "provide_examples": true,
                    "check_assumptions": true
                },
                "feedback_requests": true
            }
        },
        "resource_limits": {
            "max_concurrent_tasks": 3,
            "max_memory_mb": 2048,
            "max_execution_time_minutes": 30,
            "max_file_size_mb": 100,
            "max_api_calls_per_hour": 100,
            "cost_limits": {
                "daily_limit": 10.0,
                "weekly_limit": 50.0,
                "monthly_limit": 200.0,
                "per_task_limit": 5.0,
                "currency": "USD"
            }
        },
        "security_settings": {
            "access_level": "ReadWrite",
            "data_permissions": {
                "can_read_sensitive": false,
                "can_write_sensitive": false,
                "can_delete_data": false,
                "can_share_data": false,
                "data_retention_days": 30
            },
            "network_restrictions": {
                "allowed_domains": ["github.com", "gitlab.com", "stackoverflow.com"],
                "blocked_domains": [],
                "require_https": true,
                "max_requests_per_minute": 60
            },
            "audit_settings": {
                "log_all_interactions": true,
                "log_data_access": true,
                "log_tool_usage": true,
                "retention_days": 90
            },
            "encryption_requirements": {
                "encrypt_data_at_rest": true,
                "encrypt_data_in_transit": true,
                "encryption_algorithm": "AES-256",
                "key_rotation_days": 90
            }
        }
    }).to_string()
}

fn create_analyst_template_config() -> String {
    serde_json::json!({
        "model_config": {
            "primary_model": "anthropic/claude-3-sonnet",
            "fallback_models": ["openai/gpt-4", "google/gemini-pro"],
            "image_model": "anthropic/claude-3-sonnet",
            "thinking_level": "high",
            "verbose_level": "on",
            "temperature": 0.2,
            "max_tokens": 6000
        },
        "capabilities": {
            "skills": ["data_analysis", "statistics", "visualization", "reporting", "sql"],
            "tools_enabled": {
                "exec_tools": true,
                "file_operations": true,
                "web_access": true,
                "api_calls": true,
                "database_access": true,
                "system_commands": false
            },
            "features_enabled": {
                "memory_search": true,
                "heartbeat": false,
                "human_delay": false,
                "subagents": false,
                "block_streaming": true,
                "context_pruning": true,
                "auto_save": true,
                "collaborative_mode": true
            },
            "integrations": ["tableau", "powerbi", "excel", "google_sheets", "slack"],
            "custom_capabilities": {}
        },
        "behavior_settings": {
            "personality": {
                "tone": "professional",
                "expertise_level": "expert",
                "specialization": ["data_science", "statistics", "business_intelligence"],
                "language_preference": "english",
                "cultural_context": null
            },
            "communication_style": {
                "response_length": "comprehensive",
                "technical_level": "moderate",
                "code_style": "documented",
                "explanation_style": "practical"
            },
            "response_preferences": {
                "include_confidence": true,
                "include_reasoning": true,
                "include_alternatives": true,
                "include_sources": true,
                "format_preference": "markdown"
            },
            "working_hours": {
                "timezone": "UTC",
                "active_hours": {
                    "monday": {"enabled": true, "start_time": "08:00", "end_time": "20:00", "breaks": ["12:00-13:00", "17:00-17:30"]},
                    "tuesday": {"enabled": true, "start_time": "08:00", "end_time": "20:00", "breaks": ["12:00-13:00", "17:00-17:30"]},
                    "wednesday": {"enabled": true, "start_time": "08:00", "end_time": "20:00", "breaks": ["12:00-13:00", "17:00-17:30"]},
                    "thursday": {"enabled": true, "start_time": "08:00", "end_time": "20:00", "breaks": ["12:00-13:00", "17:00-17:30"]},
                    "friday": {"enabled": true, "start_time": "08:00", "end_time": "20:00", "breaks": ["12:00-13:00", "17:00-17:30"]},
                    "saturday": {"enabled": true, "start_time": "09:00", "end_time": "14:00", "breaks": ["11:30-12:00"]},
                    "sunday": {"enabled": false, "start_time": "09:00", "end_time": "14:00", "breaks": []}
                },
                "break_schedule": [],
                "availability_calendar": {}
            },
            "interaction_patterns": {
                "greeting_style": "professional",
                "farewell_style": "professional",
                "error_handling": {
                    "approach": "professional",
                    "offer_solutions": true,
                    "request_clarification": true,
                    "escalation_threshold": 6
                },
                "clarification_preferences": {
                    "ask_questions": true,
                    "confirm_understanding": true,
                    "provide_examples": true,
                    "check_assumptions": true
                },
                "feedback_requests": true
            }
        },
        "resource_limits": {
            "max_concurrent_tasks": 2,
            "max_memory_mb": 4096,
            "max_execution_time_minutes": 45,
            "max_file_size_mb": 500,
            "max_api_calls_per_hour": 150,
            "cost_limits": {
                "daily_limit": 15.0,
                "weekly_limit": 75.0,
                "monthly_limit": 300.0,
                "per_task_limit": 10.0,
                "currency": "USD"
            }
        },
        "security_settings": {
            "access_level": "ReadWrite",
            "data_permissions": {
                "can_read_sensitive": true,
                "can_write_sensitive": false,
                "can_delete_data": false,
                "can_share_data": true,
                "data_retention_days": 60
            },
            "network_restrictions": {
                "allowed_domains": ["tableau.com", "powerbi.com", "github.com"],
                "blocked_domains": [],
                "require_https": true,
                "max_requests_per_minute": 80
            },
            "audit_settings": {
                "log_all_interactions": true,
                "log_data_access": true,
                "log_tool_usage": true,
                "retention_days": 180
            },
            "encryption_requirements": {
                "encrypt_data_at_rest": true,
                "encrypt_data_in_transit": true,
                "encryption_algorithm": "AES-256",
                "key_rotation_days": 60
            }
        }
    }).to_string()
}

fn create_content_template_config() -> String {
    serde_json::json!({
        "model_config": {
            "primary_model": "anthropic/claude-3-sonnet",
            "fallback_models": ["openai/gpt-4", "google/gemini-pro"],
            "image_model": "anthropic/claude-3-sonnet",
            "thinking_level": "medium",
            "verbose_level": "full",
            "temperature": 0.7,
            "max_tokens": 3000
        },
        "capabilities": {
            "skills": ["writing", "editing", "content_creation", "copywriting", "storytelling"],
            "tools_enabled": {
                "exec_tools": false,
                "file_operations": true,
                "web_access": true,
                "api_calls": false,
                "database_access": false,
                "system_commands": false
            },
            "features_enabled": {
                "memory_search": true,
                "heartbeat": false,
                "human_delay": true,
                "subagents": false,
                "block_streaming": true,
                "context_pruning": true,
                "auto_save": true,
                "collaborative_mode": true
            },
            "integrations": ["wordpress", "medium", "notion", "slack"],
            "custom_capabilities": {}
        },
        "behavior_settings": {
            "personality": {
                "tone": "casual",
                "expertise_level": "intermediate",
                "specialization": ["creative_writing", "content_marketing", "storytelling"],
                "language_preference": "english",
                "cultural_context": null
            },
            "communication_style": {
                "response_length": "detailed",
                "technical_level": "simple",
                "code_style": "minimal",
                "explanation_style": "conceptual"
            },
            "response_preferences": {
                "include_confidence": false,
                "include_reasoning": false,
                "include_alternatives": true,
                "include_sources": false,
                "format_preference": "markdown"
            },
            "working_hours": {
                "timezone": "UTC",
                "active_hours": {
                    "monday": {"enabled": true, "start_time": "10:00", "end_time": "19:00", "breaks": ["13:00-14:00"]},
                    "tuesday": {"enabled": true, "start_time": "10:00", "end_time": "19:00", "breaks": ["13:00-14:00"]},
                    "wednesday": {"enabled": true, "start_time": "10:00", "end_time": "19:00", "breaks": ["13:00-14:00"]},
                    "thursday": {"enabled": true, "start_time": "10:00", "end_time": "19:00", "breaks": ["13:00-14:00"]},
                    "friday": {"enabled": true, "start_time": "10:00", "end_time": "19:00", "breaks": ["13:00-14:00"]},
                    "saturday": {"enabled": true, "start_time": "11:00", "end_time": "16:00", "breaks": []},
                    "sunday": {"enabled": false, "start_time": "11:00", "end_time": "16:00", "breaks": []}
                },
                "break_schedule": [],
                "availability_calendar": {}
            },
            "interaction_patterns": {
                "greeting_style": "friendly",
                "farewell_style": "friendly",
                "error_handling": {
                    "approach": "casual",
                    "offer_solutions": true,
                    "request_clarification": true,
                    "escalation_threshold": 8
                },
                "clarification_preferences": {
                    "ask_questions": true,
                    "confirm_understanding": true,
                    "provide_examples": true,
                    "check_assumptions": false
                },
                "feedback_requests": true
            }
        },
        "resource_limits": {
            "max_concurrent_tasks": 2,
            "max_memory_mb": 1024,
            "max_execution_time_minutes": 20,
            "max_file_size_mb": 50,
            "max_api_calls_per_hour": 50,
            "cost_limits": {
                "daily_limit": 8.0,
                "weekly_limit": 40.0,
                "monthly_limit": 160.0,
                "per_task_limit": 3.0,
                "currency": "USD"
            }
        },
        "security_settings": {
            "access_level": "ReadOnly",
            "data_permissions": {
                "can_read_sensitive": false,
                "can_write_sensitive": false,
                "can_delete_data": false,
                "can_share_data": true,
                "data_retention_days": 30
            },
            "network_restrictions": {
                "allowed_domains": ["medium.com", "wordpress.com", "notion.so"],
                "blocked_domains": [],
                "require_https": true,
                "max_requests_per_minute": 30
            },
            "audit_settings": {
                "log_all_interactions": true,
                "log_data_access": false,
                "log_tool_usage": false,
                "retention_days": 60
            },
            "encryption_requirements": {
                "encrypt_data_at_rest": true,
                "encrypt_data_in_transit": true,
                "encryption_algorithm": "AES-256",
                "key_rotation_days": 90
            }
        }
    }).to_string()
}

fn create_research_template_config() -> String {
    serde_json::json!({
        "model_config": {
            "primary_model": "anthropic/claude-3-sonnet",
            "fallback_models": ["openai/gpt-4", "google/gemini-pro"],
            "image_model": null,
            "thinking_level": "high",
            "verbose_level": "full",
            "temperature": 0.3,
            "max_tokens": 5000
        },
        "capabilities": {
            "skills": ["research", "analysis", "synthesis", "fact_checking", "citation"],
            "tools_enabled": {
                "exec_tools": false,
                "file_operations": true,
                "web_access": true,
                "api_calls": true,
                "database_access": false,
                "system_commands": false
            },
            "features_enabled": {
                "memory_search": true,
                "heartbeat": false,
                "human_delay": false,
                "subagents": false,
                "block_streaming": true,
                "context_pruning": true,
                "auto_save": true,
                "collaborative_mode": true
            },
            "integrations": ["scholar", "pubmed", "arxiv", "wikipedia", "jstor"],
            "custom_capabilities": {}
        },
        "behavior_settings": {
            "personality": {
                "tone": "formal",
                "expertise_level": "expert",
                "specialization": ["academic_research", "critical_analysis", "information_synthesis"],
                "language_preference": "english",
                "cultural_context": null
            },
            "communication_style": {
                "response_length": "comprehensive",
                "technical_level": "moderate",
                "code_style": "minimal",
                "explanation_style": "step_by_step"
            },
            "response_preferences": {
                "include_confidence": true,
                "include_reasoning": true,
                "include_alternatives": true,
                "include_sources": true,
                "format_preference": "markdown"
            },
            "working_hours": {
                "timezone": "UTC",
                "active_hours": {
                    "monday": {"enabled": true, "start_time": "08:00", "end_time": "22:00", "breaks": ["12:00-13:00", "18:00-18:30"]},
                    "tuesday": {"enabled": true, "start_time": "08:00", "end_time": "22:00", "breaks": ["12:00-13:00", "18:00-18:30"]},
                    "wednesday": {"enabled": true, "start_time": "08:00", "end_time": "22:00", "breaks": ["12:00-13:00", "18:00-18:30"]},
                    "thursday": {"enabled": true, "start_time": "08:00", "end_time": "22:00", "breaks": ["12:00-13:00", "18:00-18:30"]},
                    "friday": {"enabled": true, "start_time": "08:00", "end_time": "22:00", "breaks": ["12:00-13:00", "18:00-18:30"]},
                    "saturday": {"enabled": true, "start_time": "09:00", "end_time": "18:00", "breaks": ["12:00-13:00"]},
                    "sunday": {"enabled": true, "start_time": "09:00", "end_time": "18:00", "breaks": ["12:00-13:00"]}
                },
                "break_schedule": [],
                "availability_calendar": {}
            },
            "interaction_patterns": {
                "greeting_style": "formal",
                "farewell_style": "formal",
                "error_handling": {
                    "approach": "professional",
                    "offer_solutions": true,
                    "request_clarification": true,
                    "escalation_threshold": 5
                },
                "clarification_preferences": {
                    "ask_questions": true,
                    "confirm_understanding": true,
                    "provide_examples": true,
                    "check_assumptions": true
                },
                "feedback_requests": true
            }
        },
        "resource_limits": {
            "max_concurrent_tasks": 3,
            "max_memory_mb": 3072,
            "max_execution_time_minutes": 60,
            "max_file_size_mb": 200,
            "max_api_calls_per_hour": 200,
            "cost_limits": {
                "daily_limit": 20.0,
                "weekly_limit": 100.0,
                "monthly_limit": 400.0,
                "per_task_limit": 15.0,
                "currency": "USD"
            }
        },
        "security_settings": {
            "access_level": "ReadWrite",
            "data_permissions": {
                "can_read_sensitive": true,
                "can_write_sensitive": false,
                "can_delete_data": false,
                "can_share_data": true,
                "data_retention_days": 90
            },
            "network_restrictions": {
                "allowed_domains": ["scholar.google.com", "pubmed.ncbi.nlm.nih.gov", "arxiv.org"],
                "blocked_domains": [],
                "require_https": true,
                "max_requests_per_minute": 100
            },
            "audit_settings": {
                "log_all_interactions": true,
                "log_data_access": true,
                "log_tool_usage": true,
                "retention_days": 365
            },
            "encryption_requirements": {
                "encrypt_data_at_rest": true,
                "encrypt_data_in_transit": true,
                "encryption_algorithm": "AES-256",
                "key_rotation_days": 90
            }
        }
    }).to_string()
}

fn create_manager_template_config() -> String {
    serde_json::json!({
        "model_config": {
            "primary_model": "anthropic/claude-3-sonnet",
            "fallback_models": ["openai/gpt-4", "google/gemini-pro"],
            "image_model": null,
            "thinking_level": "high",
            "verbose_level": "on",
            "temperature": 0.4,
            "max_tokens": 4000
        },
        "capabilities": {
            "skills": ["project_management", "planning", "coordination", "communication", "reporting"],
            "tools_enabled": {
                "exec_tools": true,
                "file_operations": true,
                "web_access": true,
                "api_calls": true,
                "database_access": true,
                "system_commands": false
            },
            "features_enabled": {
                "memory_search": true,
                "heartbeat": true,
                "human_delay": true,
                "subagents": true,
                "block_streaming": true,
                "context_pruning": true,
                "auto_save": true,
                "collaborative_mode": true
            },
            "integrations": ["jira", "asana", "trello", "slack", "teams", "notion"],
            "custom_capabilities": {}
        },
        "behavior_settings": {
            "personality": {
                "tone": "professional",
                "expertise_level": "expert",
                "specialization": ["project_management", "team_leadership", "strategic_planning"],
                "language_preference": "english",
                "cultural_context": null
            },
            "communication_style": {
                "response_length": "comprehensive",
                "technical_level": "moderate",
                "code_style": "documented",
                "explanation_style": "practical"
            },
            "response_preferences": {
                "include_confidence": true,
                "include_reasoning": true,
                "include_alternatives": true,
                "include_sources": true,
                "format_preference": "markdown"
            },
            "working_hours": {
                "timezone": "UTC",
                "active_hours": {
                    "monday": {"enabled": true, "start_time": "08:00", "end_time": "20:00", "breaks": ["12:00-13:00", "17:00-17:30"]},
                    "tuesday": {"enabled": true, "start_time": "08:00", "end_time": "20:00", "breaks": ["12:00-13:00", "17:00-17:30"]},
                    "wednesday": {"enabled": true, "start_time": "08:00", "end_time": "20:00", "breaks": ["12:00-13:00", "17:00-17:30"]},
                    "thursday": {"enabled": true, "start_time": "08:00", "end_time": "20:00", "breaks": ["12:00-13:00", "17:00-17:30"]},
                    "friday": {"enabled": true, "start_time": "08:00", "end_time": "20:00", "breaks": ["12:00-13:00", "17:00-17:30"]},
                    "saturday": {"enabled": true, "start_time": "09:00", "end_time": "14:00", "breaks": ["11:30-12:00"]},
                    "sunday": {"enabled": false, "start_time": "09:00", "end_time": "14:00", "breaks": []}
                },
                "break_schedule": [],
                "availability_calendar": {}
            },
            "interaction_patterns": {
                "greeting_style": "professional",
                "farewell_style": "professional",
                "error_handling": {
                    "approach": "professional",
                    "offer_solutions": true,
                    "request_clarification": true,
                    "escalation_threshold": 4
                },
                "clarification_preferences": {
                    "ask_questions": true,
                    "confirm_understanding": true,
                    "provide_examples": true,
                    "check_assumptions": true
                },
                "feedback_requests": true
            }
        },
        "resource_limits": {
            "max_concurrent_tasks": 5,
            "max_memory_mb": 4096,
            "max_execution_time_minutes": 45,
            "max_file_size_mb": 100,
            "max_api_calls_per_hour": 150,
            "cost_limits": {
                "daily_limit": 25.0,
                "weekly_limit": 125.0,
                "monthly_limit": 500.0,
                "per_task_limit": 12.0,
                "currency": "USD"
            }
        },
        "security_settings": {
            "access_level": "Administrator",
            "data_permissions": {
                "can_read_sensitive": true,
                "can_write_sensitive": true,
                "can_delete_data": true,
                "can_share_data": true,
                "data_retention_days": 180
            },
            "network_restrictions": {
                "allowed_domains": ["jira.com", "asana.com", "trello.com", "slack.com"],
                "blocked_domains": [],
                "require_https": true,
                "max_requests_per_minute": 120
            },
            "audit_settings": {
                "log_all_interactions": true,
                "log_data_access": true,
                "log_tool_usage": true,
                "retention_days": 365
            },
            "encryption_requirements": {
                "encrypt_data_at_rest": true,
                "encrypt_data_in_transit": true,
                "encryption_algorithm": "AES-256",
                "key_rotation_days": 60
            }
        }
    }).to_string()
}
