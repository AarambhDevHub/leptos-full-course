//! Module 09: SSR and Hydration.
//! This module demonstrates server-rendered routes, route params, metadata,
//! and server-side 404 status handling.
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Title};
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::hooks::use_params_map;
use leptos_router::path;
use leptos_router::SsrMode;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use axum::http::StatusCode;

const MODULE_TITLE: &str = "Module 09 - SSR and Hydration";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Article {
    id: u32,
    slug: String,
    title: String,
    excerpt: String,
    body: String,
}

// ============================================================================
// LESSON 1: Full-Stack App Structure
// ============================================================================

/// Explains the `articles` step in this lesson.
///
/// What it does: performs the `articles` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
#[allow(dead_code)]
fn articles() -> Vec<Article> {
    vec![
        Article { id: 1, slug: "server-rendering".into(), title: "Server Rendering".into(), excerpt: "Send useful HTML before WASM loads.".into(), body: "Leptos renders the route on the server, streams HTML, then hydrates interactive islands in the browser.".into() },
        Article { id: 2, slug: "hydration".into(), title: "Hydration".into(), excerpt: "Attach Rust behavior to existing markup.".into(), body: "Hydration reuses server HTML and wires event listeners and reactive subscriptions after the WASM bundle loads.".into() },
        Article { id: 3, slug: "metadata".into(), title: "Metadata".into(), excerpt: "SSR can populate the document head.".into(), body: "Title and Meta components write into the HTML head during server rendering, improving SEO and previews.".into() },
    ]
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
        // Default title only. Each page sets its own description: `Meta` tags are not
        // de-duplicated, so a second description here would end up in the head twice.
        <Title text=MODULE_TITLE/>
        <Router>
            <main style="max-width: 860px; margin: 0 auto; padding: 32px; font-family: system-ui, sans-serif;">
                <Routes fallback=|| view! { <NotFoundPage/> }>
                    <Route path=path!("/") view=BlogList/>
                    // Async SSR waits for the article before sending HTML, so the article's own title and
                    // description are in the <head> and the body is complete in the first response.
                    // (With default streaming, tags set inside a Suspense arrive too late for the head.)
                    <Route path=path!("/blog/:slug") view=ArticlePage ssr=SsrMode::Async/>
                </Routes>
            </main>
        </Router>
    }
}

// ============================================================================
// LESSON 3: Server Functions And Trusted Work
// ============================================================================

/// Explains the `get articles` step in this lesson.
///
/// What it does: runs the `get articles` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn get_articles() -> Result<Vec<Article>, ServerFnError> {
    Ok(articles())
}

/// Explains the `get article` step in this lesson.
///
/// What it does: runs the `get article` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn get_article(slug: String) -> Result<Article, ServerFnError> {
    articles()
        .into_iter()
        .find(|article| article.slug == slug)
        .ok_or_else(|| ServerFnError::ServerError("article not found".into()))
}

/// Explains the `blog list` step in this lesson.
///
/// What it does: renders the `BlogList` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn BlogList() -> impl IntoView {
    let articles = Resource::new(|| (), |_| get_articles());

    view! {
        <Title text="SSR Blog"/>
        <Meta name="description" content="Article list rendered on the server"/>
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h1>{MODULE_TITLE}</h1>
            <Suspense fallback=|| view! { <p>"Loading article skeleton..."</p> }>
                {move || articles.get().map(|result| match result {
                    Ok(items) => view! {
                        <div style="display: grid; gap: 12px;">
                            {items.into_iter().map(|article| view! {
                                <article style="padding: 14px; background: #f6f7fb; border-radius: 8px;">
                                    <h2><A href=format!("/blog/{}", article.slug)>{article.title}</A></h2>
                                    <p>{article.excerpt}</p>
                                </article>
                            }).collect_view()}
                        </div>
                    }.into_any(),
                    Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
                })}
            </Suspense>
        </section>
    }
}

/// Explains the `article page` step in this lesson.
///
/// What it does: renders the `ArticlePage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ArticlePage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();
    let article = Resource::new(slug, get_article);

    view! {
        <Suspense fallback=|| view! { <p>"Loading article..."</p> }>
            {move || article.get().map(|result| match result {
                Ok(article) => view! {
                    <article style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
                        <Title text=article.title.clone()/>
                        <Meta name="description" content=article.excerpt.clone()/>
                        <A href="/">"Back to articles"</A>
                        <h1>{article.title}</h1>
                        <p style="color: #5f6b7a;">{article.excerpt}</p>
                        <p>{article.body}</p>
                    </article>
                }.into_any(),
                Err(_) => view! { <NotFoundPage/> }.into_any(),
            })}
        </Suspense>
    }
}

/// Explains the `not found page` step in this lesson.
///
/// What it does: renders the `NotFoundPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn NotFoundPage() -> impl IntoView {
    #[cfg(feature = "ssr")]
    if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
        response.set_status(StatusCode::NOT_FOUND);
    }

    view! {
        <section style="border: 1px solid #fecaca; background: #fff7f7; border-radius: 8px; padding: 20px;">
            <Title text="404 - Article not found"/>
            <h1>"404"</h1>
            <p>"This route or article does not exist."</p>
            <A href="/">"Return to blog"</A>
        </section>
    }
}
