//! The a11y sweep over every docs page (todo 760). Not part of the suite:
//! `cargo run -p e2e -- sweep` serves the docs site and runs only this.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Scheme, Viewport};

#[test]
fn every_docs_page() {
    e2e::sweep::run().unwrap();
}

const BURGER: &str = "button[aria-controls=docs-nav]";

async fn eval_bool(page: &chromiumoxide::Page, script: &str) -> bool {
    page.evaluate(script).await.unwrap().into_value().unwrap()
}

async fn wait_for(page: &chromiumoxide::Page, what: &str, script: &str) {
    for _ in 0..100 {
        if eval_bool(page, script).await {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("gave up waiting for {what}");
}

/// Todo 1088: any route change closes the mobile drawer, not only a tap on a nav link.
#[test]
fn a_route_change_closes_the_drawer() {
    block_on(async {
        let fixture = Fixture::open_until("/", Viewport::Mobile, Scheme::Light, "#docs-main *")
            .await
            .unwrap();
        let page = &fixture.page;
        let open =
            format!("document.querySelector('{BURGER}').getAttribute('aria-expanded') === 'true'");
        let closed = format!("!({open})");
        let path = "location.pathname";

        // Through the search.
        page.find_element(BURGER)
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        wait_for(page, "the drawer to open", &open).await;
        keyboard::press_with(
            page,
            keyboard::Key {
                key: "k",
                code: "KeyK",
                vk: 75,
                text: None,
            },
            keyboard::CTRL,
        )
        .await
        .unwrap();
        keyboard::type_text(page, "box").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait_for(page, "the route to change", &format!("{path} !== '/'")).await;
        wait_for(page, "the drawer to close after the search", &closed).await;

        // Through the browser's back button.
        page.find_element(BURGER)
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        wait_for(page, "the drawer to open again", &open).await;
        page.evaluate("history.back()").await.unwrap();
        wait_for(page, "the route to change back", &format!("{path} === '/'")).await;
        wait_for(page, "the drawer to close after back", &closed).await;

        fixture.close().await.unwrap();
    });
}

/// Each label and border of the landing page's gradient surfaces against the gradient's
/// two stops and their midpoint (the lowest ratio), then axe over the page.
const GRADIENT_MEASURE: &str = r#"(() => {
    const canvas = document.createElement('canvas').getContext('2d', { willReadFrequently: true });
    const rgb = css => {
        canvas.clearRect(0, 0, 1, 1);
        canvas.fillStyle = '#000';
        canvas.fillStyle = css;
        canvas.fillRect(0, 0, 1, 1);
        return Array.from(canvas.getImageData(0, 0, 1, 1).data).slice(0, 3);
    };
    const lum = c => {
        const [r, g, b] = c.map(v => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; });
        return 0.2126 * r + 0.7152 * g + 0.0722 * b;
    };
    const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); };
    const stop = (paper, name) => {
        const t = document.createElement('span');
        t.style.color = `var(${name})`;
        paper.append(t);
        const c = getComputedStyle(t).color;
        t.remove();
        return rgb(c);
    };
    const out = [];
    const least = (paper, colour) => {
        const from = stop(paper, '--lsx-gradient-from');
        const to = stop(paper, '--lsx-gradient-to');
        const mid = from.map((v, i) => Math.round((v + to[i]) / 2));
        return Math.min(ratio(colour, from), ratio(colour, mid), ratio(colour, to)).toFixed(2);
    };
    const measure = (name, el, paper, prop) => {
        const value = getComputedStyle(el)[prop];
        out.push(`${name} ${prop}=${value} vs gradient: min ${least(paper, rgb(value))}:1`);
    };
    const title = document.querySelector('#closing-title');
    const closing = document.querySelector('section[aria-labelledby=closing-title] > div > div');
    measure('closing h2', title, closing, 'color');
    const links = closing.querySelectorAll('a');
    measure('get-started fill', links[0], closing, 'backgroundColor');
    const fill = getComputedStyle(links[0]);
    out.push(`get-started label ${fill.color} vs its fill: ${ratio(rgb(fill.color), rgb(fill.backgroundColor)).toFixed(2)}:1`);
    measure('browse text', links[1], closing, 'color');
    measure('browse border', links[1], closing, 'borderTopColor');
    document.querySelectorAll('section[aria-labelledby=stats-title] li').forEach((li, i) => {
        const spans = li.querySelectorAll('span');
        measure('stat ' + i + ' value', spans[0], li, 'color');
        measure('stat ' + i + ' title', spans[1], li, 'color');
    });
    return out;
})()"#;

#[test]
fn the_landing_gradients_are_measured() {
    block_on(async {
        for (scheme, viewport) in [
            (Scheme::Light, Viewport::Desktop),
            (Scheme::Dark, Viewport::Desktop),
            (Scheme::Light, Viewport::Mobile),
            (Scheme::Dark, Viewport::Mobile),
        ] {
            let fixture = Fixture::open_until("/", viewport, scheme, "#docs-main *")
                .await
                .unwrap();
            let lines: Vec<String> = fixture
                .page
                .evaluate(GRADIENT_MEASURE)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            println!("== {} {}", scheme.name(), viewport.name());
            for line in &lines {
                println!("{line}");
                let least: f64 = line
                    .split("min ")
                    .nth(1)
                    .and_then(|rest| rest.trim_end_matches(":1").parse().ok())
                    .unwrap_or(f64::MAX);
                let floor = if line.contains("border") || line.contains("fill") {
                    3.0
                } else {
                    4.5
                };
                assert!(least >= floor, "under {floor}:1: {line}");
            }
            let run = e2e::passes::contrast::run_full(&fixture.page, "#docs-main")
                .await
                .unwrap();
            for violation in &run.violations {
                for node in &violation.nodes {
                    println!("axe violation {}: {}", violation.id, node.target);
                }
            }
            println!(
                "axe: {} violations, {} incomplete",
                run.violations.len(),
                run.incomplete.len()
            );
            fixture.close().await.unwrap();
        }
    });
}

/// The docs demos follow their own rule: the trigger's blur closes the list.
#[test]
fn a_combobox_demo_closes_on_an_outside_click() {
    block_on(async {
        for mode in ["Select", "Suggestions"] {
            let viewport = Viewport::Desktop;
            let fixture =
                Fixture::open_until("/form/combobox", viewport, Scheme::Light, "#docs-main *")
                    .await
                    .unwrap();
            let page = &fixture.page;
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            let js = format!(
                "(() => {{ const q = [...document.querySelectorAll('[role=radio],button,label')]; \
                 const m = q.find(e => e.textContent.trim() === '{mode}'); if (m) m.click(); return !!m; }})()"
            );
            eval_bool(page, &js).await;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            e2e::passes::pointer::click(page, "[role=combobox]")
                .await
                .unwrap();
            if mode == "Suggestions" {
                keyboard::type_text(page, "a").await.unwrap();
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let before = eval_bool(page, "!!document.querySelector('[role=listbox]')").await;
            e2e::passes::pointer::click(page, "#docs-main h1")
                .await
                .unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let after = eval_bool(page, "!!document.querySelector('[role=listbox]')").await;
            let expanded: String = page
                .evaluate("document.querySelector('[role=combobox]').getAttribute('aria-expanded')")
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(before, "{mode}: the list never opened");
            assert!(!after, "{mode}: the list stayed open after a click outside");
            assert_eq!(expanded, "false", "{mode}");
            let _ = fixture.close().await;
        }
    });
}
