//! Module 02: Components, Props, and Children.
//! Topics: required props, optional props, defaults, children slot, enum props.
//! Run: cd module-02-components && trunk serve
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

#[derive(Clone, PartialEq)]
enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `button` step in this lesson.
///
/// What it does: renders the `Button` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Button(
    #[prop(into)] label: String,
    #[prop(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_click: Option<Callback<web_sys::MouseEvent>>,
) -> impl IntoView {
    let (bg, text_color) = match &variant {
        ButtonVariant::Primary => ("#1d4ed8", "white"),
        ButtonVariant::Secondary => ("#6b7280", "white"),
        ButtonVariant::Danger => ("#dc2626", "white"),
        ButtonVariant::Ghost => ("transparent", "#374151"),
    };
    let style = format!(
        "padding: 8px 16px; background: {bg}; color: {text_color}; border: 1px solid {bg}; \
         border-radius: 6px; cursor: pointer; margin: 4px; font: inherit;"
    );

    view! {
        <button
            style=style
            disabled=disabled
            on:click=move |event| {
                if let Some(callback) = &on_click {
                    callback.run(event);
                }
            }
        >
            {label}
        </button>
    }
}

/// Explains the `card` step in this lesson.
///
/// What it does: renders the `Card` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Card(
    #[prop(into)] title: String,
    #[prop(optional)] subtitle: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div style="border: 1px solid #e5e7eb; border-radius: 8px; overflow: hidden; margin: 8px 0;">
            <div style="padding: 12px 16px; background: #f9fafb; border-bottom: 1px solid #e5e7eb;">
                <h3 style="margin: 0 0 2px; font-size: 1rem;">{title}</h3>
                {subtitle.map(|s| view! { <p style="margin: 0; font-size: 0.85rem; color: #6b7280;">{s}</p> })}
            </div>
            <div style="padding: 16px;">{children()}</div>
        </div>
    }
}

#[derive(Clone)]
enum BadgeColor {
    Blue,
    Green,
    Yellow,
    Red,
    Gray,
}

/// Explains the `badge` step in this lesson.
///
/// What it does: renders the `Badge` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Badge(
    #[prop(into)] text: String,
    #[prop(default = BadgeColor::Blue)] color: BadgeColor,
) -> impl IntoView {
    let (bg, fg) = match &color {
        BadgeColor::Blue => ("#dbeafe", "#1e40af"),
        BadgeColor::Green => ("#dcfce7", "#166534"),
        BadgeColor::Yellow => ("#fef9c3", "#854d0e"),
        BadgeColor::Red => ("#fee2e2", "#991b1b"),
        BadgeColor::Gray => ("#f3f4f6", "#374151"),
    };

    view! {
        <span style=format!(
            "display: inline-block; padding: 2px 8px; border-radius: 999px; \
             background: {bg}; color: {fg}; font-size: 0.78rem; font-weight: 500; margin: 2px;"
        )>
            {text}
        </span>
    }
}

#[derive(Clone, PartialEq)]
enum AlertType {
    Info,
    Success,
    Warning,
    Error,
}

/// Explains the `alert` step in this lesson.
///
/// What it does: renders the `Alert` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Alert(
    #[prop(into)] message: String,
    #[prop(default = AlertType::Info)] alert_type: AlertType,
    #[prop(default = false)] dismissible: bool,
) -> impl IntoView {
    let (dismissed, set_dismissed) = signal(false);
    let (border, bg, icon) = match &alert_type {
        AlertType::Info => ("#3b82f6", "#eff6ff", "i"),
        AlertType::Success => ("#22c55e", "#f0fdf4", "ok"),
        AlertType::Warning => ("#f59e0b", "#fffbeb", "!"),
        AlertType::Error => ("#ef4444", "#fef2f2", "x"),
    };

    view! {
        <Show when=move || !dismissed.get()>
            <div style=format!(
                "display: flex; align-items: center; padding: 10px 14px; background: {bg}; \
                 border-left: 4px solid {border}; border-radius: 6px; margin: 8px 0; gap: 8px;"
            )>
                <span>{icon}</span>
                <span style="flex: 1;">{message.clone()}</span>
                {dismissible.then(|| view! {
                    <button
                        on:click=move |_| set_dismissed.set(true)
                        style="background: none; border: none; cursor: pointer; font-size: 1rem; color: #6b7280;"
                    >
                        "x"
                    </button>
                })}
            </div>
        </Show>
    }
}

#[derive(Clone, PartialEq)]
enum AvatarSize {
    Sm,
    Md,
    Lg,
}

/// Explains the `avatar` step in this lesson.
///
/// What it does: renders the `Avatar` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Avatar(
    #[prop(into)] name: String,
    #[prop(optional)] image_url: Option<String>,
    #[prop(default = AvatarSize::Md)] size: AvatarSize,
) -> impl IntoView {
    let px = match &size {
        AvatarSize::Sm => "32px",
        AvatarSize::Md => "48px",
        AvatarSize::Lg => "64px",
    };
    let font = match &size {
        AvatarSize::Sm => "0.75rem",
        AvatarSize::Md => "1rem",
        AvatarSize::Lg => "1.5rem",
    };
    let initials = name
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();

    view! {
        <div style=format!(
            "width: {px}; height: {px}; border-radius: 50%; display: inline-flex; \
             align-items: center; justify-content: center; overflow: hidden; \
             background: #6366f1; color: white; font-size: {font}; font-weight: 600; margin: 4px;"
        )>
            {if let Some(url) = image_url {
                view! { <img src=url alt=name.clone() style="width: 100%; height: 100%; object-fit: cover;"/> }.into_any()
            } else {
                view! { <span>{initials}</span> }.into_any()
            }}
        </div>
    }
}

/// Explains the `app` step in this lesson.
///
/// What it does: renders the `App` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn App() -> impl IntoView {
    let (clicks, set_clicks) = signal(0_u32);

    view! {
        <main style="max-width: 720px; margin: 2rem auto; padding: 0 1rem; font-family: system-ui, sans-serif;">
            <h1 style="border-bottom: 2px solid #e2e8f0; padding-bottom: 0.5rem;">
                "Module 02 - Components"
            </h1>
            <Card title="Button showcase" subtitle="All four variants".to_string()>
                <Button label="Primary" on_click=Callback::new(move |_| set_clicks.update(|n| *n += 1))/>
                <Button label="Secondary" variant=ButtonVariant::Secondary/>
                <Button label="Danger" variant=ButtonVariant::Danger/>
                <Button label="Ghost" variant=ButtonVariant::Ghost/>
                <Button label="Disabled" disabled=true/>
                <p style="margin: 8px 0 0;">{move || format!("Total primary clicks: {}", clicks.get())}</p>
            </Card>
            <Card title="Badge component" subtitle="Color-coded status labels".to_string()>
                <Badge text="In progress" color=BadgeColor::Blue/>
                <Badge text="Shipped" color=BadgeColor::Green/>
                <Badge text="Warning" color=BadgeColor::Yellow/>
                <Badge text="Failed" color=BadgeColor::Red/>
                <Badge text="Draft" color=BadgeColor::Gray/>
            </Card>
            <Card title="Alert component" subtitle="Dismissible and non-dismissible".to_string()>
                <Alert message="Informational message." alert_type=AlertType::Info/>
                <Alert message="Operation succeeded!" alert_type=AlertType::Success dismissible=true/>
                <Alert message="This is a warning." alert_type=AlertType::Warning dismissible=true/>
                <Alert message="An error occurred." alert_type=AlertType::Error/>
            </Card>
            <Card title="Avatar component" subtitle="Initials fallback, three sizes".to_string()>
                <Avatar name="Darshan Vichhi" size=AvatarSize::Sm/>
                <Avatar name="Aarambh Dev Hub" size=AvatarSize::Md/>
                <Avatar name="Leptos Rust" size=AvatarSize::Lg/>
            </Card>
        </main>
    }
}
