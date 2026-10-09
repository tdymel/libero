use super::tree::{pages, title_at};
use crate::Route;

/// The pages before and after `route` in sidebar order with their page titles, for the docs pager.
pub fn neighbours(route: &Route) -> [Option<(Route, String)>; 2] {
    let pages = pages();
    let path = route.to_string();
    let Some(at) = pages.iter().position(|(id, ..)| *id == path) else {
        return [None, None];
    };
    let pick = |index: Option<usize>| {
        let index = index.filter(|index| *index < pages.len())?;
        Some((
            pages[index].0.parse::<Route>().ok()?,
            title_at(&pages, index),
        ))
    };
    [pick(at.checked_sub(1)), pick(Some(at + 1))]
}

/// The sidebar label of `route`'s page.
pub fn page_label(route: &Route) -> Option<&'static str> {
    let path = route.to_string();
    pages()
        .into_iter()
        .find(|(id, ..)| *id == path)
        .map(|(_, label, _)| label)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(route: Route) -> [Option<String>; 2] {
        neighbours(&route).map(|side| side.map(|(_, title)| title))
    }

    /// The pager prints the page title, so a repeated label is not ambiguous.
    #[test]
    fn the_pager_names_a_repeated_label_by_its_page_title() {
        let some = |title: &str| Some(title.to_string());
        assert_eq!(
            titles(Route::ToolbarPage {}),
            [some("Tldr"), some("Form getting started")]
        );
        assert_eq!(
            titles(Route::AccessibilityPage {}),
            [some("Blockquote"), some("FocusTrap")]
        );
        assert_eq!(
            titles(Route::HooksPage {}),
            [some("Accessibility settings"), some("Element handle")]
        );
    }
}
