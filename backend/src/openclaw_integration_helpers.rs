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

// Original OpenClaw Integration Functions (Preserved for compatibility)

/// Get comprehensive agent configurations from OpenClaw
pub async fn get_openclaw_agent_configs_original(
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

// Helper functions for original implementation
pub fn parse_openclaw_agent_config(agent_entry: &Value, defaults: &Value) -> Option<OpenClawAgentConfig> {
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

pub fn get_agent_capabilities(config: Option<&OpenClawAgentConfig>) -> Value {
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

pub async fn apply_agent_config_to_db(pool: &SqlitePool, agent_id: &str, config: &OpenClawAgentConfig) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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

    Ok(config_hash)
}

pub fn validate_agent_config_internal(config: &OpenClawAgentConfig) -> Result<(), String> {
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
