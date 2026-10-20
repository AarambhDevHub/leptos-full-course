//! Module 04: Control Flow.
//! Topics: Show, For, inline conditionals, Option rendering, ErrorBoundary.
//! Run: cd module-04-control-flow && trunk serve
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use leptos::prelude::*;
use std::{error::Error, fmt};

// ============================================================================
// LESSON 1: Bootstrapping The Browser Module
// ============================================================================

/// Explains the `main` step in this lesson.
///
/// What it does: starts the runnable entry point for this module.
/// Why it matters: It makes the module runnable and shows learners where execution really begins.
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `login gate` step in this lesson.
///
/// What it does: renders the `LoginGate` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn LoginGate() -> impl IntoView {
    let logged_in = RwSignal::new(false);
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());

    view! {
        <section class="panel">
            <h2>"LoginGate - <Show/>"</h2>
            <Show
                when=move || logged_in.get()
                fallback=move || view! {
                    <form class="stack" on:submit=move |event| {
                        event.prevent_default();
                        if email.read().contains('@') && password.read().len() >= 4 {
                            logged_in.set(true);
                        }
                    }>
                        <input
                            type="email"
                            placeholder="email"
                            prop:value=email
                            on:input:target=move |event| email.set(event.target().value())
                        />
                        <input
                            type="password"
                            placeholder="password"
                            prop:value=password
                            on:input:target=move |event| password.set(event.target().value())
                        />
                        <button type="submit">"Log in"</button>
                    </form>
                }
            >
                <div class="success">
                    <strong>"Welcome back."</strong>
                    <p>{move || format!("Signed in as {}", email.get())}</p>
                    <button on:click=move |_| logged_in.set(false)>"Log out"</button>
                </div>
            </Show>
        </section>
    }
}

#[derive(Clone)]
struct Todo {
    id: u64,
    title: String,
}

/// Explains the `todo list` step in this lesson.
///
/// What it does: renders the `TodoList` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn TodoList() -> impl IntoView {
    let draft = RwSignal::new(String::new());
    let todos = RwSignal::new(vec![
        Todo {
            id: 1,
            title: "Render a keyed list".into(),
        },
        Todo {
            id: 2,
            title: "Delete one row without rebuilding the rest".into(),
        },
    ]);

    // Keys must be unique and stable, so ids come from a counter that only ever goes up.
    // (Using `items.len() + 1` would reuse an id after a delete, and `<For/>` would skip the new row.)
    let next_id = RwSignal::new(3_u64);

    let add_todo = move || {
        let title = draft.read().trim().to_string();
        if title.is_empty() {
            return;
        }
        let id = next_id.get();
        next_id.set(id + 1);
        todos.update(|items| items.push(Todo { id, title }));
        draft.set(String::new());
    };

    view! {
        <section class="panel">
            <h2>"TodoList - keyed <For/>"</h2>
            <form class="row" on:submit=move |event| {
                event.prevent_default();
                add_todo();
            }>
                <input
                    placeholder="New todo"
                    prop:value=draft
                    on:input:target=move |event| draft.set(event.target().value())
                />
                <button type="submit">"Add"</button>
            </form>
            <ul class="list">
                <For
                    each=move || todos.get()
                    key=|todo| todo.id
                    children=move |todo| {
                        let id = todo.id;
                        view! {
                            <li class="row between">
                                <span><code>{format!("#{}", todo.id)}</code> " " {todo.title}</span>
                                <button class="danger" on:click=move |_| {
                                    todos.update(|items| items.retain(|item| item.id != id));
                                }>"Delete"</button>
                            </li>
                        }
                    }
                />
            </ul>
        </section>
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Overview,
    Code,
    Docs,
}

/// Explains the `tab panel` step in this lesson.
///
/// What it does: renders the `TabPanel` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn TabPanel() -> impl IntoView {
    let tab = RwSignal::new(Tab::Overview);
    let button_class = move |current| {
        if tab.get() == current {
            "tab active"
        } else {
            "tab"
        }
    };

    view! {
        <section class="panel">
            <h2>"TabPanel - inline conditionals"</h2>
            <div class="tabs">
                <button class=move || button_class(Tab::Overview) on:click=move |_| tab.set(Tab::Overview)>"Overview"</button>
                <button class=move || button_class(Tab::Code) on:click=move |_| tab.set(Tab::Code)>"Code"</button>
                <button class=move || button_class(Tab::Docs) on:click=move |_| tab.set(Tab::Docs)>"Docs"</button>
            </div>
            {move || match tab.get() {
                Tab::Overview => view! { <p>"Control flow in Leptos composes normal Rust with reactive views."</p> }.into_any(),
                Tab::Code => view! { <pre>"move || if signal.get() { view! { ... } } else { view! { ... } }"</pre> }.into_any(),
                Tab::Docs => view! { <p>"Use <Show/> for simple branches and <For/> for keyed dynamic collections."</p> }.into_any(),
            }}
        </section>
    }
}

/// Explains the `user card` step in this lesson.
///
/// What it does: renders the `UserCard` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn UserCard(
    #[prop(into)] name: String,
    #[prop(optional)] avatar_url: Option<String>,
) -> impl IntoView {
    view! {
        <article class="user-card">
            {avatar_url
                .map(|url| view! { <img class="avatar" src=url alt=name.clone()/> }.into_any())
                .unwrap_or_else(|| view! { <div class="avatar placeholder">{name.chars().next().unwrap_or('?')}</div> }.into_any())}
            <div>
                <strong>{name}</strong>
                <p>"Option rendering keeps missing media explicit."</p>
            </div>
        </article>
    }
}

#[derive(Debug, Clone)]
struct DivideError(String);

impl fmt::Display for DivideError {
    /// Explains the `fmt` step in this lesson.
    ///
    /// What it does: implements the `fmt` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for DivideError {}

/// Explains the `safe divider` step in this lesson.
///
/// What it does: renders the `SafeDivider` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn SafeDivider() -> impl IntoView {
    let a = RwSignal::new(10.0_f64);
    let b = RwSignal::new(2.0_f64);
    let result = RwSignal::new(Ok::<f64, DivideError>(5.0));

    view! {
        <section class="panel">
            <h2>"SafeDivider - ErrorBoundary"</h2>
            <div class="row">
                <input
                    type="number"
                    prop:value=move || a.get().to_string()
                    on:input:target=move |event| {
                        if let Ok(value) = event.target().value().parse() {
                            a.set(value);
                        }
                    }
                />
                <span>"/"</span>
                <input
                    type="number"
                    prop:value=move || b.get().to_string()
                    on:input:target=move |event| {
                        if let Ok(value) = event.target().value().parse() {
                            b.set(value);
                        }
                    }
                />
                <button on:click=move |_| {
                    if b.get() == 0.0 {
                        result.set(Err(DivideError("division by zero".into())));
                    } else {
                        result.set(Ok(a.get() / b.get()));
                    }
                }>"Divide"</button>
            </div>
            <ErrorBoundary fallback=|errors| view! {
                <div class="error">
                    <strong>"Caught error"</strong>
                    <ul>{move || errors.get().into_iter().map(|(_, error)| view! { <li>{error.to_string()}</li> }).collect_view()}</ul>
                </div>
            }>
                {move || result.get().map(|value| view! { <p class="success">"Result: " {format!("{value:.2}")}</p> })}
            </ErrorBoundary>
        </section>
    }
}

/// Explains the `app` step in this lesson.
///
/// What it does: renders the `App` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn App() -> impl IntoView {
    view! {
        <style>{STYLE}</style>
        <main>
            <h1>"Module 04 - Control Flow"</h1>
            <LoginGate/>
            <TodoList/>
            <TabPanel/>
            <section class="panel">
                <h2>"UserCard - Option rendering"</h2>
                <UserCard name="Aarambh Dev Hub"/>
                <UserCard name="Leptos Learner" avatar_url="https://placehold.co/96x96".to_string()/>
            </section>
            <SafeDivider/>
        </main>
    }
}

const STYLE: &str = r#"
body { margin: 0; font-family: system-ui, sans-serif; background: #f8fafc; color: #17202a; }
main { max-width: 860px; margin: 0 auto; padding: 32px 16px; }
.panel { background: white; border: 1px solid #d9dee8; border-radius: 8px; padding: 18px; margin: 14px 0; }
.stack { display: grid; gap: 10px; max-width: 360px; }
.row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.between { justify-content: space-between; }
input, button { font: inherit; padding: 8px 10px; border-radius: 6px; border: 1px solid #b9c1d0; }
button { background: #1f6feb; color: white; cursor: pointer; border-color: #1f6feb; }
button.danger { background: #dc2626; border-color: #dc2626; }
.list { padding: 0; list-style: none; display: grid; gap: 8px; }
.list li { padding: 8px; background: #f8fafc; border-radius: 6px; }
.tabs { display: flex; gap: 8px; margin-bottom: 10px; }
.tab { background: #e5e7eb; color: #111827; border-color: #d1d5db; }
.tab.active { background: #1f6feb; color: white; }
.user-card { display: flex; gap: 12px; align-items: center; padding: 10px; border: 1px solid #e5e7eb; border-radius: 8px; margin: 8px 0; }
.avatar { width: 48px; height: 48px; border-radius: 50%; object-fit: cover; }
.placeholder { display: grid; place-items: center; background: #4f46e5; color: white; font-weight: 700; }
.success { background: #ecfdf5; color: #065f46; padding: 10px; border-radius: 6px; }
.error { background: #fef2f2; color: #991b1b; padding: 10px; border-radius: 6px; }
"#;
