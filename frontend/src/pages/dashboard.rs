use leptos::*;
use crate::components::{agent_card::AgentCard, task_list::TaskList};

#[derive(Clone, Debug)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub status: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub task_count: u32,
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: String,
    pub assigned_agent: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let (agents, set_agents) = create_signal(Vec::<Agent>::new());
    let (tasks, set_tasks) = create_signal(Vec::<Task>::new());
    let (loading, set_loading) = create_signal(true);

    // Mock data for demonstration
    create_effect(move |_| {
        set_loading.set(true);
        
        // Simulate API call
        setTimeout(move || {
            let mock_agents = vec![
                Agent {
                    id: "agent-1".to_string(),
                    name: "Web Scraper".to_string(),
                    status: "active".to_string(),
                    last_seen: chrono::Utc::now(),
                    task_count: 5,
                },
                Agent {
                    id: "agent-2".to_string(),
                    name: "Data Processor".to_string(),
                    status: "idle".to_string(),
                    last_seen: chrono::Utc::now(),
                    task_count: 0,
                },
            ];

            let mock_tasks = vec![
                Task {
                    id: "task-1".to_string(),
                    title: "Scrape product data".to_string(),
                    status: "in_progress".to_string(),
                    assigned_agent: Some("agent-1".to_string()),
                    created_at: chrono::Utc::now(),
                },
                Task {
                    id: "task-2".to_string(),
                    title: "Process customer data".to_string(),
                    status: "pending".to_string(),
                    assigned_agent: None,
                    created_at: chrono::Utc::now(),
                },
            ];

            set_agents.set(mock_agents);
            set_tasks.set(mock_tasks);
            set_loading.set(false);
        }, 1000);
    });

    view! {
        <div class="dashboard">
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 mb-2">
                    "Dashboard"
                </h1>
                <p class="text-gray-600">
                    "Monitor your AI agents and tasks"
                </p>
            </div>

            <Show when=move || loading.get()>
                <div class="text-center py-8">
                    <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    <p class="mt-2 text-gray-600">"Loading dashboard..."</p>
                </div>
            </Show>

            <Show when=move || !loading.get()>
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    // Agents Section
                    <div class="bg-white rounded-lg shadow-md p-6">
                        <h2 class="text-xl font-semibold text-gray-800 mb-4">
                            "Active Agents"
                        </h2>
                        <div class="space-y-4">
                            <For
                                each=move || agents.get()
                                key=|agent| agent.id.clone()
                                children=move |agent| {
                                    view! {
                                        <AgentCard agent=agent/>
                                    }
                                }
                            />
                        </div>
                    </div>

                    // Tasks Section
                    <div class="bg-white rounded-lg shadow-md p-6">
                        <h2 class="text-xl font-semibold text-gray-800 mb-4">
                            "Recent Tasks"
                        </h2>
                        <TaskList tasks=tasks/>
                    </div>
                </div>

                // Stats Overview
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mt-8">
                    <div class="bg-white rounded-lg shadow-md p-6">
                        <h3 class="text-lg font-semibold text-gray-800 mb-2">
                            "Total Agents"
                        </h3>
                        <p class="text-3xl font-bold text-blue-600">
                            {agents.get().len()}
                        </p>
                    </div>
                    <div class="bg-white rounded-lg shadow-md p-6">
                        <h3 class="text-lg font-semibold text-gray-800 mb-2">
                            "Active Tasks"
                        </h3>
                        <p class="text-3xl font-bold text-green-600">
                            {tasks.get().iter().filter(|t| t.status == "in_progress").count()}
                        </p>
                    </div>
                    <div class="bg-white rounded-lg shadow-md p-6">
                        <h3 class="text-lg font-semibold text-gray-800 mb-2">
                            "Pending Tasks"
                        </h3>
                        <p class="text-3xl font-bold text-yellow-600">
                            {tasks.get().iter().filter(|t| t.status == "pending").count()}
                        </p>
                    </div>
                </div>
            </Show>
        </div>
    }
}
