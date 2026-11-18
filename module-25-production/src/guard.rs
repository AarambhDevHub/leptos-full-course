//! Module 25: pure production guards.
//!
//! Everything here is plain Rust with no framework types, so each rule can be
//! unit tested without a server or a database. The server functions call these
//! functions; they do not repeat the rules.

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const MIN_SECRET_LEN: usize = 32;
pub const NAME_MIN: usize = 2;
pub const NAME_MAX: usize = 80;
pub const EMAIL_MAX: usize = 254;
pub const PASSWORD_MIN: usize = 8;
pub const PASSWORD_MAX: usize = 128;
pub const TITLE_MAX: usize = 200;
pub const BODY_MAX: usize = 10_000;
pub const SESSION_SECONDS: u32 = 12 * 60 * 60;
/// Request bodies larger than this are refused before any handler runs.
pub const BODY_LIMIT_BYTES: usize = 64 * 1024;

/// Example values that appear in this repository or in tutorials. Even when
/// they are long enough they are public, so they are refused by name.
const PLACEHOLDER_SECRETS: [&str; 6] = [
    "change-this",
    "change-me",
    "changeme",
    "replace-me",
    "module-25-production-secret",
    "your-secret",
];

/// Accepts a JWT signing secret only when it is present and long enough.
/// There is deliberately no default: a default secret is a public secret.
pub fn validate_secret(value: Option<&str>) -> Result<String, String> {
    match value.map(str::trim) {
        None | Some("") => Err("JWT_SECRET is not set".into()),
        Some(secret) if secret.chars().count() < MIN_SECRET_LEN => Err(format!(
            "JWT_SECRET is too short: use at least {MIN_SECRET_LEN} characters"
        )),
        Some(secret)
            if PLACEHOLDER_SECRETS
                .iter()
                .any(|bad| secret.to_ascii_lowercase().contains(bad)) =>
        {
            Err("JWT_SECRET is an example value: generate a random one".into())
        }
        Some(secret) => Ok(secret.to_string()),
    }
}

/// Builds the Set-Cookie value for the session. `secure` adds the Secure flag.
pub fn session_cookie(token: &str, secure: bool) -> String {
    let mut cookie =
        format!("prod_session={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={SESSION_SECONDS}");
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
}

/// Builds the Set-Cookie value that expires the session.
pub fn cleared_cookie(secure: bool) -> String {
    let mut cookie = "prod_session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0".to_string();
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
}

/// Secure cookies are the default. Only the exact text `false` turns them off,
/// which is meant for local development over plain http.
pub fn cookie_secure(value: Option<&str>) -> bool {
    !matches!(value.map(|v| v.trim().to_ascii_lowercase()), Some(v) if v == "false")
}

fn char_len(text: &str) -> usize {
    text.chars().count()
}

/// A deliberately small email check: one at sign, text on both sides, a dot in
/// the domain that is not at either end, no spaces. Real proof of an address
/// is a confirmation email, which this module does not send.
pub fn looks_like_email(email: &str) -> bool {
    if email.is_empty() || char_len(email) > EMAIL_MAX || email.chars().any(char::is_whitespace) {
        return false;
    }
    let mut parts = email.split('@');
    let (Some(local), Some(domain), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
}

/// Cleans and checks registration input. Returns (name, email) normalised.
pub fn validate_registration(
    name: &str,
    email: &str,
    password: &str,
) -> Result<(String, String), &'static str> {
    let name = name.trim();
    let email = email.trim().to_lowercase();
    if !(NAME_MIN..=NAME_MAX).contains(&char_len(name)) || name.contains('\0') {
        return Err("name must be between 2 and 80 characters");
    }
    if !looks_like_email(&email) {
        return Err("enter a valid email address");
    }
    if !(PASSWORD_MIN..=PASSWORD_MAX).contains(&char_len(password)) || password.contains('\0') {
        return Err("password must be between 8 and 128 characters");
    }
    Ok((name.to_string(), email))
}

/// Cleans and checks a note. Returns (title, body).
pub fn validate_note(title: &str, body: &str) -> Result<(String, String), &'static str> {
    let title = title.trim();
    if title.is_empty() {
        return Err("title is required");
    }
    if char_len(title) > TITLE_MAX {
        return Err("title must be at most 200 characters");
    }
    if char_len(body) > BODY_MAX {
        return Err("body must be at most 10000 characters");
    }
    if title.contains('\0') || body.contains('\0') {
        return Err("text contains a character that cannot be stored");
    }
    Ok((title.to_string(), body.to_string()))
}

/// Counts failed sign-ins per key (the email) inside a time window.
/// It lives in memory, so it is per process: with several server copies each
/// keeps its own count. See the README for what production would add.
pub struct AttemptLimiter {
    max_failures: u32,
    window: Duration,
    entries: HashMap<String, (u32, Instant)>,
}

impl AttemptLimiter {
    pub fn new(max_failures: u32, window: Duration) -> Self {
        Self {
            max_failures,
            window,
            entries: HashMap::new(),
        }
    }

    /// True when this key may try now.
    pub fn allowed(&mut self, key: &str, now: Instant) -> bool {
        match self.entries.get(key) {
            Some((count, first)) if now.duration_since(*first) < self.window => {
                *count < self.max_failures
            }
            Some(_) => {
                self.entries.remove(key);
                true
            }
            None => true,
        }
    }

    pub fn record_failure(&mut self, key: &str, now: Instant) {
        // Never let the map grow without bound under a flood of random emails.
        if self.entries.len() >= 10_000 && !self.entries.contains_key(key) {
            let window = self.window;
            self.entries
                .retain(|_, (_, first)| now.duration_since(*first) < window);
            if self.entries.len() >= 10_000 {
                return;
            }
        }
        let entry = self.entries.entry(key.to_string()).or_insert((0, now));
        if now.duration_since(entry.1) >= self.window {
            *entry = (0, now);
        }
        entry.0 += 1;
    }

    pub fn record_success(&mut self, key: &str) {
        self.entries.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_must_exist_and_be_long() {
        assert!(validate_secret(None).is_err());
        assert!(validate_secret(Some("   ")).is_err());
        assert!(validate_secret(Some("module-25-production-secret")).is_err());
        assert!(validate_secret(Some("change-this-secret-in-production")).is_err());
        let good = "a".repeat(32);
        assert_eq!(validate_secret(Some(&good)).unwrap(), good);
    }

    #[test]
    fn cookie_has_security_flags() {
        let secure = session_cookie("tok", true);
        assert!(secure.contains("HttpOnly") && secure.contains("SameSite=Lax"));
        assert!(secure.ends_with("; Secure"));
        assert!(!session_cookie("tok", false).contains("Secure"));
        assert!(cleared_cookie(true).contains("Max-Age=0"));
        assert!(cleared_cookie(true).ends_with("; Secure"));
    }

    #[test]
    fn secure_cookie_is_the_default() {
        assert!(cookie_secure(None));
        assert!(cookie_secure(Some("true")));
        assert!(cookie_secure(Some("")));
        assert!(cookie_secure(Some("nope")));
        assert!(!cookie_secure(Some("false")));
        assert!(!cookie_secure(Some(" FALSE ")));
    }

    #[test]
    fn emails() {
        for good in ["a@b.co", "first.last@mail.example.org"] {
            assert!(looks_like_email(good), "{good}");
        }
        for bad in [
            "", "@", "a@", "@b.co", "a@b", "a@.co", "a@b.", "a@@b.co", "a b@c.de", "a@b..co",
        ] {
            assert!(!looks_like_email(bad), "{bad}");
        }
        assert!(!looks_like_email(&format!("{}@b.co", "a".repeat(300))));
    }

    #[test]
    fn registration_rules() {
        let (name, email) =
            validate_registration("  Asha ", " ASHA@Example.COM ", "password1").unwrap();
        assert_eq!(
            (name.as_str(), email.as_str()),
            ("Asha", "asha@example.com")
        );
        assert!(validate_registration("A", "a@b.co", "password1").is_err());
        assert!(validate_registration(&"n".repeat(81), "a@b.co", "password1").is_err());
        assert!(validate_registration("Asha", "@", "password1").is_err());
        assert!(validate_registration("Asha", "a@b.co", "short").is_err());
        assert!(validate_registration("Asha", "a@b.co", &"p".repeat(129)).is_err());
        assert!(validate_registration("Asha", "a@b.co", "pass\0word1").is_err());
        // length counts characters, not bytes
        assert!(validate_registration("Zoë", "a@b.co", "pässwörd").is_ok());
    }

    #[test]
    fn note_rules() {
        assert_eq!(validate_note(" Hi ", "body").unwrap().0, "Hi");
        assert!(validate_note("  ", "b").is_err());
        assert!(validate_note(&"t".repeat(201), "b").is_err());
        assert!(validate_note("t", &"b".repeat(10_001)).is_err());
        assert!(validate_note("nul\0", "b").is_err());
        assert!(validate_note("t", "b\0").is_err());
        assert!(validate_note(&"t".repeat(200), &"b".repeat(10_000)).is_ok());
    }

    #[test]
    fn limiter_blocks_then_recovers() {
        let t0 = Instant::now();
        let mut l = AttemptLimiter::new(3, Duration::from_secs(60));
        for _ in 0..3 {
            assert!(l.allowed("a", t0));
            l.record_failure("a", t0);
        }
        assert!(!l.allowed("a", t0 + Duration::from_secs(30)));
        assert!(l.allowed("b", t0), "other keys are unaffected");
        assert!(
            l.allowed("a", t0 + Duration::from_secs(61)),
            "window expired"
        );
        l.record_failure("a", t0 + Duration::from_secs(61));
        assert!(
            l.allowed("a", t0 + Duration::from_secs(62)),
            "count restarted"
        );
    }

    #[test]
    fn success_clears_failures() {
        let t0 = Instant::now();
        let mut l = AttemptLimiter::new(2, Duration::from_secs(60));
        l.record_failure("a", t0);
        l.record_failure("a", t0);
        assert!(!l.allowed("a", t0));
        l.record_success("a");
        assert!(l.allowed("a", t0));
    }

    #[test]
    fn limiter_memory_is_bounded() {
        let t0 = Instant::now();
        let mut l = AttemptLimiter::new(3, Duration::from_secs(60));
        for i in 0..20_000 {
            l.record_failure(&format!("k{i}"), t0);
        }
        assert!(l.entries.len() <= 10_000);
    }
}
