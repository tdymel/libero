//! Accessibility tree snapshots.
//!
//! The computed tree, as the browser exposes it to assistive technology, not
//! the ARIA attributes scraped off the DOM. A role can be implicit, and an
//! accessible name can come from six places; only the browser knows the answer.
//!
//! ## Scoping: follow the ARIA relations, not DOM containment
//!
//! A popover is portaled to an outlet at the document root, so its listbox is
//! no descendant of its trigger and a subtree walk misses it entirely. What
//! ties them together is `aria-controls` - which is precisely what makes it a
//! popover - so the snapshot follows that edge. This generalises to every
//! portaled component in the library (`codebase/use-popover`).

use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::accessibility::{AxNode, AxNodeId, GetFullAxTreeParams};
use chromiumoxide::cdp::browser_protocol::dom::{
    BackendNodeId, DescribeNodeParams, GetDocumentParams, QuerySelectorParams,
};

/// Render the accessibility subtree under `selector` as stable, indented text.
///
/// Anything volatile is dropped: node ids, and the generated `lsx-<n>` ids that
/// change per render. What is left is role, accessible name, and the states
/// that carry meaning.
pub async fn snapshot(page: &Page, selector: &str) -> Result<String> {
    let backend_id = backend_node_id(page, selector).await?;
    let nodes = page
        .execute(GetFullAxTreeParams::default())
        .await
        .context("get the accessibility tree")?
        .result
        .nodes
        .clone();

    let by_id: HashMap<AxNodeId, &AxNode> = nodes.iter().map(|n| (n.node_id.clone(), n)).collect();
    let root = nodes
        .iter()
        .find(|n| n.backend_dom_node_id == Some(backend_id))
        .with_context(|| format!("no accessibility node for {selector}"))?;

    let mut out = String::new();
    render(root, &by_id, 0, &mut out);

    // Follow `aria-controls` out of the subtree, for a caller whose root does
    // not hold the portal outlet. A target the walk above already rendered is
    // named but not rendered twice - the fixture's own root holds the outlet
    // (`fixtures/src/main.rs`), so for `Suite` that is every target, and the
    // line records the relation rather than the content.
    for target in controlled_selectors(page, selector).await? {
        if let Ok(id) = backend_node_id(page, &target).await
            && let Some(node) = nodes.iter().find(|n| n.backend_dom_node_id == Some(id))
        {
            if contains(page, selector, &target).await? {
                out.push_str(&format!(
                    "--> aria-controls {} (rendered above)\n",
                    stable(&target)
                ));
            } else {
                out.push_str(&format!("--> aria-controls {}\n", stable(&target)));
                render(node, &by_id, 0, &mut out);
            }
        }
    }

    Ok(out)
}

/// Whether the element at `inner` sits inside the one at `outer`.
async fn contains(page: &Page, outer: &str, inner: &str) -> Result<bool> {
    Ok(page
        .evaluate(format!(
            "(() => {{ const o = document.querySelector({}); const i = document.querySelector({}); \
             return !!o && !!i && o.contains(i); }})()",
            serde_json::to_string(outer)?,
            serde_json::to_string(inner)?
        ))
        .await?
        .into_value()?)
}

/// The ids named by any `aria-controls` inside the subtree, as selectors.
///
/// Returned even when the target does not exist, so that a dangling reference
/// shows up in the snapshot as a missing section rather than silently
/// disappearing. Todo 101 was exactly that bug: `aria-activedescendant` named
/// an option that was not in the DOM.
async fn controlled_selectors(page: &Page, selector: &str) -> Result<Vec<String>> {
    let ids: Vec<String> = page
        .evaluate(format!(
            r#"(() => {{
                const root = document.querySelector({});
                if (!root) return [];
                const out = [];
                for (const el of [root, ...root.querySelectorAll('[aria-controls]')]) {{
                    const v = el.getAttribute && el.getAttribute('aria-controls');
                    if (v) for (const id of v.split(/\s+/)) if (id) out.push(id);
                }}
                return [...new Set(out)];
            }})()"#,
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;
    // `[id="..."]` rather than `#id`: an id is allowed characters that a CSS
    // id selector would have to escape, and a caller supplies their own id.
    Ok(ids
        .into_iter()
        .map(|id| format!("[id={}]", serde_json::to_string(&id).unwrap_or_default()))
        .collect())
}

/// Roles dropped from the snapshot.
///
/// `InlineTextBox` is a line-box artefact: the browser emits one per rendered
/// line, so the same text produces a different tree at 1280px and at 390px, and
/// re-wrapping churns the baseline for no accessibility reason at all. It
/// carries nothing a screen reader user experiences that its `StaticText`
/// parent does not.
const NOISE: &[&str] = &["InlineTextBox"];

/// Replace generated ids with a placeholder.
///
/// `use_id` hands out `lsx-<n>` in render order, so the same component is
/// `lsx-0` alone in a fixture and `lsx-7` once anything is added above it. The
/// snapshot is about the shape of the tree, not about the counter, and a
/// baseline that churns on an unrelated addition is a baseline people learn to
/// accept without reading.
fn stable(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(at) = rest.find("lsx-") {
        out.push_str(&rest[..at + 4]);
        rest = &rest[at + 4..];
        let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        if digits > 0 {
            out.push('N');
            rest = &rest[digits..];
        }
    }
    out.push_str(rest);
    out
}

fn render(node: &AxNode, by_id: &HashMap<AxNodeId, &AxNode>, depth: usize, out: &mut String) {
    if node.ignored {
        // An ignored node contributes nothing to AT, but its children can.
        for child in node.child_ids.iter().flatten() {
            if let Some(child) = by_id.get(child) {
                render(child, by_id, depth, out);
            }
        }
        return;
    }

    let role = node
        .role
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .and_then(|v| v.as_str())
        .unwrap_or("none");

    if NOISE.contains(&role) {
        return;
    }
    let name = node
        .name
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // `value` is a **field** of the node, not one of its `properties`.
    //
    // This is why filtering for a property called `valuenow` did nothing: the
    // list gained the word and the output never changed. A slider's whole state
    // lives here, so a baseline without it stays green on a slider that has
    // stopped reporting where it is - the exact regression the snapshot exists
    // to catch.
    let value = node
        .value
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .unwrap_or_else(|| v.to_string())
        })
        .filter(|v| !v.is_empty());

    out.push_str(&"  ".repeat(depth));
    out.push_str(role);
    if !name.is_empty() {
        out.push_str(&format!(" \"{name}\""));
    }
    if let Some(value) = value {
        out.push_str(&format!(" = {value}"));
    }
    for state in states(node) {
        out.push_str(&format!(" [{state}]"));
    }
    out.push('\n');

    for child in node.child_ids.iter().flatten() {
        if let Some(child) = by_id.get(child) {
            render(child, by_id, depth + 1, out);
        }
    }
}

/// States where `false` is **not** the same as absent, so they are printed
/// either way.
///
/// A combobox that says `aria-expanded="false"` and one that says nothing at
/// all are different components to a screen reader, and only one of them is
/// correct. Dropping false here would have hidden that distinction in the very
/// first snapshot this suite took.
const KEEP_WHEN_FALSE: &[&str] = &["expanded", "checked", "pressed", "selected"];

/// The properties worth pinning. Deliberately a short list: every property
/// included is one more reason for a snapshot to churn on an unrelated change,
/// and a noisy snapshot gets accepted without being read.
fn states(node: &AxNode) -> Vec<String> {
    const KEEP: &[&str] = &[
        "expanded",
        "selected",
        "checked",
        "disabled",
        "required",
        "invalid",
        "pressed",
        "level",
        "valuemin",
        "valuemax",
        "valuetext",
        "haspopup",
        "live",
        "atomic",
        "busy",
        "modal",
        "readonly",
        "multiselectable",
    ];

    let mut out = Vec::new();
    for prop in node.properties.iter().flatten() {
        let name = format!("{:?}", prop.name).to_lowercase();
        if !KEEP.contains(&name.as_str()) {
            continue;
        }
        match prop.value.value.as_ref() {
            // Chromium reports some of these as the *string* "false" rather
            // than a boolean, so both spellings have to be recognised or the
            // tree fills with `invalid=false` style noise.
            Some(v)
                if (v.as_bool() == Some(false) || v.as_str() == Some("false"))
                    && !KEEP_WHEN_FALSE.contains(&name.as_str()) => {}
            Some(v) => {
                let text = v
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| v.to_string());
                // Chromium reports some properties with an empty value rather
                // than omitting them - `valuetext` on a slider with no custom
                // label is the case that surfaced this. Printing `valuetext=`
                // is noise that reads like a defect.
                if text.is_empty() {
                    continue;
                }
                if text == "true" {
                    out.push(name);
                } else {
                    out.push(format!("{name}={text}"));
                }
            }
            None => {}
        }
    }
    out.sort();
    out
}

async fn backend_node_id(page: &Page, selector: &str) -> Result<BackendNodeId> {
    let doc = page
        .execute(GetDocumentParams::default())
        .await
        .context("get the document")?;
    let found = page
        .execute(QuerySelectorParams::new(doc.result.root.node_id, selector))
        .await
        .with_context(|| format!("query {selector}"))?;
    if *found.result.node_id.inner() == 0 {
        bail!("no element matches {selector}");
    }
    let described = page
        .execute(
            DescribeNodeParams::builder()
                .node_id(found.result.node_id)
                .build(),
        )
        .await
        .with_context(|| format!("describe {selector}"))?;
    Ok(described.result.node.backend_node_id)
}

#[cfg(test)]
mod tests {
    use super::stable;

    #[test]
    fn generated_ids_are_normalised() {
        assert_eq!(stable("[id=\"lsx-0-listbox\"]"), "[id=\"lsx-N-listbox\"]");
        assert_eq!(stable("lsx-12-option-3"), "lsx-N-option-3");
        // A caller's own id is not ours to rewrite.
        assert_eq!(stable("[id=\"city-field\"]"), "[id=\"city-field\"]");
    }
}
