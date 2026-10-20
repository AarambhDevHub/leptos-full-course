//! Module 11: Authentication.
//! A full-stack JWT and cookie auth lesson with SQLx, Argon2, and protected
//! Leptos routes.
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

const MODULE_TITLE: &str = "Module 11 - Authentication";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct User {
    id: Uuid,
    name: String,
    email: String,
    password_hash: String,
    role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    id: Uuid,
    name: String,
    email: String,
    role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    email: String,
    role: String,
    exp: usize,
}

pub type AuthContext = RwSignal<Option<UserInfo>>;

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
    let auth = RwSignal::new(None::<UserInfo>);
    provide_context(auth);

    let me = Resource::new(|| (), |_| get_me());
    Effect::new(move |_| {
        if let Some(Ok(user)) = me.get() {
            auth.set(user);
        }
    });

    view! {
        <Title text=MODULE_TITLE/>
        <Meta name="description" content="JWT cookie authentication in the Leptos 0.8 full course"/>
        <Router>
            <main style="max-width: 1040px; margin: 0 auto; padding: 32px; font-family: system-ui, sans-serif;">
                <Header/>
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/login") view=LoginPage/>
                    <Route path=path!("/register") view=RegisterPage/>
                    <Route path=path!("/me") view=MePage/>
                    <Route path=path!("/admin") view=AdminPage/>
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

/// Explains the `jwt secret` step in this lesson.
///
/// What it does: loads the JWT signing secret used by the authentication lesson.
/// Why it matters: JWT signing must be centralized so every token is created and verified with the same secret.
#[cfg(feature = "ssr")]
fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-only-module-11-secret".into())
}

/// Explains the `hash password` step in this lesson.
///
/// What it does: turns a plain password into an Argon2 password hash.
/// Why it matters: Passwords should never be stored directly, even in a course example.
#[cfg(feature = "ssr")]
fn hash_password(password: &str) -> Result<String, ServerFnError> {
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
    use argon2::Argon2;

    // A fresh random salt for every password: two users with the same password get different hashes,
    // and one precomputed table cannot attack all of them at once. The salt is stored inside the hash string.
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(ServerFnError::new)
}

#[cfg(feature = "ssr")]
fn verify_password(password: &str, password_hash: &str) -> Result<bool, ServerFnError> {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;

    let parsed_hash = PasswordHash::new(password_hash).map_err(ServerFnError::new)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Explains the `make token` step in this lesson.
///
/// What it does: creates a signed JWT that represents the authenticated user.
/// Why it matters: A signed token lets the browser keep a session without storing server state in memory.
#[cfg(feature = "ssr")]
fn make_token(user: &User) -> Result<String, ServerFnError> {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.clone(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(ServerFnError::new)
}

/// Explains the `decode token` step in this lesson.
///
/// What it does: verifies a JWT and converts it back into trusted claims.
/// Why it matters: Protected pages need a trusted way to turn a cookie back into user identity.
#[cfg(feature = "ssr")]
fn decode_token(token: &str) -> Result<Claims, ServerFnError> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(ServerFnError::new)
}

/// Explains the `set cookie` step in this lesson.
///
/// What it does: writes the auth session token into an HttpOnly response cookie.
/// Why it matters: HttpOnly cookies protect the token from normal browser JavaScript access.
#[cfg(feature = "ssr")]
fn set_cookie(token: &str) -> Result<(), ServerFnError> {
    use axum::http::{header, HeaderValue};

    let response = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("ResponseOptions missing"))?;
    // HttpOnly keeps scripts from reading the token. Add `Secure` as well when the site is served over HTTPS.
    let cookie = format!("auth_session={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age=86400");
    response.append_header(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(ServerFnError::new)?,
    );
    Ok(())
}

/// Explains the `clear cookie` step in this lesson.
///
/// What it does: expires the auth session cookie during logout.
/// Why it matters: Logout should invalidate the browser session from the server response.
#[cfg(feature = "ssr")]
fn clear_cookie() -> Result<(), ServerFnError> {
    use axum::http::{header, HeaderValue};

    let response = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("ResponseOptions missing"))?;
    response.append_header(
        header::SET_COOKIE,
        HeaderValue::from_static("auth_session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0"),
    );
    Ok(())
}

/// Explains the `auth cookie` step in this lesson.
///
/// What it does: reads the auth cookie from the incoming request headers.
/// Why it matters: Server functions cannot trust client state; they read identity from the request.
#[cfg(feature = "ssr")]
fn auth_cookie() -> Option<String> {
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
            (Some("auth_session"), Some(value)) => Some(value.to_string()),
            _ => None,
        }
    })
}

// ============================================================================
// LESSON 3: Server Functions And Trusted Work
// ============================================================================

/// Explains the `register` step in this lesson.
///
/// What it does: runs the `register` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn register(name: String, email: String, password: String) -> Result<(), ServerFnError> {
    let name = name.trim().to_string();
    let email = email.trim().to_lowercase();
    if name.len() < 2 {
        return Err(ServerFnError::new("name must be at least 2 characters"));
    }
    if !email.contains('@') || password.len() < 8 {
        return Err(ServerFnError::new(
            "email must be valid and password must be at least 8 characters",
        ));
    }

    #[cfg(feature = "ssr")]
    {
        let password_hash = hash_password(&password)?;
        sqlx::query(
            "INSERT INTO users (id, name, email, password_hash, role) VALUES ($1, $2, $3, $4, 'user')",
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(email)
        .bind(password_hash)
        .execute(&pool()?)
        .await
        .map_err(|err| match &err {
            // Tell the user what they can fix, and keep database details (table and constraint names) on the server.
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

/// Explains the `login` step in this lesson.
///
/// What it does: runs the `login` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn login(email: String, password: String) -> Result<UserInfo, ServerFnError> {
    let email = email.trim().to_lowercase();

    #[cfg(feature = "ssr")]
    {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, email, password_hash, role FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&pool()?)
        .await
        .map_err(ServerFnError::new)?
        .ok_or_else(|| ServerFnError::new("invalid credentials"))?;

        if !verify_password(&password, &user.password_hash)? {
            return Err(ServerFnError::new("invalid credentials"));
        }

        let token = make_token(&user)?;
        set_cookie(&token)?;
        return Ok(UserInfo {
            id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
        });
    }

    #[cfg(not(feature = "ssr"))]
    {
        Ok(UserInfo {
            id: Uuid::new_v4(),
            name: "Hydrated user".into(),
            email,
            role: "user".into(),
        })
    }
}

/// Explains the `logout` step in this lesson.
///
/// What it does: runs the `logout` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    clear_cookie()?;
    Ok(())
}

/// Explains the `get me` step in this lesson.
///
/// What it does: runs the `get me` operation on the server and returns typed data to the client.
/// Why it matters: It demonstrates how Leptos keeps trusted work on the server while preserving a Rust function call shape in the UI.
#[server]
pub async fn get_me() -> Result<Option<UserInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let Some(token) = auth_cookie() else {
            return Ok(None);
        };
        let claims = match decode_token(&token) {
            Ok(claims) => claims,
            Err(_) => return Ok(None),
        };
        let id = Uuid::parse_str(&claims.sub).map_err(ServerFnError::new)?;
        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, email, password_hash, role FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&pool()?)
        .await
        .map_err(ServerFnError::new)?;

        return Ok(user.map(|user| UserInfo {
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

/// Explains the `header` step in this lesson.
///
/// What it does: renders the `Header` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn Header() -> impl IntoView {
    let auth = expect_context::<AuthContext>();

    view! {
        <nav style="display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-bottom: 24px; padding-bottom: 16px; border-bottom: 1px solid #d9dee8;">
            <div>
                <strong>{MODULE_TITLE}</strong>
                <p style="margin: 4px 0 0; color: #56616f;">"JWT claims, HttpOnly cookies, and protected Leptos routes."</p>
            </div>
            <div style="display: flex; align-items: center; gap: 12px;">
                <A href="/">"Home"</A>
                <A href="/me">"Me"</A>
                <A href="/admin">"Admin"</A>
                {move || match auth.get() {
                    Some(user) => view! {
                        <span>{user.email}</span>
                        <LogoutButton/>
                    }.into_any(),
                    None => view! {
                        <A href="/login">"Login"</A>
                        <A href="/register">"Register"</A>
                    }.into_any(),
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
            <h1>"Authentication flow"</h1>
            <p>"Register creates a user row with an Argon2 password hash. Login verifies the hash, signs a JWT, and stores it in an HttpOnly cookie. Protected pages read an AuthContext initialized from get_me()."</p>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 12px;">
                <AuthStep title="1. Register" body="Insert a user with a hashed password and default user role."/>
                <AuthStep title="2. Login" body="Verify password, sign claims, set auth_session cookie."/>
                <AuthStep title="3. Load Me" body="Read cookie from request headers and hydrate auth state."/>
                <AuthStep title="4. Protect" body="Render profile/admin only when the role is authorized."/>
            </div>
        </section>
    }
}

/// Explains the `auth step` step in this lesson.
///
/// What it does: renders the `AuthStep` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn AuthStep(title: &'static str, body: &'static str) -> impl IntoView {
    view! {
        <article style="background: #f6f7fb; border-radius: 8px; padding: 16px;">
            <h2 style="font-size: 1rem;">{title}</h2>
            <p>{body}</p>
        </article>
    }
}

/// Explains the `register page` step in this lesson.
///
/// What it does: renders the `RegisterPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn RegisterPage() -> impl IntoView {
    let action = ServerAction::<Register>::new();
    let pending = action.pending();
    let value = action.value();

    view! {
        <section style="max-width: 520px;">
            <h1>"Create account"</h1>
            <div style="display: grid; gap: 10px;">
                <ActionForm action=action>
                    <input name="name" placeholder="Name" autocomplete="name"/>
                    <input name="email" type="email" placeholder="Email" autocomplete="email"/>
                    <input name="password" type="password" placeholder="Password" autocomplete="new-password"/>
                    <button type="submit" disabled=move || pending.get()>
                        {move || if pending.get() { "Creating..." } else { "Register" }}
                    </button>
                </ActionForm>
            </div>
            {move || value.get().map(|result| match result {
                Ok(()) => view! { <p style="color: #166534;">"Account created. You can log in now."</p> }.into_any(),
                Err(error) => view! { <p style="color: #b91c1c;">{error.to_string()}</p> }.into_any(),
            })}
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
    let action = ServerAction::<Login>::new();
    let pending = action.pending();
    let value = action.value();

    Effect::new(move |_| {
        if let Some(Ok(user)) = value.get() {
            auth.set(Some(user));
            navigate("/me", Default::default());
        }
    });

    view! {
        <section style="max-width: 520px;">
            <h1>"Login"</h1>
            <div style="display: grid; gap: 10px;">
                <ActionForm action=action>
                    <input name="email" type="email" placeholder="Email" autocomplete="email"/>
                    <input name="password" type="password" placeholder="Password" autocomplete="current-password"/>
                    <button type="submit" disabled=move || pending.get()>
                        {move || if pending.get() { "Checking..." } else { "Login" }}
                    </button>
                </ActionForm>
            </div>
            {move || value.get().and_then(|result| result.err()).map(|error| {
                view! { <p style="color: #b91c1c;">{error.to_string()}</p> }
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
                let _ = logout().await;
                auth.set(None);
                navigate("/", Default::default());
            });
        }>"Logout"</button>
    }
}

/// Explains the `me page` step in this lesson.
///
/// What it does: renders the `MePage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn MePage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if auth.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! {
        {move || match auth.get() {
            Some(user) => view! {
                <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
                    <h1>"Profile"</h1>
                    <dl>
                        <dt>"Name"</dt><dd>{user.name}</dd>
                        <dt>"Email"</dt><dd>{user.email}</dd>
                        <dt>"Role"</dt><dd>{user.role}</dd>
                    </dl>
                </section>
            }.into_any(),
            None => view! { <p>"Redirecting to login..."</p> }.into_any(),
        }}
    }
}

/// Explains the `admin page` step in this lesson.
///
/// What it does: renders the `AdminPage` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn AdminPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();

    view! {
        {move || match auth.get() {
            Some(user) if user.role == "admin" => view! {
                <section style="border: 1px solid #d9dee8; border-radius: 8px; padding: 20px;">
                    <h1>"Admin Console"</h1>
                    <p>"Only users whose JWT claims include role=admin can see this page."</p>
                </section>
            }.into_any(),
            Some(_) => view! {
                <section style="border: 1px solid #fecaca; border-radius: 8px; padding: 20px; background: #fef2f2;">
                    <h1>"403"</h1>
                    <p>"You are authenticated, but your role is not admin."</p>
                </section>
            }.into_any(),
            None => view! { <p>"Login before visiting the admin route."</p> }.into_any(),
        }}
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
            <p>"The requested auth route does not exist."</p>
        </section>
    }
}
