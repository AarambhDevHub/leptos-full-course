//! Module 15: Testing.
//! Topics: unit tests, state tests, utilities, wasm_bindgen_test, visible reports.
//! Run: cd module-15-testing && trunk serve
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

/// Explains the `validate email` step in this lesson.
///
/// What it does: checks a small email validation rule used by the testing lesson.
/// Why it matters: The test module needs a simple deterministic rule to demonstrate assertions.
fn validate_email(input: &str) -> bool {
    // A quick sanity check, not a full email parser. The only real proof that an address works is
    // sending a message to it. The edge-case tests at the bottom of this file pin down these rules.
    let Some((local, domain)) = input.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.contains('@')
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !input.contains(char::is_whitespace)
}

#[derive(Clone, Debug, PartialEq)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}

#[derive(Clone, Debug, Default)]
struct TodoState {
    items: Vec<Todo>,
    /// The id of the last todo that was added. It only ever goes up, so an id is never reused,
    /// even after a delete. (`items.len() + 1` would hand out an id that is still in the list.)
    last_id: u32,
}

impl TodoState {
    /// Explains the `add` step in this lesson.
    ///
    /// What it does: implements the `add` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    fn add(&mut self, title: impl Into<String>) -> u32 {
        let id = self.last_id + 1;
        self.last_id = id;
        self.items.push(Todo {
            id,
            title: title.into(),
            done: false,
        });
        id
    }

    /// Explains the `toggle` step in this lesson.
    ///
    /// What it does: implements the `toggle` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    fn toggle(&mut self, id: u32) {
        if let Some(todo) = self.items.iter_mut().find(|todo| todo.id == id) {
            todo.done = !todo.done;
        }
    }

    /// Explains the `delete` step in this lesson.
    ///
    /// What it does: implements the `delete` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    fn delete(&mut self, id: u32) {
        self.items.retain(|todo| todo.id != id);
    }
}

/// Explains the `format file size` step in this lesson.
///
/// What it does: formats byte counts so tests can cover utility behavior.
/// Why it matters: Small pure functions are ideal examples for unit testing.
fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    // Pick the unit by the number the user will actually see. 1,048,575 bytes is 1023.999 KB, which
    // rounds to "1024.0 KB", so it must be shown as "1.0 MB" instead.
    let shown = |value: f64| (value * 10.0).round() / 10.0;
    let bytes_f = bytes as f64;
    if shown(bytes_f / MB) >= 1024.0 {
        format!("{:.1} GB", bytes_f / GB)
    } else if shown(bytes_f / KB) >= 1024.0 {
        format!("{:.1} MB", bytes_f / MB)
    } else if bytes_f >= KB {
        format!("{:.1} KB", bytes_f / KB)
    } else {
        format!("{bytes} B")
    }
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
    let count = RwSignal::new(0_i32);
    view! {
        <div>
            <strong>{count}</strong>
            <button on:click=move |_| count.update(|value| *value += 1)>"+1"</button>
        </div>
    }
}

/// Explains the `test report` step in this lesson.
///
/// What it does: renders the `TestReport` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn TestReport() -> impl IntoView {
    let cases = vec![
        ("valid email", validate_email("darshan@example.com")),
        ("invalid email", !validate_email("darshan-example")),
        ("size 0", format_file_size(0) == "0 B"),
        ("size kb", format_file_size(1024) == "1.0 KB"),
        ("todo add", {
            let mut state = TodoState::default();
            state.add("ship tests");
            state.items.len() == 1
        }),
        ("todo toggle", {
            let mut state = TodoState::default();
            let id = state.add("toggle");
            state.toggle(id);
            state.items[0].done
        }),
        ("todo delete", {
            let mut state = TodoState::default();
            let id = state.add("delete");
            state.delete(id);
            state.items.is_empty()
        }),
    ];

    view! {
        <section class="panel">
            <h2>"Visible Runtime Test Report"</h2>
            <table>
                <thead><tr><th>"Case"</th><th>"Result"</th></tr></thead>
                <tbody>
                    {cases.into_iter().map(|(name, passed)| view! {
                        <tr>
                            <td>{name}</td>
                            <td class=if passed { "pass" } else { "fail" }>{if passed { "PASS" } else { "FAIL" }}</td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
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
            <h1>"Module 15 - Testing"</h1>
            <section class="panel">
                <h2>"Component Under Test"</h2>
                <Counter/>
            </section>
            <TestReport/>
        </main>
    }
}

const STYLE: &str = r#"
body { margin: 0; font-family: system-ui, sans-serif; background: #f8fafc; color: #17202a; }
main { max-width: 860px; margin: 0 auto; padding: 32px 16px; }
.panel { background: white; border: 1px solid #d9dee8; border-radius: 8px; padding: 18px; margin: 14px 0; }
button { font: inherit; padding: 8px 10px; border-radius: 6px; border: 1px solid #1f6feb; background: #1f6feb; color: white; cursor: pointer; margin-left: 8px; }
table { width: 100%; border-collapse: collapse; }
th, td { text-align: left; border-bottom: 1px solid #e5e7eb; padding: 8px; }
.pass { color: #047857; font-weight: 800; }
.fail { color: #b91c1c; font-weight: 800; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    /// Explains the `validates email examples` step in this lesson.
    ///
    /// What it does: implements the `validates email examples` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    #[test]
    fn validates_email_examples() {
        assert!(validate_email("a@b.com"));
        assert!(validate_email("darshan@example.dev"));
        assert!(!validate_email("missing-at.com"));
        assert!(!validate_email("missing-dot@example"));
        assert!(!validate_email(""));
    }

    /// Explains the `todo state add toggle delete` step in this lesson.
    ///
    /// What it does: implements the `todo state add toggle delete` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    #[test]
    fn todo_state_add_toggle_delete() {
        let mut state = TodoState::default();
        let id = state.add("write test");
        assert_eq!(state.items.len(), 1);
        state.toggle(id);
        assert!(state.items[0].done);
        state.delete(id);
        assert!(state.items.is_empty());
    }

    /// Explains the `file sizes are formatted` step in this lesson.
    ///
    /// What it does: implements the `file sizes are formatted` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    #[test]
    fn file_sizes_are_formatted() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// Explains the `counter signal updates` step in this lesson.
    ///
    /// What it does: implements the `counter signal updates` operation for this lesson data type or local helper.
    /// Why it matters: Method-sized behavior keeps state transitions small enough to test and explain.
    #[wasm_bindgen_test]
    fn counter_signal_updates() {
        let count = RwSignal::new(0_i32);
        assert_eq!(count.get(), 0);
        count.set(5);
        assert_eq!(count.get(), 5);
    }
}

#[cfg(test)]
mod edge_case_tests {
    //! Tests written after asking, for each helper, "what could still go wrong?".
    //! The happy-path tests above pass; these look at the edges.
    use super::*;

    #[test]
    fn ids_are_never_reused_after_a_delete() {
        let mut state = TodoState::default();
        let first = state.add("first");
        let second = state.add("second");
        state.delete(first);
        let third = state.add("third");
        assert_ne!(third, second, "a new todo must not reuse the id of a todo that is still in the list");
        let mut ids: Vec<u32> = state.items.iter().map(|todo| todo.id).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count, "every id in the list must be unique");
    }

    #[test]
    fn rejects_an_email_with_nothing_before_the_at_sign() {
        assert!(!validate_email("@b.com"));
    }

    #[test]
    fn rejects_an_email_with_two_at_signs() {
        assert!(!validate_email("a@@b.com"));
    }

    #[test]
    fn rejects_domains_that_start_or_end_with_a_dot() {
        assert!(!validate_email("a@.com"));
        assert!(!validate_email("a@b."));
    }

    #[test]
    fn rejects_an_email_with_a_space() {
        assert!(!validate_email("a b@c.com"));
    }

    #[test]
    fn file_sizes_round_up_into_the_next_unit() {
        assert_eq!(format_file_size(1023), "1023 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1_048_575), "1.0 MB");
        assert_eq!(format_file_size(1_073_741_823), "1.0 GB");
    }
}
