//! Module 17 server entry point.
//! Axum owns the HTTP listener while Leptos renders the app and server functions.
//! `cargo-leptos` builds this native server and the browser hydration bundle.
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

// ============================================================================
// LESSON 1: Starting The Axum And Leptos Server
// ============================================================================

/// Explains the `main` step in this lesson.
///
/// What it does: starts the runnable entry point for this module.
/// Why it matters: It makes the module runnable and shows learners where execution really begins.
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{
        extract::{DefaultBodyLimit, Multipart},
        http::StatusCode,
        routing::post,
        Router,
    };
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use module_17_file_uploads::app::*;
    use std::path::Path;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    /// Explains the `upload handler` step in this lesson.
    ///
    /// What it does: handles the `upload handler` request path or event callback for this module.
    /// Why it matters: Handler functions make server and browser events explicit instead of hiding them inside setup code.
    async fn upload_handler(mut multipart: Multipart) -> Result<&'static str, (StatusCode, String)> {
        tokio::fs::create_dir_all("./uploads")
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|err| (err.status(), err.body_text()))?
        {
            let Some(file_name) = field.file_name().map(sanitize_upload_name) else {
                continue;
            };
            let path = Path::new("./uploads").join(file_name);
            // Report failures (for example a file over the size limit) instead of answering "ok".
            let bytes = field
                .bytes()
                .await
                .map_err(|err| (err.status(), err.body_text()))?;
            tokio::fs::write(path, bytes)
                .await
                .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
        }
        Ok("ok")
    }

    /// Explains the `sanitize upload name` step in this lesson.
    ///
    /// What it does: normalizes a multipart upload file name before saving it.
    /// Why it matters: Multipart file names come from the client, so the server must clean them first.
    fn sanitize_upload_name(name: &str) -> String {
        let clean = name
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
            .collect::<String>();
        if clean.is_empty() {
            "upload.bin".into()
        } else {
            clean
        }
    }

    let app = Router::new()
        // Axum rejects bodies over 2 MB by default; allow up to the 10 MB advertised in .env.example.
        .route(
            "/upload",
            post(upload_handler).layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES)),
        )
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// Explains the `main` step in this lesson.
///
/// What it does: starts the runnable entry point for this module.
/// Why it matters: It makes the module runnable and shows learners where execution really begins.
#[cfg(not(feature = "ssr"))]
pub fn main() {
    // The browser entry point is exported from `src/lib.rs` for hydration builds.
}
