//! Module 14: Animations.
//! Topics: transitions, deferred removal, drawers, animated values, timed loops.
//! Run: cd module-14-animations && trunk serve
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use gloo_timers::callback::{Interval, Timeout};
use gloo_timers::future::sleep;
use leptos::prelude::*;
use std::time::Duration;

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

/// Explains the `fade in out` step in this lesson.
///
/// What it does: renders the `FadeInOut` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn FadeInOut() -> impl IntoView {
    let visible = RwSignal::new(true);
    let mounted = RwSignal::new(true);

    view! {
        <section class="panel">
            <h2>"FadeInOut with deferred removal"</h2>
            <button on:click=move |_| {
                if visible.get() {
                    visible.set(false);
                    Timeout::new(420, move || mounted.set(false)).forget();
                } else {
                    mounted.set(true);
                    visible.set(true);
                }
            }>"Toggle"</button>
            <Show when=move || mounted.get()>
                <div
                    class="fade-box"
                    style:opacity=move || if visible.get() { "1" } else { "0" }
                    style:transition="opacity 0.4s ease"
                >
                    "The node stays mounted until the exit animation completes."
                </div>
            </Show>
        </section>
    }
}

/// Explains the `slide drawer` step in this lesson.
///
/// What it does: renders the `SlideDrawer` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn SlideDrawer() -> impl IntoView {
    let open = RwSignal::new(false);

    view! {
        <section class="panel drawer-demo">
            <h2>"SlideDrawer"</h2>
            <button on:click=move |_| open.set(true)>"Open drawer"</button>
            <div
                class="overlay"
                style:opacity=move || if open.get() { "1" } else { "0" }
                style:pointer-events=move || if open.get() { "auto" } else { "none" }
                on:click=move |_| open.set(false)
            ></div>
            <aside
                class="drawer"
                style:transform=move || if open.get() { "translateX(0)" } else { "translateX(-110%)" }
            >
                <h3>"Navigation"</h3>
                <p>"Transform transitions are GPU-friendly and predictable."</p>
                <button on:click=move |_| open.set(false)>"Close"</button>
            </aside>
        </section>
    }
}

/// Explains the `animated counter` step in this lesson.
///
/// What it does: renders the `AnimatedCounter` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn AnimatedCounter() -> impl IntoView {
    let target = RwSignal::new(0.0_f64);
    let display = RwSignal::new(0.0_f64);

    let animate_to = move |next: f64| {
        target.set(next);
        let from = display.get();
        leptos::task::spawn_local(async move {
            for step in 1..=24 {
                sleep(Duration::from_millis(16)).await;
                let t = step as f64 / 24.0;
                display.set(from + (next - from) * t);
            }
        });
    };

    view! {
        <section class="panel">
            <h2>"AnimatedCounter"</h2>
            <p class="big-number">{move || format!("{:.0}", display.get())}</p>
            <button on:click=move |_| animate_to(target.get() + 25.0)>"+25"</button>
            <button on:click=move |_| animate_to(0.0)>"Reset"</button>
        </section>
    }
}

/// Explains the `staggered list` step in this lesson.
///
/// What it does: renders the `StaggeredList` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn StaggeredList() -> impl IntoView {
    let run = RwSignal::new(0_u32);
    let items = ["Signals", "Effects", "Memos", "Resources", "Actions"];

    view! {
        <section class="panel">
            <h2>"StaggeredList"</h2>
            <button on:click=move |_| run.update(|value| *value += 1)>"Replay"</button>
            <ul class="staggered" data-run=move || run.get().to_string()>
                {items.into_iter().enumerate().map(|(index, item)| view! {
                    <li style=format!("animation-delay: {}ms", index * 100)>{item}</li>
                }).collect_view()}
            </ul>
        </section>
    }
}

/// Explains the `bouncing ball` step in this lesson.
///
/// What it does: renders the `BouncingBall` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn BouncingBall() -> impl IntoView {
    let running = RwSignal::new(false);
    let y = RwSignal::new(0.0_f64);
    let velocity = RwSignal::new(0.0_f64);

    Interval::new(16, move || {
        if !running.get() {
            return;
        }
        velocity.update(|value| *value += 0.8);
        y.update(|position| {
            *position += velocity.get();
            if *position > 170.0 {
                *position = 170.0;
                velocity.update(|value| *value *= -0.72);
            }
        });
    })
    .forget();

    view! {
        <section class="panel">
            <h2>"BouncingBall loop"</h2>
            <button on:click=move |_| {
                y.set(0.0);
                velocity.set(0.0);
                running.set(true);
            }>"Drop ball"</button>
            <button on:click=move |_| running.set(false)>"Pause"</button>
            <div class="stage">
                <div class="ball" style:transform=move || format!("translateY({:.1}px)", y.get())></div>
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
    view! {
        <style>{STYLE}</style>
        <main>
            <h1>"Module 14 - Animations"</h1>
            <FadeInOut/>
            <SlideDrawer/>
            <AnimatedCounter/>
            <StaggeredList/>
            <BouncingBall/>
        </main>
    }
}

const STYLE: &str = r#"
body { margin: 0; font-family: system-ui, sans-serif; background: #f8fafc; color: #17202a; }
main { max-width: 940px; margin: 0 auto; padding: 32px 16px; }
.panel { background: white; border: 1px solid #d9dee8; border-radius: 8px; padding: 18px; margin: 14px 0; position: relative; overflow: hidden; }
button { font: inherit; padding: 8px 10px; border-radius: 6px; border: 1px solid #1f6feb; background: #1f6feb; color: white; cursor: pointer; margin: 3px; }
.fade-box { margin-top: 12px; padding: 20px; border-radius: 8px; background: #eef2ff; color: #3730a3; }
.overlay { position: fixed; inset: 0; background: rgba(15,23,42,.36); transition: opacity .3s ease; z-index: 2; }
.drawer { position: fixed; top: 0; bottom: 0; left: 0; width: min(320px, 80vw); background: white; color: #17202a; padding: 24px; box-shadow: 12px 0 30px rgba(0,0,0,.18); transition: transform .3s ease; z-index: 3; }
.big-number { font-size: 3rem; font-weight: 800; margin: 8px 0; }
.staggered { display: grid; gap: 8px; padding: 0; list-style: none; }
.staggered li { padding: 10px; border-radius: 8px; background: #eef2ff; animation: slideIn .45s both; }
@keyframes slideIn { from { opacity: 0; transform: translateX(-20px); } to { opacity: 1; transform: translateX(0); } }
.stage { height: 220px; border: 1px solid #cbd5e1; border-radius: 8px; margin-top: 12px; position: relative; background: linear-gradient(#f8fafc, #e0f2fe); }
.ball { width: 44px; height: 44px; border-radius: 50%; background: #f97316; position: absolute; top: 0; left: 48%; box-shadow: inset -8px -8px 0 rgba(0,0,0,.12); }
"#;
