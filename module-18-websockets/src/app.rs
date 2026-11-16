//! Module 18: WebSockets.
//! A JSON chat protocol with browser WebSocket lifecycle state and reconnect
//! guidance. The `/ws` broadcast handler is registered in `src/main.rs`.
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

const MODULE_TITLE: &str = "Module 18 - WebSockets";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum WsMsg {
    Chat { user: String, text: String },
    Join { user: String },
    Leave { user: String },
}

/// The longest user name the room accepts, in characters.
pub const MAX_NAME_CHARS: usize = 32;
/// The longest chat text the room accepts, in characters.
pub const MAX_TEXT_CHARS: usize = 500;
/// The largest WebSocket message the server will read, in bytes (enough for the limits above, even in Hindi).
pub const MAX_MESSAGE_BYTES: usize = 8 * 1024;
/// How many times the browser retries a dropped connection before giving up.
pub const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Explains the `reconnect delay ms` step in this lesson.
///
/// What it does: returns how long to wait before reconnect attempt number `attempt` (1, 2, 3...): one second, two, then four seconds for every later attempt.
/// Why it matters: Waiting longer after each failure (exponential backoff) stops a crowd of browsers from hammering a server that is trying to recover.
pub fn reconnect_delay_ms(attempt: u32) -> u64 {
    500 * 2_u64.pow(attempt.clamp(1, 3))
}

/// Explains the `parse client message` step in this lesson.
///
/// What it does: validates one text frame from a browser and returns the cleaned message to broadcast.
/// Why it matters: Everything that arrives over a socket is untrusted. Without this check the room relayed garbage, impersonation-sized names and multi-megabyte texts to everybody.
pub fn parse_client_message(text: &str) -> Result<WsMsg, &'static str> {
    let message: WsMsg = serde_json::from_str(text).map_err(|_| "not a valid chat message")?;
    let clean_name = |user: String| -> Result<String, &'static str> {
        let user = user.trim().to_string();
        if user.is_empty() {
            Err("empty name")
        } else if user.chars().count() > MAX_NAME_CHARS {
            Err("name too long")
        } else {
            Ok(user)
        }
    };
    match message {
        WsMsg::Chat { user, text } => {
            let text = text.trim().to_string();
            if text.is_empty() {
                Err("empty message")
            } else if text.chars().count() > MAX_TEXT_CHARS {
                Err("message too long")
            } else {
                Ok(WsMsg::Chat {
                    user: clean_name(user)?,
                    text,
                })
            }
        }
        WsMsg::Join { user } => Ok(WsMsg::Join {
            user: clean_name(user)?,
        }),
        WsMsg::Leave { user } => Ok(WsMsg::Leave {
            user: clean_name(user)?,
        }),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum SocketStatus {
    Connecting,
    Connected,
    Disconnected,
}

impl SocketStatus {
    // ============================================================================
    // LESSON 1: Full-Stack App Structure
    // ============================================================================

    /// Explains the `label` step in this lesson.
    ///
    /// What it does: implements the `label` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    fn label(self) -> &'static str {
        match self {
            SocketStatus::Connecting => "Connecting",
            SocketStatus::Connected => "Connected",
            SocketStatus::Disconnected => "Disconnected",
        }
    }
}

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
        <Meta name="description" content="WebSocket chat in the Leptos 0.8 full course"/>
        <Router>
            <main style="max-width: 1040px; margin: 0 auto; padding: 32px; font-family: system-ui, sans-serif;">
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=ChatPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Explains the `chat page` step in this lesson.
///
/// What it does: renders the `ChatPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ChatPage() -> impl IntoView {
    let user = RwSignal::new("learner".to_string());
    let draft = RwSignal::new(String::new());
    let status = RwSignal::new(SocketStatus::Disconnected);
    let retries = RwSignal::new(0_u32);
    let messages = RwSignal::new(Vec::<WsMsg>::new());

    view! {
        <section style="display: grid; gap: 20px;">
            <header>
                <h1>{MODULE_TITLE}</h1>
                <p>"This lesson uses a typed JSON protocol and an Axum broadcast room at /ws."</p>
            </header>
            <ConnectionPanel status retries messages user/>
            <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
                <h2>"Send a message"</h2>
                <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                    <input prop:value=user on:input:target=move |event| user.set(event.target().value())/>
                    <input
                        placeholder="Message"
                        prop:value=draft
                        on:input:target=move |event| draft.set(event.target().value())
                    />
                    <button
                        disabled=move || status.get() != SocketStatus::Connected
                        on:click=move |_| {
                            let text = draft.get_untracked();
                            if text.trim().is_empty() {
                                return;
                            }
                            // No local copy: the room sends every message back to its sender too,
                            // so the log shows it exactly once, in the order everybody sees it.
                            #[cfg(feature = "hydrate")]
                            send_ws_message(WsMsg::Chat { user: user.get_untracked(), text });
                            #[cfg(not(feature = "hydrate"))]
                            let _ = text;
                            draft.set(String::new());
                        }
                    >"Send"</button>
                </div>
            </section>
            <MessageLog messages/>
            <ProtocolPanel/>
        </section>
    }
}

/// Explains the `connection panel` step in this lesson.
///
/// What it does: renders the `ConnectionPanel` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ConnectionPanel(
    status: RwSignal<SocketStatus>,
    retries: RwSignal<u32>,
    messages: RwSignal<Vec<WsMsg>>,
    user: RwSignal<String>,
) -> impl IntoView {
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = messages;
        let _ = user;
    }

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Connection"</h2>
            <p>"Status: " <strong>{move || status.get().label()}</strong></p>
            <p>"Reconnect attempts: " {retries}</p>
            <button
                disabled=move || status.get() != SocketStatus::Disconnected
                on:click=move |_| {
                    status.set(SocketStatus::Connecting);
                    retries.set(0);
                    #[cfg(feature = "hydrate")]
                    connect_ws(status, retries, messages, user.get_untracked());
                }
            >"Connect"</button>
            <button
                disabled=move || status.get() == SocketStatus::Disconnected
                on:click=move |_| {
                    #[cfg(feature = "hydrate")]
                    leave_ws();
                    status.set(SocketStatus::Disconnected);
                    retries.set(0);
                }
            >"Leave"</button>
        </section>
    }
}

/// Explains the `websocket url` step in this lesson.
///
/// What it does: performs the `websocket url` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
#[cfg(feature = "hydrate")]
fn websocket_url() -> String {
    let location = web_sys::window().unwrap().location();
    let protocol = if location.protocol().unwrap_or_default() == "https:" {
        "wss"
    } else {
        "ws"
    };
    let host = location.host().unwrap_or_else(|_| "localhost:3000".into());
    format!("{protocol}://{host}/ws")
}

#[cfg(feature = "hydrate")]
thread_local! {
    static ACTIVE_SOCKET: std::cell::RefCell<Option<web_sys::WebSocket>> = const { std::cell::RefCell::new(None) };
    /// Bumped by every Connect and by Leave. A `close` event that belongs to an older
    /// connection sees a different number and does not start a reconnect.
    static GENERATION: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Explains the `next generation` step in this lesson.
///
/// What it does: starts a new connection "era" and returns its number.
/// Why it matters: Old sockets and pending reconnect timers compare their number with the current one, so only the newest connection may reconnect.
#[cfg(feature = "hydrate")]
fn next_generation() -> u32 {
    GENERATION.with(|generation| {
        generation.set(generation.get() + 1);
        generation.get()
    })
}

/// Explains the `leave ws` step in this lesson.
///
/// What it does: closes the active socket on purpose and cancels any pending reconnect.
/// Why it matters: Flipping a status label does not close a connection. The server announces the departure when the socket closes.
#[cfg(feature = "hydrate")]
fn leave_ws() {
    next_generation();
    ACTIVE_SOCKET.with(|slot| {
        if let Some(socket) = slot.borrow_mut().take() {
            let _ = socket.close();
        }
    });
}

/// Explains the `connect ws` step in this lesson.
///
/// What it does: opens one socket, wires its events to the signals, and reconnects with backoff if the connection drops by itself.
/// Why it matters: Each connection owns a generation number, so Connect twice, Leave, and a late reconnect timer can never leave two live sockets or an unwanted reconnect.
#[cfg(feature = "hydrate")]
fn connect_ws(
    status: RwSignal<SocketStatus>,
    retries: RwSignal<u32>,
    messages: RwSignal<Vec<WsMsg>>,
    user: String,
) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    // Close any earlier socket first. Bumping the generation makes its close event harmless.
    leave_ws();
    let generation = next_generation();

    let Ok(socket) = web_sys::WebSocket::new(&websocket_url()) else {
        status.set(SocketStatus::Disconnected);
        return;
    };

    let on_open = Closure::<dyn FnMut()>::wrap(Box::new({
        let socket = socket.clone();
        let user = user.clone();
        move || {
            status.set(SocketStatus::Connected);
            // A good connection resets the budget, so a long session never runs out of retries.
            retries.set(0);
            send_ws_message_with_socket(&socket, WsMsg::Join { user: user.clone() });
        }
    }));
    socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    on_open.forget();

    let on_message = Closure::<dyn FnMut(web_sys::MessageEvent)>::wrap(Box::new(
        move |event: web_sys::MessageEvent| {
            if let Some(text) = event.data().as_string() {
                if let Ok(message) = serde_json::from_str::<WsMsg>(&text) {
                    messages.update(|items| items.push(message));
                }
            }
        },
    ));
    socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    let on_close = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        if GENERATION.with(|current| current.get()) != generation {
            return; // replaced or left on purpose: do not reconnect
        }
        let attempt = retries.get_untracked() + 1;
        if attempt > MAX_RECONNECT_ATTEMPTS {
            status.set(SocketStatus::Disconnected);
            return;
        }
        retries.set(attempt);
        status.set(SocketStatus::Connecting);
        let user = user.clone();
        leptos::task::spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(reconnect_delay_ms(
                attempt,
            )))
            .await;
            // The user may have pressed Leave or Connect while we were waiting.
            if GENERATION.with(|current| current.get()) == generation {
                connect_ws(status, retries, messages, user);
            }
        });
    }));
    socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    on_close.forget();

    ACTIVE_SOCKET.with(|slot| slot.replace(Some(socket)));
}

/// Explains the `send ws message` step in this lesson.
///
/// What it does: performs the `send ws message` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
#[cfg(feature = "hydrate")]
fn send_ws_message(message: WsMsg) {
    ACTIVE_SOCKET.with(|slot| {
        if let Some(socket) = slot.borrow().as_ref() {
            send_ws_message_with_socket(socket, message);
        }
    });
}

/// Explains the `send ws message with socket` step in this lesson.
///
/// What it does: performs the `send ws message with socket` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
#[cfg(feature = "hydrate")]
fn send_ws_message_with_socket(socket: &web_sys::WebSocket, message: WsMsg) {
    if let Ok(text) = serde_json::to_string(&message) {
        let _ = socket.send_with_str(&text);
    }
}

/// Explains the `message log` step in this lesson.
///
/// What it does: renders the `MessageLog` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn MessageLog(messages: RwSignal<Vec<WsMsg>>) -> impl IntoView {
    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Room messages"</h2>
            <ul style="display: grid; gap: 8px; padding-left: 0; list-style: none;">
                <For
                    each=move || { messages.get().into_iter().enumerate().collect::<Vec<_>>() }
                    key=|(index, _)| *index
                    children=|(_, message)| view! {
                        <li style="padding: 10px; background: #f6f7fb; border-radius: 8px;">
                            {match message {
                                WsMsg::Chat { user, text } => format!("{user}: {text}"),
                                WsMsg::Join { user } => format!("{user} joined"),
                                WsMsg::Leave { user } => format!("{user} left"),
                            }}
                        </li>
                    }
                />
            </ul>
        </section>
    }
}

/// Explains the `protocol panel` step in this lesson.
///
/// What it does: renders the `ProtocolPanel` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ProtocolPanel() -> impl IntoView {
    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"JSON protocol"</h2>
            <pre style="white-space: pre-wrap;">{
                r#"enum WsMsg {
    Chat { user, text },
    Join { user },
    Leave { user },
}"#
            }</pre>
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
            <p>"The requested WebSocket route does not exist."</p>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chat(user: &str, text: &str) -> String {
        serde_json::to_string(&WsMsg::Chat {
            user: user.into(),
            text: text.into(),
        })
        .unwrap()
    }

    #[test]
    fn backoff_grows_then_stays_at_four_seconds() {
        let delays: Vec<u64> = (1..=6).map(reconnect_delay_ms).collect();
        assert_eq!(delays, vec![1000, 2000, 4000, 4000, 4000, 4000]);
    }

    #[test]
    fn backoff_never_decreases() {
        for attempt in 1..20 {
            assert!(reconnect_delay_ms(attempt + 1) >= reconnect_delay_ms(attempt));
        }
    }

    #[test]
    fn accepts_a_normal_chat_message_and_trims_it() {
        let parsed = parse_client_message(&chat(" Asha ", "  hello  ")).unwrap();
        assert_eq!(
            parsed,
            WsMsg::Chat {
                user: "Asha".into(),
                text: "hello".into()
            }
        );
    }

    #[test]
    fn rejects_text_that_is_not_our_protocol() {
        assert!(parse_client_message("not json at all").is_err());
        assert!(parse_client_message("{}").is_err());
        assert!(parse_client_message(r#"{"Shout":{"text":"hi"}}"#).is_err());
    }

    #[test]
    fn rejects_empty_and_whitespace_only_parts() {
        assert!(parse_client_message(&chat("Asha", "   ")).is_err());
        assert!(parse_client_message(&chat("  ", "hi")).is_err());
        assert!(parse_client_message(r#"{"Join":{"user":""}}"#).is_err());
    }

    #[test]
    fn length_limits_count_characters_not_bytes() {
        // 500 Hindi letters are 1500 bytes but exactly at the limit, so they are accepted.
        let hindi = "न".repeat(MAX_TEXT_CHARS);
        assert!(parse_client_message(&chat("Asha", &hindi)).is_ok());
        assert!(parse_client_message(&chat("Asha", &format!("{hindi}न"))).is_err());
        assert!(parse_client_message(&chat(&"a".repeat(MAX_NAME_CHARS), "hi")).is_ok());
        assert!(parse_client_message(&chat(&"a".repeat(MAX_NAME_CHARS + 1), "hi")).is_err());
    }

    #[test]
    fn a_maximum_size_message_fits_in_the_frame_limit() {
        let biggest = chat(&"न".repeat(MAX_NAME_CHARS), &"न".repeat(MAX_TEXT_CHARS));
        assert!(biggest.len() < MAX_MESSAGE_BYTES);
    }

    #[test]
    fn join_and_leave_are_accepted() {
        assert_eq!(
            parse_client_message(r#"{"Join":{"user":"Asha"}}"#).unwrap(),
            WsMsg::Join {
                user: "Asha".into()
            }
        );
        assert_eq!(
            parse_client_message(r#"{"Leave":{"user":"Asha"}}"#).unwrap(),
            WsMsg::Leave {
                user: "Asha".into()
            }
        );
    }
}
