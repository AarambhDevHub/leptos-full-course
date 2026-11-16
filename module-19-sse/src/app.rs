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

/// What the browser tells us about one stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum StreamStatus {
    Closed,
    Connecting,
    Open,
    /// The connection dropped and the browser is trying again by itself.
    Reconnecting,
    /// The server said it is finished and we closed the stream on purpose.
    Finished,
}

impl StreamStatus {
    /// Explains the `label` step in this lesson.
    ///
    /// What it does: returns the text shown for this status.
    /// Why it matters: A small enum makes every state the page can be in explicit.
    fn label(self) -> &'static str {
        match self {
            StreamStatus::Closed => "closed",
            StreamStatus::Connecting => "connecting",
            StreamStatus::Open => "open",
            StreamStatus::Reconnecting => "connection lost, reconnecting",
            StreamStatus::Finished => "finished",
        }
    }
}

/// Explains the `status after error` step in this lesson.
///
/// What it does: decides what an `error` event means, from the EventSource `readyState` (0 connecting, 1 open, 2 closed).
/// Why it matters: EventSource fires `error` both for a dropped connection that it will retry and for a connection that is finished for good. Only `readyState` tells them apart.
#[allow(dead_code)]
fn status_after_error(ready_state: u16) -> StreamStatus {
    if ready_state == 2 {
        StreamStatus::Closed
    } else {
        StreamStatus::Reconnecting
    }
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
                <SsePanel title="Clock" endpoint="/sse/clock" event="clock"/>
                <SsePanel title="Ticks" endpoint="/sse/ticks" event="tick"/>
                <SsePanel title="Progress" endpoint="/sse/progress" event="progress"/>
                <SsePanel title="Notifications" endpoint="/sse/notifications" event="notification"/>
            </div>
            <LifecyclePanel/>
        </section>
    }
}

/// Explains the `sse panel` step in this lesson.
///
/// What it does: renders the `SsePanel` component for one focused part of the lesson UI.
/// Why it matters: The panel owns its stream, so it connects once, can disconnect, and closes the stream when the component leaves the page.
#[component]
fn SsePanel(
    title: &'static str,
    endpoint: &'static str,
    /// The event name the server sends on this endpoint.
    event: &'static str,
) -> impl IntoView {
    let status = RwSignal::new(StreamStatus::Closed);
    let latest = RwSignal::new("No events yet".to_string());
    let count = RwSignal::new(0_u32);

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = event;
    }

    // Cleanup belongs to the component, which has an owner. Inside a click handler there is no owner to attach it to.
    on_cleanup(move || {
        #[cfg(feature = "hydrate")]
        close_event_source(endpoint);
    });

    view! {
        <article style="border: 1px solid #d9dee8; border-radius: 8px; padding: 18px;">
            <h2>{title}</h2>
            <p>"Endpoint: " <code>{endpoint}</code></p>
            <p>"Event name: " <code>{event}</code></p>
            <p>"Status: " <strong>{move || status.get().label()}</strong></p>
            <p>"Events: " {count}</p>
            <pre style="white-space: pre-wrap; background: #f6f7fb; padding: 10px; border-radius: 8px;">{latest}</pre>
            <button
                disabled=move || matches!(status.get(), StreamStatus::Connecting | StreamStatus::Open | StreamStatus::Reconnecting)
                on:click=move |_| {
                    status.set(StreamStatus::Connecting);
                    count.set(0);
                    latest.set("No events yet".to_string());
                    #[cfg(feature = "hydrate")]
                    open_event_source(endpoint, event, status, latest, count);
                }
            >"Connect"</button>
            <button
                disabled=move || matches!(status.get(), StreamStatus::Closed | StreamStatus::Finished)
                on:click=move |_| {
                    #[cfg(feature = "hydrate")]
                    close_event_source(endpoint);
                    status.set(StreamStatus::Closed);
                }
            >"Disconnect"</button>
        </article>
    }
}

#[cfg(feature = "hydrate")]
thread_local! {
    /// One live EventSource per endpoint, so it can be closed from anywhere.
    static SOURCES: std::cell::RefCell<std::collections::HashMap<&'static str, web_sys::EventSource>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Explains the `close event source` step in this lesson.
///
/// What it does: closes and forgets the stream of one endpoint, if there is one.
/// Why it matters: An EventSource that is not closed keeps its connection and reconnects forever, even after the page stopped caring.
#[cfg(feature = "hydrate")]
fn close_event_source(endpoint: &'static str) {
    SOURCES.with(|sources| {
        if let Some(source) = sources.borrow_mut().remove(endpoint) {
            source.close();
        }
    });
}

/// Explains the `open event source` step in this lesson.
///
/// What it does: opens a stream and listens for the one event name the server sends, plus a `done` event that ends the stream.
/// Why it matters: A server event with a name is delivered only to listeners for that name. `onmessage` receives unnamed events only, so it never saw our `tick` or `clock` events.
#[cfg(feature = "hydrate")]
fn open_event_source(
    endpoint: &'static str,
    event: &'static str,
    status: RwSignal<StreamStatus>,
    latest: RwSignal<String>,
    count: RwSignal<u32>,
) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    // Pressing Connect twice must not leave two streams running.
    close_event_source(endpoint);

    let Ok(source) = web_sys::EventSource::new(endpoint) else {
        status.set(StreamStatus::Closed);
        return;
    };

    let on_open = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        status.set(StreamStatus::Open);
    }));
    source.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    on_open.forget();

    let on_event = Closure::<dyn FnMut(web_sys::MessageEvent)>::wrap(Box::new(
        move |message: web_sys::MessageEvent| {
            if let Some(text) = message.data().as_string() {
                latest.set(text);
                count.update(|value| *value += 1);
            }
        },
    ));
    let _ = source.add_event_listener_with_callback(event, on_event.as_ref().unchecked_ref());
    on_event.forget();

    // The server ends a finite stream with `done`. Closing here stops the browser from reconnecting and restarting it.
    let on_done = Closure::<dyn FnMut(web_sys::MessageEvent)>::wrap(Box::new({
        move |_message: web_sys::MessageEvent| {
            close_event_source(endpoint);
            status.set(StreamStatus::Finished);
        }
    }));
    let _ = source.add_event_listener_with_callback("done", on_done.as_ref().unchecked_ref());
    on_done.forget();

    let on_error = Closure::<dyn FnMut()>::wrap(Box::new({
        let source = source.clone();
        move || {
            status.set(status_after_error(source.ready_state()));
        }
    }));
    source.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_error.forget();

    SOURCES.with(|sources| sources.borrow_mut().insert(endpoint, source));
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
                <li>"Attach open and error handlers, and one listener for each named event the server sends."</li>
                <li>"Update Leptos signals from each callback."</li>
                <li>"End a finite stream with a done event, and close the stream on Disconnect and in on_cleanup."</li>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_connection_is_closed_not_reconnecting() {
        // readyState 2 (CLOSED): the browser will not retry.
        assert_eq!(status_after_error(2), StreamStatus::Closed);
    }

    #[test]
    fn connecting_or_open_error_means_the_browser_is_retrying() {
        assert_eq!(status_after_error(0), StreamStatus::Reconnecting);
        assert_eq!(status_after_error(1), StreamStatus::Reconnecting);
    }

    #[test]
    fn every_status_has_a_distinct_label() {
        let labels = [
            StreamStatus::Closed,
            StreamStatus::Connecting,
            StreamStatus::Open,
            StreamStatus::Reconnecting,
            StreamStatus::Finished,
        ]
        .map(StreamStatus::label);
        let mut sorted = labels.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), labels.len());
    }
}
