//! Module 19 server entry point.
//! Axum owns the HTTP listener while Leptos renders the app and server functions.
//! `cargo-leptos` builds this native server and the browser hydration bundle.
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

// ============================================================================
// LESSON 1: Starting The Axum And Leptos Server
// ============================================================================

/// Explains the `main` step in this lesson.
///
/// What it does: starts the runnable entry point for this module.
/// Why it matters: It makes the module runnable and shows learners where execution really begins.
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{
        response::sse::{Event, KeepAlive, Sse},
        routing::get,
        Router,
    };
    use futures_util::StreamExt;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use module_19_sse::app::*;
    use std::{convert::Infallible, time::Duration};
    use tokio_stream::wrappers::IntervalStream;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    /// Explains the `clock stream` step in this lesson.
    ///
    /// What it does: implements the `clock stream` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    async fn clock_stream() -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
        let stream = IntervalStream::new(tokio::time::interval(Duration::from_secs(1))).map(|_| {
            Ok(Event::default()
                .event("clock")
                .data(chrono::Utc::now().to_rfc3339()))
        });
        Sse::new(stream).keep_alive(KeepAlive::default())
    }

    /// Explains the `tick stream` step in this lesson.
    ///
    /// What it does: implements the `tick stream` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    async fn tick_stream() -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
        let mut tick = 0_u64;
        let stream =
            IntervalStream::new(tokio::time::interval(Duration::from_millis(500))).map(move |_| {
                tick += 1;
                Ok(Event::default().event("tick").data(tick.to_string()))
            });
        Sse::new(stream).keep_alive(KeepAlive::default())
    }

    /// Explains the `progress stream` step in this lesson.
    ///
    /// What it does: implements the `progress stream` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    async fn progress_stream() -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
        let stream = IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
            .take(11)
            .enumerate()
            .map(|(step, _)| {
                Ok(Event::default()
                    .event("progress")
                    .data((step * 10).to_string()))
            });
        Sse::new(stream).keep_alive(KeepAlive::default())
    }

    /// Explains the `notification stream` step in this lesson.
    ///
    /// What it does: implements the `notification stream` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    async fn notification_stream(
    ) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
        let messages = [
            "Build started",
            "Assets compiled",
            "WASM optimized",
            "Deployment ready",
        ];
        let stream = IntervalStream::new(tokio::time::interval(Duration::from_secs(2)))
            .enumerate()
            .map(move |(index, _)| {
                Ok(Event::default()
                    .event("notification")
                    .data(messages[index % messages.len()]))
            });
        Sse::new(stream).keep_alive(KeepAlive::default())
    }

    let app = Router::new()
        .route("/sse/clock", get(clock_stream))
        .route("/sse/ticks", get(tick_stream))
        .route("/sse/progress", get(progress_stream))
        .route("/sse/notifications", get(notification_stream))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// Explains the `main` step in this lesson.
///
/// What it does: starts the runnable entry point for this module.
/// Why it matters: It makes the module runnable and shows learners where execution really begins.
#[cfg(not(feature = "ssr"))]
pub fn main() {
    // The browser entry point is exported from `src/lib.rs` for hydration builds.
}
