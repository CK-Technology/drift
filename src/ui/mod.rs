use axum::{
    body::Body,
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_meta::*;
use leptos_router::*;

use crate::server::AppState;

pub mod components;
pub mod pages;

pub fn router() -> Router<AppState> {
    let leptos_options = LeptosOptions::builder()
        .output_name("drift")
        .site_pkg_dir("pkg")
        .build();

    Router::new()
        .leptos_routes(&leptos_options, generate_route_list(|| view! { <App/> }), || view! { <App/> })
        .route("/favicon.ico", get(favicon))
        .fallback(leptos_axum::file_and_error_handler(shell))
}

async fn favicon() -> impl IntoResponse {
    // Return proper favicon - in production this would serve the actual favicon file
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
        <rect width="100" height="100" fill="#1e3a8a"/>
        <path d="M20 30 L80 30 L70 50 L60 40 L40 40 L30 50 Z" fill="#06b6d4"/>
        <path d="M35 55 Q50 45 65 55 Q50 65 35 55" fill="#0ea5e9"/>
        <path d="M45 20 L55 35 L50 40 L45 35 Z" fill="#22d3ee"/>
    </svg>"#;

    (
        [("content-type", "image/svg+xml")],
        svg
    )
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html lang="en"/>
        <Title text="Drift Registry - Modern OCI Registry"/>
        <Meta name="description" content="Drift - A modern, high-performance OCI Registry + Web UI for Bolt, Docker, and Podman"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        // Include Tailwind CSS
        <Link rel="stylesheet" href="https://cdn.tailwindcss.com"/>

        // Include custom CSS
        <Style>
            {include_str!("../static/styles.css")}
        </Style>

        <Router>
            <Routes>
                <Route path="/" view=HomePage/>
                <Route path="/repositories" view=RepositoriesPage/>
                <Route path="/repositories/:name" view=RepositoryDetailPage/>
                <Route path="/bolt" view=BoltPage/>
                <Route path="/bolt/profiles" view=BoltProfilesPage/>
                <Route path="/bolt/plugins" view=BoltPluginsPage/>
                <Route path="/settings" view=SettingsPage/>
                <Route path="/auth/login" view=LoginPage/>
                <Route path="/*any" view=NotFoundPage/>
            </Routes>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <components::navbar::Navbar/>

            <main class="container mx-auto px-4 py-8">
                <div class="text-center mb-12">
                    <h1 class="text-4xl md:text-6xl font-bold text-gray-900 dark:text-white mb-4">
                        "🌊 Drift Registry"
                    </h1>
                    <p class="text-xl text-gray-600 dark:text-gray-300 mb-8">
                        "Modern, high-performance OCI Registry + Web UI"
                    </p>
                    <div class="flex flex-wrap justify-center gap-2 mb-8">
                        <span class="px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm">
                            "Bolt Compatible"
                        </span>
                        <span class="px-3 py-1 bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200 rounded-full text-sm">
                            "Docker Compatible"
                        </span>
                        <span class="px-3 py-1 bg-purple-100 dark:bg-purple-900 text-purple-800 dark:text-purple-200 rounded-full text-sm">
                            "Podman Compatible"
                        </span>
                    </div>
                </div>

                <components::dashboard::Dashboard/>
            </main>
        </div>
    }
}

#[component]
fn RepositoriesPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <components::navbar::Navbar/>
            <main class="container mx-auto px-4 py-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-8">
                    "📦 Repositories"
                </h1>
                <pages::repositories::RepositoriesList/>
            </main>
        </div>
    }
}

#[component]
fn RepositoryDetailPage() -> impl IntoView {
    let params = use_params_map();
    let name = move || params.with(|params| params.get("name").cloned().unwrap_or_default());

    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <components::navbar::Navbar/>
            <main class="container mx-auto px-4 py-8">
                <pages::repositories::RepositoryDetail name=name()/>
            </main>
        </div>
    }
}

#[component]
fn BoltPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <components::navbar::Navbar/>
            <main class="container mx-auto px-4 py-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-8">
                    "⚡ Bolt Integration"
                </h1>
                <pages::bolt::BoltDashboard/>
            </main>
        </div>
    }
}

#[component]
fn BoltProfilesPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <components::navbar::Navbar/>
            <main class="container mx-auto px-4 py-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-8">
                    "🎮 Gaming Profiles"
                </h1>
                <pages::bolt::ProfilesList/>
            </main>
        </div>
    }
}

#[component]
fn BoltPluginsPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <components::navbar::Navbar/>
            <main class="container mx-auto px-4 py-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-8">
                    "🔧 Plugins"
                </h1>
                <pages::bolt::PluginsList/>
            </main>
        </div>
    }
}

#[component]
fn SettingsPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <components::navbar::Navbar/>
            <main class="container mx-auto px-4 py-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-8">
                    "⚙️ Settings"
                </h1>
                <pages::settings::SettingsPanel/>
            </main>
        </div>
    }
}

#[component]
fn LoginPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
            <pages::auth::LoginForm/>
        </div>
    }
}

#[component]
fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
            <div class="text-center">
                <h1 class="text-6xl font-bold text-gray-900 dark:text-white mb-4">"404"</h1>
                <p class="text-xl text-gray-600 dark:text-gray-300 mb-8">"Page not found"</p>
                <a href="/" class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
                    "Go Home"
                </a>
            </div>
        </div>
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoResponse {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>"Drift Registry"</title>
                <link rel="stylesheet" href="https://cdn.tailwindcss.com"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}