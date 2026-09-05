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

/// Run axe over the subtree at `selector`.
pub async fn run(page: &Page, selector: &str) -> Result<Vec<Violation>> {
    let injected: bool = page
        .evaluate("typeof window.axe !== 'undefined'")
        .await?
        .into_value()?;
    if !injected {
        page.evaluate(axe_source()?).await?;
    }

    let script = format!(
        r#"(async () => {{
            const result = await window.axe.run(document.querySelector({}), {{
                runOnly: {{ type: 'rule', values: {} }},
            }});
            return result.violations.map(v => ({{
                id: v.id,
                help: v.help,
                nodes: v.nodes.map(n => ({{
                    html: n.html,
                    failure_summary: n.failureSummary || null,
                }})),
            }}));
        }})()"#,
        serde_json::to_string(selector)?,
        serde_json::to_string(RULES)?,
    );

    let violations = page.evaluate(script).await?.into_value()?;
    Ok(violations)
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
