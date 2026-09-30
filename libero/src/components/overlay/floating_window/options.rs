use dioxus::prelude::*;

use crate::{
    components::{
        common::{Input, Parts, parts_enum},
        layout::Placement,
        overlay::MenuPart,
    },
    sx::{Sx, ThemeAwareValue},
};

/// A window's position and size in viewport pixels, handed to
/// `onmove`/`onresize` so a caller can persist it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// How a window looks and behaves, shared by every opening.
#[derive(Clone, Default, PartialEq)]
pub struct FloatingWindowOptions {
    /// Title bar heading, and the accessible name unless `aria_label` is set.
    pub title: Option<String>,
    pub aria_label: Option<String>,
    /// Where it first appears. Dragging takes over from there.
    pub placement: Input<Placement>,
    /// Draws the corner resize handle.
    pub resizable: bool,
    /// Pins it where `placement` put it: no moving.
    pub pinned: bool,
    pub z_index: Input<ThemeAwareValue>,
    /// On the window itself; its `min_*`/`max_*` clamp a resize, within the viewport.
    /// Unset, the minimum is 12rem by 6rem.
    pub sx: Input<Sx>,
    /// Styles for the inner parts, under `sx`.
    pub parts: Input<Parts<FloatingWindowPart>>,
    /// The title-bar menu's `parts`: the menu is portaled, out of `parts`' reach.
    /// [`FloatingWindowPart::Menu`] stays its trigger.
    pub menu_parts: Input<Parts<MenuPart>>,
    /// After a drag, a keyboard or button move, or a Reset.
    pub onmove: Option<Callback<WindowRect>>,
    /// After a resize, by pointer, keyboard or button, or a Reset.
    pub onresize: Option<Callback<WindowRect>>,
}

parts_enum! {
    /// A floating window's inner parts, for [`FloatingWindowOptions::parts`].
    /// Matched as direct children, so a window's content cannot reach them.
    pub enum FloatingWindowPart {
        /// The row holding the move handle, the menu and the close button.
        TitleBar = "title-bar" => "& > [data-slot='title-bar']",
        /// The keyboard and pointer move handle around the title.
        Handle = "handle" => "& > [data-slot='title-bar'] > [data-slot='handle']",
        Title = "title" => "& > [data-slot='title-bar'] > [data-slot='handle'] > [data-slot='title']",
        /// The Move/Resize/Reset menu's trigger.
        Menu = "menu" => "& > [data-slot='title-bar'] [data-slot='menu']",
        Close = "close" => "& > [data-slot='title-bar'] > [data-slot='close']",
        /// The step buttons the menu's Move or Resize shows.
        Steps = "steps" => "& > [data-slot='steps']",
        Body = "body" => "& > [data-slot='body']",
        /// The corner resize handle, when `resizable`.
        Resize = "resize" => "& > [data-slot='resize']",
    }
}
