use leptos::*;

#[component]
pub fn SearchBar(
    #[prop(optional)] placeholder: Option<&'static str>,
    #[prop(optional)] value: Option<ReadSignal<String>>,
    #[prop(optional)] on_input: Option<Box<dyn Fn(String) + 'static>>,
    #[prop(optional)] on_search: Option<Box<dyn Fn(String) + 'static>>,
    #[prop(optional)] suggestions: Option<ReadSignal<Vec<String>>>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let (search_value, set_search_value) = create_signal(String::new());
    let (is_focused, set_is_focused) = create_signal(false);
    let (show_suggestions, set_show_suggestions) = create_signal(false);

    let placeholder_text = placeholder.unwrap_or("Search repositories, profiles, plugins...");

    view! {
        <div class=format!("relative {}", class.unwrap_or(""))>
            <div class="relative">
                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                    <svg class="h-5 w-5 text-gray-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
                    </svg>
                </div>
                <input
                    type="text"
                    class=format!(
                        "input-modern pl-10 pr-12 {}",
                        if is_focused() { "ring-2 ring-blue-500 border-transparent" } else { "" }
                    )
                    placeholder=placeholder_text
                    prop:value=move || {
                        if let Some(val) = value {
                            val()
                        } else {
                            search_value()
                        }
                    }
                    on:input=move |ev| {
                        let val = event_target_value(&ev);
                        set_search_value(val.clone());
                        if let Some(handler) = &on_input {
                            handler(val);
                        }
                        set_show_suggestions(true);
                    }
                    on:focus=move |_| {
                        set_is_focused(true);
                        set_show_suggestions(true);
                    }
                    on:blur=move |_| {
                        set_is_focused(false);
                        // Delay hiding suggestions to allow clicking
                        set_timeout(|| set_show_suggestions(false), std::time::Duration::from_millis(150));
                    }
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            let val = if let Some(v) = value { v() } else { search_value() };
                            if let Some(handler) = &on_search {
                                handler(val);
                            }
                            set_show_suggestions(false);
                        }
                    }
                />
                <div class="absolute inset-y-0 right-0 pr-3 flex items-center">
                    <button
                        class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded transition-colors"
                        on:click=move |_| {
                            let val = if let Some(v) = value { v() } else { search_value() };
                            if let Some(handler) = &on_search {
                                handler(val);
                            }
                        }
                    >
                        <span class="sr-only">"Search"</span>
                        <svg class="h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5-5 5M6 12h12" />
                        </svg>
                    </button>
                </div>
            </div>

            // Suggestions dropdown
            {move || {
                if show_suggestions() && suggestions.is_some() {
                    let sug = suggestions.unwrap();
                    let suggestions_list = sug();
                    if !suggestions_list.is_empty() {
                        view! {
                            <div class="absolute z-50 w-full mt-1 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 max-h-60 overflow-y-auto">
                                {suggestions_list.into_iter().map(|suggestion| {
                                    view! {
                                        <button
                                            class="w-full px-4 py-2 text-left hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors first:rounded-t-lg last:rounded-b-lg"
                                            on:click=move |_| {
                                                set_search_value(suggestion.clone());
                                                if let Some(handler) = &on_search {
                                                    handler(suggestion.clone());
                                                }
                                                set_show_suggestions(false);
                                            }
                                        >
                                            <div class="flex items-center space-x-2">
                                                <svg class="h-4 w-4 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
                                                </svg>
                                                <span class="text-sm text-gray-700 dark:text-gray-300">{suggestion}</span>
                                            </div>
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }
                } else {
                    view! {}.into_view()
                }
            }}
        </div>
    }
}

#[component]
pub fn SearchFilters(
    #[prop(optional)] filters: Option<ReadSignal<Vec<FilterOption>>>,
    #[prop(optional)] on_filter_change: Option<Box<dyn Fn(String, String) + 'static>>,
) -> impl IntoView {
    let (expanded, set_expanded) = create_signal(false);

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4">
            <div class="flex items-center justify-between">
                <h3 class="text-sm font-medium text-gray-900 dark:text-white">
                    "Filters"
                </h3>
                <button
                    class="btn-ghost text-xs"
                    on:click=move |_| set_expanded(!expanded())
                >
                    {move || if expanded() { "Hide" } else { "Show" }}
                </button>
            </div>

            {move || {
                if expanded() {
                    view! {
                        <div class="mt-4 space-y-4">
                            // Type filter
                            <div>
                                <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    "Type"
                                </label>
                                <select class="w-full text-sm border border-gray-300 dark:border-gray-600 rounded-md px-3 py-1 bg-white dark:bg-gray-700 text-gray-900 dark:text-white">
                                    <option value="">"All"</option>
                                    <option value="repository">"Repositories"</option>
                                    <option value="profile">"Bolt Profiles"</option>
                                    <option value="plugin">"Bolt Plugins"</option>
                                </select>
                            </div>

                            // Tags filter
                            <div>
                                <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    "Tags"
                                </label>
                                <div class="flex flex-wrap gap-2">
                                    {["latest", "stable", "beta", "gaming", "competitive"].map(|tag| {
                                        view! {
                                            <label class="flex items-center">
                                                <input type="checkbox" class="rounded border-gray-300 text-blue-600 focus:ring-blue-500 mr-1" />
                                                <span class="text-xs text-gray-600 dark:text-gray-400">{tag}</span>
                                            </label>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>

                            // Sort filter
                            <div>
                                <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    "Sort by"
                                </label>
                                <select class="w-full text-sm border border-gray-300 dark:border-gray-600 rounded-md px-3 py-1 bg-white dark:bg-gray-700 text-gray-900 dark:text-white">
                                    <option value="updated">"Recently Updated"</option>
                                    <option value="name">"Name"</option>
                                    <option value="downloads">"Most Downloaded"</option>
                                    <option value="size">"Size"</option>
                                </select>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }
            }}
        </div>
    }
}

#[derive(Clone, Debug)]
pub struct FilterOption {
    pub key: String,
    pub label: String,
    pub value: String,
    pub selected: bool,
}

#[component]
pub fn QuickSearch(
    suggestions: ReadSignal<Vec<String>>,
    #[prop(optional)] on_select: Option<Box<dyn Fn(String) + 'static>>,
) -> impl IntoView {
    view! {
        <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <h4 class="text-sm font-medium text-gray-900 dark:text-white mb-3">
                "Quick Search"
            </h4>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-2">
                {move || {
                    suggestions().into_iter().take(8).map(|item| {
                        view! {
                            <button
                                class="text-left px-3 py-2 text-sm text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-white dark:hover:bg-gray-700 rounded-md transition-colors"
                                on:click=move |_| {
                                    if let Some(handler) = &on_select {
                                        handler(item.clone());
                                    }
                                }
                            >
                                {item}
                            </button>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}