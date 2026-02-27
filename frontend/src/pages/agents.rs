use leptos::*;

#[component]
pub fn Agents() -> impl IntoView {
    view! {
        <div class="agents-page">
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 mb-2">
                    "Agents"
                </h1>
                <p class="text-gray-600">
                    "Manage and monitor your AI agents"
                </p>
            </div>

            <div class="bg-white rounded-lg shadow-md p-6">
                <div class="flex justify-between items-center mb-6">
                    <h2 class="text-xl font-semibold text-gray-800">
                        "Agent Management"
                    </h2>
                    <button class="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 transition-colors">
                        "Add New Agent"
                    </button>
                </div>

                <div class="text-center py-8">
                    <p class="text-gray-500">
                        "Agent management interface coming soon..."
                    </p>
                </div>
            </div>
        </div>
    }
}
