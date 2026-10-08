use std::rc::Rc;

use dioxus::prelude::*;

use crate::platform::{
    ElementApi, FocusMove, SilentFocusSubscription, focus_entered_from, focus_visible, silent_focus,
};

type Group = Box<dyn Fn() -> Vec<Option<Rc<MountedData>>>>;
type OnChange = Box<dyn Fn(FocusChange)>;

struct Current {
    group: Group,
    onchange: OnChange,
}

/// Focus arriving in or leaving one element of a [`use_focus_within`] group.
pub(crate) struct FocusChange<'a> {
    /// Its index in the group.
    pub(crate) element: usize,
    /// Focus arrived in it (`focusin`), else it left (`focusout`).
    pub(crate) within: bool,
    /// Whether focus is in any element of the group now. `None` where focus
    /// has not landed yet: a `focusout`.
    pub(crate) in_group: Option<bool>,
    source: Source<'a>,
}

/// What reported a [`FocusChange`].
enum Source<'a> {
    Event(&'a Event<FocusData>),
    Move(&'a dyn FocusMove),
}

impl FocusChange<'_> {
    /// [`focus_visible`] of the `focusin`; `None` for a silent move.
    pub(crate) fn focus_visible(&self) -> Option<bool> {
        match self.source {
            Source::Event(event) => focus_visible(event),
            Source::Move(_) => None,
        }
    }

    /// [`focus_entered_from`] of the `focusin`, or of the silent move.
    pub(crate) fn entered_from(&self, boundary: &str) -> Option<Option<Box<dyn ElementApi>>> {
        match self.source {
            Source::Event(event) => focus_entered_from(event, boundary),
            Source::Move(moved) => moved.entered_from(boundary),
        }
    }
}

/// The focus listeners of a [`use_focus_within`] group, for its elements.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct FocusWithin {
    current: CopyValue<Rc<Current>>,
}

impl FocusWithin {
    /// The `focusin` (or `focus`) listener of the group's element `element`.
    pub(crate) fn focusin(&self, element: usize) -> impl FnMut(Event<FocusData>) + 'static {
        let current = self.current;
        move |event| {
            let current = current.peek().clone();
            (current.onchange)(FocusChange {
                element,
                within: true,
                in_group: Some(true),
                source: Source::Event(&event),
            });
        }
    }

    /// The `focusout` (or `blur`) listener of the group's element `element`.
    pub(crate) fn focusout(&self, element: usize) -> impl FnMut(Event<FocusData>) + 'static {
        let current = self.current;
        move |event| {
            let current = current.peek().clone();
            (current.onchange)(FocusChange {
                element,
                within: false,
                in_group: None,
                source: Source::Event(&event),
            });
        }
    }

    /// Replaces the group and `onchange`, as a render of the hook does.
    pub(crate) fn watch(
        &self,
        group: impl Fn() -> Vec<Option<Rc<MountedData>>> + 'static,
        onchange: impl Fn(FocusChange) + 'static,
    ) {
        let mut current = self.current;
        current.set(Rc::new(Current {
            group: Box::new(group),
            onchange: Box::new(onchange),
        }));
    }
}

/// Calls `onchange` as focus enters or leaves an element of `group`, also for a
/// move that fires no focus event (Blitz's Tab, libero's own `focus()`).
pub(crate) fn use_focus_within(
    group: impl Fn() -> Vec<Option<Rc<MountedData>>> + 'static,
    onchange: impl Fn(FocusChange) + 'static,
) -> FocusWithin {
    let focus = FocusWithin {
        current: use_hook(|| {
            CopyValue::new(Rc::new(Current {
                group: Box::new(Vec::new),
                onchange: Box::new(|_| {}),
            }))
        }),
    };
    focus.watch(group, onchange);
    // Fixed per build, so the hook order holds; the web pays no hook slot.
    if silent_focus().is_some() {
        let current = focus.current;
        use_hook(|| {
            let subscription: Option<Box<dyn SilentFocusSubscription>> = silent_focus()
                .map(|api| api.on_move(Box::new(move |moved| report(current, moved))));
            Rc::new(subscription)
        });
    }
    focus
}

/// One silent move, as the changes of each element it moved into or out of.
fn report(current: CopyValue<Rc<Current>>, moved: &dyn FocusMove) {
    let Ok(current) = current.try_peek().map(|current| current.clone()) else {
        return;
    };
    let held: Vec<(bool, bool)> = (current.group)()
        .iter()
        .map(|mounted| {
            mounted.as_ref().map_or((false, false), |mounted| {
                (moved.was_in(mounted), moved.is_in(mounted))
            })
        })
        .collect();
    let in_group = Some(held.iter().any(|&(_, is)| is));
    for (element, within) in changes(&held) {
        (current.onchange)(FocusChange {
            element,
            within,
            in_group,
            source: Source::Move(moved),
        });
    }
}

/// Each element whose `(was, is)` differs, as `(index, is)`: those focus left
/// first, so a caller counting moves sees the arrival last, as on the web.
fn changes(held: &[(bool, bool)]) -> Vec<(usize, bool)> {
    [false, true]
        .into_iter()
        .flat_map(|within| {
            held.iter()
                .enumerate()
                .filter(move |&(_, &(was, is))| was != is && is == within)
                .map(move |(element, _)| (element, within))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::changes;

    #[test]
    fn a_move_between_elements_reports_the_one_left_first() {
        assert_eq!(
            changes(&[(false, true), (true, false)]),
            [(1, false), (0, true)]
        );
    }

    #[test]
    fn an_element_that_kept_or_never_held_focus_is_not_reported() {
        assert_eq!(changes(&[(true, true), (false, false)]), []);
        assert_eq!(changes(&[(true, true), (false, true)]), [(1, true)]);
    }
}
