//! The a11y sweep over every docs page (todo 760). Not part of the suite:
//! `cargo run -p e2e -- sweep` serves the docs site and runs only this. The docs
//! behaviour tests alone: `cargo run -p e2e -- sweep a_route_change the_landing a_combobox`.

use e2e::browser::block_on;
use e2e::passes::contrast::COLOUR_JS;
use e2e::passes::{contrast, keyboard};
use e2e::{Fixture, Scheme, Viewport, wait};

#[test]
fn every_docs_page() {
    e2e::sweep::run().unwrap();
}

const BURGER: &str = "button[aria-controls=docs-nav]";

/// Any console message the page reported fails, at `close`.
async fn close(fixture: Fixture) {
    let messages = fixture.console.drain();
    fixture.close().await.unwrap();
    assert!(
        messages.is_empty(),
        "the page reported:\n  {}",
        messages.join("\n  ")
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
        wait::for_js_true(page, &open, "the drawer to open")
            .await
            .unwrap();
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
        wait::for_js_true(page, &format!("{path} !== '/'"), "the route to change")
            .await
            .unwrap();
        wait::for_js_true(page, &closed, "the drawer to close after the search")
            .await
            .unwrap();
        // Todo 1693: the h1 takes focus once `main` is no longer inert.
        wait::for_js_true(page, ON_H1, "the h1 to take focus after the search")
            .await
            .unwrap();

        // Through the browser's back button.
        page.find_element(BURGER)
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        wait::for_js_true(page, &open, "the drawer to open again")
            .await
            .unwrap();
        page.evaluate("history.back()").await.unwrap();
        wait::for_js_true(page, &format!("{path} === '/'"), "the route to change back")
            .await
            .unwrap();
        wait::for_js_true(page, &closed, "the drawer to close after back")
            .await
            .unwrap();
        wait::for_js_true(page, ON_H1, "the h1 to take focus after back")
            .await
            .unwrap();

        close(fixture).await;
    });
}

const ON_H1: &str = "document.activeElement === document.querySelector('#docs-main h1')";

/// Todo 1693: the open drawer leaves `main` inert, Escape closes it onto the burger, and the
/// skip link moves focus to `main`.
#[test]
fn the_drawer_and_the_skip_link_move_focus() {
    block_on(async {
        let fixture = Fixture::open_until(
            "/about/styling",
            Viewport::Mobile,
            Scheme::Light,
            "#docs-main h1",
        )
        .await
        .unwrap();
        let page = &fixture.page;

        page.find_element(BURGER)
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#docs-main').closest('[inert]') !== null",
            "main to go inert under the drawer",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.activeElement === document.querySelector('{BURGER}') \
                 && document.querySelector('#docs-main').closest('[inert]') === null"
            ),
            "Escape to close the drawer onto the burger",
        )
        .await
        .unwrap();

        page.evaluate("document.querySelector('a[href=\"#docs-main\"]').focus()")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement === document.querySelector('#docs-main')",
            "the skip link to focus main",
        )
        .await
        .unwrap();

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
        wait::for_js_true(
            page,
            "document.querySelector('#docs-main h1').textContent === 'Page not found'",
            "the not-found heading",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.title === 'Page not found - Libero'",
            "the not-found tab title",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "!!document.querySelector('#docs-main a[href=\"/\"]')",
            "a link home",
        )
        .await
        .unwrap();
        close(fixture).await;
    });
}

/// Clicks the docs tab named `label`.
fn doc_tab_js(label: &str) -> String {
    format!(
        "[...document.querySelectorAll('#docs-main [role=tab]')].find(t => t.textContent.includes('{label}')).click()"
    )
}

/// Todo 2182: a visited doc tab's panel stays mounted, hidden and inert; a media page's Usage
/// panel unmounts, so its player stops.
#[test]
fn a_doc_tab_switch_hides_the_old_panel() {
    block_on(async {
        let fixture = Fixture::open_until(
            "/form/switch",
            Viewport::Desktop,
            Scheme::Light,
            "#docs-main [role=tabpanel] > div > *",
        )
        .await
        .unwrap();
        let page = &fixture.page;
        page.evaluate(doc_tab_js("Properties")).await.unwrap();
        let kept = "(() => { const kids = [...document.querySelector('#docs-main [role=tabpanel]').children]; \
            const off = kids.filter(k => k.hidden); \
            return off.some(k => k.childElementCount > 0) && off.every(k => k.inert); })()";
        wait::for_js_true(page, kept, "the Usage panel kept, hidden and inert")
            .await
            .unwrap();
        close(fixture).await;

        let fixture = Fixture::open_until(
            "/data-display/video",
            Viewport::Desktop,
            Scheme::Light,
            "#docs-main video",
        )
        .await
        .unwrap();
        let page = &fixture.page;
        page.evaluate(doc_tab_js("Properties")).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('#docs-main video')",
            "the Video page's players unmounted",
        )
        .await
        .unwrap();
        close(fixture).await;
    });
}

/// Todo 2437: the Marquee preview is as wide as its box, so the box does not scroll
/// sideways and the pause toggle stays in view, left to right and right to left.
#[test]
fn the_marquee_preview_fits_its_box() {
    block_on(async {
        let fixture = Fixture::open_until(
            "/data-display/marquee",
            Viewport::Desktop,
            Scheme::Light,
            "#docs-main [data-slot='pause']",
        )
        .await
        .unwrap();
        let page = &fixture.page;
        let fits = "(() => { const pause = document.querySelector('#docs-main [data-slot=pause]'); \
            let box = pause.parentElement; \
            while (box && getComputedStyle(box).overflowX !== 'auto') box = box.parentElement; \
            const p = pause.getBoundingClientRect(), b = box.getBoundingClientRect(); \
            return box.scrollWidth <= box.clientWidth && p.left >= b.left && p.right <= b.right; })()";
        wait::for_js_true(page, fits, "the preview to fit its box")
            .await
            .unwrap();
        page.evaluate("document.documentElement.dir = 'rtl'")
            .await
            .unwrap();
        wait::for_js_true(page, fits, "the preview to fit its box right to left")
            .await
            .unwrap();
        close(fixture).await;
    });
}

/// Each label and border of the landing page's gradient cards against the gradient's two
/// stops and their midpoint (the lowest ratio), then axe over the page. The stops are
/// translucent tints, so each is read over the first opaque background behind the card.
/// The body of an arrow function, after `COLOUR_JS`.
const GRADIENT_MEASURE: &str = r#"
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
    const ratio = CONTRAST;
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
"#;

/// Shortfalls filed and not fixed yet: a line's name, the lowest ratio it may read, and why.
/// A drop below that ratio still fails.
const KNOWN_SHORT: &[(&str, f64, &str)] = &[];

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
                .evaluate(format!("(() => {{ {COLOUR_JS} {GRADIENT_MEASURE} }})()"))
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

/// Main-thread milliseconds of the CDP counters since `before`.
async fn spent(
    page: &chromiumoxide::Page,
    before: &std::collections::BTreeMap<String, f64>,
) -> (std::collections::BTreeMap<String, f64>, [f64; 3]) {
    use chromiumoxide::cdp::browser_protocol::performance::GetMetricsParams;
    let now: std::collections::BTreeMap<String, f64> = page
        .execute(GetMetricsParams::default())
        .await
        .unwrap()
        .result
        .metrics
        .iter()
        .map(|m| (m.name.clone(), m.value))
        .collect();
    let ms = |key: &str| (now.get(key).unwrap_or(&0.0) - before.get(key).unwrap_or(&0.0)) * 1e3;
    let spent = [
        ms("TaskDuration"),
        ms("ScriptDuration"),
        ms("LayoutDuration") + ms("RecalcStyleDuration"),
    ];
    (now, spent)
}

/// A client-side navigation, as the router's own back button would do it, then until 60 ms
/// pass with under 2 ms of main-thread work.
async fn go(page: &chromiumoxide::Page, path: &str, ready: &str) {
    page.evaluate(format!(
        "history.pushState(null, '', {path:?}); dispatchEvent(new PopStateEvent('popstate'))"
    ))
    .await
    .unwrap();
    wait::for_js_true(page, ready, path).await.unwrap();
    quiet(page).await;
}

/// Until 60 ms pass with under 2 ms of main-thread work.
async fn quiet(page: &chromiumoxide::Page) {
    let (mut last, _) = spent(page, &Default::default()).await;
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(60)).await;
        let (now, [task, ..]) = spent(page, &last).await;
        if task < 2.0 {
            return;
        }
        last = now;
    }
}

/// The docs shell's own interactions (todo 2070): the header's scheme toggle, a theme-set
/// pick from its menu, and a sidebar link between two pages. Medians of 16, a warm-up first.
/// `E2E_RELEASE=1 cargo run -p e2e -- sweep docs_interactions --ignored`; `PERF_CASE` picks.
#[test]
#[ignore = "docs interaction report, run on request"]
fn docs_interactions() {
    const TOOLS: &str = "[aria-label='Site tools']";
    const SCHEME: &str = "(document.documentElement.getAttribute('data-lsx-theme') ?? 'light')";
    const PAGE_BG: &str = "getComputedStyle(document.body).backgroundColor";
    block_on(async {
        // A release `dx` still pre-compresses for a while after it reports ready.
        for _ in 0..3 {
            if e2e::sweep::discover().await.is_ok() {
                break;
            }
        }
        let mut opened = Err(anyhow::anyhow!("not tried"));
        for _ in 0..3 {
            opened = Fixture::open_until(
                "/buttons/button",
                Viewport::Desktop,
                Scheme::Light,
                "#docs-main h1",
            )
            .await;
            if opened.is_ok() {
                break;
            }
        }
        let fixture = opened.unwrap();
        let page = &fixture.page;
        let front = e2e::frames::bring_to_front(page).await.unwrap();
        page.execute(chromiumoxide::cdp::browser_protocol::performance::EnableParams::default())
            .await
            .unwrap();
        // `PERF_PROFILE=1`: the 30 functions most sampled per action, as `docs_mount`.
        let profiling = std::env::var_os("PERF_PROFILE").is_some();
        if profiling {
            use chromiumoxide::cdp::js_protocol::profiler;
            page.execute(profiler::EnableParams::default())
                .await
                .unwrap();
            page.execute(profiler::SetSamplingIntervalParams::new(100))
                .await
                .unwrap();
        }
        let only = std::env::var("PERF_CASE").ok();
        let mut rows: Vec<(&str, Vec<[f64; 3]>)> = Vec::new();
        let reps = 17;
        for name in ["scheme toggle", "theme-set pick", "sidebar link"] {
            if only
                .as_deref()
                .is_some_and(|o| !o.split(',').any(|o| name.contains(o)))
            {
                continue;
            }
            let mut hot = std::collections::BTreeMap::<String, i64>::new();
            let mut taken = Vec::new();
            for rep in 0..reps {
                quiet(page).await;
                if name == "theme-set pick" {
                    // Opening the menu is not the pick: it goes before the counters.
                    e2e::passes::pointer::click(page, &format!("{TOOLS} [data-slot=picker]"))
                        .await
                        .unwrap();
                    wait::for_js_true(
                        page,
                        "!!document.querySelector('[role=menu]')",
                        "the theme menu",
                    )
                    .await
                    .unwrap();
                    quiet(page).await;
                }
                if profiling {
                    page.execute(chromiumoxide::cdp::js_protocol::profiler::StartParams::default())
                        .await
                        .unwrap();
                }
                let (before, _) = spent(page, &Default::default()).await;
                match name {
                    "scheme toggle" => {
                        let was: String =
                            page.evaluate(SCHEME).await.unwrap().into_value().unwrap();
                        e2e::passes::pointer::click(page, &format!("{TOOLS} [data-slot=toggle]"))
                            .await
                            .unwrap();
                        wait::for_js_true(
                            page,
                            &format!("{SCHEME} !== {was:?}"),
                            "the scheme to flip",
                        )
                        .await
                        .unwrap();
                    }
                    "theme-set pick" => {
                        let set = if rep % 2 == 0 { "Ayu" } else { "Libero" };
                        let bg: String =
                            page.evaluate(PAGE_BG).await.unwrap().into_value().unwrap();
                        page.evaluate(format!(
                            "(e => {{ e.setAttribute('data-e2e', 'set'); e.scrollIntoView({{ block: 'nearest' }}); }})\
                             ([...document.querySelectorAll('[role=menuitemradio]')].find((e) => e.textContent.trim() === {set:?}))"
                        ))
                        .await
                        .unwrap();
                        e2e::passes::pointer::click(page, "[data-e2e=set]")
                            .await
                            .unwrap();
                        wait::for_js_true(
                            page,
                            &format!("{PAGE_BG} !== {bg:?}"),
                            "the page colour to change",
                        )
                        .await
                        .unwrap();
                    }
                    _ => {
                        let to = if rep % 2 == 0 {
                            "/buttons/action-icon"
                        } else {
                            "/buttons/button"
                        };
                        e2e::passes::pointer::click(page, &format!("#docs-nav a[href='{to}']"))
                            .await
                            .unwrap();
                        wait::for_js_true(page, &format!("location.pathname === {to:?} && !!document.querySelector('#docs-main h1')"), to).await.unwrap();
                    }
                }
                quiet(page).await;
                let (_, times) = spent(page, &before).await;
                if profiling {
                    let profile = page
                        .execute(chromiumoxide::cdp::js_protocol::profiler::StopParams::default())
                        .await
                        .unwrap();
                    for node in &profile.result.profile.nodes {
                        *hot.entry(node.call_frame.function_name.clone())
                            .or_default() += node.hit_count.unwrap_or(0);
                    }
                }
                if rep > 0 {
                    taken.push(times);
                }
            }
            let mut hot: Vec<_> = hot.into_iter().filter(|(_, n)| *n > 0).collect();
            hot.sort_by_key(|a| std::cmp::Reverse(a.1));
            for (function, samples) in hot.iter().take(30) {
                println!("profile | {name} | {samples:>5} | {function}");
            }
            rows.push((name, taken));
        }
        println!("| Action | task | script | layout | rest |\n|---|---|---|---|---|");
        for (name, reps) in &rows {
            let median = |i: usize| {
                let mut v: Vec<f64> = reps.iter().map(|r| r[i]).collect();
                v.sort_by(f64::total_cmp);
                v[v.len() / 2]
            };
            let [task, script, layout] = [median(0), median(1), median(2)];
            println!(
                "| {name} | {task:.1} | {script:.1} | {layout:.1} | {:.1} |",
                task - script - layout
            );
        }
        // A fixed loop: a run whose host was busy reads slow here too, so A/B runs compare.
        let mut control: Vec<f64> = Vec::new();
        for _ in 0..9 {
            control.push(
                page.evaluate(
                    "(() => { const t = performance.now(); let x = 0; \
                     for (let i = 0; i < 2e6; i++) x += Math.sqrt(i); \
                     return performance.now() - t + (x < 0 ? 1 : 0); })()",
                )
                .await
                .unwrap()
                .into_value()
                .unwrap(),
            );
        }
        control.sort_by(f64::total_cmp);
        println!("| cpu control | {:.1} | | | |", control[4]);
        front.release().await.unwrap();
        close(fixture).await;
    });
}

/// Each docs page's client-side mount (todo 2031): task, script and style plus layout ms,
/// medians of three navigations from the not-found page, and the elements in `#docs-main`;
/// slowest first. `E2E_RELEASE=1 cargo run -p e2e -- sweep docs_mount --ignored`;
/// `PERF_CASE` picks pages by part of the path.
#[test]
#[ignore = "docs mount report, run on request"]
fn docs_mount() {
    use chromiumoxide::cdp::js_protocol::profiler;
    const BASE: &str = "/no-such-page";
    const AT_BASE: &str =
        "document.querySelector('#docs-main h1')?.textContent === 'Page not found'";
    block_on(async {
        let only = std::env::var("PERF_CASE").ok();
        // A release `dx` still pre-compresses the icon sets for a while after it reports ready.
        let mut found = e2e::sweep::discover().await;
        for _ in 0..3 {
            if found.is_ok() {
                break;
            }
            found = e2e::sweep::discover().await;
        }
        let pages: Vec<String> = found
            .unwrap()
            .into_iter()
            .filter(|p| p != "/" && p != BASE)
            .filter(|p| {
                only.as_deref()
                    .is_none_or(|o| o.split(',').any(|o| p.contains(o)))
            })
            .collect();
        let fixture = Fixture::open_until(BASE, Viewport::Desktop, Scheme::Light, "#docs-main h1")
            .await
            .unwrap();
        let page = &fixture.page;
        let front = e2e::frames::bring_to_front(page).await.unwrap();
        page.execute(chromiumoxide::cdp::browser_protocol::performance::EnableParams::default())
            .await
            .unwrap();
        // `PERF_PROFILE=1`: the 30 functions most sampled (100 us) per page, by self time.
        let profiling = std::env::var_os("PERF_PROFILE").is_some();
        if profiling {
            page.evaluate(
                "(() => { const seen = window.__styleReads = {}; const get = CSSStyleDeclaration.prototype.getPropertyValue; \
                 CSSStyleDeclaration.prototype.getPropertyValue = function (name) { seen[name] = (seen[name] || 0) + 1; return get.call(this, name); }; })()",
            )
            .await
            .unwrap();
            page.execute(profiler::EnableParams::default())
                .await
                .unwrap();
            page.execute(profiler::SetSamplingIntervalParams::new(100))
                .await
                .unwrap();
        }
        let mut rows = Vec::new();
        for path in &pages {
            let ready = format!(
                "location.pathname === {path:?} && !!document.querySelector('#docs-main h1') && !({AT_BASE})"
            );
            let mut reps: Vec<[f64; 4]> = Vec::new();
            let mut hot = std::collections::BTreeMap::<String, i64>::new();
            // One unmeasured visit first: its wasm paths and styles warm up.
            for rep in 0..4 {
                go(page, BASE, AT_BASE).await;
                if profiling {
                    page.execute(profiler::StartParams::default())
                        .await
                        .unwrap();
                }
                let (before, _) = spent(page, &Default::default()).await;
                go(page, path, &ready).await;
                let (_, [task, script, layout]) = spent(page, &before).await;
                if profiling {
                    let profile = page.execute(profiler::StopParams::default()).await.unwrap();
                    for node in &profile.result.profile.nodes {
                        *hot.entry(node.call_frame.function_name.clone())
                            .or_default() += node.hit_count.unwrap_or(0);
                    }
                }
                let nodes: f64 = page
                    .evaluate("document.querySelectorAll('#docs-main *').length")
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                if rep > 0 {
                    reps.push([task, script, layout, nodes]);
                }
            }
            let median = |i: usize| {
                let mut v: Vec<f64> = reps.iter().map(|r| r[i]).collect();
                v.sort_by(f64::total_cmp);
                v[v.len() / 2]
            };
            rows.push((path.clone(), [median(0), median(1), median(2), median(3)]));
            let mut hot: Vec<_> = hot.into_iter().filter(|(_, n)| *n > 0).collect();
            hot.sort_by_key(|a| std::cmp::Reverse(a.1));
            for (name, samples) in hot.iter().take(30) {
                println!("profile | {path} | {samples:>5} | {name}");
            }
            if profiling {
                let reads: String = page
                    .evaluate("(() => { const r = JSON.stringify(window.__styleReads); for (const k in window.__styleReads) delete window.__styleReads[k]; return r; })()")
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                println!("style reads | {path} | {reads}");
            }
        }
        rows.sort_by(|a, b| b.1[0].total_cmp(&a.1[0]));
        println!("| Page | task | script | layout | rest | elements |\n|---|---|---|---|---|---|");
        for (path, [task, script, layout, nodes]) in &rows {
            println!(
                "| {path} | {task:.1} | {script:.1} | {layout:.1} | {:.1} | {nodes} |",
                task - script - layout
            );
        }
        front.release().await.unwrap();
        close(fixture).await;
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
