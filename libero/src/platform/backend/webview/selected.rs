use std::rc::Rc;

use dioxus::html::geometry::{PixelsRect, PixelsSize, PixelsVector2D};
use dioxus::html::{
    MountedData, MountedError, MountedResult, RenderedElementBacking, ScrollBehavior,
    ScrollLogicalPosition, ScrollToOptions,
};
use serde_json::{Value, json};

use super::{eval_with, runs_scripts};
use crate::platform::{Dimensions, PlatformError, Read};

/// The document's first match of `selector`, found again on every call (todo 2666).
pub(crate) fn mounted_by_selector(selector: &str) -> Option<Rc<MountedData>> {
    runs_scripts().then(|| {
        Rc::new(MountedData::new(SelectedElement {
            selector: selector.to_string(),
        }))
    })
}

/// A `MountedData` backing with no node in Rust: the page looks `selector` up per call.
struct SelectedElement {
    selector: String,
}

/// `data` is `[selector, ...]`; the script sees the match as `found`, maybe `null`
/// (an invalid selector too).
const FIND_SELECTED: &str =
    "let found = null;\ntry { found = document.querySelector(data[0]); } catch {}";

type Mounted<T> = std::pin::Pin<Box<dyn std::future::Future<Output = MountedResult<T>>>>;

impl SelectedElement {
    /// Started now, in the calling scope; answered by the returned future.
    fn read<T: serde::de::DeserializeOwned + 'static>(&self, script: &'static str) -> Mounted<T> {
        let read = eval_with(
            json!([self.selector]),
            &format!("{FIND_SELECTED}\n{script}"),
        );
        Box::pin(async move {
            read.join::<T>()
                .await
                .map_err(|_| MountedError::NotSupported)
        })
    }

    fn command(&self, argument: Value, script: &'static str) -> Mounted<()> {
        eval_with(
            json!([self.selector, argument]),
            &format!("{FIND_SELECTED}\nconst value = data[1];\nif (found) {{ {script} }}"),
        );
        Box::pin(std::future::ready(Ok(())))
    }
}

impl RenderedElementBacking for SelectedElement {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    /// No match measures 0x0, which a measure takes for an element not rendered.
    fn get_client_rect(&self) -> Mounted<PixelsRect> {
        let read = self.read::<(f64, f64, f64, f64)>(
            "if (!found) return [0, 0, 0, 0];
            const rect = found.getBoundingClientRect();
            return [rect.x, rect.y, rect.width, rect.height];",
        );
        Box::pin(async move {
            let (x, y, width, height) = read.await?;
            Ok(PixelsRect::new(
                (x, y).into(),
                PixelsSize::new(width, height),
            ))
        })
    }

    fn get_scroll_offset(&self) -> Mounted<PixelsVector2D> {
        let read =
            self.read::<(f64, f64)>("return [found?.scrollLeft ?? 0, found?.scrollTop ?? 0];");
        Box::pin(async move {
            let (x, y) = read.await?;
            Ok(PixelsVector2D::new(x, y))
        })
    }

    fn get_scroll_size(&self) -> Mounted<PixelsSize> {
        let read =
            self.read::<(f64, f64)>("return [found?.scrollWidth ?? 0, found?.scrollHeight ?? 0];");
        Box::pin(async move {
            let (width, height) = read.await?;
            Ok(PixelsSize::new(width, height))
        })
    }

    fn scroll_to(&self, options: ScrollToOptions) -> Mounted<()> {
        let position = |at: ScrollLogicalPosition| match at {
            ScrollLogicalPosition::Start => "start",
            ScrollLogicalPosition::Center => "center",
            ScrollLogicalPosition::End => "end",
            ScrollLogicalPosition::Nearest => "nearest",
        };
        self.command(
            json!({
                "behavior": behavior(options.behavior),
                "block": position(options.vertical),
                "inline": position(options.horizontal),
            }),
            "found.scrollIntoView(value);",
        )
    }

    fn scroll(&self, coordinates: PixelsVector2D, scroll: ScrollBehavior) -> Mounted<()> {
        self.command(
            json!({ "left": coordinates.x, "top": coordinates.y, "behavior": behavior(scroll) }),
            "found.scrollTo(value);",
        )
    }

    fn set_focus(&self, focus: bool) -> Mounted<()> {
        self.command(json!(focus), "if (value) found.focus(); else found.blur();")
    }
}

/// An `<img>` found by selector reads its natural size page-side (961); `None` for any
/// other backing. Zero until decoded, reported as not there, as on the web.
pub(crate) fn selected_natural_size(mounted: &MountedData) -> Option<Read<Dimensions>> {
    let read = mounted
        .downcast::<SelectedElement>()?
        .read::<Option<(f64, f64)>>(
            "return found?.naturalWidth && found.naturalHeight
            ? [found.naturalWidth, found.naturalHeight] : null;",
        );
    Some(Box::pin(async move {
        match read.await {
            Ok(Some((width, height))) => Ok(Dimensions { width, height }),
            _ => Err(PlatformError::NotFound),
        }
    }))
}

fn behavior(behavior: ScrollBehavior) -> &'static str {
    match behavior {
        ScrollBehavior::Instant => "instant",
        ScrollBehavior::Smooth => "smooth",
    }
}
