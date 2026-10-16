//! Module 22: JS Interop.
//! Topics: localStorage, clipboard, intersection hooks, Chart.js, CustomEvent bus.
//! Run: cd module-22-js-interop && trunk serve
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use gloo_events::EventListener;
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

/// Explains the `storage` step in this lesson.
///
/// What it does: performs the `storage` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `local storage panel` step in this lesson.
///
/// What it does: renders the `LocalStoragePanel` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn LocalStoragePanel() -> impl IntoView {
    let key = RwSignal::new("course-note".to_string());
    let value = RwSignal::new(String::new());
    let status = RwSignal::new(String::from("Ready"));

    view! {
        <section class="panel">
            <h2>"localStorage CRUD"</h2>
            <div class="stack">
                <input prop:value=key on:input:target=move |event| key.set(event.target().value())/>
                <input placeholder="Value" prop:value=value on:input:target=move |event| value.set(event.target().value())/>
                <div class="row">
                    <button on:click=move |_| {
                        if let Some(storage) = storage() {
                            let _ = storage.set_item(&key.get(), &value.get());
                            status.set("Saved".into());
                        }
                    }>"Save"</button>
                    <button on:click=move |_| {
                        if let Some(storage) = storage() {
                            value.set(storage.get_item(&key.get()).ok().flatten().unwrap_or_default());
                            status.set("Loaded".into());
                        }
                    }>"Load"</button>
                    <button on:click=move |_| {
                        if let Some(storage) = storage() {
                            let _ = storage.remove_item(&key.get());
                            value.set(String::new());
                            status.set("Deleted".into());
                        }
                    }>"Delete"</button>
                </div>
            </div>
            <p>{status}</p>
        </section>
    }
}

/// Explains the `clipboard panel` step in this lesson.
///
/// What it does: renders the `ClipboardPanel` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ClipboardPanel() -> impl IntoView {
    let text = RwSignal::new("Copied from Rust/WASM".to_string());
    let status = RwSignal::new(String::from("Clipboard idle"));

    view! {
        <section class="panel">
            <h2>"navigator.clipboard via js_sys::eval"</h2>
            <input prop:value=text on:input:target=move |event| text.set(event.target().value())/>
            <button on:click=move |_| {
                let escaped = text.get().replace('\\', "\\\\").replace('"', "\\\"");
                let script = format!("navigator.clipboard && navigator.clipboard.writeText(\"{escaped}\")");
                match js_sys::eval(&script) {
                    Ok(_) => status.set("Copy requested".into()),
                    Err(_) => status.set("Clipboard call failed".into()),
                }
            }>"Copy"</button>
            <p>{status}</p>
        </section>
    }
}

/// Explains the `use intersection` step in this lesson.
///
/// What it does: packages the `use intersection` behavior into a reusable hook-style helper.
/// Why it matters: Hooks keep repeated reactive wiring out of the components that only need to consume the behavior.
fn use_intersection(_node: NodeRef<html::Div>) -> ReadSignal<bool> {
    let (visible, _set_visible) = signal(false);
    visible
}

/// Explains the `chart available` step in this lesson.
///
/// What it does: checks whether the Chart.js script has created `window.Chart`.
/// Why it matters: JavaScript interop should verify that a global dependency exists before calling into it.
fn chart_available() -> bool {
    js_sys::eval("Boolean(window.Chart)")
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

/// Explains the `wait for chart js` step in this lesson.
///
/// What it does: waits briefly for the Chart.js script tag to finish loading before rendering the chart.
/// Why it matters: External scripts can load after the WASM app starts, so the component needs a readiness signal instead of a blank canvas.
fn wait_for_chart_js(chart_ready: RwSignal<bool>, status: RwSignal<String>, attempt: u8) {
    if chart_available() {
        chart_ready.set(true);
        status.set("Chart.js ready".into());
        return;
    }
    if attempt >= 50 {
        status.set("Chart.js script did not load".into());
        return;
    }

    let timeout = gloo_timers::callback::Timeout::new(100, move || {
        wait_for_chart_js(chart_ready, status, attempt + 1);
    });
    timeout.forget();
}

/// Explains the `render chart` step in this lesson.
///
/// What it does: passes Rust-owned data into Chart.js and recreates the chart instance on data changes.
/// Why it matters: Destroying the old chart prevents Chart.js from rejecting repeated renders on the same canvas.
fn render_chart(points: &[i32]) {
    let data = points
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let script = format!(
        r##"
(() => {{
    const canvas = document.getElementById("interop-chart");
    if (!window.Chart || !canvas) return;
    if (window.__interopChart) {{
        window.__interopChart.destroy();
    }}
    window.__interopChart = new Chart(canvas, {{
        type: "bar",
        data: {{
            labels: ["A", "B", "C", "D", "E"],
            datasets: [{{
                label: "Rust data",
                data: [{data}],
                backgroundColor: "#1f6feb"
            }}]
        }},
        options: {{
            responsive: true,
            maintainAspectRatio: false,
            animation: false
        }}
    }});
}})();
"##
    );
    let _ = js_sys::eval(&script);
}

/// Explains the `intersection panel` step in this lesson.
///
/// What it does: renders the `IntersectionPanel` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn IntersectionPanel() -> impl IntoView {
    let target_ref: NodeRef<html::Div> = NodeRef::new();
    let visible = use_intersection(target_ref);
    let simulated = RwSignal::new(false);

    view! {
        <section class="panel">
            <h2>"IntersectionObserver hook shape"</h2>
            <button on:click=move |_| simulated.update(|value| *value = !*value)>"Simulate intersection"</button>
            <div node_ref=target_ref class="target">
                {move || if visible.get() || simulated.get() { "Visible" } else { "Waiting for observer" }}
            </div>
        </section>
    }
}

/// Explains the `chart panel` step in this lesson.
///
/// What it does: renders the `ChartPanel` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ChartPanel() -> impl IntoView {
    let points = RwSignal::new(vec![12, 19, 8, 15, 22]);
    let initial_ready = chart_available();
    let chart_ready = RwSignal::new(initial_ready);
    let status = RwSignal::new(if initial_ready {
        "Chart.js ready".to_string()
    } else {
        "Loading Chart.js from script tag".to_string()
    });

    if !initial_ready {
        wait_for_chart_js(chart_ready, status, 0);
    }

    Effect::new(move |_| {
        let data = points.get();
        if chart_ready.get() {
            render_chart(&data);
            status.set("Chart rendered from Rust data".into());
        }
    });

    view! {
        <section class="panel">
            <h2>"Chart.js bridge"</h2>
            <button on:click=move |_| points.update(|items| items.rotate_left(1))>"Rotate data"</button>
            <div class="chart-wrap">
                <canvas id="interop-chart" width="480" height="180"></canvas>
            </div>
            <p>{status}</p>
        </section>
    }
}

/// Explains the `event bus panel` step in this lesson.
///
/// What it does: renders the `EventBusPanel` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn EventBusPanel() -> impl IntoView {
    let received = RwSignal::new(0_u32);

    if let Some(window) = web_sys::window() {
        let listener = EventListener::new(&window, "lesson-event", move |_| {
            received.update(|count| *count += 1);
        });
        listener.forget();
    }

    view! {
        <section class="panel">
            <h2>"CustomEvent bus"</h2>
            <button on:click=move |_| {
                if let Some(window) = web_sys::window() {
                    if let Ok(event) = web_sys::CustomEvent::new("lesson-event") {
                        let _ = window.dispatch_event(&event);
                    }
                }
            }>"Emit event"</button>
            <p>{move || format!("Events received: {}", received.get())}</p>
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
            <h1>"Module 22 - JS Interop"</h1>
            <LocalStoragePanel/>
            <ClipboardPanel/>
            <IntersectionPanel/>
            <ChartPanel/>
            <EventBusPanel/>
        </main>
    }
}

const STYLE: &str = r#"
body { margin: 0; font-family: system-ui, sans-serif; background: #f8fafc; color: #17202a; }
main { max-width: 900px; margin: 0 auto; padding: 32px 16px; }
.panel { background: white; border: 1px solid #d9dee8; border-radius: 8px; padding: 18px; margin: 14px 0; }
.row { display: flex; gap: 10px; flex-wrap: wrap; align-items: center; }
.stack { display: grid; gap: 8px; max-width: 460px; }
input, button { font: inherit; padding: 8px 10px; border-radius: 6px; border: 1px solid #94a3b8; }
button { background: #1f6feb; color: white; cursor: pointer; border-color: #1f6feb; }
.target { margin-top: 12px; min-height: 96px; display: grid; place-items: center; border: 1px dashed #94a3b8; border-radius: 8px; background: #f8fafc; }
.chart-wrap { width: 100%; max-width: 520px; height: 220px; border: 1px solid #e5e7eb; border-radius: 8px; padding: 8px; }
canvas { width: 100%; height: 100%; }
"#;
