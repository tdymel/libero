use dioxus::{dioxus_core::AttributeValue, prelude::*};
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{
            Glyph, HtmlTag, Input, Part, States, Variables, attr, base_props, input_from_str,
            names_itself, parts_enum, use_name_warning, variables,
        },
        layout::{Box, paper_sx, use_box},
        typography::Title,
    },
    context::{Dismiss, IconSlot, ModalContext},
    hooks::{current_localization, use_id, use_theme},
    str_enum::str_enum,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, DIALOG_DEFAULT_SIZE, DIALOG_SIZE, PAPER_RADIUS, Size, SizeCss},
    utils::warn,
};

const DIALOG_RADIUS_VAR: CssVar = CssVar::new("--lsx-dialog-radius");

// `Paper`'s chrome, plus what makes it a dialog.
static DIALOG_BASE_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        // Back on: `Modal`'s frame lets backdrop clicks fall through.
        .pointer_events("auto")
        // A flex item shrinks to content, so `size` would only cap, not fill.
        // No margin: with `width: 100%` it overflows a container (2561).
        .width("100%")
        .max_width(DIALOG_DEFAULT_SIZE.overridable())
        .padding("lg")
        .border_radius(DIALOG_RADIUS_VAR.value_or(PAPER_RADIUS.value()))
        .box_shadow(SizeCss::SHADOW.value(Size::Xl))
        .when("full-screen-always", full_screen_sx())
        .media(
            format!("not all and (min-width: {})", Size::Sm.breakpoint_value()),
            sx().when("full-screen-phone", full_screen_sx()),
        )
});

/// Edge to edge over `Modal`'s frame: its negative margins cancel the frame's `md`
/// padding, and `stretch` and `flex-grow` fill it. Plain CSS, so no centred first frame.
fn full_screen_sx() -> Sx {
    sx().margin(format!("calc(-1 * {})", SizeCss::SPACING.value(Size::Md)))
        .align_self("stretch")
        .flex_grow("1")
        .max_width("none")
        .border_radius("0")
        .box_shadow("none")
}

str_enum! {
    /// When a [`Dialog`] in a modal fills the screen instead of centring.
    pub enum DialogFullScreen {
        /// Centred at every width.
        #[default]
        Never = "never",
        /// Below the `sm` breakpoint, where a centred dialog leaves slivers of page.
        Phone = "phone",
        Always = "always",
    }
}

input_from_str!(DialogFullScreen);

static DIALOG_HEADER_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("flex-start")
        // Not `space-between`: a lone close button would sit top-left.
        .justify_content("flex-end")
        .gap("sm")
        .margin_bottom("md")
        // The 30px close box reaches into the padding, its glyph where a bare one sat.
        .selector(
            "& > button",
            sx().margin_top("-5px").margin_inline_end("-5px"),
        )
});

static DIALOG_HEADER_TITLE_SX: StaticSx = StaticSx::new(|| {
    sx().margin("0")
        .flex("1")
        // `overflow-wrap` leaves min-content alone, so a long word would push past the dialog.
        .min_width("0")
});

fn dialog_variables(props: &DialogProps, default_radius: Option<Size>) -> Variables {
    variables()
        .with(
            DIALOG_RADIUS_VAR,
            props
                .radius
                .as_ref()
                .cloned()
                .or(default_radius.map(ThemeAwareValue::from))
                .and_then(|radius| radius.resolve(Some(SizeCss::RADIUS))),
        )
        .with(
            DIALOG_SIZE.override_var(),
            props.size.resolve(Some(DIALOG_SIZE)),
        )
}

parts_enum! {
    /// [`Dialog`]'s inner parts, for its `parts` prop. Matched as direct children,
    /// so a dialog in the content keeps its own styles.
    pub enum DialogPart {
        /// The row holding the title and the close button.
        Header = "header" => "& > [data-slot='header']",
        Title = "title" => "& > [data-slot='header'] > [data-slot='title']",
        Close = "close" => "& > [data-slot='header'] > [data-slot='close']",
    }
}

base_props! {
    parts(DialogPart);
    pub struct DialogProps {
        /// Replaces the title as the name; keep the title's words in it (WCAG 2.5.3).
        #[props(default, into)]
        aria_label: Option<String>,
        /// Heading, and the accessible name unless `aria_label` overrides it.
        #[props(default, into)]
        title: Option<String>,
        /// Defaults to on inside a modal, or when `onclose` is set.
        #[props(default)]
        close_button: Option<bool>,
        /// The close button outside a modal; inside one it closes the modal.
        #[props(default)]
        onclose: Option<EventHandler<()>>,
        /// Inside a modal: whether Escape, the backdrop, Back, a bottom drawer's swipe or the
        /// header close button (`Dismiss::CloseButton`) may close it; `false` keeps it open.
        /// Unset, all close, but an `alertdialog` ignores the backdrop.
        #[props(default)]
        ondismiss: Option<Callback<Dismiss, bool>>,
        /// The close button's accessible name, e.g. "Close cart".
        #[props(default, into)]
        close_label: Option<String>,
        /// A size step on the radius scale, or any CSS, as `radius: "0"`. Unset,
        /// `theme.dialog.radius`, else `Paper`'s.
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Unset, `theme.dialog.size`.
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        /// Inside a modal, fills the screen edge to edge instead of centring: `"phone"`
        /// below the `sm` breakpoint, `"always"` at every width. Unset, centred.
        #[props(default, into)]
        full_screen: Input<DialogFullScreen>,
        /// Layered onto Dialog's own - e.g. `Drawer`'s anchor/size vars.
        #[props(default, into)]
        variables: Input<Variables>,
        children: Element,
    }
}

/// A dialog surface with a title and close button. Inside a modal it closes
/// the modal; it does not position itself.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::Dialog;
/// # fn app() -> Element {
/// let mut open = use_signal(|| true);
/// rsx! {
///     if open() {
///         Dialog { title: "Tip", onclose: move |_| open.set(false), "Drag to reorder." }
///     }
/// }
/// # }
/// ```
///
/// A confirmation is an `alertdialog`, described by its message; the backdrop
/// does not close it. `ondismiss` keeps a form with unsaved input open, the
/// header close button included:
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, Dialog, Text};
/// # use libero::hooks::{ModalScope, use_modal};
/// # fn app() -> Element {
/// let confirm = use_modal(|s: ModalScope<(), bool>| rsx! {
///     Dialog { title: "Delete file?", role: "alertdialog", aria_describedby: "delete-message",
///         Text { id: "delete-message", "notes.md goes for good." }
///         Button { onclick: move |_| s.resolve(true), "Delete" }
///     }
/// });
/// let mut draft = use_signal(String::new);
/// let note = use_modal(move |_: ModalScope<()>| rsx! {
///     Dialog { title: "New note", ondismiss: move |_| draft.read().is_empty(),
///         textarea { aria_label: "Note", oninput: move |e| draft.set(e.value()) }
///     }
/// });
/// # rsx! {}
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/overlay/dialog>
#[component]
pub fn Dialog(props: DialogProps) -> Element {
    let modal = try_use_context::<ModalContext>().filter(ModalContext::is_modal);
    let is_modal = modal.is_some();
    let close_button = props
        .close_button
        .unwrap_or(is_modal || props.onclose.is_some());
    let title_id = use_id();
    // A blank one names nothing: no empty heading, and it does not mask a good title.
    let aria_label = props
        .aria_label
        .clone()
        .filter(|label| !label.trim().is_empty());
    let title = props.title.clone().filter(|title| !title.trim().is_empty());
    let spread_name = names_itself(&props.attributes);
    use_name_warning(
        aria_label.is_some() || title.is_some() || spread_name,
        "Dialog: no `title`, `aria_label` or `aria-labelledby`, so it is announced as just \
         \"dialog\".",
    );
    let dead_button = close_button && !is_modal && props.onclose.is_none();
    use_hook(move || {
        if dead_button {
            warn("Dialog: `close_button` outside a modal and no `onclose`, so it closes nothing.");
        }
    });
    // Stable for the memoized header; swaps in this render's `onclose`.
    let onclose = props.onclose;
    let close = use_callback(move |()| match (modal, &onclose) {
        (Some(modal), _) => modal.dismiss(Dismiss::CloseButton),
        (None, Some(onclose)) => onclose.call(()),
        (None, None) => {}
    });
    // A spread `role: "alertdialog"` survives; any other role gives way to `dialog`.
    let alert = props.attributes.iter().any(|attribute| {
        attribute.name == "role"
            && matches!(&attribute.value, AttributeValue::Text(role) if role.trim() == "alertdialog")
    });
    let ondismiss = props.ondismiss;
    let guard = use_callback(move |reason: Dismiss| match ondismiss {
        Some(ondismiss) => ondismiss.call(reason),
        // APG: an alert dialog expects an answer, so a stray click does not count as one.
        None => !(alert && reason == Dismiss::Backdrop),
    });
    let slot = modal.and_then(|modal| modal.dismiss_guard);
    use_hook(move || {
        if let Some(mut slot) = slot {
            slot.set(Some(guard));
        }
    });
    use_drop(move || {
        if let Some(mut slot) = slot
            && let Ok(mut slot) = slot.try_write()
            && *slot == Some(guard)
        {
            *slot = None;
        }
    });
    let variables: Input<Variables> = dialog_variables(&props, use_theme().dialog.radius)
        .merge(props.variables.unwrap_or_default())
        .into();
    // Outside a modal there is no frame to fill.
    let full_screen = match is_modal {
        true => props.full_screen.copied_or_default(),
        false => DialogFullScreen::Never,
    };
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("full-screen-phone", full_screen == DialogFullScreen::Phone)
        .with(
            "full-screen-always",
            full_screen == DialogFullScreen::Always,
        )
        .into();

    // Pushed after the caller's, to win a duplicate name.
    let mut attributes = props.attributes;
    if !alert {
        attributes.push(attr("role", "dialog"));
    }
    if is_modal {
        attributes.push(attr("aria-modal", "true"));
        // Focusable by script and by a click on its text, so focus and the
        // modal's keys stay inside when nothing in it takes focus (APG).
        attributes.push(attr("tabindex", "-1"));
    }
    if aria_label.is_none() && title.is_some() && !spread_name {
        attributes.push(attr("aria-labelledby", title_id()));
    }
    if let Some(aria_label) = aria_label {
        attributes.push(attr("aria-label", aria_label));
    }

    let mut children = Vec::with_capacity(2);
    if title.is_some() || close_button {
        children.push(rsx! {
            DialogHeader {
                title,
                title_id,
                close_button,
                close_label: props.close_label,
                close,
            }
        });
    }
    children.push(props.children);

    // `Paper`'s body without its scope, which would re-render with every dialog render.
    use_box()
        .framework_sx(&DIALOG_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .variables(&variables)
        .prepare()
        .render(HtmlTag::Div, attributes, children)
}

/// Its own scope, so a dialog re-rendered for its `children` skips the header.
#[component]
fn DialogHeader(
    title: Option<String>,
    title_id: Signal<String>,
    close_button: bool,
    close_label: Option<String>,
    close: Callback<()>,
) -> Element {
    rsx! {
        Box { framework_sx: &DIALOG_HEADER_SX, "data-slot": DialogPart::Header.slot(),
            if let Some(title) = title {
                Title {
                    "data-slot": DialogPart::Title.slot(),
                    id: title_id(),
                    size: "xl",
                    sx: &DIALOG_HEADER_TITLE_SX,
                    "{title}"
                }
            }
            if close_button {
                ActionIcon {
                    "data-slot": DialogPart::Close.slot(),
                    variant: "standard",
                    color: "muted",
                    size: "sm",
                    aria_label: close_label
                        .filter(|label| !label.trim().is_empty())
                        .unwrap_or_else(|| current_localization().common.close.to_string()),
                    onclick: move |_: MouseEvent| close.call(()),
                    Glyph { slot: IconSlot::Close, icon: lucide::x::outlined }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        let table: Vec<_> = DialogPart::ALL
            .iter()
            .map(|part| (part.slot(), part.selector()))
            .collect();

        assert_eq!(
            table,
            [
                ("header", "& > [data-slot='header']"),
                ("title", "& > [data-slot='header'] > [data-slot='title']"),
                ("close", "& > [data-slot='header'] > [data-slot='close']"),
            ]
        );
    }

    /// Todo 2562: a blank `aria_label` does not mask a good title, and a blank title
    /// renders no empty heading.
    #[test]
    fn a_blank_name_names_nothing() {
        fn app() -> Element {
            rsx! {
                crate::LiberoProvider {
                    Dialog { id: "labelled", title: "Filters", aria_label: " ", "one" }
                    Dialog { id: "blank", title: "  ", aria_label: "Notes", "two" }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("aria-labelledby="), "{html}");
        assert!(!html.contains("aria-label=\" \""), "{html}");
        assert_eq!(html.matches("<h2").count(), 1, "{html}");
        assert!(html.contains("aria-label=\"Notes\""), "{html}");
    }

    /// Todo 3791: a blank `close_label` falls back to the localized close.
    #[test]
    fn a_blank_close_label_falls_back() {
        fn app() -> Element {
            rsx! {
                crate::LiberoProvider {
                    Dialog { title: "Notes", close_button: true, close_label: " ", "one" }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(!html.contains("aria-label=\" \""), "{html}");
        assert!(html.contains("aria-label=\"Close\""), "{html}");
    }

    /// 3978: `full_screen` marks a dialog in a modal only; outside one there is no frame to fill.
    #[test]
    fn full_screen_applies_inside_a_modal() {
        #[component]
        fn InModal() -> Element {
            use_context_provider(|| ModalContext {
                onclose: Some(Callback::new(|()| {})),
                dismiss_guard: None,
            });
            rsx! {
                Dialog { title: "Inside", full_screen: "phone", "one" }
            }
        }
        fn app() -> Element {
            rsx! {
                crate::LiberoProvider {
                    InModal {}
                    Dialog { title: "Outside", full_screen: "always", "two" }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert_eq!(
            html.matches("data-state=\"full-screen").count(),
            1,
            "{html}"
        );
        assert!(html.contains("data-state=\"full-screen-phone\""), "{html}");
    }

    /// Todo 2564: a spread `alertdialog` survives, any other role gives way.
    #[test]
    fn only_alertdialog_overrides_the_role() {
        fn app() -> Element {
            rsx! {
                crate::LiberoProvider {
                    Dialog { title: "Delete?", role: "alertdialog", "one" }
                    Dialog { title: "Notes", role: "region", "two" }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert_eq!(html.matches("role=\"alertdialog\"").count(), 1, "{html}");
        // The last of a duplicate name wins in the DOM.
        assert!(html.contains("role=\"region\" role=\"dialog\""), "{html}");
    }
}
