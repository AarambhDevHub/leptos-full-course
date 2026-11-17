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
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

/// The longest storage key the panel accepts.
const MAX_KEY_LEN: usize = 100;
/// The longest value the panel saves (browsers allow about five megabytes per site).
const MAX_VALUE_LEN: usize = 10_000;

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
/// What it does: returns browser storage, or a sentence that says why it is not available.
/// Why it matters: Storage can be missing (blocked cookies, some private windows). The old code returned `None` and every button then did nothing, with no message at all.
fn storage() -> Result<web_sys::Storage, String> {
    let window = web_sys::window().ok_or_else(|| "There is no browser window".to_string())?;
    match window.local_storage() {
        Ok(Some(storage)) => Ok(storage),
        Ok(None) => Err("localStorage is not available in this browser".into()),
        Err(_) => Err("The browser denied access to localStorage".into()),
    }
}

/// Explains the `clean key` step in this lesson.
///
/// What it does: trims a storage key and rejects empty or very long ones.
/// Why it matters: Empty keys and giant values are accepted by the browser but are almost always a mistake, and a clear message beats a silent oddity.
fn clean_key(raw: &str) -> Result<&str, &'static str> {
    let key = raw.trim();
    if key.is_empty() {
        Err("Type a key first")
    } else if key.chars().count() > MAX_KEY_LEN {
        Err("That key is too long (100 characters at most)")
    } else {
        Ok(key)
    }
}

/// Explains the `save value` step in this lesson.
///
/// What it does: saves a value and reports what really happened.
/// Why it matters: `set_item` returns an error when the storage is full or blocked. The old code threw that error away and always said "Saved".
fn save_value(key: &str, value: &str) -> Result<(), String> {
    let key = clean_key(key)?;
    if value.chars().count() > MAX_VALUE_LEN {
        return Err("That value is too long (10000 characters at most)".into());
    }
    storage()?
        .set_item(key, value)
        .map_err(|_| "The browser refused to save (storage is full or blocked)".to_string())
}

/// Explains the `load value` step in this lesson.
///
/// What it does: reads a value, or `None` when nothing is stored under the key.
/// Why it matters: "Nothing there" and "an empty text" are different answers, and the old panel said "Loaded" for both.
fn load_value(key: &str) -> Result<Option<String>, String> {
    let key = clean_key(key)?;
    storage()?
        .get_item(key)
        .map_err(|_| "The browser refused to read that key".to_string())
}

/// Explains the `delete value` step in this lesson.
///
/// What it does: removes a key and reports what really happened.
/// Why it matters: Every storage call can fail, so every call returns a result the panel can show.
fn delete_value(key: &str) -> Result<(), String> {
    let key = clean_key(key)?;
    storage()?
        .remove_item(key)
        .map_err(|_| "The browser refused to delete that key".to_string())
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `local storage panel` step in this lesson.
///
/// What it does: renders the `LocalStoragePanel` component for one focused part of the lesson UI.
/// Why it matters: The status line now tells the truth: saved, loaded, nothing found, or the reason a call failed.
#[component]
fn LocalStoragePanel() -> impl IntoView {
    let key = RwSignal::new("course-note".to_string());
    let value = RwSignal::new(String::new());
    let status = RwSignal::new(String::from("Ready"));

    view! {
        <section class="panel">
            <h2>"localStorage CRUD"</h2>
            <div class="stack">
                <input aria-label="Key" prop:value=key on:input:target=move |event| key.set(event.target().value())/>
                <input aria-label="Value" placeholder="Value" prop:value=value on:input:target=move |event| value.set(event.target().value())/>
                <div class="row">
                    <button on:click=move |_| {
                        status.set(match save_value(&key.get_untracked(), &value.get_untracked()) {
                            Ok(()) => "Saved".to_string(),
                            Err(message) => message,
                        });
                    }>"Save"</button>
                    <button on:click=move |_| {
                        status.set(match load_value(&key.get_untracked()) {
                            Ok(Some(found)) => {
                                value.set(found);
                                "Loaded".to_string()
                            }
                            Ok(None) => "Nothing is stored under that key".to_string(),
                            Err(message) => message,
                        });
                    }>"Load"</button>
                    <button on:click=move |_| {
                        status.set(match delete_value(&key.get_untracked()) {
                            Ok(()) => {
                                value.set(String::new());
                                "Deleted".to_string()
                            }
                            Err(message) => message,
                        });
                    }>"Delete"</button>
                </div>
            </div>
            <p role="status">{status}</p>
        </section>
    }
}

/// Explains the `describe error` step in this lesson.
///
/// What it does: turns a JavaScript error value into a short readable name.
/// Why it matters: A rejected promise carries an object such as `NotAllowedError`, and `{:?}` on it prints nothing useful.
fn describe_error(error: &JsValue) -> String {
    js_sys::Reflect::get(error, &JsValue::from_str("name"))
        .ok()
        .and_then(|name| name.as_string())
        .or_else(|| error.as_string())
        .unwrap_or_else(|| "unknown error".to_string())
}

/// Explains the `copy to clipboard` step in this lesson.
///
/// What it does: writes text to the clipboard with the real asynchronous API and waits for the browser's answer.
/// Why it matters: The old version built a script string and ran it with `eval`. A line break in the text made it a syntax error, `eval` returned success while the promise inside was still pending or rejected, and on a page without a secure origin `navigator.clipboard` does not exist at all, yet the panel said "Copy requested".
async fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "There is no browser window".to_string())?;
    let navigator = window.navigator();
    let available = js_sys::Reflect::get(&navigator, &JsValue::from_str("clipboard"))
        .map(|value| !value.is_undefined() && !value.is_null())
        .unwrap_or(false);
    if !available {
        return Err("The clipboard needs a secure page (https or localhost)".into());
    }
    wasm_bindgen_futures::JsFuture::from(navigator.clipboard().write_text(text))
        .await
        .map(|_| ())
        .map_err(|error| format!("The browser blocked the copy ({})", describe_error(&error)))
}

/// Explains the `clipboard panel` step in this lesson.
///
/// What it does: renders the `ClipboardPanel` component for one focused part of the lesson UI.
/// Why it matters: The button starts an async task, and the status line changes only when the browser has really answered.
#[component]
fn ClipboardPanel() -> impl IntoView {
    let text = RwSignal::new("Copied from Rust/WASM".to_string());
    let status = RwSignal::new(String::from("Clipboard idle"));

    view! {
        <section class="panel">
            <h2>"navigator.clipboard (async API)"</h2>
            <input aria-label="Text to copy" prop:value=text on:input:target=move |event| text.set(event.target().value())/>
            <button on:click=move |_| {
                let value = text.get_untracked();
                status.set("Copying...".into());
                leptos::task::spawn_local(async move {
                    let message = match copy_to_clipboard(&value).await {
                        Ok(()) => "Copied".to_string(),
                        Err(message) => message,
                    };
                    let _ = status.try_set(message);
                });
            }>"Copy"</button>
            <p role="status">{status}</p>
        </section>
    }
}

/// Explains the `observer handle` step in this lesson.
///
/// What it does: keeps an `IntersectionObserver` and the Rust closure it calls alive together.
/// Why it matters: If the closure is dropped, the browser would call a function that no longer exists. Keeping both in one value lets us drop them together when the component goes away.
struct ObserverHandle {
    observer: web_sys::IntersectionObserver,
    _callback: Closure<dyn FnMut(js_sys::Array)>,
}

/// Explains the `use intersection` step in this lesson.
///
/// What it does: watches an element with a real `IntersectionObserver` and returns a signal that is true while the element is on the screen.
/// Why it matters: The old hook created a signal that nothing ever changed, so the panel said "Waiting for observer" forever and only a fake button made it say "Visible". Now the observer is created once the element exists, and it is disconnected when the component is removed.
fn use_intersection(node: NodeRef<html::Div>) -> ReadSignal<bool> {
    let (visible, set_visible) = signal(false);
    let handle: StoredValue<Option<ObserverHandle>, LocalStorage> = StoredValue::new_local(None);

    Effect::new(move |_| {
        let Some(element) = node.get() else {
            return;
        };
        if handle.with_value(|existing| existing.is_some()) {
            return;
        }
        let callback = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
            // The last entry is the most recent state of the element.
            if let Ok(entry) = entries
                .pop()
                .dyn_into::<web_sys::IntersectionObserverEntry>()
            {
                let _ = set_visible.try_set(entry.is_intersecting());
            }
        });
        if let Ok(observer) = web_sys::IntersectionObserver::new(callback.as_ref().unchecked_ref())
        {
            observer.observe(&element);
            handle.set_value(Some(ObserverHandle {
                observer,
                _callback: callback,
            }));
        }
    });

    on_cleanup(move || {
        if let Some(Some(handle)) = handle.try_update_value(|value| value.take()) {
            handle.observer.disconnect();
        }
    });

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
/// Why it matters: External scripts can load after the WASM app starts, so the component needs a readiness signal instead of a blank canvas. The signals are written with `try_set`, so a wait that outlives the component cannot write into a signal that is already gone.
fn wait_for_chart_js(chart_ready: RwSignal<bool>, status: RwSignal<String>, attempt: u8) {
    if chart_available() {
        let _ = chart_ready.try_set(true);
        let _ = status.try_set("Chart.js ready".into());
        return;
    }
    if attempt >= 50 {
        let _ = status.try_set("Chart.js script did not load".into());
        return;
    }

    let timeout = gloo_timers::callback::Timeout::new(100, move || {
        wait_for_chart_js(chart_ready, status, attempt + 1);
    });
    timeout.forget();
}

/// Explains the `chart script` step in this lesson.
///
/// What it does: builds the JavaScript that draws the bar chart, and returns `true` from the script when it really drew.
/// Why it matters: The data are `i32` numbers, so nothing but digits and minus signs can ever be written into the script. Keeping the script in a pure function lets a test check it without a browser.
fn chart_script(points: &[i32]) -> String {
    let data = points
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r##"
(() => {{
    const canvas = document.getElementById("interop-chart");
    if (!window.Chart || !canvas) return false;
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
    return true;
}})()
"##
    )
}

/// Explains the `render chart` step in this lesson.
///
/// What it does: passes Rust-owned data into Chart.js and recreates the chart instance on data changes.
/// Why it matters: Destroying the old chart prevents Chart.js from rejecting repeated renders on the same canvas, and the result tells the caller whether anything was really drawn (the old version ignored every error and always reported success).
fn render_chart(points: &[i32]) -> Result<(), String> {
    match js_sys::eval(&chart_script(points)) {
        Ok(value) if value.as_bool() == Some(true) => Ok(()),
        Ok(_) => Err("Chart.js or the canvas was not found".into()),
        Err(error) => Err(format!("Chart.js failed ({})", describe_error(&error))),
    }
}

/// Explains the `intersection panel` step in this lesson.
///
/// What it does: renders the `IntersectionPanel` component for one focused part of the lesson UI.
/// Why it matters: The box sits below a tall spacer, so you can scroll and watch the real observer change the text.
#[component]
fn IntersectionPanel() -> impl IntoView {
    let target_ref: NodeRef<html::Div> = NodeRef::new();
    let visible = use_intersection(target_ref);

    view! {
        <section class="panel">
            <h2>"IntersectionObserver"</h2>
            <p>"Scroll down: the box below reports when it is on the screen."</p>
            <div class="spacer" aria-hidden="true"></div>
            <div node_ref=target_ref class="target" role="status">
                {move || if visible.get() { "Visible" } else { "Waiting for the observer" }}
            </div>
        </section>
    }
}

/// Explains the `chart panel` step in this lesson.
///
/// What it does: renders the `ChartPanel` component for one focused part of the lesson UI.
/// Why it matters: The status line reports what the script returned, and the chart is destroyed when the panel is removed so no chart stays attached to a canvas that no longer exists.
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
            match render_chart(&data) {
                Ok(()) => status.set("Chart rendered from Rust data".into()),
                Err(message) => status.set(message),
            }
        }
    });

    on_cleanup(|| {
        let _ = js_sys::eval(
            "if (window.__interopChart) { window.__interopChart.destroy(); window.__interopChart = null; }",
        );
    });

    view! {
        <section class="panel">
            <h2>"Chart.js bridge"</h2>
            <button on:click=move |_| points.update(|items| items.rotate_left(1))>"Rotate data"</button>
            <div class="chart-wrap">
                <canvas id="interop-chart" width="480" height="180"></canvas>
            </div>
            <p role="status">{status}</p>
        </section>
    }
}

/// Explains the `event bus panel` step in this lesson.
///
/// What it does: renders the `EventBusPanel` component for one focused part of the lesson UI.
/// Why it matters: The listener used to be leaked with `forget`, so it stayed on the window for ever, and every time the panel was shown again another listener was added and the counter went up by two, then three. Now the panel owns the listener, and dropping it removes it from the window.
#[component]
fn EventBusPanel() -> impl IntoView {
    let received = RwSignal::new(0_u32);

    // Owned by this component: when the component is removed, the listener is dropped and unregistered.
    let _listener: StoredValue<Option<EventListener>, LocalStorage> =
        StoredValue::new_local(web_sys::window().map(|window| {
            EventListener::new(&window, "lesson-event", move |_| {
                let _ = received.try_update(|count| *count += 1);
            })
        }));

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
            <p role="status">{move || format!("Events received: {}", received.get())}</p>
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
button:focus-visible, input:focus-visible { outline: 3px solid #0b3d91; outline-offset: 2px; }
.spacer { height: 70vh; }
.target { margin-top: 12px; min-height: 96px; display: grid; place-items: center; border: 1px dashed #94a3b8; border-radius: 8px; background: #f8fafc; }
.chart-wrap { width: 100%; max-width: 520px; height: 220px; border: 1px solid #e5e7eb; border-radius: 8px; padding: 8px; }
canvas { width: 100%; height: 100%; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_trimmed_and_checked() {
        assert_eq!(clean_key("  note  "), Ok("note"));
        assert!(clean_key("").is_err());
        assert!(clean_key("   ").is_err());
        assert!(clean_key(&"k".repeat(MAX_KEY_LEN)).is_ok());
        assert!(clean_key(&"k".repeat(MAX_KEY_LEN + 1)).is_err());
        // Length counts characters, not bytes, so Hindi keys are not punished.
        assert!(clean_key(&"क".repeat(MAX_KEY_LEN)).is_ok());
    }

    #[test]
    fn chart_script_carries_only_numbers() {
        let script = chart_script(&[12, -3, 0]);
        assert!(script.contains("data: [12,-3,0]"));
        assert!(script.contains("return true;"));
        assert!(script.contains("return false;"));
    }

    #[test]
    fn chart_script_handles_no_data() {
        assert!(chart_script(&[]).contains("data: []"));
    }
}
