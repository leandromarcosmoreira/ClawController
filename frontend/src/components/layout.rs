use leptos::*;
use leptos_router::*;
use crate::components::navigation::Navigation;
use crate::pages::{dashboard::Dashboard, agents::Agents, tasks::Tasks, settings::Settings};

#[component]
pub fn Layout() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50">
            <Navigation/>
            <main class="container mx-auto px-4 py-8">
                <Routes>
                    <Route path="/" view=Dashboard/>
                    <Route path="/agents" view=Agents/>
                    <Route path="/tasks" view=Tasks/>
                    <Route path="/settings" view=Settings/>
                </Routes>
            </main>
        </div>
    }
}
