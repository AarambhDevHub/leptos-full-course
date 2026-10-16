//! Module 25 production server entry point.
//! Demonstrates production concerns around startup, database readiness,
//! middleware, observability, migrations, and graceful shutdown.
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
    use axum::{http::StatusCode, routing::get, Router};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use module_25_production::app::*;
    use std::time::Duration;
    use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer, trace::TraceLayer};

    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/leptos_course".into());
    let pool = connect_with_retry(&database_url, 5).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run production migrations");

    let readiness_pool = pool.clone();
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route(
            "/ready",
            get(move || {
                let pool = readiness_pool.clone();
                async move {
                    match sqlx::query("SELECT 1").execute(&pool).await {
                        Ok(_) => StatusCode::OK,
                        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
                    }
                }
            }),
        )
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let pool = pool.clone();
                move || provide_context(pool.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(leptos_options);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

/// Explains the `connect with retry` step in this lesson.
///
/// What it does: connects to PostgreSQL with startup retry behavior.
/// Why it matters: Production servers often start before the database is immediately ready.
#[cfg(feature = "ssr")]
async fn connect_with_retry(database_url: &str, attempts: u32) -> sqlx::PgPool {
    let mut last_error = None;
    for attempt in 1..=attempts {
        match sqlx::PgPool::connect(database_url).await {
            Ok(pool) => return pool,
            Err(error) => {
                tracing::warn!(attempt, error = %error, "database connection failed");
                last_error = Some(error);
                tokio::time::sleep(std::time::Duration::from_secs(attempt as u64)).await;
            }
        }
    }
    panic!(
        "connect to PostgreSQL after retries: {}",
        last_error
            .map(|error| error.to_string())
            .unwrap_or_else(|| "unknown error".into())
    );
}

/// Explains the `shutdown signal` step in this lesson.
///
/// What it does: waits for Ctrl+C or SIGTERM so the server can stop cleanly.
/// Why it matters: Graceful shutdown prevents dropped in-flight requests during deploys.
#[cfg(feature = "ssr")]
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// Explains the `main` step in this lesson.
///
/// What it does: starts the runnable entry point for this module.
/// Why it matters: It makes the module runnable and shows learners where execution really begins.
#[cfg(not(feature = "ssr"))]
pub fn main() {
    // The browser entry point is exported from `src/lib.rs` for hydration builds.
}
