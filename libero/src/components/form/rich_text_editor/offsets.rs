//! Caret offsets between the model (chars, an inline node or hard break counts 1) and
//! the DOM (UTF-16 code units, a `br` or an atom element counts 1).

use super::model::Inline;

/// The DOM offset of model offset `chars` in a leaf holding `inlines`.
pub(crate) fn to_dom(inlines: &[Inline], chars: usize) -> usize {
    let mut left = chars;
    let mut units = 0;
    for inline in inlines {
        match inline {
            Inline::Text { text, .. } => {
                for c in text.chars() {
                    if left == 0 {
                        return units;
                    }
                    units += c.len_utf16();
                    left -= 1;
                }
            }
            Inline::HardBreak | Inline::Node { .. } => {
                if left == 0 {
                    return units;
                }
                units += 1;
                left -= 1;
            }
        }
    }
    units
}

/// The model offset of DOM offset `units`. A unit inside a surrogate pair rounds down.
pub(crate) fn to_model(inlines: &[Inline], units: usize) -> usize {
    let mut left = units;
    let mut chars = 0;
    for inline in inlines {
        match inline {
            Inline::Text { text, .. } => {
                for c in text.chars() {
                    let width = c.len_utf16();
                    if left < width {
                        return chars;
                    }
                    left -= width;
                    chars += 1;
                }
            }
            Inline::HardBreak | Inline::Node { .. } => {
                if left == 0 {
                    return chars;
                }
                left -= 1;
                chars += 1;
            }
        }
    }
    chars
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf() -> Vec<Inline> {
        vec![
            Inline::text("a😀"),
            Inline::HardBreak,
            Inline::Node {
                name: "mention".into(),
                attrs: Default::default(),
            },
            Inline::text("é日"),
        ]
    }

    #[test]
    fn astral_chars_take_two_units() {
        let inlines = leaf();
        let pairs = [(0, 0), (1, 1), (2, 3), (3, 4), (4, 5), (5, 6), (6, 7)];
        for (chars, units) in pairs {
            assert_eq!(to_dom(&inlines, chars), units, "to_dom {chars}");
            assert_eq!(to_model(&inlines, units), chars, "to_model {units}");
        }
    }

    #[test]
    fn a_unit_inside_a_pair_rounds_down() {
        assert_eq!(to_model(&leaf(), 2), 1);
    }

    #[test]
    fn offsets_past_the_end_clamp() {
        assert_eq!(to_dom(&leaf(), 99), 7);
        assert_eq!(to_model(&leaf(), 99), 6);
        assert_eq!(to_dom(&[], 3), 0);
    }
}
