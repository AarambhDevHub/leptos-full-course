//! Module 05: Forms and Events.
//! Topics: controlled inputs, validation, preferences, multi-step forms, NodeRef.
//! Run: cd module-05-forms && trunk serve
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use leptos::html;
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

/// Explains the `live text input` step in this lesson.
///
/// What it does: renders the `LiveTextInput` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn LiveTextInput() -> impl IntoView {
    let text = RwSignal::new(String::new());
    let chars = Memo::new(move |_| text.read().chars().count());

    view! {
        <section class="panel">
            <h2>"Controlled Text Input"</h2>
            <input
                placeholder="Type a preview"
                prop:value=text
                on:input:target=move |event| text.set(event.target().value())
            />
            <p>"Characters: " <strong>{chars}</strong></p>
            <p class="preview">{move || if text.read().is_empty() { "Preview appears here".into() } else { text.get() }}</p>
        </section>
    }
}

/// Explains the `login form` step in this lesson.
///
/// What it does: renders the `LoginForm` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn LoginForm() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let email_error = RwSignal::new(String::new());
    let password_error = RwSignal::new(String::new());
    let success = RwSignal::new(String::new());

    view! {
        <section class="panel">
            <h2>"Login Validation"</h2>
            <form class="stack" on:submit=move |event| {
                event.prevent_default();
                email_error.set(String::new());
                password_error.set(String::new());
                success.set(String::new());

                let mut valid = true;
                if !email.read().contains('@') {
                    email_error.set("Email must contain @".into());
                    valid = false;
                }
                if password.read().len() < 8 {
                    password_error.set("Password must be at least 8 characters".into());
                    valid = false;
                }
                if valid {
                    success.set(format!("Logged in as {}", email.get()));
                }
            }>
                <label>"Email" <input prop:value=email on:input:target=move |event| email.set(event.target().value())/></label>
                <small class="error-text">{email_error}</small>
                <label>"Password" <input type="password" prop:value=password on:input:target=move |event| password.set(event.target().value())/></label>
                <small class="error-text">{password_error}</small>
                <button type="submit">"Log in"</button>
            </form>
            <p class="success-text">{success}</p>
        </section>
    }
}

/// Explains the `preference form` step in this lesson.
///
/// What it does: renders the `PreferenceForm` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn PreferenceForm() -> impl IntoView {
    let email = RwSignal::new(true);
    let sms = RwSignal::new(false);
    let product = RwSignal::new(true);
    let density = RwSignal::new("comfortable".to_string());
    let timezone = RwSignal::new("UTC".to_string());

    view! {
        <section class="panel">
            <h2>"Preferences"</h2>
            <div class="grid">
                <fieldset>
                    <legend>"Notifications"</legend>
                    <label><input type="checkbox" prop:checked=email on:change:target=move |event| email.set(event.target().checked())/>" Email"</label>
                    <label><input type="checkbox" prop:checked=sms on:change:target=move |event| sms.set(event.target().checked())/>" SMS"</label>
                    <label><input type="checkbox" prop:checked=product on:change:target=move |event| product.set(event.target().checked())/>" Product updates"</label>
                </fieldset>
                <fieldset>
                    <legend>"Density"</legend>
                    {["compact", "comfortable", "spacious"].into_iter().map(|choice| view! {
                        <label>
                            <input
                                type="radio"
                                name="density"
                                value=choice
                                prop:checked=move || density.get() == choice
                                on:change=move |_| density.set(choice.to_string())
                            />
                            {format!(" {choice}")}
                        </label>
                    }).collect_view()}
                </fieldset>
                <label>
                    "Timezone"
                    <select prop:value=timezone on:change:target=move |event| timezone.set(event.target().value())>
                        <option value="UTC">"UTC"</option>
                        <option value="Asia/Kolkata">"Asia/Kolkata"</option>
                        <option value="America/New_York">"America/New_York"</option>
                    </select>
                </label>
            </div>
            <div class="summary">
                {move || format!(
                    "Email: {}, SMS: {}, Product: {}, Density: {}, Timezone: {}",
                    email.get(),
                    sms.get(),
                    product.get(),
                    density.get(),
                    timezone.get()
                )}
            </div>
        </section>
    }
}

/// Explains the `multi step form` step in this lesson.
///
/// What it does: renders the `MultiStepForm` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn MultiStepForm() -> impl IntoView {
    let step = RwSignal::new(1_u8);
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm = RwSignal::new(String::new());
    let submitted = RwSignal::new(false);
    let progress = move || format!("{}%", (step.get() as u32 - 1) * 50);

    view! {
        <section class="panel">
            <h2>"Multi-step Form"</h2>
            <div class="progress"><div style:width=progress></div></div>
            {move || if step.get() == 1 {
                view! {
                    <div class="stack">
                        <input placeholder="Name" prop:value=name on:input:target=move |event| name.set(event.target().value())/>
                        <input placeholder="Email" prop:value=email on:input:target=move |event| email.set(event.target().value())/>
                        <button on:click=move |_| step.set(2)>"Next"</button>
                    </div>
                }.into_any()
            } else if step.get() == 2 {
                view! {
                    <div class="stack">
                        <input type="password" placeholder="Password" prop:value=password on:input:target=move |event| password.set(event.target().value())/>
                        <input type="password" placeholder="Confirm password" prop:value=confirm on:input:target=move |event| confirm.set(event.target().value())/>
                        <div class="row"><button on:click=move |_| step.set(1)>"Back"</button><button on:click=move |_| step.set(3)>"Review"</button></div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="stack">
                        <p>{move || format!("Name: {} | Email: {}", name.get(), email.get())}</p>
                        <p>{move || if password.get() == confirm.get() { "Passwords match" } else { "Passwords do not match" }}</p>
                        <div class="row">
                            <button on:click=move |_| step.set(2)>"Back"</button>
                            <button on:click=move |_| submitted.set(true)>"Submit"</button>
                        </div>
                        <p class="success-text">{move || submitted.get().then_some("Submitted")}</p>
                    </div>
                }.into_any()
            }}
        </section>
    }
}

/// Explains the `search node ref` step in this lesson.
///
/// What it does: renders the `SearchNodeRef` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn SearchNodeRef() -> impl IntoView {
    let search_ref: NodeRef<html::Input> = NodeRef::new();
    let submitted = RwSignal::new(String::from("Nothing searched yet"));

    view! {
        <section class="panel">
            <h2>"Uncontrolled Search with NodeRef"</h2>
            <form class="row" on:submit=move |event| {
                event.prevent_default();
                if let Some(input) = search_ref.get() {
                    submitted.set(input.value());
                }
            }>
                <input node_ref=search_ref placeholder="Read only on submit"/>
                <button type="submit">"Search"</button>
            </form>
            <p>{submitted}</p>
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
            <h1>"Module 05 - Forms and Events"</h1>
            <LiveTextInput/>
            <LoginForm/>
            <PreferenceForm/>
            <MultiStepForm/>
            <SearchNodeRef/>
        </main>
    }
}

const STYLE: &str = r#"
body { margin: 0; font-family: system-ui, sans-serif; background: #f8fafc; color: #17202a; }
main { max-width: 900px; margin: 0 auto; padding: 32px 16px; }
.panel { background: white; border: 1px solid #d9dee8; border-radius: 8px; padding: 18px; margin: 14px 0; }
.stack { display: grid; gap: 10px; max-width: 430px; }
.row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px; }
input, button, select, textarea { font: inherit; padding: 8px 10px; border-radius: 6px; border: 1px solid #b9c1d0; }
button { background: #1f6feb; color: white; cursor: pointer; border-color: #1f6feb; }
label { display: grid; gap: 4px; margin: 4px 0; }
fieldset label { display: block; }
.preview, .summary { background: #f8fafc; border-radius: 6px; padding: 10px; }
.error-text { color: #b91c1c; min-height: 1rem; }
.success-text { color: #047857; font-weight: 600; }
.progress { height: 10px; background: #e5e7eb; border-radius: 999px; overflow: hidden; margin-bottom: 12px; }
.progress div { height: 100%; background: #1f6feb; transition: width 0.2s ease; }
"#;
