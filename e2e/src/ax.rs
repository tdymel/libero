//! Snapshots of the browser's computed accessibility tree, not scraped ARIA attributes.
//! Follows `aria-controls` out of the subtree, since popovers are portaled (`codebase/use-popover`).

use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::accessibility::{AxNode, AxNodeId, GetFullAxTreeParams};
use chromiumoxide::cdp::browser_protocol::dom::{
    BackendNodeId, DescribeNodeParams, GetDocumentParams, Node, QuerySelectorParams,
};

/// Renders the accessibility subtree under `selector` as indented text: role, name
/// and meaningful states, without volatile ids.
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

    let current = current_by_node(page).await?;
    let mut out = String::new();
    render(root, &by_id, &current, 0, &mut out);

    // Targets already rendered are only named; missing ones are recorded (review 7, E7).
    for target in controlled_selectors(page, selector).await? {
        let Ok(id) = backend_node_id(page, &target).await else {
            out.push_str(&format!(
                "--> aria-controls {} (missing)\n",
                stable(&target)
            ));
            continue;
        };
        let Some(node) = nodes.iter().find(|n| n.backend_dom_node_id == Some(id)) else {
            out.push_str(&format!(
                "--> aria-controls {} (not in the accessibility tree)\n",
                stable(&target)
            ));
            continue;
        };
        if contains(page, selector, &target).await? {
            out.push_str(&format!(
                "--> aria-controls {} (rendered above)\n",
                stable(&target)
            ));
        } else {
            out.push_str(&format!("--> aria-controls {}\n", stable(&target)));
            render(node, &by_id, &current, 0, &mut out);
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

/// The ids named by any `aria-controls` in the subtree, as selectors. Dangling ones
/// are kept, so the snapshot shows them as `(missing)` (todo 101).
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
    // `[id="..."]`, not `#id`: caller ids may hold characters `#` would need escaped.
    Ok(ids
        .into_iter()
        .map(|id| format!("[id={}]", serde_json::to_string(&id).unwrap_or_default()))
        .collect())
}

/// Roles dropped from the snapshot. `InlineTextBox` is one per wrapped line, so it churns with the viewport.
const NOISE: &[&str] = &["InlineTextBox"];

/// Replaces generated `lsx-<n>` ids, which shift whenever something renders above.
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

fn render(
    node: &AxNode,
    by_id: &HashMap<AxNodeId, &AxNode>,
    current: &HashMap<BackendNodeId, String>,
    depth: usize,
    out: &mut String,
) {
    if node.ignored {
        // An ignored node contributes nothing to AT, but its children can.
        for child in node.child_ids.iter().flatten() {
            if let Some(child) = by_id.get(child) {
                render(child, by_id, current, depth, out);
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
        .unwrap_or("")
        // Chrome versions disagree on a name's edge whitespace (a label's " A").
        .trim();

    // `value` is a field of the node, not a property: filtering for `valuenow` found nothing.
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
    let description = node
        .description
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty());

    let mut states = states(node);
    if let Some(token) = node.backend_dom_node_id.and_then(|id| current.get(&id)) {
        states.push(format!("current={token}"));
        states.sort();
    }

    // A bare `generic` says nothing to AT, and Chrome versions disagree on
    // which wrappers they expose (an `overflow: hidden` div), so it is flattened.
    if role == "generic"
        && name.is_empty()
        && value.is_none()
        && description.is_none()
        && states.is_empty()
    {
        for child in node.child_ids.iter().flatten() {
            if let Some(child) = by_id.get(child) {
                render(child, by_id, current, depth, out);
            }
        }
        return;
    }

    out.push_str(&"  ".repeat(depth));
    out.push_str(role);
    if !name.is_empty() {
        out.push_str(&format!(" \"{name}\""));
    }
    if let Some(value) = value {
        out.push_str(&format!(" = {value}"));
    }
    for state in states {
        out.push_str(&format!(" [{state}]"));
    }
    out.push('\n');
    // A detached error text changes no state, only the description (1798).
    if let Some(description) = description {
        out.push_str(&"  ".repeat(depth + 1));
        out.push_str(&format!("description \"{description}\"\n"));
    }

    for child in node.child_ids.iter().flatten() {
        if let Some(child) = by_id.get(child) {
            render(child, by_id, current, depth + 1, out);
        }
    }
}

/// States where `false` differs from absent to a screen reader, so printed either way.
const KEEP_WHEN_FALSE: &[&str] = &["expanded", "checked", "pressed", "selected"];

/// The properties worth pinning. Kept short: each one is another reason to churn.
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
        "orientation",
        "autocomplete",
        "roledescription",
        "keyshortcuts",
    ];

    let mut out = Vec::new();
    for prop in node.properties.iter().flatten() {
        let name = format!("{:?}", prop.name).to_lowercase();
        if !KEEP.contains(&name.as_str()) {
            continue;
        }
        match prop.value.value.as_ref() {
            // Chromium sends some as the string "false", not a boolean.
            Some(v)
                if (v.as_bool() == Some(false) || v.as_str() == Some("false"))
                    && !KEEP_WHEN_FALSE.contains(&name.as_str()) => {}
            Some(v) => {
                let text = v
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| v.to_string());
                // Chromium sends some empty rather than omitting them (`valuetext` on a slider).
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

/// Each element's `aria-current` token, by backend node id. The CDP tree
/// reports no `current` property, so this one state comes off the DOM (todo 600).
async fn current_by_node(page: &Page) -> Result<HashMap<BackendNodeId, String>> {
    fn walk(node: &Node, out: &mut HashMap<BackendNodeId, String>) {
        let attributes = node.attributes.as_deref().unwrap_or_default();
        for pair in attributes.chunks(2) {
            if let [name, value] = pair
                && name == "aria-current"
                && !value.is_empty()
                && value != "false"
            {
                out.insert(node.backend_node_id, value.clone());
            }
        }
        let nested = [&node.children, &node.shadow_roots];
        for child in nested.into_iter().flatten().flatten() {
            walk(child, out);
        }
    }

    let doc = page
        .execute(GetDocumentParams::builder().depth(-1).pierce(true).build())
        .await
        .context("get the document")?;
    let mut out = HashMap::new();
    walk(&doc.result.root, &mut out);
    Ok(out)
}

/// The computed accessible description of `selector`, or `""`.
pub async fn description(page: &Page, selector: &str) -> Result<String> {
    let backend_id = backend_node_id(page, selector).await?;
    let nodes = page
        .execute(GetFullAxTreeParams::default())
        .await
        .context("get the accessibility tree")?
        .result
        .nodes
        .clone();
    let node = nodes
        .iter()
        .find(|n| n.backend_dom_node_id == Some(backend_id))
        .with_context(|| format!("no accessibility node for {selector}"))?;
    Ok(node
        .description
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string())
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
