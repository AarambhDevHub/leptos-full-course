//! Module 20: Caching and Optimization.
//! Manual stale-while-revalidate, paginated resources, accumulated pages,
//! server cache headers, and polling dashboards.
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Title};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use serde::{Deserialize, Serialize};

const MODULE_TITLE: &str = "Module 20 - Caching and Optimization";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Item {
    id: u32,
    title: String,
    summary: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StatsSnapshot {
    requests: u64,
    errors: u64,
    latency_ms: u64,
    uptime: String,
}

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
        <Meta name="description" content="Caching patterns in the Leptos 0.8 full course"/>
        <Router>
            <main style="max-width: 1080px; margin: 0 auto; padding: 32px; font-family: system-ui, sans-serif;">
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=CachingPage/>
                </Routes>
            </main>
        </Router>
    }
}

// ============================================================================
// LESSON 3: Server Functions And Trusted Work
// ============================================================================

/// Explains the `list items` step in this lesson.
///
/// What it does: runs the `list items` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn list_items(page: u32, limit: u32) -> Result<Vec<Item>, ServerFnError> {
    let start = page.saturating_sub(1) * limit;
    Ok((0..limit)
        .map(|offset| {
            let id = start + offset + 1;
            Item {
                id,
                title: format!("Cached item {id}"),
                summary: format!(
                    "Page {page} row {} fetched from the server function.",
                    offset + 1
                ),
            }
        })
        .collect())
}

/// Explains the `cached headline` step in this lesson.
///
/// What it does: runs the `cached headline` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn cached_headline() -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
        response.insert_header(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("public, max-age=60, stale-while-revalidate=30"),
        );
    }

    Ok(format!(
        "fresh at {}",
        chrono::Utc::now().format("%H:%M:%S UTC")
    ))
}

/// Explains the `dashboard stats` step in this lesson.
///
/// What it does: runs the `dashboard stats` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn dashboard_stats() -> Result<StatsSnapshot, ServerFnError> {
    let seconds = chrono::Utc::now().timestamp().max(0) as u64;
    Ok(StatsSnapshot {
        requests: 12_000 + seconds % 900,
        errors: seconds % 7,
        latency_ms: 80 + seconds % 40,
        uptime: format!("{}h {}m", seconds / 3600 % 24, seconds / 60 % 60),
    })
}

/// Explains the `caching page` step in this lesson.
///
/// What it does: renders the `CachingPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn CachingPage() -> impl IntoView {
    view! {
        <section style="display: grid; gap: 20px;">
            <header>
                <h1>{MODULE_TITLE}</h1>
                <p>"Resources are cacheable by shape: source signals, local stale state, explicit refetch, and HTTP headers all matter."</p>
            </header>
            <SwrPanel/>
            <PaginatedList/>
            <InfiniteList/>
            <PollingDashboard/>
        </section>
    }
}

/// Explains the `swr panel` step in this lesson.
///
/// What it does: renders the `SwrPanel` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn SwrPanel() -> impl IntoView {
    let refresh = RwSignal::new(0_u32);
    let cached = RwSignal::new(None::<String>);
    let headline = Resource::new(move || refresh.get(), |_| cached_headline());

    Effect::new(move |_| {
        if let Some(Ok(value)) = headline.get() {
            cached.set(Some(value));
        }
    });

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Manual stale-while-revalidate"</h2>
            <p>"Cached value shown immediately: " <strong>{move || cached.get().unwrap_or_else(|| "none".into())}</strong></p>
            <Suspense fallback=|| view! { <p>"Revalidating..."</p> }>
                {move || headline.get().map(|result| match result {
                    Ok(value) => view! { <p>"Fresh value: " {value}</p> }.into_any(),
                    Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
                })}
            </Suspense>
            <button on:click=move |_| refresh.update(|value| *value += 1)>"Revalidate"</button>
        </section>
    }
}

/// Explains the `paginated list` step in this lesson.
///
/// What it does: renders the `PaginatedList` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn PaginatedList() -> impl IntoView {
    let page = RwSignal::new(1_u32);
    let items = Resource::new(move || page.get(), |page| list_items(page, 5));

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Paginated resource"</h2>
            <div style="display: flex; gap: 8px; align-items: center;">
                <button disabled=move || page.get() == 1 on:click=move |_| page.update(|value| *value = value.saturating_sub(1).max(1))>"Prev"</button>
                <strong>"Page " {page}</strong>
                <button on:click=move |_| page.update(|value| *value += 1)>"Next"</button>
            </div>
            <Suspense fallback=|| view! { <p>"Loading page..."</p> }>
                {move || items.get().map(render_items)}
            </Suspense>
        </section>
    }
}

/// Explains the `infinite list` step in this lesson.
///
/// What it does: renders the `InfiniteList` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn InfiniteList() -> impl IntoView {
    let next_page = RwSignal::new(1_u32);
    let accumulated = RwSignal::new(Vec::<Item>::new());
    let page_items = Resource::new(move || next_page.get(), |page| list_items(page, 4));

    Effect::new(move |_| {
        if let Some(Ok(items)) = page_items.get() {
            accumulated.update(|current| {
                for item in items {
                    if !current.iter().any(|existing| existing.id == item.id) {
                        current.push(item);
                    }
                }
            });
        }
    });

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Accumulated infinite list"</h2>
            <p>"A real infinite scroller would increment this page signal from an IntersectionObserver sentinel."</p>
            {move || render_items(Ok(accumulated.get()))}
            <button on:click=move |_| next_page.update(|value| *value += 1)>"Load more"</button>
        </section>
    }
}

/// Explains the `render items` step in this lesson.
///
/// What it does: converts `render items` data into a reusable view fragment.
/// Why it matters: Rendering helpers keep repeated UI branches compact and easier to compare.
fn render_items(result: Result<Vec<Item>, ServerFnError>) -> AnyView {
    match result {
        Ok(items) => view! {
            <ul style="display: grid; gap: 8px; padding-left: 0; list-style: none;">
                <For
                    each=move || items.clone()
                    key=|item| item.id
                    children=|item| view! {
                        <li style="padding: 10px; background: #f6f7fb; border-radius: 8px;">
                            <strong>{item.title}</strong>
                            <p>{item.summary}</p>
                        </li>
                    }
                />
            </ul>
        }
        .into_any(),
        Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
    }
}

/// Explains the `polling dashboard` step in this lesson.
///
/// What it does: renders the `PollingDashboard` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn PollingDashboard() -> impl IntoView {
    let stats = Resource::new(|| (), |_| dashboard_stats());

    #[cfg(feature = "hydrate")]
    leptos::task::spawn_local(async move {
        loop {
            gloo_timers::future::sleep(std::time::Duration::from_secs(5)).await;
            stats.refetch();
        }
    });

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Polling dashboard"</h2>
            <button on:click=move |_| stats.refetch()>"Refresh now"</button>
            <Suspense fallback=|| view! { <p>"Loading stats..."</p> }>
                {move || stats.get().map(|result| match result {
                    Ok(stats) => view! {
                        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 12px;">
                            <StatCard label="Requests" value=stats.requests.to_string()/>
                            <StatCard label="Errors" value=stats.errors.to_string()/>
                            <StatCard label="Latency" value=format!("{} ms", stats.latency_ms)/>
                            <StatCard label="Uptime" value=stats.uptime/>
                        </div>
                    }.into_any(),
                    Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
                })}
            </Suspense>
        </section>
    }
}

/// Explains the `stat card` step in this lesson.
///
/// What it does: renders the `StatCard` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn StatCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <article style="background: #f6f7fb; border-radius: 8px; padding: 16px;">
            <span style="color: #56616f;">{label}</span>
            <strong style="display: block; font-size: 1.4rem;">{value}</strong>
        </article>
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
            <p>"The requested caching route does not exist."</p>
        </section>
    }
}
