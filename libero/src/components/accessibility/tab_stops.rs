use std::collections::HashSet;

use crate::{components::common::FOCUSABLE_SELECTOR, hooks::ElementHandle, platform::ElementApi};

/// Each item's form as a number, 0 outside one: the items inside a form are one
/// run per form in document order, so the runs' sizes tell which is whose.
fn form_owners(root: &ElementHandle, items: &[Box<dyn ElementApi>]) -> Vec<usize> {
    let forms = root.query_selector_all("form").unwrap_or_default();
    if forms.is_empty() {
        return vec![0; items.len()];
    }
    let mut runs = forms.iter().enumerate().map(|(at, form)| {
        let inside = form.query_selector_all(FOCUSABLE_SELECTOR);
        (at + 1, inside.map_or(0, |inside| inside.len()))
    });
    let (mut owner, mut left) = (0, 0);
    items
        .iter()
        .map(|item| {
            if !item.matches("form *").unwrap_or(false) {
                return 0;
            }
            while left == 0 {
                (owner, left) = runs.next().unwrap_or((usize::MAX, usize::MAX));
            }
            left -= 1;
            owner
        })
        .collect()
}

/// Which of `items` are Tab stops: a native radio group (same `name` and form)
/// is one, its `:checked` radio or else its first (its last for `backwards`, as a browser's Shift+Tab).
pub(crate) fn tab_stops(
    root: &ElementHandle,
    items: &[Box<dyn ElementApi>],
    backwards: bool,
) -> Vec<bool> {
    let attr = |item: &dyn ElementApi, name| item.attribute(name).ok().flatten();
    let group = |item: &dyn ElementApi| {
        let radio = attr(item, "type").is_some_and(|ty| ty.eq_ignore_ascii_case("radio"));
        radio
            .then(|| attr(item, "name"))
            .flatten()
            .filter(|name| !name.is_empty())
    };
    let names: Vec<Option<String>> = items.iter().map(|item| group(item.as_ref())).collect();
    if names.iter().all(Option::is_none) {
        return vec![true; items.len()];
    }
    let owners = form_owners(root, items);
    let keys: Vec<Option<(&String, usize)>> = names
        .iter()
        .zip(&owners)
        .map(|(name, owner)| Some((name.as_ref()?, *owner)))
        .collect();
    let checked: Vec<bool> = items
        .iter()
        .zip(&keys)
        .map(|(item, key)| key.is_some() && item.matches(":checked").unwrap_or(false))
        .collect();
    let decided: HashSet<_> = keys
        .iter()
        .zip(&checked)
        .filter_map(|(key, checked)| key.filter(|_| *checked))
        .collect();
    // The walk's direction picks which radio of an undecided group comes first.
    let walk: Vec<usize> = match backwards {
        true => (0..keys.len()).rev().collect(),
        false => (0..keys.len()).collect(),
    };
    let mut seen = HashSet::new();
    let mut stops = vec![true; keys.len()];
    for at in walk {
        let Some(key) = keys[at] else {
            continue;
        };
        let stop = !seen.contains(&key) && (checked[at] || !decided.contains(&key));
        if stop {
            seen.insert(key);
        }
        stops[at] = stop;
    }
    stops
}

/// Item indices in the browser's Tab order: positive `tabindex` ascending, then the rest in DOM order.
pub(crate) fn tab_order(items: &[Box<dyn ElementApi>]) -> Vec<usize> {
    let tabindex = |item: &dyn ElementApi| {
        let value = item.attribute("tabindex").ok().flatten();
        value.and_then(|text| text.trim().parse::<i32>().ok())
    };
    let indices: Vec<Option<i32>> = items.iter().map(|item| tabindex(item.as_ref())).collect();
    order_by_tabindex(&indices)
}

fn order_by_tabindex(tabindexes: &[Option<i32>]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..tabindexes.len()).collect();
    // Stable: equal `tabindex` and the unordered keep their DOM order.
    order.sort_by_key(|at| match tabindexes[*at] {
        Some(index) if index > 0 => (0, index),
        _ => (1, 0),
    });
    order
}

/// `root`'s first Tab stop in Tab order that takes focus (the last, `last`); whether one did.
pub(crate) fn focus_edge(root: &ElementHandle, last: bool) -> bool {
    let Ok(items) = root.query_selector_all(FOCUSABLE_SELECTOR) else {
        return false;
    };
    let stops = tab_stops(root, &items, last);
    let order = tab_order(&items);
    match last {
        true => focus_first_stop(&items, &stops, order.into_iter().rev()),
        false => focus_first_stop(&items, &stops, order.into_iter()),
    }
}

/// Focuses the first stop of `order` that takes it: a `display: none` one ignores `focus()`.
pub(crate) fn focus_first_stop(
    items: &[Box<dyn ElementApi>],
    stops: &[bool],
    order: impl Iterator<Item = usize>,
) -> bool {
    order
        .filter(|at| stops.get(*at).copied().unwrap_or(true))
        .any(|at| items[at].focus().is_ok() && items[at].is_focused())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_tabindex_comes_first_in_ascending_order() {
        let tabindexes = [None, Some(0), Some(3), Some(1), Some(3), None];
        assert_eq!(order_by_tabindex(&tabindexes), [3, 2, 4, 0, 1, 5]);
    }

    #[test]
    fn without_a_positive_tabindex_the_dom_order_stays() {
        assert_eq!(order_by_tabindex(&[None, Some(0), None]), [0, 1, 2]);
    }
}
