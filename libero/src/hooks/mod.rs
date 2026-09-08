mod cache;
mod clipboard;
mod color_scheme;
mod dismiss;
mod drag;
mod element;
mod focus_return;
mod id;
mod local_state;
mod popover;
mod portal;
mod presence;
mod ripple;
mod stylesheet;
mod theme;
mod typeahead;

pub(crate) use cache::use_cache;
pub use clipboard::{Clipboard, use_clipboard};
pub use color_scheme::{ColorSchemeHandle, use_color_scheme};
pub(crate) use dismiss::{
    DismissHandle, DismissOptions, escape_closes, use_dismiss, use_dismiss_layer,
    use_field_list_layer,
};
pub use drag::{Drag, DragMove, DragOptions, DragPoint, DragStart, drag_handle_sx, use_drag};
pub use element::{ElementHandle, use_element};
pub use focus_return::{FocusReturn, use_focus_return};
pub(crate) use id::id_selector;
pub use id::{use_id, use_root_id};
pub(crate) use local_state::{LocalState, use_local_state};
pub(crate) use popover::use_popover_on;
pub use popover::{
    Align, Placed, Placement, PopoverHandle, PopoverOptions, PopoverWidth, Rect, Side, use_popover,
};
pub use portal::use_portal;
pub use presence::{Presence, use_presence};
pub(crate) use ripple::{ripple_sx, use_ripple};
pub use stylesheet::use_stylesheet;
pub(crate) use stylesheet::{SxSource, use_box_css, use_css};
pub use theme::{ThemeSetHandle, use_theme, use_theme_set};
pub(crate) use typeahead::{TYPEAHEAD_RESET, Typeahead, typeahead_match, use_typeahead};

// The overlay hooks render a component, so they live beside it (todo 178).
// Re-exported here, where they have always been public. The layer-order
// guard (`tests/all/layer_order.rs`) allows these lines and nothing else.
pub use crate::components::overlay::{
    use_drawer::{DrawerOptions, use_drawer},
    use_floating_window::{FloatingWindowHandle, use_floating_window},
    use_lightbox::{LightboxItem, LightboxOpening, LightboxOptions, use_lightbox},
    use_modal::{ModalHandle, ModalScope, Opening, OpeningFuture, use_modal, use_modal_close},
};
