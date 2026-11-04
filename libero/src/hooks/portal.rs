use dioxus::prelude::*;

use crate::context::{PortalEntry, PortalHost, ThemeScopeContext};
use crate::sx::{SCHEME_ATTRIBUTE, THEME_SCOPE_ATTRIBUTE};
use crate::theme::Theme;
use crate::utils::unique_id;

/// One component's claim on a slot in `PortalOutlet`, written through
/// [`show`](Self::show) after its hooks have run.
#[derive(Clone, Copy)]
pub(crate) struct PortalSlot {
    // Not a `Signal`: read once per entry, a signal made many portals quadratic.
    id: u64,
    host: PortalHost,
    scope: Option<ThemeScopeContext>,
}

impl PortalSlot {
    /// Publishes `content`, escaping any ancestor stacking/clipping context.
    /// Rendered, not a closure: `PortalOutlet` would read your signals cross-scope.
    pub(crate) fn show(&self, content: Option<Element>) {
        let content = content.map(|content| self.scoped(content));
        let mut host = self.host;
        // A closed slot re-rendered closed: a write would re-render the
        // outlet for nothing, once per closed `Select` or `Menu` on the page.
        let unchanged = content.is_none()
            && host
                .entries
                .peek()
                .iter()
                .any(|entry| entry.id == self.id && entry.render.is_none());
        if unchanged {
            return;
        }
        let mut entries = host.entries.write();
        match entries.iter_mut().find(|entry| entry.id == self.id) {
            Some(entry) => entry.render = content,
            None => entries.push(PortalEntry {
                id: self.id,
                render: content,
            }),
        }
    }
}

impl PortalSlot {
    /// `content` in a box carrying the nearest [`ThemeScope`](crate::ThemeScope)'s theme, for its
    /// CSS and for `use_theme` inside.
    fn scoped(&self, content: Element) -> Element {
        let Some(scope) = self.scope else {
            return content;
        };
        let Some(name) = scope.name.cloned() else {
            return content;
        };
        let theme = scope.theme.cloned();
        let scheme = theme.map_or(dioxus::core::AttributeValue::None, |theme| {
            dioxus::core::AttributeValue::Text(theme.surface.color_scheme().to_string())
        });
        let attribute = vec![
            Attribute::new(
                THEME_SCOPE_ATTRIBUTE,
                dioxus::core::AttributeValue::Text(name.clone()),
                None,
                false,
            ),
            Attribute::new(SCHEME_ATTRIBUTE, scheme, None, false),
        ];
        // `contents`: the box must not become the overlay's containing block or take a line.
        rsx! {
            div { display: "contents", ..attribute,
                ScopeContext { name, theme, {content} }
            }
        }
    }
}

/// Provides the slot's scope to the portalled content, which mounts under the outlet.
#[component]
fn ScopeContext(
    name: String,
    #[props(!optional)] theme: Option<&'static Theme>,
    children: Element,
) -> Element {
    // Owned here: the slot's memos and the props' signals belong to scopes outside the outlet.
    let name = use_memo(use_reactive!(|name| Some(name)));
    let theme = use_memo(use_reactive!(|theme| theme));
    use_context_provider(|| ThemeScopeContext { name, theme });
    children
}

/// Claims a portal slot for this component, and releases it on drop.
pub(crate) fn use_portal_slot() -> PortalSlot {
    let host = use_context::<PortalHost>();
    let id = use_hook(unique_id);
    let scope = try_use_context::<ThemeScopeContext>();

    use_drop(move || {
        let mut host = host;
        host.entries.write().retain(|entry| entry.id != id);
    });

    PortalSlot { id, host, scope }
}

/// Renders `content` via `PortalOutlet`, like [`PortalSlot::show`].
/// Re-registers each call; deregisters on drop.
pub(crate) fn use_portal(content: Option<Element>) {
    use_portal_slot().show(content);
}
