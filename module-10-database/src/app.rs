//! Module 10: Database.
//! A PostgreSQL-backed notes CRUD lesson using SQLx and server functions.
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use leptos::prelude::*;
use leptos::server::ServerAction;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Title};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ssr")]
use sqlx::PgPool;

const MODULE_TITLE: &str = "Module 10 - Database";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Note {
    id: Uuid,
    title: String,
    body: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// LESSON 1: Full-Stack App Structure
// ============================================================================

/// Explains the `shell` step in this lesson.
///
/// What it does: builds the HTML document used by server rendering and hydration.
/// Why it matters: It keeps the SSR document, metadata, and hydration scripts in one predictable place.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
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
/// What it does: renders the `App` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text=MODULE_TITLE/>
        <Meta name="description" content="Database-backed CRUD in the Leptos 0.8 full course"/>
        <Router>
            <main style="max-width: 980px; margin: 0 auto; padding: 32px; font-family: system-ui, sans-serif;">
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=NotesPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Explains the `pool` step in this lesson.
///
/// What it does: retrieves the PostgreSQL pool that the server provided through Leptos context.
/// Why it matters: Server functions should receive infrastructure through context instead of constructing it repeatedly.
#[cfg(feature = "ssr")]
fn pool() -> Result<PgPool, ServerFnError> {
    use_context::<PgPool>().ok_or_else(|| ServerFnError::new("PgPool missing from context"))
}

// ============================================================================
// LESSON 3: Server Functions And Trusted Work
// ============================================================================

/// Explains the `list notes` step in this lesson.
///
/// What it does: runs the `list notes` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn list_notes() -> Result<Vec<Note>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        sqlx::query_as::<_, Note>(
            "SELECT id, title, body, created_at FROM notes ORDER BY created_at DESC",
        )
        .fetch_all(&pool()?)
        .await
        .map_err(ServerFnError::new)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// Explains the `create note` step in this lesson.
///
/// What it does: runs the `create note` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn create_note(title: String, body: String) -> Result<Note, ServerFnError> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("title is required"));
    }

    #[cfg(feature = "ssr")]
    {
        sqlx::query_as::<_, Note>(
            "INSERT INTO notes (id, title, body) VALUES ($1, $2, $3) RETURNING id, title, body, created_at",
        )
        .bind(Uuid::new_v4())
        .bind(title)
        .bind(body)
        .fetch_one(&pool()?)
        .await
        .map_err(ServerFnError::new)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Note {
            id: Uuid::new_v4(),
            title,
            body,
            created_at: chrono::Utc::now(),
        })
    }
}

/// Explains the `update note` step in this lesson.
///
/// What it does: runs the `update note` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn update_note(id: String, title: String, body: String) -> Result<Note, ServerFnError> {
    let id = Uuid::parse_str(&id).map_err(ServerFnError::new)?;
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("title is required"));
    }

    #[cfg(feature = "ssr")]
    {
        sqlx::query_as::<_, Note>(
            "UPDATE notes SET title = $2, body = $3 WHERE id = $1 RETURNING id, title, body, created_at",
        )
        .bind(id)
        .bind(title)
        .bind(body)
        .fetch_one(&pool()?)
        .await
        .map_err(ServerFnError::new)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Note {
            id,
            title,
            body,
            created_at: chrono::Utc::now(),
        })
    }
}

/// Explains the `delete note` step in this lesson.
///
/// What it does: runs the `delete note` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn delete_note(id: String) -> Result<(), ServerFnError> {
    let id = Uuid::parse_str(&id).map_err(ServerFnError::new)?;

    #[cfg(feature = "ssr")]
    {
        sqlx::query("DELETE FROM notes WHERE id = $1")
            .bind(id)
            .execute(&pool()?)
            .await
            .map_err(ServerFnError::new)?;
    }
    Ok(())
}

/// Explains the `notes page` step in this lesson.
///
/// What it does: renders the `NotesPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn NotesPage() -> impl IntoView {
    let notes = Resource::new(|| (), |_| list_notes());
    let local_notes = RwSignal::new(Vec::<Note>::new());
    let editing_id = RwSignal::new(None::<Uuid>);
    let error = RwSignal::new(String::new());

    Effect::new(move |_| {
        if let Some(Ok(items)) = notes.get() {
            local_notes.set(items);
        }
    });

    let refetch = Callback::new(move |_| notes.refetch());

    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h1>{MODULE_TITLE}</h1>
            <CreateNoteForm on_saved=refetch/>
            <p style="color: #b91c1c;">{error}</p>
            <Suspense fallback=|| view! { <p>"Loading notes..."</p> }>
                {move || notes.get().map(|result| match result {
                    Ok(_) => view! {
                        <div style="display: grid; gap: 12px;">
                            <For
                                each=move || local_notes.get()
                                key=|note| note.id
                                children=move |note| view! {
                                    <NoteCard
                                        note=note
                                        local_notes=local_notes
                                        editing_id=editing_id
                                        on_changed=refetch
                                        error=error
                                    />
                                }
                            />
                        </div>
                    }.into_any(),
                    Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
                })}
            </Suspense>
        </section>
    }
}

/// Explains the `create note form` step in this lesson.
///
/// What it does: renders the `CreateNoteForm` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn CreateNoteForm(on_saved: Callback<()>) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let body = RwSignal::new(String::new());
    let action = ServerAction::<CreateNote>::new();
    let pending = action.pending();

    Effect::new(move |_| {
        if let Some(Ok(_)) = action.value().get() {
            title.set(String::new());
            body.set(String::new());
            on_saved.run(());
        }
    });

    view! {
        <form style="display: grid; gap: 8px; margin-bottom: 18px;" on:submit=move |event| {
            event.prevent_default();
            action.dispatch(CreateNote { title: title.get(), body: body.get() });
        }>
            <input placeholder="Title" prop:value=title on:input:target=move |event| title.set(event.target().value())/>
            <textarea placeholder="Body" prop:value=body on:input:target=move |event| body.set(event.target().value())></textarea>
            <button type="submit" disabled=move || pending.get()>{move || if pending.get() { "Saving..." } else { "Create note" }}</button>
        </form>
    }
}

/// Explains the `note card` step in this lesson.
///
/// What it does: renders the `NoteCard` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn NoteCard(
    note: Note,
    local_notes: RwSignal<Vec<Note>>,
    editing_id: RwSignal<Option<Uuid>>,
    on_changed: Callback<()>,
    error: RwSignal<String>,
) -> impl IntoView {
    let title = RwSignal::new(note.title.clone());
    let body = RwSignal::new(note.body.clone());
    let id = note.id;
    let note_for_delete = note.clone();

    view! {
        <article style="padding: 14px; background: #f6f7fb; border-radius: 8px;">
            {move || if editing_id.get() == Some(id) {
                view! {
                    <div style="display: grid; gap: 8px;">
                        <input prop:value=title on:input:target=move |event| title.set(event.target().value())/>
                        <textarea prop:value=body on:input:target=move |event| body.set(event.target().value())></textarea>
                        <div>
                            <button on:click=move |_| {
                                leptos::task::spawn_local(async move {
                                    match update_note(id.to_string(), title.get(), body.get()).await {
                                        Ok(saved) => {
                                            local_notes.update(|items| {
                                                if let Some(item) = items.iter_mut().find(|item| item.id == id) {
                                                    *item = saved;
                                                }
                                            });
                                            editing_id.set(None);
                                            on_changed.run(());
                                        }
                                        Err(err) => error.set(err.to_string()),
                                    }
                                });
                            }>"Save"</button>
                            <button on:click=move |_| editing_id.set(None)>"Cancel"</button>
                        </div>
                    </div>
                }.into_any()
            } else {
                let note_for_delete = note_for_delete.clone();
                let note_for_view = note.clone();
                view! {
                    <div>
                        // Read the note from the shared list, not from the copy captured when this card was
                        // built: `<For/>` keeps the same card for an existing key, so a captured copy would
                        // keep showing the old title after an edit.
                        {move || {
                            let item = local_notes
                                .read()
                                .iter()
                                .find(|item| item.id == id)
                                .cloned()
                                .unwrap_or_else(|| note_for_view.clone());
                            view! {
                                <h2>{item.title.clone()}</h2>
                                <p>{note_preview(&item.body)}</p>
                                <small>{item.created_at.format("%Y-%m-%d %H:%M UTC").to_string()}</small>
                            }
                        }}
                        <div>
                            <button on:click=move |_| editing_id.set(Some(id))>"Edit"</button>
                            <button on:click=move |_| {
                                let original = note_for_delete.clone();
                                let position = local_notes.read().iter().position(|item| item.id == id).unwrap_or(0);
                                local_notes.update(|items| items.retain(|item| item.id != id));
                                leptos::task::spawn_local(async move {
                                    if let Err(err) = delete_note(id.to_string()).await {
                                        local_notes.update(|items| items.insert(position.min(items.len()), original));
                                        error.set(err.to_string());
                                    } else {
                                        on_changed.run(());
                                    }
                                });
                            }>"Delete"</button>
                        </div>
                    </div>
                }.into_any()
            }}
        </article>
    }
}

/// Explains the `not found` step in this lesson.
///
/// What it does: renders the `NotFound` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
/// Builds the short preview shown on a note card.
///
/// It counts characters, not bytes. Slicing a `String` at a byte index panics when that index falls
/// inside a multi-byte character (Hindi, Gujarati, emoji), and a panic kills the whole WebAssembly app.
fn note_preview(body: &str) -> String {
    const PREVIEW_CHARS: usize = 50;
    if body.chars().count() > PREVIEW_CHARS {
        format!("{}...", body.chars().take(PREVIEW_CHARS).collect::<String>())
    } else {
        body.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::note_preview;

    #[test]
    fn short_text_is_unchanged() {
        assert_eq!(note_preview("hello"), "hello");
    }

    #[test]
    fn long_ascii_text_is_cut_at_fifty_characters() {
        let preview = note_preview(&"a".repeat(80));
        assert_eq!(preview, format!("{}...", "a".repeat(50)));
    }

    #[test]
    fn long_hindi_text_does_not_panic() {
        let body = "नमस्ते दुनिया यह एक लंबा नोट है जो मराठी और हिंदी में लिखा गया है";
        let preview = note_preview(body);
        assert!(preview.ends_with("..."));
        assert_eq!(preview.chars().count(), 53);
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section>
            <h1>"404"</h1>
            <p>"The requested lesson route does not exist in this module."</p>
        </section>
    }
}
