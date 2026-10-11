use dioxus::prelude::*;

use crate::platform::{ShareData, ShareOutcome, clipboard, share};

/// Opens the system share sheet and tracks the outcome. Where the target has
/// no share sheet, copies the link (else the text, else the title) instead.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_share;
/// # use libero::platform::ShareData;
/// # fn app() -> Element {
/// let mut share = use_share();
/// let status = if share.copied() { "Link copied" } else if share.failed() { "Share failed" } else { "" };
///
/// rsx! {
///     button {
///         onclick: move |_| share.share(ShareData::url("https://libero-ui.dev")),
///         "Share"
///     }
///     span { role: "status", "{status}" }
/// }
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct Share {
    outcome: Signal<Option<ShareOutcome>>,
    pending: Signal<bool>,
}

impl Share {
    /// Shares `data`. Call it in the press handler: the browser opens the
    /// sheet only within the press. A press while one is pending is ignored.
    pub fn share(&mut self, data: ShareData) {
        self.share_then(data, |_| {});
    }

    /// [`share`](Self::share), then `done` with the outcome.
    pub(crate) fn share_then(
        &mut self,
        data: ShareData,
        done: impl FnOnce(ShareOutcome) + 'static,
    ) {
        if *self.pending.peek() {
            return;
        }
        let fallback = data.fallback_text();
        let sharing = share(data);
        self.pending.set(true);
        self.outcome.set(None);
        let mut this = *self;
        spawn(async move {
            let mut outcome = sharing.await;
            if outcome == ShareOutcome::Unsupported
                && let Some(text) = fallback
            {
                outcome = copied(text).await;
            }
            this.pending.set(false);
            this.outcome.set(Some(outcome.clone()));
            done(outcome);
        });
    }

    /// Clears the outcome.
    pub fn reset(&mut self) {
        self.outcome.set(None);
    }

    /// The last share's outcome, `None` before one settles. Reactive.
    pub fn outcome(&self) -> Option<ShareOutcome> {
        (self.outcome)()
    }

    /// Whether the last share reached the share sheet. Reactive.
    pub fn shared(&self) -> bool {
        self.outcome() == Some(ShareOutcome::Shared)
    }

    /// Whether the last share fell back to the clipboard. Reactive.
    pub fn copied(&self) -> bool {
        self.outcome() == Some(ShareOutcome::Copied)
    }

    /// Whether the last share failed, or had neither a share sheet nor a
    /// text to copy. A cancelled sheet is no failure. Reactive.
    pub fn failed(&self) -> bool {
        matches!(
            self.outcome(),
            Some(ShareOutcome::Failed(_) | ShareOutcome::Unsupported)
        )
    }

    /// Whether a share is still open. Reactive.
    pub fn pending(&self) -> bool {
        (self.pending)()
    }
}

/// The fallback's outcome: `Copied`, or `Failed` when the clipboard refused.
async fn copied(text: String) -> ShareOutcome {
    let Some(clipboard) = clipboard() else {
        crate::utils::warn("share fallback on a target without a clipboard");
        return ShareOutcome::Failed("no share sheet and no clipboard".to_string());
    };
    match clipboard.write_text(text).await {
        Ok(()) => ShareOutcome::Copied,
        Err(error) => ShareOutcome::Failed(format!("clipboard: {error:?}")),
    }
}

/// A [`Share`] for this component. The [`Share`](crate::components::Share)
/// component is the ready-made control built on it.
pub fn use_share() -> Share {
    Share {
        outcome: use_signal(|| None),
        pending: use_signal(|| false),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
