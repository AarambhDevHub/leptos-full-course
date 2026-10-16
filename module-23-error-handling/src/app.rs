//! Module 23: Error Handling.
//! Root boundaries, retryable resources, partial failure isolation, optimistic
//! rollback, and server-side client error logging.
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
use std::{error::Error, fmt};

const MODULE_TITLE: &str = "Module 23 - Error Handling";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Item {
    id: u32,
    title: String,
}

#[derive(Clone, Debug)]
struct CourseError(String);

impl fmt::Display for CourseError {
    // ============================================================================
    // LESSON 1: Full-Stack App Structure
    // ============================================================================

    /// Explains the `fmt` step in this lesson.
    ///
    /// What it does: implements the `fmt` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for CourseError {}

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
    let crashed = RwSignal::new(false);

    view! {
        <Title text=MODULE_TITLE/>
        <Meta name="description" content="Error handling patterns in the Leptos 0.8 full course"/>
        <Router>
            <main style="max-width: 1080px; margin: 0 auto; padding: 32px; font-family: system-ui, sans-serif;">
                <ErrorBoundary fallback=|errors| {
                    let message = errors
                        .get()
                        .into_iter()
                        .map(|(_, error)| error.to_string())
                        .collect::<Vec<_>>()
                        .join("\n");
                    leptos::task::spawn_local({
                        let message = message.clone();
                        async move {
                            let _ = log_client_error(message, "root-boundary".into()).await;
                        }
                    });
                    view! {
                        <section style="border: 1px solid #fecaca; border-radius: 8px; padding: 20px; background: #fef2f2;">
                            <h1>"Crash report"</h1>
                            <pre style="white-space: pre-wrap;">{message}</pre>
                            <button on:click=move |_| {
                                #[cfg(feature = "hydrate")]
                                if let Some(window) = web_sys::window() {
                                    let _ = window.location().reload();
                                }
                            }>"Reload"</button>
                        </section>
                    }
                }>
                    {move || if crashed.get() {
                        Err(CourseError("Root boundary demo crash".into()))
                    } else {
                        Ok(view! {
                            <Routes fallback=|| view! { <NotFound/> }>
                                <Route path=path!("/") view=move || view! { <ErrorPage crashed/> }/>
                            </Routes>
                        })
                    }}
                </ErrorBoundary>
            </main>
        </Router>
    }
}

// ============================================================================
// LESSON 3: Server Functions And Trusted Work
// ============================================================================

/// Explains the `unstable data` step in this lesson.
///
/// What it does: runs the `unstable data` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn unstable_data(kind: String, attempt: u32) -> Result<String, ServerFnError> {
    if kind == "always-fail" || attempt < 2 {
        return Err(ServerFnError::ServerError(format!(
            "{kind} failed on attempt {attempt}"
        )));
    }
    Ok(format!("{kind} recovered on attempt {attempt}"))
}

/// Explains the `delete item remote` step in this lesson.
///
/// What it does: runs the `delete item remote` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn delete_item_remote(id: u32) -> Result<(), ServerFnError> {
    if id == 2 {
        Err(ServerFnError::ServerError(
            "server rejected deletion for protected item 2".into(),
        ))
    } else {
        Ok(())
    }
}

/// Explains the `log client error` step in this lesson.
///
/// What it does: runs the `log client error` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn log_client_error(msg: String, stack: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    tracing::error!(message = %msg, stack = %stack, "client error captured");
    Ok(())
}

/// Explains the `error page` step in this lesson.
///
/// What it does: renders the `ErrorPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ErrorPage(crashed: RwSignal<bool>) -> impl IntoView {
    view! {
        <section style="display: grid; gap: 20px;">
            <header>
                <h1>{MODULE_TITLE}</h1>
                <p>"Errors should be isolated, logged, retryable where possible, and reversible when UI optimism fails."</p>
                <button on:click=move |_| crashed.set(true)>"Trigger root boundary"</button>
            </header>
            <RetryResource/>
            <PartialFailure/>
            <OptimisticRollback/>
        </section>
    }
}

/// Explains the `retry resource` step in this lesson.
///
/// What it does: renders the `RetryResource` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn RetryResource() -> impl IntoView {
    let attempt = RwSignal::new(0_u32);
    let data = Resource::new(
        move || attempt.get(),
        |attempt| unstable_data("profile".into(), attempt),
    );

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Retryable resource"</h2>
            <p>"Attempt: " {attempt}</p>
            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || data.get().map(|result| match result {
                    Ok(value) => view! { <p style="color: #166534;">{value}</p> }.into_any(),
                    Err(error) => view! {
                        <div style="color: #b91c1c;">
                            <p>{error.to_string()}</p>
                            <button on:click=move |_| attempt.update(|value| *value += 1)>"Retry with backoff"</button>
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </section>
    }
}

/// Explains the `partial failure` step in this lesson.
///
/// What it does: renders the `PartialFailure` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn PartialFailure() -> impl IntoView {
    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Partial failure isolation"</h2>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 12px;">
                <PartialCard title="Inventory" kind="inventory"/>
                <PartialCard title="Billing" kind="always-fail"/>
                <PartialCard title="Shipping" kind="shipping"/>
            </div>
        </section>
    }
}

/// Explains the `partial card` step in this lesson.
///
/// What it does: renders the `PartialCard` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn PartialCard(title: &'static str, kind: &'static str) -> impl IntoView {
    let data = Resource::new(|| (), move |_| unstable_data(kind.into(), 3));

    view! {
        <article style="background: #f6f7fb; border-radius: 8px; padding: 16px;">
            <h3>{title}</h3>
            <ErrorBoundary fallback=|errors| view! {
                <div style="color: #b91c1c;">
                    <strong>"Only this panel failed"</strong>
                    <ul>{move || errors.get().into_iter().map(|(_, error)| view! { <li>{error.to_string()}</li> }).collect_view()}</ul>
                </div>
            }>
                <Suspense fallback=|| view! { <p>"Loading panel..."</p> }>
                    {move || data.get().map(|result| {
                        result.map(|value| view! { <p>{value}</p> })
                    })}
                </Suspense>
            </ErrorBoundary>
        </article>
    }
}

/// Explains the `optimistic rollback` step in this lesson.
///
/// What it does: renders the `OptimisticRollback` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn OptimisticRollback() -> impl IntoView {
    let items = RwSignal::new(vec![
        Item {
            id: 1,
            title: "Delete succeeds".into(),
        },
        Item {
            id: 2,
            title: "Delete rolls back".into(),
        },
        Item {
            id: 3,
            title: "Another normal item".into(),
        },
    ]);
    let toast = RwSignal::new(String::new());

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Optimistic rollback"</h2>
            <p style="color: #b91c1c;">{toast}</p>
            <ul style="display: grid; gap: 8px; padding-left: 0; list-style: none;">
                <For
                    each=move || items.get()
                    key=|item| item.id
                    children=move |item| {
                        let id = item.id;
                        view! {
                            <li style="display: flex; justify-content: space-between; gap: 12px; padding: 10px; background: #f6f7fb; border-radius: 8px;">
                                <span>{item.title.clone()}</span>
                                <button on:click=move |_| {
                                    let original = item.clone();
                                    let position = items.read().iter().position(|candidate| candidate.id == id).unwrap_or(0);
                                    items.update(|list| list.retain(|candidate| candidate.id != id));
                                    toast.set(String::new());
                                    leptos::task::spawn_local(async move {
                                        if let Err(error) = delete_item_remote(id).await {
                                            items.update(|list| list.insert(position.min(list.len()), original));
                                            toast.set(error.to_string());
                                        }
                                    });
                                }>"Delete"</button>
                            </li>
                        }
                    }
                />
            </ul>
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
            <p>"The requested error-handling route does not exist."</p>
        </section>
    }
}
