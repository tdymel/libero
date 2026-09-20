//! Answers the accessibility media features in libero's own sheets: always under
//! Blitz, whose stylo cannot match them, and for a forced reduced motion.
//! Throw the Blitz half out once stylo matches them (todo 954).

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

/// The web's `matchMedia`, Android's WebView and the Linux desktop portal under
/// Blitz; `None` elsewhere.
pub(crate) fn a11y_media() -> Option<&'static dyn A11yMediaApi> {
    backend::a11y_media()
}

/// Whether the renderer cannot match the features itself: only Blitz.
pub(crate) const fn answers_a11y_media() -> bool {
    backend::ANSWERS_A11Y_MEDIA
}

/// What libero writes into its sheets per feature; `None` leaves the test to the renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct A11yAnswers {
    pub(crate) reduced_motion: Option<bool>,
    pub(crate) forced_colors: Option<bool>,
    pub(crate) contrast: Option<Contrast>,
    pub(crate) reduced_transparency: Option<bool>,
}

impl A11yAnswers {
    /// Every feature from `system` where the renderer cannot match them, then a
    /// forced reduced motion on top.
    pub(crate) fn new(system: AccessibilityPreferences, forced_motion: Option<bool>) -> Self {
        let mut answers = if answers_a11y_media() {
            Self {
                reduced_motion: Some(system.reduced_motion),
                forced_colors: Some(system.forced_colors),
                contrast: Some(system.contrast),
                reduced_transparency: Some(system.reduced_transparency),
            }
        } else {
            Self::default()
        };
        answers.reduced_motion = forced_motion.or(answers.reduced_motion);
        answers
    }

    fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// `Some(matches)` for a feature test answered here, `None` for any other.
    /// `feature` is the text between the parentheses.
    fn answer(&self, feature: &str) -> Option<bool> {
        let (name, value) = match feature.split_once(':') {
            Some((name, value)) => (name.trim(), Some(value.trim())),
            None => (feature.trim(), None),
        };
        let (on, active) = match name {
            "prefers-reduced-motion" => (self.reduced_motion?, "reduce"),
            "prefers-reduced-transparency" => (self.reduced_transparency?, "reduce"),
            "forced-colors" => (self.forced_colors?, "active"),
            "prefers-contrast" => {
                let contrast = self.contrast?;
                return Some(match value {
                    None => contrast != Contrast::NoPreference,
                    Some(value) => value == contrast.as_str(),
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

    /// `query`, one entry of a media query list, as it reads once the answered
    /// tests are gone: `not all` if one fails, `all` if nothing else is left.
    /// `None` when no test is answered or the query is not a plain `and` chain.
    fn answer_query(&self, query: &str) -> Option<String> {
        let tests: Vec<&str> = query.split(" and ").map(str::trim).collect();
        let features = tests
            .iter()
            .map(|test| {
                let feature = test.strip_prefix('(')?.strip_suffix(')')?;
                (!feature.contains(['(', ')'])).then_some(feature)
            })
            .collect::<Option<Vec<_>>>()?;
        let answers: Vec<_> = features
            .iter()
            .map(|feature| self.answer(feature))
            .collect();
        if answers.iter().all(Option::is_none) {
            return None;
        }
        if answers.contains(&Some(false)) {
            return Some("not all".into());
        }
        let kept: Vec<&str> = tests
            .iter()
            .zip(&answers)
            .filter_map(|(test, answer)| answer.is_none().then_some(*test))
            .collect();
        Some(if kept.is_empty() {
            "all".into()
        } else {
            kept.join(" and ")
        })
    }
}

/// `css` with every `@media` query that tests an answered feature settled to
/// `all`, `not all`, or its remaining tests.
pub(crate) fn answer_a11y_media<'a>(css: &'a str, answers: &A11yAnswers) -> Cow<'a, str> {
    const NAMES: [&str; 4] = [
        "prefers-reduced-motion",
        "forced-colors",
        "prefers-contrast",
        "prefers-reduced-transparency",
    ];
    if answers.is_empty() || !NAMES.iter().any(|name| css.contains(name)) {
        return Cow::Borrowed(css);
    }
    let mut answered = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(at) = rest.find("@media") {
        let start = at + "@media".len();
        let Some(length) = rest[start..].find('{') else {
            break;
        };
        answered.push_str(&rest[..start]);
        let queries: Vec<String> = rest[start..start + length]
            .split(',')
            .map(|query| match answers.answer_query(query) {
                Some(settled) => {
                    let lead = &query[..query.len() - query.trim_start().len()];
                    let trail = &query[query.trim_end().len()..];
                    format!("{lead}{settled}{trail}")
                }
                None => query.to_owned(),
            })
            .collect();
        answered.push_str(&queries.join(","));
        rest = &rest[start + length..];
    }
    answered.push_str(rest);
    Cow::Owned(answered)
}

thread_local! {
    static CURRENT: Cell<A11yAnswers> = Cell::new(A11yAnswers::default());
}

/// What `LiberoProvider` answered last, for motion started from Rust.
pub(crate) fn current_a11y_answers() -> A11yAnswers {
    CURRENT.get()
}

pub(crate) fn set_current_a11y_answers(answers: A11yAnswers) {
    CURRENT.set(answers);
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULE: &str = "@media (prefers-reduced-motion: reduce){.a{transition:none;}}";

    fn motion(reduced: bool) -> A11yAnswers {
        A11yAnswers {
            reduced_motion: Some(reduced),
            ..Default::default()
        }
    }

    #[test]
    fn a_reduced_motion_query_matches_only_when_answered_so() {
        assert_eq!(
            answer_a11y_media(RULE, &motion(true)),
            "@media all{.a{transition:none;}}"
        );
        assert_eq!(
            answer_a11y_media(RULE, &motion(false)),
            "@media not all{.a{transition:none;}}"
        );
    }

    #[test]
    fn other_tests_in_the_query_are_kept() {
        let css = "@media (min-width: 48rem) and (forced-colors:active){.a{b:c;}}";
        let forced = |on| A11yAnswers {
            forced_colors: Some(on),
            ..Default::default()
        };
        assert_eq!(
            answer_a11y_media(css, &forced(true)),
            "@media (min-width: 48rem){.a{b:c;}}"
        );
        assert_eq!(
            answer_a11y_media(css, &forced(false)),
            "@media not all{.a{b:c;}}"
        );
    }

    #[test]
    fn each_query_of_a_list_is_answered_alone() {
        let css = "@media (prefers-reduced-motion: reduce), (max-width: 30rem){.a{b:c;}}";
        assert_eq!(
            answer_a11y_media(css, &motion(false)),
            "@media not all, (max-width: 30rem){.a{b:c;}}"
        );
    }

    #[test]
    fn an_unanswered_feature_and_other_grammar_are_left_alone() {
        let contrast = "@media (prefers-contrast: more){.a{b:c;}}";
        assert_eq!(answer_a11y_media(contrast, &motion(true)), contrast);
        let negated = "@media not (prefers-reduced-motion: reduce){.a{b:c;}}";
        assert_eq!(answer_a11y_media(negated, &motion(true)), negated);
    }

    #[test]
    fn values_and_the_boolean_form_answer_as_the_web_does() {
        let answers = A11yAnswers {
            reduced_motion: Some(false),
            forced_colors: Some(false),
            contrast: Some(Contrast::More),
            reduced_transparency: Some(false),
        };
        let answer = |feature| answers.answer(feature);
        assert_eq!(answer("prefers-contrast: more"), Some(true));
        assert_eq!(answer("prefers-contrast"), Some(true));
        assert_eq!(answer("prefers-contrast: less"), Some(false));
        assert_eq!(answer("prefers-reduced-motion: no-preference"), Some(true));
        assert_eq!(answer("forced-colors: none"), Some(true));
        assert_eq!(answer("prefers-reduced-transparency"), Some(false));
        assert_eq!(answer("min-width: 0px"), None);
    }

    #[test]
    fn a_forced_reduced_motion_wins_over_the_system() {
        let system = AccessibilityPreferences {
            reduced_motion: true,
            ..Default::default()
        };
        assert_eq!(
            A11yAnswers::new(system, Some(false)).reduced_motion,
            Some(false)
        );
    }

    #[test]
    fn nothing_answered_borrows_the_css() {
        assert!(matches!(
            answer_a11y_media(RULE, &A11yAnswers::default()),
            Cow::Borrowed(_)
        ));
    }
}
