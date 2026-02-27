use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod components;
mod pages;
mod services;

use components::layout::Layout;
use pages::{dashboard::Dashboard, agents::Agents, tasks::Tasks, settings::Settings};

#[component]
pub fn App() -> impl IntoView {
    // Provides context for Meta tags
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/clawcontroller-frontend.css"/>
        
        <Router>
            <Routes>
                <Route path="/" view=Layout>
                    <Route path="/" view=Dashboard/>
                    <Route path="/agents" view=Agents/>
                    <Route path="/tasks" view=Tasks/>
                    <Route path="/settings" view=Settings/>
                </Route>
            </Routes>
        </Router>
    }
}
