use leptos::*;
use crate::pages::dashboard::Agent;

#[component]
pub fn AgentCard(agent: Agent) -> impl IntoView {
    let status_color = match agent.status.as_str() {
        "active" => "green",
        "idle" => "yellow",
        "error" => "red",
        _ => "gray",
    };

    view! {
        <div class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow">
            <div class="flex justify-between items-start mb-2">
                <h3 class="text-lg font-semibold text-gray-800">
                    {agent.name}
                </h3>
                <span class=format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-{}-100 text-{}-800", status_color, status_color)>
                    {agent.status}
                </span>
            </div>
            
            <div class="space-y-1 text-sm text-gray-600">
                <p>
                    "ID: " {agent.id}
                </p>
                <p>
                    "Tasks: " {agent.task_count}
                </p>
                <p>
                    "Last seen: " {agent.last_seen.format("%Y-%m-%d %H:%M:%S UTC")}
                </p>
            </div>

            <div class="mt-4 flex space-x-2">
                <button class="text-blue-600 hover:text-blue-800 text-sm font-medium">
                    "View Details"
                </button>
                <button class="text-red-600 hover:text-red-800 text-sm font-medium">
                    "Stop Agent"
                </button>
            </div>
        </div>
    }
}
