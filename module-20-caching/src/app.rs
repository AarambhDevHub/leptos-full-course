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

/// The most rows one page request may ask for.
pub const MAX_PAGE_SIZE: u32 = 50;
/// The highest page number the demo data supports.
pub const MAX_PAGE: u32 = 1_000;

/// Explains the `page window` step in this lesson.
///
/// What it does: validates a page number and size from the client, and returns the id of the first row and the number of rows.
/// Why it matters: A server function is a public door. Without limits, one request with a huge size built a 100 MB answer, and a huge page number overflowed and crashed the handler.
pub fn page_window(page: u32, limit: u32) -> Result<(u32, u32), &'static str> {
    if limit == 0 || limit > MAX_PAGE_SIZE {
        return Err("page size must be between 1 and 50");
    }
    if page == 0 || page > MAX_PAGE {
        return Err("page number out of range");
    }
    // At most 999 * 50 = 49,950, which always fits.
    Ok(((page - 1) * limit, limit))
}

/// Explains the `merge unique` step in this lesson.
///
/// What it does: appends the items whose id is not in the list yet, keeping the order.
/// Why it matters: A page that arrives twice (a refetch, a retry) must never create duplicate rows.
fn merge_unique(current: &mut Vec<Item>, incoming: Vec<Item>) {
    for item in incoming {
        if !current.iter().any(|existing| existing.id == item.id) {
            current.push(item);
        }
    }
}

/// Explains the `can load more` step in this lesson.
///
/// What it does: decides whether the Load more button may ask for another page.
/// Why it matters: Asking for page 3 while page 2 is still in flight can make the answer for page 2 get discarded and leave a hole in the list, so the next page is requested only after the last one was added.
fn can_load_more(loaded_page: u32, requested_page: u32, failed: bool) -> bool {
    failed || (loaded_page >= requested_page && requested_page < MAX_PAGE)
}

/// Explains the `poll delay secs` step in this lesson.
///
/// What it does: returns how long to wait before the next poll: 5 seconds normally, doubling after each failure up to 60 seconds.
/// Why it matters: A dashboard that keeps asking every 5 seconds while the server is struggling makes the problem worse.
pub fn poll_delay_secs(consecutive_failures: u32) -> u64 {
    (5_u64 << consecutive_failures.min(4)).min(60)
}
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
    let (start, count) = page_window(page, limit).map_err(ServerFnError::new)?;
    Ok((0..count)
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

/// Explains the `fresh headline` step in this lesson.
///
/// What it does: returns the current server time with no caching headers at all.
/// Why it matters: The manual stale-while-revalidate panel does its own caching in a signal, so the network must always give it a fresh answer.
#[server]
pub async fn fresh_headline() -> Result<String, ServerFnError> {
    Ok(format!(
        "fresh at {}",
        chrono::Utc::now().format("%H:%M:%S UTC")
    ))
}

/// Explains the `cached headline` step in this lesson.
///
/// What it does: returns the server time as a GET request, with a `Cache-Control` header on the API response only.
/// Why it matters: Browsers and CDNs only reuse GET responses, never POST. The header is also skipped when the same function runs during server rendering, because there the response options belong to the whole HTML page and a shared cache would keep the page.
#[server(endpoint = "cached_headline", input = server_fn::codec::GetUrl)]
pub async fn cached_headline() -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let is_api_call = use_context::<axum::http::request::Parts>()
            .map(|parts| parts.uri.path().starts_with("/api/"))
            .unwrap_or(false);
        if is_api_call {
            if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
                response.insert_header(
                    axum::http::header::CACHE_CONTROL,
                    axum::http::HeaderValue::from_static(
                        "public, max-age=60, stale-while-revalidate=30",
                    ),
                );
            }
        }
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
            <HttpCachePanel/>
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
    let headline = Resource::new(move || refresh.get(), |_| fresh_headline());

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

/// Explains the `http cache panel` step in this lesson.
///
/// What it does: fetches a GET server function that carries `Cache-Control`, so the browser's own HTTP cache can answer repeated requests.
/// Why it matters: Pressing the button again within 60 seconds shows the same server time, because the browser never asked the server. After that, the old answer is shown while a fresh one is fetched in the background.
#[component]
fn HttpCachePanel() -> impl IntoView {
    let fetches = RwSignal::new(0_u32);
    let headline = Resource::new(move || fetches.get(), |_| cached_headline());

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"HTTP cache (Cache-Control on a GET)"</h2>
            <p>"Press the button twice within 60 seconds: the time stays the same, because the browser answers from its own cache."</p>
            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || headline.get().map(|result| match result {
                    Ok(value) => view! { <p>"Server said: " <strong>{value}</strong></p> }.into_any(),
                    Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
                })}
            </Suspense>
            <button on:click=move |_| fetches.update(|value| *value += 1)>"Fetch again"</button>
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
                <button disabled=move || page.get() >= MAX_PAGE on:click=move |_| page.update(|value| *value = (*value + 1).min(MAX_PAGE))>"Next"</button>
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
    let loaded_page = RwSignal::new(0_u32);
    let failed = RwSignal::new(false);
    let accumulated = RwSignal::new(Vec::<Item>::new());
    let page_items = Resource::new(move || next_page.get(), |page| list_items(page, 4));

    Effect::new(move |_| match page_items.get() {
        Some(Ok(items)) => {
            accumulated.update(|current| merge_unique(current, items));
            loaded_page.set(next_page.get_untracked());
            failed.set(false);
        }
        Some(Err(_)) => failed.set(true),
        None => {}
    });

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Accumulated infinite list"</h2>
            <p>"A real infinite scroller would increment this page signal from an IntersectionObserver sentinel."</p>
            {move || render_items(Ok(accumulated.get()))}
            {move || failed.get().then(|| view! { <p role="alert" style="color: #b91c1c;">"Could not load that page. Press the button to try again."</p> })}
            <button
                disabled=move || !can_load_more(loaded_page.get(), next_page.get(), failed.get())
                on:click=move |_| {
                    if failed.get_untracked() {
                        failed.set(false);
                        page_items.refetch();
                    } else {
                        next_page.update(|value| *value += 1);
                    }
                }
            >"Load more"</button>
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
    let failures = RwSignal::new(0_u32);

    Effect::new(move |_| match stats.get() {
        Some(Ok(_)) => failures.set(0),
        Some(Err(_)) => failures.update(|count| *count += 1),
        None => {}
    });

    // The loop stops when the component is removed, skips hidden tabs, and backs off after failures.
    #[cfg(feature = "hydrate")]
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let alive = Arc::new(AtomicBool::new(true));
        on_cleanup({
            let alive = alive.clone();
            move || alive.store(false, Ordering::Relaxed)
        });
        leptos::task::spawn_local(async move {
            loop {
                let delay = poll_delay_secs(failures.get_untracked());
                gloo_timers::future::sleep(std::time::Duration::from_secs(delay)).await;
                if !alive.load(Ordering::Relaxed) {
                    break;
                }
                if !document().hidden() {
                    stats.refetch();
                }
            }
        });
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u32) -> Item {
        Item {
            id,
            title: format!("t{id}"),
            summary: String::new(),
        }
    }

    #[test]
    fn page_window_maps_pages_to_rows() {
        assert_eq!(page_window(1, 5), Ok((0, 5)));
        assert_eq!(page_window(2, 5), Ok((5, 5)));
        assert_eq!(
            page_window(MAX_PAGE, MAX_PAGE_SIZE),
            Ok(((MAX_PAGE - 1) * MAX_PAGE_SIZE, MAX_PAGE_SIZE))
        );
    }

    #[test]
    fn page_window_rejects_hostile_input() {
        assert!(page_window(0, 5).is_err());
        assert!(page_window(1, 0).is_err());
        assert!(page_window(1, MAX_PAGE_SIZE + 1).is_err());
        assert!(page_window(1, 1_000_000).is_err());
        assert!(page_window(u32::MAX, 10).is_err()); // used to overflow and panic
        assert!(page_window(MAX_PAGE + 1, 10).is_err());
    }

    #[test]
    fn merge_unique_keeps_order_and_skips_duplicates() {
        let mut list = vec![item(1), item(2)];
        merge_unique(&mut list, vec![item(2), item(3), item(3), item(4)]);
        let ids: Vec<u32> = list.iter().map(|i| i.id).collect();
        assert_eq!(ids, vec![1, 2, 3, 4]);
    }

    #[test]
    fn load_more_waits_for_the_page_in_flight() {
        assert!(!can_load_more(1, 2, false)); // page 2 requested, not loaded yet
        assert!(can_load_more(2, 2, false));
        assert!(can_load_more(1, 2, true)); // failed: retry allowed
        assert!(!can_load_more(MAX_PAGE, MAX_PAGE, false)); // nothing after the last page
    }

    #[test]
    fn polling_backs_off_and_has_a_ceiling() {
        let delays: Vec<u64> = (0..7).map(poll_delay_secs).collect();
        assert_eq!(delays, vec![5, 10, 20, 40, 60, 60, 60]);
    }
}
