use leptos::*;

#[component]
pub fn BoltDashboard() -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
                    <span class="mr-2">"🎮"</span>
                    "Gaming Profiles"
                </h2>
                <p class="text-gray-600 dark:text-gray-300 mb-4">
                    "Optimize your gaming performance with curated profiles for popular games."
                </p>
                <a href="/bolt/profiles" class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
                    "Browse Profiles"
                    <span class="ml-2">"→"</span>
                </a>
            </div>

            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
                    <span class="mr-2">"🔧"</span>
                    "Plugins"
                </h2>
                <p class="text-gray-600 dark:text-gray-300 mb-4">
                    "Extend Bolt's capabilities with GPU optimization and enhancement plugins."
                </p>
                <a href="/bolt/plugins" class="inline-flex items-center px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors">
                    "Browse Plugins"
                    <span class="ml-2">"→"</span>
                </a>
            </div>
        </div>
    }
}

#[component]
pub fn ProfilesList() -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <ProfileCard
                    name="Steam Gaming"
                    description="Optimized profile for Steam gaming with NVIDIA GPUs"
                    downloads=15420
                    rating=4.8
                    tags=vec!["gaming".to_string(), "steam".to_string(), "nvidia".to_string()]
                />
                <ProfileCard
                    name="Competitive FPS"
                    description="High-performance profile for competitive FPS games"
                    downloads=8930
                    rating=4.9
                    tags=vec!["competitive".to_string(), "fps".to_string(), "low-latency".to_string()]
                />
                <ProfileCard
                    name="Streaming Setup"
                    description="Balanced profile for gaming while streaming"
                    downloads=5620
                    rating=4.6
                    tags=vec!["streaming".to_string(), "obs".to_string(), "balanced".to_string()]
                />
            </div>
        </div>
    }
}

#[component]
pub fn PluginsList() -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <PluginCard
                    name="NVIDIA DLSS Optimizer"
                    description="Advanced DLSS optimization plugin for RTX GPUs"
                    downloads=5420
                    rating=4.7
                    plugin_type="GPU Optimization"
                />
                <PluginCard
                    name="Audio Enhancer"
                    description="Real-time audio enhancement for gaming"
                    downloads=3210
                    rating=4.5
                    plugin_type="Audio Enhancement"
                />
                <PluginCard
                    name="Network Optimizer"
                    description="Reduce latency and optimize network performance"
                    downloads=2890
                    rating=4.8
                    plugin_type="Network Optimization"
                />
            </div>
        </div>
    }
}

#[component]
fn ProfileCard(
    name: &'static str,
    description: &'static str,
    downloads: u32,
    rating: f32,
    tags: Vec<String>,
) -> impl IntoView {
    view! {
        <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-6 hover:shadow-md transition-shadow">
            <div class="flex items-start justify-between mb-4">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{name}</h3>
                <div class="flex items-center space-x-1">
                    <span class="text-yellow-400">"⭐"</span>
                    <span class="text-sm text-gray-600 dark:text-gray-400">{format!("{:.1}", rating)}</span>
                </div>
            </div>

            <p class="text-gray-600 dark:text-gray-300 text-sm mb-4">{description}</p>

            <div class="flex flex-wrap gap-2 mb-4">
                {tags.into_iter().map(|tag| view! {
                    <span class="px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs rounded">
                        {tag}
                    </span>
                }).collect::<Vec<_>>()}
            </div>

            <div class="flex items-center justify-between">
                <span class="text-sm text-gray-500 dark:text-gray-400">
                    {format!("{} downloads", downloads)}
                </span>
                <button class="px-3 py-1 bg-green-600 text-white text-sm rounded hover:bg-green-700 transition-colors">
                    "Install"
                </button>
            </div>
        </div>
    }
}

#[component]
fn PluginCard(
    name: &'static str,
    description: &'static str,
    downloads: u32,
    rating: f32,
    plugin_type: &'static str,
) -> impl IntoView {
    view! {
        <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-6 hover:shadow-md transition-shadow">
            <div class="flex items-start justify-between mb-4">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{name}</h3>
                <div class="flex items-center space-x-1">
                    <span class="text-yellow-400">"⭐"</span>
                    <span class="text-sm text-gray-600 dark:text-gray-400">{format!("{:.1}", rating)}</span>
                </div>
            </div>

            <p class="text-gray-600 dark:text-gray-300 text-sm mb-4">{description}</p>

            <div class="mb-4">
                <span class="px-2 py-1 bg-purple-100 dark:bg-purple-900 text-purple-800 dark:text-purple-200 text-xs rounded">
                    {plugin_type}
                </span>
            </div>

            <div class="flex items-center justify-between">
                <span class="text-sm text-gray-500 dark:text-gray-400">
                    {format!("{} downloads", downloads)}
                </span>
                <button class="px-3 py-1 bg-purple-600 text-white text-sm rounded hover:bg-purple-700 transition-colors">
                    "Install"
                </button>
            </div>
        </div>
    }
}