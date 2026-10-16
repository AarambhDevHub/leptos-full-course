//! Module 12: State Management.
//! Topics: context, shared state, state machines, persistence, notifications.
//! Run: cd module-12-state-management && trunk serve
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use gloo_storage::{LocalStorage, Storage};
use gloo_timers::callback::Timeout;
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
enum Theme {
    Light,
    Dark,
}

#[derive(Clone)]
struct CartItem {
    id: u32,
    name: String,
    price: f64,
    qty: u32,
}

#[derive(Clone, Copy, PartialEq)]
enum WizardStep {
    Name,
    Email,
    Review,
    Done,
}

#[derive(Clone, Copy)]
enum Level {
    Info,
    Success,
    Warning,
}

#[derive(Clone)]
struct Notification {
    id: u32,
    message: String,
    level: Level,
}

/// Explains the `use notify` step in this lesson.
///
/// What it does: packages the `use notify` behavior into a reusable hook-style helper.
/// Why it matters: Hooks keep repeated reactive wiring out of the components that only need to consume the behavior.
fn use_notify() -> impl Fn(String, Level) + Clone {
    let notifications = use_context::<RwSignal<Vec<Notification>>>().expect("notification context");
    move |message, level| {
        let id = notifications.read().len() as u32 + 1;
        notifications.update(|items| items.push(Notification { id, message, level }));
        let handle = Timeout::new(3_000, move || {
            notifications.update(|items| items.retain(|item| item.id != id));
        });
        handle.forget();
    }
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `theme toggle` step in this lesson.
///
/// What it does: renders the `ThemeToggle` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ThemeToggle() -> impl IntoView {
    let theme = use_context::<RwSignal<Theme>>().expect("theme context");
    view! {
        <button on:click=move |_| {
            theme.update(|value| *value = if *value == Theme::Light { Theme::Dark } else { Theme::Light });
        }>
            {move || if theme.get() == Theme::Light { "Switch to dark" } else { "Switch to light" }}
        </button>
    }
}

/// Explains the `product card` step in this lesson.
///
/// What it does: renders the `ProductCard` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ProductCard(id: u32, #[prop(into)] name: String, price: f64) -> impl IntoView {
    let cart = use_context::<RwSignal<Vec<CartItem>>>().expect("cart context");
    let notify = use_notify();
    let name_for_click = name.clone();

    view! {
        <article class="card">
            <strong>{name}</strong>
            <p>{format!("${price:.2}")}</p>
            <button on:click=move |_| {
                cart.update(|items| {
                    if let Some(existing) = items.iter_mut().find(|item| item.id == id) {
                        existing.qty += 1;
                    } else {
                        items.push(CartItem { id, name: name_for_click.clone(), price, qty: 1 });
                    }
                });
                notify(format!("Added {name_for_click}"), Level::Success);
            }>"Add to Cart"</button>
        </article>
    }
}

/// Explains the `cart summary` step in this lesson.
///
/// What it does: renders the `CartSummary` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn CartSummary() -> impl IntoView {
    let cart = use_context::<RwSignal<Vec<CartItem>>>().expect("cart context");
    let count = Memo::new(move |_| cart.read().iter().map(|item| item.qty).sum::<u32>());
    let total = Memo::new(move |_| {
        cart.read()
            .iter()
            .map(|item| item.price * item.qty as f64)
            .sum::<f64>()
    });

    view! {
        <section class="panel">
            <h2>"Shopping Cart Context"</h2>
            <div class="grid">
                <ProductCard id=1 name="Leptos Hoodie" price=64.0/>
                <ProductCard id=2 name="Rust Mug" price=18.5/>
                <ProductCard id=3 name="WASM Sticker Pack" price=9.0/>
            </div>
            <p><strong>{count}</strong> " items, total " <strong>{move || format!("${:.2}", total.get())}</strong></p>
            <ul>
                <For
                    each=move || cart.get()
                    key=|item| item.id
                    children=|item| view! {
                        <li>{item.name} " x" {item.qty}</li>
                    }
                />
            </ul>
        </section>
    }
}

/// Explains the `wizard` step in this lesson.
///
/// What it does: renders the `Wizard` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Wizard() -> impl IntoView {
    let step = RwSignal::new(WizardStep::Name);
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let progress = move || match step.get() {
        WizardStep::Name => "0%",
        WizardStep::Email => "33%",
        WizardStep::Review => "66%",
        WizardStep::Done => "100%",
    };

    view! {
        <section class="panel">
            <h2>"Wizard State Machine"</h2>
            <div class="progress"><div style:width=progress></div></div>
            {move || match step.get() {
                WizardStep::Name => view! {
                    <div class="stack">
                        <input placeholder="Name" prop:value=name on:input:target=move |event| name.set(event.target().value())/>
                        <button on:click=move |_| step.set(WizardStep::Email)>"Next"</button>
                    </div>
                }.into_any(),
                WizardStep::Email => view! {
                    <div class="stack">
                        <input placeholder="Email" prop:value=email on:input:target=move |event| email.set(event.target().value())/>
                        <div class="row"><button on:click=move |_| step.set(WizardStep::Name)>"Back"</button><button on:click=move |_| step.set(WizardStep::Review)>"Review"</button></div>
                    </div>
                }.into_any(),
                WizardStep::Review => view! {
                    <div class="stack">
                        <p>{move || format!("{} <{}>", name.get(), email.get())}</p>
                        <div class="row"><button on:click=move |_| step.set(WizardStep::Email)>"Back"</button><button on:click=move |_| step.set(WizardStep::Done)>"Finish"</button></div>
                    </div>
                }.into_any(),
                WizardStep::Done => view! { <p class="success">"Done. State machine reached its terminal state."</p> }.into_any(),
            }}
        </section>
    }
}

/// Explains the `persistent counter` step in this lesson.
///
/// What it does: renders the `PersistentCounter` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn PersistentCounter() -> impl IntoView {
    let count = RwSignal::new(LocalStorage::get::<i32>("counter_v1").unwrap_or(0));

    Effect::new(move |_| {
        let _ = LocalStorage::set("counter_v1", count.get());
    });

    view! {
        <section class="panel">
            <h2>"LocalStorage Persistence"</h2>
            <p>"Counter: " <strong>{count}</strong></p>
            <button on:click=move |_| count.update(|value| *value += 1)>"+1"</button>
            <button on:click=move |_| count.set(0)>"Reset"</button>
        </section>
    }
}

/// Explains the `notification toast` step in this lesson.
///
/// What it does: renders the `NotificationToast` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn NotificationToast() -> impl IntoView {
    let notifications = use_context::<RwSignal<Vec<Notification>>>().expect("notification context");

    view! {
        <div class="toasts">
            <For
                each=move || notifications.get()
                key=|notification| notification.id
                children=move |notification| {
                    let class = match notification.level {
                        Level::Info => "toast info",
                        Level::Success => "toast success",
                        Level::Warning => "toast warning",
                    };
                    view! { <div class=class>{notification.message}</div> }
                }
            />
        </div>
    }
}

/// Explains the `notification demo` step in this lesson.
///
/// What it does: renders the `NotificationDemo` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn NotificationDemo() -> impl IntoView {
    let notify = use_notify();
    let notify_info = notify.clone();
    let notify_success = notify.clone();
    view! {
        <section class="panel">
            <h2>"Notification System"</h2>
            <button on:click=move |_| notify_info("Info notification".into(), Level::Info)>"Info"</button>
            <button on:click=move |_| notify_success("Saved successfully".into(), Level::Success)>"Success"</button>
            <button on:click=move |_| notify("Check your input".into(), Level::Warning)>"Warning"</button>
        </section>
    }
}

/// Explains the `app` step in this lesson.
///
/// What it does: renders the `App` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn App() -> impl IntoView {
    let theme = RwSignal::new(Theme::Light);
    provide_context(theme);
    provide_context(RwSignal::new(Vec::<CartItem>::new()));
    provide_context(RwSignal::new(Vec::<Notification>::new()));

    view! {
        <style>{STYLE}</style>
        <div class=move || if theme.get() == Theme::Light { "app light" } else { "app dark" }>
            <main>
                <h1>"Module 12 - State Management"</h1>
                <section class="panel row between">
                    <span>"Theme context is provided at the root."</span>
                    <ThemeToggle/>
                </section>
                <CartSummary/>
                <Wizard/>
                <PersistentCounter/>
                <NotificationDemo/>
                <NotificationToast/>
            </main>
        </div>
    }
}

const STYLE: &str = r#"
body { margin: 0; font-family: system-ui, sans-serif; }
.app { min-height: 100vh; padding: 1px 0; }
.light { background: #f8fafc; color: #17202a; }
.dark { background: #111827; color: #e5e7eb; }
main { max-width: 940px; margin: 0 auto; padding: 32px 16px; }
.panel { background: color-mix(in srgb, currentColor 4%, transparent); border: 1px solid #94a3b8; border-radius: 8px; padding: 18px; margin: 14px 0; }
.grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; }
.card { border: 1px solid #cbd5e1; border-radius: 8px; padding: 12px; }
.stack { display: grid; gap: 10px; max-width: 430px; }
.row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.between { justify-content: space-between; }
input, button { font: inherit; padding: 8px 10px; border-radius: 6px; border: 1px solid #94a3b8; }
button { background: #1f6feb; color: white; cursor: pointer; border-color: #1f6feb; margin: 3px; }
.progress { height: 10px; background: #cbd5e1; border-radius: 999px; overflow: hidden; margin-bottom: 12px; }
.progress div { height: 100%; background: #1f6feb; transition: width 0.2s ease; }
.success { color: #047857; font-weight: 700; }
.toasts { position: fixed; right: 16px; bottom: 16px; display: grid; gap: 8px; }
.toast { padding: 10px 12px; border-radius: 8px; color: white; box-shadow: 0 8px 30px rgba(0,0,0,.18); }
.toast.info { background: #2563eb; }
.toast.success { background: #047857; }
.toast.warning { background: #b45309; }
"#;
