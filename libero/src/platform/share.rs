//! Handing a link, a text or files to the system share sheet.

use std::{future::Future, pin::Pin};

/// What to share. At least one field must be set; blank strings count as unset.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShareData {
    /// A title, used as the subject where the target has one.
    pub title: Option<String>,
    /// A message.
    pub text: Option<String>,
    /// A link.
    pub url: Option<String>,
    /// Files; a target without file sharing still gets the title, text and url.
    pub files: Vec<ShareFile>,
}

/// One file in [`ShareData::files`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShareFile {
    /// The file name the receiving app shows.
    pub name: String,
    /// The MIME type, `image/png` say.
    pub mime: String,
    /// The content.
    pub bytes: Vec<u8>,
}

impl ShareData {
    /// A [`ShareData`] with only a link.
    pub fn url(url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            ..Self::default()
        }
    }

    fn field(value: &Option<String>) -> Option<&str> {
        value.as_deref().filter(|value| !value.trim().is_empty())
    }

    /// Whether nothing is set.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
            && [&self.title, &self.text, &self.url]
                .into_iter()
                .all(|value| Self::field(value).is_none())
    }

    /// What a target without a share sheet copies instead: the url, else the text, else the title.
    pub(crate) fn fallback_text(&self) -> Option<String> {
        [&self.url, &self.text, &self.title]
            .into_iter()
            .find_map(Self::field)
            .map(str::to_string)
    }
}

/// What became of a share.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShareOutcome {
    /// Handed to the share sheet. On Android this means the sheet opened.
    Shared,
    /// No share sheet, so the link or text went to the clipboard instead
    /// ([`use_share`](crate::hooks::use_share) only).
    Copied,
    /// The user closed the share sheet, or it had no target.
    Cancelled,
    /// The target has no share sheet for this data.
    Unsupported,
    /// Nothing was shared, with the reason.
    Failed(String),
}

/// The reason when [`ShareData`] holds nothing.
pub(crate) const NOTHING_TO_SHARE: &str = "nothing to share";

pub(crate) type Sharing = Pin<Box<dyn Future<Output = ShareOutcome>>>;

/// One share sheet per target. Must start the share before returning: the
/// browser allows it only within the user's press.
pub(crate) trait ShareApi {
    fn share(&self, data: ShareData) -> Sharing;
}

/// Opens the system share sheet with `data`. The sheet opens on the call,
/// so call it in the press handler; the future reports the outcome.
///
/// Returns [`ShareOutcome::Unsupported`] where no share sheet exists:
/// [`use_share`](crate::hooks::use_share) copies the link there instead.
///
/// Docs: <https://libero-ui.dev/buttons/share-button>
/// ```rust,no_run
/// use libero::platform::{ShareData, ShareOutcome, share};
///
/// async fn invite() {
///     let outcome = share(ShareData::url("https://libero-ui.dev")).await;
///     if let ShareOutcome::Failed(reason) = outcome {
///         eprintln!("share failed: {reason}");
///     }
/// }
/// ```
pub fn share(data: ShareData) -> impl Future<Output = ShareOutcome> {
    if data.is_empty() {
        crate::utils::warn("share called with nothing to share");
        return Box::pin(std::future::ready(ShareOutcome::Failed(
            NOTHING_TO_SHARE.to_string(),
        ))) as Sharing;
    }
    match share_api() {
        Some(api) => api.share(data),
        None => Box::pin(std::future::ready(ShareOutcome::Unsupported)),
    }
}

/// Shares through `navigator.share`, files only where `canShare` takes them.
/// Answers `shared`, `cancelled`, `unsupported` or the error's name and message.
#[cfg(any(
    target_arch = "wasm32",
    all(not(target_os = "android"), not(feature = "native"))
))]
pub(crate) const SHARE_SCRIPT: &str = "
    const share = async (title, text, url, files) => {
        if (typeof navigator.share !== 'function') return 'unsupported';
        const payload = {};
        if (title != null) payload.title = title;
        if (text != null) payload.text = text;
        if (url != null) payload.url = url;
        const checks = typeof navigator.canShare === 'function';
        if (files.length && checks) {
            const list = files.map(([name, type, bytes]) => new File([typeof bytes === 'string'
                ? Uint8Array.from(atob(bytes), (char) => char.charCodeAt(0)) : bytes], name, { type }));
            if (navigator.canShare({ files: list })) payload.files = list;
        }
        if (!Object.keys(payload).length) return 'unsupported';
        if (checks && !navigator.canShare(payload)) return 'unsupported';
        try {
            await navigator.share(payload);
            return 'shared';
        } catch (error) {
            return error?.name === 'AbortError' ? 'cancelled' : `${error?.name}: ${error?.message}`;
        }
    };";

/// The [`ShareOutcome`] for [`SHARE_SCRIPT`]'s answer.
#[cfg(any(
    test,
    target_arch = "wasm32",
    all(not(target_os = "android"), not(feature = "native"))
))]
pub(crate) fn outcome_of(answer: Option<String>) -> ShareOutcome {
    match answer.as_deref() {
        Some("shared") => ShareOutcome::Shared,
        Some("cancelled") => ShareOutcome::Cancelled,
        Some("unsupported") => ShareOutcome::Unsupported,
        Some(reason) => ShareOutcome::Failed(reason.to_string()),
        None => ShareOutcome::Failed("the share script did not answer".to_string()),
    }
}

/// The non-blank title, text and url, in [`SHARE_SCRIPT`]'s argument order.
#[cfg(any(target_arch = "wasm32", not(feature = "native")))]
pub(crate) fn text_fields(data: &ShareData) -> [Option<&str>; 3] {
    [&data.title, &data.text, &data.url].map(ShareData::field)
}

#[cfg(target_arch = "wasm32")]
mod web {
    use js_sys::{Array, Function, Promise, Uint8Array};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    use super::{SHARE_SCRIPT, ShareApi, ShareData, Sharing, outcome_of, text_fields};

    pub(super) struct WebShare;

    impl ShareApi for WebShare {
        fn share(&self, data: ShareData) -> Sharing {
            let script = Function::new_with_args(
                "title, text, url, files",
                &format!("{SHARE_SCRIPT}\nreturn share(title, text, url, files);"),
            );
            let files: Array = data
                .files
                .iter()
                .map(|file| {
                    let bytes = Uint8Array::from(file.bytes.as_slice());
                    Array::of3(
                        &file.name.as_str().into(),
                        &file.mime.as_str().into(),
                        &bytes,
                    )
                })
                .collect();
            let values: Array = text_fields(&data)
                .into_iter()
                .map(|field| field.map_or(JsValue::NULL, JsValue::from_str))
                .collect();
            values.push(&files);
            // Runs `navigator.share` now, inside the press's activation.
            let answer = script.apply(&JsValue::NULL, &values);
            Box::pin(async move {
                let Some(promise) = answer
                    .ok()
                    .and_then(|answer| answer.dyn_into::<Promise>().ok())
                else {
                    return outcome_of(None);
                };
                outcome_of(
                    JsFuture::from(promise)
                        .await
                        .ok()
                        .and_then(|answer| answer.as_string()),
                )
            })
        }
    }

    pub(super) static SHARE: WebShare = WebShare;
}

#[cfg(test)]
thread_local! {
    static FAKE: std::cell::Cell<Option<&'static dyn ShareApi>> = const { std::cell::Cell::new(None) };
}

/// Puts back the platform's share sheet when dropped.
#[cfg(test)]
pub(crate) struct FakeShareGuard;

#[cfg(test)]
impl Drop for FakeShareGuard {
    fn drop(&mut self) {
        FAKE.with(|fake| fake.set(None));
    }
}

/// Answers `share` with `fake` on this thread until the guard drops.
#[cfg(test)]
pub(crate) fn fake_share(fake: &'static dyn ShareApi) -> FakeShareGuard {
    FAKE.with(|slot| slot.set(Some(fake)));
    FakeShareGuard
}

/// `None` where the target has no share sheet: Blitz, a server render.
fn share_api() -> Option<&'static dyn ShareApi> {
    #[cfg(test)]
    if let Some(fake) = FAKE.with(std::cell::Cell::get) {
        return Some(fake);
    }
    #[cfg(target_arch = "wasm32")]
    return Some(&web::SHARE);
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        not(feature = "native")
    ))]
    return super::backend::webview_share();
    #[cfg(all(target_os = "android", not(feature = "native")))]
    return Some(&super::android_share::SHARE);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return None;
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        let mut future = std::pin::pin!(future);
        let waker = std::task::Waker::noop();
        match future
            .as_mut()
            .poll(&mut std::task::Context::from_waker(waker))
        {
            std::task::Poll::Ready(value) => value,
            std::task::Poll::Pending => panic!("the fake answers at once"),
        }
    }

    struct Recording;

    thread_local! {
        static SHARED: RefCell<Vec<ShareData>> = const { RefCell::new(Vec::new()) };
    }

    impl ShareApi for Recording {
        fn share(&self, data: ShareData) -> Sharing {
            SHARED.with_borrow_mut(|shared| shared.push(data));
            Box::pin(std::future::ready(ShareOutcome::Cancelled))
        }
    }

    static RECORDING: Recording = Recording;

    #[test]
    fn share_hands_the_data_to_the_platform_before_it_is_awaited() {
        let _fake = fake_share(&RECORDING);
        let data = ShareData {
            title: Some("Libero".into()),
            files: vec![ShareFile {
                name: "a.txt".into(),
                mime: "text/plain".into(),
                bytes: b"a".to_vec(),
            }],
            ..ShareData::url("https://libero-ui.dev")
        };
        let sharing = share(data.clone());
        assert_eq!(SHARED.with_borrow(Clone::clone), [data]);
        assert_eq!(block_on(sharing), ShareOutcome::Cancelled);
    }

    #[test]
    fn nothing_to_share_fails_without_asking_the_platform() {
        let _fake = fake_share(&RECORDING);
        let blank = ShareData {
            text: Some("  ".into()),
            ..ShareData::default()
        };
        assert_eq!(
            block_on(share(blank)),
            ShareOutcome::Failed(NOTHING_TO_SHARE.to_string())
        );
        assert!(SHARED.with_borrow(Vec::is_empty));
    }

    #[cfg(not(any(feature = "desktop", target_os = "android")))]
    #[test]
    fn share_is_unsupported_without_a_page() {
        assert_eq!(
            block_on(share(ShareData::url("https://libero-ui.dev"))),
            ShareOutcome::Unsupported
        );
    }

    #[test]
    fn the_fallback_copies_the_url_else_the_text_else_the_title() {
        let all = ShareData {
            title: Some("title".into()),
            text: Some("text".into()),
            ..ShareData::url("url")
        };
        assert_eq!(all.fallback_text().as_deref(), Some("url"));
        let text = ShareData {
            url: Some(" ".into()),
            ..all.clone()
        };
        assert_eq!(text.fallback_text().as_deref(), Some("text"));
        let title = ShareData {
            title: Some("title".into()),
            ..ShareData::default()
        };
        assert_eq!(title.fallback_text().as_deref(), Some("title"));
        assert_eq!(ShareData::default().fallback_text(), None);
    }

    #[test]
    fn the_script_answer_maps_to_an_outcome() {
        let answer = |text: &str| outcome_of(Some(text.to_string()));
        assert_eq!(answer("shared"), ShareOutcome::Shared);
        assert_eq!(answer("cancelled"), ShareOutcome::Cancelled);
        assert_eq!(answer("unsupported"), ShareOutcome::Unsupported);
        assert_eq!(
            answer("NotAllowedError: denied"),
            ShareOutcome::Failed("NotAllowedError: denied".into())
        );
        assert!(matches!(outcome_of(None), ShareOutcome::Failed(_)));
    }
}
