use dioxus::prelude::*;

use crate::platform::clipboard;

/// Writes to the clipboard and tracks "copied" and "failed" flags the caller clears.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_clipboard;
/// # fn app() -> Element {
/// let mut clipboard = use_clipboard();
/// let status = if clipboard.copied() { "Copied" } else if clipboard.failed() { "Copy failed" } else { "Copy" };
///
/// rsx! {
///     button {
///         onclick: move |_| {
///             clipboard.reset();
///             clipboard.copy("text");
///         },
///         "{status}"
///     }
/// }
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct Clipboard {
    copied: Signal<bool>,
    failed: Signal<bool>,
}

impl Clipboard {
    /// Copies `text`. `copied()` flips once the platform confirms; a denied
    /// write, or a target without a clipboard, raises `failed()` instead.
    pub fn copy(&mut self, text: impl Into<String>) {
        let Some(clipboard) = clipboard() else {
            crate::utils::warn("clipboard write on a target without one");
            self.settle(false);
            return;
        };
        let write = clipboard.write_text(text.into());
        let mut this = *self;
        spawn(async move {
            let written = write.await.is_ok();
            if !written {
                // A denied permission, the common failure on the web.
                crate::utils::warn("clipboard write failed");
            }
            this.settle(written);
        });
    }

    fn settle(&mut self, written: bool) {
        self.copied.set(written);
        self.failed.set(!written);
    }

    /// Clears both `copied()` and `failed()`.
    pub fn reset(&mut self) {
        self.copied.set(false);
        self.failed.set(false);
    }

    /// Whether the last copy landed. Reactive.
    pub fn copied(&self) -> bool {
        (self.copied)()
    }

    /// Whether the last copy failed. Reactive.
    pub fn failed(&self) -> bool {
        (self.failed)()
    }
}

/// A [`Clipboard`] for this component. [`CopyButton`](crate::components::CopyButton)
/// is the ready-made control built on it.
pub fn use_clipboard() -> Clipboard {
    Clipboard {
        copied: use_signal(|| false),
        failed: use_signal(|| false),
    }
}
