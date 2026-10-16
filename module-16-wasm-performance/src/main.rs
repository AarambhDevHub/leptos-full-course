//! Module 16: WASM Performance.
//! Topics: batching, memoization, untracked reads, lazy lists, binary size.
//! Run: cd module-16-wasm-performance && trunk serve
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use leptos::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static MEMO_FIB_CALLS: AtomicUsize = AtomicUsize::new(0);
static RAW_FIB_CALLS: AtomicUsize = AtomicUsize::new(0);

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

/// Explains the `now` step in this lesson.
///
/// What it does: performs the `now` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
fn now() -> f64 {
    web_sys::window()
        .and_then(|window| window.performance())
        .map(|performance| performance.now())
        .unwrap_or_default()
}

/// Explains the `fib` step in this lesson.
///
/// What it does: performs the `fib` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
fn fib(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        n => fib(n - 1) + fib(n - 2),
    }
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `batching benchmark` step in this lesson.
///
/// What it does: renders the `BatchingBenchmark` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn BatchingBenchmark() -> impl IntoView {
    let a = RwSignal::new(0_i32);
    let b = RwSignal::new(0_i32);
    let c = RwSignal::new(0_i32);
    let unbatched = RwSignal::new(0.0_f64);
    let batched = RwSignal::new(0.0_f64);

    view! {
        <section class="panel">
            <h2>"Signal batching benchmark"</h2>
            <p>{move || format!("a={} b={} c={}", a.get(), b.get(), c.get())}</p>
            <button on:click=move |_| {
                let start = now();
                a.update(|value| *value += 1);
                b.update(|value| *value += 1);
                c.update(|value| *value += 1);
                unbatched.set((now() - start).max(0.1));
            }>"Unbatched update"</button>
            <button on:click=move |_| {
                let start = now();
                batch(move || {
                    a.update(|value| *value += 1);
                    b.update(|value| *value += 1);
                    c.update(|value| *value += 1);
                });
                batched.set((now() - start).max(0.1));
            }>"Batched update"</button>
            <Bar label="unbatched" value=unbatched/>
            <Bar label="batched" value=batched/>
        </section>
    }
}

/// Explains the `bar` step in this lesson.
///
/// What it does: renders the `Bar` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Bar(label: &'static str, value: RwSignal<f64>) -> impl IntoView {
    view! {
        <div class="bar-row">
            <span>{label}</span>
            <div class="bar"><div style:width=move || format!("{}px", value.get() * 80.0)></div></div>
            <code>{move || format!("{:.3}ms", value.get())}</code>
        </div>
    }
}

/// Explains the `memo benchmark` step in this lesson.
///
/// What it does: renders the `MemoBenchmark` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn MemoBenchmark() -> impl IntoView {
    let input = RwSignal::new(30_u32);
    let memo_fib = Memo::new(move |_| {
        MEMO_FIB_CALLS.fetch_add(1, Ordering::Relaxed);
        fib(input.get())
    });
    let raw_fib = move || {
        RAW_FIB_CALLS.fetch_add(1, Ordering::Relaxed);
        fib(input.get())
    };

    view! {
        <section class="panel">
            <h2>"Memo vs derived signal"</h2>
            <input
                type="number"
                min="1"
                max="35"
                prop:value=move || input.get().to_string()
                on:input:target=move |event| {
                    if let Ok(value) = event.target().value().parse() {
                        input.set(value);
                    }
                }
            />
            <div class="grid">
                <div>
                    <h3>"Memo consumers"</h3>
                    {(0..10).map(|index| view! { <p>{index} ": " {memo_fib}</p> }).collect_view()}
                </div>
                <div>
                    <h3>"Derived consumers"</h3>
                    {(0..10).map(|index| view! { <p>{index} ": " {raw_fib}</p> }).collect_view()}
                </div>
            </div>
            <p>{move || format!(
                "memo calls: {}, derived calls: {}",
                MEMO_FIB_CALLS.load(Ordering::Relaxed),
                RAW_FIB_CALLS.load(Ordering::Relaxed)
            )}</p>
        </section>
    }
}

/// Explains the `untrack demo` step in this lesson.
///
/// What it does: renders the `UntrackDemo` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn UntrackDemo() -> impl IntoView {
    let a = RwSignal::new(0_i32);
    let b = RwSignal::new(0_i32);
    let normal_runs = RwSignal::new(0_u32);
    let untracked_runs = RwSignal::new(0_u32);

    Effect::new(move |_| {
        let _ = (a.get(), b.get());
        normal_runs.update(|value| *value += 1);
    });
    Effect::new(move |_| {
        let _ = a.get();
        let _ = b.get_untracked();
        untracked_runs.update(|value| *value += 1);
    });

    view! {
        <section class="panel">
            <h2>"Untracked reads"</h2>
            <button on:click=move |_| a.update(|value| *value += 1)>"Update A"</button>
            <button on:click=move |_| b.update(|value| *value += 1)>"Update B"</button>
            <p>{move || format!("a={} b={}", a.get(), b.get())}</p>
            <p>{move || format!("normal effect runs: {}", normal_runs.get())}</p>
            <p>{move || format!("untracked-B effect runs: {}", untracked_runs.get())}</p>
        </section>
    }
}

/// Explains the `lazy list` step in this lesson.
///
/// What it does: renders the `LazyList` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn LazyList() -> impl IntoView {
    let visible = RwSignal::new(30_u32);
    view! {
        <section class="panel">
            <h2>"Lazy list"</h2>
            <p>"The list models an IntersectionObserver strategy by hydrating more rows on demand."</p>
            <button on:click=move |_| visible.update(|value| *value = (*value + 30).min(200))>"Reveal more"</button>
            <div class="long-list">
                <For
                    each=move || { (1..=200).collect::<Vec<_>>() }
                    key=|item| *item
                    children=move |item| view! {
                        <div class=move || { if item <= visible.get() { "real-row" } else { "placeholder-row" } }>
                            {move || if item <= visible.get() {
                                format!("Hydrated row {item}")
                            } else {
                                "Placeholder".to_string()
                            }}
                        </div>
                    }
                />
            </div>
        </section>
    }
}

/// Explains the `size guidance` step in this lesson.
///
/// What it does: renders the `SizeGuidance` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn SizeGuidance() -> impl IntoView {
    const PROFILE: &str =
        "release: opt-level=z, lto=true, codegen-units=1, panic=abort, wasm-opt=z";

    view! {
        <section class="panel">
            <h2>"WASM binary size guidance"</h2>
            <ul>
                <li>"Keep web-sys feature lists tight."</li>
                <li>"Prefer memoized derived work for repeated consumers."</li>
                <li>"Use LTO and wasm-opt for release builds."</li>
                <li>"Split rarely used UI behind routes or Suspense boundaries."</li>
            </ul>
            <code>{PROFILE}</code>
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
            <h1>"Module 16 - WASM Performance"</h1>
            <BatchingBenchmark/>
            <MemoBenchmark/>
            <UntrackDemo/>
            <LazyList/>
            <SizeGuidance/>
        </main>
    }
}

const STYLE: &str = r#"
body { margin: 0; font-family: system-ui, sans-serif; background: #f8fafc; color: #17202a; }
main { max-width: 960px; margin: 0 auto; padding: 32px 16px; }
.panel { background: white; border: 1px solid #d9dee8; border-radius: 8px; padding: 18px; margin: 14px 0; }
button, input { font: inherit; padding: 8px 10px; border-radius: 6px; border: 1px solid #1f6feb; margin: 3px; }
button { background: #1f6feb; color: white; cursor: pointer; }
.grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 18px; }
.bar-row { display: grid; grid-template-columns: 90px 1fr 80px; gap: 10px; align-items: center; margin: 8px 0; }
.bar { height: 12px; border-radius: 999px; background: #e5e7eb; overflow: hidden; }
.bar div { height: 100%; min-width: 4px; background: #1f6feb; }
.long-list { max-height: 340px; overflow: auto; display: grid; gap: 6px; }
.real-row, .placeholder-row { min-height: 34px; border-radius: 6px; padding: 8px; }
.real-row { background: #ecfdf5; color: #065f46; }
.placeholder-row { background: #e5e7eb; color: #64748b; }
code { display: inline-block; background: #111827; color: #e5e7eb; padding: 8px; border-radius: 6px; }
"#;
