//! Module 25: Production.
//! Capstone application combining auth, protected routes, notes CRUD,
//! operational activity, metadata, and root-level error handling.
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

// Server function bodies use `return` inside a feature block because the block
// for the other feature follows it; that shape is clearer for learners.
#![allow(clippy::needless_return)]

#[cfg(feature = "ssr")]
use crate::guard;
use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos::server::ServerAction;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Title};
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::hooks::use_navigate;
use leptos_router::path;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ssr")]
use sqlx::PgPool;

const MODULE_TITLE: &str = "Module 25 - Production";

/// Database row for a user, including the password hash.
/// Server only, and deliberately not Serialize or Debug: a type that can be
/// sent to the browser or printed in a log must never hold the hash.
#[cfg(feature = "ssr")]
#[derive(Clone, sqlx::FromRow)]
struct ProdUser {
    id: Uuid,
    name: String,
    email: String,
    password_hash: String,
    role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ProdUserInfo {
    id: Uuid,
    name: String,
    email: String,
    role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProdClaims {
    sub: String,
    email: String,
    role: String,
    exp: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ProdNote {
    id: Uuid,
    user_id: Uuid,
    title: String,
    body: String,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ActivityEvent {
    id: i64,
    message: String,
    severity: String,
    at: String,
}

/// Real counts read from the database for the dashboard.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DashboardCounts {
    notes: i64,
    events: i64,
}

/// The signed-in user as the UI knows it.
///
/// `me` is the server's answer, loaded during server rendering and handed to
/// the browser, so a protected page is correct in the very first HTML.
/// `local` holds what this tab just learned (a login or a logout) and wins
/// over `me` until the page is reloaded.
#[derive(Clone, Copy)]
pub struct Session {
    me: Resource<Result<Option<ProdUserInfo>, ServerFnError>>,
    local: RwSignal<Option<Option<ProdUserInfo>>>,
}

impl Session {
    pub fn user(&self) -> Option<ProdUserInfo> {
        match self.local.get() {
            Some(user) => user,
            None => self.me.get().and_then(|result| result.ok()).flatten(),
        }
    }

    pub fn set(&self, user: Option<ProdUserInfo>) {
        self.local.set(Some(user));
    }
}

pub type ThemeContext = RwSignal<Theme>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
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
    let theme = RwSignal::new(Theme::Light);
    provide_context(theme);
    provide_context(Session {
        me: Resource::new(|| (), |_| prod_me()),
        local: RwSignal::new(None),
    });

    view! {
        <Title text=MODULE_TITLE/>
        <Meta name="description" content="Production capstone for the Leptos 0.8 full course"/>
        <Router>
            <div style=move || match theme.get() {
                Theme::Light => "min-height: 100vh; background: #ffffff; color: #111827;".to_string(),
                Theme::Dark => "min-height: 100vh; background: #111827; color: #f9fafb;".to_string(),
            }>
                <main style="max-width: 1120px; margin: 0 auto; padding: 32px; font-family: system-ui, sans-serif;">
                    <ErrorBoundary fallback=|errors| view! {
                        <section style="border: 1px solid #fecaca; border-radius: 8px; padding: 20px; background: #fef2f2; color: #111827;">
                            <h1>"Application error"</h1>
                            <ul>{move || errors.get().into_iter().map(|(_, error)| view! { <li>{error.to_string()}</li> }).collect_view()}</ul>
                        </section>
                    }>
                        <AppNav/>
                        <Routes fallback=|| view! { <NotFoundPage/> }>
                            <Route path=path!("/") view=HomePage/>
                            <Route path=path!("/login") view=LoginPage/>
                            <Route path=path!("/register") view=RegisterPage/>
                            <Route path=path!("/dashboard") view=DashboardPage/>
                            <Route path=path!("/notes") view=NotesPage/>
                            <Route path=path!("/profile") view=ProfilePage/>
                            <Route path=path!("/settings") view=SettingsPage/>
                            <Route path=path!("/activity") view=ActivityPage/>
                        </Routes>
                    </ErrorBoundary>
                </main>
            </div>
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

/// Explains the `prod secret` step in this lesson.
///
/// What it does: loads the JWT signing secret used by the production capstone.
/// Why it matters: Production auth needs a single secret source that can come from environment configuration.
#[cfg(feature = "ssr")]
static SECRET: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Reads and checks JWT_SECRET once. The server calls this before it listens,
/// so a missing or weak secret stops the start instead of silently using a
/// default that everyone could read in the course repository.
#[cfg(feature = "ssr")]
pub fn init_secret() -> Result<(), String> {
    let secret = guard::validate_secret(std::env::var("JWT_SECRET").ok().as_deref())?;
    let _ = SECRET.set(secret);
    Ok(())
}

#[cfg(feature = "ssr")]
fn prod_secret() -> Result<&'static str, ServerFnError> {
    SECRET
        .get()
        .map(String::as_str)
        .ok_or_else(|| internal("session", "JWT secret was not initialised"))
}

/// Whether the session cookie gets the Secure flag (the default).
/// COOKIE_SECURE=false turns it off for local development over plain http.
#[cfg(feature = "ssr")]
fn secure_cookies() -> bool {
    static SECURE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *SECURE.get_or_init(|| guard::cookie_secure(std::env::var("COOKIE_SECURE").ok().as_deref()))
}

/// Logs the real error on the server and gives the browser a generic one.
/// Database messages can name tables, columns and values.
#[cfg(feature = "ssr")]
fn internal(context: &str, error: impl std::fmt::Display) -> ServerFnError {
    tracing::error!("{context} failed: {error}");
    ServerFnError::new("something went wrong on the server, please try again")
}

/// Failed sign-ins per email, shared by every request in this process.
#[cfg(feature = "ssr")]
fn limiter() -> std::sync::MutexGuard<'static, guard::AttemptLimiter> {
    static LIMITER: std::sync::OnceLock<std::sync::Mutex<guard::AttemptLimiter>> =
        std::sync::OnceLock::new();
    LIMITER
        .get_or_init(|| {
            std::sync::Mutex::new(guard::AttemptLimiter::new(
                5,
                std::time::Duration::from_secs(15 * 60),
            ))
        })
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Writes one row to the activity feed. Best effort: a failed log write is
/// reported on the server and never breaks the action it describes.
#[cfg(feature = "ssr")]
async fn log_event(pool: &PgPool, user_id: Option<Uuid>, severity: &str, message: &str) {
    if let Err(error) =
        sqlx::query("INSERT INTO activity_events (user_id, message, severity) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(message)
            .bind(severity)
            .execute(pool)
            .await
    {
        tracing::error!("activity log write failed: {error}");
    }
}

/// Explains the `hash password` step in this lesson.
///
/// What it does: turns a plain password into an Argon2 password hash.
/// Why it matters: Passwords should never be stored directly, even in a course example.
#[cfg(feature = "ssr")]
async fn hash_password(password: &str) -> Result<String, ServerFnError> {
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
    use argon2::Argon2;

    let password = password.to_owned();
    // Argon2 is slow on purpose, so it runs on the blocking pool and never
    // stalls the async workers that serve other requests.
    tokio::task::spawn_blocking(move || {
        // A fresh random salt for every password, stored inside the hash string.
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| internal("hash password", error))?
    .map_err(|error| internal("hash password", error))
}

/// Explains the `verify password` step in this lesson.
///
/// What it does: checks a login password against the stored Argon2 hash.
/// Why it matters: Authentication is only useful if the server can prove the submitted password matches the hash.
#[cfg(feature = "ssr")]
async fn verify_password(password: &str, password_hash: &str) -> Result<bool, ServerFnError> {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;

    let password = password.to_owned();
    let password_hash = password_hash.to_owned();
    tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&password_hash).map_err(|error| error.to_string())?;
        Ok::<bool, String>(
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok(),
        )
    })
    .await
    .map_err(|error| internal("verify password", error))?
    .map_err(|error| internal("verify password", error))
}

/// A real hash of a throwaway password. Login verifies against it when the
/// email is unknown, so an unknown email costs the same time as a wrong
/// password and the response time does not reveal which emails exist.
#[cfg(feature = "ssr")]
async fn dummy_hash() -> Result<&'static str, ServerFnError> {
    static DUMMY: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();
    DUMMY
        .get_or_try_init(|| async { hash_password("not-a-real-password").await })
        .await
        .map(String::as_str)
}

/// Explains the `sign user` step in this lesson.
///
/// What it does: creates the production capstone JWT for a signed-in user.
/// Why it matters: The capstone reuses the same session idea in a larger production-style app.
#[cfg(feature = "ssr")]
fn sign_user(user: &ProdUser) -> Result<String, ServerFnError> {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let exp = (chrono::Utc::now() + chrono::Duration::hours(12)).timestamp() as usize;
    let claims = ProdClaims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.clone(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(prod_secret()?.as_bytes()),
    )
    .map_err(|error| internal("sign token", error))
}

/// Explains the `decode user` step in this lesson.
///
/// What it does: verifies the production capstone JWT and extracts its claims.
/// Why it matters: Every protected server action must verify identity before it touches user data.
#[cfg(feature = "ssr")]
fn decode_user(token: &str) -> Result<ProdClaims, ServerFnError> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    decode::<ProdClaims>(
        token,
        &DecodingKey::from_secret(prod_secret()?.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| ServerFnError::new("login required"))
}

/// Explains the `write cookie` step in this lesson.
///
/// What it does: writes the production session token into an HttpOnly response cookie.
/// Why it matters: The capstone keeps session handling explicit and auditable.
#[cfg(feature = "ssr")]
fn write_cookie(token: &str) -> Result<(), ServerFnError> {
    use axum::http::{header, HeaderValue};

    let response = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("ResponseOptions missing"))?;
    let cookie = guard::session_cookie(token, secure_cookies());
    response.append_header(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|error| internal("write cookie", error))?,
    );
    Ok(())
}

/// Explains the `clear prod cookie` step in this lesson.
///
/// What it does: expires the production session cookie during logout.
/// Why it matters: Clearing the cookie is the server-side half of logging out.
#[cfg(feature = "ssr")]
fn clear_prod_cookie() -> Result<(), ServerFnError> {
    use axum::http::{header, HeaderValue};

    let response = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("ResponseOptions missing"))?;
    response.append_header(
        header::SET_COOKIE,
        HeaderValue::from_str(&guard::cleared_cookie(secure_cookies()))
            .map_err(|error| internal("clear cookie", error))?,
    );
    Ok(())
}

/// Explains the `prod cookie` step in this lesson.
///
/// What it does: reads the production auth cookie from the incoming request headers.
/// Why it matters: Protected production routes need request-derived identity, not UI-only state.
#[cfg(feature = "ssr")]
fn prod_cookie() -> Option<String> {
    use axum::http::request::Parts;

    let parts = use_context::<Parts>()?;
    let raw = parts
        .headers
        .get(axum::http::header::COOKIE)?
        .to_str()
        .ok()?;
    raw.split(';').find_map(|cookie| {
        let mut pieces = cookie.trim().splitn(2, '=');
        match (pieces.next(), pieces.next()) {
            (Some("prod_session"), Some(value)) => Some(value.to_string()),
            _ => None,
        }
    })
}

/// Explains the `current user id` step in this lesson.
///
/// What it does: derives the current user id from the verified session token.
/// Why it matters: Database queries use this guard so users only read or mutate their own records.
#[cfg(feature = "ssr")]
async fn current_user_id() -> Result<Uuid, ServerFnError> {
    let token = prod_cookie().ok_or_else(|| ServerFnError::new("login required"))?;
    let claims = decode_user(&token)?;
    Uuid::parse_str(&claims.sub).map_err(|_| ServerFnError::new("login required"))
}

// ============================================================================
// LESSON 3: Server Functions And Trusted Work
// ============================================================================

/// Explains the `prod register` step in this lesson.
///
/// What it does: runs the `prod register` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn prod_register(
    name: String,
    email: String,
    password: String,
) -> Result<(), ServerFnError> {
    let (name, email) =
        guard::validate_registration(&name, &email, &password).map_err(ServerFnError::new)?;

    #[cfg(feature = "ssr")]
    {
        let pool = pool()?;
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO production_users (id, name, email, password_hash, role) VALUES ($1, $2, $3, $4, 'user')",
        )
        .bind(id)
        .bind(name)
        .bind(email)
        .bind(hash_password(&password).await?)
        .execute(&pool)
        .await
        .map_err(|err| match &err {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                ServerFnError::new("that email is already registered")
            }
            _ => internal("register", err),
        })?;
        log_event(&pool, Some(id), "info", "Account created").await;
    }
    Ok(())
}

/// Explains the `prod login` step in this lesson.
///
/// What it does: checks the password, records the attempt, and sets the session cookie.
/// Why it matters: Login is the most attacked endpoint of any app. It uses one
/// error for every failure, spends the same time on an unknown email, and
/// stops guessing after five failures per email in fifteen minutes.
#[server]
pub async fn prod_login(email: String, password: String) -> Result<ProdUserInfo, ServerFnError> {
    let email = email.trim().to_lowercase();

    #[cfg(feature = "ssr")]
    {
        const FAILED: &str = "invalid credentials";
        let now = std::time::Instant::now();
        if !limiter().allowed(&email, now) {
            return Err(ServerFnError::new(
                "too many failed attempts, try again in a few minutes",
            ));
        }
        // Oversized input is refused before it reaches the slow hash.
        if email.chars().count() > guard::EMAIL_MAX
            || password.chars().count() > guard::PASSWORD_MAX
        {
            limiter().record_failure(&email, now);
            return Err(ServerFnError::new(FAILED));
        }

        let pool = pool()?;
        let found = sqlx::query_as::<_, ProdUser>(
            "SELECT id, name, email, password_hash, role FROM production_users WHERE email = $1",
        )
        .bind(&email)
        .fetch_optional(&pool)
        .await
        .map_err(|error| internal("login lookup", error))?;

        let hash = match &found {
            Some(user) => user.password_hash.as_str(),
            None => dummy_hash().await?,
        };
        let password_ok = verify_password(&password, hash).await?;
        let user = match found {
            Some(user) if password_ok => user,
            other => {
                limiter().record_failure(&email, std::time::Instant::now());
                if let Some(user) = other {
                    log_event(&pool, Some(user.id), "warn", "Failed sign-in attempt").await;
                }
                return Err(ServerFnError::new(FAILED));
            }
        };

        limiter().record_success(&email);
        write_cookie(&sign_user(&user)?)?;
        log_event(&pool, Some(user.id), "info", "Signed in").await;
        return Ok(ProdUserInfo {
            id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
        });
    }

    #[cfg(not(feature = "ssr"))]
    {
        Ok(ProdUserInfo {
            id: Uuid::new_v4(),
            name: "Local user".into(),
            email,
            role: "user".into(),
        })
    }
}

/// Explains the `prod logout` step in this lesson.
///
/// What it does: runs the `prod logout` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn prod_logout() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        if let (Ok(user_id), Ok(pool)) = (current_user_id().await, pool()) {
            log_event(&pool, Some(user_id), "info", "Signed out").await;
        }
        clear_prod_cookie()?;
    }
    Ok(())
}

/// Explains the `prod me` step in this lesson.
///
/// What it does: runs the `prod me` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn prod_me() -> Result<Option<ProdUserInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        // No cookie, a bad signature, or an expired token all mean "nobody".
        let Ok(id) = current_user_id().await else {
            return Ok(None);
        };
        return sqlx::query_as::<_, ProdUserInfo>(
            "SELECT id, name, email, role FROM production_users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&pool()?)
        .await
        .map_err(|error| internal("load session user", error));
    }

    #[cfg(not(feature = "ssr"))]
    {
        Ok(None)
    }
}

/// Explains the `prod notes` step in this lesson.
///
/// What it does: runs the `prod notes` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn prod_notes() -> Result<Vec<ProdNote>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user_id = current_user_id().await?;
        return sqlx::query_as::<_, ProdNote>(
            "SELECT id, user_id, title, body, updated_at FROM production_notes WHERE user_id = $1 ORDER BY updated_at DESC LIMIT 200",
        )
        .bind(user_id)
        .fetch_all(&pool()?)
        .await
        .map_err(|error| internal("list notes", error));
    }

    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// Explains the `prod create note` step in this lesson.
///
/// What it does: runs the `prod create note` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn prod_create_note(title: String, body: String) -> Result<ProdNote, ServerFnError> {
    let (title, body) = guard::validate_note(&title, &body).map_err(ServerFnError::new)?;

    #[cfg(feature = "ssr")]
    {
        let user_id = current_user_id().await?;
        let pool = pool()?;
        let note = sqlx::query_as::<_, ProdNote>(
            "INSERT INTO production_notes (id, user_id, title, body) VALUES ($1, $2, $3, $4) RETURNING id, user_id, title, body, updated_at",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(title)
        .bind(body)
        .fetch_one(&pool)
        .await
        .map_err(|error| internal("create note", error))?;
        log_event(&pool, Some(user_id), "info", "Note created").await;
        return Ok(note);
    }

    #[cfg(not(feature = "ssr"))]
    {
        Ok(ProdNote {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            title,
            body,
            updated_at: chrono::Utc::now(),
        })
    }
}

/// Explains the `prod delete note` step in this lesson.
///
/// What it does: runs the `prod delete note` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn prod_delete_note(id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let note_id = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("invalid note id"))?;
        let user_id = current_user_id().await?;
        let pool = pool()?;
        let result = sqlx::query("DELETE FROM production_notes WHERE id = $1 AND user_id = $2")
            .bind(note_id)
            .bind(user_id)
            .execute(&pool)
            .await
            .map_err(|error| internal("delete note", error))?;
        // Zero rows means the note does not exist or belongs to someone else.
        // Both look the same to the caller, and neither is reported as success.
        if result.rows_affected() == 0 {
            return Err(ServerFnError::new("note not found"));
        }
        log_event(&pool, Some(user_id), "info", "Note deleted").await;
    }
    Ok(())
}

/// Explains the `prod activity` step in this lesson.
///
/// What it does: runs the `prod activity` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn prod_activity() -> Result<Vec<ActivityEvent>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user_id = current_user_id().await?;
        return sqlx::query_as::<_, ActivityEvent>(
            "SELECT id, message, severity, to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS \"UTC\"') AS at \
             FROM activity_events WHERE user_id = $1 ORDER BY created_at DESC, id DESC LIMIT 50",
        )
        .bind(user_id)
        .fetch_all(&pool()?)
        .await
        .map_err(|error| internal("list activity", error));
    }

    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// Explains the `prod stats` step in this lesson.
///
/// What it does: counts this user's notes and activity rows in the database.
/// Why it matters: A dashboard number must be measured, never typed in.
#[server]
pub async fn prod_stats() -> Result<DashboardCounts, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user_id = current_user_id().await?;
        let (notes, events) = sqlx::query_as::<_, (i64, i64)>(
            "SELECT (SELECT COUNT(*) FROM production_notes WHERE user_id = $1), \
                    (SELECT COUNT(*) FROM activity_events WHERE user_id = $1)",
        )
        .bind(user_id)
        .fetch_one(&pool()?)
        .await
        .map_err(|error| internal("load stats", error))?;
        return Ok(DashboardCounts { notes, events });
    }

    #[cfg(not(feature = "ssr"))]
    {
        Ok(DashboardCounts {
            notes: 0,
            events: 0,
        })
    }
}

/// Explains the `app nav` step in this lesson.
///
/// What it does: renders the `AppNav` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn AppNav() -> impl IntoView {
    let session = expect_context::<Session>();
    let theme = expect_context::<ThemeContext>();

    view! {
        <nav style="display: flex; justify-content: space-between; align-items: center; gap: 16px; margin-bottom: 24px; padding-bottom: 16px; border-bottom: 1px solid #d9dee8;">
            <div>
                <strong>{MODULE_TITLE}</strong>
                <p style="margin: 4px 0 0; color: #6b7280;">"Production app shell, auth, CRUD, and operations."</p>
            </div>
            <div style="display: flex; gap: 10px; align-items: center; flex-wrap: wrap;">
                <A href="/">"Home"</A>
                <A href="/dashboard">"Dashboard"</A>
                <A href="/notes">"Notes"</A>
                <A href="/activity">"Activity"</A>
                <A href="/settings">"Settings"</A>
                <button on:click=move |_| {
                    theme.update(|value| *value = if *value == Theme::Light { Theme::Dark } else { Theme::Light });
                }>{move || if theme.get() == Theme::Light { "Dark" } else { "Light" }}</button>
                // The session is a resource, so it is read inside Suspense.
                <Suspense fallback=|| ()>
                    {move || match session.user() {
                        Some(user) => view! { <span>{user.email}</span> <LogoutButton/> }.into_any(),
                        None => view! { <A href="/login">"Login"</A> <A href="/register">"Register"</A> }.into_any(),
                    }}
                </Suspense>
            </div>
        </nav>
    }
}

/// Explains the `home page` step in this lesson.
///
/// What it does: renders the `HomePage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
            <Title text="Production Home"/>
            <Meta name="description" content="Production-ready Leptos capstone home"/>
            <h1>"Production capstone"</h1>
            <p>"This final module combines database-backed auth, protected routes, notes CRUD, metadata, error boundaries, and operational endpoints."</p>
        </section>
    }
}

/// Explains the `login page` step in this lesson.
///
/// What it does: renders the `LoginPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn LoginPage() -> impl IntoView {
    let session = expect_context::<Session>();
    let navigate = use_navigate();
    let action = ServerAction::<ProdLogin>::new();
    let pending = action.pending();
    let value = action.value();

    Effect::new(move |_| {
        if let Some(Ok(user)) = value.get() {
            session.set(Some(user));
            navigate("/dashboard", Default::default());
        }
    });

    view! {
        <section style="max-width: 520px;">
            <Title text="Login"/>
            <h1>"Login"</h1>
            <div style="display: grid; gap: 10px;">
                <ActionForm action=action>
                    <label for="login-email">"Email"</label>
                    <input id="login-email" name="email" type="email" autocomplete="username" required maxlength="254"/>
                    <label for="login-password">"Password"</label>
                    <input id="login-password" name="password" type="password" autocomplete="current-password" required maxlength="128"/>
                    <button type="submit" disabled=move || pending.get()>{move || if pending.get() { "Signing in..." } else { "Login" }}</button>
                </ActionForm>
            </div>
            <p role="alert" style="color: #b91c1c;">{move || value.get().and_then(|result| result.err()).map(|error| error.to_string())}</p>
        </section>
    }
}

/// Explains the `register page` step in this lesson.
///
/// What it does: renders the `RegisterPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn RegisterPage() -> impl IntoView {
    let action = ServerAction::<ProdRegister>::new();
    let pending = action.pending();
    let value = action.value();

    view! {
        <section style="max-width: 520px;">
            <Title text="Register"/>
            <h1>"Register"</h1>
            <div style="display: grid; gap: 10px;">
                <ActionForm action=action>
                    <label for="reg-name">"Name"</label>
                    <input id="reg-name" name="name" autocomplete="name" required minlength="2" maxlength="80"/>
                    <label for="reg-email">"Email"</label>
                    <input id="reg-email" name="email" type="email" autocomplete="email" required maxlength="254"/>
                    <label for="reg-password">"Password (8 to 128 characters)"</label>
                    <input id="reg-password" name="password" type="password" autocomplete="new-password" required minlength="8" maxlength="128"/>
                    <button type="submit" disabled=move || pending.get()>{move || if pending.get() { "Creating..." } else { "Register" }}</button>
                </ActionForm>
            </div>
            <p role="status">{move || value.get().map(|result| match result {
                Ok(()) => view! { <span style="color: #166534;">"Account created. Login next."</span> }.into_any(),
                Err(error) => view! { <span style="color: #b91c1c;">{error.to_string()}</span> }.into_any(),
            })}</p>
        </section>
    }
}

/// Explains the `logout button` step in this lesson.
///
/// What it does: renders the `LogoutButton` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn LogoutButton() -> impl IntoView {
    let session = expect_context::<Session>();
    let navigate = use_navigate();

    view! {
        <button on:click=move |_| {
            let navigate = navigate.clone();
            leptos::task::spawn_local(async move {
                // Only say "logged out" when the server confirmed it cleared the cookie.
                if prod_logout().await.is_ok() {
                    session.set(None);
                    navigate("/", Default::default());
                }
            });
        }>"Logout"</button>
    }
}

/// Explains the `protected` step in this lesson.
///
/// What it does: renders the `Protected` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Protected(children: ChildrenFn) -> impl IntoView {
    let session = expect_context::<Session>();

    view! {
        <Suspense fallback=|| view! { <p role="status">"Checking your session..."</p> }>
            {move || if session.user().is_some() {
                children().into_any()
            } else {
                view! {
                    <section style="border: 1px solid #fde68a; border-radius: 8px; padding: 20px; background: #fffbeb; color: #111827;">
                        <h1>"Login required"</h1>
                        <p>"Sign in to see this page."</p>
                        <A href="/login">"Go to login"</A>
                    </section>
                }.into_any()
            }}
        </Suspense>
    }
}

/// Explains the `dashboard page` step in this lesson.
///
/// What it does: renders the `DashboardPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn DashboardPage() -> impl IntoView {
    let stats = Resource::new(|| (), |_| prod_stats());

    view! {
        <Protected>
            <Title text="Dashboard"/>
            <section style="display: grid; gap: 16px;">
                <h1>"Dashboard"</h1>
                <Suspense fallback=|| view! { <p role="status">"Loading numbers..."</p> }>
                    {move || stats.get().map(|result| match result {
                        Ok(stats) => view! {
                            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px;">
                                <Metric label="Your notes" value=stats.notes.to_string()/>
                                <Metric label="Your activity events" value=stats.events.to_string()/>
                            </div>
                        }.into_any(),
                        Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
                    })}
                </Suspense>
                <p>"Server health is served at /health and database readiness at /ready. A deployment platform checks those, not this page."</p>
            </section>
        </Protected>
    }
}

/// Explains the `metric` step in this lesson.
///
/// What it does: renders one labelled number.
/// Why it matters: The value is a String that the caller measured, not a constant typed into the view.
#[component]
fn Metric(label: &'static str, value: String) -> impl IntoView {
    view! {
        <article style="background: #f6f7fb; color: #111827; border-radius: 8px; padding: 16px;">
            <span>{label}</span>
            <strong style="display: block; font-size: 1.3rem;">{value}</strong>
        </article>
    }
}

/// Explains the `notes page` step in this lesson.
///
/// What it does: renders the `NotesPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn NotesPage() -> impl IntoView {
    let notes = Resource::new(|| (), |_| prod_notes());
    let create = ServerAction::<ProdCreateNote>::new();
    let pending = create.pending();
    let delete_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if let Some(Ok(_)) = create.value().get() {
            notes.refetch();
        }
    });

    view! {
        <Protected>
            <Title text="Notes"/>
            <section style="display: grid; gap: 16px;">
                <h1>"Notes"</h1>
                <div style="display: grid; gap: 8px;">
                    <ActionForm action=create>
                        <label for="note-title">"Title"</label>
                        <input id="note-title" name="title" required maxlength="200"/>
                        <label for="note-body">"Body"</label>
                        <textarea id="note-body" name="body" maxlength="10000"></textarea>
                        <button type="submit" disabled=move || pending.get()>{move || if pending.get() { "Saving..." } else { "Create note" }}</button>
                    </ActionForm>
                </div>
                <p role="alert" style="color: #b91c1c;">
                    {move || create.value().get().and_then(|result| result.err()).map(|error| error.to_string())}
                    {move || delete_error.get()}
                </p>
                <Suspense fallback=|| view! { <p>"Loading notes..."</p> }>
                    {move || notes.get().map(|result| match result {
                        Ok(items) => view! {
                            <div style="display: grid; gap: 8px;">
                                <For
                                    each=move || items.clone()
                                    key=|note| note.id
                                    children=move |note| view! {
                                        <article style="background: #f6f7fb; color: #111827; border-radius: 8px; padding: 14px;">
                                            <h2>{note.title.clone()}</h2>
                                            <p>{note.body.clone()}</p>
                                            <small>{note.updated_at.format("%Y-%m-%d %H:%M UTC").to_string()}</small>
                                            <button on:click=move |_| {
                                                leptos::task::spawn_local(async move {
                                                    // Show a failed delete instead of pretending it worked.
                                                    delete_error.set(prod_delete_note(note.id.to_string()).await.err().map(|error| error.to_string()));
                                                    notes.refetch();
                                                });
                                            }>"Delete"</button>
                                        </article>
                                    }
                                />
                            </div>
                        }.into_any(),
                        Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
                    })}
                </Suspense>
            </section>
        </Protected>
    }
}

/// Explains the `profile page` step in this lesson.
///
/// What it does: renders the `ProfilePage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ProfilePage() -> impl IntoView {
    let session = expect_context::<Session>();

    view! {
        <Protected>
            <Title text="Profile"/>
            {move || session.user().map(|user| view! {
                <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
                    <h1>"Profile"</h1>
                    <p>"Name: " {user.name}</p>
                    <p>"Email: " {user.email}</p>
                    <p>"Role: " {user.role}</p>
                </section>
            })}
        </Protected>
    }
}

/// Explains the `settings page` step in this lesson.
///
/// What it does: renders the `SettingsPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn SettingsPage() -> impl IntoView {
    let theme = expect_context::<ThemeContext>();

    view! {
        <Protected>
            <Title text="Settings"/>
            <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
                <h1>"Settings"</h1>
                <label>
                    <input
                        type="checkbox"
                        prop:checked=move || theme.get() == Theme::Dark
                        on:change=move |_| {
                            theme.update(|value| *value = if *value == Theme::Light { Theme::Dark } else { Theme::Light });
                        }
                    />
                    " Dark mode"
                </label>
            </section>
        </Protected>
    }
}

/// Explains the `activity page` step in this lesson.
///
/// What it does: renders the `ActivityPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn ActivityPage() -> impl IntoView {
    let events = Resource::new(|| (), |_| prod_activity());

    view! {
        <Protected>
            <Title text="Activity"/>
            <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
                <h1>"Activity feed"</h1>
                <Suspense fallback=|| view! { <p>"Loading activity..."</p> }>
                    {move || events.get().map(|result| match result {
                        Ok(events) if events.is_empty() => view! { <p>"No activity yet."</p> }.into_any(),
                        Ok(events) => view! {
                            <ul>
                                {events.into_iter().map(|event| view! {
                                    <li><strong>{event.severity}</strong> " - " {event.message} " (" {event.at} ")"</li>
                                }).collect_view()}
                            </ul>
                        }.into_any(),
                        Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
                    })}
                </Suspense>
            </section>
        </Protected>
    }
}

/// Explains the `not found page` step in this lesson.
///
/// What it does: renders the `NotFoundPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn NotFoundPage() -> impl IntoView {
    #[cfg(feature = "ssr")]
    if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
        response.set_status(axum::http::StatusCode::NOT_FOUND);
    }

    view! {
        <section>
            <Title text="404"/>
            <h1>"404"</h1>
            <p>"The requested production route does not exist."</p>
        </section>
    }
}
