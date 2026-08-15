use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{SxLayer, context::LiberoContext, sx::Sx};

/// Registers an [`Sx`](crate::sx::Sx) value and returns the class name to put
/// on any element - including ones libero doesn't provide a component for
/// (e.g. a raw `select`). Uses the `UserCustom` layer, which has priority
/// over every other layer (including a component's own dynamic prop-driven
/// styles), so it always wins.
pub fn use_class(sx: &Sx) -> Option<String> {
    use_sx(sx, SxLayer::UserCustom)
}

pub(crate) fn use_sx(sx: &Sx, layer: SxLayer) -> Option<String> {
    let mut context = use_context::<LiberoContext>();
    let key = (layer, sx.hash());
    let active_registration = use_hook(|| Rc::new(RefCell::new(None::<(SxLayer, u64)>)));

    {
        let mut active_registration = active_registration.borrow_mut();

        if sx.is_empty() {
            if let Some(active_key) = active_registration.take() {
                context.sx_registry.release(active_key);
                *context.sx_registry_version.write() += 1;
            }
        } else {
            match *active_registration {
                Some(active_key) if active_key == key => {}
                Some(active_key) => {
                    context.sx_registry.release(active_key);
                    context.sx_registry.acquire(sx, layer);
                    *active_registration = Some(key);
                    *context.sx_registry_version.write() += 1;
                }
                None => {
                    context.sx_registry.acquire(sx, layer);
                    *active_registration = Some(key);
                    *context.sx_registry_version.write() += 1;
                }
            }
        }
    }

    {
        let active_registration = active_registration.clone();
        let sx_registry = context.sx_registry.clone();
        let mut sx_registry_version = context.sx_registry_version;
        use_drop(move || {
            if let Some(active_key) = active_registration.borrow_mut().take() {
                sx_registry.release(active_key);
                *sx_registry_version.write() += 1;
            }
        });
    }

    if sx.is_empty() {
        None
    } else {
        Some(sx.class_name())
    }
}
