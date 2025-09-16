use leptos::*;

#[component]
pub fn Card(
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] hover: Option<bool>,
    #[prop(optional)] glass: Option<bool>,
    children: Children,
) -> impl IntoView {
    let base_class = if glass.unwrap_or(false) {
        "card-glass"
    } else {
        "card"
    };

    let hover_class = if hover.unwrap_or(true) {
        " hover:shadow-xl hover:-translate-y-1"
    } else {
        ""
    };

    let final_class = format!("{}{} {}", base_class, hover_class, class.unwrap_or(""));

    view! {
        <div class=final_class>
            {children()}
        </div>
    }
}

#[component]
pub fn StatsCard(
    title: &'static str,
    value: String,
    icon: &'static str,
    color: &'static str,
    #[prop(optional)] trend: Option<f32>,
    #[prop(optional)] subtitle: Option<String>,
) -> impl IntoView {
    let (bg_class, icon_class, trend_color) = match color {
        "blue" => ("from-blue-500 to-blue-600", "text-blue-100", "text-blue-600"),
        "green" => ("from-green-500 to-green-600", "text-green-100", "text-green-600"),
        "purple" => ("from-purple-500 to-purple-600", "text-purple-100", "text-purple-600"),
        "orange" => ("from-orange-500 to-orange-600", "text-orange-100", "text-orange-600"),
        "red" => ("from-red-500 to-red-600", "text-red-100", "text-red-600"),
        _ => ("from-gray-500 to-gray-600", "text-gray-100", "text-gray-600"),
    };

    view! {
        <div class="stats-card group">
            <div class="flex items-center justify-between">
                <div class="flex-1">
                    <p class="text-sm font-medium text-gray-600 dark:text-gray-400 mb-1">
                        {title}
                    </p>
                    <p class="text-3xl font-bold text-gray-900 dark:text-white mb-1">
                        {value}
                    </p>
                    {if let Some(subtitle) = subtitle {
                        view! {
                            <p class="text-xs text-gray-500 dark:text-gray-400">
                                {subtitle}
                            </p>
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }}
                </div>
                <div class=format!("w-16 h-16 rounded-2xl bg-gradient-to-br {} flex items-center justify-center transform group-hover:scale-110 transition-transform duration-200", bg_class)>
                    <span class=format!("text-2xl {}", icon_class)>
                        {icon}
                    </span>
                </div>
            </div>

            {if let Some(trend_value) = trend {
                let (trend_icon, trend_text) = if trend_value > 0.0 {
                    ("↗", format!("+{:.1}%", trend_value))
                } else if trend_value < 0.0 {
                    ("↘", format!("{:.1}%", trend_value))
                } else {
                    ("→", "0%".to_string())
                };

                view! {
                    <div class="mt-4 flex items-center">
                        <span class=format!("text-sm font-medium {}", trend_color)>
                            {trend_icon} " " {trend_text}
                        </span>
                        <span class="text-xs text-gray-500 dark:text-gray-400 ml-2">
                            "vs last period"
                        </span>
                    </div>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}

#[component]
pub fn FeatureCard(
    title: &'static str,
    description: &'static str,
    icon: &'static str,
    #[prop(optional)] href: Option<&'static str>,
    #[prop(optional)] badge: Option<&'static str>,
) -> impl IntoView {
    let card_content = view! {
        <div class="p-6">
            <div class="flex items-start space-x-4">
                <div class="flex-shrink-0">
                    <div class="w-12 h-12 bg-gradient-to-br from-blue-500 to-cyan-500 rounded-xl flex items-center justify-center">
                        <span class="text-white text-xl">{icon}</span>
                    </div>
                </div>
                <div class="flex-1 min-w-0">
                    <div class="flex items-start justify-between">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                            {title}
                        </h3>
                        {if let Some(badge_text) = badge {
                            view! {
                                <span class="px-2 py-1 text-xs font-medium bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full">
                                    {badge_text}
                                </span>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }}
                    </div>
                    <p class="text-gray-600 dark:text-gray-300 mt-2">
                        {description}
                    </p>
                </div>
            </div>
        </div>
    };

    if let Some(link) = href {
        view! {
            <a href=link class="card block group hover:scale-105 transition-transform duration-200">
                {card_content}
                <div class="px-6 pb-6">
                    <span class="text-blue-600 dark:text-blue-400 text-sm font-medium group-hover:text-blue-700 dark:group-hover:text-blue-300">
                        "Learn more →"
                    </span>
                </div>
            </a>
        }.into_view()
    } else {
        view! {
            <div class="card">
                {card_content}
            </div>
        }.into_view()
    }
}

#[component]
pub fn ImageCard(
    title: &'static str,
    tags: Vec<String>,
    size: &'static str,
    pulls: u32,
    updated: &'static str,
) -> impl IntoView {
    view! {
        <Card class="hover:shadow-2xl transition-all duration-300">
            <div class="p-6">
                <div class="flex items-start justify-between mb-4">
                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white truncate">
                        {title}
                    </h3>
                    <div class="flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400">
                        <span class="w-2 h-2 bg-green-500 rounded-full"></span>
                        <span>"Active"</span>
                    </div>
                </div>

                <div class="flex flex-wrap gap-2 mb-4">
                    {tags.into_iter().take(3).map(|tag| view! {
                        <span class="px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs rounded-full">
                            {tag}
                        </span>
                    }).collect::<Vec<_>>()}
                </div>

                <div class="grid grid-cols-2 gap-4 text-sm">
                    <div>
                        <span class="text-gray-500 dark:text-gray-400">"Size: "</span>
                        <span class="font-medium text-gray-900 dark:text-white">{size}</span>
                    </div>
                    <div>
                        <span class="text-gray-500 dark:text-gray-400">"Pulls: "</span>
                        <span class="font-medium text-gray-900 dark:text-white">{pulls.to_string()}</span>
                    </div>
                    <div class="col-span-2">
                        <span class="text-gray-500 dark:text-gray-400">"Updated: "</span>
                        <span class="font-medium text-gray-900 dark:text-white">{updated}</span>
                    </div>
                </div>

                <div class="mt-4 flex items-center space-x-2">
                    <button class="flex-1 btn-primary text-sm py-2">
                        "Pull Image"
                    </button>
                    <button class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
                        "📋"
                    </button>
                    <button class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
                        "⚙️"
                    </button>
                </div>
            </div>
        </Card>
    }
}