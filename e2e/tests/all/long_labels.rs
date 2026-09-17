//! A label with no break opportunity wraps instead of widening the page, in
//! `Tree` rows, `NavLink` and `Menu` items (1.4.10, todo 518), and in
//! `Tabs`, `Timeline`, `Menubar` (todo 543), and a horizontal `DataList`.

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::{Fixture, Viewport, passes::pointer, wait};

/// Every overflowing part as `[what, scrollWidth, clientWidth]`, the page included.
const OVERFLOWS: &str = "(() => {
    const parts = [
        ['page', document.documentElement],
        ...[...document.querySelectorAll('[role=treeitem] > div')].map(e => ['tree row', e]),
        ...[...document.querySelectorAll('#nav a')].map(e => ['nav link', e]),
        ...[...document.querySelectorAll('[data-menu-index]')].map(e => ['menu item', e]),
        ...['#tabs', '#timeline', '#menubar', '#menubar-crowded', '#data-list'].flatMap(root =>
            [...document.querySelectorAll(`${root}, ${root} *`)].map(e => [root + ' ' + e.tagName, e])),
    ];
    const menu = document.querySelector('[role=menu]');
    // A crowded tab strip scrolls inside itself by design; each tab must still fit it.
    const out = parts.filter(([, e]) => e.scrollWidth > e.clientWidth && e.getAttribute('role') !== 'tablist')
        .map(([what, e]) => [what, e.scrollWidth, e.clientWidth]);
    const strip = document.querySelector('#tabs [role=tablist]');
    for (const tab of strip.querySelectorAll('[role=tab]'))
        if (tab.offsetWidth > strip.clientWidth) out.push(['tab', tab.offsetWidth, strip.clientWidth]);
    if (menu.getBoundingClientRect().right > innerWidth) out.push(['menu box', menu.getBoundingClientRect().right, innerWidth]);
    // Todo 481: these keep one line and cut the label, so only their boxes must fit.
    for (const e of document.querySelectorAll('#single-line, #single-line *'))
        if (e.getBoundingClientRect().right > innerWidth + 0.5) out.push(['single line ' + (e.id || e.tagName), e.getBoundingClientRect().right, innerWidth]);
    return out;
})()";

/// Per todo 481 control: `[id, one line, cut, full text in its title or text]`.
/// Button and Chip cut at their edge with no ellipsis: their children stay
/// unwrapped (todos 662, 674).
const SINGLE_LINE: &str = "(() => {
    const long = 'Versandkostenberechnungsgrundlagenverordnungsentwurfsbearbeitungsstelle';
    const oneLine = e => !!e && getComputedStyle(e).whiteSpace === 'nowrap';
    const cut = e => !!e && getComputedStyle(e).textOverflow === 'ellipsis' && e.scrollWidth > e.clientWidth;
    const clipped = e => !!e && getComputedStyle(e).overflow === 'hidden' && e.scrollWidth > e.clientWidth;
    const r = e => e.getBoundingClientRect();
    const rows = [];
    for (const id of ['button', 'button-full', 'button-row', 'chip', 'chip-row', 'chip-icon']) {
        const root = document.getElementById(id);
        // Todo 636: the icon keeps its room before the text.
        const icon = root.querySelector('svg');
        const whole = !icon || (r(icon).width > 0 && r(icon).left >= r(root).left);
        rows.push([id, oneLine(root), clipped(root), root.textContent === long && whole]);
    }
    // A checkbox chip's own `<label>` clips.
    const filter = document.querySelector('#chip-filter label');
    rows.push(['chip-filter', oneLine(filter), clipped(filter), filter.textContent === long]);
    // A caller's own text span ellipsizes, and the x stays whole beside it.
    const root = document.getElementById('chip-trailing');
    const text = root.querySelector('[data-text]');
    const x = root.querySelector('[data-slot=chip-trailing]');
    rows.push(['chip-trailing', oneLine(text), cut(text), text.textContent === long
        && r(x).left - r(text).right >= 2 && r(x).right <= r(root).right]);
    for (const id of ['segmented', 'segmented-full']) {
        const label = [...document.querySelectorAll(`#${id} label`)].find(l => l.title === long);
        const span = label && label.querySelector('span');
        rows.push([id, oneLine(span), cut(span), !!label]);
    }
    return rows;
})()";

#[test]
fn a_long_label_wraps_instead_of_widening_the_page() {
    block_on(async {
        let fixture = Fixture::open("/long-labels", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        for width in [390, 320] {
            page.execute(SetDeviceMetricsOverrideParams::new(width, 800, 1.0, true))
                .await
                .unwrap();
            pointer::click(page, "[aria-haspopup=menu]").await.unwrap();
            wait::for_visible(page, "[role=menu]").await.unwrap();
            let overflows: Vec<serde_json::Value> = page
                .evaluate(OVERFLOWS)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(overflows.is_empty(), "{width}px: {overflows:?}");
            let single: Vec<(String, bool, bool, bool)> = page
                .evaluate(SINGLE_LINE)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            for (id, one_line, cut, full) in &single {
                assert!(*one_line && *cut && *full, "{width}px {id}: {single:?}");
            }
            pointer::click(page, "[aria-haspopup=menu]").await.unwrap();
            wait::for_js_true(
                page,
                "!document.querySelector('[role=menu]')",
                "the menu closed",
            )
            .await
            .unwrap();
        }
        fixture.console.assert_clean("long labels").unwrap();
        fixture.close().await.unwrap();
    });
}
