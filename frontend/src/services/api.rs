use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use leptos::*;

const API_BASE_URL: &str = "http://localhost:8080/api";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub status: String,
    pub last_seen: String,
    pub task_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: String,
    pub assigned_agent: Option<String>,
    pub created_at: String,
}

pub async fn fetch_agents() -> Result<Vec<Agent>, String> {
    let response = Request::get(&format!("{}/agents", API_BASE_URL))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch agents: {}", e))?;

    if response.ok() {
        let api_response: ApiResponse<Vec<Agent>> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if api_response.success {
            Ok(api_response.data.unwrap_or_default())
        } else {
            Err(api_response.error.unwrap_or("Unknown error".to_string()))
        }
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

pub async fn fetch_tasks() -> Result<Vec<Task>, String> {
    let response = Request::get(&format!("{}/tasks", API_BASE_URL))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch tasks: {}", e))?;

    if response.ok() {
        let api_response: ApiResponse<Vec<Task>> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if api_response.success {
            Ok(api_response.data.unwrap_or_default())
        } else {
            Err(api_response.error.unwrap_or("Unknown error".to_string()))
        }
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

pub async fn create_agent(agent_data: serde_json::Value) -> Result<Agent, String> {
    let response = Request::post(&format!("{}/agents", API_BASE_URL))
        .json(&agent_data)
        .map_err(|e| format!("Failed to serialize agent data: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to create agent: {}", e))?;

    if response.ok() {
        let api_response: ApiResponse<Agent> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if api_response.success {
            Ok(api_response.data.unwrap())
        } else {
            Err(api_response.error.unwrap_or("Unknown error".to_string()))
        }
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

pub async fn create_task(task_data: serde_json::Value) -> Result<Task, String> {
    let response = Request::post(&format!("{}/tasks", API_BASE_URL))
        .json(&task_data)
        .map_err(|e| format!("Failed to serialize task data: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to create task: {}", e))?;

    if response.ok() {
        let api_response: ApiResponse<Task> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if api_response.success {
            Ok(api_response.data.unwrap())
        } else {
            Err(api_response.error.unwrap_or("Unknown error".to_string()))
        }
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

// Reactive API hooks
pub fn use_agents() -> ReadSignal<Vec<Agent>> {
    let (agents, set_agents) = create_signal(Vec::new());
    
    spawn_local(async move {
        match fetch_agents().await {
            Ok(data) => set_agents.set(data),
            Err(e) => web_sys::console::log_1(&format!("Error fetching agents: {}", e).into()),
        }
    });

    agents
}

pub fn use_tasks() -> ReadSignal<Vec<Task>> {
    let (tasks, set_tasks) = create_signal(Vec::new());
    
    spawn_local(async move {
        match fetch_tasks().await {
            Ok(data) => set_tasks.set(data),
            Err(e) => web_sys::console::log_1(&format!("Error fetching tasks: {}", e).into()),
        }
    });

    tasks
}
