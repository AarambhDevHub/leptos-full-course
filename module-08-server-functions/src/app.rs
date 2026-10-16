//! Module 08: Server Functions.
//! This module demonstrates server-only logic, ServerAction, ActionForm, and
//! a small in-memory todo store.
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
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

#[cfg(feature = "ssr")]
static TODOS: Mutex<Vec<TodoItem>> = Mutex::new(Vec::new());
#[cfg(feature = "ssr")]
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TodoItem {
    id: u64,
    title: String,
    done: bool,
}

const MODULE_TITLE: &str = "Module 08 - Server Functions";

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
        <Meta name="description" content="Server functions in the Leptos 0.8 full course"/>
        <Router>
            <main style="max-width: 940px; margin: 0 auto; padding: 32px; font-family: system-ui, sans-serif;">
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

// ============================================================================
// LESSON 3: Server Functions And Trusted Work
// ============================================================================

/// Explains the `get server message` step in this lesson.
///
/// What it does: runs the `get server message` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn get_server_message() -> Result<String, ServerFnError> {
    let host = std::env::var("HOSTNAME").unwrap_or_else(|_| "local-development-host".into());
    Ok(format!("Hello from the server running on {host}."))
}

/// Explains the `add` step in this lesson.
///
/// What it does: runs the `add` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn add(a: f64, b: f64) -> Result<f64, ServerFnError> {
    Ok(a + b)
}

/// Explains the `list todos` step in this lesson.
///
/// What it does: runs the `list todos` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn list_todos() -> Result<Vec<TodoItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let mut todos = TODOS
            .lock()
            .map_err(|_| ServerFnError::new("todo store lock poisoned"))?;
        if todos.is_empty() {
            todos.push(TodoItem {
                id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
                title: "Learn #[server]".into(),
                done: false,
            });
            todos.push(TodoItem {
                id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
                title: "Submit an ActionForm".into(),
                done: true,
            });
        }
        Ok(todos.clone())
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// Explains the `add todo` step in this lesson.
///
/// What it does: runs the `add todo` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn add_todo(title: String) -> Result<TodoItem, ServerFnError> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("title cannot be empty"));
    }

    #[cfg(feature = "ssr")]
    {
        let todo = TodoItem {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            title,
            done: false,
        };
        TODOS
            .lock()
            .map_err(|_| ServerFnError::new("todo store lock poisoned"))?
            .push(todo.clone());
        Ok(todo)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(TodoItem {
            id: 0,
            title,
            done: false,
        })
    }
}

/// Explains the `toggle todo` step in this lesson.
///
/// What it does: runs the `toggle todo` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn toggle_todo(id: u64) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        if let Some(todo) = TODOS
            .lock()
            .map_err(|_| ServerFnError::new("todo store lock poisoned"))?
            .iter_mut()
            .find(|todo| todo.id == id)
        {
            todo.done = !todo.done;
        }
    }
    Ok(())
}

/// Explains the `delete todo` step in this lesson.
///
/// What it does: runs the `delete todo` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn delete_todo(id: u64) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        TODOS
            .lock()
            .map_err(|_| ServerFnError::new("todo store lock poisoned"))?
            .retain(|todo| todo.id != id);
    }
    Ok(())
}

/// Explains the `home page` step in this lesson.
///
/// What it does: renders the `HomePage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <Header/>
        <ServerMessage/>
        <Adder/>
        <TodoApp/>
    }
}

/// Explains the `header` step in this lesson.
///
/// What it does: renders the `Header` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Header() -> impl IntoView {
    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px; margin-bottom: 16px;">
            <p style="color: #164a8b; font-weight: 700;">"Leptos 0.8 SSR"</p>
            <h1>{MODULE_TITLE}</h1>
            <p>"Server functions let the browser call typed Rust functions while the real work stays on the server."</p>
        </section>
    }
}

/// Explains the `server message` step in this lesson.
///
/// What it does: renders the `ServerMessage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ServerMessage() -> impl IntoView {
    let message = Resource::new(|| (), |_| get_server_message());

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px; margin-bottom: 16px;">
            <h2>"Server Resource"</h2>
            <Suspense fallback=|| view! { <p>"Loading server hostname..."</p> }>
                {move || message.get().map(|result| match result {
                    Ok(message) => view! { <p>{message}</p> }.into_any(),
                    Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
                })}
            </Suspense>
        </section>
    }
}

/// Explains the `adder` step in this lesson.
///
/// What it does: renders the `Adder` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Adder() -> impl IntoView {
    let a = RwSignal::new(2.0);
    let b = RwSignal::new(3.0);
    let action = ServerAction::<Add>::new();
    let pending = action.pending();
    let value = action.value();

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px; margin-bottom: 16px;">
            <h2>"ServerAction Add"</h2>
            <input type="number" prop:value=move || a.get().to_string() on:input:target=move |event| if let Ok(value) = event.target().value().parse() { a.set(value); }/>
            <input type="number" prop:value=move || b.get().to_string() on:input:target=move |event| if let Ok(value) = event.target().value().parse() { b.set(value); }/>
            <button on:click=move |_| {
                action.dispatch(Add { a: a.get(), b: b.get() });
            }>"Add on server"</button>
            <p>{move || if pending.get() { "Pending..." } else { "Ready" }}</p>
            <p>{move || value.get().map(|result| match result {
                Ok(sum) => format!("Result: {sum}"),
                Err(error) => error.to_string(),
            })}</p>
        </section>
    }
}

/// Explains the `todo app` step in this lesson.
///
/// What it does: renders the `TodoApp` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn TodoApp() -> impl IntoView {
    let todos = Resource::new(|| (), |_| list_todos());
    let add_action = ServerAction::<AddTodo>::new();
    let toggle_action = ServerAction::<ToggleTodo>::new();
    let delete_action = ServerAction::<DeleteTodo>::new();

    Effect::new(move |_| {
        let _ = add_action.version().get();
        let _ = toggle_action.version().get();
        let _ = delete_action.version().get();
        todos.refetch();
    });

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Todo server functions"</h2>
            <ActionForm action=add_action>
                <input type="text" name="title" placeholder="New todo"/>
                <button type="submit">"Add"</button>
            </ActionForm>
            <Suspense fallback=|| view! { <p>"Loading todos..."</p> }>
                {move || todos.get().map(|result| match result {
                    Ok(items) => view! {
                        <ul style="display: grid; gap: 8px; padding-left: 0; list-style: none;">
                            <For
                                each=move || items.clone()
                                key=|todo| todo.id
                                children=move |todo| {
                                    let id = todo.id;
                                    view! {
                                        <li style="display: flex; gap: 8px; justify-content: space-between; padding: 10px; background: #f6f7fb; border-radius: 8px;">
                                            <span>{if todo.done { "[done] " } else { "" }}{todo.title}</span>
                                            <span>
                                                <button on:click=move |_| {
                                                    toggle_action.dispatch(ToggleTodo { id });
                                                }>"Toggle"</button>
                                                <button on:click=move |_| {
                                                    delete_action.dispatch(DeleteTodo { id });
                                                }>"Delete"</button>
                                            </span>
                                        </li>
                                    }
                                }
                            />
                        </ul>
                    }.into_any(),
                    Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
                })}
            </Suspense>
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
            <p>"The requested lesson route does not exist in this module."</p>
        </section>
    }
}
