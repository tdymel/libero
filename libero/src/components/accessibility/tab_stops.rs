use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
};

use crate::{components::common::FOCUSABLE_SELECTOR, platform::ElementApi};

/// An item's form owner.
#[derive(Clone, PartialEq, Eq, Hash)]
enum Owner {
    /// The root's forms in document order from 1; 0 for none.
    Form(usize),
    /// A `form` attribute naming no form inside the root: a form outside it, or none (the
    /// platform cannot tell which, so a missing id still groups apart from the formless).
    Named(String),
}

/// How many `form` ancestors `element` has, counted by growing `form form ... <tag>` selectors.
fn form_depth(element: &dyn ElementApi, tag: &str) -> usize {
    let mut selector = format!("form {tag}");
    let mut depth = 0;
    while element.matches(&selector).unwrap_or(false) {
        depth += 1;
        selector.insert_str(0, "form ");
    }
    depth
}

/// Each item's form: its `form` attribute's, else its nearest ancestor form in `root` (4600). The
/// items under a form are one run in document order, so a stack of open runs tells which is whose.
fn form_owners(root: &dyn ElementApi, items: &[Box<dyn ElementApi>]) -> Vec<Owner> {
    let forms = root.query_selector_all("form").unwrap_or_default();
    let ids: Vec<Option<String>> = forms
        .iter()
        .map(|form| form.attribute("id").ok().flatten())
        .collect();
    // Each form with items: its number, its depth counting itself and its run's length.
    let mut ahead = forms
        .iter()
        .enumerate()
        .filter_map(|(at, form)| {
            let inside = form.query_selector_all(FOCUSABLE_SELECTOR).ok()?.len();
            (inside > 0).then(|| (at + 1, form_depth(form.as_ref(), "form") + 1, inside))
        })
        .peekable();
    let mut open: Vec<(usize, usize, usize)> = Vec::new();
    items
        .iter()
        .map(|item| {
            let depth = if forms.is_empty() {
                0
            } else {
                form_depth(item.as_ref(), "*")
            };
            while open.last().is_some_and(|(_, _, left)| *left == 0) {
                open.pop();
            }
            // Only a form inside the open one opens here: a later sibling form waits for its run.
            while let Some(&(form, level, inside)) = ahead.peek()
                && depth >= level
                && open.last().is_none_or(|(_, top, _)| level > *top)
            {
                open.push((form, level, inside));
                ahead.next();
            }
            let around = open.last().map_or(0, |(form, _, _)| *form);
            for (_, _, left) in &mut open {
                *left = left.saturating_sub(1);
            }
            match item.attribute("form").ok().flatten() {
                // A `form` naming nothing leaves the item without a form, whatever surrounds it.
                Some(id) if id.is_empty() => Owner::Form(0),
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
    // No two radios sharing a name: every radio is a stop, so no owner needs reading.
    let mut seen = HashSet::new();
    if names.iter().flatten().all(|name| seen.insert(name)) {
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

/// Which of `items` are Tab stops: a native radio group (same `name` and form) is one, its
/// `:checked` radio or else its first. `backwards` picks the last, as Firefox's Shift+Tab does
/// (Chromium lands on the first).
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
    walk_stops(root, items, order, backwards).0
}

/// [`focus_first_stop`], but a renderer that applies focus later cannot confirm it: with no
/// stop confirmed, the first that took `focus()`. Only for a trap, which has nowhere to cross to (4793).
#[cfg(feature = "focus-trap")]
pub(crate) fn focus_stop_unconfirmed(
    root: &dyn ElementApi,
    items: &[Box<dyn ElementApi>],
    order: impl Iterator<Item = usize>,
    backwards: bool,
) -> bool {
    let (taken, first_ok) = walk_stops(root, items, order, backwards);
    taken || first_ok.is_some_and(|at| items[at].focus().is_ok())
}

/// Whether a stop took focus, and the first one whose `focus()` did not fail.
fn walk_stops(
    root: &dyn ElementApi,
    items: &[Box<dyn ElementApi>],
    order: impl Iterator<Item = usize>,
    backwards: bool,
) -> (bool, Option<usize>) {
    let groups = radio_groups(root, items);
    let stops = stops_of(items, &groups, backwards);
    let order: Vec<usize> = order.collect();
    let first_ok = Cell::new(None);
    let takes = |at: usize| {
        if items[at].focus().is_err() {
            return false;
        }
        first_ok.set(first_ok.get().or(Some(at)));
        items[at].is_focused()
    };
    let taken = order.iter().enumerate().any(|(step, &at)| {
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
    });
    (taken, first_ok.get())
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

    /// A radio, a form or a root: answers attributes, `form ... *`, `:checked` and focus.
    #[derive(Clone, Default)]
    struct Fake {
        attrs: Vec<(&'static str, &'static str)>,
        /// How many forms enclose it.
        depth: usize,
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
            let mut words: Vec<&str> = selector.split(' ').collect();
            Ok(match words.pop() {
                Some("*" | "form") if words.iter().all(|word| *word == "form") => {
                    self.depth >= words.len()
                }
                _ => selector == ":checked" && self.checked,
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
            depth: 1,
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

    fn in_depth(radio: Fake, depth: usize) -> Fake {
        Fake { depth, ..radio }
    }

    fn form_of(children: &[&Fake]) -> Fake {
        Fake {
            children: children.iter().map(|child| (*child).clone()).collect(),
            ..Fake::default()
        }
    }

    /// Two sibling forms each hold a radio of one name: two groups.
    #[test]
    fn sibling_forms_keep_their_radios_apart() {
        let focused = Rc::new(Cell::new(0));
        let first = in_depth(radio(1, &focused, "size"), 1);
        let second = in_depth(radio(2, &focused, "size"), 1);
        let third = in_depth(radio(3, &focused, "size"), 1);
        let root = Fake {
            children: vec![form_of(&[&first]), form_of(&[&second, &third])],
            ..Fake::default()
        };
        let groups = radio_groups(&root, &boxed(&[first, second, third]));
        assert_ne!(groups[0], groups[1]);
        assert_eq!(groups[1], groups[2]);
    }

    /// An outer form holds Z1 and an inner form holding Z2 and Z3: two groups, not one.
    #[test]
    fn a_nested_form_owns_its_own_radios() {
        let focused = Rc::new(Cell::new(0));
        let z1 = in_depth(radio(1, &focused, "size"), 1);
        let z2 = in_depth(radio(2, &focused, "size"), 2);
        let z3 = in_depth(radio(3, &focused, "size"), 2);
        let inner = Fake {
            depth: 1,
            ..form_of(&[&z2, &z3])
        };
        let outer = form_of(&[&z1, &z2, &z3]);
        let z4 = in_depth(radio(4, &focused, "size"), 1);
        let root = Fake {
            children: vec![outer, inner, form_of(&[&z4])],
            ..Fake::default()
        };
        let groups = radio_groups(&root, &boxed(&[z1, z2, z3, z4]));
        assert_ne!(groups[0], groups[1]);
        assert_eq!(groups[1], groups[2]);
        assert_ne!(groups[0], groups[3]);
        assert_ne!(groups[1], groups[3]);
    }

    /// The root sits in a form and holds one: the radio around the inner form is not its radio.
    #[test]
    fn a_root_inside_a_form_keeps_the_forms_apart() {
        let focused = Rc::new(Cell::new(0));
        let around = in_depth(radio(1, &focused, "size"), 1);
        let first = in_depth(radio(2, &focused, "size"), 2);
        let second = in_depth(radio(3, &focused, "size"), 2);
        let inner = Fake {
            depth: 1,
            ..form_of(&[&first, &second])
        };
        let root = Fake {
            children: vec![inner],
            ..Fake::default()
        };
        let groups = radio_groups(&root, &boxed(&[around, first, second]));
        assert_ne!(groups[0], groups[1]);
        assert_eq!(groups[1], groups[2]);
    }

    /// `form=""` leaves a radio without a form even inside one: it joins the formless radios.
    #[test]
    fn an_empty_form_attribute_groups_with_the_formless() {
        let focused = Rc::new(Cell::new(0));
        let mut inside = in_depth(radio(1, &focused, "size"), 1);
        inside.attrs.push(("form", ""));
        let loose = radio(2, &focused, "size");
        let root = Fake {
            children: vec![Fake {
                children: vec![inside.clone()],
                ..Fake::default()
            }],
            ..Fake::default()
        };
        let groups = radio_groups(&root, &boxed(&[inside, loose]));
        assert_eq!(groups[0], groups[1]);
    }

    /// Form A holds a `form=""` radio, then a formless radio and form B follow: the two group (4759).
    #[test]
    fn a_later_sibling_form_does_not_own_the_radios_before_it() {
        let focused = Rc::new(Cell::new(0));
        let mut emptied = in_depth(radio(1, &focused, "tone"), 1);
        emptied.attrs.push(("form", ""));
        let loose = radio(2, &focused, "tone");
        let other = in_depth(radio(3, &focused, "plan"), 1);
        let root = Fake {
            children: vec![form_of(&[&emptied]), form_of(&[&other])],
            ..Fake::default()
        };
        let groups = radio_groups(&root, &boxed(&[emptied, loose, other]));
        assert_eq!(groups[0], groups[1]);
    }

    #[test]
    fn radios_with_distinct_names_are_all_stops() {
        let focused = Rc::new(Cell::new(0));
        let items = boxed(&[radio(1, &focused, "a"), radio(2, &focused, "b")]);
        assert_eq!(radio_groups(&Fake::default(), &items), [None, None]);
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

    /// Only a hidden control is left: the walk reports no stop, so a caller crosses past it (4793).
    #[test]
    fn a_hidden_last_control_is_no_stop() {
        let focused = Rc::new(Cell::new(0));
        let hidden = Fake {
            id: 1,
            refuses: true,
            focused: focused.clone(),
            ..Fake::default()
        };
        let (root, items) = (Fake::default(), boxed(&[hidden]));
        assert!(!focus_first_stop(&root, &items, 0..1, false));
        #[cfg(feature = "focus-trap")]
        assert!(focus_stop_unconfirmed(&root, &items, 0..1, false));
    }
}
