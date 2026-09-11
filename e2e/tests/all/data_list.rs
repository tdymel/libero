//! `DataList`: the term is written inline, and still takes the list's styling.

use e2e::browser::block_on;
use e2e::{Fixture, Viewport};

/// The `<dt>` in column one and bold, its `<dd>` in column two with no margin.
const LAID_OUT: &str = "(() => { \
    const dt = document.querySelector('#list > dt#term'); \
    const dd = dt && dt.nextElementSibling; \
    if (!dt || !dd || dd.tagName !== 'DD') return 'missing'; \
    const t = getComputedStyle(dt), d = getComputedStyle(dd); \
    return [t.fontWeight, t.gridColumnStart, d.gridColumnStart, d.marginLeft].join(' '); \
})()";

#[test]
fn a_term_and_its_description_sit_side_by_side() {
    block_on(async {
        let fixture = Fixture::open("/data-list", Viewport::Desktop)
            .await
            .unwrap();

        let laid_out: String = fixture
            .page
            .evaluate(LAID_OUT)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(laid_out, "600 1 2 0px");

        fixture
            .console
            .assert_clean("the data list fixture")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
