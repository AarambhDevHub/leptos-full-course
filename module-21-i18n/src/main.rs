//! Module 21: i18n and Accessibility.
//! Topics: translations, language context, RTL, ARIA, focus-friendly UI.
//! Run: cd module-21-i18n && trunk serve
//!
//! ## Lesson Reading Guide
//!
//! This file follows the documentation style used in `demo.rs`: a
//! module overview, visible `LESSON` banners, and function-level
//! comments that explain both what each function does and why it
//! matters in a Leptos application.

use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;

/// The key under which the chosen language is remembered in the browser.
const STORAGE_KEY: &str = "course_lang";

// ============================================================================
// LESSON 1: Bootstrapping The Browser Module
// ============================================================================

/// Explains the `main` step in this lesson.
///
/// What it does: starts the runnable entry point for this module.
/// Why it matters: It makes the module runnable and shows learners where execution really begins.
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Lang {
    En,
    Es,
    Hi,
    Ar,
}

impl Lang {
    const ALL: [Lang; 4] = [Lang::En, Lang::Es, Lang::Hi, Lang::Ar];

    /// Explains the `code` step in this lesson.
    ///
    /// What it does: returns the BCP 47 language code, the one browsers and screen readers understand.
    /// Why it matters: The `lang` attribute is what makes a screen reader switch to the right voice.
    fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Es => "es",
            Lang::Hi => "hi",
            Lang::Ar => "ar",
        }
    }

    /// Explains the `dir` step in this lesson.
    ///
    /// What it does: returns the writing direction of the language, `rtl` for Arabic and `ltr` for the rest.
    /// Why it matters: Both branches of the old direction check said `ltr`, so right-to-left was never shown. Now the direction is a property of the language.
    fn dir(self) -> &'static str {
        match self {
            Lang::Ar => "rtl",
            _ => "ltr",
        }
    }

    /// Explains the `native name` step in this lesson.
    ///
    /// What it does: returns the name of the language written in that language.
    /// Why it matters: A person who cannot read the current language must still recognise their own.
    fn native_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Es => "Español",
            Lang::Hi => "हिन्दी",
            Lang::Ar => "العربية",
        }
    }

    /// Explains the `from tag` step in this lesson.
    ///
    /// What it does: reads a language tag such as `hi`, `hi-IN` or `es_MX` and returns the matching language.
    /// Why it matters: Browsers report regional tags, but we translate by language only.
    fn from_tag(tag: &str) -> Option<Lang> {
        let primary = tag.split(['-', '_']).next()?.to_ascii_lowercase();
        Lang::ALL.into_iter().find(|lang| lang.code() == primary)
    }
}

/// Explains the `pick language` step in this lesson.
///
/// What it does: chooses the starting language: a saved choice first, then the browser language, then English.
/// Why it matters: A visitor should not have to find the language switcher before they can read the page, and should not have to choose again on every visit.
fn pick_language(saved: Option<&str>, browser: Option<&str>) -> Lang {
    saved
        .and_then(Lang::from_tag)
        .or_else(|| browser.and_then(Lang::from_tag))
        .unwrap_or(Lang::En)
}

/// Explains the `initial language` step in this lesson.
///
/// What it does: reads the saved language and the browser language, and picks one.
/// Why it matters: It is the only place that touches browser storage and the navigator, so the choice itself stays testable.
fn initial_language() -> Lang {
    let saved: Option<String> = LocalStorage::get(STORAGE_KEY).ok();
    let browser = web_sys::window().and_then(|window| window.navigator().language());
    pick_language(saved.as_deref(), browser.as_deref())
}

type Table = &'static [(&'static str, &'static str)];

const EN: Table = &[
    ("title", "i18n and Accessibility"),
    ("greeting", "Welcome to the course"),
    ("checkout", "Checkout"),
    ("save", "Save settings"),
    ("alert", "Your settings were saved"),
    ("name", "Name"),
    ("submit", "Submit"),
    ("direction", "Reading direction"),
    ("direction_note", "This text reads from left to right"),
];

const ES: Table = &[
    ("title", "i18n y Accesibilidad"),
    ("greeting", "Bienvenido al curso"),
    ("checkout", "Pagar"),
    ("save", "Guardar ajustes"),
    ("alert", "Tus ajustes fueron guardados"),
    ("name", "Nombre"),
    ("submit", "Enviar"),
    ("direction", "Dirección de lectura"),
    ("direction_note", "Este texto se lee de izquierda a derecha"),
];

const HI: Table = &[
    ("title", "अंतरराष्ट्रीयकरण और पहुंच"),
    ("greeting", "कोर्स में आपका स्वागत है"),
    ("checkout", "चेकआउट"),
    ("save", "सेटिंग सहेजें"),
    ("alert", "आपकी सेटिंग सहेज दी गई"),
    ("name", "नाम"),
    ("submit", "जमा करें"),
    ("direction", "पढ़ने की दिशा"),
    ("direction_note", "यह पाठ बाएं से दाएं पढ़ा जाता है"),
];

const AR: Table = &[
    ("title", "التدويل وإمكانية الوصول"),
    ("greeting", "مرحبًا بك في الدورة"),
    ("checkout", "الدفع"),
    ("save", "حفظ الإعدادات"),
    ("alert", "تم حفظ إعداداتك"),
    ("name", "الاسم"),
    ("submit", "إرسال"),
    ("direction", "اتجاه القراءة"),
    ("direction_note", "هذا النص يُقرأ من اليمين إلى اليسار"),
];

/// Explains the `table` step in this lesson.
///
/// What it does: returns the static list of messages for one language.
/// Why it matters: The old code built three nested `HashMap`s again on every single lookup. Static tables cost nothing at runtime.
fn table(lang: Lang) -> Table {
    match lang {
        Lang::En => EN,
        Lang::Es => ES,
        Lang::Hi => HI,
        Lang::Ar => AR,
    }
}

/// Explains the `lookup` step in this lesson.
///
/// What it does: finds the message for a key in one language, or `None` if it is missing.
/// Why it matters: Returning an option lets a test find missing translations before a visitor sees a raw key.
fn lookup(lang: Lang, key: &str) -> Option<&'static str> {
    table(lang)
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, message)| *message)
}

/// Explains the `t` step in this lesson.
///
/// What it does: translates a key, and shows the key itself when no translation exists.
/// Why it matters: A visible key is ugly but honest, and the completeness test keeps it from ever reaching users.
fn t(lang: Lang, key: &'static str) -> &'static str {
    lookup(lang, key).unwrap_or(key)
}

/// Explains the `use lang` step in this lesson.
///
/// What it does: packages the `use lang` behavior into a reusable hook-style helper.
/// Why it matters: Hooks keep repeated reactive wiring out of the components that only need to consume the behavior.
fn use_lang() -> RwSignal<Lang> {
    use_context::<RwSignal<Lang>>().expect("language context")
}

/// Explains the `next menu index` step in this lesson.
///
/// What it does: returns the menu item that should get focus after a key press, or `None` if the key is not a menu key.
/// Why it matters: A menu that only shows one item in the tab order must also answer the arrow keys, or keyboard users can never reach the other items. Arrows follow what the eye sees, so in a right-to-left page the left arrow moves forward.
fn next_menu_index(current: usize, len: usize, key: &str, rtl: bool) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let forward = (current + 1) % len;
    let backward = (current + len - 1) % len;
    match key {
        "ArrowRight" => Some(if rtl { backward } else { forward }),
        "ArrowLeft" => Some(if rtl { forward } else { backward }),
        "Home" => Some(0),
        "End" => Some(len - 1),
        _ => None,
    }
}

// ============================================================================
// LESSON 2: Components And UI Composition
// ============================================================================

/// Explains the `choose` step in this lesson.
///
/// What it does: switches the language and remembers the choice in the browser.
/// Why it matters: A deliberate choice is saved, an automatic guess is not, so the guess can still follow the browser later.
fn choose(lang: RwSignal<Lang>, option: Lang) {
    lang.set(option);
    let _ = LocalStorage::set(STORAGE_KEY, option.code());
}

/// Explains the `language switcher` step in this lesson.
///
/// What it does: renders the `LanguageSwitcher` component for one focused part of the lesson UI.
/// Why it matters: Each button names its own language in that language (with a `lang` attribute so it is pronounced correctly), and `aria-pressed` tells screen reader users which one is active, which a blue background cannot.
#[component]
fn LanguageSwitcher() -> impl IntoView {
    let lang = use_lang();
    view! {
        <section class="panel">
            <h2>"Language Switcher"</h2>
            <div class="row" role="group" aria-label="Language">
                {Lang::ALL
                    .into_iter()
                    .map(|option| {
                        view! {
                            <button
                                lang=option.code()
                                aria-pressed=move || if lang.get() == option { "true" } else { "false" }
                                class:active=move || lang.get() == option
                                on:click=move |_| choose(lang, option)
                            >
                                {option.native_name()}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

/// Explains the `greeting card` step in this lesson.
///
/// What it does: renders the `GreetingCard` component for one focused part of the lesson UI.
/// Why it matters: Keeping this UI section isolated makes the signal, context, action, or routing pattern easier to study.
#[component]
fn GreetingCard() -> impl IntoView {
    let lang = use_lang();
    view! {
        <section class="panel">
            <h2>{move || t(lang.get(), "title")}</h2>
            <p>{move || t(lang.get(), "greeting")}</p>
        </section>
    }
}

/// Explains the `checkout button` step in this lesson.
///
/// What it does: renders the `CheckoutButton` component for one focused part of the lesson UI.
/// Why it matters: The visible text already is the accessible name. The old `aria-label` repeated it, and a label that differs from the visible text breaks voice control ("click Checkout").
#[component]
fn CheckoutButton() -> impl IntoView {
    let lang = use_lang();
    view! {
        <section class="panel">
            <h2>"Button Label"</h2>
            <button>{move || t(lang.get(), "checkout")}</button>
        </section>
    }
}

/// Explains the `alert banner` step in this lesson.
///
/// What it does: renders a Save button and a live region that announces the confirmation only after the save.
/// Why it matters: A live region announces changes to its content. The old banner always held the message, so it was announced every time the language changed, even though nothing was saved. The region itself stays in the page from the start, because browsers only announce changes to a region they already know.
#[component]
fn AlertBanner() -> impl IntoView {
    let lang = use_lang();
    let saved = RwSignal::new(false);

    // A new language starts a fresh state, so the old message is never read out again.
    Effect::new(move |_| {
        lang.get();
        saved.set(false);
    });

    view! {
        <section class="panel">
            <h2>"ARIA live region"</h2>
            <button on:click=move |_| saved.set(true)>{move || t(lang.get(), "save")}</button>
            <div class="alert-region" role="status" aria-live="polite">
                {move || saved.get().then(|| view! { <div class="alert">{t(lang.get(), "alert")}</div> })}
            </div>
        </section>
    }
}

/// Explains the `accessible form` step in this lesson.
///
/// What it does: renders the `AccessibleForm` component for one focused part of the lesson UI.
/// Why it matters: A real `label` tied to the input by `for` and `id` gives the input its name and makes the label clickable. `autocomplete` lets browsers and password managers fill the field.
#[component]
fn AccessibleForm() -> impl IntoView {
    let lang = use_lang();
    let name = RwSignal::new(String::new());
    view! {
        <section class="panel">
            <h2>"Accessible Form"</h2>
            <form class="stack" on:submit=move |event| event.prevent_default()>
                <label for="name-input">{move || t(lang.get(), "name")}</label>
                <input
                    id="name-input"
                    autocomplete="name"
                    prop:value=name
                    on:input:target=move |event| name.set(event.target().value())
                />
                <button type="submit">{move || t(lang.get(), "submit")}</button>
            </form>
        </section>
    }
}

/// Explains the `direction preview` step in this lesson.
///
/// What it does: renders the `DirectionPreview` component for one focused part of the lesson UI.
/// Why it matters: The direction now comes from the language (`Lang::dir`), and the sentence is one complete translated message, not two fragments joined with a colon, because word order and punctuation differ between languages.
#[component]
fn DirectionPreview() -> impl IntoView {
    let lang = use_lang();
    view! {
        <section class="panel">
            <h2>{move || t(lang.get(), "direction")}</h2>
            <div class="direction-card" dir=move || lang.get().dir()>
                {move || t(lang.get(), "direction_note")}
            </div>
        </section>
    }
}

/// Explains the `keyboard menu` step in this lesson.
///
/// What it does: renders a menu with one tab stop and arrow key, Home and End navigation that also moves focus.
/// Why it matters: Only the selected item was in the tab order and nothing answered the arrow keys, so keyboard users could never reach the other items. Now Tab enters the menu once and the arrows move inside it.
#[component]
fn KeyboardMenu() -> impl IntoView {
    let lang = use_lang();
    let selected = RwSignal::new(0_usize);
    let items = ["Overview", "Lessons", "Community"];
    let refs: [NodeRef<leptos::html::Button>; 3] = [NodeRef::new(), NodeRef::new(), NodeRef::new()];

    view! {
        <section class="panel">
            <h2>"Keyboard-friendly menu"</h2>
            <div role="menubar" aria-label="Main menu" class="menu">
                {items.into_iter().enumerate().map(|(index, item)| view! {
                    <button
                        node_ref=refs[index]
                        role="menuitem"
                        tabindex=move || if selected.get() == index { "0" } else { "-1" }
                        aria-current=move || if selected.get() == index { "true" } else { "false" }
                        class:active=move || selected.get() == index
                        on:click=move |_| selected.set(index)
                        on:keydown=move |event: web_sys::KeyboardEvent| {
                            let rtl = lang.get_untracked().dir() == "rtl";
                            if let Some(next) = next_menu_index(index, items.len(), &event.key(), rtl) {
                                event.prevent_default();
                                selected.set(next);
                                if let Some(button) = refs[next].get() {
                                    let _ = button.focus();
                                }
                            }
                        }
                    >
                        {item}
                    </button>
                }).collect_view()}
            </div>
        </section>
    }
}

/// Explains the `app` step in this lesson.
///
/// What it does: provides the language, and keeps the page's `lang`, `dir` and title in step with it.
/// Why it matters: The `lang` and `dir` on the `html` element are what screen readers and the browser's layout engine use. Without them a Hindi page was read with an English voice, and no page could flip to right-to-left.
#[component]
fn App() -> impl IntoView {
    let lang = RwSignal::new(initial_language());
    provide_context(lang);

    Effect::new(move |_| {
        let current = lang.get();
        if let Some(root) = document().document_element() {
            let _ = root.set_attribute("lang", current.code());
            let _ = root.set_attribute("dir", current.dir());
        }
        document().set_title(t(current, "title"));
    });

    view! {
        <style>{STYLE}</style>
        <main>
            <h1>"Module 21 - i18n and Accessibility"</h1>
            <LanguageSwitcher/>
            <GreetingCard/>
            <CheckoutButton/>
            <AlertBanner/>
            <AccessibleForm/>
            <DirectionPreview/>
            <KeyboardMenu/>
        </main>
    }
}

const STYLE: &str = r#"
body { margin: 0; font-family: system-ui, sans-serif; background: #f8fafc; color: #17202a; }
main { max-width: 900px; margin: 0 auto; padding: 32px 16px; }
.panel { background: white; border: 1px solid #d9dee8; border-radius: 8px; padding: 18px; margin: 14px 0; }
.row, .menu { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.stack { display: grid; gap: 8px; max-width: 360px; }
button, input { font: inherit; padding: 8px 10px; border-radius: 6px; border: 1px solid #94a3b8; }
button { background: #e5e7eb; cursor: pointer; }
button.active { background: #1f6feb; color: white; border-color: #1f6feb; }
.alert { padding: 12px; border-radius: 8px; background: #ecfdf5; color: #065f46; font-weight: 700; }
.direction-card { border: 1px dashed #94a3b8; border-radius: 8px; padding: 14px; }
.alert-region { min-height: 3.2em; margin-top: 10px; }
button:focus-visible, input:focus-visible { outline: 3px solid #1f6feb; outline-offset: 2px; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_every_key() {
        for lang in Lang::ALL {
            for (key, _) in EN {
                let message = lookup(lang, key);
                assert!(message.is_some(), "{lang:?} is missing {key}");
                assert!(
                    !message.unwrap().trim().is_empty(),
                    "{lang:?} has an empty {key}"
                );
            }
        }
    }

    #[test]
    fn no_language_has_extra_or_duplicate_keys() {
        for lang in Lang::ALL {
            let keys: Vec<&str> = table(lang).iter().map(|(key, _)| *key).collect();
            let mut sorted = keys.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), keys.len(), "{lang:?} repeats a key");
            assert_eq!(
                keys.len(),
                EN.len(),
                "{lang:?} has a different number of keys"
            );
        }
    }

    #[test]
    fn only_arabic_is_right_to_left() {
        assert_eq!(Lang::Ar.dir(), "rtl");
        for lang in [Lang::En, Lang::Es, Lang::Hi] {
            assert_eq!(lang.dir(), "ltr");
        }
    }

    #[test]
    fn language_codes_round_trip() {
        for lang in Lang::ALL {
            assert_eq!(Lang::from_tag(lang.code()), Some(lang));
        }
    }

    #[test]
    fn regional_tags_map_to_the_language() {
        assert_eq!(Lang::from_tag("hi-IN"), Some(Lang::Hi));
        assert_eq!(Lang::from_tag("es_MX"), Some(Lang::Es));
        assert_eq!(Lang::from_tag("AR-EG"), Some(Lang::Ar));
        assert_eq!(Lang::from_tag("fr-FR"), None);
        assert_eq!(Lang::from_tag(""), None);
    }

    #[test]
    fn starting_language_prefers_saved_then_browser_then_english() {
        assert_eq!(pick_language(Some("hi"), Some("es-ES")), Lang::Hi);
        assert_eq!(pick_language(None, Some("es-ES")), Lang::Es);
        assert_eq!(pick_language(Some("klingon"), Some("ar")), Lang::Ar);
        assert_eq!(pick_language(None, Some("de")), Lang::En);
        assert_eq!(pick_language(None, None), Lang::En);
    }

    #[test]
    fn a_missing_key_shows_the_key() {
        assert_eq!(t(Lang::En, "does_not_exist"), "does_not_exist");
    }

    #[test]
    fn arrows_move_and_wrap() {
        assert_eq!(next_menu_index(0, 3, "ArrowRight", false), Some(1));
        assert_eq!(next_menu_index(2, 3, "ArrowRight", false), Some(0));
        assert_eq!(next_menu_index(0, 3, "ArrowLeft", false), Some(2));
    }

    #[test]
    fn arrows_follow_the_eye_in_right_to_left() {
        assert_eq!(next_menu_index(0, 3, "ArrowLeft", true), Some(1));
        assert_eq!(next_menu_index(0, 3, "ArrowRight", true), Some(2));
    }

    #[test]
    fn home_end_and_other_keys() {
        assert_eq!(next_menu_index(1, 3, "Home", false), Some(0));
        assert_eq!(next_menu_index(1, 3, "End", false), Some(2));
        assert_eq!(next_menu_index(1, 3, "a", false), None);
        assert_eq!(next_menu_index(0, 0, "ArrowRight", false), None);
    }
}
