//! Module 17: File Uploads.
//! Browser file metadata, drag/drop feedback, XHR upload progress, and
//! server-side file listing/deletion.
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Title};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use serde::{Deserialize, Serialize};

const MODULE_TITLE: &str = "Module 17 - File Uploads";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileInfo {
    name: String,
    size: u64,
    mime: String,
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
        <Meta name="description" content="File upload handling in the Leptos 0.8 full course"/>
        <Router>
            <main style="max-width: 1040px; margin: 0 auto; padding: 32px; font-family: system-ui, sans-serif;">
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=UploadPage/>
                </Routes>
            </main>
        </Router>
    }
}

// ============================================================================
// LESSON 3: Server Functions And Trusted Work
// ============================================================================

/// Explains the `list files` step in this lesson.
///
/// What it does: runs the `list files` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn list_files() -> Result<Vec<FileInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use std::fs;

        let mut files = Vec::new();
        fs::create_dir_all("./uploads").map_err(ServerFnError::new)?;
        for entry in fs::read_dir("./uploads").map_err(ServerFnError::new)? {
            let entry = entry.map_err(ServerFnError::new)?;
            let metadata = entry.metadata().map_err(ServerFnError::new)?;
            if metadata.is_file() {
                files.push(FileInfo {
                    name: entry.file_name().to_string_lossy().to_string(),
                    size: metadata.len(),
                    mime: "application/octet-stream".into(),
                });
            }
        }
        files.sort_by(|left, right| left.name.cmp(&right.name));
        return Ok(files);
    }

    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// Explains the `delete file` step in this lesson.
///
/// What it does: runs the `delete file` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn delete_file(name: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let safe_name = sanitize_name(&name)?;
        let path = std::path::Path::new("./uploads").join(safe_name);
        if path.exists() {
            std::fs::remove_file(path).map_err(ServerFnError::new)?;
        }
    }
    Ok(())
}

/// Explains the `sanitize name` step in this lesson.
///
/// What it does: normalizes an uploaded file name before touching the filesystem.
/// Why it matters: Filesystem examples must prevent path traversal and unsafe file names.
#[cfg(feature = "ssr")]
fn sanitize_name(name: &str) -> Result<String, ServerFnError> {
    let clean = name
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
        .collect::<String>();
    if clean.is_empty() || clean == "." || clean == ".." {
        return Err(ServerFnError::new("invalid file name"));
    }
    Ok(clean)
}

/// Explains the `upload page` step in this lesson.
///
/// What it does: renders the `UploadPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn UploadPage() -> impl IntoView {
    let selected = RwSignal::new(Vec::<FileInfo>::new());
    let dragover = RwSignal::new(false);
    let progress = RwSignal::new(0.0_f64);
    let upload_version = RwSignal::new(0_u32);
    let uploaded = Resource::new(move || upload_version.get(), |_| list_files());
    let error = RwSignal::new(String::new());

    view! {
        <section style="display: grid; gap: 20px;">
            <header>
                <h1>{MODULE_TITLE}</h1>
                <p>"Select or drop files to inspect browser metadata, then send them through the multipart upload route."</p>
            </header>
            <UploadZone selected progress dragover upload_version/>
            <ProgressBar progress/>
            <SelectedFiles files=selected/>
            <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <h2>"Files on server"</h2>
                    <button on:click=move |_| uploaded.refetch()>"Refresh"</button>
                </div>
                <p style="color: #b91c1c;">{error}</p>
                <Suspense fallback=|| view! { <p>"Reading uploads directory..."</p> }>
                    {move || uploaded.get().map(|result| match result {
                        Ok(files) if files.is_empty() => view! { <p>"No uploaded files yet."</p> }.into_any(),
                        Ok(files) => view! {
                            <ul style="display: grid; gap: 8px; padding-left: 0; list-style: none;">
                                <For
                                    each=move || files.clone()
                                    key=|file| file.name.clone()
                                    children=move |file| {
                                        let name = file.name.clone();
                                        view! {
                                            <li style="display: flex; justify-content: space-between; gap: 12px; padding: 10px; background: #f6f7fb; border-radius: 8px;">
                                                <span>{file.name.clone()} " - " {format_size(file.size)}</span>
                                                <button on:click=move |_| {
                                                    let name = name.clone();
                                                    leptos::task::spawn_local(async move {
                                                        match delete_file(name).await {
                                                            Ok(()) => uploaded.refetch(),
                                                            Err(err) => error.set(err.to_string()),
                                                        }
                                                    });
                                                }>"Delete"</button>
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                        }.into_any(),
                        Err(err) => view! { <p style="color: #b91c1c;">{err.to_string()}</p> }.into_any(),
                    })}
                </Suspense>
            </section>
        </section>
    }
}

/// Explains the `upload zone` step in this lesson.
///
/// What it does: renders the `UploadZone` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn UploadZone(
    selected: RwSignal<Vec<FileInfo>>,
    progress: RwSignal<f64>,
    dragover: RwSignal<bool>,
    upload_version: RwSignal<u32>,
) -> impl IntoView {
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = selected;
        let _ = progress;
        let _ = upload_version;
    }

    view! {
        <section
            style=move || {
                let color = if dragover.get() { "#164a8b" } else { "#d9dee8" };
                format!("border: 2px dashed {color}; border-radius: 8px; padding: 28px; background: #f8fafc;")
            }
            on:dragover=move |event| {
                event.prevent_default();
                dragover.set(true);
            }
            on:dragleave=move |_| dragover.set(false)
            on:drop=move |event| {
                event.prevent_default();
                dragover.set(false);
                #[cfg(feature = "hydrate")]
                if let Some(list) = event.data_transfer().and_then(|data| data.files()) {
                    selected.set(files_from_list(&list));
                    start_upload(&list, progress, upload_version);
                }
            }
        >
            <h2>"Upload zone"</h2>
            <p>"Drop files here or use the file picker."</p>
            <input
                type="file"
                multiple
                on:change:target=move |event| {
                    #[cfg(not(feature = "hydrate"))]
                    let _ = &event;
                    #[cfg(feature = "hydrate")]
                    if let Some(list) = event.target().files() {
                        selected.set(files_from_list(&list));
                        start_upload(&list, progress, upload_version);
                    }
                }
            />
        </section>
    }
}

/// Explains the `files from list` step in this lesson.
///
/// What it does: performs the `files from list` step used by this module.
/// Why it matters: Naming this step as a function keeps the lesson code readable and gives learners a place to inspect the behavior.
#[cfg(feature = "hydrate")]
fn files_from_list(list: &web_sys::FileList) -> Vec<FileInfo> {
    (0..list.length())
        .filter_map(|index| list.item(index))
        .map(|file| FileInfo {
            name: file.name(),
            size: file.size() as u64,
            mime: file.type_(),
        })
        .collect()
}

/// Explains the `start upload` step in this lesson.
///
/// What it does: builds multipart `FormData` from the selected browser files and posts it to `/upload` with XHR progress events.
/// Why it matters: Upload progress only means something when the same request body is also sent to the Axum upload route.
#[cfg(feature = "hydrate")]
fn start_upload(list: &web_sys::FileList, progress: RwSignal<f64>, upload_version: RwSignal<u32>) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    progress.set(0.0);
    if list.length() == 0 {
        return;
    }

    let Ok(form) = web_sys::FormData::new() else {
        return;
    };
    for index in 0..list.length() {
        if let Some(file) = list.item(index) {
            let blob: &web_sys::Blob = file.unchecked_ref();
            let _ = form.append_with_blob_and_filename("files", blob, &file.name());
        }
    }

    let Ok(xhr) = web_sys::XmlHttpRequest::new() else {
        return;
    };
    if xhr.open_with_async("POST", "/upload", true).is_err() {
        return;
    }

    if let Ok(upload) = xhr.upload() {
        let on_progress = Closure::<dyn FnMut(web_sys::ProgressEvent)>::wrap(Box::new({
            move |event: web_sys::ProgressEvent| {
                if event.length_computable() {
                    progress.set(event.loaded() / event.total());
                }
            }
        }));
        upload.set_onprogress(Some(on_progress.as_ref().unchecked_ref()));
        on_progress.forget();
    }

    let on_load = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        progress.set(1.0);
        upload_version.update(|version| *version += 1);
    }));
    xhr.set_onload(Some(on_load.as_ref().unchecked_ref()));
    on_load.forget();

    let _ = xhr.send_with_opt_form_data(Some(&form));
}

/// Explains the `progress bar` step in this lesson.
///
/// What it does: renders the `ProgressBar` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ProgressBar(progress: RwSignal<f64>) -> impl IntoView {
    view! {
        <section>
            <label>"Upload progress"</label>
            <div style="height: 12px; background: #e5e7eb; border-radius: 999px; overflow: hidden;">
                <div style=move || format!("height: 100%; width: {:.0}%; background: #164a8b;", progress.get() * 100.0)></div>
            </div>
        </section>
    }
}

/// Explains the `selected files` step in this lesson.
///
/// What it does: renders the `SelectedFiles` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn SelectedFiles(files: RwSignal<Vec<FileInfo>>) -> impl IntoView {
    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <h2>"Selected files"</h2>
            {move || if files.get().is_empty() {
                view! { <p>"No files selected."</p> }.into_any()
            } else {
                view! {
                    <ul>
                        <For
                            each=move || files.get()
                            key=|file| file.name.clone()
                            children=|file| view! {
                                <li>
                                    <strong>{file.name}</strong>
                                    " - "
                                    {format_size(file.size)}
                                    " - "
                                    {if file.mime.is_empty() { "unknown".into() } else { file.mime }}
                                </li>
                            }
                        />
                    </ul>
                }.into_any()
            }}
        </section>
    }
}

/// Explains the `format size` step in this lesson.
///
/// What it does: formats a byte count into a learner-friendly file size.
/// Why it matters: Formatting helpers keep UI code readable and reusable.
fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    let bytes = bytes as f64;
    if bytes < KB {
        format!("{bytes:.0} B")
    } else if bytes < KB * KB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{:.1} MB", bytes / KB / KB)
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
            <p>"The requested upload route does not exist."</p>
        </section>
    }
}
