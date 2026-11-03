//! A static sheet's blocks for one axis word, the body of its range copies (todo 5652).

use super::{Sx, SxEntry, SxModifierKey, SxPropertyKey};
use crate::css::condition_groups;

impl Sx {
    /// Every block the `data-state` word `token` switches on, with `token` dropped from its
    /// condition (`size-md && card` stays `when("card", ..)`). Descends through conditions and
    /// at-rules, not selectors: a selector's block keys another element's words.
    pub(crate) fn axis_block(&self, token: &str) -> Sx {
        self.entries()
            .iter()
            .fold(Sx::new(), |copy, entry| match entry {
                SxEntry::Nested {
                    modifier: SxModifierKey::Condition(condition),
                    sx,
                } => condition_groups(condition)
                    .into_iter()
                    .fold(copy, |copy, group| {
                        let holds = group.iter().any(|word| word == token);
                        let rest: Vec<_> = group.into_iter().filter(|word| word != token).collect();
                        let block = match holds {
                            true => sx.clone(),
                            false => sx.axis_block(token),
                        };
                        match (block.entries().is_empty(), holds && rest.is_empty()) {
                            (true, _) => copy,
                            (false, true) => copy.and(block),
                            (false, false) => copy.when(rest.join(" && "), block),
                        }
                    }),
                SxEntry::Nested {
                    modifier:
                        modifier @ (SxModifierKey::Media(_)
                        | SxModifierKey::Breakpoint(_)
                        | SxModifierKey::Supports(_)
                        | SxModifierKey::Container { .. }),
                    sx,
                } => {
                    let block = sx.axis_block(token);
                    match block.entries().is_empty() {
                        true => copy,
                        false => copy.modifier(modifier.clone(), block),
                    }
                }
                _ => copy,
            })
    }

    /// `token` swapped for the run `word` in nested selectors (`& [data-state~="card"] > x`),
    /// for a part that carries the same run words as the root.
    pub(crate) fn rekeyed(self, token: &str, word: &str) -> Sx {
        let (from, to) = (
            format!(r#"[data-state~="{token}"]"#),
            format!(r#"[data-state~="{word}"]"#),
        );
        self.entries()
            .iter()
            .fold(Sx::new(), |copy, entry| match entry {
                SxEntry::Declaration {
                    property: SxPropertyKey::Known(property),
                    value,
                } => copy.with_known_property(*property, value.clone()),
                SxEntry::Declaration {
                    property: SxPropertyKey::Raw(name),
                    value,
                } => copy.with(name.clone(), value.clone()),
                SxEntry::Nested { modifier, sx } => {
                    let modifier = match modifier {
                        SxModifierKey::Selector(selector) => {
                            SxModifierKey::Selector(selector.replace(&from, &to))
                        }
                        other => other.clone(),
                    };
                    copy.modifier(modifier, sx.clone().rekeyed(token, word))
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::sx::sx;

    #[test]
    fn a_block_comes_out_without_its_word() {
        let sheet = sx()
            .display("flex")
            .when("size-sm", sx().padding("4px"))
            .when("size-md", sx().padding("8px"))
            .when("disabled", sx().opacity("0.5"));
        assert_eq!(sheet.axis_block("size-sm"), sx().padding("4px"));
    }

    #[test]
    fn nested_and_compound_blocks_keep_their_other_words() {
        let sheet = sx()
            .when("card", sx().when("size-sm", sx().padding("4px")))
            .when("size-sm && inline", sx().gap("2px"))
            .when("size-xs || size-sm", sx().height("20px"))
            .media("(hover: hover)", sx().when("size-sm", sx().color("red")))
            .selector("& > b", sx().when("size-sm", sx().color("blue")));
        assert_eq!(
            sheet.axis_block("size-sm"),
            sx().when("card", sx().padding("4px"))
                .when("inline", sx().gap("2px"))
                .height("20px")
                .media("(hover: hover)", sx().color("red"))
        );
    }

    #[test]
    fn a_part_keyed_on_the_token_is_keyed_on_the_run_word() {
        let block = sx()
            .padding("4px")
            .selector(r#"& [data-state~="card"] > i"#, sx().inset("-1px"));
        assert_eq!(
            block.rekeyed("card", "md-:card"),
            sx().padding("4px")
                .selector(r#"& [data-state~="md-:card"] > i"#, sx().inset("-1px"))
        );
    }

    /// The copy's selector is the plain block's, the word swapped: same specificity.
    #[test]
    fn a_copy_keeps_the_plain_blocks_specificity() {
        use crate::css::Stylesheet;
        let sheet = sx().when("size-sm", sx().padding("4px"));
        let plain = Stylesheet::rooted_at(&sheet, "c");
        let copy = Stylesheet::rooted_at(
            &sx().media(
                "(min-width: 1px)",
                sx().when("md-:size-sm", sheet.axis_block("size-sm")),
            ),
            "c",
        );
        assert!(
            plain
                .as_str()
                .contains(r#".c[data-state~="size-sm"]{padding:4px"#),
            "{}",
            plain.as_str()
        );
        assert!(
            copy.as_str()
                .contains(r#".c[data-state~="md-:size-sm"]{padding:4px"#),
            "{}",
            copy.as_str()
        );
    }
}
