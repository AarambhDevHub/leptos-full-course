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

/// The longest piece of a client error report that is written to the log.
pub const MAX_LOG_FIELD: usize = 500;

/// The kinds of data the demo server function knows about.
#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
const KNOWN_KINDS: [&str; 4] = ["profile", "inventory", "shipping", "always-fail"];

/// Explains the `clean log field` step in this lesson.
///
/// What it does: makes text from the browser safe to write into a log line: line breaks and other control characters are shown as visible escapes, and long text is cut.
/// Why it matters: The error report endpoint is public. Without this, one request could add a fake log line (`real message` then a new line that looks like `INFO admin logged in`), or write two megabytes into the log.
pub fn clean_log_field(raw: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (count, ch) in raw.chars().enumerate() {
        if count >= max_chars {
            out.push_str("...");
            break;
        }
        match ch {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push(' '),
            c if c.is_control() => out.push('?'),
            c => out.push(c),
        }
    }
    out
}

/// Explains the `log budget` step in this lesson.
///
/// What it does: counts how many error reports were accepted in the current minute, and says no after thirty.
/// Why it matters: One broken page in a loop, or one bad visitor, could otherwise write thousands of lines per second and fill the disk.
#[derive(Debug)]
pub struct LogBudget {
    window_start: Option<u64>,
    used: u32,
}

impl LogBudget {
    pub const WINDOW_SECS: u64 = 60;
    pub const MAX_PER_WINDOW: u32 = 30;

    pub const fn new() -> Self {
        Self {
            window_start: None,
            used: 0,
        }
    }

    /// Asks for one more report at time `now` (seconds). Returns `false` when the budget is spent.
    pub fn allow(&mut self, now: u64) -> bool {
        let expired = match self.window_start {
            Some(start) => now.saturating_sub(start) >= Self::WINDOW_SECS,
            None => true,
        };
        if expired {
            self.window_start = Some(now);
            self.used = 0;
        }
        if self.used < Self::MAX_PER_WINDOW {
            self.used += 1;
            true
        } else {
            false
        }
    }
}

impl Default for LogBudget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "ssr")]
static LOG_BUDGET: std::sync::Mutex<LogBudget> = std::sync::Mutex::new(LogBudget::new());

/// Explains the `restore in order` step in this lesson.
///
/// What it does: puts a deleted item back where it used to be, judged by the original order of the list and not by an old index.
/// Why it matters: The old rollback inserted the item at the index it had when it was clicked. If another item was deleted in the meantime, that index was wrong and the item came back in the wrong place.
fn restore_in_order(list: &mut Vec<Item>, item: Item, original_order: &[u32]) {
    let rank = |id: u32| {
        original_order
            .iter()
            .position(|candidate| *candidate == id)
            .unwrap_or(usize::MAX)
    };
    let item_rank = rank(item.id);
    let index = list
        .iter()
        .position(|existing| rank(existing.id) > item_rank)
        .unwrap_or(list.len());
    list.insert(index, item);
}

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
                    // Report once per change of the errors. An effect does not run during server
                    // rendering, and it does not repeat when the fallback is only re-drawn.
                    Effect::new({
                        let errors = errors.clone();
                        move |_| {
                            let report = join_errors(&errors.get());
                            leptos::task::spawn_local(async move {
                                let _ = log_client_error(report, "root-boundary".into()).await;
                            });
                        }
                    });
                    let message = join_errors(&errors.get_untracked());
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

/// Explains the `join errors` step in this lesson.
///
/// What it does: turns the errors caught by a boundary into one text, one error per line.
/// Why it matters: The crash report and the report sent to the server must show the same text.
fn join_errors(errors: &Errors) -> String {
    errors
        .clone()
        .into_iter()
        .map(|(_, error)| error.to_string())
        .collect::<Vec<_>>()
        .join("\n")
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
    // A public door: only the kinds this demo knows are accepted, so a caller cannot make the server echo a huge text.
    if !KNOWN_KINDS.contains(&kind.as_str()) {
        return Err(ServerFnError::ServerError("unknown kind".into()));
    }
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
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs())
            .unwrap_or(0);
        let allowed = LOG_BUDGET
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .allow(now);
        if !allowed {
            return Err(ServerFnError::ServerError("too many error reports".into()));
        }
        tracing::error!(
            message = %clean_log_field(&msg, MAX_LOG_FIELD),
            stack = %clean_log_field(&stack, MAX_LOG_FIELD),
            "client error captured"
        );
    }
    #[cfg(not(feature = "ssr"))]
    let _ = (&msg, &stack);
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
                            <button on:click=move |_| attempt.update(|value| *value += 1)>"Retry"</button>
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
    // The order the list had at the start, used to put a failed delete back in the right place.
    let original_order = StoredValue::new(
        items
            .get_untracked()
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>(),
    );

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Optimistic rollback"</h2>
            <p role="alert" style="color: #b91c1c;">{toast}</p>
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
                                    items.update(|list| list.retain(|candidate| candidate.id != id));
                                    toast.set(String::new());
                                    leptos::task::spawn_local(async move {
                                        if let Err(error) = delete_item_remote(id).await {
                                            items.update(|list| restore_in_order(list, original, &original_order.get_value()));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u32) -> Item {
        Item {
            id,
            title: format!("item {id}"),
        }
    }

    fn ids(list: &[Item]) -> Vec<u32> {
        list.iter().map(|item| item.id).collect()
    }

    #[test]
    fn log_fields_cannot_forge_a_new_line() {
        let cleaned = clean_log_field("real\n2026-01-01 INFO fake: admin logged in\r\n", 500);
        assert!(!cleaned.contains('\n') && !cleaned.contains('\r'));
        assert!(cleaned.starts_with("real\\n2026-01-01"));
        assert_eq!(clean_log_field("a\u{1b}[31mb", 50), "a?[31mb");
    }

    #[test]
    fn log_fields_are_cut() {
        let long = "x".repeat(2_000_000);
        let cleaned = clean_log_field(&long, MAX_LOG_FIELD);
        assert_eq!(cleaned.len(), MAX_LOG_FIELD + 3);
        assert!(cleaned.ends_with("..."));
        // Hindi counts as characters, not bytes.
        assert_eq!(clean_log_field("नमस्ते", 100), "नमस्ते");
    }

    #[test]
    fn the_log_budget_is_spent_then_renewed() {
        let mut budget = LogBudget::new();
        for _ in 0..LogBudget::MAX_PER_WINDOW {
            assert!(budget.allow(1_000));
        }
        assert!(!budget.allow(1_000));
        assert!(!budget.allow(1_059));
        assert!(budget.allow(1_060));
    }

    #[test]
    fn a_failed_delete_returns_to_its_place() {
        let order = [1, 2, 3];
        // Delete 2 (it will fail later), then delete 1 (it succeeds): list is [3].
        let mut list = vec![item(3)];
        restore_in_order(&mut list, item(2), &order);
        assert_eq!(ids(&list), vec![2, 3]);
        // The old index based code gave [3, 2] here.
        let mut list = vec![item(1), item(2)];
        restore_in_order(&mut list, item(3), &order);
        assert_eq!(ids(&list), vec![1, 2, 3]);
        let mut list = vec![item(2), item(3)];
        restore_in_order(&mut list, item(1), &order);
        assert_eq!(ids(&list), vec![1, 2, 3]);
    }

    #[test]
    fn only_known_kinds_are_accepted() {
        assert!(KNOWN_KINDS.contains(&"profile"));
        assert!(!KNOWN_KINDS.contains(&"anything-else"));
    }
}
