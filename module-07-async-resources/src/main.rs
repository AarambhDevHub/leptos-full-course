//! Module 07: Async and Resources.
//! Topics: Resource, Suspense, Transition, refetch, resource errors.
//! Run: cd module-07-async-resources && trunk serve
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Post {
    id: u32,
    title: String,
    body: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct User {
    id: u32,
    name: String,
    email: String,
}

/// Explains the `fetch posts` step in this lesson.
///
/// What it does: loads or computes the `fetch posts` data used by an async lesson example.
/// Why it matters: Separating async work from rendering keeps resource and suspense examples easier to reason about.
async fn fetch_posts() -> Result<Vec<Post>, String> {
    Request::get("https://jsonplaceholder.typicode.com/posts?_limit=5")
        .send()
        .await
        .map_err(|error| error.to_string())?
        .json::<Vec<Post>>()
        .await
        .map_err(|error| error.to_string())
}

/// Explains the `fetch users` step in this lesson.
///
/// What it does: loads or computes the `fetch users` data used by an async lesson example.
/// Why it matters: Separating async work from rendering keeps resource and suspense examples easier to reason about.
async fn fetch_users(query: String) -> Result<Vec<User>, String> {
    let mut users = Request::get("https://jsonplaceholder.typicode.com/users")
        .send()
        .await
        .map_err(|error| error.to_string())?
        .json::<Vec<User>>()
        .await
        .map_err(|error| error.to_string())?;

    let query = query.trim().to_lowercase();
    if !query.is_empty() {
        users.retain(|user| {
            user.name.to_lowercase().contains(&query) || user.email.to_lowercase().contains(&query)
        });
    }
    Ok(users)
}

/// Explains the `fetch maybe bad` step in this lesson.
///
/// What it does: loads or computes the `fetch maybe bad` data used by an async lesson example.
/// Why it matters: Separating async work from rendering keeps resource and suspense examples easier to reason about.
async fn fetch_maybe_bad(use_bad_url: bool) -> Result<String, String> {
    let url = if use_bad_url {
        "https://jsonplaceholder.typicode.invalid/posts"
    } else {
        "https://jsonplaceholder.typicode.com/posts/1"
    };

    Request::get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .text()
        .await
        .map_err(|error| error.to_string())
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `skeleton` step in this lesson.
///
/// What it does: renders the `Skeleton` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Skeleton() -> impl IntoView {
    view! {
        <div class="skeletons">
            <div></div><div></div><div></div>
        </div>
    }
}

/// Explains the `post list` step in this lesson.
///
/// What it does: renders the `PostList` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn PostList() -> impl IntoView {
    let posts = LocalResource::new(fetch_posts);
    let refresh_count = RwSignal::new(0_u32);

    Effect::new(move |_| {
        if matches!(posts.get(), Some(Ok(_))) {
            refresh_count.update(|count| *count += 1);
        }
    });

    view! {
        <section class="panel">
            <h2>"PostList - Resource + Suspense"</h2>
            <div class="row">
                <button on:click=move |_| posts.refetch()>"Refetch posts"</button>
                <span>{move || format!("successful refreshes: {}", refresh_count.get())}</span>
            </div>
            <Suspense fallback=Skeleton>
                {move || posts.get().map(|result| match result {
                    Ok(posts) => view! {
                        <div class="cards">
                            {posts.into_iter().map(|post| view! {
                                <article>
                                    <strong>{format!("Post {}", post.id)}</strong>
                                    <h3>{post.title}</h3>
                                    <p>{post.body}</p>
                                </article>
                            }).collect_view()}
                        </div>
                    }.into_any(),
                    Err(error) => view! { <p class="error">{error}</p> }.into_any(),
                })}
            </Suspense>
        </section>
    }
}

/// Explains the `user search` step in this lesson.
///
/// What it does: renders the `UserSearch` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn UserSearch() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let users = LocalResource::new(move || fetch_users(query.get()));

    view! {
        <section class="panel">
            <h2>"UserSearch - Transition keeps old results"</h2>
            <input
                placeholder="Search by name or email"
                prop:value=query
                on:input:target=move |event| query.set(event.target().value())
            />
            <Transition fallback=|| view! { <p>"Searching..."</p> }>
                {move || users.get().map(|result| match result {
                    Ok(users) => view! {
                        <ul class="cards">
                            {users.into_iter().map(|user| view! {
                                <li>
                                    <strong>{format!("{}: {}", user.id, user.name)}</strong>
                                    <span>{format!(" - {}", user.email)}</span>
                                </li>
                            }).collect_view()}
                        </ul>
                    }.into_any(),
                    Err(error) => view! { <p class="error">{error}</p> }.into_any(),
                })}
            </Transition>
        </section>
    }
}

/// Explains the `error resource demo` step in this lesson.
///
/// What it does: renders the `ErrorResourceDemo` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ErrorResourceDemo() -> impl IntoView {
    let use_bad_url = RwSignal::new(false);
    let body = LocalResource::new(move || fetch_maybe_bad(use_bad_url.get()));

    view! {
        <section class="panel">
            <h2>"Error Resource"</h2>
            <label>
                <input
                    type="checkbox"
                    prop:checked=use_bad_url
                    on:change:target=move |event| use_bad_url.set(event.target().checked())
                />
                " Use invalid URL"
            </label>
            <Suspense fallback=|| view! { <p>"Loading endpoint..."</p> }>
                {move || body.get().map(|result| match result {
                    Ok(text) => view! { <pre>{text.chars().take(240).collect::<String>()}</pre> }.into_any(),
                    Err(error) => view! {
                        <div class="error">
                            <strong>"Fetch failed"</strong>
                            <p>{error}</p>
                        </div>
                    }.into_any(),
                })}
            </Suspense>
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
            <h1>"Module 07 - Async and Resources"</h1>
            <PostList/>
            <UserSearch/>
            <ErrorResourceDemo/>
        </main>
    }
}

const STYLE: &str = r#"
body { margin: 0; font-family: system-ui, sans-serif; background: #f8fafc; color: #17202a; }
main { max-width: 940px; margin: 0 auto; padding: 32px 16px; }
.panel { background: white; border: 1px solid #d9dee8; border-radius: 8px; padding: 18px; margin: 14px 0; }
.row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
input, button { font: inherit; padding: 8px 10px; border-radius: 6px; border: 1px solid #b9c1d0; }
button { background: #1f6feb; color: white; cursor: pointer; border-color: #1f6feb; }
.cards { display: grid; gap: 10px; padding: 0; list-style: none; }
.cards article, .cards li { padding: 12px; background: #f8fafc; border-radius: 8px; border: 1px solid #e5e7eb; }
.skeletons { display: grid; gap: 8px; }
.skeletons div { height: 64px; border-radius: 8px; background: linear-gradient(90deg, #e5e7eb, #f8fafc, #e5e7eb); }
.error { background: #fef2f2; color: #991b1b; padding: 10px; border-radius: 6px; }
pre { white-space: pre-wrap; background: #111827; color: #e5e7eb; padding: 10px; border-radius: 6px; }
"#;
