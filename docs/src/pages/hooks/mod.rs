use dioxus::prelude::*;
use libero::{use_formats_handle, use_localization_handle, use_theme_set};

mod overview;
mod use_clipboard;
mod use_color_scheme;
mod use_combobox;
mod use_drag;
mod use_drawer;
mod use_element;
mod use_floating_window;
mod use_focus_return;
mod use_form;
mod use_form_context;
mod use_formats;
mod use_formats_handle;
mod use_id;
mod use_lightbox;
mod use_localization;
mod use_localization_handle;
mod use_menu;
mod use_modal;
mod use_modal_close;
mod use_notifications;
mod use_notifications_with;
mod use_popover;
mod use_scroll_area;
mod use_scroller;
mod use_spotlight;
mod use_stylesheet;
mod use_theme;
mod use_theme_set;

pub use overview::HooksPage;
pub use use_clipboard::UseClipboardPage;
pub use use_color_scheme::UseColorSchemePage;
pub use use_combobox::UseComboboxPage;
pub use use_drag::UseDragPage;
pub use use_drawer::UseDrawerPage;
pub use use_element::UseElementPage;
pub use use_floating_window::UseFloatingWindowPage;
pub use use_focus_return::UseFocusReturnPage;
pub use use_form::UseFormPage;
pub use use_form_context::UseFormContextPage;
pub use use_formats::UseFormatsPage;
pub use use_formats_handle::UseFormatsHandlePage;
pub use use_id::UseIdPage;
pub use use_lightbox::UseLightboxPage;
pub use use_localization::UseLocalizationPage;
pub use use_localization_handle::UseLocalizationHandlePage;
pub use use_menu::UseMenuPage;
pub use use_modal::UseModalPage;
pub use use_modal_close::UseModalClosePage;
pub use use_notifications::UseNotificationsPage;
pub use use_notifications_with::UseNotificationsWithPage;
pub use use_popover::UsePopoverPage;
pub use use_scroll_area::UseScrollAreaPage;
pub use use_scroller::UseScrollerPage;
pub use use_spotlight::UseSpotlightPage;
pub use use_stylesheet::UseStylesheetPage;
pub use use_theme::UseThemePage;
pub use use_theme_set::UseThemeSetPage;

/// Around a demo that switches the site's theme set, language or formats:
/// puts back whichever changed when the page is left.
#[component]
fn KeepSite(children: Element) -> Element {
    let themes = use_theme_set();
    let localization = use_localization_handle();
    let formats = use_formats_handle();
    let site = use_hook(|| (themes.get(), localization.get(), formats.get()));
    use_drop(move || {
        if themes.get() != site.0 {
            themes.set(site.0);
        }
        localization.set(site.1);
        formats.set(site.2);
    });
    children
}
