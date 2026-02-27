use leptos::*;

#[component]
pub fn Settings() -> impl IntoView {
    view! {
        <div class="settings-page">
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 mb-2">
                    "Settings"
                </h1>
                <p class="text-gray-600">
                    "Configure ClawController preferences"
                </p>
            </div>

            <div class="bg-white rounded-lg shadow-md p-6">
                <h2 class="text-xl font-semibold text-gray-800 mb-6">
                    "General Settings"
                </h2>

                <div class="space-y-6">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            "API Endpoint"
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                            placeholder="http://localhost:8080"
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            "WebSocket URL"
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                            placeholder="ws://localhost:8080/ws"
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            "Refresh Interval (seconds)"
                        </label>
                        <input
                            type="number"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                            value="5"
                        />
                    </div>

                    <div class="flex items-center">
                        <input
                            type="checkbox"
                            id="notifications"
                            class="mr-2"
                        />
                        <label for="notifications" class="text-sm text-gray-700">
                            "Enable desktop notifications"
                        </label>
                    </div>
                </div>

                <div class="mt-8">
                    <button class="bg-blue-600 text-white px-6 py-2 rounded-md hover:bg-blue-700 transition-colors">
                        "Save Settings"
                    </button>
                </div>
            </div>
        </div>
    }
}
