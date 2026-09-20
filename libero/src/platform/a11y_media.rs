//! Stands in for the accessibility media features Blitz's stylo cannot match.
//! Throw it out once stylo matches them (todo 954).

use std::borrow::Cow;
use std::cell::Cell;

use super::backend;
use crate::tokens::{AccessibilityPreferences, Contrast};

/// The platform's own accessibility settings.
pub(crate) trait A11yMediaApi {
    /// What the platform is set to right now.
    fn system(&self) -> AccessibilityPreferences;

    /// Calls `callback` when a setting changes, until the subscription drops.
    fn on_change(
        &self,
        callback: Box<dyn Fn(AccessibilityPreferences)>,
    ) -> Box<dyn A11yMediaSubscription>;
}

/// Dropping it stops the callbacks.
pub(crate) trait A11yMediaSubscription {}

/// The web's `matchMedia` and the Linux desktop portal under Blitz; `None` elsewhere.
pub(crate) fn a11y_media() -> Option<&'static dyn A11yMediaApi> {
    backend::a11y_media()
}

/// Whether libero answers the features itself, in the CSS it emits: only Blitz.
pub(crate) const fn answers_a11y_media() -> bool {
    backend::ANSWERS_A11Y_MEDIA
}

/// A feature test that always, and one that never, matches.
const MATCHES: &str = "(min-width: 0px)";
const NEVER: &str = "(max-width: -1px)";

/// `Some(matches)` for a feature test this stands in for, `None` for any other.
/// `feature` is the text between the parentheses.
fn answer(preferences: &AccessibilityPreferences, feature: &str) -> Option<bool> {
    let (name, value) = match feature.split_once(':') {
        Some((name, value)) => (name.trim(), Some(value.trim())),
        None => (feature.trim(), None),
    };
    let (on, active) = match name {
        "prefers-reduced-motion" => (preferences.reduced_motion, "reduce"),
        "prefers-reduced-transparency" => (preferences.reduced_transparency, "reduce"),
        "forced-colors" => (preferences.forced_colors, "active"),
        "prefers-contrast" => {
            return Some(match value {
                None => preferences.contrast != Contrast::NoPreference,
                Some(value) => value == preferences.contrast.as_str(),
            });
        }
        _ => return None,
    };
    Some(match value {
        None => on,
        Some(value) if value == active => on,
        Some("no-preference" | "none") => !on,
        Some(_) => false,
    })
}

/// `css` with each test of the four features swapped for one stylo matches
/// exactly where `preferences` says it does.
pub(crate) fn answer_a11y_media<'a>(
    css: &'a str,
    preferences: &AccessibilityPreferences,
) -> Cow<'a, str> {
    const NAMES: [&str; 4] = [
        "prefers-reduced-motion",
        "forced-colors",
        "prefers-contrast",
        "prefers-reduced-transparency",
    ];
    if !NAMES.iter().any(|name| css.contains(name)) {
        return Cow::Borrowed(css);
    }
    let mut answered = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(open) = rest.find('(') {
        answered.push_str(&rest[..open]);
        let from = &rest[open..];
        let test = from
            .find(')')
            .and_then(|close| Some((close, answer(preferences, &from[1..close])?)));
        match test {
            Some((close, matches)) => {
                answered.push_str(if matches { MATCHES } else { NEVER });
                rest = &from[close + 1..];
            }
            None => {
                answered.push('(');
                rest = &from[1..];
            }
        }
    }
    answered.push_str(rest);
    Cow::Owned(answered)
}

thread_local! {
    static CURRENT: Cell<AccessibilityPreferences> = Cell::new(AccessibilityPreferences::default());
}

/// What `LiberoProvider` resolved last, for motion started from Rust.
pub(crate) fn current_a11y_media() -> AccessibilityPreferences {
    CURRENT.get()
}

pub(crate) fn set_current_a11y_media(preferences: AccessibilityPreferences) {
    CURRENT.set(preferences);
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULE: &str = "@media (prefers-reduced-motion: reduce){.a{transition:none;}}";

    #[test]
    fn a_reduced_motion_test_matches_only_when_asked_for() {
        let reduced = AccessibilityPreferences {
            reduced_motion: true,
            ..Default::default()
        };
        assert_eq!(
            answer_a11y_media(RULE, &reduced),
            "@media (min-width: 0px){.a{transition:none;}}"
        );
        assert_eq!(
            answer_a11y_media(RULE, &AccessibilityPreferences::default()),
            "@media (max-width: -1px){.a{transition:none;}}"
        );
    }

    #[test]
    fn other_tests_in_the_query_are_kept() {
        let css = "@media (min-width: 48rem) and (forced-colors:active){.a{b:c;}}";
        let forced = AccessibilityPreferences {
            forced_colors: true,
            ..Default::default()
        };
        assert_eq!(
            answer_a11y_media(css, &forced),
            "@media (min-width: 48rem) and (min-width: 0px){.a{b:c;}}"
        );
    }

    #[test]
    fn values_and_the_boolean_form_answer_as_the_web_does() {
        let preferences = AccessibilityPreferences {
            contrast: Contrast::More,
            ..Default::default()
        };
        let answer = |feature| answer(&preferences, feature);
        assert_eq!(answer("prefers-contrast: more"), Some(true));
        assert_eq!(answer("prefers-contrast"), Some(true));
        assert_eq!(answer("prefers-contrast: less"), Some(false));
        assert_eq!(answer("prefers-reduced-motion: no-preference"), Some(true));
        assert_eq!(answer("forced-colors: none"), Some(true));
        assert_eq!(answer("prefers-reduced-transparency"), Some(false));
        assert_eq!(answer("min-width: 0px"), None);
    }

    #[test]
    fn css_without_the_features_is_borrowed() {
        let css = "@media (min-width: 48rem){.a{b:c;}}";
        assert!(matches!(
            answer_a11y_media(css, &AccessibilityPreferences::default()),
            Cow::Borrowed(_)
        ));
    }
}
