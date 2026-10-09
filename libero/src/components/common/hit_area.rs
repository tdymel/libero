use crate::sx::{Sx, sx};

/// A finger, not a mouse. Blitz reports a fine pointer, so natively this never matches.
pub(crate) const COARSE_POINTER: &str = "(pointer: coarse)";

/// Half the shortfall below 44px (WCAG 2.5.5) out on each side; `0` from 44px up.
const COARSE_INSET: &str = "min(0px, calc((100% - 44px) / 2))";

/// Grows a positioned element's press target to 44x44 on a coarse pointer, drawn size
/// unchanged (todo 2707). `pseudo` is a free `::before`/`::after`; an `overflow` clips it.
pub(crate) fn coarse_hit_area_sx(pseudo: &str) -> Sx {
    sx().media(
        COARSE_POINTER,
        sx().selector(
            pseudo,
            sx().content("\"\"")
                .position("absolute")
                .inset(COARSE_INSET),
        ),
    )
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
}
