use dioxus::prelude::*;
use libero::use_theme_set;

mod overview;
mod use_debounce;
mod use_drag;
mod use_element;
mod use_geolocation;
mod use_history;
mod use_hotkeys;
mod use_intersection;
mod use_long_press;
mod use_media_query;
mod use_stylesheet;
mod use_theme_set;
mod use_timers;

pub use overview::HooksPage;
pub use use_debounce::UseDebouncePage;
pub use use_drag::UseDragPage;
pub use use_element::UseElementPage;
pub use use_geolocation::UseGeolocationPage;
pub use use_history::UseHistoryPage;
pub use use_hotkeys::UseHotkeysPage;
pub use use_intersection::UseIntersectionPage;
pub use use_long_press::UseLongPressPage;
pub use use_media_query::UseMediaQueryPage;
pub use use_stylesheet::UseStylesheetPage;
pub use use_theme_set::UseThemeSetPage;
pub use use_timers::UseTimersPage;

/// Around a demo that switches the site's theme set: puts it back when the
/// page is left.
#[component]
fn KeepSite(children: Element) -> Element {
    let themes = use_theme_set();
    let site = use_hook(|| themes.get());
    use_drop(move || {
        if themes.get() != site {
            themes.set(site);
        }
    });
    children
}
