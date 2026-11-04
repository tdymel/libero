use crate::{
    components::common::{
        Input, States, Variables, input_from_str, inset_focus_ring_sx, variables,
    },
    platform::clamps_lines,
    str_enum::str_enum,
    sx::{Sx, sx},
    theme::{CssVar, FOCUS_RING_WIDTH, Responsive, Size, SizeCss},
};

// A line count, or one per breakpoint: `line_clamp: responsive(1).md(3)`.
crate::sx::input::input_from_responsive!(u32);

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

/// The one cut a text takes: `line_clamp` wins over `truncate`, and `0` is off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cut {
    None,
    Line(Truncate),
    Lines(u32),
    /// A count per breakpoint, which [`clamp_sx`] declares.
    LinesPerRange,
}

impl Cut {
    pub(crate) fn of(truncate: &Input<Truncate>, line_clamp: &Input<Responsive<u32>>) -> Self {
        let clamp = line_clamp.as_ref();
        let ranged = clamp.is_some_and(|clamp| {
            !clamp.is_plain() && (clamp.base() > 0 || clamp.breakpoints().any(|(_, n)| n > 0))
        });
        let lines = clamp.map(Responsive::base).filter(|&lines| lines > 0);
        match (ranged, lines, truncate.as_ref()) {
            (true, _, _) => Self::LinesPerRange,
            (false, Some(lines), _) => Self::Lines(lines),
            (false, None, Some(&cut)) => Self::Line(cut),
            (false, None, None) => Self::None,
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
            (
                LINE_CLAMP_STATE,
                matches!(self, Self::Lines(_) | Self::LinesPerRange),
            ),
        ]
    }

    /// A per-breakpoint clamp with a `truncate` cuts one line where its count is `0` (5659): the
    /// clamp and the truncate as run words. `None` otherwise, the cut's plain words.
    pub(crate) fn runs(
        self,
        truncate: &Input<Truncate>,
        line_clamp: &Input<Responsive<u32>>,
    ) -> Option<Responsive<&'static str>> {
        let (Self::LinesPerRange, Some(&cut), Some(clamp)) =
            (self, truncate.as_ref(), line_clamp.as_ref())
        else {
            return None;
        };
        let lifted = clamp.base() == 0 || clamp.breakpoints().any(|(_, n)| n == 0);
        lifted.then(|| {
            clamp.map(|n| match n {
                0 => cut.state_name(),
                _ => LINE_CLAMP_STATE,
            })
        })
    }

    /// The cut's words on `states`: [`Cut::runs`]' run words when set.
    pub(crate) fn with_states(
        self,
        states: States,
        runs: Option<&Responsive<&'static str>>,
    ) -> States {
        match runs {
            Some(runs) => states.responsive(runs, |word| word),
            None => self
                .states()
                .into_iter()
                .fold(states, |states, (state, on)| states.with(state, on)),
        }
    }

    /// Whether the text sits in a `bdi`, which a start cut in any range needs.
    pub(crate) fn isolates(self, runs: Option<&Responsive<&'static str>>) -> bool {
        let start = Truncate::Start.state_name();
        self.is_start()
            || runs.is_some_and(|runs| {
                runs.base() == start || runs.breakpoints().any(|(_, word)| word == start)
            })
    }

    pub(crate) fn variables(self) -> Input<Variables> {
        let lines = match self {
            Self::Lines(lines) => Some(lines.to_string()),
            _ => None,
        };
        variables().with(LINES_VAR, lines).into()
    }
}

/// A per-breakpoint `line_clamp`'s counts, for the user layer (an inline var would beat the
/// breakpoints); `none` lifts the clamp where the count is `0`. `None` when plain.
pub(crate) fn clamp_sx(line_clamp: &Input<Responsive<u32>>) -> Option<Sx> {
    let clamp = line_clamp.as_ref().filter(|clamp| !clamp.is_plain())?;
    let lines = |n: u32| match n {
        0 => "none".to_string(),
        n => n.to_string(),
    };
    let base = sx().var(LINES_VAR, lines(clamp.base()));
    Some(clamp.breakpoints().fold(base, |base, (size, n)| {
        base.breakpoint(size, sx().var(LINES_VAR, lines(n)))
    }))
}

// Clip, not hidden: focus cannot scroll a clip box, so a tabbed-to link leaves the line in place (5462).
fn clip_sx() -> Sx {
    sx().overflow("clip").selector(
        "& :focus-visible",
        // Inset: the cut box clips an outer ring (5463).
        inset_focus_ring_sx(&format!("calc(-1 * {})", FOCUS_RING_WIDTH.value())),
    )
}

fn one_line_sx() -> Sx {
    clip_sx()
        .display("block")
        .text_overflow("ellipsis")
        .white_space("nowrap")
}

fn clamp_height(line_height: SizeCss, size: Size) -> Sx {
    sx().max_height(format!(
        "calc({} * {} * 1em)",
        line_height.value(size),
        LINES_VAR.value()
    ))
}

/// Blitz's clamp height at `size`, for a size's range copy; nothing where line clamp works.
pub(crate) fn clamp_height_sx(line_height: SizeCss, size: Size) -> Sx {
    match clamps_lines() {
        true => sx(),
        false => sx().when(LINE_CLAMP_STATE, clamp_height(line_height, size)),
    }
}

/// The rules of [`Cut`]'s states, for a text whose line height is `line_height` per size.
pub(crate) fn truncate_sx(line_height: SizeCss) -> Sx {
    // Direction flips the cut to the start; the `bdi` inside keeps the text itself reading as written.
    let start = one_line_sx()
        .with("direction", "rtl")
        .text_align("left")
        .rtl(sx().with("direction", "ltr").text_align("right"));
    let clamp = clip_sx();
    // Blitz has no line clamp: a height of N lines cuts the text, without the ellipsis.
    let clamp = match clamps_lines() {
        true => clamp
            .display("-webkit-box")
            .with("-webkit-box-orient", "vertical")
            .with("-webkit-line-clamp", LINES_VAR.value()),
        false => Size::ALL
            .into_iter()
            .fold(clamp.display("block"), |clamp, size| {
                clamp.when(size.state_name(), clamp_height(line_height, size))
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
        assert_eq!(Cut::of(&end, &3.into()), Cut::Lines(3));
        assert_eq!(Cut::of(&end, &0.into()), Cut::Line(Truncate::End));
        assert_eq!(Cut::of(&end, &Input::None), Cut::Line(Truncate::End));
        assert_eq!(Cut::of(&Input::None, &0.into()), Cut::None);
    }

    #[test]
    fn a_clamp_per_breakpoint_declares_each_count_and_lifts_it_at_zero() {
        let clamp: Input<Responsive<u32>> = crate::theme::responsive(0).md(3).into();
        let cut = Cut::of(&Input::None, &clamp);

        assert_eq!(cut, Cut::LinesPerRange);
        assert_eq!(cut.variables(), Input::from(variables()));
        assert_eq!(
            clamp_sx(&clamp),
            Some(
                sx().var(LINES_VAR, "none")
                    .breakpoint(Size::Md, sx().var(LINES_VAR, "3"))
            )
        );
        assert_eq!(clamp_sx(&3.into()), None);
        assert_eq!(
            Cut::of(&Input::None, &crate::theme::responsive(0).md(0).into()),
            Cut::None
        );
    }

    /// Todo 5659: a `0` range with a `truncate` cuts one line there.
    #[test]
    fn a_zero_range_with_a_truncate_runs_the_truncate_there() {
        let end = Input::from(Truncate::End);
        let clamp: Input<Responsive<u32>> = crate::theme::responsive(0).md(3).into();
        let cut = Cut::of(&end, &clamp);
        assert_eq!(
            cut.runs(&end, &clamp),
            Some(crate::theme::responsive("truncate-end").md("line-clamp"))
        );
        // No truncate, or no `0` range: the plain clamp words.
        assert_eq!(cut.runs(&Input::None, &clamp), None);
        let counts: Input<Responsive<u32>> = crate::theme::responsive(2).md(3).into();
        assert_eq!(Cut::of(&end, &counts).runs(&end, &counts), None);
        assert!(cut.isolates(cut.runs(&Input::from(Truncate::Start), &clamp).as_ref()));
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
            "overflow:clip",
        ] {
            assert!(css.contains(declaration), "{declaration} missing: {css}");
        }
    }
}
