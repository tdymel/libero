/// Whether a key press carries a modifier that marks it a browser or OS
/// shortcut rather than ordinary typing.
///
/// Shift is deliberately not one of them: it is how a capital letter is typed,
/// so a typeahead that ignored `Shift+A` would refuse half the alphabet. Every
/// list that types to search - `Menu`, `Menubar`, `Tree` and `Select` - asks
/// this one question.
pub(crate) fn has_shortcut_modifier(event: &dioxus::prelude::KeyboardEvent) -> bool {
    use dioxus::prelude::ModifiersInteraction;

    let modifiers = event.data().modifiers();
    modifiers.ctrl() || modifiers.alt() || modifiers.meta()
}

/// What an arrow, Home, End, PageUp or PageDown under Ctrl, Alt or Meta means
/// to a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NavigationChord {
    /// Alt+ArrowDown: a combobox opens its popup (APG).
    Open,
    /// Alt+ArrowUp: a combobox closes its popup (APG).
    Close,
    /// Any other: the text caret's or the browser's (Alt+ArrowLeft is Back).
    Browser,
}

/// The press's [`NavigationChord`]; `None` for a plain press or another key.
pub(crate) fn navigation_chord(event: &dioxus::prelude::KeyboardEvent) -> Option<NavigationChord> {
    use dioxus::prelude::{Key, ModifiersInteraction};

    let navigation = matches!(
        event.key(),
        Key::ArrowDown
            | Key::ArrowUp
            | Key::ArrowLeft
            | Key::ArrowRight
            | Key::Home
            | Key::End
            | Key::PageUp
            | Key::PageDown
    );
    if !navigation || !has_shortcut_modifier(event) {
        return None;
    }
    let modifiers = event.data().modifiers();
    let alt_only = modifiers.alt() && !modifiers.ctrl() && !modifiers.meta();
    Some(match event.key() {
        Key::ArrowDown if alt_only => NavigationChord::Open,
        Key::ArrowUp if alt_only => NavigationChord::Close,
        _ => NavigationChord::Browser,
    })
}

/// A non-global attribute to push into a `Vec<Attribute>` by hand.
pub(crate) fn attr<T>(
    name: &'static str,
    value: impl dioxus::core::IntoAttributeValue<T>,
) -> dioxus::prelude::Attribute {
    dioxus::prelude::Attribute::new(name, value, None, false)
}

/// A `box-shadow` that draws nothing, as the tail of a shadow list. An
/// element with no resting shadow still needs the list to parse.
const NO_SHADOW: &str = "0 0 #0000";

/// The library's `:focus-visible` ring: a dark stripe with a light halo on
/// both sides of it.
///
/// The halo is a `box-shadow` spreading past the stripe, and painting order
/// does the rest - a `box-shadow` is painted *under* the element's `outline`.
/// With the default numbers a 6px halo, a 2px stripe and a 2px offset land as
/// light 0-2px, dark 2-4px, light 4-6px. So the stripe always has the halo
/// next to it, and the indicator reads at the ratio between its own two tones
/// rather than against a surface the caller owns - which is the only surface
/// there is, for a ring drawn outside the element.
///
/// The stripe still yields to `--lsx-focus-contrast` where a surface
/// publishes one: a component that knows what reads against itself keeps
/// winning. Every number and both colours come from `theme.focus_ring`.
///
/// The stripe is drawn twice, as the outline and as a shadow between two halo
/// shadows: Blitz paints the outline under the shadows (todo 478). The web
/// paints the outline over the same pixels, and forced colours keep only it.
pub(crate) fn focus_ring_sx() -> crate::sx::Sx {
    use crate::theme::{
        FOCUS_RING_COLOR, FOCUS_RING_HALO, FOCUS_RING_HALO_SPREAD, FOCUS_RING_OFFSET,
        FOCUS_RING_WIDTH, OWN_SHADOW,
    };
    use crate::tokens::NamedColorCss;

    let stripe = NamedColorCss::FOCUS_CONTRAST.value_or(FOCUS_RING_COLOR.value());
    let (offset, width) = (FOCUS_RING_OFFSET.value(), FOCUS_RING_WIDTH.value());
    let halo = FOCUS_RING_HALO.value();
    crate::sx::sx()
        .outline(format!("{width} solid {stripe}"))
        .outline_offset(offset.clone())
        .box_shadow(format!(
            "0 0 0 {offset} {halo},0 0 0 calc({offset} + {width}) {stripe},0 0 0 {} {halo},{}",
            FOCUS_RING_HALO_SPREAD.value(),
            OWN_SHADOW.value_or(NO_SHADOW)
        ))
}

/// The same ring, inset into the element's own fill - a picked day, a
/// highlighted option, the current thumbnail.
///
/// No halo, on purpose. An inset ring's surround is the element's own fill,
/// which the component chose and can be sure of; a halo would only paint a
/// light band *outside* an indicator that is drawn inside. `offset` is
/// negative, and `-width` or beyond puts the whole stripe on the fill.
pub(crate) fn inset_focus_ring_sx(offset: &str) -> crate::sx::Sx {
    focus_ring_sx()
        .outline_offset(offset)
        .box_shadow(crate::theme::OWN_SHADOW.value_or(NO_SHADOW))
}

/// A resting `box-shadow` on a focusable, written so the focus ring can
/// compose it back in.
///
/// The ring's halo is itself a `box-shadow`, and a `:focus-visible` arm that
/// sets the property drops whatever the resting rule put there - an
/// `Elevated` button would lose its elevation for as long as it held focus.
/// Declaring the value twice, once as the property and once as
/// `--lsx-own-shadow`, is what lets the ring put it back.
pub(crate) fn shadow_sx(shadow: String) -> crate::sx::Sx {
    crate::sx::sx()
        .box_shadow(shadow.clone())
        .var(crate::theme::OWN_SHADOW, shadow)
}

/// The stand-in that draws a focus ring for an element the focus lands
/// *inside* of - a field's frame, a checkbox's box. Placed after the
/// focusable, so `:focus-visible ~ [data-ring]` reaches it: a sibling rule.
/// The obvious `:has(:focus-visible)` never matches natively, because stylo
/// rejects `:has()` ([[codebase/blitz-platform-gaps]]), and there is no
/// `:focus-visible-within`.
///
/// It covers its containing block - the nearest positioned ancestor, which
/// must be the element the ring belongs to - and is styled there with
/// [`ring_overlay_sx`]. Hidden from assistive tech, and never a tab stop.
pub(crate) fn ring_overlay() -> dioxus::prelude::Element {
    use dioxus::prelude::*;

    rsx! {
        span { "data-ring": true, "aria-hidden": "true" }
    }
}

/// The overlay's own box: over the whole padding box, with the owner's
/// corners, and never in the way of a click. An owner with a border moves it
/// out by the border's width, so the ring's offset is measured from the same
/// edge as an outline on the owner.
pub(crate) fn ring_overlay_sx() -> crate::sx::Sx {
    crate::sx::sx()
        .position("absolute")
        .inset("0")
        .border_radius("inherit")
        .pointer_events("none")
}

/// A user-supplied value as a CSS string literal, quotes included. Rust's
/// `{:?}` is not CSS escaping - it emits `\u{...}`, which no selector parses.
pub(crate) fn css_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            // A hex escape needs its terminating space, or the next character
            // is read as part of the escape.
            '\0'..='\u{1f}' | '\u{7f}' => out.push_str(&format!("\\{:x} ", ch as u32)),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::{css_string, focus_ring_sx, inset_focus_ring_sx, shadow_sx};
    use crate::css::Stylesheet;

    #[test]
    fn the_ring_draws_both_tones_and_takes_every_number_from_the_theme() {
        let css = Stylesheet::from(&focus_ring_sx());
        let css = css.as_str();

        assert!(
            css.contains(
                "outline:var(--lsx-focus-ring-width) solid \
                 var(--lsx-focus-contrast, var(--lsx-focus-ring-color));"
            ),
            "{css}"
        );
        assert!(
            css.contains("outline-offset:var(--lsx-focus-ring-offset);"),
            "{css}"
        );
        // Halo, stripe, halo, then the resting shadow under all three.
        assert!(
            css.contains(
                "box-shadow:0 0 0 var(--lsx-focus-ring-offset) var(--lsx-focus-ring-halo),\
                 0 0 0 calc(var(--lsx-focus-ring-offset) + var(--lsx-focus-ring-width)) \
                 var(--lsx-focus-contrast, var(--lsx-focus-ring-color)),\
                 0 0 0 var(--lsx-focus-ring-halo-spread) var(--lsx-focus-ring-halo),\
                 var(--lsx-own-shadow, 0 0 #0000);"
            ),
            "{css}"
        );
    }

    #[test]
    fn an_inset_ring_drops_the_halo_but_keeps_a_resting_shadow() {
        let css = Stylesheet::from(&inset_focus_ring_sx("-4px"));
        let css = css.as_str();

        assert!(css.contains("outline-offset:-4px;"), "{css}");
        assert!(!css.contains("--lsx-focus-ring-halo-spread"), "{css}");
        assert!(
            css.contains("box-shadow:var(--lsx-own-shadow, 0 0 #0000);"),
            "{css}"
        );
    }

    #[test]
    fn a_resting_shadow_is_published_as_well_as_drawn() {
        let css = Stylesheet::from(&shadow_sx("0 1px 2px #0003".to_string()));
        let css = css.as_str();

        assert!(css.contains("box-shadow:0 1px 2px #0003;"), "{css}");
        assert!(css.contains("--lsx-own-shadow:0 1px 2px #0003;"), "{css}");
    }

    #[test]
    fn quotes_and_escapes_only_what_css_requires() {
        assert_eq!(css_string("node-1"), r#""node-1""#);
        assert_eq!(css_string(r#"a"b\c"#), r#""a\"b\\c""#);
        assert_eq!(css_string("über"), r#""über""#);
        assert_eq!(css_string("a\nb"), "\"a\\a b\"");
    }
}
