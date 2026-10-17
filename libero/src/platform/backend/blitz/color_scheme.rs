use std::{
    rc::{Rc, Weak},
    time::Duration,
};

use blitz_traits::shell;

use super::{Doc, doc};
use crate::{
    platform::{
        ColorSchemeApi, ColorSchemeSubscription,
        storage::{keep, kept},
    },
    tokens::{COLOR_SCHEME_STORAGE_KEY, ColorScheme, ColorSchemeSetting},
};

pub(in crate::platform::backend) fn color_scheme() -> Option<&'static dyn ColorSchemeApi> {
    Some(&COLOR_SCHEME)
}

/// The viewport's scheme. Blitz sends no change event, so it is read at the
/// first [`Outlet`](super::Outlet) mount, then every [`SCHEME_INTERVAL`] while subscribed.
pub(super) struct BlitzColorScheme;

static COLOR_SCHEME: BlitzColorScheme = BlitzColorScheme;

/// How late a live theme switch reaches Rust at most; the CSS follows at once.
const SCHEME_INTERVAL: Duration = Duration::from_millis(500);

fn viewport_scheme(doc: &Doc) -> Option<ColorScheme> {
    let anchor = doc.anchor()?;
    let doc = anchor.try_doc()?;
    Some(match doc.viewport().color_scheme {
        shell::ColorScheme::Dark => ColorScheme::Dark,
        shell::ColorScheme::Light => ColorScheme::Light,
    })
}

pub(super) fn check_scheme(doc: &Doc) {
    let Some(scheme) = viewport_scheme(doc) else {
        return;
    };
    if doc.scheme.replace(scheme) == scheme {
        return;
    }
    doc.scheme_callbacks.each(|callback| callback(scheme));
}

impl ColorSchemeApi for BlitzColorScheme {
    /// Before the first frame there is no document to ask, so this answers the
    /// last scheme known, and the first [`Outlet`](super::Outlet) mount corrects it.
    fn system(&self) -> ColorScheme {
        let Some(doc) = doc() else {
            return ColorScheme::Light;
        };
        if let Some(scheme) = viewport_scheme(&doc) {
            doc.scheme.set(scheme);
        }
        doc.scheme.get()
    }

    fn on_change(&self, callback: Box<dyn Fn(ColorScheme)>) -> Box<dyn ColorSchemeSubscription> {
        let Some(doc) = doc() else {
            return Box::new(BlitzColorSchemeSubscription(0, Weak::new()));
        };
        let id = doc.scheme_callbacks.add(Rc::from(callback));
        let mut poll = doc.scheme_poll.borrow_mut();
        if poll.is_none() {
            let weak = Rc::downgrade(&doc);
            *poll = crate::platform::backend::thread::timer().map(|timer| {
                timer.every(
                    SCHEME_INTERVAL,
                    Box::new(move || {
                        if let Some(doc) = weak.upgrade() {
                            check_scheme(&doc);
                        }
                    }),
                )
            });
        }
        Box::new(BlitzColorSchemeSubscription(id, Rc::downgrade(&doc)))
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

struct BlitzColorSchemeSubscription(u64, Weak<Doc>);

impl ColorSchemeSubscription for BlitzColorSchemeSubscription {}

impl Drop for BlitzColorSchemeSubscription {
    fn drop(&mut self) {
        let Some(doc) = self.1.upgrade() else {
            return;
        };
        if !doc.scheme_callbacks.remove(self.0) {
            // Taken out before it drops, so no borrow is held across its drop.
            let poll = doc.scheme_poll.borrow_mut().take();
            drop(poll);
        }
    }
}
