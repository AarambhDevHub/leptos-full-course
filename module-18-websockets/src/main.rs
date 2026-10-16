//! Module 18 server entry point.
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
        extract::ws::{Message, WebSocket, WebSocketUpgrade},
        response::IntoResponse,
        routing::get,
        Router,
    };
    use futures_util::{SinkExt, StreamExt};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use module_18_websockets::app::*;
    use std::sync::OnceLock;
    use tokio::sync::broadcast;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    /// Explains the `room` step in this lesson.
    ///
    /// What it does: implements the `room` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    fn room() -> &'static broadcast::Sender<String> {
        static ROOM: OnceLock<broadcast::Sender<String>> = OnceLock::new();
        ROOM.get_or_init(|| {
            let (tx, _) = broadcast::channel(64);
            tx
        })
    }

    /// Explains the `ws handler` step in this lesson.
    ///
    /// What it does: handles the `ws handler` request path or event callback for this module.
    /// Why it matters: Handler functions make server and browser events explicit instead of hiding them inside setup code.
    async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
        ws.on_upgrade(handle_socket)
    }

    /// Explains the `handle socket` step in this lesson.
    ///
    /// What it does: handles the `handle socket` request path or event callback for this module.
    /// Why it matters: Handler functions make server and browser events explicit instead of hiding them inside setup code.
    async fn handle_socket(socket: WebSocket) {
        let mut rx = room().subscribe();
        let (mut sender, mut receiver) = socket.split();

        let send_task = tokio::spawn(async move {
            while let Ok(text) = rx.recv().await {
                if sender.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
        });

        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let _ = room().send(text.to_string());
        }

        send_task.abort();
    }

    let app = Router::new()
        .route("/ws", get(ws_handler))
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
