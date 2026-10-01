//! Contrast and ARIA-validity rules through vendored, test-only `axe-core` 4.10.2 (MPL-2.0).
//! Axe finds defects a snapshot would baseline (todo 101), and knows what text is drawn on (todo 241).

use anyhow::{Context, Result, bail};
use chromiumoxide::Page;
use serde::Deserialize;

/// Read at runtime, not `include_str!`: the file is gitignored and extracted on first use.
fn axe_source() -> Result<&'static str> {
    static SOURCE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    if let Some(source) = SOURCE.get() {
        return Ok(source);
    }
    let path = crate::vendor::ensure_axe()?;
    let source = std::fs::read_to_string(&path)
        .with_context(|| format!("cannot read {}", path.display()))?;
    Ok(SOURCE.get_or_init(|| source))
}

/// The rules this suite runs; the full set adds page-level findings about the fixture.
const RULES: &[&str] = &[
    "color-contrast",
    "aria-valid-attr",
    "aria-valid-attr-value",
    "aria-required-attr",
    "aria-required-children",
    "aria-required-parent",
    "aria-roles",
    "aria-allowed-attr",
    "aria-hidden-focus",
    "label",
    "button-name",
    "link-name",
    "duplicate-id-aria",
];

#[derive(Debug, Deserialize)]
pub struct Violation {
    pub id: String,
    pub help: String,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub html: String,
    #[serde(default)]
    pub failure_summary: Option<String>,
    /// axe's CSS selector for the node, frames and shadow roots joined by ` > `.
    #[serde(default)]
    pub target: String,
}

/// Lifts `Modal`'s `body { overflow: hidden }` for one axe run (todo 327): axe treats the short body as
/// clipping the fixed dialog and skips its contrast. Only on unscrollable pages; `__moved` asserts no reflow.
const UNLOCK: &str = r#"
    const __boxes = () => Array.from(document.querySelectorAll('body *')).map(el => {
        const r = el.getBoundingClientRect();
        return r.x + ',' + r.y + ',' + r.width + ',' + r.height;
    });
    const __lifting = getComputedStyle(document.body).overflow.split(' ')[0] === 'hidden'
        && document.body.getBoundingClientRect().height < window.innerHeight;
    let __style = null;
    let __moved = null;
    if (__lifting) {
        const before = __boxes();
        __style = document.createElement('style');
        __style.textContent = 'body { overflow: visible !important; }';
        document.head.append(__style);
        const after = __boxes();
        const at = before.findIndex((box, i) => box !== after[i]);
        __moved = before.length !== after.length
            ? 'the number of elements changed from ' + before.length + ' to ' + after.length
            : (at >= 0 ? 'element ' + at + ' moved from ' + before[at] + ' to ' + after[at] : null);
    }
    try {
"#;

/// Puts the scroll lock back, closing [`UNLOCK`]'s `try`. A `finally`, so a throw or
/// early `return` cannot leave the lock lifted for the rest of the page.
const RELOCK: &str = "} finally { if (__style) { __style.remove(); } }";

async fn inject(page: &Page) -> Result<()> {
    let injected: bool = page
        .evaluate("typeof window.axe !== 'undefined'")
        .await?
        .into_value()?;
    if !injected {
        page.evaluate(axe_source()?).await?;
    }
    Ok(())
}

/// One axe run: its violations, and the nodes it could not decide.
#[derive(Debug, Deserialize)]
pub struct Run {
    pub violations: Vec<Violation>,
    /// Text over an image, a gradient or a pseudo-element: axe measures no ratio.
    pub incomplete: Vec<Violation>,
}

/// Run axe over the subtree at `selector`.
pub async fn run(page: &Page, selector: &str) -> Result<Vec<Violation>> {
    Ok(run_full(page, selector).await?.violations)
}

/// [`run`], keeping axe's undecided nodes too.
pub async fn run_full(page: &Page, selector: &str) -> Result<Run> {
    let audit = run_axe(page, selector, RULES, &[]).await?;
    Ok(Run {
        violations: audit.violations,
        incomplete: audit.incomplete,
    })
}

#[derive(Debug, Deserialize)]
struct Coverage {
    /// On-screen elements holding their own text, under the selector.
    wanted: usize,
    /// Of those, the ones `color-contrast` never evaluated.
    missing: Vec<String>,
}

/// One axe run over `root` with `rules`, and the coverage of each of `covers` read from it.
#[derive(Debug, Deserialize)]
struct Audit {
    /// `Some` when lifting the scroll lock changed the layout, which would make
    /// every reading in this run one of a page nobody sees.
    moved: Option<String>,
    violations: Vec<Violation>,
    incomplete: Vec<Violation>,
    coverage: Vec<Coverage>,
}

/// One `axe.run` serves both the violations and every coverage selector: it is the
/// battery's main cost, and `color-contrast` reads the same in a run with more rules.
async fn run_axe(page: &Page, root: &str, rules: &[&str], covers: &[&str]) -> Result<Audit> {
    inject(page).await?;

    let script = format!(
        r#"(async () => {{
            {UNLOCK}
            const result = await window.axe.run(document.querySelector({root}), {{
                runOnly: {{ type: 'rule', values: {rules} }},
            }});
            const shape = v => ({{
                id: v.id,
                help: v.help,
                nodes: v.nodes.map(n => ({{
                    html: n.html,
                    failure_summary: n.failureSummary || null,
                    target: n.target.flat().join(' > '),
                }})),
            }});

            // Still inside the lift: `wanted` must judge the same layout axe judged.
            const seen = new Set();
            for (const bucket of ['passes', 'violations', 'incomplete']) {{
                for (const rule of result[bucket]) {{
                    if (rule.id !== 'color-contrast') continue;
                    for (const node of rule.nodes) {{
                        const target = node.target[0];
                        const css = Array.isArray(target) ? target[target.length - 1] : target;
                        document.querySelectorAll(css).forEach(el => seen.add(el));
                    }}
                }}
            }}

            // axe's `isDisabled`: the nearest native `disabled` or `aria-disabled`
            // decides. WCAG 1.4.3 exempts inactive controls, so axe skips them.
            const disabled = el => {{
                for (let at = el; at; at = at.parentElement) {{
                    if (['FIELDSET', 'BUTTON', 'SELECT', 'INPUT', 'TEXTAREA'].includes(at.nodeName)
                        && at.hasAttribute('disabled')) return true;
                    const aria = at.getAttribute('aria-disabled');
                    if (aria) return aria.toLowerCase() === 'true';
                }}
                return false;
            }};
            // Chromium still gives a closed `<details>` body a rect, but nobody sees it and axe skips it.
            const folded = el => {{
                for (let d = el.closest('details:not([open])'); d;
                     d = d.parentElement && d.parentElement.closest('details:not([open])')) {{
                    const summary = d.querySelector(':scope > summary');
                    if (!summary || !summary.contains(el)) return true;
                }}
                return false;
            }};
            // Out of sight on purpose: inside a visually-hidden wrapper (1x1, overflow
            // hidden), or scrolled out of a scroller. A box clipped to nothing still counts.
            const clipped = el => {{
                const e = el.getBoundingClientRect();
                for (let at = el.parentElement; at; at = at.parentElement) {{
                    const s = getComputedStyle(at);
                    if (s.overflow === 'visible') continue;
                    const r = at.getBoundingClientRect();
                    if (r.width <= 1 && r.height <= 1) return true;
                    const scrolls = /auto|scroll/.test(s.overflowX + s.overflowY);
                    if (scrolls && (e.bottom <= r.top || e.top >= r.bottom
                        || e.right <= r.left || e.left >= r.right)) return true;
                }}
                return false;
            }};

            const coverage = {covers}.map(selector => {{
            const wanted = [];
            for (const host of document.querySelectorAll(selector)) {{
                for (const el of [host, ...host.querySelectorAll('*')]) {{
                    // Axe reports the element owning the text, with a letter or digit
                    // (`ignoreUnicode` skips punctuation-only text).
                    const owns = Array.from(el.childNodes)
                        .some(n => n.nodeType === Node.TEXT_NODE && /[\p{{L}}\p{{N}}]/u.test(n.textContent));
                    if (!owns) continue;
                    if (el.closest('[aria-hidden=true]')) continue;
                    if (disabled(el)) continue;
                    // A label of a disabled control is inactive too.
                    const label = el.closest('label');
                    if (label && label.control && disabled(label.control)) continue;
                    if (el.closest('[inert]') || folded(el) || clipped(el)) continue;
                    const style = getComputedStyle(el);
                    if (style.display === 'none' || style.visibility === 'hidden'
                        || style.opacity === '0') continue;
                    const r = el.getBoundingClientRect();
                    // Smaller is a visually-hidden recipe, which axe rightly skips.
                    if (r.width < 4 || r.height < 4) continue;
                    if (r.bottom <= 0 || r.right <= 0
                        || r.top >= window.innerHeight || r.left >= window.innerWidth) continue;
                    wanted.push(el);
                }}
            }}

            return {{
                wanted: wanted.length,
                missing: wanted.filter(el => !seen.has(el))
                    .map(el => el.outerHTML.slice(0, 200)),
            }};
            }});

            return {{
                moved: __moved,
                violations: result.violations.map(shape),
                incomplete: result.incomplete.map(shape),
                coverage,
            }};
            {RELOCK}
        }})()"#,
        root = serde_json::to_string(root)?,
        rules = serde_json::to_string(rules)?,
        covers = serde_json::to_string(covers)?,
    );

    let audit: Audit = page.evaluate(script).await?.into_value()?;
    if let Some(moved) = &audit.moved {
        bail!(
            "lifting the modal scroll lock for the axe run changed the layout ({moved}), \
             so the contrast reading would be of a page nobody sees. See `UNLOCK` in \
             `passes/contrast.rs` and todo 327."
        );
    }
    Ok(audit)
}

/// Every on-screen element under `selector` holding its own text must appear in axe's
/// `color-contrast` passes, violations or incomplete: "found nothing" is not "looked" (todo 327).
pub async fn assert_covers(page: &Page, root: &str, selector: &str) -> Result<usize> {
    let mut audit = run_axe(page, root, &["color-contrast"], &[selector]).await?;
    check_coverage(selector, audit.coverage.remove(0))
}

/// The violations under `root` past the waivers, then each of `covers` as [`assert_covers`],
/// from one axe run. Returns the text count each cover held; `fired` as [`check_violations`].
pub async fn assert_clean_and_covered(
    page: &Page,
    root: &str,
    waivers: &[Waiver],
    fired: &mut [bool],
    covers: &[&str],
) -> Result<Vec<usize>> {
    let audit = run_axe(page, root, RULES, covers).await?;
    let violations = with_measured(page, audit.violations, &audit.incomplete).await?;
    check_violations(violations, root, waivers, fired)?;
    covers
        .iter()
        .zip(audit.coverage)
        .map(|(selector, coverage)| check_coverage(selector, coverage))
        .collect()
}

fn check_coverage(selector: &str, coverage: Coverage) -> Result<usize> {
    if !coverage.missing.is_empty() {
        let mut report = String::new();
        for html in &coverage.missing {
            report.push_str(&format!("\n      {html}"));
        }
        bail!(
            "contrast coverage: {} of {} on-screen text element(s) under {selector} were \
             never evaluated by axe's `color-contrast` rule, so a contrast failure in them \
             would read as green:{report}",
            coverage.missing.len(),
            coverage.wanted,
        );
    }
    Ok(coverage.wanted)
}

/// A known violation not fixed yet. It names the owning todo and is still printed.
/// `Suite::run` fails on a waiver that matched nothing, so a fixed one gets removed.
pub struct Waiver {
    /// The axe rule id, e.g. `color-contrast`.
    pub rule: &'static str,
    /// Matched against the offending element's html, so a waiver is narrow.
    pub contains: &'static str,
    /// `color-contrast` only: the lowest ratio waived, so a further drop still fails.
    /// Other rules ignore it.
    pub floor: f64,
    pub why: &'static str,
}

impl Waiver {
    /// Whether this waives `node`, reported under axe rule `rule`.
    pub fn covers(&self, rule: &str, node: &Node) -> bool {
        self.rule == rule
            && node.html.contains(self.contains)
            && (rule != "color-contrast" || ratio_of(node).is_some_and(|r| r >= self.floor))
    }
}

/// The ratio axe (`contrast of 4.05 (`) or [`with_measured`] (`axe: 1.03:1`) reported.
fn ratio_of(node: &Node) -> Option<f64> {
    let summary = node.failure_summary.as_deref()?;
    let (_, rest) = summary
        .split_once("contrast of ")
        .or_else(|| summary.split_once("measured past axe: "))?;
    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// Todo 297: `success` (4.04:1) and `warning` (3.27:1) fail as text; a decision to keep them.
pub const TODO_297_SUCCESS: Waiver = Waiver {
    rule: "color-contrast",
    contains: "success",
    floor: 4.04,
    why: "todo 297 - `success` is 4.04:1 as text, kept",
};
pub const TODO_297_WARNING: Waiver = Waiver {
    rule: "color-contrast",
    contains: "warning",
    floor: 3.27,
    why: "todo 297 - `warning` is 3.27:1 as text, kept",
};
pub const TODO_297: &[Waiver] = &[TODO_297_SUCCESS, TODO_297_WARNING];

/// Todo 899: a CodeBlock line number's approved floor is 4.27:1, under the 4.5:1 this pass wants.
pub const LINE_NUMBERS: &[Waiver] = &[Waiver {
    rule: "color-contrast",
    contains: "data-slot=\"line-number\"",
    floor: 4.27,
    why: "todo 899 - a line number's approved floor is 4.27:1",
}];

/// Fail on any violation, printing what axe said rather than a bare count.
pub async fn assert_clean(page: &Page, selector: &str) -> Result<()> {
    assert_clean_except(page, selector, &[]).await
}

/// As `assert_clean`, but tolerating the named waivers.
pub async fn assert_clean_except(page: &Page, selector: &str, waivers: &[Waiver]) -> Result<()> {
    let run = run_full(page, selector).await?;
    let violations = with_measured(page, run.violations, &run.incomplete).await?;
    check_violations(
        violations,
        selector,
        waivers,
        &mut vec![false; waivers.len()],
    )
}

/// axe leaves text over a gradient or a sibling layer undecided, which read as green
/// (todo 1649). The sweep's reading measures it, and a shortfall joins `violations`.
async fn with_measured(
    page: &Page,
    mut violations: Vec<Violation>,
    incomplete: &[Violation],
) -> Result<Vec<Violation>> {
    let undecided: Vec<&Node> = incomplete
        .iter()
        .filter(|v| v.id == "color-contrast")
        .flat_map(|v| &v.nodes)
        .collect();
    let readings = crate::sweep::text_contrast(page, &undecided).await?;
    let nodes: Vec<Node> = undecided
        .into_iter()
        .zip(readings)
        .filter_map(|(node, reading)| {
            let r = reading.filter(|r| r.ratio < r.need)?;
            Some(Node {
                html: node.html.clone(),
                failure_summary: Some(format!(
                    "measured past axe: {:.2}:1 ({} on {}, {}px {}), wants {}:1",
                    r.ratio, r.fg, r.bg, r.size, r.weight, r.need
                )),
                target: node.target.clone(),
            })
        })
        .collect();
    if !nodes.is_empty() {
        violations.push(Violation {
            id: "color-contrast".to_string(),
            help: "Text axe left undecided (a gradient or a layer under it) misses its ratio"
                .to_string(),
            nodes,
        });
    }
    Ok(violations)
}

/// `fired[i]` turns true once `waivers[i]` covered a node.
fn check_violations(
    all: Vec<Violation>,
    selector: &str,
    waivers: &[Waiver],
    fired: &mut [bool],
) -> Result<()> {
    let mut violations = Vec::new();
    let mut waived = Vec::new();

    for violation in all {
        let mut bad = Vec::new();
        for node in violation.nodes {
            let Some(at) = waivers.iter().position(|w| w.covers(&violation.id, &node)) else {
                bad.push(node);
                continue;
            };
            fired[at] = true;
            let ratio = ratio_of(&node).map_or(String::new(), |r| format!(" ({r}:1)"));
            waived.push(format!(
                "{}{ratio} - {}\n      {}",
                violation.id, waivers[at].why, node.html
            ));
        }
        if !bad.is_empty() {
            violations.push(Violation {
                nodes: bad,
                ..violation
            });
        }
    }

    if !waived.is_empty() {
        eprintln!(
            "axe: {} waived violation(s) under {selector}:",
            waived.len()
        );
        for line in &waived {
            eprintln!("  {line}");
        }
    }

    if violations.is_empty() {
        return Ok(());
    }
    let mut report = String::new();
    for v in &violations {
        report.push_str(&format!("\n  {} - {}", v.id, v.help));
        for node in &v.nodes {
            report.push_str(&format!("\n      {}", node.html));
            if let Some(summary) = &node.failure_summary {
                report.push_str(&format!("\n      {}", summary.replace('\n', "\n      ")));
            }
        }
    }
    bail!(
        "axe reported {} violation(s) under {selector}:{report}",
        violations.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contrast(html: &str, summary: &str) -> Violation {
        Violation {
            id: "color-contrast".into(),
            help: String::new(),
            nodes: vec![Node {
                html: html.into(),
                failure_summary: Some(summary.into()),
                target: String::new(),
            }],
        }
    }

    const AXE: &str = "Fix any of the following:\n  Element has insufficient color contrast of \
                       4.05 (foreground color: #2e7d32, background color: #ffffff, font size: \
                       10.5pt (14px), font weight: normal). Expected contrast ratio of 4.5:1";

    #[test]
    fn a_ratio_is_read_from_axe_and_from_the_measured_summary() {
        let axe = contrast("<b class=\"success\">", AXE);
        assert_eq!(ratio_of(&axe.nodes[0]), Some(4.05));
        let measured = contrast("<b>", "measured past axe: 1.03:1 (rgb(0, 0, 0) on ...)");
        assert_eq!(ratio_of(&measured.nodes[0]), Some(1.03));
    }

    #[test]
    fn a_waiver_holds_down_to_its_floor_and_no_further() {
        let mut fired = [false; 2];
        check_violations(
            vec![contrast("<b class=\"success\">", AXE)],
            "#root",
            TODO_297,
            &mut fired,
        )
        .unwrap();
        assert_eq!(fired, [true, false]);

        let lower = AXE.replace("of 4.05", "of 2.10");
        let mut fired = [false; 2];
        let error = check_violations(
            vec![contrast("<b class=\"success\">", &lower)],
            "#root",
            TODO_297,
            &mut fired,
        )
        .unwrap_err();
        assert!(error.to_string().contains("1 violation"), "{error}");
        assert_eq!(fired, [false, false]);
    }
}
