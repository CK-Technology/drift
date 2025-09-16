use leptos::*;

#[derive(Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
    Success,
}

#[derive(Clone, PartialEq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

#[component]
pub fn Button(
    #[prop(optional)] variant: Option<ButtonVariant>,
    #[prop(optional)] size: Option<ButtonSize>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional)] loading: Option<bool>,
    #[prop(optional)] icon: Option<&'static str>,
    #[prop(optional)] icon_position: Option<&'static str>, // "left" | "right"
    #[prop(optional)] onclick: Option<Box<dyn Fn() + 'static>>,
    #[prop(optional)] href: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(ButtonVariant::Primary);
    let size = size.unwrap_or(ButtonSize::Medium);
    let disabled = disabled.unwrap_or(false);
    let loading = loading.unwrap_or(false);
    let icon_pos = icon_position.unwrap_or("left");

    let variant_class = match variant {
        ButtonVariant::Primary => "btn-primary",
        ButtonVariant::Secondary => "btn-secondary",
        ButtonVariant::Ghost => "btn-ghost",
        ButtonVariant::Danger => "px-6 py-3 bg-gradient-to-r from-red-600 to-red-700 text-white font-medium rounded-lg hover:from-red-700 hover:to-red-800 focus:ring-2 focus:ring-red-500 focus:ring-offset-2 transition-all duration-200 transform hover:scale-105 active:scale-95",
        ButtonVariant::Success => "px-6 py-3 bg-gradient-to-r from-green-600 to-green-700 text-white font-medium rounded-lg hover:from-green-700 hover:to-green-800 focus:ring-2 focus:ring-green-500 focus:ring-offset-2 transition-all duration-200 transform hover:scale-105 active:scale-95",
    };

    let size_class = match size {
        ButtonSize::Small => "px-3 py-1.5 text-sm",
        ButtonSize::Medium => "px-6 py-3 text-base",
        ButtonSize::Large => "px-8 py-4 text-lg",
    };

    let disabled_class = if disabled || loading {
        "opacity-50 cursor-not-allowed transform-none hover:transform-none"
    } else {
        ""
    };

    let final_class = format!("{} {} {} {}", variant_class, size_class, disabled_class, class.unwrap_or(""));

    let button_content = view! {
        <div class="flex items-center justify-center space-x-2">
            {if loading {
                view! {
                    <svg class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                }.into_view()
            } else if let Some(icon_text) = icon {
                if icon_pos == "left" {
                    view! {
                        <span class="icon-animated">{icon_text}</span>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }
            } else {
                view! {}.into_view()
            }}

            <span>{children()}</span>

            {if let Some(icon_text) = icon {
                if icon_pos == "right" && !loading {
                    view! {
                        <span class="icon-animated">{icon_text}</span>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }
            } else {
                view! {}.into_view()
            }}
        </div>
    };

    if let Some(link) = href {
        view! {
            <a href=link class=final_class>
                {button_content}
            </a>
        }.into_view()
    } else {
        view! {
            <button
                class=final_class
                disabled=disabled || loading
                on:click=move |_| {
                    if let Some(handler) = &onclick {
                        if !disabled && !loading {
                            handler();
                        }
                    }
                }
            >
                {button_content}
            </button>
        }.into_view()
    }
}

#[component]
pub fn IconButton(
    icon: &'static str,
    #[prop(optional)] variant: Option<ButtonVariant>,
    #[prop(optional)] size: Option<ButtonSize>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional)] tooltip: Option<&'static str>,
    #[prop(optional)] onclick: Option<Box<dyn Fn() + 'static>>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let variant = variant.unwrap_or(ButtonVariant::Ghost);
    let size = size.unwrap_or(ButtonSize::Medium);
    let disabled = disabled.unwrap_or(false);

    let size_class = match size {
        ButtonSize::Small => "w-8 h-8 text-sm",
        ButtonSize::Medium => "w-10 h-10 text-base",
        ButtonSize::Large => "w-12 h-12 text-lg",
    };

    let variant_class = match variant {
        ButtonVariant::Primary => "bg-blue-600 hover:bg-blue-700 text-white",
        ButtonVariant::Secondary => "bg-gray-200 hover:bg-gray-300 text-gray-700 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-300",
        ButtonVariant::Ghost => "hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400",
        ButtonVariant::Danger => "bg-red-600 hover:bg-red-700 text-white",
        ButtonVariant::Success => "bg-green-600 hover:bg-green-700 text-white",
    };

    let final_class = format!("inline-flex items-center justify-center rounded-lg font-medium transition-all duration-200 transform hover:scale-105 active:scale-95 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 {} {} {}", size_class, variant_class, class.unwrap_or(""));

    view! {
        <button
            class=final_class
            disabled=disabled
            title=tooltip.unwrap_or("")
            on:click=move |_| {
                if let Some(handler) = &onclick {
                    if !disabled {
                        handler();
                    }
                }
            }
        >
            <span class="icon-animated">{icon}</span>
        </button>
    }
}

#[component]
pub fn ButtonGroup(
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("inline-flex rounded-lg shadow-sm {}", class.unwrap_or(""))>
            {children()}
        </div>
    }
}

#[component]
pub fn FloatingActionButton(
    icon: &'static str,
    #[prop(optional)] onclick: Option<Box<dyn Fn() + 'static>>,
    #[prop(optional)] tooltip: Option<&'static str>,
) -> impl IntoView {
    view! {
        <button
            class="fab"
            title=tooltip.unwrap_or("")
            on:click=move |_| {
                if let Some(handler) = &onclick {
                    handler();
                }
            }
        >
            <span class="text-xl">{icon}</span>
        </button>
    }
}