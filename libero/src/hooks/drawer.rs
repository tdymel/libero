use dioxus::prelude::*;

use crate::{
    components::{Drawer, DrawerAnchor, Input},
    hooks::{ModalHandle, ModalScope, use_modal},
    sx::ThemeAwareValue,
    theme::Size,
};

/// How the panel docks, shared by every opening.
#[derive(Clone, Default, PartialEq)]
pub struct DrawerOptions {
    /// The edge it docks to.
    pub anchor: Input<DrawerAnchor>,
    /// Width when docked left or right, height when docked top or bottom.
    pub size: Input<Size>,
    pub z_index: Input<ThemeAwareValue>,
}

/// A drawer is [`use_modal`] with a docked panel around the content: same
/// handle, same openings, same results.
///
/// ```ignore
/// let options = DrawerOptions { anchor: "right".into(), ..Default::default() };
/// let nav = use_drawer(options, |s: ModalScope<()>| rsx! {
///     Anchor { to: "/", "Home" }
/// });
/// nav.open();
/// ```
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
                {content}
            }
        }
    })
}
