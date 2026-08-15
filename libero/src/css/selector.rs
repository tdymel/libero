/// Expands a comma-separated `selector()` pattern against the current
/// selectors: `&` is replaced by each current selector, a bare suffix (e.g.
/// `::before`) is appended to it. E.g. `"&::before, &::after"` against
/// `[".cls"]` yields `[".cls::before", ".cls::after"]`.
pub(crate) fn expand_selector(pattern: &str, current: &[String]) -> Vec<String> {
    pattern
        .split(',')
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
}
