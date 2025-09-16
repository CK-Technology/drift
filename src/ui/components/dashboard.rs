use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistryStats {
    total_repositories: u32,
    total_images: u32,
    total_downloads: u32,
    storage_used_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecentActivity {
    action: String,
    repository: String,
    user: String,
    timestamp: String,
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let (stats, set_stats) = create_signal(None::<RegistryStats>);
    let (recent_activity, set_recent_activity) = create_signal(Vec::<RecentActivity>::new());

    // Load dashboard data
    create_effect(move |_| {
        spawn_local(async move {
            // Mock data - would fetch from API
            set_stats(Some(RegistryStats {
                total_repositories: 42,
                total_images: 156,
                total_downloads: 12840,
                storage_used_gb: 15.7,
            }));

            set_recent_activity(vec![
                RecentActivity {
                    action: "Pushed".to_string(),
                    repository: "gaming/steam-optimized".to_string(),
                    user: "admin".to_string(),
                    timestamp: "2 minutes ago".to_string(),
                },
                RecentActivity {
                    action: "Downloaded".to_string(),
                    repository: "bolt/competitive-fps".to_string(),
                    user: "gamer123".to_string(),
                    timestamp: "5 minutes ago".to_string(),
                },
                RecentActivity {
                    action: "Created".to_string(),
                    repository: "dev/nodejs-app".to_string(),
                    user: "developer".to_string(),
                    timestamp: "10 minutes ago".to_string(),
                },
            ]);
        });
    });

    view! {
        <div class="space-y-8">
            // Stats cards
            {move || {
                if let Some(stats) = stats() {
                    view! {
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                            <StatsCard
                                title="Repositories"
                                value=stats.total_repositories.to_string()
                                icon="📦"
                                color="blue"
                            />
                            <StatsCard
                                title="Container Images"
                                value=stats.total_images.to_string()
                                icon="🐳"
                                color="green"
                            />
                            <StatsCard
                                title="Downloads"
                                value=format!("{:.1}K", stats.total_downloads as f64 / 1000.0)
                                icon="⬇️"
                                color="purple"
                            />
                            <StatsCard
                                title="Storage Used"
                                value=format!("{:.1} GB", stats.storage_used_gb)
                                icon="💾"
                                color="orange"
                            />
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                            {(0..4).map(|_| view! {
                                <div class="animate-pulse">
                                    <div class="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
                                        <div class="h-4 bg-gray-200 dark:bg-gray-600 rounded mb-4"></div>
                                        <div class="h-8 bg-gray-200 dark:bg-gray-600 rounded"></div>
                                    </div>
                                </div>
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_view()
                }
            }}

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                // Recent activity
                <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm">
                    <div class="p-6 border-b border-gray-200 dark:border-gray-700">
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white flex items-center">
                            <span class="mr-2">"📊"</span>
                            "Recent Activity"
                        </h2>
                    </div>
                    <div class="p-6">
                        <div class="space-y-4">
                            {move || {
                                recent_activity().into_iter().map(|activity| {
                                    view! {
                                        <div class="flex items-center space-x-4 p-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
                                            <div class="flex-shrink-0">
                                                <div class="w-8 h-8 bg-blue-100 dark:bg-blue-900 rounded-full flex items-center justify-center">
                                                    <span class="text-blue-600 dark:text-blue-400 text-sm">
                                                        {match activity.action.as_str() {
                                                            "Pushed" => "⬆️",
                                                            "Downloaded" => "⬇️",
                                                            "Created" => "✨",
                                                            _ => "📝"
                                                        }}
                                                    </span>
                                                </div>
                                            </div>
                                            <div class="flex-1 min-w-0">
                                                <p class="text-sm text-gray-900 dark:text-white">
                                                    <span class="font-medium">{activity.user}</span>
                                                    " "
                                                    <span class="text-gray-600 dark:text-gray-400">{activity.action.to_lowercase()}</span>
                                                    " "
                                                    <span class="font-medium text-blue-600 dark:text-blue-400">{activity.repository}</span>
                                                </p>
                                                <p class="text-xs text-gray-500 dark:text-gray-400">
                                                    {activity.timestamp}
                                                </p>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()
                            }}
                        </div>
                    </div>
                </div>

                // Quick actions
                <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm">
                    <div class="p-6 border-b border-gray-200 dark:border-gray-700">
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white flex items-center">
                            <span class="mr-2">"🚀"</span>
                            "Quick Actions"
                        </h2>
                    </div>
                    <div class="p-6">
                        <div class="grid grid-cols-1 gap-4">
                            <QuickActionCard
                                title="Push Image"
                                description="Push a new container image to the registry"
                                icon="⬆️"
                                action="docker push localhost:5000/myapp:latest"
                            />
                            <QuickActionCard
                                title="Pull Image"
                                description="Pull an existing image from the registry"
                                icon="⬇️"
                                action="docker pull localhost:5000/myapp:latest"
                            />
                            <QuickActionCard
                                title="Bolt Profile"
                                description="Install a gaming optimization profile"
                                icon="⚡"
                                action="bolt profiles install steam-gaming"
                            />
                            <QuickActionCard
                                title="Browse Registry"
                                description="Explore repositories and tags"
                                icon="🔍"
                                action=""
                            />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StatsCard(
    title: &'static str,
    value: String,
    icon: &'static str,
    color: &'static str,
) -> impl IntoView {
    let (bg_class, text_class, border_class) = match color {
        "blue" => ("bg-blue-50 dark:bg-blue-900/20", "text-blue-600 dark:text-blue-400", "border-blue-200 dark:border-blue-800"),
        "green" => ("bg-green-50 dark:bg-green-900/20", "text-green-600 dark:text-green-400", "border-green-200 dark:border-green-800"),
        "purple" => ("bg-purple-50 dark:bg-purple-900/20", "text-purple-600 dark:text-purple-400", "border-purple-200 dark:border-purple-800"),
        "orange" => ("bg-orange-50 dark:bg-orange-900/20", "text-orange-600 dark:text-orange-400", "border-orange-200 dark:border-orange-800"),
        _ => ("bg-gray-50 dark:bg-gray-900/20", "text-gray-600 dark:text-gray-400", "border-gray-200 dark:border-gray-800"),
    };

    view! {
        <div class=format!("bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm border {}", border_class)>
            <div class="flex items-center justify-between">
                <div>
                    <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
                        {title}
                    </p>
                    <p class="text-2xl font-bold text-gray-900 dark:text-white">
                        {value}
                    </p>
                </div>
                <div class=format!("w-12 h-12 rounded-lg flex items-center justify-center {}", bg_class)>
                    <span class=format!("text-xl {}", text_class)>
                        {icon}
                    </span>
                </div>
            </div>
        </div>
    }
}

#[component]
fn QuickActionCard(
    title: &'static str,
    description: &'static str,
    icon: &'static str,
    action: &'static str,
) -> impl IntoView {
    view! {
        <div class="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors cursor-pointer group">
            <div class="flex items-start space-x-3">
                <div class="flex-shrink-0">
                    <span class="text-xl">{icon}</span>
                </div>
                <div class="flex-1 min-w-0">
                    <h3 class="text-sm font-medium text-gray-900 dark:text-white group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors">
                        {title}
                    </h3>
                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        {description}
                    </p>
                    {if !action.is_empty() {
                        view! {
                            <code class="text-xs bg-gray-100 dark:bg-gray-600 text-gray-800 dark:text-gray-200 px-2 py-1 rounded mt-2 inline-block">
                                {action}
                            </code>
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }}
                </div>
            </div>
        </div>
    }
}