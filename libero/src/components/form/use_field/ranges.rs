use crate::{
    components::common::{Input, States},
    hooks::use_range_css,
    sx::StaticSx,
    theme::{Responsive, Size},
};

/// Registers `sheet`'s range copies for the run words in `states` (`base-md:size-sm`), so a
/// part carrying the field's states follows its responsive size and radius. A hook; a plain
/// field registers nothing.
pub(crate) fn use_state_ranges(sheet: &'static StaticSx, states: Option<&Input<States>>) {
    let axes = states
        .and_then(Input::as_ref)
        .map(run_axes)
        .unwrap_or_default();
    let mut copies: Vec<_> = axes
        .iter()
        .flat_map(|axis| {
            axis.range_copies(token_of, move |word| {
                sheet
                    .axis_block(token_of(word))
                    .rekeyed(token_of(word), word)
            })
        })
        .collect();
    // A rule on two axes' words (`card && size-sm`) keys plain words in each axis's copy.
    for (at, axis) in axes.iter().enumerate() {
        for other in &axes[at + 1..] {
            copies.extend(
                axis.cross_copies(token_of, other, token_of, move |word, other| {
                    sheet
                        .axis_block(token_of(word))
                        .axis_block(token_of(other))
                        .rekeyed(token_of(word), word)
                        .rekeyed(token_of(other), other)
                }),
            );
        }
    }
    use_range_css(sheet, copies);
}

/// The token a run word holds: `size-sm` of `base-md:size-sm`.
fn token_of(word: &'static str) -> &'static str {
    word.split_once(':').map_or(word, |(_, token)| token)
}

/// The run words of each responsive axis (`size`, `radius`), as values whose runs are the
/// ones the words came from, so `range_copies(token_of, ..)` names the same words.
fn run_axes(states: &States) -> Vec<Responsive<&'static str>> {
    let mut axes: Vec<Responsive<&'static str>> = Vec::new();
    for (word, _) in states.iter().filter(|(_, active)| *active) {
        let Some((range, _)) = word.split_once(':') else {
            continue;
        };
        let Some((from, _)) = range.split_once('-') else {
            continue;
        };
        // An axis's runs come in order from `base`, so `base` opens the next axis.
        match (from, axes.last_mut()) {
            ("base", _) => axes.push(Responsive::new(word)),
            (from, Some(value)) => {
                if let Some(from) = Size::parse_dynamic(from) {
                    *value = value.with(from, word);
                }
            }
            _ => {}
        }
    }
    axes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::responsive;

    #[test]
    fn run_words_rebuild_the_value_they_came_from() {
        let sizes = responsive(Size::Sm).md(Size::Xl);
        let radii = responsive(Size::Xs).sm(Size::Lg).xl(Size::Xs);
        let states = States::default()
            .active("disabled")
            .responsive(&sizes, |size| size.state_name())
            .responsive(&radii, |size| size.radius_state_name())
            .responsive(&responsive("outline").lg("filled"), |word| word);
        let axes = run_axes(&states);
        assert_eq!(axes.len(), 3);
        let words =
            |axis: &Responsive<&'static str>| axis.state_tokens(token_of).collect::<Vec<_>>();
        assert_eq!(words(&axes[2]), ["base-lg:outline", "lg-:filled"]);
        assert_eq!(
            words(&axes[0]),
            sizes
                .state_tokens(|size| size.state_name())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            words(&axes[1]),
            radii
                .state_tokens(|size| size.radius_state_name())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn plain_states_spell_no_axis() {
        let states = States::default().active("size-md").active("radius-sm");
        assert!(run_axes(&states).is_empty());
    }
}
