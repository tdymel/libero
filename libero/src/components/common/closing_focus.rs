use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    hooks::ElementHandle,
    platform::{ElementApi, PlatformError, document, when_free},
};

/// What a closing region owes: the selector to focus, or, where the render
/// could not ask, every `(region, target)` pair to ask again later.
enum Owed {
    Nothing,
    Known(String),
    Unknown(Vec<(String, String)>),
}

/// Focus return out of content that closes around focus: a "Continue" button
/// in an `Accordion` panel or a `Stepper` step. Asked during render, against
/// the DOM the previous render left; repaid from the owner's effect.
#[derive(Clone)]
pub(crate) struct ClosingFocus {
    root: ElementHandle,
    owed: Rc<RefCell<Owed>>,
}

pub(crate) fn use_closing_focus(root: ElementHandle) -> ClosingFocus {
    let owed = use_hook(|| Rc::new(RefCell::new(Owed::Nothing)));
    ClosingFocus { root, owed }
}

fn holds_focus(root: &ElementHandle, region: &str) -> Result<(), PlatformError> {
    root.query_selector(&format!("{region} :focus")).map(drop)
}

fn focus(root: &ElementHandle, target: &str) {
    let _ = root
        .query_selector(target)
        .and_then(|target| target.focus());
}

impl ClosingFocus {
    /// `region` (a selector under the root) is closing: if it holds focus,
    /// `target` takes it at [`repay`](Self::repay). `true` once that is known.
    pub(crate) fn closing(&self, region: String, target: String) -> bool {
        let mut owed = self.owed.borrow_mut();
        match holds_focus(&self.root, &region) {
            Ok(()) => {
                *owed = Owed::Known(target);
                true
            }
            // Blitz renders with the document borrowed: asked again once free.
            Err(PlatformError::Unsupported) => {
                match &mut *owed {
                    Owed::Unknown(pairs) => pairs.push((region, target)),
                    Owed::Nothing => *owed = Owed::Unknown(vec![(region, target)]),
                    Owed::Known(_) => {}
                }
                false
            }
            Err(_) => false,
        }
    }

    /// From the owner's effect. An unknown answer is asked at the end of the
    /// poll natively, while the closing content is still mounted.
    pub(crate) fn repay(&self) {
        let owed = std::mem::replace(&mut *self.owed.borrow_mut(), Owed::Nothing);
        let root = self.root;
        match owed {
            Owed::Nothing => {}
            Owed::Known(target) => focus(&root, &target),
            Owed::Unknown(pairs) => when_free(move || {
                // A region already gone took focus with it, if it held it:
                // Blitz leaves focus on `<html>`, the one element holding `body`.
                let lost = || {
                    document()
                        .and_then(|document| document.active_element())
                        .is_none_or(|active| {
                            !active.is_connected() || active.query_selector("body").is_ok()
                        })
                };
                let held = pairs.iter().find(|(region, _)| {
                    holds_focus(&root, region).is_ok()
                        || (root.query_selector(region).is_err() && lost())
                });
                if let Some((_, target)) = held {
                    focus(&root, target);
                }
            }),
        }
    }
}
