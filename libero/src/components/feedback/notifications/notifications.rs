use dioxus::{core::current_scope_id, prelude::*};

use super::{
    data::NotificationLive,
    hotkey::use_hotkey,
    item::{DrawRef, ItemProps},
    stack::NotificationStack,
    store::NotificationStore,
};
use crate::{
    components::{common::Input, layout::Box},
    hooks::{use_localization, use_portal_slot, use_theme},
    localization::fill,
    sx::{StaticSx, sx},
    theme::{AutoClose, Placement},
};

/// A contained host's box: the stacks' positioned ancestor.
static CONTAINED_SX: StaticSx = StaticSx::new(|| sx().position("relative"));

/// The landmark around every stack. It covers the host, so a stack anchors to
/// the host even where `position` resolves against the parent (Blitz, todo 682).
static LANDMARK_SX: StaticSx =
    StaticSx::new(|| sx().position("absolute").inset("0").pointer_events("none"));

/// The host where notifications render. Render it once; a `contained` one has
/// its own queue.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Notifications, use_notifications};
/// # fn app() -> Element {
/// # rsx! {
/// Notifications { contained: true,
///     SaveButton {} // its `use_notifications()` shows them in this box
/// }
/// # } }
/// # #[component] fn SaveButton() -> Element { rsx! {} }
/// ```
///
/// Docs: <https://libero-ui.dev/feedback/notifications>
#[component]
pub fn Notifications(
    /// The default stack. A change moves only notifications shown after it.
    #[props(default, into)]
    placement: Input<Placement>,
    /// Shown at once per stack; the rest wait.
    #[props(default)]
    limit: Option<usize>,
    /// Unless a notification says otherwise.
    #[props(default)]
    auto_close: Option<AutoClose>,
    /// Draws inside its own box, with a queue of its own. Read once, on mount.
    #[props(default)]
    contained: bool,
    /// Focuses the newest notification from anywhere.
    #[props(default = Key::F8)]
    hotkey: Key,
    /// Rendered inside a contained host, before its stacks.
    #[props(default)]
    children: Option<Element>,
) -> Element {
    let theme = use_theme();
    let localization = use_localization();
    let store = use_hook(|| {
        if contained {
            provide_context(NotificationStore::new(current_scope_id()))
        } else {
            try_consume_context::<NotificationStore>().unwrap_or_else(|| {
                dioxus::core::provide_root_context(NotificationStore::new(ScopeId::ROOT))
            })
        }
    });
    use_hotkey(store, hotkey.clone());
    let key_name = match &hotkey {
        Key::Character(text) => text.to_uppercase(),
        key => key.to_string(),
    };
    let region_label = fill(localization.notifications.region, &[("key", &key_name)]);

    let host_placement = placement.copied_or(theme.notifications.placement);
    let limit = limit.unwrap_or(theme.notifications.limit);
    let auto_close = auto_close.unwrap_or(theme.notifications.auto_close);
    let exit_ms = theme.notifications.transition_duration;

    let entries = store.entries.read();
    // Every placement, always: a live region must exist before content is added,
    // or nothing is announced (todo 447).
    let mut drawn = Vec::new();
    let stacks = Placement::ALL.iter().map(|&placement| {
        let items = entries
            .iter()
            .filter(|entry| {
                let stack = entry.drawn_in.get().or(entry.placement);
                stack.unwrap_or(host_placement) == placement
            })
            .take(limit)
            .inspect(|entry| entry.drawn_in.set(Some(placement)))
            .map(|entry| ItemProps {
                store,
                id: entry.id,
                draw: DrawRef(entry.draw.clone()),
                auto_close: match entry.auto_close.unwrap_or(auto_close) {
                    AutoClose::Never => None,
                    AutoClose::After(ms) => Some(ms),
                },
                leaving: entry.leaving,
                exit_ms,
                live: entry.live,
            })
            .collect::<Vec<_>>();
        let (assertive, polite): (Vec<_>, Vec<_>) = items
            .into_iter()
            .partition(|item| item.live == NotificationLive::Assertive);
        drawn.push(
            assertive
                .iter()
                .chain(&polite)
                .map(|item| item.id)
                .collect(),
        );

        rsx! {
            NotificationStack {
                key: "{placement.as_str()}",
                placement,
                fixed: !contained,
                assertive,
                polite,
            }
        }
    });
    let stacks = stacks.collect::<Vec<_>>();
    let idle = drawn.iter().all(Vec::is_empty);
    let mut stored = store.drawn;
    stored.set(drawn);
    // One landmark for every stack: nine would crowd the landmark list.
    let content = rsx! {
        Box {
            framework_sx: &LANDMARK_SX,
            role: "region",
            "aria-label": region_label,
            {stacks.into_iter()}
        }
    };
    drop(entries);

    let slot = use_portal_slot();
    if contained {
        slot.show(None);
        return rsx! {
            Box { framework_sx: &CONTAINED_SX,
                "data-notifications-host": store.id.to_string(),
                {children}
                {content}
            }
        };
    }
    // The empty regions stay mounted; the outlet need not follow a scroll for them.
    slot.show_idle(content, idle);
    rsx! {}
}
