use dioxus::prelude::*;
use libero::use_theme_set;

mod overview;
mod use_clipboard;
mod use_drag;
mod use_element;
mod use_focus_return;
mod use_id;
mod use_stylesheet;
mod use_theme_set;

pub use overview::HooksPage;
pub use use_clipboard::UseClipboardPage;
pub use use_drag::UseDragPage;
pub use use_element::UseElementPage;
pub use use_focus_return::UseFocusReturnPage;
pub use use_id::UseIdPage;
pub use use_stylesheet::UseStylesheetPage;
pub use use_theme_set::UseThemeSetPage;

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
