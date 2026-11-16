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
    use tokio::sync::broadcast::{self, error::RecvError};

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
    /// Why it matters: The upgrade is where the server sets the largest message it is willing to read, before any client text is parsed.
    async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
        ws.max_message_size(MAX_MESSAGE_BYTES)
            .max_frame_size(MAX_MESSAGE_BYTES)
            .on_upgrade(handle_socket)
    }

    /// Explains the `broadcast` step in this lesson.
    ///
    /// What it does: serializes one message and sends it to every connected client.
    /// Why it matters: The server always re-serializes a message it has validated, so clients only ever receive well-formed JSON.
    fn broadcast(message: &WsMsg) {
        if let Ok(text) = serde_json::to_string(message) {
            let _ = room().send(text);
        }
    }

    /// Explains the `handle socket` step in this lesson.
    ///
    /// What it does: relays validated messages from one client to the room, and tells the room when that client leaves.
    /// Why it matters: It keeps the connection alive through pings and binary frames, survives a slow reader that misses messages, and announces a departure even when the browser just vanishes.
    async fn handle_socket(socket: WebSocket) {
        let mut rx = room().subscribe();
        let (mut sender, mut receiver) = socket.split();

        let send_task = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(text) => {
                        if sender.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    // A slow client fell more than 64 messages behind. Skip what it missed
                    // and keep going, instead of silently ending its stream.
                    Err(RecvError::Lagged(missed)) => {
                        tracing::warn!("client lagged, skipped {missed} messages");
                    }
                    Err(RecvError::Closed) => break,
                }
            }
        });

        let mut joined_as: Option<String> = None;
        while let Some(Ok(frame)) = receiver.next().await {
            match frame {
                Message::Text(text) => match parse_client_message(&text) {
                    Ok(message) => {
                        match &message {
                            WsMsg::Join { user } => joined_as = Some(user.clone()),
                            WsMsg::Leave { .. } => joined_as = None,
                            WsMsg::Chat { .. } => {}
                        }
                        broadcast(&message);
                    }
                    Err(reason) => tracing::debug!("dropped a client message: {reason}"),
                },
                Message::Close(_) => break,
                // Pings and pongs are answered by Axum. Binary frames are not part of our protocol.
                _ => {}
            }
        }

        if let Some(user) = joined_as {
            broadcast(&WsMsg::Leave { user });
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
