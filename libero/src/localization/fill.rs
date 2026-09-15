use std::fmt::Display;

/// Fills a template's named holes: `{n}` with the value paired with `"n"`.
///
/// One pass over the template, so a value that itself contains `{n}` stays
/// literal. A hole with no value, or a template with no holes, is kept as is.
///
/// ```
/// use libero::localization::fill;
///
/// assert_eq!(fill("Slide {n} of {m}", &[("n", &3), ("m", &7)]), "Slide 3 of 7");
/// ```
pub fn fill(template: &str, holes: &[(&str, &dyn Display)]) -> String {
    let mut filled = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        filled.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let value = after.find('}').and_then(|close| {
            let name = &after[..close];
            holes
                .iter()
                .find(|(hole, _)| *hole == name)
                .map(|(_, value)| (close, value))
        });
        match value {
            Some((close, value)) => {
                filled.push_str(&value.to_string());
                rest = &after[close + 1..];
            }
            None => {
                filled.push('{');
                rest = after;
            }
        }
    }
    filled.push_str(rest);
    filled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_hole_is_filled() {
        assert_eq!(
            fill("Slide {n} of {m}", &[("n", &3), ("m", &7)]),
            "Slide 3 of 7"
        );
        assert_eq!(fill("{m}, then {n}", &[("n", &1), ("m", &2)]), "2, then 1");
    }

    #[test]
    fn a_value_containing_a_hole_stays_literal() {
        assert_eq!(
            fill("{n} more: {names}", &[("names", &"{n}"), ("n", &2)]),
            "2 more: {n}"
        );
    }

    #[test]
    fn a_template_without_holes_or_with_unknown_ones_is_kept() {
        assert_eq!(fill("Bild", &[("n", &4)]), "Bild");
        assert_eq!(fill("{x} and {", &[("n", &4)]), "{x} and {");
    }
}
