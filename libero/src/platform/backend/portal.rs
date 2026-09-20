//! The desktop portal's accessibility settings, for Blitz on Linux (todo 954).

use std::future::poll_fn;
use std::pin::Pin;
use std::time::Duration;

use dbus::arg::{RefArg, Variant};
use dbus::blocking::Connection;
use dbus::message::MatchRule;
use dioxus::core::Task;
use dioxus::prelude::spawn;
use futures_channel::mpsc;
use futures_core::Stream;

use crate::platform::a11y_media::{A11yMediaApi, A11yMediaSubscription};
use crate::tokens::{AccessibilityPreferences, Contrast};

const DESTINATION: &str = "org.freedesktop.portal.Desktop";
const PATH: &str = "/org/freedesktop/portal/desktop";
const SETTINGS: &str = "org.freedesktop.portal.Settings";
const APPEARANCE: &str = "org.freedesktop.appearance";
/// GNOME's own keys, for a portal older than `reduced-motion`.
const GNOME_INTERFACE: &str = "org.gnome.desktop.interface";
const GNOME_A11Y: &str = "org.gnome.desktop.a11y.interface";
/// A missing portal must not hold up the first frame.
const TIMEOUT: Duration = Duration::from_millis(250);

struct Portal;

static PORTAL: Portal = Portal;

pub(super) fn a11y_media() -> Option<&'static dyn A11yMediaApi> {
    Some(&PORTAL)
}

/// One setting as a number (`true` is 1), `None` where it is not set.
fn setting(connection: &Connection, namespace: &str, key: &str) -> Option<u64> {
    let proxy = connection.with_proxy(DESTINATION, PATH, TIMEOUT);
    let (value,): (Variant<Box<dyn RefArg>>,) = proxy
        .method_call(SETTINGS, "ReadOne", (namespace, key))
        .ok()?;
    value.0.as_u64()
}

fn read(connection: &Connection) -> AccessibilityPreferences {
    let reduced_motion = match setting(connection, APPEARANCE, "reduced-motion") {
        Some(reduce) => reduce == 1,
        None => setting(connection, GNOME_INTERFACE, "enable-animations") == Some(0),
    };
    let high_contrast = match setting(connection, APPEARANCE, "contrast") {
        Some(1) => true,
        _ => setting(connection, GNOME_A11Y, "high-contrast") == Some(1),
    };
    AccessibilityPreferences {
        reduced_motion,
        contrast: if high_contrast {
            Contrast::More
        } else {
            Contrast::NoPreference
        },
        ..AccessibilityPreferences::default()
    }
}

impl A11yMediaApi for Portal {
    fn system(&self) -> AccessibilityPreferences {
        Connection::new_session()
            .map(|connection| read(&connection))
            .unwrap_or_default()
    }

    /// A thread waits on the portal's `SettingChanged` and re-reads them all.
    fn on_change(
        &self,
        callback: Box<dyn Fn(AccessibilityPreferences)>,
    ) -> Box<dyn A11yMediaSubscription> {
        let (sender, mut receiver) = mpsc::unbounded();
        std::thread::spawn(move || watch(sender));
        let task = spawn(async move {
            while let Some(preferences) = poll_fn(|cx| Pin::new(&mut receiver).poll_next(cx)).await
            {
                callback(preferences);
            }
        });
        Box::new(PortalSubscription(task))
    }
}

fn watch(sender: mpsc::UnboundedSender<AccessibilityPreferences>) {
    let Ok(connection) = Connection::new_session() else {
        return;
    };
    let notify = sender.clone();
    let changed = MatchRule::new_signal(SETTINGS, "SettingChanged");
    let added = connection.add_match(changed, move |(namespace,): (String,), connection, _| {
        if [APPEARANCE, GNOME_INTERFACE, GNOME_A11Y].contains(&namespace.as_str()) {
            let _ = notify.unbounded_send(read(connection));
        }
        true
    });
    if added.is_err() {
        return;
    }
    // Ends within a second of the subscription's drop.
    while !sender.is_closed() && connection.process(Duration::from_secs(1)).is_ok() {}
}

struct PortalSubscription(Task);

impl A11yMediaSubscription for PortalSubscription {}

impl Drop for PortalSubscription {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reads the desktop this runs on: only that the calls go through.
    #[test]
    #[ignore = "reads the session bus of the machine it runs on"]
    fn the_portal_answers() {
        let connection = Connection::new_session().expect("a session bus");
        let animations = setting(&connection, GNOME_INTERFACE, "enable-animations");
        eprintln!(
            "enable-animations {animations:?}, read {:?}",
            read(&connection)
        );
        assert!(animations.is_some());
    }
}
