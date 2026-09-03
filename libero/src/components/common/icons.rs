use dioxus::prelude::*;

/// The library's glyphs.
///
/// libero ships no icon set on purpose - an icon set is a design decision a
/// project makes, not a component library. What lives here is the short list a
/// *component* cannot do without: a close button has to draw something, and
/// three modules drawing their own x is how the library ended up with three
/// slightly different ones.
///
/// **The convention, library-wide**: a new glyph goes here, and a module that
/// already has a private one moves it here the next time it is touched. The
/// glyphs still in `components/form/glyphs.rs` are the backlog.
///
/// A glyph here carries no size. It inherits `currentColor` and fills whatever
/// box it is given - `ActionIcon`'s base has `& svg { width: 100%; height: 100% }`,
/// which beats an `svg` presentation attribute anyway, so a hardcoded
/// `width="16px"` inside one was never doing anything. A caller that needs a
/// size states it on the element around the glyph.
///
/// `aria-hidden` is on the glyph rather than left to the caller: none of these
/// carry meaning a reader needs, and the one place a glyph *is* the whole
/// control - an icon-only button - is `ActionIcon`, which requires its own
/// `aria_label`.
#[component]
pub(crate) fn CloseIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M18 6L6 18" }
            path { d: "M6 6l12 12" }
        }
    }
}

/// Points down while its disclosure is closed. The rotation that turns it is
/// the caller's, on the element around it.
#[component]
pub(crate) fn ChevronDownIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M6 9l6 6 6-6" }
        }
    }
}

/// Two bars. Filled rather than stroked, so it keeps its weight at the small
/// sizes a corner control is drawn at.
#[component]
pub(crate) fn PauseIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path { d: "M6 5h4v14H6zM14 5h4v14h-4z" }
        }
    }
}

/// A right-pointing triangle, [`PauseIcon`]'s other state.
#[component]
pub(crate) fn PlayIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path { d: "M8 5v14l11-7z" }
        }
    }
}

/// A done mark: a completed step's marker.
#[component]
pub(crate) fn CheckIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M5 12l5 5 9-10" }
        }
    }
}

/// Points at a submenu, which opens to the right. A menu that flips its
/// submenu to the left keeps it pointing right, as native menus do.
#[component]
pub(crate) fn ChevronRightIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M9 6l6 6-6 6" }
        }
    }
}
