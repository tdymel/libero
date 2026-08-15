/// Splits a `when()` condition into an OR of AND-groups of state names.
/// `&&` binds tighter than `||`, so splitting on `||` first and then on `&&`
/// within each half already yields disjunctive-normal form (parentheses are
/// not supported).
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
