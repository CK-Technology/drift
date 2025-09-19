use leptos::*;
use crate::ui::components::{button::*, card::*};

#[component]
pub fn LoginForm() -> impl IntoView {
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (is_loading, set_is_loading) = create_signal(false);
    let (show_password, set_show_password) = create_signal(false);
    let (login_method, set_login_method) = create_signal("credentials".to_string());
    let (error_message, set_error_message) = create_signal(Option::<String>::None);

    let handle_credentials_login = move |_| {
        set_is_loading.set(true);
        set_error_message.set(None);
        // TODO: Implement credentials authentication
    };

    let handle_oauth_login = move |provider: &str| {
        set_is_loading.set(true);
        set_error_message.set(None);
        // TODO: Redirect to OAuth provider
        match provider {
            "azure" => web_sys::window().unwrap().location().set_href("/auth/azure").unwrap(),
            "github" => web_sys::window().unwrap().location().set_href("/auth/github").unwrap(),
            "google" => web_sys::window().unwrap().location().set_href("/auth/google").unwrap(),
            _ => {}
        }
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 via-white to-indigo-100 dark:from-gray-900 dark:via-gray-800 dark:to-blue-900 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-8">
                // Header section
                <div class="text-center">
                    <div class="relative mx-auto w-20 h-20 mb-6">
                        <img
                            src="/assets/icons/drift-64x64.png"
                            alt="Drift Logo"
                            class="w-full h-full rounded-2xl shadow-lg"
                        />
                        <div class="absolute inset-0 rounded-2xl bg-gradient-to-r from-blue-600/20 to-purple-600/20"></div>
                    </div>
                    <h2 class="text-3xl font-bold bg-gradient-to-r from-gray-900 to-gray-600 dark:from-white dark:to-gray-300 bg-clip-text text-transparent">
                        "Welcome to Drift"
                    </h2>
                    <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                        "Your container registry & gaming optimization platform"
                    </p>
                </div>

                // Main auth card
                <div class="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl shadow-xl border border-white/20 dark:border-gray-700/50 p-8">
                    // Method toggle
                    <div class="flex rounded-lg bg-gray-100 dark:bg-gray-700 p-1 mb-6">
                        <button
                            class=move || format!(
                                "flex-1 py-2 px-4 text-sm font-medium rounded-md transition-all duration-200 {}",
                                if login_method.get() == "credentials" {
                                    "bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm"
                                } else {
                                    "text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
                                }
                            )
                            on:click=move |_| set_login_method("credentials".to_string())
                        >
                            "Credentials"
                        </button>
                        <button
                            class=move || format!(
                                "flex-1 py-2 px-4 text-sm font-medium rounded-md transition-all duration-200 {}",
                                if login_method.get() == "sso" {
                                    "bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm"
                                } else {
                                    "text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
                                }
                            )
                            on:click=move |_| set_login_method("sso".to_string())
                        >
                            "SSO"
                        </button>
                    </div>

                    // Error message
                    {move || {
                        if let Some(error) = error_message.get() {
                            view! {
                                <div class="mb-4 p-3 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800">
                                    <p class="text-sm text-red-600 dark:text-red-400">{error}</p>
                                </div>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }
                    }}

                    // Login content
                    {move || {
                        if login_method.get() == "credentials" {
                            view! {
                                <form class="space-y-6" on:submit=handle_credentials_login>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                            "Username"
                                        </label>
                                        <input
                                            type="text"
                                            class="input-modern"
                                            placeholder="Enter your username"
                                            prop:disabled=is_loading
                                            on:input=move |ev| {
                                                let val = event_target_value(&ev);
                                                set_username.set(val);
                                            }
                                            prop:value=username
                                        />
                                    </div>

                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                            "Password"
                                        </label>
                                        <div class="relative">
                                            <input
                                                type=move || if show_password.get() { "text" } else { "password" }
                                                class="input-modern pr-10"
                                                placeholder="Enter your password"
                                                prop:disabled=is_loading
                                                on:input=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    set_password.set(val);
                                                }
                                                prop:value=password
                                            />
                                            <button
                                                type="button"
                                                class="absolute inset-y-0 right-0 pr-3 flex items-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                                                on:click=move |_| set_show_password.update(|s| *s = !*s)
                                            >
                                                {move || {
                                                    if show_password.get() {
                                                        view! {
                                                            <svg class="h-5 w-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L3 3m6.878 6.878L21 21" />
                                                            </svg>
                                                        }
                                                    } else {
                                                        view! {
                                                            <svg class="h-5 w-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                                                            </svg>
                                                        }
                                                    }
                                                }}
                                            </button>
                                        </div>
                                    </div>

                                    <div class="flex items-center justify-between">
                                        <label class="flex items-center">
                                            <input type="checkbox" class="rounded border-gray-300 text-blue-600 focus:ring-blue-500 focus:ring-offset-0"/>
                                            <span class="ml-2 text-sm text-gray-600 dark:text-gray-400">"Remember me"</span>
                                        </label>
                                        <a href="#" class="text-sm text-blue-600 dark:text-blue-400 hover:underline">
                                            "Forgot password?"
                                        </a>
                                    </div>

                                    <Button
                                        variant=ButtonVariant::Primary
                                        size=ButtonSize::Large
                                        full_width=true
                                        loading=is_loading.get()
                                        disabled={move || username.get().is_empty() || password.get().is_empty()}()
                                        on:click=handle_credentials_login
                                    >
                                        "Sign In"
                                    </Button>
                                </form>
                            }.into_view()
                        } else {
                            view! {
                                <div class="space-y-4">
                                    <p class="text-sm text-gray-600 dark:text-gray-400 text-center mb-6">
                                        "Choose your preferred sign-in method"
                                    </p>

                                    // Azure AD SSO
                                    <Button
                                        variant=ButtonVariant::Outline
                                        size=ButtonSize::Large
                                        full_width=true
                                        loading=is_loading.get()
                                        onclick=move |_| handle_oauth_login("azure")
                                    >
                                        <div class="flex items-center justify-center space-x-3">
                                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                                                <path d="M1.194 7.543v8.913l5.15 2.981V18.45l-3.245-1.838v-6.257L1.194 7.543zM22.806 7.543l-1.905 2.813v6.257L17.656 18.45v-1.987l5.15-2.981V7.543zM12 2.25L2.194 7.543 12 12.836l9.806-5.293L12 2.25z"/>
                                            </svg>
                                            <span>"Continue with Azure AD"</span>
                                        </div>
                                    </Button>

                                    // GitHub SSO
                                    <Button
                                        variant=ButtonVariant::Outline
                                        size=ButtonSize::Large
                                        full_width=true
                                        loading=is_loading.get()
                                        onclick=move |_| handle_oauth_login("github")
                                    >
                                        <div class="flex items-center justify-center space-x-3">
                                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                                                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                                            </svg>
                                            <span>"Continue with GitHub"</span>
                                        </div>
                                    </Button>

                                    // Google SSO
                                    <Button
                                        variant=ButtonVariant::Outline
                                        size=ButtonSize::Large
                                        full_width=true
                                        loading=is_loading.get()
                                        onclick=move |_| handle_oauth_login("google")
                                    >
                                        <div class="flex items-center justify-center space-x-3">
                                            <svg class="w-5 h-5" viewBox="0 0 24 24">
                                                <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                                                <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                                                <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                                                <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                                            </svg>
                                            <span>"Continue with Google"</span>
                                        </div>
                                    </Button>
                                </div>
                            }.into_view()
                        }
                    }}

                    // Footer
                    <div class="mt-8 pt-6 border-t border-gray-200 dark:border-gray-700">
                        <p class="text-center text-sm text-gray-600 dark:text-gray-400">
                            "Need access? "
                            <a href="mailto:admin@drift.local" class="text-blue-600 dark:text-blue-400 hover:underline font-medium">
                                "Contact your administrator"
                            </a>
                        </p>
                    </div>
                </div>

                // Additional info
                <div class="text-center">
                    <p class="text-xs text-gray-500 dark:text-gray-400">
                        "By signing in, you agree to our Terms of Service and Privacy Policy"
                    </p>
                </div>
            </div>
        </div>
    }
}