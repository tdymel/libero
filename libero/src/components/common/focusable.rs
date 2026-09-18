/// `FocusTrap` wraps arbitrary consumer children, so the tab order has to
/// exclude what a browser would never focus. The `:not(..)` tail drops
/// `hidden`/`inert`/`aria-hidden` elements *and their descendants* - without
/// it, Tab strands focus on something invisible - and anything with
/// `tabindex="-1"`, which a browser's Tab skips even on a button. Without that,
/// Tab walked every roving item: all six of a lightbox's thumbnails, where the
/// strip has one tab stop. `:disabled` too: a disabled button with
/// `tabindex="0"` (a calendar's Nav at `min`) matched the `[tabindex]` arm.
/// Deliberately still matched: a visually-hidden-but-focusable element, which
/// is what [`FocusTrapInitialFocus`](crate::components::FocusTrapInitialFocus) is.
pub(crate) const FOCUSABLE_SELECTOR: &str = concat!(
    ":is(a[href], button:not([disabled]), textarea:not([disabled]), ",
    "input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex=\"-1\"]), ",
    "summary, iframe, audio[controls], video[controls], ",
    "[contenteditable]:not([contenteditable=\"false\"]))",
    ":not([hidden], [inert], [aria-hidden=\"true\"], [tabindex=\"-1\"], :disabled, ",
    "[hidden] *, [inert] *, [aria-hidden=\"true\"] *)"
);

#[cfg(test)]
mod tests {
    use super::FOCUSABLE_SELECTOR;

    /// There is no DOM in a unit test, so this pins the one clause the
    /// browser pass measured: the exclusion list, not the `:is(..)` list, has
    /// to carry `tabindex="-1"`, or a button with it stays a Tab stop.
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
}
