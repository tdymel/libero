/// Expands a comma-separated `selector()` pattern against the current
/// selectors: `&` is replaced by each current selector, a bare suffix (e.g.
/// `::before`) is appended to it. E.g. `"&::before, &::after"` against
/// `[".cls"]` yields `[".cls::before", ".cls::after"]`.
pub(crate) fn expand_selector(pattern: &str, current: &[String]) -> Vec<String> {
    split_top_level_commas(pattern)
        .into_iter()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .flat_map(|part| {
            current.iter().map(move |selector| {
                if part.contains('&') {
                    part.replace('&', selector)
                } else {
                    format!("{selector}{part}")
                }
            })
        })
        .collect()
}

/// Splits on the commas that separate selectors, and not on those inside
/// `:is(a, b)`, `[title='a, b']` or a quoted string.
fn split_top_level_commas(pattern: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut start = 0;

    for (index, char) in pattern.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match (quote, char) {
            (_, '\\') => escaped = true,
            (Some(open), _) if char == open => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(char),
            (None, '(' | '[') => depth += 1,
            (None, ')' | ']') => depth = depth.saturating_sub(1),
            (None, ',') if depth == 0 => {
                parts.push(&pattern[start..index]);
                start = index + 1;
            }
            (None, _) => {}
        }
    }
    parts.push(&pattern[start..]);
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_suffix_without_ampersand_is_appended() {
        assert_eq!(
            expand_selector("::before", &[".cls".to_string()]),
            vec![".cls::before"]
        );
    }

    #[test]
    fn explicit_ampersand_matches_plain_suffix() {
        assert_eq!(
            expand_selector("&::before", &[".cls".to_string()]),
            vec![".cls::before"]
        );
    }

    #[test]
    fn comma_list_expands_each_part() {
        assert_eq!(
            expand_selector("&::before, &::after", &[".cls".to_string()]),
            vec![".cls::before", ".cls::after"]
        );
    }

    #[test]
    fn mixed_ampersand_and_plain_parts_behave_the_same() {
        assert_eq!(
            expand_selector("&:hover, ::focus", &[".cls".to_string()]),
            vec![".cls:hover", ".cls::focus"]
        );
    }

    #[test]
    fn whitespace_around_parts_is_trimmed() {
        assert_eq!(
            expand_selector(" &::before , &::after ", &[".cls".to_string()]),
            vec![".cls::before", ".cls::after"]
        );
    }

    #[test]
    fn expands_across_multiple_current_selectors() {
        assert_eq!(
            expand_selector(
                "&::before, &::after",
                &[
                    ".cls[data-state~=\"a\"]".to_string(),
                    ".cls[data-state~=\"b\"]".to_string()
                ]
            ),
            vec![
                ".cls[data-state~=\"a\"]::before",
                ".cls[data-state~=\"b\"]::before",
                ".cls[data-state~=\"a\"]::after",
                ".cls[data-state~=\"b\"]::after",
            ]
        );
    }

    /// `Calendar`'s disabled cells: split on the inner comma, the second
    /// alternative became `.cls[data-slot='cell']` and matched nothing.
    #[test]
    fn a_comma_inside_a_functional_pseudo_class_is_not_a_separator() {
        assert_eq!(
            expand_selector(
                "& :is([data-slot='day'], [data-slot='cell']):disabled",
                &[".cls".to_string()]
            ),
            vec![".cls :is([data-slot='day'], [data-slot='cell']):disabled"]
        );
    }

    #[test]
    fn nested_lists_and_quoted_commas_stay_in_their_part() {
        assert_eq!(
            expand_selector("&:not(:has(a, b)), &[title='x, (y']", &[".cls".to_string()]),
            vec![".cls:not(:has(a, b))", ".cls[title='x, (y']"]
        );
    }
}
