//! Backend localization. The diagnostics and install-log strings are produced
//! in Rust, so they need translating too — but a desktop app has exactly one
//! UI language at a time, so "current locale" is a single global setting
//! rather than something threaded through every function signature.
//!
//! The frontend pushes the active locale via the `set_locale` command (and on
//! startup); Rust strings are then written as `tr("中文", "English")`, which
//! picks the right side at call time. Default is English to match the
//! frontend's system-language default for non-zh machines.

use std::sync::atomic::{AtomicU8, Ordering};

const ZH: u8 = 0;
const EN: u8 = 1;

// Default English; the frontend syncs the real locale right after mount.
static LOCALE: AtomicU8 = AtomicU8::new(EN);

/// Set the active backend locale. `locale` is "zh" or "en" (anything else is
/// treated as English).
pub fn set_locale(locale: &str) {
    LOCALE.store(if locale == "zh" { ZH } else { EN }, Ordering::Relaxed);
}

fn is_zh() -> bool {
    LOCALE.load(Ordering::Relaxed) == ZH
}

/// Pick the Chinese or English variant of a string based on the active locale.
/// Returned as `String` because most call sites build it via `format!`.
pub fn tr(zh: &str, en: &str) -> String {
    if is_zh() {
        zh.to_string()
    } else {
        en.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tr_follows_locale() {
        set_locale("zh");
        assert_eq!(tr("你好", "Hello"), "你好");
        set_locale("en");
        assert_eq!(tr("你好", "Hello"), "Hello");
        // Unknown locale falls back to English.
        set_locale("fr");
        assert_eq!(tr("你好", "Hello"), "Hello");
    }
}
