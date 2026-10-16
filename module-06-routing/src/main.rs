//! Module 06: Routing.
//! Topics: Router, Routes, links, params, nested routes, programmatic navigation.
//! Run: cd module-06-routing && trunk serve
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use leptos::prelude::*;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes, A};
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::path;

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

#[derive(Clone)]
struct User {
    id: u32,
    name: &'static str,
    role: &'static str,
}

/// Explains the `users` step in this lesson.
///
/// What it does: performs the `users` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
fn users() -> Vec<User> {
    vec![
        User {
            id: 1,
            name: "Ada Lovelace",
            role: "Compiler pioneer",
        },
        User {
            id: 2,
            name: "Grace Hopper",
            role: "Debugger",
        },
        User {
            id: 3,
            name: "Barbara Liskov",
            role: "Type systems",
        },
        User {
            id: 4,
            name: "Graydon Hoare",
            role: "Rust creator",
        },
        User {
            id: 5,
            name: "Carol Nichols",
            role: "Rust educator",
        },
    ]
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `nav bar` step in this lesson.
///
/// What it does: renders the `NavBar` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn NavBar() -> impl IntoView {
    view! {
        <nav>
            <A href="/" exact=true>"Home"</A>
            <A href="/about">"About"</A>
            <A href="/users">"Users"</A>
            <A href="/dashboard">"Dashboard"</A>
            <A href="/dashboard/settings">"Settings"</A>
            <A href="/contact">"Contact"</A>
        </nav>
    }
}

/// Explains the `home page` step in this lesson.
///
/// What it does: renders the `HomePage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn HomePage() -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <section class="panel">
            <h2>"Home"</h2>
            <p>"This page demonstrates client-side navigation with the Leptos 0.8 router."</p>
            <button on:click=move |_| navigate("/dashboard", Default::default())>
                "Go to Dashboard"
            </button>
        </section>
    }
}

/// Explains the `about page` step in this lesson.
///
/// What it does: renders the `AboutPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn AboutPage() -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"About"</h2>
            <p>"Routes are declared as components and resolved without a page reload."</p>
        </section>
    }
}

/// Explains the `contact page` step in this lesson.
///
/// What it does: renders the `ContactPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ContactPage() -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"Contact"</h2>
            <p>"Static routes use fixed paths like /about and /contact."</p>
        </section>
    }
}

/// Explains the `user list page` step in this lesson.
///
/// What it does: renders the `UserListPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn UserListPage() -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"Users"</h2>
            <ul class="cards">
                {users().into_iter().map(|user| view! {
                    <li>
                        <A href=format!("/users/{}", user.id)>
                            <strong>{user.name}</strong>
                            <span>{format!(" - {}", user.role)}</span>
                        </A>
                    </li>
                }).collect_view()}
            </ul>
        </section>
    }
}

/// Explains the `user detail page` step in this lesson.
///
/// What it does: renders the `UserDetailPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn UserDetailPage() -> impl IntoView {
    let params = use_params_map();
    let user_id = move || params.read().get("id").unwrap_or_default();

    view! {
        <section class="panel">
            <h2>"User Detail"</h2>
            <p>"User ID: " <strong>{user_id}</strong></p>
            <A href="/users">"Back to users"</A>
        </section>
    }
}

/// Explains the `dashboard layout` step in this lesson.
///
/// What it does: renders the `DashboardLayout` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn DashboardLayout() -> impl IntoView {
    view! {
        <section class="panel dashboard">
            <aside>
                <h2>"Dashboard"</h2>
                <A href="/dashboard" exact=true>"Overview"</A>
                <A href="/dashboard/settings">"Settings"</A>
            </aside>
            <div class="dashboard-body">
                <Outlet/>
            </div>
        </section>
    }
}

/// Explains the `dashboard home` step in this lesson.
///
/// What it does: renders the `DashboardHome` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn DashboardHome() -> impl IntoView {
    view! {
        <div>
            <h3>"Dashboard Home"</h3>
            <p>"Nested index route rendered inside the dashboard layout."</p>
        </div>
    }
}

/// Explains the `dashboard settings` step in this lesson.
///
/// What it does: renders the `DashboardSettings` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn DashboardSettings() -> impl IntoView {
    let email_reports = RwSignal::new(true);
    view! {
        <div>
            <h3>"Dashboard Settings"</h3>
            <label>
                <input
                    type="checkbox"
                    prop:checked=email_reports
                    on:change:target=move |event| email_reports.set(event.target().checked())
                />
                " Email weekly routing reports"
            </label>
        </div>
    }
}

/// Explains the `not found page` step in this lesson.
///
/// What it does: renders the `NotFoundPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn NotFoundPage() -> impl IntoView {
    view! {
        <section class="panel error">
            <h2>"404"</h2>
            <p>"No route matched this URL."</p>
            <A href="/">"Return home"</A>
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
        <Router>
            <main>
                <h1>"Module 06 - Routing"</h1>
                <NavBar/>
                <Routes fallback=|| view! { <NotFoundPage/> }>
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/about") view=AboutPage/>
                    <Route path=path!("/contact") view=ContactPage/>
                    <Route path=path!("/users") view=UserListPage/>
                    <Route path=path!("/users/:id") view=UserDetailPage/>
                    <ParentRoute path=path!("/dashboard") view=DashboardLayout>
                        <Route path=path!("") view=DashboardHome/>
                        <Route path=path!("settings") view=DashboardSettings/>
                    </ParentRoute>
                </Routes>
            </main>
        </Router>
    }
}

const STYLE: &str = r#"
body { margin: 0; font-family: system-ui, sans-serif; background: #f8fafc; color: #17202a; }
main { max-width: 940px; margin: 0 auto; padding: 32px 16px; }
nav { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 16px; }
a { color: #1f6feb; text-decoration: none; }
nav a, aside a { padding: 8px 10px; border-radius: 6px; background: #e5e7eb; color: #111827; }
a[aria-current="page"] { background: #1f6feb; color: white; }
.panel { background: white; border: 1px solid #d9dee8; border-radius: 8px; padding: 18px; margin: 14px 0; }
button, input { font: inherit; padding: 8px 10px; border-radius: 6px; border: 1px solid #b9c1d0; }
button { background: #1f6feb; color: white; cursor: pointer; border-color: #1f6feb; }
.cards { display: grid; gap: 8px; padding: 0; list-style: none; }
.cards li a { display: block; padding: 12px; border: 1px solid #e5e7eb; border-radius: 6px; }
.dashboard { display: grid; grid-template-columns: minmax(160px, 220px) 1fr; gap: 18px; }
aside { display: grid; align-content: start; gap: 8px; }
.dashboard-body { border-left: 1px solid #e5e7eb; padding-left: 18px; }
.error { border-color: #fecaca; background: #fff7f7; }
"#;
