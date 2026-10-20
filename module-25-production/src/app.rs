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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ProdUser {
    id: Uuid,
    name: String,
    email: String,
    password_hash: String,
    role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
pub struct ActivityEvent {
    id: u32,
    message: String,
    severity: String,
    at: String,
}

pub type AuthContext = RwSignal<Option<ProdUserInfo>>;
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
    let auth = RwSignal::new(None::<ProdUserInfo>);
    let theme = RwSignal::new(Theme::Light);
    provide_context(auth);
    provide_context(theme);

    let me = Resource::new(|| (), |_| prod_me());
    Effect::new(move |_| {
        if let Some(Ok(user)) = me.get() {
            auth.set(user);
        }
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
fn prod_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "module-25-production-secret".into())
}

/// Explains the `hash password` step in this lesson.
///
/// What it does: turns a plain password into an Argon2 password hash.
/// Why it matters: Passwords should never be stored directly, even in a course example.
#[cfg(feature = "ssr")]
fn hash_password(password: &str) -> Result<String, ServerFnError> {
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
    use argon2::Argon2;

    // A fresh random salt for every password, stored inside the resulting hash string.
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(ServerFnError::new)
}

/// Explains the `verify password` step in this lesson.
///
/// What it does: checks a login password against the stored Argon2 hash.
/// Why it matters: Authentication is only useful if the server can prove the submitted password matches the hash.
#[cfg(feature = "ssr")]
fn verify_password(password: &str, password_hash: &str) -> Result<bool, ServerFnError> {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;

    let parsed_hash = PasswordHash::new(password_hash).map_err(ServerFnError::new)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
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
        &EncodingKey::from_secret(prod_secret().as_bytes()),
    )
    .map_err(ServerFnError::new)
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
        &DecodingKey::from_secret(prod_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(ServerFnError::new)
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
    let cookie = format!("prod_session={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age=43200");
    response.append_header(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(ServerFnError::new)?,
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
        HeaderValue::from_static("prod_session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0"),
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
    Uuid::parse_str(&claims.sub).map_err(ServerFnError::new)
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
    let name = name.trim().to_string();
    let email = email.trim().to_lowercase();
    if name.len() < 2 || password.len() < 8 || !email.contains('@') {
        return Err(ServerFnError::new(
            "valid name, email, and 8 character password required",
        ));
    }

    #[cfg(feature = "ssr")]
    {
        sqlx::query(
            "INSERT INTO production_users (id, name, email, password_hash, role) VALUES ($1, $2, $3, $4, 'user')",
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(email)
        .bind(hash_password(&password)?)
        .execute(&pool()?)
        .await
        .map_err(|err| match &err {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                ServerFnError::new("that email is already registered")
            }
            _ => {
                tracing::error!("register failed: {err}");
                ServerFnError::new("could not create the account, please try again")
            }
        })?;
    }
    Ok(())
}

/// Explains the `prod login` step in this lesson.
///
/// What it does: runs the `prod login` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn prod_login(email: String, password: String) -> Result<ProdUserInfo, ServerFnError> {
    let email = email.trim().to_lowercase();

    #[cfg(feature = "ssr")]
    {
        let user = sqlx::query_as::<_, ProdUser>(
            "SELECT id, name, email, password_hash, role FROM production_users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&pool()?)
        .await
        .map_err(ServerFnError::new)?
        .ok_or_else(|| ServerFnError::new("invalid credentials"))?;

        if !verify_password(&password, &user.password_hash)? {
            return Err(ServerFnError::new("invalid credentials"));
        }

        write_cookie(&sign_user(&user)?)?;
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
    clear_prod_cookie()?;
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
        let Some(token) = prod_cookie() else {
            return Ok(None);
        };
        let claims = match decode_user(&token) {
            Ok(claims) => claims,
            Err(_) => return Ok(None),
        };
        let id = Uuid::parse_str(&claims.sub).map_err(ServerFnError::new)?;
        let user = sqlx::query_as::<_, ProdUser>(
            "SELECT id, name, email, password_hash, role FROM production_users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&pool()?)
        .await
        .map_err(ServerFnError::new)?;
        return Ok(user.map(|user| ProdUserInfo {
            id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
        }));
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
            "SELECT id, user_id, title, body, updated_at FROM production_notes WHERE user_id = $1 ORDER BY updated_at DESC",
        )
        .bind(user_id)
        .fetch_all(&pool()?)
        .await
        .map_err(ServerFnError::new);
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
    if title.trim().is_empty() {
        return Err(ServerFnError::new("title is required"));
    }

    #[cfg(feature = "ssr")]
    {
        let user_id = current_user_id().await?;
        return sqlx::query_as::<_, ProdNote>(
            "INSERT INTO production_notes (id, user_id, title, body) VALUES ($1, $2, $3, $4) RETURNING id, user_id, title, body, updated_at",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(title)
        .bind(body)
        .fetch_one(&pool()?)
        .await
        .map_err(ServerFnError::new);
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
        let note_id = Uuid::parse_str(&id).map_err(ServerFnError::new)?;
        let user_id = current_user_id().await?;
        sqlx::query("DELETE FROM production_notes WHERE id = $1 AND user_id = $2")
            .bind(note_id)
            .bind(user_id)
            .execute(&pool()?)
            .await
            .map_err(ServerFnError::new)?;
    }
    Ok(())
}

/// Explains the `prod activity` step in this lesson.
///
/// What it does: runs the `prod activity` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn prod_activity() -> Result<Vec<ActivityEvent>, ServerFnError> {
    Ok(vec![
        ActivityEvent {
            id: 1,
            message: "Health check passed".into(),
            severity: "ok".into(),
            at: chrono::Utc::now().format("%H:%M:%S").to_string(),
        },
        ActivityEvent {
            id: 2,
            message: "Notes cache refreshed".into(),
            severity: "info".into(),
            at: chrono::Utc::now().format("%H:%M:%S").to_string(),
        },
        ActivityEvent {
            id: 3,
            message: "Realtime channel idle".into(),
            severity: "info".into(),
            at: chrono::Utc::now().format("%H:%M:%S").to_string(),
        },
    ])
}

/// Explains the `app nav` step in this lesson.
///
/// What it does: renders the `AppNav` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn AppNav() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
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
                {move || match auth.get() {
                    Some(user) => view! { <span>{user.email}</span> <LogoutButton/> }.into_any(),
                    None => view! { <A href="/login">"Login"</A> <A href="/register">"Register"</A> }.into_any(),
                }}
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
    let auth = expect_context::<AuthContext>();
    let navigate = use_navigate();
    let action = ServerAction::<ProdLogin>::new();
    let pending = action.pending();
    let value = action.value();

    Effect::new(move |_| {
        if let Some(Ok(user)) = value.get() {
            auth.set(Some(user));
            navigate("/dashboard", Default::default());
        }
    });

    view! {
        <section style="max-width: 520px;">
            <Title text="Login"/>
            <h1>"Login"</h1>
            <div style="display: grid; gap: 10px;">
                <ActionForm action=action>
                    <input name="email" type="email" placeholder="Email"/>
                    <input name="password" type="password" placeholder="Password"/>
                    <button type="submit" disabled=move || pending.get()>{move || if pending.get() { "Signing in..." } else { "Login" }}</button>
                </ActionForm>
            </div>
            {move || value.get().and_then(|result| result.err()).map(|error| view! { <p style="color: #b91c1c;">{error.to_string()}</p> })}
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
                    <input name="name" placeholder="Name"/>
                    <input name="email" type="email" placeholder="Email"/>
                    <input name="password" type="password" placeholder="Password"/>
                    <button type="submit" disabled=move || pending.get()>{move || if pending.get() { "Creating..." } else { "Register" }}</button>
                </ActionForm>
            </div>
            {move || value.get().map(|result| match result {
                Ok(()) => view! { <p style="color: #166534;">"Account created. Login next."</p> }.into_any(),
                Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
            })}
        </section>
    }
}

/// Explains the `logout button` step in this lesson.
///
/// What it does: renders the `LogoutButton` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn LogoutButton() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let navigate = use_navigate();

    view! {
        <button on:click=move |_| {
            let navigate = navigate.clone();
            leptos::task::spawn_local(async move {
                let _ = prod_logout().await;
                auth.set(None);
                navigate("/", Default::default());
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
    let auth = expect_context::<AuthContext>();

    view! {
        {move || if auth.get().is_some() {
            children().into_any()
        } else {
            view! {
                <section style="border: 1px solid #fde68a; border-radius: 8px; padding: 20px; background: #fffbeb; color: #111827;">
                    <h1>"Login required"</h1>
                    <p>"This route is protected by AuthContext."</p>
                    <A href="/login">"Go to login"</A>
                </section>
            }.into_any()
        }}
    }
}

/// Explains the `dashboard page` step in this lesson.
///
/// What it does: renders the `DashboardPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn DashboardPage() -> impl IntoView {
    view! {
        <Protected>
            <Title text="Dashboard"/>
            <section style="display: grid; gap: 16px;">
                <h1>"Dashboard"</h1>
                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px;">
                    <Metric label="Health" value="ok"/>
                    <Metric label="Readiness" value="/ready"/>
                    <Metric label="Tracing" value="json"/>
                    <Metric label="Compression" value="gzip"/>
                </div>
            </section>
        </Protected>
    }
}

/// Explains the `metric` step in this lesson.
///
/// What it does: renders the `Metric` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Metric(label: &'static str, value: &'static str) -> impl IntoView {
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
                        <input name="title" placeholder="Title"/>
                        <textarea name="body" placeholder="Body"></textarea>
                        <button type="submit" disabled=move || pending.get()>{move || if pending.get() { "Saving..." } else { "Create note" }}</button>
                    </ActionForm>
                </div>
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
                                                    let _ = prod_delete_note(note.id.to_string()).await;
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
    let auth = expect_context::<AuthContext>();

    view! {
        <Protected>
            <Title text="Profile"/>
            {move || auth.get().map(|user| view! {
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
