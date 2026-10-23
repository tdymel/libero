use dioxus::prelude::*;

use super::use_modal::{ModalHandle, ModalScope, use_modal};
use crate::{
    components::{
        common::Input,
        overlay::{Drawer, DrawerAnchor},
    },
    context::Dismiss,
    sx::{Sx, ThemeAwareValue},
};

/// How the panel docks, shared by every opening.
#[derive(Clone, Default, PartialEq)]
pub struct DrawerOptions {
    /// The edge it docks to.
    pub anchor: Input<DrawerAnchor>,
    /// Width when docked start or end, height when docked top or bottom: a size word or any
    /// CSS, as `size: "24rem".into()`.
    pub size: Input<ThemeAwareValue>,
    pub z_index: Input<ThemeAwareValue>,
    /// Names the panel, which is a dialog. With neither this nor `title` it is a `warn()`.
    pub aria_label: Option<String>,
    /// A visible heading at the top of the panel that names it.
    pub title: Option<String>,
    /// Styles the panel. Its content is yours; a bottom drawer's swipe strip is
    /// `& > [data-slot='handle']`.
    pub sx: Input<Sx>,
    /// Whether Escape, the backdrop, Back or a swipe may close it; `false` keeps it
    /// open. Unset, all close. As [`Dialog`](crate::components::Dialog)'s.
    pub ondismiss: Option<Callback<Dismiss, bool>>,
    /// Docked bottom, a touch swipe down from the panel's top padding closes it;
    /// `true` draws a grab bar there. Other anchors ignore it.
    pub handle: bool,
    /// Draws a close button in the header, beside `title`. As `Dialog`'s, it closes
    /// without asking `ondismiss`.
    pub close_button: bool,
}

/// A drawer is [`use_modal`] with a docked panel around the content: same
/// handle, same openings, same results.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::Anchor;
/// # use libero::hooks::{DrawerOptions, ModalScope, use_drawer};
/// # fn app() -> Element {
/// let options = DrawerOptions {
///     anchor: "end".into(),
///     aria_label: Some("Navigation".into()),
///     ..Default::default()
/// };
/// let nav = use_drawer(options, |s: ModalScope<()>| rsx! {
///     Anchor { to: "/", "Home" }
/// });
/// nav.open();
/// # rsx! {}
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/overlay/drawer>
pub fn use_drawer<S, R>(
    options: DrawerOptions,
    mut render: impl FnMut(ModalScope<S, R>) -> Element + 'static,
) -> ModalHandle<S, R>
where
    S: Clone + 'static,
    R: Clone + 'static,
{
    use_modal(move |scope| {
        let content = render(scope);

        rsx! {
            Drawer {
                anchor: options.anchor.clone(),
                size: options.size.clone(),
                z_index: options.z_index.clone(),
                aria_label: options.aria_label.clone(),
                title: options.title.clone(),
                sx: options.sx.clone(),
                ondismiss: options.ondismiss,
                handle: options.handle,
                close_button: options.close_button,
                {content}
            }
        }
    })
}
