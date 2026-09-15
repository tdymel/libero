use std::rc::Rc;

use dioxus::{
    html::{FileData, FormData, FormValue, HasFileData, HasFormData},
    prelude::*,
};

use super::backend;

/// `Form`'s `onclick` and `onkeydown` where the renderer fires no `submit` of
/// its own (Blitz): a submit button's click and Enter in a text field run
/// `submit`. `None` where it does.
pub(crate) fn submit_listeners(
    form: impl Fn() -> Option<Rc<MountedData>> + Copy + 'static,
    submit: Callback<FormEvent>,
) -> Option<(
    impl FnMut(MouseEvent) + 'static,
    impl FnMut(KeyboardEvent) + 'static,
)> {
    backend::emulates_submit().then_some((
        move |event: MouseEvent| {
            if event.default_action_enabled() && submit_click(form()) {
                submit.call(submit_event(form()));
            }
        },
        move |event: KeyboardEvent| {
            if implicit_submit(&event, form()) {
                event.prevent_default();
                submit.call(submit_event(form()));
            }
        },
    ))
}

/// Whether a click inside `form` activated one of its submit buttons.
fn submit_click(form: Option<Rc<MountedData>>) -> bool {
    form.is_some_and(|form| backend::activated_submitter(&form))
}

/// Enter in a text field of `form`, which submits it on the web when the form
/// has a submit button, or only one such field; in a checkbox or radio, only
/// through a submit button.
fn implicit_submit(event: &Event<KeyboardData>, form: Option<Rc<MountedData>>) -> bool {
    event.key() == Key::Enter
        && event.default_action_enabled()
        && !event.is_composing()
        && form.is_some_and(|form| backend::implicit_submission(&form))
}

/// The `submit` the renderer did not fire, for `Form`'s own handler. Carries
/// the named controls' values where the renderer can read them.
pub(crate) fn submit_event(form: Option<Rc<MountedData>>) -> FormEvent {
    let values = form
        .map(|form| backend::form_values(&form))
        .unwrap_or_default();
    Event::new(Rc::new(FormData::new(Submitted(values))), false)
}

#[derive(Clone)]
struct Submitted(Vec<(String, FormValue)>);

impl HasFormData for Submitted {
    fn value(&self) -> String {
        String::new()
    }

    fn valid(&self) -> bool {
        true
    }

    fn values(&self) -> Vec<(String, FormValue)> {
        self.0.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl HasFileData for Submitted {
    fn files(&self) -> Vec<FileData> {
        Vec::new()
    }
}
