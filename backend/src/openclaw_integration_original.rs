use crate::models::*;
use axum::{
    extract::{Path, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use sqlx::SqlitePool;
use chrono::Utc;
use std::collections::HashMap;
use serde_json::Value;
use sha2::{Sha256, Digest};

// Enhanced OpenClaw Configuration Management

/// Get comprehensive agent configurations from OpenClaw
pub async fn get_openclaw_agent_configs(
    State(state): State<crate::AppState>,
) -> Result<Json<Vec<OpenClawAgentConfig>>, (StatusCode, String)> {
    let openclaw_dir = std::env::var("OPENCLAW_STATE_DIR")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.openclaw", h)))
        .unwrap_or_else(|_| "/root/.openclaw".to_string());
    let config_path = format!("{}/openclaw.json", openclaw_dir);

    let content = tokio::fs::read_to_string(&config_path).await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("Cannot read openclaw config: {}", e)))?;

    let config: Value = serde_json::from_str(&content)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid openclaw.json: {}", e)))?;

    let agents_config = config.get("agents").ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Missing agents config".to_string()))?;
    let defaults = agents_config.get("defaults").unwrap_or(&Value::Object(Default::default()));
    let list = agents_config.get("list").and_then(|l| l.as_array()).unwrap_or(&[]);

    let mut enhanced_configs = Vec::new();

    for agent_entry in list {
        if let Some(agent_config) = parse_openclaw_agent_config(agent_entry, defaults) {
            enhanced_configs.push(agent_config);
        }
    }

    Ok(Json(enhanced_configs))
}

/// Get specific agent configuration from OpenClaw
pub async fn get_openclaw_agent_config(
    Path(agent_id): Path<String>,
    State(state): State<crate::AppState>,
) -> Result<Json<OpenClawAgentConfig>, (StatusCode, String)> {
    let configs = get_openclaw_agent_configs(State(state)).await?.0;
    
    let config = configs.into_iter()
        .find(|c| c.id == agent_id)
        .ok_or((StatusCode::NOT_FOUND, "Agent not found".to_string()))?;

    Ok(Json(config))
}

/// Sync all OpenClaw configurations to ClawController database
pub async fn sync_openclaw_configs(
    State(state): State<crate::AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let openclaw_configs = get_openclaw_agent_configs(State(state.clone())).await?.0;
    let mut synced_count = 0;
    let mut errors = Vec::new();

    for config in openclaw_configs {
        match sync_single_agent_config(&state.pool, &config).await {
            Ok(_) => synced_count += 1,
            Err(e) => errors.push(format!("Agent {}: {}", config.id, e)),
        }
    }

    Ok(Json(serde_json::json!({
        "synced": synced_count,
        "errors": errors
    })))
}

/// Apply specific configuration to an agent
pub async fn apply_agent_config(
    Path(agent_id): Path<String>,
    State(state): State<crate::AppState>,
    Json(config): Json<OpenClawAgentConfig>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Validate configuration
    validate_agent_config_internal(&config)?;

    // Apply to database
    let result = apply_agent_config_to_db(&state.pool, agent_id, &config).await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "applied": result
    })))
}

/// Get enhanced agent information with full OpenClaw integration
pub async fn fetch_enhanced_openclaw_agents(
    State(state): State<crate::AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Get basic agents from database
    let db_agents = sqlx::query_as::<sqlx::Sqlite, Agent>(
        "SELECT * FROM agents ORDER BY name"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Get OpenClaw configurations
    let openclaw_configs = get_openclaw_agent_configs(State(state.clone())).await?.0;
    let config_map: HashMap<String, OpenClawAgentConfig> = openclaw_configs
        .into_iter()
        .map(|c| (c.id.clone(), c))
        .collect();

    // Merge information
    let enhanced_agents: Vec<Value> = db_agents.into_iter().map(|agent| {
        let openclaw_config = config_map.get(&agent.id);
        
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
            "capabilities": get_agent_capabilities(openclaw_config)
        })
    }).collect();

    Ok(Json(serde_json::json!({
        "data": enhanced_agents,
        "total": enhanced_agents.len()
    })))
}

/// Get all parameters for a specific agent
pub async fn get_agent_parameters(
    Path(agent_id): Path<String>,
    State(state): State<crate::AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let agent = sqlx::query_as::<sqlx::Sqlite, Agent>(
        "SELECT * FROM agents WHERE id = ?"
    )
    .bind(&agent_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let openclaw_config = get_openclaw_agent_config(Path(agent_id), State(state)).await.ok();

    let parameters = serde_json::json!({
        "basic": {
            "id": agent.id,
            "name": agent.name,
            "role": agent.role,
            "status": agent.status,
            "workspace": agent.workspace,
            "primary_model": agent.primary_model,
            "fallback_model": agent.fallback_model
        },
        "advanced": {
            "image_model": agent.image_model,
            "sandbox_mode": agent.sandbox_mode,
            "thinking_default": agent.thinking_default,
            "verbose_default": agent.verbose_default,
            "max_concurrent": agent.max_concurrent,
            "timeout_seconds": agent.timeout_seconds,
            "context_tokens": agent.context_tokens,
            "heartbeat_enabled": agent.heartbeat_enabled,
            "subagents_enabled": agent.subagents_enabled,
            "human_delay_enabled": agent.human_delay_enabled,
            "block_streaming_enabled": agent.block_streaming_enabled,
            "context_pruning_enabled": agent.context_pruning_enabled
        },
        "json_configs": {
            "skills": agent.skills.and_then(|s| serde_json::from_str(&s).ok()),
            "tools_config": agent.tools_config.and_then(|s| serde_json::from_str(&s).ok()),
            "memory_search_config": agent.memory_search_config.and_then(|s| serde_json::from_str(&s).ok())
        },
        "openclaw_source": openclaw_config
    });

    Ok(Json(parameters))
}

/// Update agent parameters
pub async fn update_agent_parameters(
    Path(agent_id): Path<String>,
    State(state): State<crate::AppState>,
    Json(params): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Begin transaction for parameter history tracking
    let mut tx = state.pool.begin().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Get current agent state
    let current_agent = sqlx::query_as::<sqlx::Sqlite, Agent>(
        "SELECT * FROM agents WHERE id = ?"
    )
    .bind(&agent_id)
    .fetch_one(&mut *tx)
    .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    // Track changes
    let mut changes_made = Vec::new();

    // Update basic parameters
    if let Some(name) = params.get("name").and_then(|v| v.as_str()) {
        if current_agent.name != name {
            record_parameter_change(&mut tx, &agent_id, "name", &current_agent.name, name, "user").await?;
            changes_made.push("name");
        }
        sqlx::query("UPDATE agents SET name = ? WHERE id = ?")
            .bind(name)
            .bind(&agent_id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(primary_model) = params.get("primary_model").and_then(|v| v.as_str()) {
        let current = current_agent.primary_model.unwrap_or_default();
        if current != primary_model {
            record_parameter_change(&mut tx, &agent_id, "primary_model", &current, primary_model, "user").await?;
            changes_made.push("primary_model");
        }
        sqlx::query("UPDATE agents SET primary_model = ? WHERE id = ?")
            .bind(primary_model)
            .bind(&agent_id)
            .execute(&mut *tx)
            .await?;
    }

    // Update advanced parameters
    if let Some(thinking_default) = params.get("thinking_default").and_then(|v| v.as_str()) {
        let current = format!("{:?}", current_agent.thinking_default.unwrap_or(crate::models::ThinkingLevel::Off));
        if current != thinking_default {
            record_parameter_change(&mut tx, &agent_id, "thinking_default", &current, thinking_default, "user").await?;
            changes_made.push("thinking_default");
        }
        let thinking_enum = match thinking_default {
            "off" => crate::models::ThinkingLevel::Off,
            "minimal" => crate::models::ThinkingLevel::Minimal,
            "low" => crate::models::ThinkingLevel::Low,
            "medium" => crate::models::ThinkingLevel::Medium,
            "high" => crate::models::ThinkingLevel::High,
            "xhigh" => crate::models::ThinkingLevel::Xhigh,
            _ => return Err((StatusCode::BAD_REQUEST, "Invalid thinking_default value".to_string())),
        };
        sqlx::query("UPDATE agents SET thinking_default = ? WHERE id = ?")
            .bind(format!("{:?}", thinking_enum))
            .bind(&agent_id)
            .execute(&mut *tx)
            .await?;
    }

    // Update JSON configurations
    if let Some(skills) = params.get("skills") {
        let skills_json = serde_json::to_string(skills)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid skills JSON: {}", e)))?;
        sqlx::query("UPDATE agents SET skills = ? WHERE id = ?")
            .bind(&skills_json)
            .bind(&agent_id)
            .execute(&mut *tx)
            .await?;
        changes_made.push("skills");
    }

    // Commit transaction
    tx.commit().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "changes_made": changes_made
    })))
}

/// Get parameter change history for an agent
pub async fn get_agent_parameter_history(
    Path(agent_id): Path<String>,
    State(state): State<crate::AppState>,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    let history = sqlx::query_as::<sqlx::Sqlite, (String, String, String, String, String, String)>(
        "SELECT parameter_name, old_value, new_value, changed_by, changed_at 
         FROM agent_parameter_history 
         WHERE agent_id = ? 
         ORDER BY changed_at DESC"
    )
    .bind(&agent_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let formatted_history: Vec<Value> = history.into_iter().map(|(param, old, new, changed_by, changed_at)| {
        serde_json::json!({
            "parameter": param,
            "old_value": old,
            "new_value": new,
            "changed_by": changed_by,
            "changed_at": changed_at
        })
    }).collect();

    Ok(Json(formatted_history))
}

/// Validate agent configuration
pub async fn validate_agent_config(
    Json(config): Json<OpenClawAgentConfig>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match validate_agent_config_internal(&config) {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "valid",
            "message": "Configuration is valid"
        }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e))
    }
}

/// Export agent configurations
pub async fn export_agent_configs(
    State(state): State<crate::AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let agents = sqlx::query_as::<sqlx::Sqlite, Agent>(
        "SELECT * FROM agents ORDER BY name"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let export_data: Vec<Value> = agents.into_iter().map(|agent| {
        serde_json::json!({
            "id": agent.id,
            "name": agent.name,
            "role": agent.role,
            "basic_config": {
                "primary_model": agent.primary_model,
                "fallback_model": agent.fallback_model,
                "workspace": agent.workspace
            },
            "advanced_config": {
                "image_model": agent.image_model,
                "sandbox_mode": agent.sandbox_mode,
                "thinking_default": agent.thinking_default,
                "verbose_default": agent.verbose_default,
                "max_concurrent": agent.max_concurrent,
                "timeout_seconds": agent.timeout_seconds,
                "context_tokens": agent.context_tokens
            },
            "feature_flags": {
                "heartbeat_enabled": agent.heartbeat_enabled,
                "subagents_enabled": agent.subagents_enabled,
                "human_delay_enabled": agent.human_delay_enabled,
                "block_streaming_enabled": agent.block_streaming_enabled,
                "context_pruning_enabled": agent.context_pruning_enabled
            },
            "json_configs": {
                "skills": agent.skills.and_then(|s| serde_json::from_str(&s).ok()),
                "tools_config": agent.tools_config.and_then(|s| serde_json::from_str(&s).ok()),
                "memory_search_config": agent.memory_search_config.and_then(|s| serde_json::from_str(&s).ok()))
            }
        })
    }).collect();

    Ok(Json(serde_json::json!({
        "agents": export_data,
        "exported_at": Utc::now(),
        "total": export_data.len()
    })))
}

/// Import agent configurations
pub async fn import_agent_configs(
    State(state): State<crate::AppState>,
    Json(import_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let agents = import_data.get("agents").and_then(|a| a.as_array())
        .ok_or((StatusCode::BAD_REQUEST, "Invalid import format".to_string()))?;

    let mut imported_count = 0;
    let mut errors = Vec::new();

    for agent_data in agents {
        match import_single_agent_config(&state.pool, agent_data).await {
            Ok(_) => imported_count += 1,
            Err(e) => errors.push(format!("Agent import failed: {}", e)),
        }
    }

    Ok(Json(serde_json::json!({
        "imported": imported_count,
        "errors": errors
    })))
}

// Helper Functions

fn parse_openclaw_agent_config(agent_entry: &Value, defaults: &Value) -> Option<OpenClawAgentConfig> {
    let id = agent_entry.get("id")?.as_str()?;
    
    Some(OpenClawAgentConfig {
        id: id.to_string(),
        name: agent_entry.get("name").or_else(|| defaults.get("name")).and_then(|v| v.as_str()).map(|s| s.to_string()),
        workspace: agent_entry.get("workspace").or_else(|| defaults.get("workspace")).and_then(|v| v.as_str()).map(|s| s.to_string()),
        agent_dir: agent_entry.get("agentDir").and_then(|v| v.as_str()).map(|s| s.to_string()),
        model: parse_model_config(agent_entry.get("model").or_else(|| defaults.get("model"))),
        image_model: parse_model_config(agent_entry.get("imageModel").or_else(|| defaults.get("imageModel"))),
        skills: agent_entry.get("skills").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
        memory_search: parse_memory_search_config(agent_entry.get("memorySearch").or_else(|| defaults.get("memorySearch"))),
        human_delay: parse_human_delay_config(agent_entry.get("humanDelay").or_else(|| defaults.get("humanDelay"))),
        heartbeat: parse_heartbeat_config(agent_entry.get("heartbeat").or_else(|| defaults.get("heartbeat"))),
        identity: parse_identity_config(agent_entry.get("identity").or_else(|| defaults.get("identity"))),
        group_chat: parse_group_chat_config(agent_entry.get("groupChat").or_else(|| defaults.get("groupChat"))),
        subagents: parse_subagents_config(agent_entry.get("subagents").or_else(|| defaults.get("subagents"))),
        sandbox: parse_sandbox_config(agent_entry.get("sandbox").or_else(|| defaults.get("sandbox"))),
        params: agent_entry.get("params").or_else(|| defaults.get("params")).cloned(),
        tools: parse_tools_config(agent_entry.get("tools").or_else(|| defaults.get("tools"))),
    })
}

fn parse_model_config(model_value: Option<&Value>) -> Option<AgentModelConfig> {
    match model_value {
        Some(Value::String(model)) => Some(AgentModelConfig {
            primary: Some(model.clone()),
            fallbacks: None,
        }),
        Some(Value::Object(obj)) => Some(AgentModelConfig {
            primary: obj.get("primary").and_then(|v| v.as_str()).map(|s| s.to_string()),
            fallbacks: obj.get("fallbacks").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
        }),
        _ => None,
    }
}

fn parse_memory_search_config(value: Option<&Value>) -> Option<MemorySearchConfig> {
    value.and_then(|v| {
        Some(MemorySearchConfig {
            enabled: v.get("enabled").and_then(|e| e.as_bool()),
            max_results: v.get("maxResults").and_then(|r| r.as_i64()).map(|r| r as i32),
            threshold: v.get("threshold").and_then(|t| t.as_f64()),
        })
    })
}

fn parse_human_delay_config(value: Option<&Value>) -> Option<HumanDelayConfig> {
    value.and_then(|v| {
        Some(HumanDelayConfig {
            enabled: v.get("enabled").and_then(|e| e.as_bool()),
            min_seconds: v.get("minSeconds").and_then(|s| s.as_f64()),
            max_seconds: v.get("maxSeconds").and_then(|s| s.as_f64()),
        })
    })
}

fn parse_heartbeat_config(value: Option<&Value>) -> Option<HeartbeatConfig> {
    value.and_then(|v| {
        Some(HeartbeatConfig {
            enabled: v.get("enabled").and_then(|e| e.as_bool()),
            every: v.get("every").and_then(|e| e.as_str()).map(|s| s.to_string()),
            active_hours: parse_active_hours_config(v.get("activeHours")),
            model: v.get("model").and_then(|m| m.as_str()).map(|s| s.to_string()),
            session: v.get("session").and_then(|s| s.as_str()).map(|s| s.to_string()),
            target: v.get("target").and_then(|t| t.as_str()).map(|s| s.to_string()),
            prompt: v.get("prompt").and_then(|p| p.as_str()).map(|s| s.to_string()),
        })
    })
}

fn parse_active_hours_config(value: Option<&Value>) -> Option<ActiveHoursConfig> {
    value.and_then(|v| {
        Some(ActiveHoursConfig {
            start: v.get("start").and_then(|s| s.as_str()).map(|s| s.to_string()),
            end: v.get("end").and_then(|e| e.as_str()).map(|s| s.to_string()),
            timezone: v.get("timezone").and_then(|t| t.as_str()).map(|s| s.to_string()),
        })
    })
}

fn parse_identity_config(value: Option<&Value>) -> Option<IdentityConfig> {
    value.and_then(|v| {
        Some(IdentityConfig {
            name: v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()),
            bio: v.get("bio").and_then(|b| b.as_str()).map(|s| s.to_string()),
        })
    })
}

fn parse_group_chat_config(value: Option<&Value>) -> Option<GroupChatConfig> {
    value.and_then(|v| {
        Some(GroupChatConfig {
            enabled: v.get("enabled").and_then(|e| e.as_bool()),
            mention_handling: v.get("mentionHandling").and_then(|m| m.as_str()).map(|s| s.to_string()),
        })
    })
}

fn parse_subagents_config(value: Option<&Value>) -> Option<SubagentsConfig> {
    value.and_then(|v| {
        Some(SubagentsConfig {
            allow_agents: v.get("allowAgents").and_then(|a| a.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
            model: parse_model_config(v.get("model")),
        })
    })
}

fn parse_sandbox_config(value: Option<&Value>) -> Option<SandboxConfig> {
    value.and_then(|v| {
        Some(SandboxConfig {
            mode: v.get("mode").and_then(|m| m.as_str()).map(|s| s.to_string()),
            docker: parse_docker_sandbox_config(v.get("docker")),
        })
    })
}

fn parse_docker_sandbox_config(value: Option<&Value>) -> Option<DockerSandboxConfig> {
    value.and_then(|v| {
        Some(DockerSandboxConfig {
            image: v.get("image").and_then(|i| i.as_str()).map(|s| s.to_string()),
            memory_mb: v.get("memoryMb").and_then(|m| m.as_i64()).map(|m| m as i32),
            cpu_cores: v.get("cpuCores").and_then(|c| c.as_f64()),
        })
    })
}

fn parse_tools_config(value: Option<&Value>) -> Option<ToolsConfig> {
    value.and_then(|v| {
        Some(ToolsConfig {
            exec: parse_exec_tools_config(v.get("exec")),
            file_ops: parse_file_ops_config(v.get("fileOps")),
            web: parse_web_tools_config(v.get("web")),
        })
    })
}

fn parse_exec_tools_config(value: Option<&Value>) -> Option<ExecToolsConfig> {
    value.and_then(|v| {
        Some(ExecToolsConfig {
            enabled: v.get("enabled").and_then(|e| e.as_bool()),
            host: v.get("host").and_then(|h| h.as_str()).map(|s| s.to_string()),
            safe_bins: v.get("safeBins").and_then(|b| b.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
            trusted_dirs: v.get("trustedDirs").and_then(|d| d.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
        })
    })
}

fn parse_file_ops_config(value: Option<&Value>) -> Option<FileOpsConfig> {
    value.and_then(|v| {
        Some(FileOpsConfig {
            enabled: v.get("enabled").and_then(|e| e.as_bool()),
            read_paths: v.get("readPaths").and_then(|p| p.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
            write_paths: v.get("writePaths").and_then(|p| p.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
        })
    })
}

fn parse_web_tools_config(value: Option<&Value>) -> Option<WebToolsConfig> {
    value.and_then(|v| {
        Some(WebToolsConfig {
            enabled: v.get("enabled").and_then(|e| e.as_bool()),
            allow_domains: v.get("allowDomains").and_then(|d| d.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
            block_domains: v.get("blockDomains").and_then(|d| d.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
        })
    })
}

fn get_agent_capabilities(config: Option<&OpenClawAgentConfig>) -> Value {
    match config {
        Some(c) => serde_json::json!({
            "models": {
                "text": c.model.as_ref().map(|m| m.primary.as_ref()).or(Some("default")).unwrap(),
                "image": c.image_model.as_ref().and_then(|m| m.primary.as_ref())
            },
            "features": {
                "memory_search": c.memory_search.as_ref().and_then(|m| m.enabled).unwrap_or(false),
                "heartbeat": c.heartbeat.as_ref().and_then(|h| h.enabled).unwrap_or(false),
                "human_delay": c.human_delay.as_ref().and_then(|h| h.enabled).unwrap_or(false),
                "subagents": c.subagents.is_some(),
                "sandbox": c.sandbox.as_ref().and_then(|s| s.mode.as_ref()).map(|m| m != "off").unwrap_or(false)
            },
            "tools": {
                "exec": c.tools.as_ref().and_then(|t| t.exec.as_ref()).and_then(|e| e.enabled).unwrap_or(false),
                "file_ops": c.tools.as_ref().and_then(|t| t.file_ops.as_ref()).and_then(|f| f.enabled).unwrap_or(false),
                "web": c.tools.as_ref().and_then(|t| t.web.as_ref()).and_then(|w| w.enabled).unwrap_or(false)
            },
            "skills": c.skills.as_ref().map(|s| s.len()).unwrap_or(0)
        }),
        None => serde_json::json!({
            "models": {"text": "unknown", "image": null},
            "features": {
                "memory_search": false,
                "heartbeat": false,
                "human_delay": false,
                "subagents": false,
                "sandbox": false
            },
            "tools": {
                "exec": false,
                "file_ops": false,
                "web": false
            },
            "skills": 0
        })
    }
}

async fn sync_single_agent_config(pool: &SqlitePool, config: &OpenClawAgentConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Calculate config hash
    let config_json = serde_json::to_string(config)?;
    let config_hash = format!("{:x}", Sha256::digest(config_json.as_bytes()));

    // Check if already synced
    let existing = sqlx::query_as::<sqlx::Sqlite, (String,)>(
        "SELECT openclaw_config_hash FROM agents WHERE id = ?"
    )
    .bind(&config.id)
    .fetch_one(pool)
    .await;

    match existing {
        Ok((hash,)) if hash == config_hash => {
            // Already up to date
            return Ok(());
        }
        _ => {
            // Need to update
            apply_agent_config_to_db(pool, &config.id, config).await?;
        }
    }

    Ok(())
}

async fn apply_agent_config_to_db(pool: &SqlitePool, agent_id: &str, config: &OpenClawAgentConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config_json = serde_json::to_string(config)?;
    let config_hash = format!("{:x}", Sha256::digest(config_json.as_bytes()));

    // Convert complex types to JSON strings
    let skills_json = config.skills.as_ref().map(|s| serde_json::to_string(s)).transpose()?;
    let tools_config_json = config.tools.as_ref().map(|t| serde_json::to_string(t)).transpose()?;
    let memory_search_json = config.memory_search.as_ref().map(|m| serde_json::to_string(m)).transpose()?;

    // Upsert agent with full configuration
    sqlx::query(
        r#"
        INSERT OR REPLACE INTO agents (
            id, name, role, status, workspace, agent_dir,
            primary_model, fallback_model, image_model,
            sandbox_mode, thinking_default, verbose_default,
            max_concurrent, timeout_seconds, context_tokens,
            skills, tools_config, memory_search_config,
            heartbeat_enabled, subagents_enabled, human_delay_enabled,
            block_streaming_enabled, context_pruning_enabled,
            openclaw_config_hash, created_at
        ) VALUES (
            ?, ?, 'SPC', 'IDLE', ?, ?,
            ?, ?, ?,
            ?, ?, ?,
            ?, ?, ?,
            ?, ?, ?,
            ?, ?, ?,
            ?, ?,
            ?, CURRENT_TIMESTAMP
        )
        "#
    )
    .bind(&config.id)
    .bind(&config.name.as_deref().unwrap_or(&config.id))
    .bind(&config.workspace)
    .bind(&config.agent_dir)
    .bind(&config.model.as_ref().and_then(|m| m.primary.as_ref()))
    .bind(&config.model.as_ref().and_then(|m| m.fallbacks.as_ref()).and_then(|f| f.first().map(|s| s.to_string())))
    .bind(&config.image_model.as_ref().and_then(|m| m.primary.as_ref()))
    .bind(&config.sandbox.as_ref().and_then(|s| s.mode.as_ref()))
    .bind(&config.params.as_ref().and_then(|p| p.get("thinkingDefault")).and_then(|v| v.as_str()))
    .bind(&config.params.as_ref().and_then(|p| p.get("verboseDefault")).and_then(|v| v.as_str()))
    .bind(&config.params.as_ref().and_then(|p| p.get("maxConcurrent")).and_then(|v| v.as_i64()).map(|i| i as i32))
    .bind(&config.params.as_ref().and_then(|p| p.get("timeoutSeconds")).and_then(|v| v.as_i64()).map(|i| i as i32))
    .bind(&config.params.as_ref().and_then(|p| p.get("contextTokens")).and_then(|v| v.as_i64()).map(|i| i as i32))
    .bind(&skills_json)
    .bind(&tools_config_json)
    .bind(&memory_search_json)
    .bind(&config.heartbeat.as_ref().and_then(|h| h.enabled))
    .bind(&config.subagents.as_ref().map(|_| true))
    .bind(&config.human_delay.as_ref().and_then(|h| h.enabled))
    .bind(&config.params.as_ref().and_then(|p| p.get("blockStreamingDefault")).and_then(|v| v.as_str()).map(|s| s == "on"))
    .bind(&config.params.as_ref().and_then(|p| p.get("contextPruning")).and_then(|v| v.get("enabled")).and_then(|e| e.as_bool()))
    .bind(&config_hash)
    .execute(pool)
    .await?;

    // Store configuration snapshot
    let snapshot_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO openclaw_config_snapshots (id, agent_id, config_hash, raw_config, is_active) VALUES (?, ?, ?, ?, 1)"
    )
    .bind(&snapshot_id)
    .bind(&config.id)
    .bind(&config_hash)
    .bind(&config_json)
    .execute(pool)
    .await?;

    Ok(())
}

async fn record_parameter_change(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    agent_id: &str,
    parameter_name: &str,
    old_value: &str,
    new_value: &str,
    changed_by: &str,
) -> Result<(), sqlx::Error> {
    let history_id = uuid::Uuid::new_v4().to_string();
    
    sqlx::query(
        "INSERT INTO agent_parameter_history (id, agent_id, parameter_name, old_value, new_value, changed_by) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&history_id)
    .bind(agent_id)
    .bind(parameter_name)
    .bind(old_value)
    .bind(new_value)
    .bind(changed_by)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

fn validate_agent_config_internal(config: &OpenClawAgentConfig) -> Result<(), String> {
    // Basic validation
    if config.id.is_empty() {
        return Err("Agent ID cannot be empty".to_string());
    }

    // Model validation
    if let Some(model) = &config.model {
        if let Some(primary) = &model.primary {
            if primary.is_empty() {
                return Err("Primary model cannot be empty".to_string());
            }
        }
    }

    // Sandbox validation
    if let Some(sandbox) = &config.sandbox {
        if let Some(mode) = &sandbox.mode {
            if !["off", "on", "docker"].contains(&mode.as_str()) {
                return Err("Invalid sandbox mode. Must be 'off', 'on', or 'docker'".to_string());
            }
        }
    }

    // Skills validation
    if let Some(skills) = &config.skills {
        if skills.is_empty() {
            return Err("Skills array cannot be empty when specified".to_string());
        }
    }

    Ok(())
}

async fn import_single_agent_config(pool: &SqlitePool, agent_data: &Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let id = agent_data.get("id").and_then(|v| v.as_str())
        .ok_or("Missing agent id")?;

    let name = agent_data.get("name").and_then(|v| v.as_str()).unwrap_or(id);

    // Extract and rebuild OpenClawAgentConfig
    let config = OpenClawAgentConfig {
        id: id.to_string(),
        name: Some(name.to_string()),
        workspace: agent_data.get("basic_config").and_then(|b| b.get("workspace")).and_then(|w| w.as_str()).map(|s| s.to_string()),
        agent_dir: None,
        model: None, // Would need more complex parsing
        image_model: None,
        skills: agent_data.get("json_configs").and_then(|j| j.get("skills")).and_then(|s| s.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
        memory_search: None,
        human_delay: None,
        heartbeat: None,
        identity: None,
        group_chat: None,
        subagents: None,
        sandbox: None,
        params: None,
        tools: None,
    };

    apply_agent_config_to_db(pool, id, &config).await?;
    Ok(())
}
