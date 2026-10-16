//! Module 13: Styling and Tailwind-like Patterns.
//! Topics: class toggles, reactive styles, CSS variables, dark mode.
//! Run: cd module-13-styling && trunk serve
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

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Tokens,
    Components,
    Layout,
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `tab bar` step in this lesson.
///
/// What it does: renders the `TabBar` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn TabBar() -> impl IntoView {
    let active = RwSignal::new(Tab::Tokens);

    view! {
        <section class="panel">
            <h2>"class:active"</h2>
            <div class="tabs">
                <button class:active=move || active.get() == Tab::Tokens on:click=move |_| active.set(Tab::Tokens)>"Tokens"</button>
                <button class:active=move || active.get() == Tab::Components on:click=move |_| active.set(Tab::Components)>"Components"</button>
                <button class:active=move || active.get() == Tab::Layout on:click=move |_| active.set(Tab::Layout)>"Layout"</button>
            </div>
            <p>{move || match active.get() {
                Tab::Tokens => "Design tokens drive a consistent UI.",
                Tab::Components => "Component classes stay small and predictable.",
                Tab::Layout => "Layout classes separate structure from decoration.",
            }}</p>
        </section>
    }
}

/// Explains the `motion card` step in this lesson.
///
/// What it does: renders the `MotionCard` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn MotionCard() -> impl IntoView {
    let visible = RwSignal::new(true);
    view! {
        <section class="panel">
            <h2>"Reactive style attributes"</h2>
            <button on:click=move |_| visible.update(|value| *value = !*value)>"Toggle card"</button>
            <div
                class="motion-card"
                style:opacity=move || if visible.get() { "1" } else { "0.35" }
                style:transform=move || if visible.get() { "translateY(0)" } else { "translateY(16px)" }
                style:color=move || if visible.get() { "var(--primary)" } else { "#64748b" }
            >
                "Opacity, transform, and color are all driven by signals."
            </div>
        </section>
    }
}

/// Explains the `gradient preview` step in this lesson.
///
/// What it does: renders the `GradientPreview` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn GradientPreview() -> impl IntoView {
    let start = RwSignal::new("#2563eb".to_string());
    let end = RwSignal::new("#f97316".to_string());

    view! {
        <section class="panel">
            <h2>"Dynamic gradient styles"</h2>
            <div class="row">
                <input type="color" prop:value=start on:input:target=move |event| start.set(event.target().value())/>
                <input type="color" prop:value=end on:input:target=move |event| end.set(event.target().value())/>
            </div>
            <div class="gradient" style:background=move || format!("linear-gradient(135deg, {}, {})", start.get(), end.get())>
                {move || format!("{} -> {}", start.get(), end.get())}
            </div>
        </section>
    }
}

/// Explains the `theme palette` step in this lesson.
///
/// What it does: renders the `ThemePalette` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ThemePalette() -> impl IntoView {
    let themes = [
        ("Ocean", "#2563eb"),
        ("Forest", "#047857"),
        ("Rose", "#be123c"),
        ("Amber", "#b45309"),
        ("Graphite", "#374151"),
    ];
    let active = RwSignal::new("#2563eb".to_string());

    Effect::new(move |_| {
        if let Some(root) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.document_element())
        {
            let _ = root.set_attribute("style", &format!("--primary: {}", active.get()));
        }
    });

    view! {
        <section class="panel">
            <h2>"CSS custom properties"</h2>
            <div class="swatches">
                {themes.into_iter().map(|(name, color)| view! {
                    <button
                        class:active=move || active.get() == color
                        style:background=color
                        on:click=move |_| active.set(color.to_string())
                        title=name
                    >
                        {name}
                    </button>
                }).collect_view()}
            </div>
            <p class="brand">"This text uses var(--primary)."</p>
        </section>
    }
}

/// Explains the `dark mode toggle` step in this lesson.
///
/// What it does: renders the `DarkModeToggle` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn DarkModeToggle() -> impl IntoView {
    let dark = RwSignal::new(false);

    Effect::new(move |_| {
        if let Some(root) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.document_element())
        {
            let class_name = if dark.get() { "dark" } else { "" };
            let _ = root.set_attribute("class", class_name);
        }
    });

    view! {
        <section class="panel">
            <h2>"Dark mode root class"</h2>
            <button on:click=move |_| dark.update(|value| *value = !*value)>
                {move || if dark.get() { "Disable dark mode" } else { "Enable dark mode" }}
            </button>
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
            <h1>"Module 13 - Styling and Tailwind"</h1>
            <TabBar/>
            <MotionCard/>
            <GradientPreview/>
            <ThemePalette/>
            <DarkModeToggle/>
        </main>
    }
}

const STYLE: &str = r#"
:root { --primary: #2563eb; }
.dark body { background: #111827; color: #e5e7eb; }
body { margin: 0; font-family: system-ui, sans-serif; background: #f8fafc; color: #17202a; transition: background .2s ease, color .2s ease; }
main { max-width: 940px; margin: 0 auto; padding: 32px 16px; }
.panel { background: color-mix(in srgb, currentColor 4%, transparent); border: 1px solid #94a3b8; border-radius: 8px; padding: 18px; margin: 14px 0; }
.row, .tabs, .swatches { display: flex; flex-wrap: wrap; gap: 10px; align-items: center; }
input, button { font: inherit; padding: 8px 10px; border-radius: 6px; border: 1px solid #94a3b8; }
button { background: #e5e7eb; color: #111827; cursor: pointer; }
button.active, .tabs button.active { background: var(--primary); color: white; border-color: var(--primary); }
.motion-card { margin-top: 14px; padding: 18px; border: 1px solid currentColor; border-radius: 8px; transition: opacity .2s ease, transform .2s ease, color .2s ease; }
.gradient { min-height: 120px; display: grid; place-items: center; margin-top: 12px; color: white; font-weight: 700; border-radius: 8px; text-shadow: 0 1px 2px rgba(0,0,0,.45); }
.swatches button { color: white; border-color: transparent; min-width: 86px; text-shadow: 0 1px 2px rgba(0,0,0,.45); }
.brand { color: var(--primary); font-weight: 800; }
"#;
