//! The a11y sweep over every docs page, both schemes and viewports (todo 760): `cargo run -p e2e -- sweep`.
//! A report, not a gate: only harness errors fail the run, after the report is written.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use futures::StreamExt;

use crate::passes::{contrast, focus, keyboard, target_size};
use crate::{Fixture, Scheme, Viewport};

/// Rendered by the docs shell once a route's page is in it.
const READY: &str = "#docs-main *";

/// Tab stops walked per page before the ring pass gives up on the rest.
const MAX_STOPS: usize = 300;

/// Pages checked at once; navigation itself is capped by the harness.
const PARALLEL: usize = 3;

const CONFIGS: [(Scheme, Viewport); 4] = [
    (Scheme::Light, Viewport::Desktop),
    (Scheme::Light, Viewport::Mobile),
    (Scheme::Dark, Viewport::Desktop),
    (Scheme::Dark, Viewport::Mobile),
];

/// Every family the report can hold, with what it checks.
const FAMILIES: &[(&str, &str)] = &[
    (
        "text-contrast",
        "axe `color-contrast`: text under 4.5:1, or 3:1 when large (1.4.3)",
    ),
    (
        "text-contrast-undetermined",
        "text neither axe nor the sweep could measure: over an image or a gradient it cannot read, or covered",
    ),
    (
        "boundary-contrast",
        "a field, checkbox, radio, switch or slider whose border, fill and spread shadow all stay under 3:1 against the surface behind it (1.4.11)",
    ),
    (
        "focus-ring-missing",
        "a tab stop whose outline, shadow and border do not change on keyboard focus (2.4.7)",
    ),
    (
        "focus-ring-contrast",
        "a focus ring under 3:1 against what it is drawn on (1.4.11)",
    ),
    (
        "target-size",
        "a pointer target under 24x24 CSS px that the spacing exception does not cover (2.5.8)",
    ),
];

/// One finding: which check, where, and what was measured.
#[derive(Debug, Clone)]
pub struct Hit {
    pub family: String,
    pub page: String,
    pub config: String,
    pub element: String,
    pub value: String,
}

/// `describe(el)`: tag, id, role and a short name, enough to find the element.
/// Generated ids (`lsx-...`, digits) are left out, so a hit reads the same on every page.
const DESCRIBE: &str = r#"const describe = el => {
    let s = el.tagName.toLowerCase();
    if (el.id && !/^(lsx-|\d)/.test(el.id)) s += '#' + el.id;
    const role = el.getAttribute('role');
    if (role) s += `[role=${role}]`;
    const slot = el.getAttribute('data-slot');
    if (slot) s += `[data-slot=${slot}]`;
    const name = (el.getAttribute('aria-label') || el.textContent || el.value || '')
        .trim().replace(/\s+/g, ' ').slice(0, 40);
    return name ? `${s} "${name}"` : s;
};"#;

/// `clipped(el)`: inside a visually-hidden wrapper (under 4px, overflow not visible).
const CLIPPED: &str = r#"const clipped = el => {
    for (let at = el.parentElement; at; at = at.parentElement) {
        if (getComputedStyle(at).overflow === 'visible') continue;
        const r = at.getBoundingClientRect();
        if (r.width < 4 || r.height < 4) return true;
    }
    return false;
};"#;

/// Run the sweep and write the report. Called by `tests/sweep.rs`.
pub fn run() -> Result<()> {
    crate::browser::block_on(async {
        let pages = discover().await?;
        eprintln!("sweep: {} pages from the docs nav", pages.len());
        let jobs: Vec<(String, Scheme, Viewport)> = pages
            .iter()
            .flat_map(|page| {
                CONFIGS
                    .iter()
                    .map(move |&(scheme, viewport)| (page.clone(), scheme, viewport))
            })
            .collect();

        let outcomes: Vec<(String, String, Result<Checked>)> = futures::stream::iter(jobs)
            .map(|(page, scheme, viewport)| async move {
                let config = format!("{} {}", scheme.name(), viewport.name());
                let outcome = check_page(&page, scheme, viewport, &config).await;
                if let Err(error) = &outcome {
                    eprintln!("sweep: {page} ({config}): {error:#}");
                }
                (page, config, outcome)
            })
            .buffer_unordered(PARALLEL)
            .collect()
            .await;

        let mut hits = Vec::new();
        let mut checked: BTreeMap<&str, usize> = BTreeMap::new();
        let mut errors = Vec::new();
        for (page, config, outcome) in outcomes {
            match outcome {
                Ok(found) => {
                    hits.extend(found.hits);
                    for (family, count) in found.checked {
                        *checked.entry(family).or_default() += count;
                    }
                }
                Err(error) => errors.push(format!("{page} ({config}): {error:#}")),
            }
        }

        let (report, summary) = render(&pages, &hits, &checked, &errors);
        let path = report_path();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).context("create the report directory")?;
        }
        std::fs::write(&path, &report).with_context(|| format!("write {}", path.display()))?;
        if let Some(dir) = crate::journal::dir() {
            let _ = std::fs::write(dir.join("a11y-sweep.md"), &report);
        }
        println!("{summary}; report {}", path.display());

        if !errors.is_empty() {
            bail!(
                "the sweep hit {} harness error(s); the report lists them:\n  {}",
                errors.len(),
                errors.join("\n  ")
            );
        }
        Ok(())
    })
}

/// `E2E_SWEEP_REPORT`, set by the runner; the artifacts directory otherwise.
fn report_path() -> PathBuf {
    std::env::var_os("E2E_SWEEP_REPORT")
        .map(PathBuf::from)
        .or_else(|| crate::journal::dir().map(|dir| dir.join("a11y-sweep.md")))
        .unwrap_or_else(|| std::env::temp_dir().join("a11y-sweep.md"))
}

/// Every page path the rendered docs nav links (all sections expanded), home first.
async fn discover() -> Result<Vec<String>> {
    let mut fixture = Fixture::open_until("/", Viewport::Desktop, Scheme::Light, READY).await?;
    // The landing page is full width with the nav hidden: read it from the first page it links.
    let hidden: bool = fixture
        .page
        .evaluate(
            "(() => { const nav = document.querySelector('#docs-nav'); \
             return !nav || getComputedStyle(nav).visibility === 'hidden'; })()",
        )
        .await?
        .into_value()?;
    if hidden {
        let first: Option<String> = fixture
            .page
            .evaluate(
                "(() => { const a = [...document.querySelectorAll('a[href^=\"/\"]')] \
                 .find(a => a.getAttribute('href') !== '/'); return a ? a.getAttribute('href') : null; })()",
            )
            .await?
            .into_value()?;
        let first = first.context("the landing page links no docs page and shows no nav")?;
        let _ = fixture.close().await;
        fixture = Fixture::open_until(&first, Viewport::Desktop, Scheme::Light, READY).await?;
    }
    let page = &fixture.page;
    const COLLAPSED: &str = "#docs-nav [role=treeitem][aria-expanded=false]";
    crate::wait::for_selector(page, "#docs-nav [role=treeitem]").await?;
    // One click per collapsed section; a closed one's row is the item's whole box.
    for _ in 0..50 {
        let before: usize = page
            .evaluate(format!(
                "(() => {{ const all = document.querySelectorAll({COLLAPSED:?}); \
                 if (all.length) all[0].scrollIntoView({{ block: 'center' }}); \
                 return all.length; }})()"
            ))
            .await?
            .into_value()?;
        if before == 0 {
            break;
        }
        crate::passes::pointer::click(page, COLLAPSED).await?;
        crate::wait::for_js_true(
            page,
            &format!("document.querySelectorAll({COLLAPSED:?}).length < {before}"),
            "a docs nav section to expand",
        )
        .await?;
    }
    let links: Vec<String> = page
        .evaluate(
            "[...document.querySelectorAll('#docs-nav a[href]')] \
             .map(a => new URL(a.href).pathname)",
        )
        .await?
        .into_value()?;
    let _ = fixture.close().await;

    let mut pages = vec!["/".to_string()];
    for link in links {
        if !pages.contains(&link) {
            pages.push(link);
        }
    }
    if pages.len() < 10 {
        bail!("the docs nav yielded only {pages:?}; the sweep would cover almost nothing");
    }
    Ok(pages)
}

async fn check_page(
    path: &str,
    scheme: Scheme,
    viewport: Viewport,
    config: &str,
) -> Result<Checked> {
    let fixture = Fixture::open_until(path, viewport, scheme, READY).await?;
    settle(&fixture.page).await?;
    let checked = check(&fixture.page, viewport, path, config).await;
    // A signals warning on close is the page's own business, not the sweep's.
    let _ = fixture.close().await;
    checked
}

/// Every check on one open page. Public for `negative.rs`, which runs it on
/// the broken fixtures to watch each family fail.
pub async fn check(page: &Page, viewport: Viewport, path: &str, config: &str) -> Result<Checked> {
    let hit = |family: &str, element: String, value: String| Hit {
        family: family.to_string(),
        page: path.to_string(),
        config: config.to_string(),
        element,
        value,
    };
    let mut hits = Vec::new();
    let mut checked = Vec::new();

    // The docs page scrolls inside a `ScrollArea`, and axe calls text below its
    // fold "overlapped": grow the viewport until nothing scrolls.
    unfold(page, viewport).await?;

    let axe = contrast::run_full(page, "body").await?;
    for violation in &axe.violations {
        let family = match violation.id.as_str() {
            "color-contrast" => "text-contrast".to_string(),
            rule => format!("axe:{rule}"),
        };
        for node in &violation.nodes {
            hits.push(hit(
                &family,
                axe_element(node),
                axe_value(node, &violation.help),
            ));
        }
    }
    // axe leaves text over a sibling layer undecided (a field's ring overlay, a
    // tab indicator): measured here against what `elementsFromPoint` puts under it.
    let undecided: Vec<&contrast::Node> = axe
        .incomplete
        .iter()
        .filter(|v| v.id == "color-contrast")
        .flat_map(|v| &v.nodes)
        .collect();
    let measured = text_contrast(page, &undecided).await?;
    checked.push(("text-contrast-undetermined", undecided.len()));
    for (node, reading) in undecided.into_iter().zip(measured) {
        let axe_said = axe_value(node, "undecided");
        match reading {
            Some(r) if r.ratio < r.need => hits.push(hit(
                "text-contrast",
                axe_element(node),
                format!(
                    "{:.2}:1 ({} on {}, {}px {}), wants {}:1; measured by the sweep, axe: {axe_said}",
                    r.ratio, r.fg, r.bg, r.size, r.weight, r.need
                ),
            )),
            Some(_) => {}
            None => hits.push(hit("text-contrast-undetermined", axe_element(node), axe_said)),
        }
    }

    let (found, controls) = boundaries(page).await?;
    checked.push(("boundary-contrast", controls));
    for (element, value) in found {
        hits.push(hit("boundary-contrast", element, value));
    }
    let (found, measured) = targets(page).await?;
    checked.push(("target-size", measured));
    for (element, value) in found {
        hits.push(hit("target-size", element, value));
    }
    let (found, stops) = rings(page).await?;
    checked.push(("focus-ring-missing", stops));
    checked.push(("focus-ring-contrast", stops));
    for (family, element, value) in found {
        hits.push(hit(family, element, value));
    }
    Ok(Checked { hits, checked })
}

/// One page's hits, and how many elements each family measured there.
pub struct Checked {
    pub hits: Vec<Hit>,
    pub checked: Vec<(&'static str, usize)>,
}

/// The sweep's own text reading, for a node axe left undecided.
#[derive(Debug, serde::Deserialize)]
struct TextReading {
    ratio: f64,
    need: f64,
    fg: String,
    bg: String,
    size: f64,
    weight: String,
}

/// Colour helpers shared by the sweep's own contrast readings.
const COLOUR: &str = r#"const parse = c => {
    // A `color-mix()` computes to `color(srgb r g b / a)`, channels 0..1.
    const srgb = c && c.match(/^color\(srgb ([\d.e-]+) ([\d.e-]+) ([\d.e-]+)(?: \/ ([\d.e-]+))?\)$/);
    if (srgb) {
        const [r, g, b] = srgb.slice(1, 4).map(v => Math.min(255, Math.max(0, v * 255)));
        return { r, g, b, a: srgb[4] === undefined ? 1 : +srgb[4] };
    }
    if (!c || !c.startsWith('rgb')) return null;
    const [r, g, b, a = 1] = c.match(/[\d.]+/g).map(Number);
    return { r, g, b, a };
};
const over = (top, under) => ({
    r: top.r * top.a + under.r * (1 - top.a),
    g: top.g * top.a + under.g * (1 - top.a),
    b: top.b * top.a + under.b * (1 - top.a),
    a: 1,
});
const lum = c => {
    const f = v => (v /= 255) <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
    return 0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b);
};
const ratio = (a, b) => {
    const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x);
    return (hi + 0.05) / (lo + 0.05);
};
const rgb = c => `rgb(${Math.round(c.r)}, ${Math.round(c.g)}, ${Math.round(c.b)})`;
const WHITE = { r: 255, g: 255, b: 255, a: 1 };"#;

/// `gradientAt(image, box, x, y)`: the colour one `linear-gradient()` paints at
/// a point of `box`, or `null` for anything else (several layers, corners, an image).
const GRADIENT: &str = r#"const gradientAt = (image, box, x, y) => {
    const m = image.match(/^linear-gradient\((.*)\)$/);
    if (!m || /gradient\(/.test(m[1]) || /url\(/.test(m[1])) return null;
    const parts = m[1].split(/,(?![^(]*\))/).map(p => p.trim());
    const sides = { 'to top': 0, 'to right': 90, 'to bottom': 180, 'to left': 270 };
    let angle = 180;
    if (/deg$/.test(parts[0])) angle = parseFloat(parts.shift());
    else if (parts[0] in sides) angle = sides[parts.shift()];
    else if (parts[0].startsWith('to ')) return null;
    const rad = angle * Math.PI / 180;
    const len = Math.abs(box.width * Math.sin(rad)) + Math.abs(box.height * Math.cos(rad));
    const stops = [];
    for (const part of parts) {
        const c = part.match(/^((?:rgba?|color)\([^)]*\))\s*(.*)$/);
        const colour = c && parse(c[1]);
        if (!colour) return null;
        const at = c[2].split(/\s+/).filter(Boolean)
            .map(v => v.endsWith('%') ? parseFloat(v) / 100 : v.endsWith('px') ? parseFloat(v) / len : NaN);
        if (at.some(isNaN)) return null;
        if (!at.length) stops.push({ colour, at: null });
        for (const a of at) stops.push({ colour, at: a });
    }
    if (stops.length < 2) return null;
    // CSS's fix-ups: ends at 0 and 1, never backwards, gaps spread evenly.
    if (stops[0].at === null) stops[0].at = 0;
    if (stops[stops.length - 1].at === null) stops[stops.length - 1].at = 1;
    let furthest = -Infinity;
    for (const s of stops) if (s.at !== null) furthest = s.at = Math.max(s.at, furthest);
    for (let i = 1; i < stops.length; i++) {
        if (stops[i].at !== null) continue;
        let j = i;
        while (stops[j].at === null) j++;
        const [from, to] = [stops[i - 1].at, stops[j].at];
        for (let k = i; k < j; k++) stops[k].at = from + (to - from) * (k - i + 1) / (j - i + 1);
    }
    const t = ((x - box.left - box.width / 2) * Math.sin(rad)
        - (y - box.top - box.height / 2) * Math.cos(rad)) / len + 0.5;
    if (t <= stops[0].at) return stops[0].colour;
    for (let i = 1; i < stops.length; i++) {
        if (t > stops[i].at) continue;
        const [p, q] = [stops[i - 1].colour, stops[i].colour];
        const f = stops[i].at > stops[i - 1].at ? (t - stops[i - 1].at) / (stops[i].at - stops[i - 1].at) : 1;
        // Premultiplied, as CSS blends: a fade to transparent does not darken.
        const a = p.a + (q.a - p.a) * f;
        const mix = k => a > 0 ? (p[k] * p.a + (q[k] * q.a - p[k] * p.a) * f) / a : 0;
        return { r: mix('r'), g: mix('g'), b: mix('b'), a };
    }
    return stops[stops.length - 1].colour;
};
// Where `e`'s background image is drawn: its box unless sized. `null` where
// that cannot be read, `tile.repeat` false for a single sized tile.
const tileOf = (e, s) => {
    const b = e.getBoundingClientRect();
    if (s.backgroundSize === 'auto' || s.backgroundSize === 'auto auto') return { box: b, repeat: true };
    const size = s.backgroundSize.split(' '), pos = s.backgroundPosition.split(' ');
    if (size.length !== 2 || pos.length !== 2 || s.backgroundRepeat !== 'no-repeat') return null;
    const len = (v, whole) => v.endsWith('px') ? parseFloat(v) : v.endsWith('%') ? whole * parseFloat(v) / 100 : NaN;
    const [w, h] = [len(size[0], b.width), len(size[1], b.height)];
    const at = (v, whole, t) => v.endsWith('px') ? parseFloat(v) : v.endsWith('%') ? (whole - t) * parseFloat(v) / 100 : NaN;
    const [x, y] = [at(pos[0], b.width, w), at(pos[1], b.height, h)];
    if ([w, h, x, y].some(isNaN)) return null;
    return { box: { left: b.left + x, top: b.top + y, width: w, height: h }, repeat: false };
};"#;

/// Text contrast axe could not decide, against the stacked backgrounds at three points
/// of the first line. `None` over an image or when covered.
async fn text_contrast(page: &Page, nodes: &[&contrast::Node]) -> Result<Vec<Option<TextReading>>> {
    if nodes.is_empty() {
        return Ok(Vec::new());
    }
    let targets: Vec<&str> = nodes.iter().map(|n| n.target.as_str()).collect();
    let script = format!(
        r#"(() => {{ {COLOUR} {GRADIENT}
            // The later checks see the scrollers as they were.
            const scrolled = [...document.querySelectorAll('*')]
                .filter(e => e.scrollHeight > e.clientHeight || e.scrollWidth > e.clientWidth)
                .map(e => [e, e.scrollTop, e.scrollLeft]);
            const [pageX, pageY] = [scrollX, scrollY];
            const readings = {targets}.map(target => {{
                let el;
                try {{ el = document.querySelector(target); }} catch (e) {{ return null; }}
                if (!el) return null;
                // The first line, cut to what its clipping ancestors show: a line wider
                // than its scroller is aimed at the part it shows.
                const shown = () => {{
                    const range = document.createRange();
                    range.selectNodeContents(el);
                    const r = [...range.getClientRects()].find(r => r.width > 0 && r.height > 0)
                        || el.getBoundingClientRect();
                    let [x0, x1, y0, y1] = [r.left, r.right, r.top, r.bottom];
                    for (let at = el.parentElement; at; at = at.parentElement) {{
                        if (getComputedStyle(at).overflow === 'visible') continue;
                        const c = at.getBoundingClientRect();
                        [x0, x1, y0, y1] = [Math.max(x0, c.left), Math.min(x1, c.right),
                            Math.max(y0, c.top), Math.min(y1, c.bottom)];
                    }}
                    return x1 > x0 && y1 - y0 >= r.height / 2 ? [x0, x1, y0, y1] : null;
                }};
                // Text past an inner scroller's fold is clipped, and axe calls it
                // "overlapped". Scrolled only then: a scroll disturbs the later checks.
                let line = shown();
                if (!line) {{
                    el.scrollIntoView({{ block: 'center', inline: 'nearest', behavior: 'instant' }});
                    line = shown();
                }}
                if (!line) return null;
                const [x0, x1, y0, y1] = line;
                // What lies under (x, y), flattened; null where an image or the unknown does.
                const backdrop = (x, y) => {{
                    const stack = document.elementsFromPoint(x, y);
                    // Something other than the text itself on top: covered, not measurable.
                    if (stack.findIndex(e => e === el || el.contains(e)) !== 0) return null;
                    const layers = [];
                    for (const e of stack) {{
                        if (['IMG', 'VIDEO', 'CANVAS', 'PICTURE', 'IFRAME'].includes(e.tagName)) return null;
                        const s = getComputedStyle(e);
                        if (s.backgroundImage !== 'none') {{
                            const tile = tileOf(e, s);
                            if (!tile) return null;
                            const {{ left, top, width, height }} = tile.box;
                            const inside = x >= left && x < left + width && y >= top && y < top + height;
                            if (inside || tile.repeat) {{
                                const c = gradientAt(s.backgroundImage, tile.box, x, y);
                                if (!c) return null;
                                if (c.a > 0) layers.push(c);
                                if (c.a >= 1) break;
                            }}
                        }}
                        const c = parse(s.backgroundColor);
                        if (c && c.a > 0) {{ layers.push(c); if (c.a >= 1) break; }}
                    }}
                    return layers.reverse().reduce((base, layer) => over(layer, base), WHITE);
                }};
                const s = getComputedStyle(el);
                const text = parse(s.color);
                if (!text) return null;
                for (let at = el; at; at = at.parentElement) text.a *= +getComputedStyle(at).opacity;
                // A quarter, the middle and three quarters along the line: a gradient
                // changes under it. The worst of the three counts.
                const y = (y0 + y1) / 2;
                const bgs = [0.25, 0.5, 0.75].map(f => backdrop(x0 + (x1 - x0) * f, y));
                if (bgs.some(bg => !bg)) return null;
                const [fg, bg] = bgs.map(bg => [over(text, bg), bg])
                    .reduce((a, b) => ratio(...a) <= ratio(...b) ? a : b);
                const size = parseFloat(s.fontSize), bold = parseInt(s.fontWeight) >= 700;
                return {{
                    ratio: ratio(fg, bg),
                    need: size >= 24 || (bold && size >= 18.66) ? 3 : 4.5,
                    fg: rgb(fg),
                    bg: rgb(bg),
                    size,
                    weight: s.fontWeight,
                }};
            }});
            for (const [e, top, left] of scrolled) e.scrollTo({{ top, left, behavior: 'instant' }});
            scrollTo({{ top: pageY, left: pageX, behavior: 'instant' }});
            return readings;
        }})()"#,
        targets = serde_json::to_string(&targets)?,
    );
    Ok(page.evaluate(script).await?.into_value()?)
}

/// Grow the viewport by the deepest vertical scroll on the page, so every
/// element lies on screen. The width, and so the layout's breakpoints, stay.
async fn unfold(page: &Page, viewport: Viewport) -> Result<()> {
    // Page-level scrollers only (half the viewport or more): a demo's own
    // fixed-height `ScrollArea` would never stop asking for more.
    const EXTRA: &str = "Math.max(0, ...[...document.querySelectorAll('*')] \
         .filter(el => el.scrollHeight > el.clientHeight + 1 && el.clientHeight >= innerHeight / 2 \
             && (el === document.documentElement \
                 || ['auto', 'scroll'].includes(getComputedStyle(el).overflowY))) \
         .map(el => el.scrollHeight - el.clientHeight))";
    let (width, height) = viewport.size();
    let mut tall = height;
    for _ in 0..3 {
        let extra: f64 = page.evaluate(EXTRA).await?.into_value()?;
        if extra < 1.0 {
            break;
        }
        tall = (tall + extra.ceil() as i64).min(30_000);
        page.execute(SetDeviceMetricsOverrideParams::new(
            width,
            tall,
            1.0,
            viewport == Viewport::Mobile,
        ))
        .await?;
        settle(page).await?;
    }
    Ok(())
}

fn axe_element(node: &contrast::Node) -> String {
    let html: String = node.html.chars().take(120).collect();
    format!("{} `{}`", node.target, html.replace('\n', " "))
}

/// The line of axe's failure summary that carries the measurement.
fn axe_value(node: &contrast::Node, help: &str) -> String {
    node.failure_summary
        .as_deref()
        .and_then(|summary| {
            summary
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty() && !line.starts_with("Fix "))
        })
        .unwrap_or(help)
        .to_string()
}

/// Web fonts loaded, the DOM done growing (markdown arrives by fetch), and
/// finite animations over. Best effort: a page still moving is swept as is.
async fn settle(page: &Page) -> Result<()> {
    const COUNT: &str = "document.fonts.status === 'loaded' \
                         ? document.getElementsByTagName('*').length : -1";
    const RUNNING: &str = "[...document.getAnimations()].filter(a => a.playState === 'running' \
                           && a.effect && a.effect.getComputedTiming().iterations !== Infinity).length";
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let (mut last, mut stable) = (-2i64, 0);
    while std::time::Instant::now() < deadline {
        let count: i64 = page.evaluate(COUNT).await?.into_value()?;
        let running: i64 = page.evaluate(RUNNING).await?.into_value()?;
        if count >= 0 && count == last && running == 0 {
            stable += 1;
            if stable >= 3 {
                break;
            }
        } else {
            stable = 0;
        }
        last = count;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    Ok(())
}

/// WCAG 1.4.11 for fields, checkboxes, radios, switches and sliders (not labelled buttons).
/// The boundary is the first visible border, fill or shadow on the control, siblings or two ancestors.
async fn boundaries(page: &Page) -> Result<(Vec<(String, String)>, usize)> {
    let script = format!(
        r#"(() => {{ {DESCRIBE} {CLIPPED} {COLOUR}
            const behind = el => {{
                const layers = [];
                for (let p = el; p; p = p.parentElement) {{
                    const c = parse(getComputedStyle(p).backgroundColor);
                    if (c && c.a > 0) {{ layers.push(c); if (c.a >= 1) break; }}
                }}
                return layers.reverse().reduce((base, layer) => over(layer, base), WHITE);
            }};
            const shown = el => {{
                const s = getComputedStyle(el);
                if (s.display === 'none' || s.visibility === 'hidden' || +s.opacity === 0) return false;
                const r = el.getBoundingClientRect();
                return r.width >= 4 && r.height >= 4 && !clipped(el);
            }};
            const indicators = el => {{
                const s = getComputedStyle(el);
                const out = behind(el.parentElement);
                const found = [];
                for (const side of ['Top', 'Right', 'Bottom', 'Left']) {{
                    const style = s[`border${{side}}Style`];
                    if (parseFloat(s[`border${{side}}Width`]) < 1 || style === 'none' || style === 'hidden') continue;
                    const c = parse(s[`border${{side}}Color`]);
                    if (c && c.a > 0) found.push(['border', ratio(over(c, out), out)]);
                }}
                const fill = parse(s.backgroundColor);
                if (fill && fill.a > 0) found.push(['fill', ratio(over(fill, out), out)]);
                for (const m of (s.boxShadow || '').matchAll(/(rgba?\([^)]*\))\s+(-?[\d.]+)px\s+(-?[\d.]+)px\s+([\d.]+)px\s+(-?[\d.]+)px/g)) {{
                    const c = parse(m[1]);
                    if (c && c.a > 0 && +m[4] === 0 && +m[5] > 0) found.push(['shadow', ratio(over(c, out), out)]);
                }}
                return {{ found, out }};
            }};
            const CONTROLS = 'input:not([type=hidden]):not([type=button]):not([type=submit])'
                + ':not([type=reset]):not([type=image]):not([type=file]), select, textarea, '
                + '[role=textbox], [role=combobox], [role=spinbutton], [role=searchbox], '
                + '[role=checkbox], [role=radio], [role=switch], [role=slider]';
            const hits = [];
            let checked = 0;
            for (const el of document.querySelectorAll(CONTROLS)) {{
                if (el.closest('[aria-hidden=true], [inert]')) continue;
                if (el.disabled || el.closest('[aria-disabled=true]')) continue;
                const s = getComputedStyle(el);
                if (s.display === 'none' || s.visibility === 'hidden') continue;
                // A control the user agent draws, unmodified, is exempt.
                if (['checkbox', 'radio', 'range'].includes(el.type) && s.appearance !== 'none' && shown(el)) continue;
                const parent = el.parentElement;
                const boxes = [el, el.previousElementSibling, el.nextElementSibling, parent, parent && parent.parentElement]
                    .filter(box => box && shown(box));
                if (!boxes.length) continue;
                checked++;
                let best = null;
                for (const box of boxes) {{
                    const {{ found, out }} = indicators(box);
                    if (!found.length) continue;
                    const [kind, value] = found.reduce((a, b) => (b[1] > a[1] ? b : a));
                    best = {{ box, kind, value, out }};
                    break;
                }}
                if (!best) {{
                    hits.push([describe(el), 'no border, fill or spread shadow on it or its frame']);
                }} else if (best.value < 3) {{
                    const via = best.box === el ? '' : ` (drawn by ${{describe(best.box).split(' ')[0]}})`;
                    hits.push([describe(el), `${{best.kind}} ${{best.value.toFixed(2)}}:1 against ${{rgb(best.out)}}${{via}}`]);
                }}
            }}
            return [hits, checked];
        }})()"#
    );
    Ok(page.evaluate(script).await?.into_value()?)
}

/// WCAG 2.5.8 over every visible pointer target, with the spacing exception
/// and the inline exception (a link inside a sentence).
async fn targets(page: &Page) -> Result<(Vec<(String, String)>, usize)> {
    let script = format!(
        r#"(() => {{ {DESCRIBE} {CLIPPED}
            document.querySelectorAll('[data-sweep-target]').forEach(el => el.removeAttribute('data-sweep-target'));
            const inline = el => {{
                if (getComputedStyle(el).display !== 'inline') return false;
                return [...el.parentElement.childNodes]
                    .some(n => n !== el && n.nodeType === Node.TEXT_NODE && n.textContent.trim());
            }};
            const names = [];
            for (const el of document.querySelectorAll({targets})) {{
                const s = getComputedStyle(el);
                if (s.display === 'none' || s.visibility === 'hidden' || s.pointerEvents === 'none') continue;
                const r = el.getBoundingClientRect();
                if (r.width <= 0 || r.height <= 0 || clipped(el)) continue;
                if (el.closest('[aria-hidden=true], [inert]')) continue;
                if (el.disabled || el.getAttribute('aria-disabled') === 'true') continue;
                if (inline(el)) continue;
                el.setAttribute('data-sweep-target', names.length);
                names.push(describe(el));
            }}
            return names;
        }})()"#,
        targets = serde_json::to_string(target_size::TARGETS)?,
    );
    let names: Vec<String> = page.evaluate(script).await?.into_value()?;
    let measured = target_size::measure_spacing(page, "[data-sweep-target]").await?;
    if measured.len() != names.len() {
        bail!(
            "target size: marked {} targets but measured {}",
            names.len(),
            measured.len()
        );
    }
    let mut hits = Vec::new();
    let count = names.len();
    for (name, spaced) in names.into_iter().zip(measured) {
        let size = format!("{:.1}x{:.1}", spaced.target.width, spaced.target.height);
        if let Err(error) = target_size::assert_sizes_spaced(&name, &[spaced]) {
            let why = error.to_string();
            let why = why.split_once(": ").map_or(why.as_str(), |(_, rest)| rest);
            hits.push((name, format!("{size}; {why}")));
        }
    }
    Ok((hits, count))
}

type Found = (&'static str, String, String);

/// Tab through the page, reading each stop focused and then again once the
/// next Tab has left it: the second reading is its unfocused baseline.
async fn rings(page: &Page) -> Result<(Vec<Found>, usize)> {
    page.evaluate(
        "document.querySelectorAll('[data-sweep-focus]').forEach(el => el.removeAttribute('data-sweep-focus')); \
         document.activeElement && document.activeElement.blur()",
    )
    .await?;
    let selector = |index: usize| format!("[data-sweep-focus='{index}']");
    let mut hits = Vec::new();
    let mut previous: Option<(usize, String, Vec<focus::Ring>)> = None;
    let mut stops = 0;
    for index in 0..=MAX_STOPS {
        keyboard::press(page, keyboard::TAB).await?;
        let (fresh, name): (bool, String) = page
            .evaluate(format!(
                r#"(() => {{ {DESCRIBE}
                    const el = document.activeElement;
                    if (!el || el === document.body || el.hasAttribute('data-sweep-focus')) return [false, ''];
                    el.setAttribute('data-sweep-focus', '{index}');
                    return [true, describe(el)];
                }})()"#
            ))
            .await?
            .into_value()?;
        if let Some((at, name, after)) = previous.take() {
            let before = focus::ring_chain(page, &selector(at)).await?;
            // Gone from the DOM once blurred (a popup's content): nothing to compare.
            if !before.is_empty() {
                hits.extend(judge(name, &before, &after));
            }
        }
        if !fresh {
            break;
        }
        if index == MAX_STOPS {
            hits.push((
                "focus-ring-missing",
                "(page)".to_string(),
                format!("stopped after {MAX_STOPS} tab stops; the rest is unchecked"),
            ));
            break;
        }
        let after = focus::ring_chain(page, &selector(index)).await?;
        previous = Some((index, name, after));
        stops += 1;
    }
    Ok((hits, stops))
}

fn judge(name: String, before: &[focus::Ring], after: &[focus::Ring]) -> Option<Found> {
    let Some(ring) = focus::pick_ring(before, after) else {
        return Some((
            "focus-ring-missing",
            name,
            "no outline, shadow or border change on it, its ancestors or ring overlays".into(),
        ));
    };
    focus::assert_ring_contrast(ring)
        .err()
        .map(|error| ("focus-ring-contrast", name, error.to_string()))
}

/// A hit's element and value with the numbers folded to `N`, the axe selector path
/// and text dropped and axe's undecided reason cut from a sweep measurement.
fn shape(hit: &Hit) -> (String, String) {
    let fold = |text: &str| {
        let mut out = String::with_capacity(text.len());
        for c in text.chars() {
            if !c.is_ascii_digit() {
                out.push(c);
            } else if !out.ends_with('N') {
                out.push('N');
            }
        }
        out
    };
    let element = hit
        .element
        .split_once(" `")
        .map_or(hit.element.as_str(), |(_, html)| {
            html.split_inclusive('>').next().unwrap_or(html)
        });
    let value = hit
        .value
        .split("; measured by the sweep")
        .next()
        .unwrap_or("");
    (fold(element), fold(value))
}

/// The report, and its one-line summary.
fn render(
    pages: &[String],
    hits: &[Hit],
    checked: &BTreeMap<&str, usize>,
    errors: &[String],
) -> (String, String) {
    // family -> shape -> the hits of that shape. A shape folds the numbers out, so
    // a code block's 500 line numbers read as one row.
    let mut grouped: BTreeMap<&str, BTreeMap<(String, String), Vec<&Hit>>> = BTreeMap::new();
    for hit in hits {
        grouped
            .entry(hit.family.as_str())
            .or_default()
            .entry(shape(hit))
            .or_default()
            .push(hit);
    }
    let count = |family: &str| hits.iter().filter(|h| h.family == family).count();
    let mut families: Vec<&str> = FAMILIES.iter().map(|(name, _)| *name).collect();
    families.extend(
        grouped
            .keys()
            .filter(|f| !families.contains(f))
            .copied()
            .collect::<Vec<_>>(),
    );

    let summary = format!(
        "a11y sweep: {} pages x {} configs, {} harness error(s); {}",
        pages.len(),
        CONFIGS.len(),
        errors.len(),
        families
            .iter()
            .map(|family| match checked.get(family) {
                Some(n) => format!("{family} {} of {n}", count(family)),
                None => format!("{family} {}", count(family)),
            })
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut out = String::new();
    let _ = writeln!(
        out,
        "# a11y sweep, {}\n\n{summary}\n",
        crate::journal::utc_now()
    );
    let _ = writeln!(
        out,
        "Configs: {}. Hits are grouped by family, then by element and measured value with numbers folded (one row per shape), with the pages and configs they appeared on.\n",
        CONFIGS
            .iter()
            .map(|(s, v)| format!("{} {}", s.name(), v.name()))
            .collect::<Vec<_>>()
            .join(", ")
    );
    if !errors.is_empty() {
        let _ = writeln!(out, "## Harness errors\n");
        for error in errors {
            let _ = writeln!(out, "- {error}");
        }
        let _ = writeln!(out);
    }
    for family in &families {
        let about = FAMILIES
            .iter()
            .find(|(name, _)| name == family)
            .map_or("an axe rule", |(_, about)| *about);
        let rows = grouped.get(family);
        // What the family looked at, so a zero reads as "measured, clean".
        let over = checked
            .get(family)
            .map_or(String::new(), |n| format!(" over {n} measured element(s)"));
        let _ = writeln!(
            out,
            "## {family}: {} hit(s), {} distinct{over}\n\n{about}\n",
            count(family),
            rows.map_or(0, BTreeMap::len)
        );
        for same in rows.into_iter().flat_map(BTreeMap::values) {
            let first = same[0];
            let mut elements: Vec<&str> = same.iter().map(|h| h.element.as_str()).collect();
            elements.sort_unstable();
            elements.dedup();
            let more = match elements.len() {
                1 => String::new(),
                n => format!(" (and {} more of this shape)", n - 1),
            };
            let _ = writeln!(out, "- {}{more}\n  {}", first.element, first.value);
            // page -> configs
            let mut on: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
            for hit in same {
                let configs = on.entry(hit.page.as_str()).or_default();
                if !configs.contains(&hit.config.as_str()) {
                    configs.push(hit.config.as_str());
                }
            }
            let places = on
                .iter()
                .map(|(page, configs)| format!("{page} ({})", configs.join(", ")))
                .collect::<Vec<_>>()
                .join("; ");
            let _ = writeln!(out, "  on {places}");
        }
        let _ = writeln!(out);
    }
    let _ = writeln!(out, "## Pages\n\n{}", pages.join(" "));
    (out, summary)
}
