//! Module 21: i18n and Accessibility.
//! Topics: translations, language context, RTL, ARIA, focus-friendly UI.
//! Run: cd module-21-i18n && trunk serve
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use leptos::prelude::*;
use std::collections::HashMap;

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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Lang {
    En,
    Es,
    Hi,
}

/// Explains the `translations` step in this lesson.
///
/// What it does: performs the `translations` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
fn translations() -> HashMap<Lang, HashMap<&'static str, &'static str>> {
    HashMap::from([
        (
            Lang::En,
            HashMap::from([
                ("title", "i18n and Accessibility"),
                ("greeting", "Welcome to the course"),
                ("checkout", "Checkout"),
                ("alert", "Your settings were saved"),
                ("name", "Name"),
                ("submit", "Submit"),
                ("direction", "Reading direction"),
            ]),
        ),
        (
            Lang::Es,
            HashMap::from([
                ("title", "i18n y Accesibilidad"),
                ("greeting", "Bienvenido al curso"),
                ("checkout", "Pagar"),
                ("alert", "Tus ajustes fueron guardados"),
                ("name", "Nombre"),
                ("submit", "Enviar"),
                ("direction", "Direccion de lectura"),
            ]),
        ),
        (
            Lang::Hi,
            HashMap::from([
                ("title", "अंतरराष्ट्रीयकरण और पहुंच"),
                ("greeting", "कोर्स में आपका स्वागत है"),
                ("checkout", "चेकआउट"),
                ("alert", "आपकी सेटिंग सहेज दी गई"),
                ("name", "नाम"),
                ("submit", "जमा करें"),
                ("direction", "पढ़ने की दिशा"),
            ]),
        ),
    ])
}

/// Explains the `t` step in this lesson.
///
/// What it does: performs the `t` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
fn t(lang: Lang, key: &'static str) -> &'static str {
    translations()
        .get(&lang)
        .and_then(|messages| messages.get(key))
        .copied()
        .unwrap_or(key)
}

/// Explains the `use lang` step in this lesson.
///
/// What it does: packages the `use lang` behavior into a reusable hook-style helper.
/// Why it matters: Hooks keep repeated reactive wiring out of the components that only need to consume the behavior.
fn use_lang() -> RwSignal<Lang> {
    use_context::<RwSignal<Lang>>().expect("language context")
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `language switcher` step in this lesson.
///
/// What it does: renders the `LanguageSwitcher` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn LanguageSwitcher() -> impl IntoView {
    let lang = use_lang();
    view! {
        <section class="panel">
            <h2>"Language Switcher"</h2>
            <div class="row" role="group" aria-label="Language">
                <button class:active=move || lang.get() == Lang::En on:click=move |_| lang.set(Lang::En)>"English"</button>
                <button class:active=move || lang.get() == Lang::Es on:click=move |_| lang.set(Lang::Es)>"Español"</button>
                <button class:active=move || lang.get() == Lang::Hi on:click=move |_| lang.set(Lang::Hi)>"हिन्दी"</button>
            </div>
        </section>
    }
}

/// Explains the `greeting card` step in this lesson.
///
/// What it does: renders the `GreetingCard` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn GreetingCard() -> impl IntoView {
    let lang = use_lang();
    view! {
        <section class="panel">
            <h2>{move || t(lang.get(), "title")}</h2>
            <p>{move || t(lang.get(), "greeting")}</p>
        </section>
    }
}

/// Explains the `checkout button` step in this lesson.
///
/// What it does: renders the `CheckoutButton` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn CheckoutButton() -> impl IntoView {
    let lang = use_lang();
    view! {
        <section class="panel">
            <h2>"Button Label"</h2>
            <button aria-label=move || t(lang.get(), "checkout")>{move || t(lang.get(), "checkout")}</button>
        </section>
    }
}

/// Explains the `alert banner` step in this lesson.
///
/// What it does: renders the `AlertBanner` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn AlertBanner() -> impl IntoView {
    let lang = use_lang();
    view! {
        <section class="panel">
            <h2>"ARIA live region"</h2>
            <div class="alert" role="status" aria-live="polite">
                {move || t(lang.get(), "alert")}
            </div>
        </section>
    }
}

/// Explains the `accessible form` step in this lesson.
///
/// What it does: renders the `AccessibleForm` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn AccessibleForm() -> impl IntoView {
    let lang = use_lang();
    let name = RwSignal::new(String::new());
    view! {
        <section class="panel">
            <h2>"Accessible Form"</h2>
            <form class="stack" on:submit=move |event| event.prevent_default()>
                <label for="name-input">{move || t(lang.get(), "name")}</label>
                <input
                    id="name-input"
                    prop:value=name
                    on:input:target=move |event| name.set(event.target().value())
                />
                <button type="submit">{move || t(lang.get(), "submit")}</button>
            </form>
        </section>
    }
}

/// Explains the `direction preview` step in this lesson.
///
/// What it does: renders the `DirectionPreview` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn DirectionPreview() -> impl IntoView {
    let lang = use_lang();
    view! {
        <section class="panel">
            <h2>{move || t(lang.get(), "direction")}</h2>
            <div
                class="direction-card"
                dir=move || if lang.get() == Lang::Hi { "ltr" } else { "ltr" }
            >
                {move || format!("{}: {}", t(lang.get(), "direction"), t(lang.get(), "greeting"))}
            </div>
        </section>
    }
}

/// Explains the `keyboard menu` step in this lesson.
///
/// What it does: renders the `KeyboardMenu` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn KeyboardMenu() -> impl IntoView {
    let selected = RwSignal::new(0_usize);
    let items = ["Overview", "Lessons", "Community"];

    view! {
        <section class="panel">
            <h2>"Keyboard-friendly menu"</h2>
            <div role="menubar" class="menu">
                {items.into_iter().enumerate().map(|(index, item)| view! {
                    <button
                        role="menuitem"
                        tabindex=move || if selected.get() == index { "0" } else { "-1" }
                        class:active=move || selected.get() == index
                        on:click=move |_| selected.set(index)
                    >
                        {item}
                    </button>
                }).collect_view()}
            </div>
        </section>
    }
}

/// Explains the `app` step in this lesson.
///
/// What it does: renders the `App` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn App() -> impl IntoView {
    provide_context(RwSignal::new(Lang::En));

    view! {
        <style>{STYLE}</style>
        <main>
            <h1>"Module 21 - i18n and Accessibility"</h1>
            <LanguageSwitcher/>
            <GreetingCard/>
            <CheckoutButton/>
            <AlertBanner/>
            <AccessibleForm/>
            <DirectionPreview/>
            <KeyboardMenu/>
        </main>
    }
}

const STYLE: &str = r#"
body { margin: 0; font-family: system-ui, sans-serif; background: #f8fafc; color: #17202a; }
main { max-width: 900px; margin: 0 auto; padding: 32px 16px; }
.panel { background: white; border: 1px solid #d9dee8; border-radius: 8px; padding: 18px; margin: 14px 0; }
.row, .menu { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.stack { display: grid; gap: 8px; max-width: 360px; }
button, input { font: inherit; padding: 8px 10px; border-radius: 6px; border: 1px solid #94a3b8; }
button { background: #e5e7eb; cursor: pointer; }
button.active { background: #1f6feb; color: white; border-color: #1f6feb; }
.alert { padding: 12px; border-radius: 8px; background: #ecfdf5; color: #065f46; font-weight: 700; }
.direction-card { border: 1px dashed #94a3b8; border-radius: 8px; padding: 14px; }
"#;
