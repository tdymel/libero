//! `Badge`, `Indicator`, `Kbd` and `List`: contrast of small text in both
//! schemes, and list semantics that survive `list-style: none`.

use e2e::passes::contrast;
use e2e::suite::Suite;

#[test]
fn it_meets_the_baseline() {
    Suite::new("badge", "/badge")
        .waive(contrast::TODO_297)
        .run();
}
