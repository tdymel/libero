/// Splits a `when()` condition into an OR of AND-groups of state names.
/// `&&` binds tighter than `||`; no parentheses.
pub(crate) fn condition_groups(condition: &str) -> Vec<Vec<String>> {
    condition
        .split("||")
        .map(|group| {
            group
                .split("&&")
                .map(|state| state.trim().to_string())
                .filter(|state| !state.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|group| !group.is_empty())
        .collect()
}

/// Normalizes a `when()` condition so equivalent spellings hash alike.
pub(crate) fn canonical_condition(condition: &str) -> String {
    condition_groups(condition)
        .into_iter()
        .map(|group| group.join(" && "))
        .collect::<Vec<_>>()
        .join(" || ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_condition() {
        assert_eq!(condition_groups("checked"), vec![vec!["checked"]]);
    }

    #[test]
    fn and_condition() {
        assert_eq!(
            condition_groups("horizontal && label"),
            vec![vec!["horizontal", "label"]]
        );
    }

    #[test]
    fn or_condition() {
        assert_eq!(
            condition_groups("hover || focus"),
            vec![vec!["hover"], vec!["focus"]]
        );
    }

    #[test]
    fn canonical_condition_normalizes_spacing() {
        assert_eq!(canonical_condition("a&&b"), "a && b");
        assert_eq!(canonical_condition("  hover  ||focus "), "hover || focus");
    }

    #[test]
    fn mixed_condition_respects_and_precedence() {
        assert_eq!(
            condition_groups("a && b || c"),
            vec![vec!["a", "b"], vec!["c"]]
        );
    }

    #[test]
    fn trims_whitespace_around_terms() {
        assert_eq!(
            condition_groups("  a  &&  b  ||  c  "),
            vec![vec!["a", "b"], vec!["c"]]
        );
    }
}
