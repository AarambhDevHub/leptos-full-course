//! Module 01: Introduction to Leptos.
//! Topics: view! macro, #[component], mount_to_body, IntoView, static vs reactive.
//! Run: cd module-01-intro && trunk serve
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use leptos::prelude::*;

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

/// Explains the `hello world` step in this lesson.
///
/// What it does: renders the `HelloWorld` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn HelloWorld() -> impl IntoView {
    view! {
        <div style="font-family: system-ui; padding: 1rem; background: #f0f9ff; border-radius: 8px; margin: 8px 0;">
            <h2 style="margin: 0 0 4px;">"Hello, Leptos!"</h2>
            <p style="margin: 0; color: #555;">"This is your first component."</p>
        </div>
    }
}

/// Explains the `welcome` step in this lesson.
///
/// What it does: renders the `Welcome` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Welcome(
    #[prop(into)] name: String,
    #[prop(optional)] subtitle: Option<String>,
) -> impl IntoView {
    view! {
        <div style="padding: 1rem; background: #fefce8; border-radius: 8px; margin: 8px 0; border-left: 4px solid #eab308;">
            <h2 style="margin: 0 0 4px;">{format!("Welcome, {name}!")}</h2>
            {subtitle.map(|s| view! { <p style="margin: 0; color: #555;">{s}</p> })}
        </div>
    }
}

/// Explains the `feature list` step in this lesson.
///
/// What it does: renders the `FeatureList` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn FeatureList(#[prop(into)] features: Vec<String>) -> impl IntoView {
    view! {
        <div style="padding: 1rem; background: #f0fdf4; border-radius: 8px; margin: 8px 0;">
            <h3 style="margin: 0 0 8px;">"What Leptos gives you:"</h3>
            <ul style="margin: 0; padding-left: 1.25rem;">
                {features
                    .iter()
                    .map(|feature| view! { <li>{feature.clone()}</li> })
                    .collect_view()}
            </ul>
        </div>
    }
}

/// Explains the `click counter` step in this lesson.
///
/// What it does: renders the `ClickCounter` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ClickCounter() -> impl IntoView {
    let (count, set_count) = signal(0_i32);
    let label = move || {
        if count.get() == 1 {
            "1 click".to_string()
        } else {
            format!("{} clicks", count.get())
        }
    };

    view! {
        <div style="padding: 1rem; background: #fdf4ff; border-radius: 8px; margin: 8px 0;">
            <h3 style="margin: 0 0 8px;">"Click Counter (your first signal)"</h3>
            <p style="margin: 0 0 8px;">{label}</p>
            <button
                style="padding: 8px 16px; background: #7c3aed; color: white; border: none; border-radius: 6px; cursor: pointer; margin-right: 8px;"
                on:click=move |_| set_count.update(|n| *n += 1)
            >
                "Click me"
            </button>
            <button
                style="padding: 8px 16px; background: #6b7280; color: white; border: none; border-radius: 6px; cursor: pointer;"
                on:click=move |_| set_count.set(0)
            >
                "Reset"
            </button>
        </div>
    }
}

/// Explains the `app` step in this lesson.
///
/// What it does: renders the `App` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn App() -> impl IntoView {
    let features = vec![
        "Fine-grained reactivity (no virtual DOM)".to_string(),
        "Full-stack SSR + hydration".to_string(),
        "Type-safe server functions".to_string(),
        "Compiles to optimized WASM".to_string(),
        "Works with the Rust ecosystem".to_string(),
    ];

    view! {
        <main style="max-width: 720px; margin: 2rem auto; padding: 0 1rem; font-family: system-ui, sans-serif;">
            <h1 style="border-bottom: 2px solid #e2e8f0; padding-bottom: 0.5rem;">
                "Module 01 - Introduction to Leptos"
            </h1>
            <HelloWorld/>
            <Welcome
                name="Aarambh Dev Hub"
                subtitle="Learning Leptos 0.8 from zero to production".to_string()
            />
            <FeatureList features=features/>
            <ClickCounter/>
        </main>
    }
}
