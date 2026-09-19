use dioxus::prelude::*;

/// The library's glyphs.
///
/// libero ships no icon set on purpose - an icon set is a design decision a
/// project makes, not a component library. What lives here is the short list a
/// *component* cannot do without: a close button has to draw something, and
/// three modules drawing their own x is how the library ended up with three
/// slightly different ones.
///
/// **The convention, library-wide**: a new glyph goes here. No component keeps
/// a private one (todo 95 swept the last of them in).
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

/// Points back: a calendar's previous month, a pager's previous page.
#[component]
pub(crate) fn ChevronLeftIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M15 6l-6 6 6 6" }
        }
    }
}

/// Points up: a vertical carousel's previous slide.
#[component]
pub(crate) fn ChevronUpIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M18 15l-6-6-6 6" }
        }
    }
}

/// A chevron against a bar, so "first" does not read as one more step back.
#[component]
pub(crate) fn ChevronFirstIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M17 6l-6 6 6 6" }
            path { d: "M7 6v12" }
        }
    }
}

/// [`ChevronFirstIcon`] mirrored.
#[component]
pub(crate) fn ChevronLastIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M7 6l6 6-6 6" }
            path { d: "M17 6v12" }
        }
    }
}

/// A sortable column's arrow. The header flips it for ascending.
#[component]
pub(crate) fn ArrowDownIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M12 5v14" }
            path { d: "M6 13l6 6 6-6" }
        }
    }
}

/// A number field's step down.
#[component]
pub(crate) fn MinusIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            "aria-hidden": "true",
            path { d: "M6 12h12" }
        }
    }
}

/// A number field's step up.
#[component]
pub(crate) fn PlusIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            "aria-hidden": "true",
            path { d: "M12 6v12" }
            path { d: "M6 12h12" }
        }
    }
}

/// A password field's reveal button, while the secret is hidden.
#[component]
pub(crate) fn EyeIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z" }
            circle { cx: "12", cy: "12", r: "3" }
        }
    }
}

/// [`EyeIcon`] struck through, while the secret is shown.
#[component]
pub(crate) fn EyeOffIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M2 12s3.5-7 10-7c2 0 3.8.7 5.2 1.6" }
            path { d: "M21.5 10.4c.3.6.5 1.1.5 1.6 0 0-3.5 7-10 7-1.3 0-2.5-.3-3.5-.7" }
            path { d: "M9.9 9.9a3 3 0 0 0 4.2 4.2" }
            path { d: "M3 3l18 18" }
        }
    }
}

/// A file field's dropzone prompt. A tray with an arrow going into it, which
/// is the shape every upload control has settled on.
#[component]
pub(crate) fn UploadIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M12 16V4" }
            path { d: "M8 8l4-4 4 4" }
            path { d: "M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" }
        }
    }
}

/// `ThemeToggle` when a press hands the choice back to the platform: a
/// disc half filled, the usual "automatic" mark.
#[component]
pub(crate) fn SystemSchemeIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "9" }
            path { d: "M12 3a9 9 0 0 0 0 18z", fill: "currentColor" }
        }
    }
}

/// `ThemeToggle` when a press switches to light.
#[component]
pub(crate) fn SunIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "4" }
            path { d: "M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M19.1 4.9l-1.4 1.4M6.3 17.7l-1.4 1.4" }
        }
    }
}

/// `ThemeToggle` when a press switches to dark.
#[component]
pub(crate) fn MoonIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M20 14.5A8.5 8.5 0 1 1 9.5 4a6.6 6.6 0 0 0 10.5 10.5z" }
        }
    }
}

/// `RepoButton`'s GitHub mark.
#[component]
pub(crate) fn GitHubIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 16 16",
            fill: "currentColor",
            "aria-hidden": "true",
            path {
                d: "M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82a7.4 7.4 0 0 1 2-.27c.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z"
            }
        }
    }
}

/// `RepoButton`'s GitLab mark (Simple Icons, CC0).
#[component]
pub(crate) fn GitLabIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path {
                d: "m23.6004 9.5927-.0337-.0862L20.3.9814a.851.851 0 0 0-.3362-.405.8748.8748 0 0 0-.9997.0539.8748.8748 0 0 0-.29.4399l-2.2055 6.748H7.5375l-2.2057-6.748a.8573.8573 0 0 0-.29-.4412.8748.8748 0 0 0-.9997-.0537.8585.8585 0 0 0-.3362.4049L.4332 9.5015l-.0325.0862a6.0657 6.0657 0 0 0 2.0119 7.0105l.0113.0087.03.0213 4.976 3.7264 2.462 1.8633 1.4995 1.1321a1.0085 1.0085 0 0 0 1.2197 0l1.4995-1.1321 2.4619-1.8633 5.006-3.7489.0125-.01a6.0682 6.0682 0 0 0 2.0094-7.003z"
            }
        }
    }
}

/// `DirectionToggle`: a pilcrow over an arrow pointing where a press turns
/// the text, left for right to left.
#[component]
pub(crate) fn TextDirectionIcon(to_rtl: bool) -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M11 3v11" }
            path { d: "M15 3v11" }
            path { d: "M18 3h-7.5a4 4 0 0 0 0 8h.5" }
            path { d: "M20 19H4" }
            path { d: if to_rtl { "M7 16l-3 3 3 3" } else { "M17 16l3 3-3 3" } }
        }
    }
}

/// `ColorField`'s eyedropper button: a pipette, tip at the bottom left.
#[component]
pub(crate) fn EyeDropperIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M11 7l6 6" }
            path { d: "M4 16l11.7 -11.7a1 1 0 0 1 1.4 0l2.6 2.6a1 1 0 0 1 0 1.4l-11.7 11.7h-4v-4z" }
        }
    }
}

/// A code block's copy button.
#[component]
pub(crate) fn CopyIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            rect { x: "9", y: "9", width: "13", height: "13", rx: "2" }
            path { d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
        }
    }
}

/// [`CopyIcon`]'s other state. A check, but not [`CheckIcon`]'s path: the two
/// were drawn apart and are kept apart so the move changed nothing visible.
#[component]
pub(crate) fn CopiedIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            polyline { points: "20 6 9 17 4 12" }
        }
    }
}

/// [`CopyIcon`]'s state after a denied write: a circled exclamation mark.
#[component]
pub(crate) fn CopyFailedIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "9" }
            path { d: "M12 8v4" }
            path { d: "M12 16h.01" }
        }
    }
}

/// An `Anchor` that opens a new tab: a box with an arrow leaving it.
#[component]
pub(crate) fn ExternalLinkIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M15 3h6v6" }
            path { d: "M10 14L21 3" }
            path { d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" }
        }
    }
}

/// The last link in `Avatar`'s fallback chain: a head and shoulders.
#[component]
pub(crate) fn PersonIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path {
                d: "M12 12a5 5 0 1 0 0-10 5 5 0 0 0 0 10Zm0 1.8c-4.1 0-7.4 2.1-7.4 4.7V21h14.8v-2.5c0-2.6-3.3-4.7-7.4-4.7Z",
            }
        }
    }
}

/// A checkbox's mark: the check, or the dash while `indeterminate`. Heavier
/// than the other glyphs (stroke 3), since it is drawn at 65% of a small box.
#[component]
pub(crate) fn CheckboxMarkIcon(indeterminate: bool) -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            // This casing: Blitz's svg parser matches `currentColor` exactly.
            stroke: "currentColor",
            stroke_width: "3",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            if indeterminate {
                path { d: "M6 12h12" }
            } else {
                path { d: "M5 13l4 4L19 7" }
            }
        }
    }
}
