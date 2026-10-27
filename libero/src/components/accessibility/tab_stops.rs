use std::collections::{HashMap, HashSet};

use crate::{components::common::FOCUSABLE_SELECTOR, platform::ElementApi};

/// An item's form owner.
#[derive(Clone, PartialEq, Eq, Hash)]
enum Owner {
    /// The root's forms in document order from 1; 0 for none.
    Form(usize),
    /// A `form` attribute naming no form inside the root.
    Named(String),
}

/// Each item's form: its `form` attribute's, else the one around it (4600). The items
/// inside a form are one run per form in document order, so the runs' sizes tell which is whose.
fn form_owners(root: &dyn ElementApi, items: &[Box<dyn ElementApi>]) -> Vec<Owner> {
    let forms = root.query_selector_all("form").unwrap_or_default();
    let ids: Vec<Option<String>> = forms
        .iter()
        .map(|form| form.attribute("id").ok().flatten())
        .collect();
    let mut runs = forms.iter().enumerate().map(|(at, form)| {
        let inside = form.query_selector_all(FOCUSABLE_SELECTOR);
        (at + 1, inside.map_or(0, |inside| inside.len()))
    });
    let (mut owner, mut left) = (0, 0);
    items
        .iter()
        .map(|item| {
            let mut around = 0;
            if !forms.is_empty() && item.matches("form *").unwrap_or(false) {
                while left == 0 {
                    (owner, left) = runs.next().unwrap_or((usize::MAX, usize::MAX));
                }
                left -= 1;
                around = owner;
            }
            match item.attribute("form").ok().flatten() {
                Some(id) => match ids.iter().position(|form| form.as_ref() == Some(&id)) {
                    Some(at) => Owner::Form(at + 1),
                    None => Owner::Named(id),
                },
                None => Owner::Form(around),
            }
        })
        .collect()
}

/// Each item's native radio group (same `name` and form) as a number; `None` for any other item.
fn radio_groups(root: &dyn ElementApi, items: &[Box<dyn ElementApi>]) -> Vec<Option<usize>> {
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
        return vec![None; items.len()];
    }
    let mut numbers = HashMap::new();
    names
        .into_iter()
        .zip(form_owners(root, items))
        .map(|(name, owner)| {
            let next = numbers.len();
            Some(*numbers.entry((name?, owner)).or_insert(next))
        })
        .collect()
}

/// Which of `items` are Tab stops: a native radio group (same `name` and form)
/// is one, its `:checked` radio or else its first (its last for `backwards`, as a browser's Shift+Tab).
#[cfg(feature = "focus-trap")]
pub(crate) fn tab_stops(
    root: &dyn ElementApi,
    items: &[Box<dyn ElementApi>],
    backwards: bool,
) -> Vec<bool> {
    stops_of(items, &radio_groups(root, items), backwards)
}

fn stops_of(items: &[Box<dyn ElementApi>], keys: &[Option<usize>], backwards: bool) -> Vec<bool> {
    if keys.iter().all(Option::is_none) {
        return vec![true; items.len()];
    }
    let checked: Vec<bool> = items
        .iter()
        .zip(keys)
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
#[cfg(any(feature = "focus-trap", feature = "hover-card"))]
pub(crate) fn focus_edge(root: &dyn ElementApi, last: bool) -> bool {
    let Ok(items) = root.query_selector_all(FOCUSABLE_SELECTOR) else {
        return false;
    };
    let order = tab_order(&items);
    match last {
        true => focus_first_stop(root, &items, order.into_iter().rev(), true),
        false => focus_first_stop(root, &items, order.into_iter(), false),
    }
}

/// Focuses the first Tab stop of `order`, walked `backwards` or not, that takes it: a
/// `display: none` one ignores `focus()`. A radio group whose stop refuses tries its other
/// radios in `order`, those after the stop first (4601).
pub(crate) fn focus_first_stop(
    root: &dyn ElementApi,
    items: &[Box<dyn ElementApi>],
    order: impl Iterator<Item = usize>,
    backwards: bool,
) -> bool {
    let groups = radio_groups(root, items);
    let stops = stops_of(items, &groups, backwards);
    let order: Vec<usize> = order.collect();
    let takes = |at: usize| items[at].focus().is_ok() && items[at].is_focused();
    order.iter().enumerate().any(|(step, &at)| {
        if !stops.get(at).copied().unwrap_or(true) {
            return false;
        }
        takes(at)
            || groups[at].is_some_and(|group| {
                let others = order[step + 1..].iter().chain(&order[..step]);
                others
                    .filter(|&&other| groups[other] == Some(group))
                    .any(|&other| takes(other))
            })
    })
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::*;
    use crate::platform::{Dimensions, PlatformError, Read};

    #[test]
    fn positive_tabindex_comes_first_in_ascending_order() {
        let tabindexes = [None, Some(0), Some(3), Some(1), Some(3), None];
        assert_eq!(order_by_tabindex(&tabindexes), [3, 2, 4, 0, 1, 5]);
    }

    #[test]
    fn without_a_positive_tabindex_the_dom_order_stays() {
        assert_eq!(order_by_tabindex(&[None, Some(0), None]), [0, 1, 2]);
    }

    /// A radio, a form or a root: answers attributes, `form *`, `:checked` and focus.
    #[derive(Clone, Default)]
    struct Fake {
        attrs: Vec<(&'static str, &'static str)>,
        in_form: bool,
        checked: bool,
        refuses: bool,
        children: Vec<Fake>,
        id: usize,
        focused: Rc<Cell<usize>>,
    }

    fn unsupported<T: 'static>() -> Read<T> {
        Box::pin(std::future::ready(Err(PlatformError::Unsupported)))
    }

    impl ElementApi for Fake {
        fn focus(&self) -> Result<(), PlatformError> {
            if !self.refuses {
                self.focused.set(self.id);
            }
            Ok(())
        }
        fn blur(&self) -> Result<(), PlatformError> {
            Ok(())
        }
        fn click(&self) -> Result<(), PlatformError> {
            Ok(())
        }
        fn reset(&self) -> Result<(), PlatformError> {
            Ok(())
        }
        fn request_submit(&self) -> Result<(), PlatformError> {
            Ok(())
        }
        fn is_focused(&self) -> bool {
            self.focused.get() == self.id
        }
        fn is_connected(&self) -> bool {
            true
        }
        fn dimensions(&self) -> Read<Dimensions> {
            unsupported()
        }
        fn client_offset(&self) -> Read<(f64, f64)> {
            unsupported()
        }
        fn scroll_size(&self) -> Read<Dimensions> {
            unsupported()
        }
        fn scroll_offset(&self) -> Read<(f64, f64)> {
            unsupported()
        }
        fn natural_size(&self) -> Read<Dimensions> {
            unsupported()
        }
        fn scroll_to(&self, _: f64, _: f64) -> Result<(), PlatformError> {
            Ok(())
        }
        fn scroll_into_view(&self, _: bool) -> Result<(), PlatformError> {
            Ok(())
        }
        fn set_files(&self, _: &[dioxus::html::FileData]) -> Result<(), PlatformError> {
            Ok(())
        }
        fn set_pointer_capture(&self, _: i32) -> Result<(), PlatformError> {
            Ok(())
        }
        fn attribute(&self, name: &str) -> Result<Option<String>, PlatformError> {
            let found = self.attrs.iter().find(|(key, _)| *key == name);
            Ok(found.map(|(_, value)| value.to_string()))
        }
        fn matches(&self, selector: &str) -> Result<bool, PlatformError> {
            Ok(match selector {
                "form *" => self.in_form,
                ":checked" => self.checked,
                _ => false,
            })
        }
        fn query_selector(&self, _: &str) -> Result<Box<dyn ElementApi>, PlatformError> {
            Err(PlatformError::NotFound)
        }
        fn query_selector_all(&self, _: &str) -> Result<Vec<Box<dyn ElementApi>>, PlatformError> {
            let children = self.children.iter().cloned();
            Ok(children
                .map(|child| Box::new(child) as Box<dyn ElementApi>)
                .collect())
        }
    }

    fn radio(id: usize, focused: &Rc<Cell<usize>>, name: &'static str) -> Fake {
        Fake {
            attrs: vec![("type", "radio"), ("name", name)],
            id,
            focused: focused.clone(),
            ..Fake::default()
        }
    }

    fn boxed(items: &[Fake]) -> Vec<Box<dyn ElementApi>> {
        let items = items.iter().cloned();
        items
            .map(|item| Box::new(item) as Box<dyn ElementApi>)
            .collect()
    }

    /// One radio inside form `#a`, one outside naming it, one outside naming `#b`.
    #[test]
    fn a_radio_follows_its_form_attribute() {
        let focused = Rc::new(Cell::new(0));
        let inside = Fake {
            in_form: true,
            ..radio(1, &focused, "size")
        };
        let mut named = radio(2, &focused, "size");
        named.attrs.push(("form", "a"));
        let mut elsewhere = radio(3, &focused, "size");
        elsewhere.attrs.push(("form", "b"));
        let form = Fake {
            attrs: vec![("id", "a")],
            children: vec![inside.clone()],
            ..Fake::default()
        };
        let root = Fake {
            children: vec![form],
            ..Fake::default()
        };
        let items = boxed(&[inside, named, elsewhere]);
        let groups = radio_groups(&root, &items);
        assert_eq!(groups[0], groups[1]);
        assert_ne!(groups[0], groups[2]);
    }

    /// The checked radio is hidden: the group's next radio takes focus, not the button after.
    #[test]
    fn a_group_whose_checked_radio_refuses_tries_its_others() {
        let focused = Rc::new(Cell::new(0));
        let hidden = Fake {
            checked: true,
            refuses: true,
            ..radio(2, &focused, "size")
        };
        let after = Fake {
            id: 4,
            focused: focused.clone(),
            ..Fake::default()
        };
        let items = boxed(&[
            radio(1, &focused, "size"),
            hidden,
            radio(3, &focused, "size"),
            after,
        ]);
        let root = Fake::default();
        assert!(focus_first_stop(&root, &items, 0..4, false));
        assert_eq!(focused.get(), 3);
        // Walked from past the stop, the group is behind and the button is next.
        focused.set(0);
        assert!(focus_first_stop(&root, &items, 2..4, false));
        assert_eq!(focused.get(), 4);
    }
}
