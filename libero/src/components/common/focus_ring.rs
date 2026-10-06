/// A `box-shadow` that draws nothing, so a shadow list always parses.
const NO_SHADOW: &str = "0 0 #0000";

/// The `:focus-visible` ring: a dark stripe between two light halo bands, so it reads on any surface.
/// The stripe is also a shadow: Blitz paints the outline under the shadows (todo 478).
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

/// The ring inset into the element's own fill, with no outer halo; `offset` is negative.
/// Inset shadows, as Blitz ignores `outline-offset`; the transparent outline serves forced colours.
pub(crate) fn inset_focus_ring_sx(offset: &str) -> crate::sx::Sx {
    use crate::theme::{FOCUS_RING_COLOR, FOCUS_RING_HALO, FOCUS_RING_WIDTH, OWN_SHADOW};
    use crate::tokens::NamedColorCss;

    let stripe = NamedColorCss::FOCUS_CONTRAST.value_or(FOCUS_RING_COLOR.value());
    let width = FOCUS_RING_WIDTH.value();
    let depth = format!("(-1 * ({offset}))");
    let halo_depth = format!("({depth} - {width})");
    focus_ring_sx()
        .outline(format!("{width} solid transparent"))
        .outline_offset(offset)
        .box_shadow(format!(
            "{},{},{}",
            inset_band(&halo_depth, &FOCUS_RING_HALO.value()),
            inset_band(&depth, &stripe),
            OWN_SHADOW.value_or(NO_SHADOW)
        ))
}

/// [`inset_focus_ring_sx`] with a painted outline, for a replaced element (`video`, `img`):
/// its content covers inset shadows, the web paints the outline over it.
pub(crate) fn inset_outline_ring_sx(offset: &str) -> crate::sx::Sx {
    use crate::theme::{FOCUS_RING_COLOR, FOCUS_RING_WIDTH};
    use crate::tokens::NamedColorCss;

    let stripe = NamedColorCss::FOCUS_CONTRAST.value_or(FOCUS_RING_COLOR.value());
    inset_focus_ring_sx(offset).outline(format!("{} solid {stripe}", FOCUS_RING_WIDTH.value()))
}

/// Four offset inset shadows: a band `depth` deep round the padding box.
fn inset_band(depth: &str, color: &str) -> String {
    format!(
        "inset calc{depth} 0 0 0 {color},inset calc(-1 * {depth}) 0 0 0 {color},\
         inset 0 calc{depth} 0 0 {color},inset 0 calc(-1 * {depth}) 0 0 {color}"
    )
}

/// A resting `box-shadow`, also published as `--lsx-own-shadow` so the focus ring,
/// itself a `box-shadow`, can compose it back in.
pub(crate) fn shadow_sx(shadow: String) -> crate::sx::Sx {
    crate::sx::sx()
        .box_shadow(shadow.clone())
        .var(crate::theme::OWN_SHADOW, shadow)
}

/// The on-state ring, `depth` of `currentColor` inside the edge (1.4.11).
/// Offset shadows, not a spread: Blitz ignores inset spread.
fn on_ring(depth: &str) -> String {
    inset_band(&format!("({depth})"), "currentColor")
}

/// A bordered control's ring depth. Blitz offsets inset shadows from the border
/// box, so 1px shows there under a 1px border; the web shows 2px (Maintainer, 715).
const ON_RING_BORDERED: &str = "2px";
const ON_RING_BORDERLESS: &str = "1px";

/// The house on-state line upright at the start edge (NavLink, a selected row).
const ON_LINE_WIDTH: &str = "2px";
const ON_LINE_LENGTH: &str = "min(50%, 1.5em)";

/// The on-state marker for a bordered control, so it never rests on a fill alone (1.4.1).
/// `resting` is a shadow kept under the ring.
pub(crate) fn on_ring_sx(resting: Option<&str>) -> crate::sx::Sx {
    let ring = on_ring(ON_RING_BORDERED);
    shadow_sx(match resting {
        Some(resting) => format!("{ring},{resting}"),
        None => ring,
    })
}

/// [`on_state_sx`] for a control without a border: a chromeless toggle.
pub(crate) fn borderless_on_state_sx() -> crate::sx::Sx {
    shadow_sx(on_ring(ON_RING_BORDERLESS)).and(forced_on_sx())
}

/// The on-state line at the start edge, for a row a full ring would confuse with focus.
/// `color` must reach 3:1 on the row's tint (1.4.11).
pub(crate) fn on_start_bar_sx(inset: &str, color: &str) -> crate::sx::Sx {
    crate::sx::sx()
        .with(
            "background-image",
            format!("linear-gradient({color}, {color})"),
        )
        .with(
            "background-size",
            format!("{ON_LINE_WIDTH} {ON_LINE_LENGTH}"),
        )
        .with("background-position", format!("left {inset} center"))
        .with("background-repeat", "no-repeat")
        .rtl(crate::sx::sx().with("background-position", format!("right {inset} center")))
        // Unforced by `forced_on_sx`: the author colour would vanish on `Highlight`.
        .media(
            crate::sx::FORCED_COLORS,
            crate::sx::sx().with(
                "background-image",
                "linear-gradient(currentColor, currentColor)",
            ),
        )
}

/// [`on_ring_sx`] for a control, plus [`forced_on_sx`].
pub(crate) fn on_state_sx(resting: Option<&str>) -> crate::sx::Sx {
    on_ring_sx(resting).and(forced_on_sx())
}

/// The `Highlight` pair for an on state under forced colours, over [`disabled_look_sx`].
/// Opted out of forcing, or Chromium backs the label with a `Canvas` plate (todo 686).
pub(crate) fn forced_on_sx() -> crate::sx::Sx {
    crate::sx::sx().media(
        crate::sx::FORCED_COLORS,
        crate::sx::sx()
            .with("forced-color-adjust", "none")
            .with("background-color", "Highlight !important")
            .border_color("Highlight !important")
            .color("HighlightText !important")
            .selector(
                "& *",
                crate::sx::sx()
                    .color("inherit !important")
                    .with("background-color", "transparent !important")
                    .border_color("currentColor !important"),
            ),
    )
}

/// The disabled look: half opacity, and `GrayText` under forced colours.
pub(crate) fn disabled_look_sx(cursor: &str) -> crate::sx::Sx {
    crate::sx::sx().opacity("0.5").cursor(cursor).media(
        crate::sx::FORCED_COLORS,
        crate::sx::sx().color("GrayText").border_color("GrayText"),
    )
}

/// Draws the ring for an element focus lands inside, via `:focus-visible ~ [data-ring]`:
/// stylo rejects `:has()`. Place it after the focusable, in a positioned owner.
pub(crate) fn ring_overlay() -> dioxus::prelude::Element {
    use dioxus::prelude::*;

    rsx! {
        span { "data-ring": true, "aria-hidden": "true" }
    }
}

/// The overlay's box: the owner's padding box and corners, never in the way of a click.
pub(crate) fn ring_overlay_sx() -> crate::sx::Sx {
    crate::sx::sx()
        .position("absolute")
        .inset("0")
        .border_radius("inherit")
        .pointer_events("none")
}

#[cfg(test)]
mod tests {
    use super::{focus_ring_sx, inset_focus_ring_sx, shadow_sx};
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
        assert!(css.contains(",var(--lsx-own-shadow, 0 0 #0000);"), "{css}");
    }

    /// Natively the outline lands outside, where a list clips it (todo 626):
    /// the stripe is inset shadows, halo first so it paints over the stripe's inside.
    #[test]
    fn an_inset_ring_draws_its_stripe_as_inset_shadows() {
        let css = Stylesheet::from(&inset_focus_ring_sx("-4px"));
        let css = css.as_str();

        assert!(
            css.contains("outline:var(--lsx-focus-ring-width) solid transparent;"),
            "{css}"
        );
        let halo = css
            .find("inset calc((-1 * (-4px)) - var(--lsx-focus-ring-width)) 0 0 0 var(--lsx-focus-ring-halo)")
            .expect("the halo band");
        let stripe = css
            .find("inset calc(-1 * (-4px)) 0 0 0 var(--lsx-focus-contrast, var(--lsx-focus-ring-color))")
            .expect("the stripe band");
        assert!(halo < stripe, "{css}");
        assert_eq!(css.matches("inset ").count(), 8, "{css}");
    }

    #[test]
    fn a_resting_shadow_is_published_as_well_as_drawn() {
        let css = Stylesheet::from(&shadow_sx("0 1px 2px #0003".to_string()));
        let css = css.as_str();

        assert!(css.contains("box-shadow:0 1px 2px #0003;"), "{css}");
        assert!(css.contains("--lsx-own-shadow:0 1px 2px #0003;"), "{css}");
    }
}
