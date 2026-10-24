use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{hooks::ElementHandle, platform::input_value};

type Check = Option<Box<dyn Fn(&Rc<MountedData>)>>;

/// Hands `emit` the text a browser wrote into `element` before it mounted, once per mount:
/// a saved-login autofill fills server-rendered HTML before hydration listens (todo 4094).
pub(crate) fn use_prefill(
    element: ElementHandle,
    shown: Option<String>,
    emit: Option<impl Fn(String) + 'static>,
    disabled: bool,
) {
    let check = use_hook(|| Rc::new(RefCell::new(None as Check)));
    *check.borrow_mut() = emit.filter(|_| !disabled).map(|emit| {
        Box::new(move |mounted: &Rc<MountedData>| {
            let text = input_value(mounted).filter(|text| !text.is_empty());
            if let Some(text) = text.filter(|text| shown.as_ref() != Some(text)) {
                emit(text);
            }
        }) as Box<dyn Fn(&Rc<MountedData>)>
    });
    use_effect(move || {
        let _ = element.mount_token();
        if let (Some(mounted), Some(check)) = (element.mounted(), &*check.borrow()) {
            check(&mounted);
        }
    });
}
