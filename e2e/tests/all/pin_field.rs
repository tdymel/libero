//! `PinField`: its label names the cells' group, so a click on it focuses the
//! first cell (todo 483).

const CELL: &str = "[role=group] input";

#[test]
fn a_click_on_the_label_focuses_the_first_cell() {
    crate::select::label_click_focuses("/pin-field", CELL);
}
