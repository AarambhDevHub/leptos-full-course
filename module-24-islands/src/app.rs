//! Module 24: Islands Architecture.
//! Leptos 0.8 has an `islands` feature (marked experimental). With it, every `#[component]`
//! renders on the server only, and only the functions marked `#[island]` are sent to the
//! browser as WebAssembly and hydrated. This module has three islands (theme, search,
//! newsletter) inside a page that is otherwise plain server HTML.
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos::server::ServerAction;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Title};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

const MODULE_TITLE: &str = "Module 24 - Islands Architecture";

/// The key under which the chosen theme is remembered in the browser.
const THEME_KEY: &str = "module24_theme";

/// Runs in the `head`, before the page is painted, and sets `data-theme` on the `html` element.
/// A saved choice wins, then the browser's colour preference, then light. Because it runs
/// before the first paint, a visitor who chose dark never sees a white flash.
const THEME_SCRIPT: &str = r#"try{var t=localStorage.getItem("module24_theme");if(!t&&window.matchMedia&&matchMedia("(prefers-color-scheme: dark)").matches){t="dark"}if(t==="dark"){document.documentElement.setAttribute("data-theme","dark")}}catch(e){}"#;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Article {
    title: &'static str,
    category: &'static str,
    excerpt: &'static str,
    read_time: &'static str,
}

const ARTICLES: &[Article] = &[
    Article {
        title: "Server HTML first",
        category: "SSR",
        excerpt: "Ship meaningful markup before the browser downloads the WASM bundle.",
        read_time: "5 min",
    },
    Article {
        title: "Hydrate only interactions",
        category: "Performance",
        excerpt: "Search, theme, and forms can become small interactive islands.",
        read_time: "7 min",
    },
    Article {
        title: "Resource waterfalls",
        category: "Data",
        excerpt: "Suspense boundaries keep independent widgets from blocking the whole page.",
        read_time: "4 min",
    },
    Article {
        title: "Progressive enhancement",
        category: "Forms",
        excerpt: "ActionForm submits as HTML first and upgrades after hydration.",
        read_time: "6 min",
    },
];

/// Explains the `filter articles` step in this lesson.
///
/// What it does: returns the articles whose title or category contains the search text, ignoring case and surrounding spaces.
/// Why it matters: The old search kept the spaces the visitor typed, so a trailing space made "Performance " match nothing. As a plain function it is also easy to test.
fn filter_articles(query: &str) -> Vec<&'static Article> {
    let needle = query.trim().to_lowercase();
    ARTICLES
        .iter()
        .filter(|article| {
            needle.is_empty()
                || article.title.to_lowercase().contains(&needle)
                || article.category.to_lowercase().contains(&needle)
        })
        .collect()
}

/// Explains the `validate email` step in this lesson.
///
/// What it does: trims and lowercases an address and checks its shape: one `@`, a local part, and a domain with a dot and no empty parts, within the length limits of the standard.
/// Why it matters: A server function is a public door. The old check was `contains('@')`, which accepted a lone `@` and a one megabyte "address". This is still not proof that the mailbox exists, only that the text can be an address.
pub fn validate_email(raw: &str) -> Result<String, &'static str> {
    let email = raw.trim().to_lowercase();
    if email.len() > 254 {
        return Err("that address is too long");
    }
    if email.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err("an address cannot contain spaces");
    }
    let mut parts = email.split('@');
    let (local, domain) = match (parts.next(), parts.next(), parts.next()) {
        (Some(local), Some(domain), None) => (local, domain),
        _ => return Err("enter a valid email"),
    };
    if local.is_empty() || local.len() > 64 {
        return Err("enter a valid email");
    }
    if !domain.contains('.') || domain.split('.').any(|label| label.is_empty()) {
        return Err("enter a valid email");
    }
    Ok(email)
}

// ============================================================================
// LESSON 1: Full-Stack App Structure
// ============================================================================

/// Explains the `shell` step in this lesson.
///
/// What it does: builds the HTML document used by server rendering and hydration.
/// Why it matters: `HydrationScripts` with `islands=true` loads the island loader instead of hydrating the whole page, and the small script in the head applies the saved theme before the first paint.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <script inner_html=THEME_SCRIPT></script>
                <AutoReload options=options.clone()/>
                <HydrationScripts options islands=true/>
                <MetaTags/>
            </head>
            <body><App/></body>
        </html>
    }
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `app` step in this lesson.
///
/// What it does: renders the `App` component, which runs only on the server in islands mode.
/// Why it matters: Everything that is not an `#[island]`, including this router, is never compiled into the WebAssembly bundle. Links are ordinary page loads.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text=MODULE_TITLE/>
        <Meta name="description" content="Islands architecture in the Leptos 0.8 full course"/>
        <style>{STYLE}</style>
        <Router>
            <main>
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=IslandsPage/>
                </Routes>
            </main>
        </Router>
    }
}

// ============================================================================
// LESSON 3: Server Functions And Trusted Work
// ============================================================================

/// Explains the `save newsletter` step in this lesson.
///
/// What it does: runs the `save newsletter` operation on the server and returns typed data to the client.
/// Why it matters: The browser's `type=email` check is only a courtesy. The server checks again with `validate_email`, because anyone can call this function without the form.
#[server]
pub async fn save_newsletter(email: String) -> Result<String, ServerFnError> {
    let email = match validate_email(&email) {
        Ok(email) => email,
        Err(message) => return Err(ServerFnError::ServerError(message.into())),
    };
    Ok(format!("{email} subscribed"))
}

/// Explains the `islands page` step in this lesson.
///
/// What it does: renders the `IslandsPage` component for one focused part of the lesson UI.
/// Why it matters: The header, the article list and the notes are server-only. Only the three islands carry WebAssembly.
#[component]
fn IslandsPage() -> impl IntoView {
    view! {
        <section class="page">
            <header>
                <h1>{MODULE_TITLE}</h1>
                <p>"Most of this page is plain server HTML. Only the theme, search and newsletter widgets are islands: small pieces that wake up in the browser."</p>
            </header>
            <ThemeIsland/>
            <SearchIsland/>
            <BlogLayout/>
            <div class="two-columns">
                <NewsletterIsland/>
                <HydrationNotes/>
            </div>
        </section>
    }
}

/// Explains the `blog layout` step in this lesson.
///
/// What it does: renders the `BlogLayout` component for one focused part of the lesson UI.
/// Why it matters: This is a normal component, so in islands mode it costs the browser no WebAssembly at all.
#[component]
fn BlogLayout() -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"Static article list"</h2>
            <div class="cards">
                {ARTICLES.iter().map(|article| view! {
                    <article class="card">
                        <span class="category">{article.category}</span>
                        <h3>{article.title}</h3>
                        <p>{article.excerpt}</p>
                        <small>{article.read_time}</small>
                    </article>
                }).collect_view()}
            </div>
        </section>
    }
}

/// Explains the `search island` step in this lesson.
///
/// What it does: renders a search box that filters the article titles as you type.
/// Why it matters: The old version waited for a client-only resource inside `Suspense`, so the server HTML showed only "Preparing search island..." and the list appeared after hydration. Now the island renders the full list on the server and only the filtering runs in the browser.
#[island]
fn SearchIsland() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let results = Memo::new(move |_| filter_articles(&query.get()));

    view! {
        <section class="panel">
            <h2>"Search island"</h2>
            <label for="search-input">"Search articles"</label>
            <input
                id="search-input"
                type="search"
                autocomplete="off"
                prop:value=query
                on:input:target=move |event| query.set(event.target().value())
            />
            <p role="status">{move || format!("{} of {} articles", results.get().len(), ARTICLES.len())}</p>
            <ul>
                {move || results.get().into_iter().map(|article| view! {
                    <li>{article.title} " - " {article.category}</li>
                }).collect_view()}
            </ul>
        </section>
    }
}

/// Explains the `initial dark` step in this lesson.
///
/// What it does: reads the theme that the script in the head already applied to the page.
/// Why it matters: The head script is the single place that decides the starting theme, so the island only reads its answer. The old island always started as light and then overwrote the saved choice.
fn initial_dark() -> bool {
    #[cfg(feature = "hydrate")]
    {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.document_element())
            .and_then(|root| root.get_attribute("data-theme"))
            .as_deref()
            == Some("dark")
    }
    #[cfg(not(feature = "hydrate"))]
    {
        false
    }
}

/// Explains the `save theme` step in this lesson.
///
/// What it does: applies a theme to the page and remembers it in the browser.
/// Why it matters: It is called only after a click, so a theme that was merely guessed from the browser setting is never saved as if the visitor had chosen it.
fn save_theme(dark: bool) {
    #[cfg(feature = "hydrate")]
    {
        let name = if dark { "dark" } else { "light" };
        if let Some(window) = web_sys::window() {
            if let Some(root) = window.document().and_then(|d| d.document_element()) {
                let _ = root.set_attribute("data-theme", name);
            }
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(THEME_KEY, name);
            }
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (dark, THEME_KEY);
    }
}

/// Explains the `theme island` step in this lesson.
///
/// What it does: renders a toggle button that switches the page between light and dark.
/// Why it matters: The colours come from CSS variables that change with `data-theme`, so every card, button and text changes together. The old code only changed the page background and left the text colour inherited by light cards, which made the text almost invisible. `aria-pressed` tells screen readers whether dark mode is on.
#[island]
fn ThemeIsland() -> impl IntoView {
    let dark = RwSignal::new(initial_dark());

    // The first run only records the starting value. Later runs are real clicks.
    Effect::new(move |previous: Option<bool>| {
        let now = dark.get();
        if previous.is_some() && previous != Some(now) {
            save_theme(now);
        }
        now
    });

    view! {
        <section class="bar">
            <strong>"Theme island"</strong>
            <button
                aria-pressed=move || if dark.get() { "true" } else { "false" }
                on:click=move |_| dark.update(|value| *value = !*value)
            >
                "Dark mode"
            </button>
        </section>
    }
}

/// Explains the `newsletter island` step in this lesson.
///
/// What it does: renders a sign-up form that calls the server function, with the result shown in a status region.
/// Why it matters: The label makes the input accessible (a placeholder is not a label), the status region exists from the start so screen readers announce the result, and the form still posts as plain HTML when JavaScript is off.
#[island]
fn NewsletterIsland() -> impl IntoView {
    let action = ServerAction::<SaveNewsletter>::new();
    let pending = action.pending();
    let value = action.value();

    view! {
        <section class="panel">
            <h2>"Newsletter island"</h2>
            <ActionForm action=action>
                <label for="email-input">"Email"</label>
                <div class="row">
                    <input id="email-input" name="email" type="email" required maxlength="254" autocomplete="email"/>
                    <button type="submit" disabled=move || pending.get()>
                        {move || if pending.get() { "Saving..." } else { "Subscribe" }}
                    </button>
                </div>
            </ActionForm>
            <div role="status">
                {move || value.get().map(|result| match result {
                    Ok(message) => view! { <p class="ok">{message}</p> }.into_any(),
                    Err(error) => view! { <p class="error">{error.to_string()}</p> }.into_any(),
                })}
            </div>
        </section>
    }
}

/// Explains the `hydration notes` step in this lesson.
///
/// What it does: lists which parts of the page ship WebAssembly and which do not.
/// Why it matters: The old card showed sizes such as "92 KB" and "18 KB" that were typed into the source and never measured. Real sizes depend on your build, so measure them yourself (see the README).
#[component]
fn HydrationNotes() -> impl IntoView {
    view! {
        <aside class="panel">
            <h2>"What ships WebAssembly"</h2>
            <table>
                <tbody>
                    <tr><th scope="row">"Article list"</th><td>"No, server HTML only"</td></tr>
                    <tr><th scope="row">"Theme island"</th><td>"Yes, one signal and one effect"</td></tr>
                    <tr><th scope="row">"Search island"</th><td>"Yes, filtering as you type"</td></tr>
                    <tr><th scope="row">"Newsletter island"</th><td>"Yes, the form and its result"</td></tr>
                </tbody>
            </table>
        </aside>
    }
}

/// Explains the `not found` step in this lesson.
///
/// What it does: renders the `NotFound` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section>
            <h1>"404"</h1>
            <p>"The requested islands route does not exist."</p>
        </section>
    }
}

const STYLE: &str = r#"
:root { --bg: #ffffff; --fg: #111827; --card: #f6f7fb; --border: #d9dee8; --accent: #164a8b; --ok: #166534; --error: #b91c1c; }
:root[data-theme="dark"] { --bg: #111827; --fg: #f9fafb; --card: #1f2937; --border: #374151; --accent: #93c5fd; --ok: #86efac; --error: #fca5a5; color-scheme: dark; }
body { margin: 0; background: var(--bg); color: var(--fg); font-family: system-ui, sans-serif; }
main { max-width: 1080px; margin: 0 auto; padding: 32px; }
.page { display: grid; gap: 20px; }
.panel, .bar { border: 1px solid var(--border); border-radius: 8px; padding: 20px; }
.bar { display: flex; justify-content: space-between; align-items: center; padding: 14px 20px; }
.cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 12px; }
.card { background: var(--card); border-radius: 8px; padding: 16px; }
.category { color: var(--accent); font-weight: 700; }
.two-columns { display: grid; grid-template-columns: minmax(0, 1fr) minmax(260px, 360px); gap: 16px; }
.row { display: flex; gap: 8px; flex-wrap: wrap; }
label { display: block; margin-bottom: 4px; font-weight: 600; }
input, button { font: inherit; color: var(--fg); background: var(--card); border: 1px solid var(--border); border-radius: 6px; padding: 8px 10px; }
button { cursor: pointer; }
button[aria-pressed="true"] { background: var(--accent); color: var(--bg); }
button:focus-visible, input:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
th { text-align: left; padding-right: 12px; }
.ok { color: var(--ok); }
.error { color: var(--error); }
@media (max-width: 720px) { .two-columns { grid-template-columns: 1fr; } }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_search_shows_everything() {
        assert_eq!(filter_articles("").len(), ARTICLES.len());
        assert_eq!(filter_articles("   ").len(), ARTICLES.len());
    }

    #[test]
    fn search_ignores_case_and_surrounding_spaces() {
        let found = filter_articles("  PERFORMANCE ");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, "Hydrate only interactions");
        assert_eq!(filter_articles("server").len(), 1);
        assert!(filter_articles("zzz").is_empty());
    }

    #[test]
    fn good_addresses_are_accepted_and_normalised() {
        assert_eq!(
            validate_email("  Good@Example.COM "),
            Ok("good@example.com".to_string())
        );
        assert!(validate_email("a.b+tag@mail.example.org").is_ok());
    }

    #[test]
    fn bad_addresses_are_refused() {
        for bad in [
            "",
            "@",
            "a@",
            "@b.com",
            "a@b",
            "a@@b.com",
            "a@b..com",
            "a@.com",
            "a@b.",
            "a b@c.com",
            "a@b.com\nbcc:x@y.com",
        ] {
            assert!(validate_email(bad).is_err(), "should refuse {bad:?}");
        }
    }

    #[test]
    fn huge_addresses_are_refused() {
        let local = "a".repeat(65);
        assert!(validate_email(&format!("{local}@example.com")).is_err());
        let huge = format!("{}@example.com", "a".repeat(1_000_000));
        assert!(validate_email(&huge).is_err());
    }
}
