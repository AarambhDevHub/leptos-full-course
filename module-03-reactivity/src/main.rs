//! Module 03: Reactivity in Depth.
//! Topics: signals, effects, memos, derived signals, RwSignal, batch updates.
//! Run: cd module-03-reactivity && trunk serve
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

/// Explains the `counter` step in this lesson.
///
/// What it does: renders the `Counter` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Counter() -> impl IntoView {
    let (count, set_count) = signal(0_i32);
    let doubled = Memo::new(move |_| count.get() * 2);

    Effect::new(move |_| {
        leptos::logging::log!("count changed to {}", count.get());
    });

    view! {
        <section style="padding: 1rem; border: 1px solid #e2e8f0; border-radius: 8px; margin: 8px 0;">
            <h3>"Counter - signal + memo + effect"</h3>
            <p>"Count: " <strong>{count}</strong></p>
            <p>"Doubled (Memo): " <strong>{doubled}</strong></p>
            <button style="padding: 6px 12px; margin: 4px;" on:click=move |_| set_count.update(|n| *n += 1)>"+"</button>
            <button style="padding: 6px 12px; margin: 4px;" on:click=move |_| set_count.update(|n| *n -= 1)>"-"</button>
            <button style="padding: 6px 12px; margin: 4px;" on:click=move |_| set_count.set(0)>"Reset"</button>
            <p style="font-size: 0.8rem; color: #6b7280;">"Open DevTools console to see effect logs."</p>
        </section>
    }
}

/// Explains the `temperature converter` step in this lesson.
///
/// What it does: renders the `TemperatureConverter` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn TemperatureConverter() -> impl IntoView {
    let (celsius, set_celsius) = signal(0.0_f64);
    let fahrenheit = move || celsius.get() * 9.0 / 5.0 + 32.0;

    view! {
        <section style="padding: 1rem; border: 1px solid #e2e8f0; border-radius: 8px; margin: 8px 0;">
            <h3>"Temperature Converter - derived signal"</h3>
            <div style="display: flex; gap: 16px; align-items: center; flex-wrap: wrap;">
                <label>
                    "Celsius: "
                    <input
                        type="number"
                        style="width: 80px; padding: 4px;"
                        prop:value=move || celsius.get().to_string()
                        on:input:target=move |event| {
                            if let Ok(value) = event.target().value().parse::<f64>() {
                                set_celsius.set(value);
                            }
                        }
                    />
                </label>
                <span>"="</span>
                <label>
                    "Fahrenheit: "
                    <input
                        type="number"
                        style="width: 80px; padding: 4px;"
                        prop:value=move || format!("{:.1}", fahrenheit())
                        on:input:target=move |event| {
                            if let Ok(value) = event.target().value().parse::<f64>() {
                                set_celsius.set((value - 32.0) * 5.0 / 9.0);
                            }
                        }
                    />
                </label>
            </div>
        </section>
    }
}

/// Explains the `word counter` step in this lesson.
///
/// What it does: renders the `WordCounter` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn WordCounter() -> impl IntoView {
    let (text, set_text) = signal(String::new());
    let word_count = Memo::new(move |_| text.get().split_whitespace().count());
    let char_count = Memo::new(move |_| text.get().chars().count());
    let unique_words = Memo::new(move |_| {
        let current = text.get();
        let mut words: Vec<&str> = current.split_whitespace().collect();
        words.sort_unstable();
        words.dedup();
        words.len()
    });

    view! {
        <section style="padding: 1rem; border: 1px solid #e2e8f0; border-radius: 8px; margin: 8px 0;">
            <h3>"Word Counter - multiple memos from one signal"</h3>
            <textarea
                style="width: 100%; height: 80px; padding: 8px; box-sizing: border-box;"
                placeholder="Type or paste text here..."
                prop:value=text
                on:input:target=move |event| set_text.set(event.target().value())
            />
            <p>"Words: " <strong>{word_count}</strong>
               " | Chars: " <strong>{char_count}</strong>
               " | Unique words: " <strong>{unique_words}</strong></p>
        </section>
    }
}

/// Explains the `color mixer` step in this lesson.
///
/// What it does: renders the `ColorMixer` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ColorMixer() -> impl IntoView {
    let (r, set_r) = signal(128_u8);
    let (g, set_g) = signal(64_u8);
    let (b, set_b) = signal(200_u8);
    let color_css = move || format!("rgb({}, {}, {})", r.get(), g.get(), b.get());
    let hex = move || format!("#{:02X}{:02X}{:02X}", r.get(), g.get(), b.get());

    let slider = |label: &'static str, value: ReadSignal<u8>, setter: WriteSignal<u8>| {
        view! {
            <label style="display: flex; align-items: center; gap: 8px; margin: 4px 0;">
                {label}
                <input
                    type="range"
                    min="0"
                    max="255"
                    prop:value=move || value.get().to_string()
                    on:input:target=move |event| {
                        if let Ok(next) = event.target().value().parse::<u8>() {
                            setter.set(next);
                        }
                    }
                    style="flex: 1;"
                />
                <code style="width: 2.5rem; text-align: right;">{move || value.get().to_string()}</code>
            </label>
        }
    };

    view! {
        <section style="padding: 1rem; border: 1px solid #e2e8f0; border-radius: 8px; margin: 8px 0;">
            <h3>"Color Mixer - derived signal from three sources"</h3>
            {slider("R", r, set_r)}
            {slider("G", g, set_g)}
            {slider("B", b, set_b)}
            <div style=move || format!(
                "margin-top: 12px; height: 60px; border-radius: 8px; background: {}; \
                 display: flex; align-items: center; justify-content: center; \
                 font-weight: 600; color: white; text-shadow: 0 1px 2px rgba(0,0,0,0.5);",
                color_css()
            )>
                {hex}
            </div>
        </section>
    }
}

/// Explains the `rw signal demo` step in this lesson.
///
/// What it does: renders the `RwSignalDemo` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn RwSignalDemo() -> impl IntoView {
    let counter = RwSignal::new(0_i32);
    let (render_count, set_render_count) = signal(0_u32);

    Effect::new(move |_| {
        let _ = counter.get();
        set_render_count.update(|n| *n += 1);
    });

    view! {
        <section style="padding: 1rem; border: 1px solid #e2e8f0; border-radius: 8px; margin: 8px 0;">
            <h3>"RwSignal + batch()"</h3>
            <p>"Counter: " <strong>{counter}</strong></p>
            <p style="font-size: 0.85rem; color: #6b7280;">"Effect ran " {render_count} " time(s)"</p>
            <button style="padding: 6px 12px; margin: 4px;" on:click=move |_| counter.update(|n| *n += 1)>"+1"</button>
            <button
                style="padding: 6px 12px; margin: 4px;"
                on:click=move |_| {
                    batch(move || {
                        counter.update(|n| *n += 10);
                        counter.update(|n| *n += 10);
                        counter.update(|n| *n += 10);
                    });
                }
            >
                "+30 (batched)"
            </button>
            <button style="padding: 6px 12px; margin: 4px;" on:click=move |_| counter.set(0)>"Reset"</button>
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
        <main style="max-width: 720px; margin: 2rem auto; padding: 0 1rem; font-family: system-ui, sans-serif;">
            <h1 style="border-bottom: 2px solid #e2e8f0; padding-bottom: 0.5rem;">"Module 03 - Reactivity"</h1>
            <Counter/>
            <TemperatureConverter/>
            <WordCounter/>
            <ColorMixer/>
            <RwSignalDemo/>
        </main>
    }
}
