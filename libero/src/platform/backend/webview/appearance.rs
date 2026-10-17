use std::rc::{Rc, Weak};

use dioxus::core::{Runtime, spawn_forever};
use serde_json::json;

use super::{PageState, eval_with, page, runs_scripts};
use crate::platform::a11y_media::A11yMediaSubscription;
use crate::platform::storage::{keep, kept};
use crate::platform::{A11yMediaApi, ColorSchemeApi, ColorSchemeSubscription};
use crate::tokens::{
    AccessibilityPreferences, COLOR_SCHEME_STORAGE_KEY, ColorScheme, ColorSchemeSetting, Contrast,
};

/// Hands `answer` the media query's result now and on every change. The script
/// starts here: from inside `spawn_forever` it never ran.
fn watch_media(query: &'static str, answer: impl Fn(bool) + 'static) {
    let script = eval_with(
        json!(query),
        "const query = window.matchMedia(data);
        dioxus.send(query.matches);
        query.addEventListener('change', () => dioxus.send(query.matches));
        await new Promise(() => {});",
    );
    spawn_forever(async move {
        let mut script = script;
        while let Ok(matches) = script.recv::<bool>().await {
            answer(matches);
        }
    });
}

/// `false` until the page first answers, then kept live by a listener.
pub(crate) fn prefers_reduced_motion() -> bool {
    if !runs_scripts() {
        return false;
    }
    let Some(page) = page() else {
        return false;
    };
    if let Some(reduced) = page.reduced_motion.get() {
        return reduced;
    }
    page.reduced_motion.set(Some(false));
    watch_media(crate::sx::REDUCED_MOTION, move |reduced| {
        page.reduced_motion.set(Some(reduced))
    });
    false
}

pub(crate) fn a11y_media() -> Option<&'static dyn A11yMediaApi> {
    runs_scripts().then_some(&A11Y_MEDIA as &'static dyn A11yMediaApi)
}

/// The page's accessibility media queries, as the web backend reads them.
struct WebViewA11yMedia;

static A11Y_MEDIA: WebViewA11yMedia = WebViewA11yMedia;

/// The defaults until the page first answers; every answer then reaches the subscribers.
fn watch_a11y(page: &Rc<PageState>) {
    if page.a11y.get().is_some() {
        return;
    }
    page.a11y.set(Some(AccessibilityPreferences::default()));
    let page = page.clone();
    let script = eval_with(
        json!([
            crate::sx::REDUCED_MOTION,
            crate::sx::FORCED_COLORS,
            "(prefers-contrast: more)",
            "(prefers-contrast: less)",
            "(prefers-reduced-transparency: reduce)",
        ]),
        "const queries = data.map((query) => window.matchMedia(query));
        const send = () => dioxus.send(queries.map((query) => query.matches));
        send();
        queries.forEach((query) => query.addEventListener('change', send));
        await new Promise(() => {});",
    );
    spawn_forever(async move {
        let mut script = script;
        while let Ok([motion, forced, more, less, transparency]) = script.recv::<[bool; 5]>().await
        {
            let preferences = AccessibilityPreferences {
                reduced_motion: motion,
                forced_colors: forced,
                contrast: match (more, less) {
                    (true, _) => Contrast::More,
                    (_, true) => Contrast::Less,
                    _ => Contrast::NoPreference,
                },
                reduced_transparency: transparency,
            };
            if page.a11y.replace(Some(preferences)) == Some(preferences) {
                continue;
            }
            let callbacks: Vec<_> = (page.a11y_callbacks.borrow().iter())
                .map(|(_, call)| call.clone())
                .collect();
            for callback in callbacks {
                callback(preferences);
            }
        }
    });
}

impl A11yMediaApi for WebViewA11yMedia {
    fn system(&self) -> AccessibilityPreferences {
        let Some(page) = page() else {
            return AccessibilityPreferences::default();
        };
        watch_a11y(&page);
        page.a11y.get().unwrap_or_default()
    }

    fn on_change(
        &self,
        callback: Box<dyn Fn(AccessibilityPreferences)>,
    ) -> Box<dyn A11yMediaSubscription> {
        let Some(page) = page() else {
            return Box::new(WebViewSubscription(Weak::new(), 0));
        };
        watch_a11y(&page);
        let id = page.next_id();
        page.a11y_callbacks
            .borrow_mut()
            .push((id, in_subscriber(callback)));
        Box::new(WebViewSubscription(Rc::downgrade(&page), id))
    }
}

/// Runs `callback` in the subscribing scope: a watcher's task runs at the root,
/// which owns none of the subscriber's signals.
fn in_subscriber<T: 'static>(callback: Box<dyn Fn(T)>) -> Rc<dyn Fn(T)> {
    let scope = Runtime::try_current().and_then(|runtime| runtime.try_current_scope_id());
    Rc::new(move |value| match (scope, Runtime::try_current()) {
        (Some(scope), Some(runtime)) => runtime.in_scope(scope, || callback(value)),
        _ => callback(value),
    })
}

/// A media callback's page and id; dropping it removes the callback.
struct WebViewSubscription(Weak<PageState>, u64);

impl A11yMediaSubscription for WebViewSubscription {}

impl ColorSchemeSubscription for WebViewSubscription {}

impl Drop for WebViewSubscription {
    fn drop(&mut self) {
        let Some(page) = self.0.upgrade() else {
            return;
        };
        page.a11y_callbacks
            .borrow_mut()
            .retain(|(id, _)| *id != self.1);
        page.scheme_callbacks
            .borrow_mut()
            .retain(|(id, _)| *id != self.1);
    }
}

pub(crate) fn color_scheme() -> Option<&'static dyn ColorSchemeApi> {
    runs_scripts().then_some(&COLOR_SCHEME as &'static dyn ColorSchemeApi)
}

/// The system scheme from the page's media query. The override lives in a file:
/// the provider reads it at mount, before any script could answer.
struct WebViewColorScheme;

static COLOR_SCHEME: WebViewColorScheme = WebViewColorScheme;

/// Light until the page first answers; a dark answer then reaches every
/// [`on_change`](ColorSchemeApi::on_change) subscriber.
fn watch_scheme(page: &Rc<PageState>) {
    if page.scheme.get().is_some() {
        return;
    }
    page.scheme.set(Some(ColorScheme::Light));
    let page = page.clone();
    watch_media(crate::theme::DARK_SCHEME_QUERY, move |dark| {
        let scheme = if dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        };
        if page.scheme.replace(Some(scheme)) == Some(scheme) {
            return;
        }
        let callbacks: Vec<_> = (page.scheme_callbacks.borrow().iter())
            .map(|(_, call)| call.clone())
            .collect();
        for callback in callbacks {
            callback(scheme);
        }
    });
}

impl ColorSchemeApi for WebViewColorScheme {
    fn system(&self) -> ColorScheme {
        let Some(page) = page() else {
            return ColorScheme::Light;
        };
        watch_scheme(&page);
        page.scheme.get().unwrap_or(ColorScheme::Light)
    }

    fn on_change(&self, callback: Box<dyn Fn(ColorScheme)>) -> Box<dyn ColorSchemeSubscription> {
        let Some(page) = page() else {
            return Box::new(WebViewSubscription(Weak::new(), 0));
        };
        watch_scheme(&page);
        let id = page.next_id();
        page.scheme_callbacks
            .borrow_mut()
            .push((id, in_subscriber(callback)));
        Box::new(WebViewSubscription(Rc::downgrade(&page), id))
    }

    fn stored(&self) -> Option<ColorSchemeSetting> {
        Some(ColorSchemeSetting::parse(
            kept(COLOR_SCHEME_STORAGE_KEY)?.trim(),
        ))
    }

    fn store(&self, setting: ColorSchemeSetting) {
        keep(COLOR_SCHEME_STORAGE_KEY, Some(setting.as_str()));
    }
}
