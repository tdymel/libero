#[cfg(feature = "use-accessibility")]
mod accessibility;
mod cache;
#[cfg(feature = "use-clipboard")]
mod clipboard;
#[cfg(feature = "use-color-scheme")]
mod color_scheme;
#[cfg(feature = "use-countdown")]
mod countdown;
#[cfg(feature = "use-debounce")]
mod debounce;
#[cfg(feature = "use-direction")]
mod direction;
// `use_popover`'s other half; `use-back` only exports `use_back`.
#[cfg(feature = "use-popover")]
mod dismiss;
#[cfg(feature = "use-drag")]
mod drag;
#[cfg(feature = "use-element")]
mod element;
#[cfg(all(feature = "use-element", feature = "use-popover"))]
mod element_rect;
#[cfg(feature = "fcm")]
mod fcm;
#[cfg(feature = "use-focus-return")]
mod focus_return;
mod focus_within;
mod formats;
#[cfg(feature = "use-fullscreen")]
mod fullscreen;
#[cfg(feature = "use-geolocation")]
mod geolocation;
#[cfg(feature = "use-history")]
mod history;
#[cfg(feature = "use-hotkeys")]
mod hotkeys;
#[cfg(feature = "use-icon")]
mod icons;
#[cfg(feature = "use-id")]
mod id;
#[cfg(feature = "use-indexed-db")]
mod indexed_db;
#[cfg(feature = "use-intersection")]
mod intersection;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod intersection_tests;
mod local_state;
mod localization;
#[cfg(feature = "use-long-press")]
mod long_press;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod long_press_tests;
#[cfg(feature = "use-media")]
mod media;
#[cfg(feature = "use-media-query")]
mod media_query;
mod permission;
#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) mod polling_tests;
#[cfg(feature = "use-popover")]
mod popover;
mod portal;
#[cfg(test)]
mod portal_tests;
#[cfg(feature = "use-timers")]
mod presence;
#[cfg(test)]
mod presence_tests;
#[cfg(feature = "use-push-subscription")]
mod push_subscription;
mod ripple;
#[cfg(feature = "use-element")]
mod scroll_lock;
#[cfg(feature = "use-share")]
mod share;
#[cfg(feature = "use-element")]
mod silent_focus;
#[cfg(feature = "use-local-storage")]
mod storage;
mod stylesheet;
mod subscription_slot;
#[cfg(feature = "use-swipe")]
mod swipe;
#[cfg(all(feature = "use-swipe", feature = "use-timers"))]
mod swipe_close;
#[cfg(feature = "use-system-notification")]
mod system_notification;
mod theme;
#[cfg(feature = "use-timers")]
mod timers;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod timers_tests;
mod typeahead;
#[cfg(feature = "use-user-media")]
mod user_media;

#[cfg(feature = "fcm")]
pub use crate::platform::{FcmError, FcmMessage, FcmOptions};
pub use crate::platform::{
    GeolocationError, GeolocationOptions, MediaDevice, NotificationAction, PermissionState,
    Position, PushEndpoint, PushError, PushOptions, StorageError, SystemNotification,
    SystemNotificationError, UserMediaError,
};
#[cfg(feature = "use-accessibility")]
pub use accessibility::{AccessibilityHandle, use_accessibility};
pub(crate) use cache::use_cache;
#[cfg(feature = "use-clipboard")]
pub use clipboard::{Clipboard, use_clipboard};
#[cfg(feature = "use-color-scheme")]
pub use color_scheme::{ColorSchemeHandle, use_color_scheme};
#[cfg(feature = "use-countdown")]
pub use countdown::{format_duration, use_countdown};
#[cfg(feature = "use-debounce")]
pub use debounce::{
    use_debounced_callback, use_debounced_value, use_throttled_callback, use_throttled_value,
};
#[cfg(feature = "use-direction")]
pub use direction::{DirectionHandle, use_direction};
#[cfg(feature = "use-back")]
pub use dismiss::use_back;
#[cfg(feature = "use-popover")]
pub(crate) use dismiss::{
    DismissHandle, DismissOptions, PressMarker, claim_back_entry, escape_closes, listener,
    use_dismiss, use_dismiss_layer, use_escape_dismiss, use_field_list_layer, use_press_away,
    use_press_marker,
};
#[cfg(feature = "use-drag")]
pub use drag::{Drag, DragMove, DragOptions, DragPoint, DragStart, drag_handle_sx, use_drag};
#[cfg(feature = "use-drag")]
pub(crate) use drag::{
    edge_scroll_step, sideways_drag_sx, use_distance_drag, use_distance_drag_cancel,
    use_drag_cancel, use_drag_with, use_early_measure, use_sideways_drag, use_sideways_drag_if,
};
#[cfg(feature = "use-element")]
pub use element::{ElementHandle, use_element};
#[cfg(feature = "use-element")]
pub(crate) use element::{use_content_changes, use_form_owner, use_resize_fallback};
#[cfg(all(feature = "use-element", feature = "use-popover"))]
pub(crate) use element_rect::{ElementRect, use_element_rect};
#[cfg(feature = "fcm")]
pub use fcm::{Fcm, use_fcm};
#[cfg(feature = "use-focus-return")]
pub use focus_return::{FocusReturn, use_focus_return};
pub(crate) use focus_within::{FocusChange, FocusWithin, use_focus_within};
pub(crate) use formats::current_formats;
pub use formats::{FormatsHandle, use_formats, use_formats_handle};
#[cfg(feature = "use-fullscreen")]
pub(crate) use fullscreen::FULLSCREEN_ATTR;
#[cfg(feature = "use-fullscreen")]
pub use fullscreen::{FullscreenHandle, use_fullscreen};
#[cfg(feature = "use-geolocation")]
pub use geolocation::{Geolocation, use_geolocation};
#[cfg(feature = "use-history")]
pub use history::{HistoryHandle, UndoHistory, use_history};
#[cfg(feature = "use-hotkeys")]
pub use hotkeys::{Hotkey, use_hotkeys};
#[cfg(feature = "use-hotkeys")]
pub(crate) use hotkeys::{aria_keyshortcuts, chord_keys};
#[cfg(feature = "use-icon")]
pub use icons::use_icon;
#[cfg(feature = "use-id")]
pub use id::use_id;
#[cfg(feature = "use-id")]
pub(crate) use id::{escape_css_string, id_selector, use_root_id};
#[cfg(feature = "use-indexed-db")]
pub use indexed_db::{StoredAsync, use_indexed_db};
#[cfg(feature = "use-intersection")]
pub use intersection::{
    InViewport, Intersection, IntersectionEntry, IntersectionOptions, use_in_viewport,
    use_intersection,
};
pub(crate) use local_state::{LocalState, use_local_state};
pub(crate) use localization::current_localization;
pub use localization::{LocalizationHandle, use_localization, use_localization_handle};
#[cfg(feature = "use-long-press")]
pub use long_press::{LongPress, LongPressOptions, use_long_press};
#[cfg(feature = "use-media")]
pub use media::{MediaError, MediaHandle, use_media};
#[cfg(feature = "use-media-query")]
pub use media_query::{use_is_mobile, use_media_query};
#[cfg(feature = "use-popover")]
pub(crate) use popover::{
    AVAILABLE_HEIGHT as POPOVER_AVAILABLE_HEIGHT, ArrowPoint, Rect, owner_link, place,
    placement_sx, use_marked_popover, use_popover_on,
};
#[cfg(feature = "use-popover")]
pub use popover::{
    Align, Placement, PopoverHandle, PopoverOptions, PopoverWidth, Side, use_popover,
};
pub(crate) use portal::{use_portal, use_portal_slot};
#[cfg(feature = "use-timers")]
pub(crate) use presence::use_presence;
#[cfg(feature = "use-push-subscription")]
pub use push_subscription::{PushSubscription, use_push_subscription};
pub(crate) use ripple::{child_ripple_sx, ripple_sx, use_ripple};
#[cfg(feature = "use-element")]
pub(crate) use scroll_lock::use_scroll_lock;
#[cfg(feature = "use-share")]
pub use share::{Share, use_share};
#[cfg(feature = "use-element")]
pub(crate) use silent_focus::{use_silent_focus_in, use_silent_focus_out, use_silent_focus_out_of};
#[cfg(feature = "use-local-storage")]
pub(crate) use storage::{LocalText, SessionText, local_text, session_text};
#[cfg(feature = "use-local-storage")]
pub use storage::{
    StorageFormat, StorageOptions, Stored, use_local_storage, use_local_storage_with,
    use_session_storage, use_session_storage_with,
};
pub use stylesheet::use_stylesheet;
pub(crate) use stylesheet::{SxSource, use_box_css, use_css};
pub(crate) use subscription_slot::{SubscriptionSlot, use_subscription_slot};
#[cfg(feature = "use-swipe")]
pub use swipe::{
    EdgeSwipeOptions, Swipe, SwipeDirection, SwipeEdge, SwipeEvent, SwipeOptions, edge_swipe_sx,
    use_edge_swipe, use_swipe,
};
#[cfg(all(feature = "use-swipe", feature = "use-timers"))]
pub(crate) use swipe_close::{SWIPE_BEYOND, SwipeClose, swipe_close_sx, use_swipe_close};
#[cfg(feature = "use-system-notification")]
pub use system_notification::{SystemNotifier, use_system_notification};
pub use theme::{ThemeSetHandle, use_theme, use_theme_set};
pub(crate) use theme::{use_glass_gradient_style, use_glass_tint, use_gradient_style};
#[cfg(feature = "use-timers")]
pub use timers::{IntervalHandle, TimeoutHandle, use_interval, use_timeout};
#[cfg(feature = "use-timers")]
pub(crate) use timers::{Scheduled, use_scheduled};
pub(crate) use typeahead::{TYPEAHEAD_RESET, Typeahead, typeahead_match, use_typeahead};
#[cfg(feature = "use-user-media")]
pub use user_media::{
    CameraFacing, UserMedia, UserMediaDevices, UserMediaOptions, use_user_media,
    use_user_media_devices,
};

// The overlay hooks render a component, so they live beside it (todo 178).
// The layer rule (`tests/all/architecture.rs`) exempts only this `use`.
// archunit: ignore components::overlay
#[cfg(feature = "modal")]
pub use crate::components::overlay::use_modal::{
    ModalHandle, ModalScope, Opening, OpeningFuture, use_modal, use_modal_close,
};
// archunit: ignore components::overlay
#[cfg(feature = "lightbox")]
pub use crate::components::overlay::lightbox::use_lightbox::{
    LightboxItem, LightboxOpening, LightboxOptions, use_lightbox,
};
// archunit: ignore components::overlay
#[cfg(feature = "drawer")]
pub use crate::components::overlay::use_drawer::{DrawerOptions, use_drawer};
// archunit: ignore components::overlay
#[cfg(feature = "floating-window")]
pub use crate::components::overlay::use_floating_window::{
    FloatingWindowHandle, use_floating_window,
};
// The sortable hooks share `Sortable`'s context and orientation, so they live beside it.
// archunit: ignore components::data_display
#[cfg(feature = "sortable")]
pub use crate::components::data_display::sortable::{
    SortableHandle, SortableItemHandle, SortableMove, SortableOptions, use_labelled_sortable_item,
    use_sortable, use_sortable_item,
};
