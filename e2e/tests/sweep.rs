//! The a11y sweep over every docs page (todo 760). Not part of the suite:
//! `cargo run -p e2e -- sweep` serves the docs site and runs only this. The docs
//! behaviour tests alone: `cargo run -p e2e -- sweep a_route_change the_landing a_combobox`.

use e2e::browser::block_on;
use e2e::passes::{contrast, keyboard};
use e2e::{Fixture, Scheme, Viewport, wait};

#[test]
fn every_docs_page() {
    e2e::sweep::run().unwrap();
}

const BURGER: &str = "button[aria-controls=docs-nav]";

async fn wait_for(page: &chromiumoxide::Page, what: &str, script: &str) {
    wait::for_js_true(page, script, what).await.unwrap();
}

/// The docs shell's page `ScrollArea` warns it has no name once the page scrolls (todo
/// filed with 1708); any other console message still fails, at `close`.
async fn close(fixture: Fixture) {
    let (known, other): (Vec<_>, Vec<_>) = fixture
        .console
        .drain()
        .into_iter()
        .partition(|m| m.contains("ScrollArea: a tab stop with no `aria-label`"));
    for message in known {
        eprintln!("console: the shell's unnamed ScrollArea, as filed: {message}");
    }
    fixture.close().await.unwrap();
    assert!(
        other.is_empty(),
        "the page reported:\n  {}",
        other.join("\n  ")
    );
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

        close(fixture).await;
    });
}

/// Todo 1692: an unknown URL lands on a page inside the shell, not a router parse error.
#[test]
fn an_unknown_url_shows_the_not_found_page() {
    block_on(async {
        let fixture = Fixture::open_until(
            "/no-such-page",
            Viewport::Desktop,
            Scheme::Light,
            "#docs-main h1",
        )
        .await
        .unwrap();
        let page = &fixture.page;
        wait_for(
            page,
            "the not-found heading",
            "document.querySelector('#docs-main h1').textContent === 'Page not found'",
        )
        .await;
        wait_for(
            page,
            "the not-found tab title",
            "document.title === 'Page not found - Libero'",
        )
        .await;
        wait_for(
            page,
            "a link home",
            "!!document.querySelector('#docs-main a[href=\"/\"]')",
        )
        .await;
        close(fixture).await;
    });
}

/// Each label and border of the landing page's gradient cards against the gradient's two
/// stops and their midpoint (the lowest ratio), then axe over the page. The stops are
/// translucent tints, so each is read over the first opaque background behind the card.
const GRADIENT_MEASURE: &str = r#"(() => {
    const canvas = document.createElement('canvas').getContext('2d', { willReadFrequently: true });
    const rgb = (css, under = 'transparent') => {
        canvas.clearRect(0, 0, 1, 1);
        canvas.fillStyle = under;
        canvas.fillRect(0, 0, 1, 1);
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
    // The card's own `background-color` paints under its image, so the walk starts there.
    const backdrop = el => {
        for (let e = el; e; e = e.parentElement) {
            const c = getComputedStyle(e).backgroundColor;
            if (rgb(c, 'white').join() === rgb(c, 'black').join()) return c;
        }
        return 'white';
    };
    // The colours in the card's `background-image`, each over its backdrop.
    const stops = paper => {
        const image = getComputedStyle(paper).backgroundImage;
        const under = backdrop(paper);
        return (image.match(/(?:rgba?|color|hsla?|oklch|oklab|lab|lch)\([^()]*\)/g) || [])
            .map(c => rgb(c, under));
    };
    const out = [];
    const measure = (name, el, paper, prop) => {
        const value = getComputedStyle(el)[prop];
        const [from, to] = stops(paper);
        if (!to) {
            out.push(`${name}: no two gradient stops in ${getComputedStyle(paper).backgroundImage}`);
            return;
        }
        const colour = rgb(value);
        const mid = from.map((v, i) => Math.round((v + to[i]) / 2));
        const least = Math.min(ratio(colour, from), ratio(colour, mid), ratio(colour, to)).toFixed(2);
        // WCAG's large text, which needs 3:1: 24px, or 18.66px bold.
        const [px, weight] = [parseFloat(getComputedStyle(el).fontSize), parseInt(getComputedStyle(el).fontWeight)];
        const large = prop === 'color' && (px >= 24 || (px >= 18.66 && weight >= 700)) ? ' (large)' : '';
        out.push(`${name}${large} ${prop}=${value} vs gradient over ${backdrop(paper)}: min ${least}:1`);
    };
    const title = document.querySelector('#closing-title');
    const closing = title.closest('section[aria-labelledby=closing-title] > div > div');
    measure('closing h2', title, closing, 'color');
    const links = closing.querySelectorAll('a');
    measure('get-started fill', links[0], closing, 'backgroundColor');
    const fill = getComputedStyle(links[0]);
    out.push(`get-started label color=${fill.color} vs its background: min ${ratio(rgb(fill.color), rgb(fill.backgroundColor)).toFixed(2)}:1`);
    measure('browse text', links[1], closing, 'color');
    measure('browse border', links[1], closing, 'borderTopColor');
    document.querySelectorAll('section[aria-labelledby=stats-title] li').forEach((li, i) => {
        const spans = li.querySelectorAll('span');
        measure('stat ' + i + ' value', spans[0], li, 'color');
        measure('stat ' + i + ' title', spans[1], li, 'color');
    });
    return out;
})()"#;

/// Shortfalls filed and not fixed yet: a line's name, the lowest ratio it may read, and why.
/// A drop below that ratio still fails.
const KNOWN_SHORT: &[(&str, f64, &str)] = &[(
    "browse text",
    4.01,
    "filed with 1708: the outlined Browse components label on the closing card's tint, \
     4.01:1 light, 4.18:1 dark",
)];

#[test]
fn the_landing_gradients_are_measured() {
    block_on(async {
        // Every scheme and viewport is read before the verdict, so one run shows them all.
        let mut short = Vec::new();
        let mut violations = Vec::new();
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
                    .unwrap_or_else(|| panic!("no `min <ratio>:1` to check in: {line}"));
                let floor = if ["border", "fill", "(large)"]
                    .iter()
                    .any(|k| line.contains(k))
                {
                    3.0
                } else {
                    4.5
                };
                let known = KNOWN_SHORT
                    .iter()
                    .find(|(name, lowest, _)| line.starts_with(name) && least >= *lowest);
                match known {
                    _ if least >= floor => {}
                    Some((_, _, why)) => println!("  known: {why}"),
                    None => short.push(format!(
                        "{} {}: under {floor}:1: {line}",
                        scheme.name(),
                        viewport.name()
                    )),
                }
            }
            let run = contrast::run_full(&fixture.page, "#docs-main")
                .await
                .unwrap();
            for violation in &run.violations {
                for node in &violation.nodes {
                    // The install line's CodeBlock numbers its lines.
                    if contrast::LINE_NUMBERS
                        .iter()
                        .any(|w| w.covers(&violation.id, node))
                    {
                        continue;
                    }
                    violations.push(format!(
                        "{} {}: {}: {}",
                        scheme.name(),
                        viewport.name(),
                        violation.id,
                        node.target
                    ));
                }
            }
            println!("axe: {} incomplete", run.incomplete.len());
            close(fixture).await;
        }
        assert!(
            short.is_empty() && violations.is_empty(),
            "the landing page's cards:\n  {}\naxe:\n  {}",
            short.join("\n  "),
            violations.join("\n  ")
        );
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
            let control = format!(
                "[...document.querySelectorAll('[role=radio],button,label')]\
                 .find(e => e.textContent.trim() === '{mode}')"
            );
            wait::for_js_true(page, &format!("!!{control}"), "the mode switch")
                .await
                .unwrap();
            page.evaluate(format!("{control}.click()")).await.unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "(m => m.getAttribute('aria-checked') === 'true' \
                     || m.getAttribute('aria-pressed') === 'true' \
                     || !!m.querySelector('input:checked') || !!m.control?.checked)({control})"
                ),
                &format!("the {mode} mode to be on"),
            )
            .await
            .unwrap();
            e2e::passes::pointer::click(page, "[role=combobox]")
                .await
                .unwrap();
            if mode == "Suggestions" {
                keyboard::type_text(page, "a").await.unwrap();
            }
            wait::for_js_true(
                page,
                "!!document.querySelector('[role=listbox]')",
                &format!("{mode}: the list to open"),
            )
            .await
            .unwrap();
            e2e::passes::pointer::click(page, "#docs-main h1")
                .await
                .unwrap();
            wait::for_js_true(
                page,
                "!document.querySelector('[role=listbox]') && document.querySelector('[role=combobox]')\
                 .getAttribute('aria-expanded') === 'false'",
                &format!("{mode}: the list to close after a click outside"),
            )
            .await
            .unwrap();
            close(fixture).await;
        }
    });
}
