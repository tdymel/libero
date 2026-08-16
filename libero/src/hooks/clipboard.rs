use dioxus::{document, prelude::*};

/// Writes to the system clipboard and tracks a transient "just copied" flag
/// - callers decide when to clear it (e.g. `onmouseleave`).
#[derive(Clone, Copy)]
pub struct Clipboard {
    copied: Signal<bool>,
}

impl Clipboard {
    /// Only ever called from a native event handler (`onclick`) - calling
    /// `document::eval` from `use_effect` hangs `dx serve`'s debug build.
    pub fn copy(&mut self, text: impl Into<String>) {
        let text = text.into();
        document::eval(&format!(
            r#"await new Promise(function(r) {{ setTimeout(r, 0); }});
            await navigator.clipboard.writeText({text:?});"#
        ));
        self.copied.set(true);
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
