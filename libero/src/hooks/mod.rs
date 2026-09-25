mod accessibility;
mod cache;
mod clipboard;
mod color_scheme;
mod debounce;
mod direction;
mod dismiss;
mod drag;
mod element;
mod focus_return;
mod focus_within;
mod formats;
mod history;
mod hotkeys;
mod icons;
mod id;
mod intersection;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod intersection_tests;
mod local_state;
mod localization;
mod long_press;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod long_press_tests;
mod media_query;
mod popover;
mod portal;
#[cfg(test)]
mod portal_tests;
mod presence;
#[cfg(test)]
mod presence_tests;
mod ripple;
mod silent_focus;
mod stylesheet;
mod theme;
mod timers;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod timers_tests;
mod typeahead;

pub use accessibility::{AccessibilityHandle, use_accessibility};
pub(crate) use cache::use_cache;
pub use clipboard::{Clipboard, use_clipboard};
pub use color_scheme::{ColorSchemeHandle, use_color_scheme};
pub use debounce::{
    use_debounced_callback, use_debounced_value, use_throttled_callback, use_throttled_value,
};
pub use direction::{DirectionHandle, use_direction};
pub(crate) use dismiss::{
    DismissHandle, DismissOptions, PressMarker, escape_closes, listener, use_dismiss,
    use_dismiss_layer, use_escape_dismiss, use_field_list_layer, use_press_marker,
};
pub use drag::{Drag, DragMove, DragOptions, DragPoint, DragStart, drag_handle_sx, use_drag};
pub(crate) use drag::{sideways_drag_sx, use_distance_drag, use_drag_with, use_sideways_drag};
pub use element::{ElementHandle, use_element};
pub(crate) use element::{use_content_changes, use_form_owner, use_resize_fallback};
pub use focus_return::{FocusReturn, use_focus_return};
pub(crate) use focus_within::{FocusChange, FocusWithin, use_focus_within};
pub(crate) use formats::current_formats;
pub use formats::{FormatsHandle, use_formats, use_formats_handle};
pub use history::{HistoryHandle, UndoHistory, use_history};
pub use hotkeys::{Hotkey, use_hotkeys};
pub use icons::use_icon;
pub use id::use_id;
pub(crate) use id::{id_selector, use_root_id};
pub use intersection::{
    InViewport, Intersection, IntersectionEntry, IntersectionOptions, use_in_viewport,
    use_intersection,
};
pub(crate) use local_state::{LocalState, use_local_state};
pub(crate) use localization::current_localization;
pub use localization::{LocalizationHandle, use_localization, use_localization_handle};
pub use long_press::{LongPress, LongPressOptions, use_long_press};
pub use media_query::{use_is_mobile, use_media_query};
pub use popover::{
    Align, Placed, Placement, PopoverHandle, PopoverOptions, PopoverWidth, Rect, Side, use_popover,
};
pub(crate) use popover::{place, use_popover_on};
pub(crate) use portal::{use_portal, use_portal_slot};
pub(crate) use presence::use_presence;
pub(crate) use ripple::{clipped_ripple_sx, ripple_sx, use_ripple};
pub(crate) use silent_focus::{use_silent_focus_in, use_silent_focus_out};
pub use stylesheet::use_stylesheet;
pub(crate) use stylesheet::{SxSource, use_box_css, use_css};
pub use theme::{ThemeSetHandle, use_theme, use_theme_set};
pub(crate) use theme::{use_glass_gradient_style, use_glass_tint, use_gradient_style};
pub use timers::{IntervalHandle, TimeoutHandle, use_interval, use_timeout};
pub(crate) use timers::{Scheduled, use_scheduled};
pub(crate) use typeahead::{TYPEAHEAD_RESET, Typeahead, typeahead_match, use_typeahead};

// The overlay hooks render a component, so they live beside it (todo 178).
// The layer rule (`tests/all/architecture.rs`) exempts only this `use`.
// archunit: ignore components::overlay
pub use crate::components::overlay::{
    lightbox::use_lightbox::{LightboxItem, LightboxOpening, LightboxOptions, use_lightbox},
    use_drawer::{DrawerOptions, use_drawer},
    use_floating_window::{FloatingWindowHandle, use_floating_window},
    use_modal::{ModalHandle, ModalScope, Opening, OpeningFuture, use_modal, use_modal_close},
};
// The sortable hooks share `Sortable`'s context and orientation, so they live beside it.
// archunit: ignore components::data_display
pub use crate::components::data_display::sortable::{
    SortableHandle, SortableItemHandle, SortableMove, SortableOptions, use_sortable,
    use_sortable_item,
};
