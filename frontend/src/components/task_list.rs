use leptos::*;
use crate::pages::dashboard::Task;

#[component]
pub fn TaskList(tasks: ReadSignal<Vec<Task>>) -> impl IntoView {
    view! {
        <div class="space-y-3">
            <For
                each=move || tasks.get()
                key=|task| task.id.clone()
                children=move |task| {
                    let status_color = match task.status.as_str() {
                        "completed" => "green",
                        "in_progress" => "blue",
                        "pending" => "yellow",
                        "failed" => "red",
                        _ => "gray",
                    };

                    view! {
                        <div class="border border-gray-200 rounded-lg p-3 hover:shadow-md transition-shadow">
                            <div class="flex justify-between items-start mb-2">
                                <h4 class="font-medium text-gray-800">
                                    {task.title}
                                </h4>
                                <span class=format!("inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-{}-100 text-{}-800", status_color, status_color)>
                                    {task.status}
                                </span>
                            </div>
                            
                            <div class="space-y-1 text-sm text-gray-600">
                                <p>
                                    "ID: " {task.id}
                                </p>
                                <Show when=move || task.assigned_agent.is_some()>
                                    <p>
                                        "Agent: " {task.assigned_agent.as_ref().unwrap().clone()}
                                    </p>
                                </Show>
                                <p>
                                    "Created: " {task.created_at.format("%Y-%m-%d %H:%M")}
                                </p>
                            </div>

                            <div class="mt-3 flex space-x-2">
                                <button class="text-blue-600 hover:text-blue-800 text-sm font-medium">
                                    "View Details"
                                </button>
                                <Show when=move || task.status == "pending">
                                    <button class="text-green-600 hover:text-green-800 text-sm font-medium">
                                        "Start Task"
                                    </button>
                                </Show>
                                <Show when=move || task.status == "in_progress">
                                    <button class="text-orange-600 hover:text-orange-800 text-sm font-medium">
                                        "Pause Task"
                                    </button>
                                </Show>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}
