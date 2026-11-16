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

/// The most bytes one upload request may carry. The server enforces it (HTTP 413) and the
/// browser checks it first, so the user hears about a too-large file before waiting for it.
pub const MAX_UPLOAD_BYTES: u64 = 10 * 1024 * 1024;

/// A multipart body is a little bigger than the files inside it (boundaries and headers).
const MULTIPART_OVERHEAD_PER_FILE: u64 = 1024;

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
/// What it does: checks that a name given to `delete_file` is already a clean, stored name.
/// Why it matters: Filesystem examples must prevent path traversal and unsafe file names.
#[cfg(feature = "ssr")]
fn sanitize_name(name: &str) -> Result<String, ServerFnError> {
    // Stored names always come out of `sanitize_file_name`, so a name that changes when
    // cleaned again was not produced by us (for example "../x") and is refused.
    let clean = sanitize_file_name(name);
    if clean != name {
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
            <UploadZone selected progress dragover upload_version error/>
            <ProgressBar progress/>
            <p style="color: #b91c1c; min-height: 1.2em;">{error}</p>
            <SelectedFiles files=selected/>
            <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <h2>"Files on server"</h2>
                    <button on:click=move |_| uploaded.refetch()>"Refresh"</button>
                </div>
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
    error: RwSignal<String>,
) -> impl IntoView {
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = selected;
        let _ = progress;
        let _ = upload_version;
        let _ = error;
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
                    start_upload(&list, progress, upload_version, error);
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
                        start_upload(&list, progress, upload_version, error);
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
/// What it does: checks the total size, builds multipart `FormData` from the selected browser files and posts it to `/upload` with XHR progress events.
/// Why it matters: The `load` event fires for every answer, even `413`, so the status is read before the upload is called a success, and network failures get their own message.
#[cfg(feature = "hydrate")]
fn start_upload(
    list: &web_sys::FileList,
    progress: RwSignal<f64>,
    upload_version: RwSignal<u32>,
    error: RwSignal<String>,
) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    progress.set(0.0);
    error.set(String::new());
    if list.length() == 0 {
        return;
    }

    // Courtesy check: refuse early, with a clear message, instead of sending megabytes for a 413.
    let sizes: Vec<u64> = (0..list.length())
        .filter_map(|index| list.item(index))
        .map(|file| file.size() as u64)
        .collect();
    if let Err(message) = check_upload_size(&sizes) {
        error.set(message);
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

    // `load` means "an answer arrived", not "the answer was good": read the status.
    let on_load = Closure::<dyn FnMut()>::wrap(Box::new({
        let xhr = xhr.clone();
        move || {
            let status = xhr.status().unwrap_or(0);
            if (200..300).contains(&status) {
                progress.set(1.0);
                upload_version.update(|version| *version += 1);
            } else {
                progress.set(0.0);
                let body = xhr.response_text().ok().flatten().unwrap_or_default();
                error.set(upload_error_message(status, &body));
            }
        }
    }));
    xhr.set_onload(Some(on_load.as_ref().unchecked_ref()));
    on_load.forget();

    // `error` fires when no answer arrived at all (offline, connection reset).
    let on_error = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        progress.set(0.0);
        error.set(upload_error_message(0, ""));
    }));
    xhr.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_error.forget();

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
/// Why it matters: The unit is chosen from the rounded number the user will see, so the screen
/// can never say `1024.0 KB` (the boundary bug from module 15, found here again).
pub fn format_size(bytes: u64) -> String {
    const STEP: f64 = 1024.0;
    let value = bytes as f64;
    // Round exactly like the printing below, then decide the unit from that rounded number.
    let shown = |number: f64| (number * 10.0).round() / 10.0;
    if bytes < 1024 {
        format!("{bytes} B")
    } else if shown(value / STEP) < STEP {
        format!("{:.1} KB", value / STEP)
    } else if shown(value / STEP / STEP) < STEP {
        format!("{:.1} MB", value / STEP / STEP)
    } else {
        format!("{:.1} GB", value / STEP / STEP / STEP)
    }
}

/// Explains the `sanitize file name` step in this lesson.
///
/// What it does: turns a client-supplied file name into a safe name for the uploads folder.
/// Why it matters: The name comes from the client, so we keep only the last path segment, drop
/// unsafe characters, never return an empty or hidden name, and keep the extension even when
/// the rest of the name (for example Hindi letters) has to be replaced.
pub fn sanitize_file_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or("");
    let keep = |part: &str| -> String {
        part.chars()
            .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
            .collect()
    };
    let (stem, extension) = match base.rsplit_once('.') {
        Some((stem, extension)) => (stem, extension),
        None => (base, ""),
    };
    let stem = keep(stem);
    let stem = stem.trim_start_matches('.');
    let stem: String = if stem.is_empty() {
        "upload".into()
    } else {
        stem.chars().take(100).collect()
    };
    let extension: String = keep(extension)
        .chars()
        .filter(|ch| *ch != '.')
        .take(16)
        .collect();
    if extension.is_empty() {
        stem
    } else {
        format!("{stem}.{extension}")
    }
}

/// Explains the `unique name` step in this lesson.
///
/// What it does: builds the name to try on attempt number `attempt` (1 is the plain name,
/// 2 is `name-2.ext`, and so on).
/// Why it matters: Two uploads with the same name must never silently overwrite each other.
pub fn unique_name(name: &str, attempt: u32) -> String {
    if attempt <= 1 {
        return name.to_string();
    }
    match name.rsplit_once('.') {
        Some((stem, extension)) => format!("{stem}-{attempt}.{extension}"),
        None => format!("{name}-{attempt}"),
    }
}

/// Explains the `check upload size` step in this lesson.
///
/// What it does: decides whether the selected files fit in one upload request.
/// Why it matters: The browser can refuse early with a clear message. The server still checks
/// too, because a browser check is only a courtesy and never a guard.
pub fn check_upload_size(sizes: &[u64]) -> Result<(), String> {
    let total: u64 = sizes.iter().sum::<u64>() + MULTIPART_OVERHEAD_PER_FILE * sizes.len() as u64;
    if total > MAX_UPLOAD_BYTES {
        return Err(format!(
            "These files add up to {}, but one upload may carry at most {}.",
            format_size(total),
            format_size(MAX_UPLOAD_BYTES)
        ));
    }
    Ok(())
}

/// Explains the `upload error message` step in this lesson.
///
/// What it does: turns the HTTP status of a finished upload into a message for the user.
/// Why it matters: An XHR `load` event fires for every answer, including errors, so the status
/// must be read before the upload is called a success.
pub fn upload_error_message(status: u16, body: &str) -> String {
    match status {
        0 => "The upload did not reach the server. Check your connection and try again.".into(),
        413 => format!(
            "The server refused the upload because it is too large (limit {}).",
            format_size(MAX_UPLOAD_BYTES)
        ),
        _ => format!("Upload failed (HTTP {status}): {}", body.trim()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_round_sizes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn never_prints_a_thousand_and_twenty_four_of_a_unit() {
        // One byte below a megabyte used to print "1024.0 KB" (the module 15 bug, again).
        assert_eq!(format_size(1_048_575), "1.0 MB");
        assert_eq!(format_size(1_073_741_823), "1.0 GB");
        assert!(!format_size(1_048_575).contains("1024"));
    }

    #[test]
    fn keeps_the_extension_when_the_name_is_not_ascii() {
        assert_eq!(sanitize_file_name("रिपोर्ट.txt"), "upload.txt");
        assert_eq!(sanitize_file_name("नोट्स.txt"), "upload.txt");
    }

    #[test]
    fn removes_paths_and_hidden_names() {
        assert_eq!(sanitize_file_name("../../evil.txt"), "evil.txt");
        assert_eq!(sanitize_file_name("C:\\temp\\photo.png"), "photo.png");
        assert_eq!(sanitize_file_name(".."), "upload");
        assert_eq!(sanitize_file_name("."), "upload");
        assert_eq!(sanitize_file_name(".bashrc"), "upload.bashrc");
        assert_eq!(sanitize_file_name(""), "upload");
    }

    #[test]
    fn keeps_normal_names_unchanged() {
        assert_eq!(
            sanitize_file_name("report-2026_v2.pdf"),
            "report-2026_v2.pdf"
        );
        assert_eq!(sanitize_file_name("archive.tar.gz"), "archive.tar.gz");
        assert_eq!(sanitize_file_name("my file (1).png"), "myfile1.png");
    }

    #[test]
    fn a_clean_name_is_a_fixed_point() {
        for name in ["a.txt", "upload", "upload.bin", "x-y_z.tar.gz"] {
            assert_eq!(
                sanitize_file_name(&sanitize_file_name(name)),
                sanitize_file_name(name)
            );
        }
    }

    #[test]
    fn duplicate_names_get_a_counter_before_the_extension() {
        assert_eq!(unique_name("dup.txt", 1), "dup.txt");
        assert_eq!(unique_name("dup.txt", 2), "dup-2.txt");
        assert_eq!(unique_name("dup", 3), "dup-3");
        assert_eq!(unique_name("a.tar.gz", 2), "a.tar-2.gz");
    }

    #[test]
    fn size_check_has_a_boundary() {
        assert!(check_upload_size(&[]).is_ok());
        assert!(check_upload_size(&[MAX_UPLOAD_BYTES - 1024]).is_ok());
        assert!(check_upload_size(&[MAX_UPLOAD_BYTES - 1023]).is_err());
        // Several small files can still be too much together.
        assert!(check_upload_size(&[4 * 1024 * 1024; 3]).is_err());
    }

    #[test]
    fn error_messages_match_the_status() {
        assert!(upload_error_message(413, "x").contains("too large"));
        assert!(upload_error_message(0, "").contains("did not reach"));
        assert!(upload_error_message(500, " boom ").contains("(HTTP 500): boom"));
    }
}
