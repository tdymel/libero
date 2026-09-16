mod cache;
mod clipboard;
mod color_scheme;
mod dismiss;
mod drag;
mod element;
mod focus_return;
mod focus_within;
mod id;
mod local_state;
mod localization;
mod popover;
mod portal;
mod presence;
mod ripple;
mod silent_focus;
mod stylesheet;
mod theme;
mod typeahead;

pub(crate) use cache::use_cache;
pub use clipboard::{Clipboard, use_clipboard};
pub use color_scheme::{ColorSchemeHandle, use_color_scheme};
pub(crate) use dismiss::{
    DismissHandle, DismissOptions, escape_closes, use_dismiss, use_dismiss_layer,
    use_escape_dismiss, use_field_list_layer,
};
pub use drag::{Drag, DragMove, DragOptions, DragPoint, DragStart, drag_handle_sx, use_drag};
pub use element::{ElementHandle, use_element};
pub(crate) use element::{use_content_changes, use_form_owner};
pub use focus_return::{FocusReturn, use_focus_return};
pub(crate) use focus_within::{FocusChange, FocusWithin, use_focus_within};
pub(crate) use id::id_selector;
pub use id::{use_id, use_root_id};
pub(crate) use local_state::{LocalState, use_local_state};
pub(crate) use localization::current_localization;
pub use localization::{LocalizationHandle, use_localization, use_localization_handle};
pub(crate) use popover::use_popover_on;
pub use popover::{
    Align, Placed, Placement, PopoverHandle, PopoverOptions, PopoverWidth, Rect, Side, use_popover,
};
pub use portal::use_portal;
pub(crate) use portal::use_portal_slot;
pub use presence::{Presence, use_presence};
pub(crate) use ripple::{clipped_ripple_sx, ripple_sx, use_ripple};
pub(crate) use silent_focus::use_silent_focus_out;
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
