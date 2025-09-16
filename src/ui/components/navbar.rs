use leptos::*;
use leptos_router::*;

#[component]
pub fn Navbar() -> impl IntoView {
    let (is_menu_open, set_menu_open) = create_signal(false);

    view! {
        <nav class="bg-white dark:bg-gray-800 shadow-lg">
            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    // Logo and brand
                    <div class="flex items-center space-x-4">
                        <a href="/" class="flex items-center space-x-2">
                            <img
                                src="/assets/icons/drift-32x32.png"
                                alt="Drift Logo"
                                class="w-8 h-8 rounded-lg"
                            />
                            <span class="text-xl font-bold text-gray-900 dark:text-white">
                                "Drift"
                            </span>
                        </a>
                    </div>

                    // Desktop navigation
                    <div class="hidden md:flex items-center space-x-8">
                        <NavLink href="/" text="Dashboard" icon="🏠"/>
                        <NavLink href="/repositories" text="Repositories" icon="📦"/>
                        <NavLink href="/bolt" text="Bolt" icon="⚡"/>
                        <NavLink href="/settings" text="Settings" icon="⚙️"/>
                    </div>

                    // User menu and mobile menu button
                    <div class="flex items-center space-x-4">
                        // Theme toggle
                        <button
                            class="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 transition-colors"
                            onclick=move |_| {
                                // TODO: Implement theme toggle
                            }
                        >
                            "🌙"
                        </button>

                        // User menu
                        <div class="relative">
                            <button class="flex items-center space-x-2 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
                                <div class="w-8 h-8 bg-gray-300 dark:bg-gray-600 rounded-full flex items-center justify-center">
                                    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                                        "A"
                                    </span>
                                </div>
                                <span class="hidden md:block text-sm font-medium text-gray-700 dark:text-gray-300">
                                    "admin"
                                </span>
                            </button>
                        </div>

                        // Mobile menu button
                        <button
                            class="md:hidden p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 transition-colors"
                            onclick=move |_| set_menu_open.update(|open| *open = !*open)
                        >
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                            </svg>
                        </button>
                    </div>
                </div>

                // Mobile menu
                <div
                    class=format!(
                        "md:hidden {}",
                        if is_menu_open() { "block" } else { "hidden" }
                    )
                >
                    <div class="px-2 pt-2 pb-3 space-y-1 border-t border-gray-200 dark:border-gray-700">
                        <MobileNavLink href="/" text="Dashboard" icon="🏠"/>
                        <MobileNavLink href="/repositories" text="Repositories" icon="📦"/>
                        <MobileNavLink href="/bolt" text="Bolt" icon="⚡"/>
                        <MobileNavLink href="/settings" text="Settings" icon="⚙️"/>
                    </div>
                </div>
            </div>
        </nav>
    }
}

#[component]
fn NavLink(href: &'static str, text: &'static str, icon: &'static str) -> impl IntoView {
    let location = use_location();
    let is_active = move || location.pathname.get() == href;

    view! {
        <a
            href=href
            class=move || format!(
                "flex items-center space-x-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors {}",
                if is_active() {
                    "text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20"
                } else {
                    "text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700"
                }
            )
        >
            <span>{icon}</span>
            <span>{text}</span>
        </a>
    }
}

#[component]
fn MobileNavLink(href: &'static str, text: &'static str, icon: &'static str) -> impl IntoView {
    let location = use_location();
    let is_active = move || location.pathname.get() == href;

    view! {
        <a
            href=href
            class=move || format!(
                "flex items-center space-x-3 px-3 py-2 rounded-lg text-base font-medium transition-colors {}",
                if is_active() {
                    "text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20"
                } else {
                    "text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700"
                }
            )
        >
            <span>{icon}</span>
            <span>{text}</span>
        </a>
    }
}