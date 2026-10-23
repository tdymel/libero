use crate::{
    components::common::{Input, Variables, input_from_str, variables},
    platform::clamps_lines,
    str_enum::str_enum,
    sx::{Sx, sx},
    theme::{CssVar, Size, SizeCss},
};

str_enum! {
    /// Which end of a one-line text is cut with an ellipsis.
    #[state_prefix = "truncate"]
    pub enum Truncate {
        #[default]
        End = "end",
        Start = "start",
    }
}

input_from_str!(Truncate);

const LINES_VAR: CssVar = CssVar::new("--lsx-lines");

const LINE_CLAMP_STATE: &str = "line-clamp";

/// The one cut a text takes: `line_clamp` wins over `truncate`, and `Some(0)` is off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cut {
    None,
    Line(Truncate),
    Lines(u32),
}

impl Cut {
    pub(crate) fn of(truncate: &Input<Truncate>, line_clamp: Option<u32>) -> Self {
        match (line_clamp.filter(|&lines| lines > 0), truncate.as_ref()) {
            (Some(lines), _) => Self::Lines(lines),
            (None, Some(&cut)) => Self::Line(cut),
            (None, None) => Self::None,
        }
    }

    pub(crate) fn is_start(self) -> bool {
        self == Self::Line(Truncate::Start)
    }

    /// The `data-state` tokens, for `States::with(token, true)`.
    pub(crate) fn states(self) -> [(&'static str, bool); 3] {
        [
            (
                Truncate::End.state_name(),
                self == Self::Line(Truncate::End),
            ),
            (Truncate::Start.state_name(), self.is_start()),
            (LINE_CLAMP_STATE, matches!(self, Self::Lines(_))),
        ]
    }

    pub(crate) fn variables(self) -> Input<Variables> {
        let lines = match self {
            Self::Lines(lines) => Some(lines.to_string()),
            _ => None,
        };
        variables().with(LINES_VAR, lines).into()
    }
}

fn one_line_sx() -> Sx {
    sx().display("block")
        .overflow("hidden")
        .text_overflow("ellipsis")
        .white_space("nowrap")
}

/// The rules of [`Cut`]'s states, for a text whose line height is `line_height` per size.
pub(crate) fn truncate_sx(line_height: SizeCss) -> Sx {
    // Direction flips the cut to the start; the `bdi` inside keeps the text itself reading as written.
    let start = one_line_sx()
        .with("direction", "rtl")
        .text_align("left")
        .rtl(sx().with("direction", "ltr").text_align("right"));
    let clamp = sx().overflow("hidden");
    // Blitz has no line clamp: a height of N lines cuts the text, without the ellipsis.
    let clamp = match clamps_lines() {
        true => clamp
            .display("-webkit-box")
            .with("-webkit-box-orient", "vertical")
            .with("-webkit-line-clamp", LINES_VAR.value()),
        false => Size::ALL
            .into_iter()
            .fold(clamp.display("block"), |clamp, size| {
                clamp.when(
                    size.state_name(),
                    sx().max_height(format!(
                        "calc({} * {} * 1em)",
                        line_height.value(size),
                        LINES_VAR.value()
                    )),
                )
            }),
    };
    sx().when(Truncate::End.state_name(), one_line_sx())
        .when(Truncate::Start.state_name(), start)
        .when(LINE_CLAMP_STATE, clamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{css::Stylesheet, sx::StaticSx, theme::TEXT_LINE_HEIGHT};

    static CUTS: StaticSx = StaticSx::new(|| truncate_sx(TEXT_LINE_HEIGHT));

    #[test]
    fn a_line_clamp_wins_over_a_truncate_and_zero_is_off() {
        let end = Input::from(Truncate::End);
        assert_eq!(Cut::of(&end, Some(3)), Cut::Lines(3));
        assert_eq!(Cut::of(&end, Some(0)), Cut::Line(Truncate::End));
        assert_eq!(Cut::of(&end, None), Cut::Line(Truncate::End));
        assert_eq!(Cut::of(&Input::None, Some(0)), Cut::None);
    }

    #[test]
    fn only_the_cut_in_effect_has_its_state() {
        let on = |cut: Cut| -> Vec<&str> {
            cut.states()
                .into_iter()
                .filter_map(|(state, on)| on.then_some(state))
                .collect()
        };
        assert_eq!(on(Cut::Line(Truncate::Start)), ["truncate-start"]);
        assert_eq!(on(Cut::Lines(2)), ["line-clamp"]);
        assert!(on(Cut::None).is_empty());
    }

    #[test]
    fn a_one_line_cut_ends_in_an_ellipsis() {
        let css = Stylesheet::from(&CUTS);
        let css = css.as_str();
        for declaration in [
            "text-overflow:ellipsis",
            "white-space:nowrap",
            "direction:rtl",
        ] {
            assert!(css.contains(declaration), "{declaration} missing: {css}");
        }
    }
}
