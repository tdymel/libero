use dioxus::prelude::*;

use crate::platform::clipboard;

/// Writes to the clipboard and tracks a "just copied" flag the caller clears.
#[derive(Clone, Copy)]
pub struct Clipboard {
    copied: Signal<bool>,
}

impl Clipboard {
    /// `copied()` only flips once the platform confirms the write.
    pub fn copy(&mut self, text: impl Into<String>) {
        let Some(clipboard) = clipboard() else {
            crate::utils::warn("clipboard write on a target without one");
            return;
        };
        let write = clipboard.write_text(text.into());
        let mut copied = self.copied;
        spawn(async move {
            if write.await.is_ok() {
                copied.set(true);
            }
        });
    }

    pub fn reset(&mut self) {
        self.copied.set(false);
    }

    pub fn copied(&self) -> bool {
        (self.copied)()
    }
}

pub fn use_clipboard() -> Clipboard {
    Clipboard {
        copied: use_signal(|| false),
    }
}
