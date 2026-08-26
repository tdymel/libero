use std::{
    future::Future,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use dioxus::prelude::*;

use crate::components::common::{Dimensions, ElementApi, PlatformError};

/// An [`ElementApi`] over a mounted element, so the same calls work wherever
/// dioxus renders - there is no `document` to query off the web, but every
/// renderer can measure and focus an element it has mounted.
///
/// Obtained from [`use_element`](crate::hooks::use_element), never built
/// directly.
pub(crate) struct MountedElement(pub(crate) Rc<MountedData>);

impl ElementApi for MountedElement {
    fn focus(&self) -> Result<(), PlatformError> {
        ready(self.0.set_focus(true))
    }

    fn blur(&self) -> Result<(), PlatformError> {
        ready(self.0.set_focus(false))
    }

    fn dimensions(&self) -> Result<Dimensions, PlatformError> {
        let rect = ready(self.0.get_client_rect())?;
        Ok(Dimensions {
            width: rect.size.width,
            height: rect.size.height,
        })
    }

    fn client_offset(&self) -> Result<(f64, f64), PlatformError> {
        let rect = ready(self.0.get_client_rect())?;
        Ok((rect.origin.x, rect.origin.y))
    }

    fn scroll_size(&self) -> Result<Dimensions, PlatformError> {
        let size = ready(self.0.get_scroll_size())?;
        Ok(Dimensions {
            width: size.width,
            height: size.height,
        })
    }

    fn scroll_offset(&self) -> Result<(f64, f64), PlatformError> {
        let offset = ready(self.0.get_scroll_offset())?;
        Ok((offset.x, offset.y))
    }

    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError> {
        ready(self.0.scroll(
            dioxus::html::geometry::PixelsVector2D::new(x, y),
            ScrollBehavior::Instant,
        ))
    }

    /// No mounted-element equivalent: dioxus exposes capture nowhere, so a
    /// drag that needs it has to reach for [`dom_api`](super::dom_api).
    fn set_pointer_capture(&self, _pointer_id: i32) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn click(&self) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// A mounted handle addresses one element; it cannot answer for the
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

/// Every renderer computes these eagerly and hands back a resolved future, so
/// one poll is always enough - and `ElementApi` is sync because its callers
/// (a drag reacting to the pointer event in hand) have nowhere to await. A
/// renderer that ever answers asynchronously reads as `Unsupported`, which is
/// the same answer these calls already give where the platform can't help.
fn ready<T>(future: impl Future<Output = MountedResult<T>>) -> Result<T, PlatformError> {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| RawWaker::new(std::ptr::null(), &VTABLE),
        |_| {},
        |_| {},
        |_| {},
    );
    // SAFETY: every entry is a no-op on a null pointer none of them reads.
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };

    let mut future = std::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(&waker))
    {
        Poll::Ready(Ok(value)) => Ok(value),
        Poll::Ready(Err(_)) | Poll::Pending => Err(PlatformError::Unsupported),
    }
}
