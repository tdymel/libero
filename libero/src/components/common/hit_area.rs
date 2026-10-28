use crate::{
    components::common::Orientation,
    sx::{Sx, sx},
    theme::CssVar,
};

/// A finger, not a mouse. Blitz reports a fine pointer, so natively this never matches.
pub(crate) const COARSE_POINTER: &str = "(pointer: coarse)";

/// The finger-sized target (WCAG 2.5.5); a row holding a [`coarse_hit_area_sx`] is this tall.
pub(crate) const COARSE_TARGET: &str = "44px";

/// Half the shortfall below 44px (WCAG 2.5.5) out on each side; `0` from 44px up.
pub(crate) const COARSE_INSET: &str = "min(0px, calc((100% - 44px) / 2))";

/// Half the shortfall below 24px (WCAG 2.5.8) out on each side; `0` from 24px up.
const MIN_INSET: &str = "min(0px, calc((100% - 24px) / 2))";

/// Grows a positioned element's press target to 24x24 for every pointer, drawn size
/// unchanged. `pseudo` is a free `::before`/`::after`; an `overflow` clips it.
pub(crate) fn min_hit_area_sx(pseudo: &str) -> Sx {
    sx().selector(
        pseudo,
        sx().content("\"\"").position("absolute").inset(MIN_INSET),
    )
}

/// How far a coarse box may reach out along a row or a column; unset, the full 44px.
const COARSE_REACH_INLINE: CssVar = CssVar::new("--lsx-coarse-reach-inline");
const COARSE_REACH_BLOCK: CssVar = CssVar::new("--lsx-coarse-reach-block");

/// [`COARSE_INSET`], reaching no further than `reach` but keeping the 24px box.
fn reach_inset(reach: &CssVar) -> String {
    format!(
        "max({COARSE_INSET}, min({MIN_INSET}, calc(-1 * {})))",
        reach.value_or("22px")
    )
}

/// Grows a positioned element's press target to 44x44 on a coarse pointer, drawn size
/// unchanged (todo 2707). `pseudo` is a free `::before`/`::after`; an `overflow` clips it.
pub(crate) fn coarse_hit_area_sx(pseudo: &str) -> Sx {
    sx().media(
        COARSE_POINTER,
        sx().selector(
            pseudo,
            sx().content("\"\"")
                .position("absolute")
                .inset_block(reach_inset(&COARSE_REACH_BLOCK))
                .inset_inline(reach_inset(&COARSE_REACH_INLINE)),
        ),
    )
}

/// Set on a bar or row with `gap` between its items along `orientation`: each item's
/// [`coarse_hit_area_sx`] then reaches half the gap that way, so no tap lands on a neighbour.
pub(crate) fn coarse_reach_sx(orientation: Orientation, gap: impl std::fmt::Display) -> Sx {
    let reach = match orientation {
        Orientation::Horizontal => COARSE_REACH_INLINE,
        Orientation::Vertical => COARSE_REACH_BLOCK,
    };
    sx().var(reach, format!("calc({gap} / 2)"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;
    use crate::sx::StaticSx;

    static PROBE: StaticSx = StaticSx::new(|| coarse_hit_area_sx("::before"));

    #[test]
    fn the_hit_area_exists_only_under_a_coarse_pointer() {
        let css = Stylesheet::from(&PROBE);
        let css = css.as_str();
        let media = css
            .find("@media (pointer: coarse)")
            .unwrap_or_else(|| panic!("no coarse query: {css}"));
        let inset = css
            .find(COARSE_INSET)
            .unwrap_or_else(|| panic!("no inset: {css}"));
        assert!(media < inset, "{css}");
        assert!(css[media..].contains("::before"), "{css}");
    }

    static ROW: StaticSx = StaticSx::new(|| coarse_reach_sx(Orientation::Horizontal, "4px"));
    static COLUMN: StaticSx = StaticSx::new(|| coarse_reach_sx(Orientation::Vertical, "6px"));

    #[test]
    fn a_bar_caps_the_reach_along_its_axis_only() {
        let css = Stylesheet::from(&PROBE);
        let css = css.as_str();
        for reach in [&COARSE_REACH_INLINE, &COARSE_REACH_BLOCK] {
            assert!(css.contains(&reach_inset(reach)), "{css}");
        }
        assert!(reach_inset(&COARSE_REACH_INLINE).contains(MIN_INSET));

        let row = Stylesheet::from(&ROW);
        assert!(
            row.as_str()
                .contains("--lsx-coarse-reach-inline:calc(4px / 2)"),
            "{}",
            row.as_str()
        );
        assert!(!row.as_str().contains("reach-block"), "{}", row.as_str());
        let column = Stylesheet::from(&COLUMN);
        assert!(
            column
                .as_str()
                .contains("--lsx-coarse-reach-block:calc(6px / 2)"),
            "{}",
            column.as_str()
        );
    }
}
