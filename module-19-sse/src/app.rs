//! Module 19: Server-Sent Events.
//! Browser EventSource lifecycle management with four independent stream
//! examples wired to Axum handlers in `src/main.rs`.
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

const MODULE_TITLE: &str = "Module 19 - Server-Sent Events";

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
        <Meta name="description" content="Server-Sent Events in the Leptos 0.8 full course"/>
        <Router>
            <main style="max-width: 1040px; margin: 0 auto; padding: 32px; font-family: system-ui, sans-serif;">
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=SsePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Explains the `sse page` step in this lesson.
///
/// What it does: renders the `SsePage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn SsePage() -> impl IntoView {
    view! {
        <section style="display: grid; gap: 20px;">
            <header>
                <h1>{MODULE_TITLE}</h1>
                <p>"EventSource opens a one-way stream, receives server events, and must be closed during cleanup."</p>
            </header>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 16px;">
                <SsePanel title="Clock" endpoint="/sse/clock"/>
                <SsePanel title="Ticks" endpoint="/sse/ticks"/>
                <SsePanel title="Progress" endpoint="/sse/progress"/>
                <SsePanel title="Notifications" endpoint="/sse/notifications"/>
            </div>
            <LifecyclePanel/>
        </section>
    }
}

/// Explains the `sse panel` step in this lesson.
///
/// What it does: renders the `SsePanel` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn SsePanel(title: &'static str, endpoint: &'static str) -> impl IntoView {
    let status = RwSignal::new("closed".to_string());
    let latest = RwSignal::new("No events yet".to_string());
    let count = RwSignal::new(0_u32);

    view! {
        <article style="border: 1px solid #d9dee8; border-radius: 8px; padding: 18px;">
            <h2>{title}</h2>
            <p>"Endpoint: " <code>{endpoint}</code></p>
            <p>"Status: " <strong>{status}</strong></p>
            <p>"Events: " {count}</p>
            <pre style="white-space: pre-wrap; background: #f6f7fb; padding: 10px; border-radius: 8px;">{latest}</pre>
            <button on:click=move |_| {
                #[cfg(feature = "hydrate")]
                open_event_source(endpoint, status, latest, count);
            }>"Connect"</button>
        </article>
    }
}

/// Explains the `open event source` step in this lesson.
///
/// What it does: performs the `open event source` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
#[cfg(feature = "hydrate")]
fn open_event_source(
    endpoint: &'static str,
    status: RwSignal<String>,
    latest: RwSignal<String>,
    count: RwSignal<u32>,
) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let Ok(source) = web_sys::EventSource::new(endpoint) else {
        status.set("failed".into());
        return;
    };

    let on_open = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        status.set("open".into());
    }));
    source.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    on_open.forget();

    let on_message = Closure::<dyn FnMut(web_sys::MessageEvent)>::wrap(Box::new(
        move |event: web_sys::MessageEvent| {
            if let Some(text) = event.data().as_string() {
                latest.set(text);
                count.update(|value| *value += 1);
            }
        },
    ));
    source.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    let on_error = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        status.set("error".into());
    }));
    source.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_error.forget();

    on_cleanup(move || {
        source.close();
    });
}

/// Explains the `lifecycle panel` step in this lesson.
///
/// What it does: renders the `LifecyclePanel` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn LifecyclePanel() -> impl IntoView {
    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Lifecycle pattern"</h2>
            <ol>
                <li>"Create EventSource with the endpoint URL."</li>
                <li>"Attach open, message, and error handlers."</li>
                <li>"Update Leptos signals from each callback."</li>
                <li>"Close the stream in on_cleanup to avoid leaked connections."</li>
            </ol>
        </section>
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
            <p>"The requested SSE route does not exist."</p>
        </section>
    }
}
