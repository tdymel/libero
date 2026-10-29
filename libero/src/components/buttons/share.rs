use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        buttons::{ActionIcon, Button},
        common::{Glyph, Input, States, Variant, base_props},
        data_display::Icon,
    },
    context::IconSlot,
    hooks::{listener, use_id, use_localization, use_share, use_silent_focus_out},
    platform::{ShareData, ShareFile, ShareOutcome},
    sx::ThemeAwareValue,
};

base_props! {
    pub struct ShareProps {
        /// A title, used as the subject where the target has one.
        #[props(default, into)]
        title: Option<String>,
        /// A message.
        #[props(default, into)]
        text: Option<String>,
        /// The link; what a target without a share sheet copies, else `text`, else `title`.
        #[props(default, into)]
        url: Option<String>,
        /// Files, shared where the target takes them; elsewhere the rest still goes.
        #[props(default)]
        files: Vec<ShareFile>,
        /// Unset with `color` also unset, no chrome of its own.
        #[props(default, into)]
        variant: Input<Variant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// With `children`, only a size word applies.
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Replaces the localized name, e.g. "Share this article".
        #[props(default, into)]
        aria_label: Option<String>,
        /// What a press shares, read after the name, e.g. "Link to this page".
        #[props(default, into)]
        label: Option<String>,
        #[props(default)]
        disabled: Option<bool>,
        /// Called with the outcome once the share settled.
        #[props(default)]
        onshare: Option<EventHandler<ShareOutcome>>,
        /// A visible label: renders a `Button` with the share glyph instead of an icon-only one.
        #[props(default)]
        children: Option<Element>,
    }
}

/// A button that opens the system share sheet, or copies the link where
/// there is none, and announces the result.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::Share;
/// # fn app() -> Element {
/// rsx! {
///     Share { url: "https://libero-ui.dev", title: "Libero", variant: "outlined" }
///     Share { url: "https://libero-ui.dev", "Share" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/share>
#[component]
pub fn Share(props: ShareProps) -> Element {
    let mut share = use_share();
    let labels = use_localization().share;
    // Blitz's Tab fires no `blur`: the same reset, from the silent move.
    let silent = use_silent_focus_out(move || {
        let mut share = share;
        share.reset();
    });
    let label_id = use_id();
    let described = props.label.as_ref().map(|_| label_id());
    let mut attributes = props.attributes.clone();
    attributes.extend(silent.map(|silent| listener("onmounted", silent.mount())));
    let pending = share.pending();
    let states: Input<States> = props
        .states
        .clone()
        .unwrap_or_default()
        .with("pending", pending)
        .into();
    let aria_label = props
        .aria_label
        .clone()
        .filter(|label| !label.trim().is_empty());

    let data = ShareData {
        title: props.title.clone(),
        text: props.text.clone(),
        url: props.url.clone(),
        files: props.files.clone(),
    };
    let onshare = props.onshare;
    // Reset first, so a second fallback copy re-announces rather than repeating unchanged text.
    let onclick = move |_: MouseEvent| {
        share.reset();
        share.share_then(data.clone(), move |outcome| {
            if let Some(onshare) = onshare {
                onshare.call(outcome);
            }
        });
    };
    let glyph = if share.copied() {
        rsx! { Glyph { slot: IconSlot::Check, icon: lucide::check::outlined } }
    } else if share.failed() {
        rsx! { Glyph { slot: IconSlot::CopyFailed, icon: lucide::circle_alert::outlined } }
    } else {
        rsx! { Glyph { slot: IconSlot::Share, icon: lucide::share_2::outlined } }
    };

    // One item for a `ButtonGroup`: its trailing status span would be the group's last child.
    rsx! {
        span { style: "display: contents",
            if let Some(children) = props.children.clone() {
                Button {
                    variant: props.variant.clone(),
                    color: props.color.clone(),
                    size: match props.size.as_ref() {
                        Some(ThemeAwareValue::Size(size)) => Input::Value(*size),
                        _ => Input::None,
                    },
                    radius: props.radius.clone(),
                    disabled: props.disabled,
                    aria_label,
                    aria_describedby: described,
                    class: props.class.clone(),
                    sx: props.sx.clone(),
                    states,
                    attributes,
                    icon: rsx! {
                        Icon { variant: "standard", size: "sm", color: "inherit", {glyph} }
                    },
                    onclick,
                    onmouseleave: move |_| share.reset(),
                    // A keyboard or touch user never leaves with a mouse.
                    onblur: move |_| share.reset(),
                    {children}
                }
            } else {
                ActionIcon {
                    aria_label: aria_label.unwrap_or_else(|| labels.share.to_string()),
                    variant: props.variant.clone(),
                    color: props.color.clone(),
                    size: props.size.clone(),
                    radius: props.radius.clone(),
                    disabled: props.disabled,
                    aria_describedby: described,
                    class: props.class.clone(),
                    sx: props.sx.clone(),
                    states,
                    attributes,
                    onclick,
                    onmouseleave: move |_| share.reset(),
                    onblur: move |_| share.reset(),
                    {glyph}
                }
            }
            // `hidden`: a description only, not read again in browse mode.
            if let Some(label) = props.label.as_deref() {
                span { id: label_id, hidden: true, "{label}" }
            }
            // Always mounted, so a screen reader is watching it when the text arrives.
            VisuallyHidden { role: "status",
                if share.copied() {
                    {labels.copied}
                } else if share.failed() {
                    {labels.share_failed}
                }
            }
        }
    }
}
