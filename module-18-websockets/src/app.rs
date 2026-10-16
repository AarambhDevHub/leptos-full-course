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
                    <button on:click=move |_| {
                        let text = draft.get();
                        if text.trim().is_empty() {
                            return;
                        }
                        messages.update(|items| {
                            items.push(WsMsg::Chat {
                                user: user.get(),
                                text: text.clone(),
                            });
                        });
                        #[cfg(feature = "hydrate")]
                        send_ws_message(WsMsg::Chat { user: user.get(), text });
                        draft.set(String::new());
                    }>"Send"</button>
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
            <button on:click=move |_| {
                status.set(SocketStatus::Connecting);
                retries.set(0);
                #[cfg(feature = "hydrate")]
                connect_ws(status, retries, messages, user.get());
            }>"Connect"</button>
            <button on:click=move |_| {
                #[cfg(feature = "hydrate")]
                send_ws_message(WsMsg::Leave { user: user.get() });
                status.set(SocketStatus::Disconnected);
            }>"Leave"</button>
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
}

/// Explains the `connect ws` step in this lesson.
///
/// What it does: performs the `connect ws` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
#[cfg(feature = "hydrate")]
fn connect_ws(
    status: RwSignal<SocketStatus>,
    retries: RwSignal<u32>,
    messages: RwSignal<Vec<WsMsg>>,
    user: String,
) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let Ok(socket) = web_sys::WebSocket::new(&websocket_url()) else {
        status.set(SocketStatus::Disconnected);
        return;
    };

    let on_open = Closure::<dyn FnMut()>::wrap(Box::new({
        let socket = socket.clone();
        move || {
            status.set(SocketStatus::Connected);
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
        status.set(SocketStatus::Disconnected);
        if retries.get() < 5 {
            let next = retries.get() + 1;
            retries.set(next);
            leptos::task::spawn_local(async move {
                let delay = 2_u64.pow(next.min(3)) * 500;
                gloo_timers::future::sleep(std::time::Duration::from_millis(delay)).await;
                connect_ws(status, retries, messages, "learner".into());
            });
        }
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
