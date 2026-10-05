use std::rc::Rc;

use dioxus::prelude::*;

use crate::platform::{Dimensions, ElementApi, PlatformError, Read};

/// An [`ElementApi`] over `MountedData`: measure, scroll and focus on any
/// renderer. The floor [`backend::element`](super::element) falls back to.
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

    /// `MountedData` has no way to reach an image's own data.
    fn natural_size(&self) -> Read<Dimensions> {
        Box::pin(std::future::ready(Err(PlatformError::Unsupported)))
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

    /// dioxus's `scrollIntoView`, side effects included: the only way a mounted
    /// handle scrolls an ancestor.
    fn scroll_into_view(&self, smooth: bool) -> Result<(), PlatformError> {
        let element = self.0.clone();
        let behavior = if smooth {
            ScrollBehavior::Smooth
        } else {
            ScrollBehavior::Instant
        };
        queue(async move {
            element
                .scroll_to_with_options(ScrollToOptions {
                    behavior,
                    vertical: ScrollLogicalPosition::Nearest,
                    horizontal: ScrollLogicalPosition::Nearest,
                })
                .await
        })
    }

    /// No mounted handle exposes pointer capture.
    fn set_pointer_capture(&self, _pointer_id: i32) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn click(&self) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// A form's reset and submit are DOM methods no mounted handle reaches.
    fn reset(&self) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn request_submit(&self) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// A `FileList` is a browser object; no mounted handle reaches one.
    fn set_files(&self, _files: &[dioxus::html::FileData]) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// A bare handle cannot see the document's focus or search below itself.
    fn is_focused(&self) -> bool {
        false
    }

    /// Cannot tell, so `true`, the trait's rule: focus return keeps working.
    fn is_connected(&self) -> bool {
        true
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

/// Offset and size from one `getBoundingClientRect`.
pub(super) fn client_rect(element: Rc<MountedData>) -> Read<((f64, f64), Dimensions)> {
    Box::pin(async move {
        let rect = failed(element.get_client_rect().await)?;
        Ok((
            (rect.origin.x, rect.origin.y),
            Dimensions {
                width: rect.size.width,
                height: rect.size.height,
            },
        ))
    })
}

/// A command nobody reads back: under a WebView an IPC round-trip, so `Ok` means
/// queued.
fn queue(
    future: impl std::future::Future<Output = MountedResult<()>> + 'static,
) -> Result<(), PlatformError> {
    spawn(async move {
        let _ = future.await;
    });
    Ok(())
}

/// Why a read failed is never actionable: the caller skips the measurement.
fn failed<T>(result: MountedResult<T>) -> Result<T, PlatformError> {
    result.map_err(|_| PlatformError::Unsupported)
}
