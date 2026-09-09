//! Contrast, and the ARIA-validity rules that come with axe.
//!
//! Vendored `axe-core` (4.10.2, MPL-2.0, `e2e/vendor/axe.min.js`), injected
//! into the page. It is a test-only file: nothing in `libero` or `docs`
//! references it and it is not shipped.
//!
//! ## Why axe rather than our own maths
//!
//! The WCAG contrast formula is trivial. Deciding what a thing is *drawn on*
//! is not, once alpha compositing, ancestor traversal and gradients are in
//! play - and that is exactly where we have been wrong before: todo 241 came
//! from a colour that passes on white and fails on a tinted code block.
//! `principles/use-crates-for-solved-domains` applies.
//!
//! ## Why axe as well as the tree snapshot
//!
//! A snapshot detects *regression*: it tells you something changed. Axe detects
//! *defects*: it tells you something is wrong with no baseline to compare
//! against. Todo 101 is the case in point - `aria-activedescendant` named an
//! option that was not in the DOM, and a snapshot would have recorded the
//! dangling reference as the baseline and gone green ever after.

use anyhow::{Context, Result, bail};
use chromiumoxide::Page;
use serde::Deserialize;

/// Read at runtime rather than `include_str!`, because the file is gitignored
/// and extracted from `vendor/axe.zip` on first use - a compile-time include
/// would fail to build on a fresh clone.
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

/// The rules this suite runs. Deliberately a short list: axe's full set
/// overlaps the AX snapshot and reports page-level findings (landmarks,
/// document title) that are about the fixture rather than the component.
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
}

/// Lift `Modal`'s scroll lock for the duration of an axe run, and prove that
/// lifting it moved nothing.
///
/// ## Why this is here at all (todo 327)
///
/// axe's `isVisibleOnScreen` runs five screen checks, and one of them is
/// `overflowHidden`: it collects the element's `overflow: hidden` ancestors and
/// calls the element hidden when any of their rects fails to overlap it.
///
/// `Modal` locks scroll with `body { overflow: hidden; }`
/// (`components/overlay/modal.rs`). That makes `<body>` an overflow-hidden
/// ancestor of everything in the dialog - and `<body>`'s border box is sized to
/// the page's own content, which on a fixture is a couple of hundred pixels,
/// while the dialog is `position: fixed` and draws wherever the viewport puts
/// it. axe's ancestor walk does not stop at the fixed-position containing
/// block, so it concludes that a 122px-tall body clips away a row at y=171 -
/// and `color-contrast` becomes **inapplicable** to it. Measured 2026-09-20:
/// every element in `/modal`'s dialog, both arrows and all six thumbnails in
/// `/lightbox`, and every result row in `/spotlight`. `/drawer` was green only
/// because its content happens to sit inside the body box. Nothing to do with
/// the scroll box, the portal or `overflow: hidden auto`, which is what the
/// todo guessed.
///
/// The browser is not clipping anything: when `<html>`'s overflow is `visible`
/// the body's overflow propagates to the viewport and the body's own used value
/// becomes `visible`, but `getComputedStyle` still reports `hidden`, which is
/// what axe reads. So this is a tool defect, and the component is right.
///
/// ## Why the harness compensates rather than the component
///
/// `body { overflow: hidden }` is the scroll lock every modal in the world
/// uses. Changing it to please a checker would be distorting the library to fit
/// a tool. Lifting it for the length of one axe run restores exactly the state
/// the page has when no modal is open, and is only done when the page is not
/// scrollable anyway (the body box is shorter than the viewport), so no
/// scrollbar can appear and nothing can reflow.
///
/// That last sentence is an assertion, not a hope: `__moved` records whether
/// any element under `<body>` changed its rect across the lift, and a run that
/// moved something fails instead of reporting a measurement taken of a layout
/// nobody sees.
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

/// Puts the scroll lock back. Always paired with [`UNLOCK`], which opens the
/// `try` this closes, so everything between the two runs with the lock lifted
/// and the lock comes back however that block leaves.
///
/// The `finally` is the whole point. axe rejects rather than returns on a rule
/// that throws or a detached root, and a `return` in the middle of the block is
/// normal. With a plain statement here, any of those would leave `__style` in
/// the document and the scroll lock lifted **for the rest of the page's life** -
/// so the next assertion on that page would measure a document we quietly
/// altered, which is precisely what the rect check exists to prevent.
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

#[derive(Debug, Deserialize)]
struct Run {
    /// `Some` when lifting the scroll lock changed the layout, which would make
    /// every reading in this run one of a page nobody sees.
    moved: Option<String>,
    violations: Vec<Violation>,
}

/// Run axe over the subtree at `selector`.
pub async fn run(page: &Page, selector: &str) -> Result<Vec<Violation>> {
    inject(page).await?;

    let script = format!(
        r#"(async () => {{
            {UNLOCK}
            const result = await window.axe.run(document.querySelector({}), {{
                runOnly: {{ type: 'rule', values: {} }},
            }});
            return {{
                moved: __moved,
                violations: result.violations.map(v => ({{
                    id: v.id,
                    help: v.help,
                    nodes: v.nodes.map(n => ({{
                        html: n.html,
                        failure_summary: n.failureSummary || null,
                    }})),
                }})),
            }};
            {RELOCK}
        }})()"#,
        serde_json::to_string(selector)?,
        serde_json::to_string(RULES)?,
    );

    let run: Run = page.evaluate(script).await?.into_value()?;
    if let Some(moved) = run.moved {
        bail!(
            "lifting the modal scroll lock for the axe run changed the layout ({moved}), \
             so the contrast reading would be of a page nobody sees. See `UNLOCK` in \
             `passes/contrast.rs` and todo 327."
        );
    }
    Ok(run.violations)
}

#[derive(Debug, Deserialize)]
struct Coverage {
    moved: Option<String>,
    /// On-screen elements holding their own text, under the selector.
    wanted: usize,
    /// Of those, the ones `color-contrast` never evaluated.
    missing: Vec<String>,
}

/// Every on-screen element under `selector` that holds text of its own must
/// have been **evaluated** by axe's `color-contrast` rule.
///
/// A contrast pass reports absence, so "axe found nothing" and "axe looked at
/// nothing" are the same green. Todo 327 is four components' worth of the
/// second, and it survived a `#ddd` plant. This is the difference asserted:
/// axe's result lists the nodes each rule passed, failed and could not decide,
/// and an element with visible text in none of those three was never checked.
pub async fn assert_covers(page: &Page, root: &str, selector: &str) -> Result<usize> {
    inject(page).await?;

    let script = format!(
        r#"(async () => {{
            {UNLOCK}
            const result = await window.axe.run(document.querySelector({root}), {{
                runOnly: {{ type: 'rule', values: ['color-contrast'] }},
            }});

            // Still inside the lift, deliberately: `wanted` below decides what
            // is on screen from `getBoundingClientRect`, and it has to judge
            // that against the same layout axe just judged. The rects are
            // asserted identical across the lift either way.
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

            const wanted = [];
            for (const host of document.querySelectorAll({selector})) {{
                for (const el of [host, ...host.querySelectorAll('*')]) {{
                    // The element that *owns* the text is the one axe reports,
                    // so a wrapper whose text lives in a child is not wanted.
                    // A letter or digit: axe's `ignoreUnicode` strips punctuation-only text and skips it.
                    const owns = Array.from(el.childNodes)
                        .some(n => n.nodeType === Node.TEXT_NODE && /[\p{{L}}\p{{N}}]/u.test(n.textContent));
                    if (!owns) continue;
                    if (el.closest('[aria-hidden=true]')) continue;
                    // Inactive text is exempt from 1.4.3, and axe skips it the same way.
                    if (el.closest('[aria-disabled=true]')) continue;
                    const style = getComputedStyle(el);
                    if (style.display === 'none' || style.visibility === 'hidden'
                        || style.opacity === '0') continue;
                    const r = el.getBoundingClientRect();
                    // Smaller than this is a visually-hidden recipe, which no
                    // sighted user reads and axe rightly skips.
                    if (r.width < 4 || r.height < 4) continue;
                    if (r.bottom <= 0 || r.right <= 0
                        || r.top >= window.innerHeight || r.left >= window.innerWidth) continue;
                    wanted.push(el);
                }}
            }}

            return {{
                moved: __moved,
                wanted: wanted.length,
                missing: wanted.filter(el => !seen.has(el))
                    .map(el => el.outerHTML.slice(0, 200)),
            }};
            {RELOCK}
        }})()"#,
        root = serde_json::to_string(root)?,
        selector = serde_json::to_string(selector)?,
    );

    let coverage: Coverage = page.evaluate(script).await?.into_value()?;
    if let Some(moved) = coverage.moved {
        bail!(
            "lifting the modal scroll lock for the coverage run changed the layout ({moved}). \
             See `UNLOCK` in `passes/contrast.rs` and todo 327."
        );
    }
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

/// A violation we know about and have decided not to fix yet.
///
/// A waiver is **not** a way to silence a failure. It names the todo that owns
/// the decision, and every waived violation is still printed, so a reader sees
/// "knowingly wrong, tracked" rather than nothing at all. A waiver with no todo
/// behind it is a bug being hidden.
pub struct Waiver {
    /// The axe rule id, e.g. `color-contrast`.
    pub rule: &'static str,
    /// Matched against the offending element's html, so a waiver is narrow.
    pub contains: &'static str,
    pub why: &'static str,
}

/// Todo 297: `success` (4.05:1) and `warning` (3.27:1) fail as text at every
/// ramp step. That is a brand decision the Maintainer owns, not a bug the suite
/// should go red on every run until it is settled.
pub const TODO_297: &[Waiver] = &[
    Waiver {
        rule: "color-contrast",
        contains: "success",
        why: "todo 297 - `success` is 4.05:1 as text; brand decision pending",
    },
    Waiver {
        rule: "color-contrast",
        contains: "warning",
        why: "todo 297 - `warning` is 3.27:1 as text; brand decision pending",
    },
];

/// Fail on any violation, printing what axe said rather than a bare count.
pub async fn assert_clean(page: &Page, selector: &str) -> Result<()> {
    assert_clean_except(page, selector, &[]).await
}

/// As `assert_clean`, but tolerating the named waivers.
pub async fn assert_clean_except(page: &Page, selector: &str, waivers: &[Waiver]) -> Result<()> {
    let all = run(page, selector).await?;
    let mut violations = Vec::new();
    let mut waived = Vec::new();

    for violation in all {
        let covered = |node: &Node| {
            waivers
                .iter()
                .find(|w| w.rule == violation.id && node.html.contains(w.contains))
        };
        let (ok, bad): (Vec<_>, Vec<_>) = violation
            .nodes
            .into_iter()
            .partition(|n| covered(n).is_some());
        for node in ok {
            let why = waivers
                .iter()
                .find(|w| w.rule == violation.id && node.html.contains(w.contains))
                .map(|w| w.why)
                .unwrap_or("");
            waived.push(format!("{} - {why}\n      {}", violation.id, node.html));
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
