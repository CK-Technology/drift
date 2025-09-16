use leptos::*;

#[component]
pub fn RepositoriesList() -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm">
            <div class="p-6">
                <div class="flex items-center justify-between mb-6">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
                        "Container Repositories"
                    </h2>
                    <button class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
                        "Create Repository"
                    </button>
                </div>

                <div class="space-y-4">
                    // Mock repository list
                    <RepositoryCard name="gaming/steam-optimized" tags=vec!["latest".to_string(), "v1.2.0".to_string()] size="2.1 GB" pulls=1540/>
                    <RepositoryCard name="bolt/competitive-fps" tags=vec!["latest".to_string(), "stable".to_string()] size="850 MB" pulls=920/>
                    <RepositoryCard name="dev/nodejs-app" tags=vec!["latest".to_string(), "v2.0.0".to_string(), "dev".to_string()] size="1.3 GB" pulls=150/>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn RepositoryDetail(name: String) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    {name.clone()}
                </h1>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                        <h3 class="text-sm font-medium text-gray-500 dark:text-gray-400">"Total Downloads"</h3>
                        <p class="text-2xl font-bold text-gray-900 dark:text-white">"1,540"</p>
                    </div>
                    <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                        <h3 class="text-sm font-medium text-gray-500 dark:text-gray-400">"Repository Size"</h3>
                        <p class="text-2xl font-bold text-gray-900 dark:text-white">"2.1 GB"</p>
                    </div>
                    <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                        <h3 class="text-sm font-medium text-gray-500 dark:text-gray-400">"Last Updated"</h3>
                        <p class="text-2xl font-bold text-gray-900 dark:text-white">"2h ago"</p>
                    </div>
                </div>
            </div>

            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Tags"</h2>
                <div class="space-y-3">
                    <TagCard tag="latest" digest="sha256:abc123..." size="2.1 GB" created="2 hours ago"/>
                    <TagCard tag="v1.2.0" digest="sha256:def456..." size="2.0 GB" created="1 day ago"/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn RepositoryCard(name: &'static str, tags: Vec<String>, size: &'static str, pulls: u32) -> impl IntoView {
    view! {
        <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
            <div class="flex items-center justify-between">
                <div class="flex-1">
                    <h3 class="text-lg font-medium text-gray-900 dark:text-white">
                        <a href=format!("/repositories/{}", name) class="hover:text-blue-600 dark:hover:text-blue-400">
                            {name}
                        </a>
                    </h3>
                    <div class="flex items-center space-x-4 mt-2">
                        <span class="text-sm text-gray-500 dark:text-gray-400">
                            {format!("{} tags", tags.len())}
                        </span>
                        <span class="text-sm text-gray-500 dark:text-gray-400">
                            {size}
                        </span>
                        <span class="text-sm text-gray-500 dark:text-gray-400">
                            {format!("{} pulls", pulls)}
                        </span>
                    </div>
                    <div class="flex flex-wrap gap-2 mt-3">
                        {tags.into_iter().take(3).map(|tag| view! {
                            <span class="px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs rounded">
                                {tag}
                            </span>
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
                <div class="flex items-center space-x-2">
                    <button class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200">
                        "📋"
                    </button>
                    <button class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200">
                        "🗑️"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TagCard(tag: &'static str, digest: &'static str, size: &'static str, created: &'static str) -> impl IntoView {
    view! {
        <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
            <div class="flex items-center justify-between">
                <div>
                    <h4 class="font-medium text-gray-900 dark:text-white">{tag}</h4>
                    <p class="text-sm text-gray-500 dark:text-gray-400 font-mono">{digest}</p>
                    <div class="flex items-center space-x-4 mt-2">
                        <span class="text-sm text-gray-500 dark:text-gray-400">{size}</span>
                        <span class="text-sm text-gray-500 dark:text-gray-400">{created}</span>
                    </div>
                </div>
                <div class="flex items-center space-x-2">
                    <button class="px-3 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-700">
                        "Copy pull command"
                    </button>
                </div>
            </div>
        </div>
    }
}