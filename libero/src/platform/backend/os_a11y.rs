//! Windows' and macOS' own accessibility settings, for Blitz there (todo 972).

use std::future::poll_fn;
use std::pin::Pin;
use std::time::Duration;

use dioxus::core::Task;
use dioxus::prelude::spawn;
use futures_channel::mpsc;
use futures_core::Stream;

use crate::platform::a11y_media::{A11yMediaApi, A11yMediaSubscription};
use crate::tokens::{AccessibilityPreferences, Contrast};

/// Neither system hands a change to a process without a window hook of its own.
const POLL: Duration = Duration::from_secs(1);

struct System;

static SYSTEM: System = System;

pub(super) fn a11y_media() -> Option<&'static dyn A11yMediaApi> {
    Some(&SYSTEM)
}

/// `SystemParametersInfoW`: no client area animation is reduced motion, high
/// contrast is forced colors, as Chromium and Firefox read them.
#[cfg(target_os = "windows")]
fn read() -> AccessibilityPreferences {
    use std::ffi::c_void;
    use windows_sys::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SPI_GETCLIENTAREAANIMATION, SPI_GETHIGHCONTRAST, SystemParametersInfoW,
    };

    let mut animation: i32 = 1;
    // SAFETY: each call writes at most the value it is handed, sized as the API asks.
    let animates = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            (&raw mut animation).cast::<c_void>(),
            0,
        )
    } == 0
        || animation != 0;
    let mut contrast = HIGHCONTRASTW {
        cbSize: size_of::<HIGHCONTRASTW>() as u32,
        dwFlags: 0,
        lpszDefaultScheme: std::ptr::null_mut(),
    };
    let high_contrast = unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            contrast.cbSize,
            (&raw mut contrast).cast::<c_void>(),
            0,
        )
    } != 0
        && contrast.dwFlags & HCF_HIGHCONTRASTON != 0;
    AccessibilityPreferences {
        reduced_motion: !animates,
        forced_colors: high_contrast,
        contrast: if high_contrast {
            Contrast::More
        } else {
            Contrast::NoPreference
        },
        ..AccessibilityPreferences::default()
    }
}

/// `NSWorkspace`'s display options, as Safari reads them.
#[cfg(target_os = "macos")]
fn read() -> AccessibilityPreferences {
    let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
    AccessibilityPreferences {
        reduced_motion: workspace.accessibilityDisplayShouldReduceMotion(),
        contrast: if workspace.accessibilityDisplayShouldIncreaseContrast() {
            Contrast::More
        } else {
            Contrast::NoPreference
        },
        reduced_transparency: workspace.accessibilityDisplayShouldReduceTransparency(),
        ..AccessibilityPreferences::default()
    }
}

impl A11yMediaApi for System {
    fn system(&self) -> AccessibilityPreferences {
        read()
    }

    /// A thread re-reads the settings each [`POLL`] and sends a change.
    fn on_change(
        &self,
        callback: Box<dyn Fn(AccessibilityPreferences)>,
    ) -> Box<dyn A11yMediaSubscription> {
        let (sender, mut receiver) = mpsc::unbounded();
        std::thread::spawn(move || watch(&sender));
        let task = spawn(async move {
            while let Some(preferences) = poll_fn(|cx| Pin::new(&mut receiver).poll_next(cx)).await
            {
                callback(preferences);
            }
        });
        Box::new(SystemSubscription(task))
    }
}

/// Ends within a [`POLL`] of the subscription's drop.
fn watch(sender: &mpsc::UnboundedSender<AccessibilityPreferences>) {
    let mut last = read();
    loop {
        std::thread::sleep(POLL);
        if sender.is_closed() {
            return;
        }
        let now = read();
        if now != last && sender.unbounded_send(now).is_err() {
            return;
        }
        last = now;
    }
}

struct SystemSubscription(Task);

impl A11yMediaSubscription for SystemSubscription {}

impl Drop for SystemSubscription {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
