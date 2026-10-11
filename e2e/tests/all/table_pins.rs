//! `Table` pinned columns: the holds at each edge, the focus scroll past a pin, and the menu's Pin.

use super::table::{AREA, menu_entry};
use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_text};
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

/// 1156-2d: scrolled to its far end, the checkbox and Name columns hold at the
/// start edge and Supplier at the end, over the scrolled cells, in LTR and RTL.
#[test]
fn pinned_columns_hold_at_their_edges_in_both_directions() {
    block_on(async {
        for (route, rtl) in [("/table/pinned", false), ("/table/pinned-rtl", true)] {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            let sign = if rtl { -1 } else { 1 };
            // Retried: the area may not overflow yet on the first poll.
            wait::for_js_true(
                page,
                &format!(
                    "(() => {{ const r = document.querySelector({AREA:?}); \
                     if (!r) return false; r.scrollLeft = {sign} * r.scrollWidth; \
                     return Math.abs(r.scrollLeft) > 300; }})()"
                ),
                &format!("{route} to scroll"),
            )
            .await
            .unwrap();
            // Edges as [start, end] in reading order: left/right flip in RTL.
            let state: String = page
                .evaluate(format!(
                    "(() => {{ const rtl = {rtl}; \
                     const edges = s => {{ const b = document.querySelector(s).getBoundingClientRect(); \
                       return rtl ? [b.right, b.left] : [b.left, b.right]; }}; \
                     const area = document.querySelector({AREA:?}); \
                     const a = area.getBoundingClientRect(); \
                     const start = rtl ? a.right : a.left; \
                     const end = rtl ? a.left + a.width - area.clientWidth : a.left + area.clientWidth; \
                     const select = edges('thead th[data-select]'); \
                     const name = edges('th[aria-label=Name]'); \
                     const supplier = edges('th[aria-label=Supplier]'); \
                     const origin = edges('th[aria-label=Origin]'); \
                     const near = (x, y) => Math.abs(x - y) <= 1; \
                     const cell = document.querySelector('tbody tr:nth-child(2) td[data-pin=start]'); \
                     const c = cell.getBoundingClientRect(); \
                     const hit = document.elementFromPoint(c.left + c.width / 2, c.top + c.height / 2); \
                     const bg = getComputedStyle(cell).backgroundColor; \
                     return [near(select[0], start), near(name[0], select[1]), \
                       near(supplier[1], end), rtl ? origin[0] > name[0] : origin[0] < name[0], \
                       cell.contains(hit), bg === getComputedStyle(cell.parentElement).backgroundColor, \
                       (() => {{ const x = document.createElement('canvas').getContext('2d'); \
                         x.fillStyle = bg; x.fillRect(0, 0, 1, 1); return x.getImageData(0, 0, 1, 1).data[3] === 255; }})(), \
                       getComputedStyle(document.querySelector('th[aria-label=Name]')).zIndex === '3' \
                       && getComputedStyle(document.querySelector('th[aria-label=Origin]')).zIndex === '2' \
                       && getComputedStyle(cell).zIndex === '1'].join('|') + ' ' + JSON.stringify([start, end, select, name, supplier]); }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            // The edges follow, for a failure message.
            assert!(
                state.starts_with("true|true|true|true|true|true|true|true "),
                "{route}: checkbox, Name at the start edge, Supplier at the end, Origin \
                 scrolled under them; the striped pinned cell on top and opaque; a pinned \
                 header (a corner under max_height) over the others: {state}"
            );
            fixture.console.assert_clean("pinned columns").unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Todo 1791, WCAG 2.4.11: scrolled to the middle, Tab across the header's
/// buttons never leaves one under the pinned Name or Supplier column.
#[test]
fn a_focused_header_button_stays_clear_of_the_pinned_columns() {
    block_on(async {
        for (route, rtl) in [("/table/pinned", false), ("/table/pinned-rtl", true)] {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            let sign = if rtl { -1 } else { 1 };
            wait::for_js_true(
                page,
                &format!(
                    "(() => {{ const r = document.querySelector({AREA:?}); \
                     if (!r || r.scrollWidth - r.clientWidth < 300) return false; \
                     r.scrollLeft = {sign} * (r.scrollWidth - r.clientWidth) / 2; \
                     const first = document.querySelector('thead th[data-select] input'); \
                     first.focus({{preventScroll: true}}); \
                     return document.activeElement === first; }})()"
                ),
                &format!("{route} to scroll to the middle"),
            )
            .await
            .unwrap();
            // Done once focus leaves the header; else clear of both pinned columns.
            let check = format!(
                "(() => {{ const f = document.activeElement, th = f && f.closest('thead th'); \
                 if (!th) return 'out'; if (th.dataset.pin) return 'pinned'; \
                 const e = s => document.querySelector(s).getBoundingClientRect(); \
                 const b = f.getBoundingClientRect(), name = e('th[aria-label=Name]'), \
                   supplier = e('th[aria-label=Supplier]'); \
                 const [lo, hi] = {rtl} ? [supplier.right, name.left] : [name.right, supplier.left]; \
                 return b.left >= lo - 1 && b.right <= hi + 1 ? 'clear' \
                   : 'under ' + JSON.stringify([th.getAttribute('aria-label'), b.left, b.right, lo, hi, \
                     getComputedStyle(document.querySelector({AREA:?})).scrollPaddingInlineStart]); }})()"
            );
            let mut clear = 0;
            for step in 0..20 {
                keyboard::press(page, keyboard::TAB).await.unwrap();
                // The focus scroll may land a frame after the focus; a timeout reads 'under' below.
                let _ = wait::for_js_true(
                    page,
                    &format!("!{check}.startsWith('under')"),
                    "the focus scroll",
                )
                .await;
                let state: String = page
                    .evaluate(check.clone())
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                match state.as_str() {
                    "out" => break,
                    "clear" => clear += 1,
                    "pinned" => {}
                    under => panic!("{route}, Tab {step}: a header button {under}"),
                }
            }
            assert!(
                clear >= 6,
                "{route}: only {clear} scrolled header buttons reached"
            );
            fixture.console.assert_clean("pinned focus scroll").unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Todo 2047: an innermost pin with no declared width pads the focus scroll by its
/// measured width, so a header button focused under it scrolls clear of it.
#[test]
fn a_focus_scroll_clears_an_unsized_pinned_column() {
    block_on(async {
        let fixture = Fixture::open("/table/pinned-unsized", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        const ORIGIN: &str = "th[aria-label=Origin] button";
        // Scrolled to the far end, Origin sits under the pinned Name; focusing it scrolls back.
        // The padding lands with the first measure, after the first paint.
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const r = document.querySelector({AREA:?}); \
                 if (!r || r.scrollWidth - r.clientWidth < 300) return false; \
                 if (getComputedStyle(r).scrollPaddingInlineStart === 'auto') return false; \
                 r.scrollLeft = r.scrollWidth; \
                 const origin = document.querySelector({ORIGIN:?}).getBoundingClientRect(); \
                 return origin.left < document.querySelector('th[aria-label=Name]').getBoundingClientRect().right - 1; }})()"
            ),
            "the measured pin padding, and Origin scrolled under Name",
        )
        .await
        .unwrap();
        page.evaluate(format!("document.querySelector({ORIGIN:?}).focus()"))
            .await
            .unwrap();
        let check = format!(
            "(() => {{ const b = document.querySelector({ORIGIN:?}).getBoundingClientRect(); \
             const name = document.querySelector('th[aria-label=Name]').getBoundingClientRect(); \
             const area = document.querySelector({AREA:?}); \
             return b.left >= name.right - 1 ? 'clear' : 'under ' + JSON.stringify([b.left, name.left, name.right, \
               area.getBoundingClientRect().left, area.scrollLeft, \
               document.activeElement === document.querySelector({ORIGIN:?}), \
               getComputedStyle(area).scrollPaddingInlineStart]); }})()"
        );
        let cleared =
            wait::for_js_true(page, &format!("{check} === 'clear'"), "the focus scroll").await;
        if cleared.is_err() {
            let state: String = page.evaluate(check).await.unwrap().into_value().unwrap();
            panic!("the focused Origin button stays under the pinned Name: {state}");
        }
        fixture
            .console
            .assert_clean("an unsized pinned column")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// 1156-5a: windowed and scrolled both ways, the header sticks and the pinned
/// columns hold at their edges, body cells in line with their headers.
#[test]
fn a_windowed_table_keeps_its_header_and_pins_while_scrolled() {
    block_on(async {
        let fixture = Fixture::open("/table/windowed-pinned", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const r = document.querySelector({AREA:?}); \
                 if (!r) return false; r.scrollLeft = r.scrollWidth; r.scrollTop = 200000; \
                 return r.scrollLeft > 300 && !!document.querySelector('tr[aria-rowindex=\"5005\"]') \
                   && !document.querySelector('tr[aria-rowindex=\"2\"]'); }})()"
            ),
            "the window to move down and right",
        )
        .await
        .unwrap();
        let state: String = page
            .evaluate(format!(
                "(() => {{ const box = s => document.querySelector(s).getBoundingClientRect(); \
                 const area = document.querySelector({AREA:?}); \
                 const a = area.getBoundingClientRect(); \
                 const end = a.left + area.clientWidth; \
                 const row = 'tr[aria-rowindex=\"5005\"] '; \
                 const select = box('thead th[data-select]'), name = box('thead th[data-pin=start]'); \
                 const origin = box('thead th:nth-child(4)'), supplier = box('thead th[data-pin=end]'); \
                 const cell = box(row + 'td[data-pin=start]'), last = box(row + 'td[data-pin=end]'); \
                 const near = (x, y) => Math.abs(x - y) <= 1; \
                 return [near(select.left, a.left), near(name.left, select.right), \
                   near(cell.left, name.left), near(cell.width, name.width), \
                   near(supplier.right, end), near(last.right, end), origin.left < name.left, \
                   near(name.top, a.top)].join('|') + ' ' + JSON.stringify([a, select, name, cell, supplier, last]); }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            state.starts_with("true|true|true|true|true|true|true|true "),
            "checkbox and Name at the start, Supplier at the end, body cells under their \
             headers, Origin scrolled under, the header at the top: {state}"
        );
        fixture
            .console
            .assert_clean("a windowed pinned table")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Whether the pinned fixture's pins hold in a scroll area `width` px wide: wide enough for
/// Name, Origin, Supplier and the checkbox column, or too narrow for any (todo 3256).
fn pins_hold(width: f64) -> Option<bool> {
    match width {
        w if w * 2.0 / 3.0 >= 760.0 => Some(true),
        w if w <= 420.0 => Some(false),
        _ => None,
    }
}

/// 1156-2d: "Pin to start" in a column's menu pins it after the pinned ones.
async fn a_menu_pin_moves_the_column<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let holds = pins_hold(d.rect(AREA).await?.width);
    d.click("button[aria-label=\"Origin column options\"]")
        .await?;
    // After Filter and the moves.
    let pin = menu_entry(d, "Pin to start").await?;
    d.click(&pin).await?;
    eventually_text(d, "#pinned", "Name,Origin|Supplier", "Pin to start").await?;
    if holds == Some(true) {
        eventually(d, "Origin to pin past Name", async |d| {
            Ok(d.attr("th[aria-label=Origin]", "data-pin")
                .await?
                .as_deref()
                == Some("start")
                && d.rect("th[aria-label=Origin]").await?.x
                    > d.rect("th[aria-label=Name]").await?.x)
        })
        .await?;
    }

    // Moved to the other edge, then unpinned: no inset of its old edge stays (3132, 3149:
    // the body cells too, Origin's before Supplier's).
    for (label, pin, insets, cell) in [
        (
            "Pin to end",
            Some("end"),
            &["inset-inline-start:auto;"][..],
            Some("tbody td[data-pin=end]"),
        ),
        (
            "Unpin",
            None,
            &["inset-inline-start:auto;", "inset-inline-end:auto;"][..],
            None,
        ),
    ] {
        d.click("button[aria-label=\"Origin column options\"]")
            .await?;
        let entry = menu_entry(d, label).await?;
        d.click(&entry).await?;
        // Released pins hold no inset and no side either.
        let pin = pin.filter(|_| holds != Some(false));
        eventually(d, &format!("Origin's header after {label}"), async |d| {
            let style = d.attr("th[aria-label=Origin]", "style").await?;
            let style = style.unwrap_or_default().replace(' ', "");
            let side = d.attr("th[aria-label=Origin]", "data-pin").await?;
            Ok((holds.is_none() || side.as_deref() == pin)
                && insets.iter().all(|inset| style.contains(inset)))
        })
        .await?;
        if let Some(cell) = cell.filter(|_| holds == Some(true)) {
            eventually(d, &format!("Origin's body cell after {label}"), async |d| {
                let style = d.attr(cell, "style").await?;
                let style = style.unwrap_or_default().replace(' ', "");
                Ok(insets.iter().all(|inset| style.contains(inset)))
            })
            .await?;
        }
    }
    Ok(())
}

e2e::scenario!(
    a_column_menu_pins_a_column,
    "/table/pinned",
    a_menu_pin_moves_the_column
);

/// Todo 3256: pinned columns that leave the rest under a third of the area are released
/// (the checkbox column stays held), and pin again when the area grows.
async fn narrow_areas_release_their_pins<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const PINS: &str = "thead th[data-pin]";
    let web = d.platform() == Platform::Web;
    if web {
        eventually(d, "Name and Supplier pinned", async |d| {
            Ok(
                d.attr("th[aria-label=Name]", "data-pin").await?.as_deref() == Some("start")
                    && d.attr("th[aria-label=Supplier]", "data-pin")
                        .await?
                        .as_deref()
                        == Some("end"),
            )
        })
        .await?;
        d.resize_viewport(412.0, 800.0).await?;
    }
    eventually(d, "the pins to be released", async |d| {
        Ok(d.rect(AREA).await?.width <= 420.0 && !d.exists(PINS).await?)
    })
    .await?;
    assert_eq!(
        d.style("thead th[data-select]", "position").await?,
        "sticky"
    );
    // Still reachable: no pinned column covers any header's menu button.
    d.click("button[aria-label=\"Origin column options\"]")
        .await?;
    menu_entry(d, "Pin to start").await?;
    d.press(keyboard::ESCAPE).await?;
    if web {
        d.resize_viewport(1280.0, 800.0).await?;
        eventually(d, "the pins to hold again", async |d| {
            Ok(
                d.attr("th[aria-label=Name]", "data-pin").await?.as_deref() == Some("start")
                    && d.attr("th[aria-label=Supplier]", "data-pin")
                        .await?
                        .as_deref()
                        == Some("end"),
            )
        })
        .await?;
    }
    Ok(())
}

e2e::scenario!(
    narrow_areas_release_their_pins_and_wide_ones_pin_again,
    "/table/pinned",
    narrow_areas_release_their_pins,
    native: skip("1024px is wide enough for the pins"),
    desktop: skip("its window cannot be narrowed")
);
