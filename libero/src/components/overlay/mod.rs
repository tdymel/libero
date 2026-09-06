mod drawer;
mod floating_window;
mod hover_card;
mod lightbox;
mod modal;
mod overlay;
mod spotlight;
mod tooltip;
// The hooks that render an overlay. Public through `hooks`, where they
// have always been named; the modules are `pub(crate)` so `hooks` can reach
// them without `components` exporting them twice.
pub(crate) mod use_drawer;
pub(crate) mod use_floating_window;
pub(crate) mod use_lightbox;
pub(crate) mod use_modal;

pub(crate) use drawer::Drawer;
pub use drawer::DrawerAnchor;
pub(crate) use floating_window::FloatingWindow;
pub use floating_window::{FloatingWindowOptions, WindowRect};
pub use hover_card::{HoverCard, HoverCardProps};
pub(crate) use lightbox::Lightbox;
pub(crate) use modal::Modal;
pub use overlay::{Overlay, OverlayProps};
pub use spotlight::{
    SpotlightAction, SpotlightHandle, SpotlightOptions, spotlight_filter, use_spotlight,
};
pub use tooltip::{Tooltip, TooltipProps};
