use std::rc::Rc;

use dioxus::prelude::*;

use crate::platform::{Dimensions, ElementApi, PlatformError, Read};

/// An [`ElementApi`] over a mounted element, so the same calls work wherever
/// dioxus renders - there is no `document` to query off the web, but every
/// renderer can measure and focus an element it has mounted.
///
/// The floor every renderer reaches: [`backend::element`](super::element)
/// falls back to this when the renderer's own handle isn't one we know.
pub(super) struct MountedElement(pub(super) Rc<MountedData>);

impl ElementApi for MountedElement {
    fn focus(&self) -> Result<(), PlatformError> {
        let element = self.0.clone();
        queue(async move { element.set_focus(true).await })
    }

    fn blur(&self) -> Result<(), PlatformError> {
        let element = self.0.clone();
        queue(async move { element.set_focus(false).await })
    }

    fn dimensions(&self) -> Read<Dimensions> {
        let element = self.0.clone();
        Box::pin(async move {
            let rect = failed(element.get_client_rect().await)?;
            Ok(Dimensions {
                width: rect.size.width,
                height: rect.size.height,
            })
        })
    }

    fn client_offset(&self) -> Read<(f64, f64)> {
        let element = self.0.clone();
        Box::pin(async move {
            let rect = failed(element.get_client_rect().await)?;
            Ok((rect.origin.x, rect.origin.y))
        })
    }

    fn scroll_size(&self) -> Read<Dimensions> {
        let element = self.0.clone();
        Box::pin(async move {
            let size = failed(element.get_scroll_size().await)?;
            Ok(Dimensions {
                width: size.width,
                height: size.height,
            })
        })
    }

    fn scroll_offset(&self) -> Read<(f64, f64)> {
        let element = self.0.clone();
        Box::pin(async move {
            let offset = failed(element.get_scroll_offset().await)?;
            Ok((offset.x, offset.y))
        })
    }

    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError> {
        let element = self.0.clone();
        queue(async move {
            element
                .scroll(
                    dioxus::html::geometry::PixelsVector2D::new(x, y),
                    ScrollBehavior::Instant,
                )
                .await
        })
    }

    /// dioxus exposes pointer capture on no mounted handle, so a drag only
    /// keeps tracking outside the element where a richer backend answered.
    fn set_pointer_capture(&self, _pointer_id: i32) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn click(&self) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// A `FileList` is a browser object; no mounted handle reaches one.
    fn set_files(&self, _files: &[dioxus::html::FileData]) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// A bare mounted handle addresses one element; it cannot answer for the
    /// document's focus or search below itself.
    fn is_focused(&self) -> bool {
        false
    }

    fn query_selector(&self, _selector: &str) -> Result<Box<dyn ElementApi>, PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn query_selector_all(
        &self,
        _selector: &str,
    ) -> Result<Vec<Box<dyn ElementApi>>, PlatformError> {
        Err(PlatformError::Unsupported)
    }
}

/// A command nobody reads a result back from. Under a webview it is an IPC
/// round-trip, so "queued" is the only thing answerable synchronously.
fn queue(
    future: impl std::future::Future<Output = MountedResult<()>> + 'static,
) -> Result<(), PlatformError> {
    spawn(async move {
        let _ = future.await;
    });
    Ok(())
}

/// Why a read failed is never actionable - the caller's only move is to skip
/// whatever needed the measurement.
fn failed<T>(result: MountedResult<T>) -> Result<T, PlatformError> {
    result.map_err(|_| PlatformError::Unsupported)
}
