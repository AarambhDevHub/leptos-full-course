//! Module 24: Islands Architecture.
//! Leptos 0.8 does not stabilize an `#[island]` macro, so this lesson models
//! island-like boundaries with mostly static HTML plus small hydrated widgets.
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos::server::ServerAction;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Title};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

const MODULE_TITLE: &str = "Module 24 - Islands Architecture";

#[derive(Clone, Copy)]
struct Article {
    title: &'static str,
    category: &'static str,
    excerpt: &'static str,
    read_time: &'static str,
}

const ARTICLES: &[Article] = &[
    Article {
        title: "Server HTML first",
        category: "SSR",
        excerpt: "Ship meaningful markup before the browser downloads the WASM bundle.",
        read_time: "5 min",
    },
    Article {
        title: "Hydrate only interactions",
        category: "Performance",
        excerpt: "Search, theme, and forms can become small interactive islands.",
        read_time: "7 min",
    },
    Article {
        title: "Resource waterfalls",
        category: "Data",
        excerpt: "Suspense boundaries keep independent widgets from blocking the whole page.",
        read_time: "4 min",
    },
    Article {
        title: "Progressive enhancement",
        category: "Forms",
        excerpt: "ActionForm submits as HTML first and upgrades after hydration.",
        read_time: "6 min",
    },
];

// ============================================================================
// LESSON 1: Full-Stack App Structure
// ============================================================================

/// Explains the `shell` step in this lesson.
///
/// What it does: builds the HTML document used by server rendering and hydration.
/// Why it matters: It keeps the SSR document, metadata, and hydration scripts in one predictable place.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body><App/></body>
        </html>
    }
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `app` step in this lesson.
///
/// What it does: renders the `App` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text=MODULE_TITLE/>
        <Meta name="description" content="Island-like Leptos architecture in the 0.8 full course"/>
        <Router>
            <main style="max-width: 1080px; margin: 0 auto; padding: 32px; font-family: system-ui, sans-serif;">
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=IslandsPage/>
                </Routes>
            </main>
        </Router>
    }
}

// ============================================================================
// LESSON 3: Server Functions And Trusted Work
// ============================================================================

/// Explains the `save newsletter` step in this lesson.
///
/// What it does: runs the `save newsletter` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn save_newsletter(email: String) -> Result<String, ServerFnError> {
    let email = email.trim().to_lowercase();
    if !email.contains('@') {
        return Err(ServerFnError::ServerError("enter a valid email".into()));
    }
    Ok(format!("{email} subscribed"))
}

/// Explains the `islands page` step in this lesson.
///
/// What it does: renders the `IslandsPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn IslandsPage() -> impl IntoView {
    view! {
        <section style="display: grid; gap: 20px;">
            <header>
                <h1>{MODULE_TITLE}</h1>
                <p>"Most of this page is static server HTML. The search, theme, and newsletter widgets hydrate as small interactive areas."</p>
            </header>
            <ThemeIsland/>
            <SearchIsland/>
            <BlogLayout/>
            <div style="display: grid; grid-template-columns: minmax(0, 1fr) minmax(260px, 360px); gap: 16px;">
                <NewsletterIsland/>
                <PerformanceCard/>
            </div>
        </section>
    }
}

/// Explains the `blog layout` step in this lesson.
///
/// What it does: renders the `BlogLayout` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn BlogLayout() -> impl IntoView {
    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Static article list"</h2>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 12px;">
                {ARTICLES.iter().map(|article| view! {
                    <article style="background: #f6f7fb; border-radius: 8px; padding: 16px;">
                        <span style="color: #164a8b; font-weight: 700;">{article.category}</span>
                        <h3>{article.title}</h3>
                        <p>{article.excerpt}</p>
                        <small>{article.read_time}</small>
                    </article>
                }).collect_view()}
            </div>
        </section>
    }
}

/// Explains the `search island` step in this lesson.
///
/// What it does: renders the `SearchIsland` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn SearchIsland() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let delayed_query = LocalResource::new(move || {
        let query = query.get();
        async move {
            #[cfg(feature = "hydrate")]
            gloo_timers::future::sleep(std::time::Duration::from_millis(150)).await;
            query
        }
    });

    view! {
        <Suspense fallback=|| view! { <p>"Preparing search island..."</p> }>
            <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
                <h2>"Search island"</h2>
                <input
                    placeholder="Search articles"
                    prop:value=query
                    on:input:target=move |event| query.set(event.target().value())
                />
                {move || delayed_query.get().map(|needle| {
                    let needle = needle.to_lowercase();
                    view! {
                        <ul>
                            {ARTICLES.iter()
                                .filter(|article| needle.is_empty()
                                    || article.title.to_lowercase().contains(&needle)
                                    || article.category.to_lowercase().contains(&needle))
                                .map(|article| view! { <li>{article.title} " - " {article.category}</li> })
                                .collect_view()}
                        </ul>
                    }
                })}
            </section>
        </Suspense>
    }
}

/// Explains the `theme island` step in this lesson.
///
/// What it does: renders the `ThemeIsland` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ThemeIsland() -> impl IntoView {
    let dark = RwSignal::new(false);

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(document) = web_sys::window().and_then(|window| window.document()) {
                if let Some(root) = document.document_element() {
                    let _ = root.set_attribute(
                        "style",
                        if dark.get() {
                            "background:#111827;color:#f9fafb;"
                        } else {
                            "background:#ffffff;color:#111827;"
                        },
                    );
                }
            }
            if let Some(storage) = web_sys::window()
                .and_then(|window| window.local_storage().ok())
                .flatten()
            {
                let _ =
                    storage.set_item("module24_theme", if dark.get() { "dark" } else { "light" });
            }
        }
    });

    view! {
        <section style="display: flex; justify-content: space-between; align-items: center; border: 1px solid #d9dee8; border-radius: 8px; padding: 14px 20px;">
            <strong>"Theme island"</strong>
            <button on:click=move |_| dark.update(|value| *value = !*value)>
                {move || if dark.get() { "Light" } else { "Dark" }}
            </button>
        </section>
    }
}

/// Explains the `newsletter island` step in this lesson.
///
/// What it does: renders the `NewsletterIsland` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn NewsletterIsland() -> impl IntoView {
    let action = ServerAction::<SaveNewsletter>::new();
    let pending = action.pending();
    let value = action.value();

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Newsletter island"</h2>
            <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                <ActionForm action=action>
                    <input name="email" type="email" placeholder="Email"/>
                    <button type="submit" disabled=move || pending.get()>
                        {move || if pending.get() { "Saving..." } else { "Subscribe" }}
                    </button>
                </ActionForm>
            </div>
            {move || value.get().map(|result| match result {
                Ok(message) => view! { <p style="color: #166534;">{message}</p> }.into_any(),
                Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
            })}
        </section>
    }
}

/// Explains the `performance card` step in this lesson.
///
/// What it does: renders the `PerformanceCard` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn PerformanceCard() -> impl IntoView {
    view! {
        <aside style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Hydration cost"</h2>
            <table>
                <tbody>
                    <tr><th>"Full page"</th><td>"92 KB WASM touched"</td></tr>
                    <tr><th>"Search island"</th><td>"18 KB interactive"</td></tr>
                    <tr><th>"Theme island"</th><td>"6 KB interactive"</td></tr>
                    <tr><th>"Newsletter"</th><td>"ActionForm enhancement"</td></tr>
                </tbody>
            </table>
        </aside>
    }
}

/// Explains the `not found` step in this lesson.
///
/// What it does: renders the `NotFound` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section>
            <h1>"404"</h1>
            <p>"The requested islands route does not exist."</p>
        </section>
    }
}
