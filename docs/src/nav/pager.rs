use super::tree::pages;
use crate::Route;

/// The pages before and after `route` in sidebar order, for the docs pager.
pub fn neighbours(route: &Route) -> [Option<(Route, &'static str)>; 2] {
    let pages = pages();
    let path = route.to_string();
    let Some(at) = pages.iter().position(|(id, ..)| *id == path) else {
        return [None, None];
    };
    let pick = |index: Option<usize>| {
        let (id, label, _) = pages.get(index?)?;
        Some((id.parse::<Route>().ok()?, *label))
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
