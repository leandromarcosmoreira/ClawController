use leptos::*;
use leptos_router::*;

#[component]
pub fn Navigation() -> impl IntoView {
    let (mobile_menu_open, set_mobile_menu_open) = create_signal(false);

    view! {
        <nav class="bg-white shadow-lg">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex justify-between h-16">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <A href="/" class="text-2xl font-bold text-blue-600">
                                "ClawController"
                            </A>
                        </div>
                    </div>

                    // Desktop navigation
                    <div class="hidden md:flex items-center space-x-4">
                        <A href="/" class="nav-link">
                            "Dashboard"
                        </A>
                        <A href="/agents" class="nav-link">
                            "Agents"
                        </A>
                        <A href="/tasks" class="nav-link">
                            "Tasks"
                        </A>
                        <A href="/settings" class="nav-link">
                            "Settings"
                        </A>
                    </div>

                    // Mobile menu button
                    <div class="md:hidden flex items-center">
                        <button
                            on:click=move |_| set_mobile_menu_open.update(|open| *open = !*open)
                            class="mobile-menu-button"
                        >
                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>

            // Mobile menu
            <Show when=move || mobile_menu_open.get()>
                <div class="md:hidden">
                    <div class="px-2 pt-2 pb-3 space-y-1 sm:px-3">
                        <A href="/" class="mobile-nav-link">
                            "Dashboard"
                        </A>
                        <A href="/agents" class="mobile-nav-link">
                            "Agents"
                        </A>
                        <A href="/tasks" class="mobile-nav-link">
                            "Tasks"
                        </A>
                        <A href="/settings" class="mobile-nav-link">
                            "Settings"
                        </A>
                    </div>
                </div>
            </Show>
        </nav>
    }
}
