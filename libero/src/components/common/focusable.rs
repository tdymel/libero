/// What Tab can reach. The `:not(..)` tail drops hidden, inert, `tabindex="-1"` (roving items)
/// and `:disabled` elements; visually hidden but focusable ones, and CSS-hidden ones, still match.
pub(crate) const FOCUSABLE_SELECTOR: &str = concat!(
    ":is(a[href], button:not([disabled]), textarea:not([disabled]), ",
    "input:not([disabled], [type=\"hidden\"]), select:not([disabled]), [tabindex]:not([tabindex=\"-1\"]), ",
    "summary, iframe, audio[controls], video[controls], ",
    "[contenteditable]:not([contenteditable=\"false\"]))",
    ":not([hidden], [inert], [aria-hidden=\"true\"], [tabindex=\"-1\"], :disabled, ",
    "[hidden] *, [inert] *, [aria-hidden=\"true\"] *)"
);

#[cfg(test)]
mod tests {
    use super::FOCUSABLE_SELECTOR;

    /// The exclusion list, not `:is(..)`, must carry `tabindex="-1"`, or a button keeps its stop.
    #[test]
    fn a_tabindex_minus_one_element_is_never_a_tab_stop() {
        let (_, excluded) = FOCUSABLE_SELECTOR
            .split_once("):not(")
            .expect("an exclusion list after the :is(..)");

        assert!(
            excluded.contains(r#"[tabindex="-1"]"#),
            "{FOCUSABLE_SELECTOR}"
        );
    }

    /// A disabled button keeps no tab stop, whatever its `tabindex`.
    #[test]
    fn a_disabled_element_is_never_a_tab_stop() {
        let (_, excluded) = FOCUSABLE_SELECTOR
            .split_once("):not(")
            .expect("an exclusion list after the :is(..)");

        assert!(excluded.contains(":disabled"), "{FOCUSABLE_SELECTOR}");
    }

    /// A form's posted value (`type="hidden"`) is never a tab stop (todo 2498).
    #[test]
    fn a_hidden_input_is_never_a_tab_stop() {
        assert!(
            FOCUSABLE_SELECTOR.contains(r#"input:not([disabled], [type="hidden"])"#),
            "{FOCUSABLE_SELECTOR}"
        );
    }
}
