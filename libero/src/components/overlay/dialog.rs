use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{
            Glyph, HtmlTag, Input, Part, Variables, attr, base_props, names_itself, parts_enum,
            use_name_warning, variables,
        },
        layout::{Box, paper_sx, use_box},
        typography::Title,
    },
    context::{IconSlot, ModalContext},
    hooks::{current_localization, use_id},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CssVar, DIALOG_SIZE, PAPER_RADIUS, Size, SizeCss},
    utils::warn,
};

const DIALOG_RADIUS_VAR: CssVar = CssVar::new("--lsx-dialog-radius");

// `Paper`'s chrome, plus what makes it a dialog.
static DIALOG_BASE_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        // Back on: `Modal`'s wrapper lets backdrop clicks fall through.
        .pointer_events("auto")
        // A flex item shrinks to content, so `size` would only cap, not fill.
        .width("100%")
        .max_width(DIALOG_SIZE.overridable(Size::Md))
        .margin("md")
        .padding("lg")
        .border_radius(DIALOG_RADIUS_VAR.value_or(PAPER_RADIUS.value()))
        .box_shadow(SizeCss::SHADOW.value(Size::Xl))
});

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

static DIALOG_HEADER_TITLE_SX: StaticSx = StaticSx::new(|| sx().margin("0").flex("1"));

fn dialog_variables(props: &DialogProps) -> Variables {
    variables()
        .with(
            DIALOG_RADIUS_VAR,
            props
                .radius
                .as_ref()
                .map(|&radius| SizeCss::RADIUS.value(radius)),
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
        /// The close button's accessible name, e.g. "Close cart".
        #[props(default, into)]
        close_label: Option<String>,
        #[props(default, into)]
        radius: Input<Size>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
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
/// Docs: <https://libero-ui.dev/overlay/dialog>
#[component]
pub fn Dialog(props: DialogProps) -> Element {
    let modal = try_use_context::<ModalContext>().filter(ModalContext::is_modal);
    let is_modal = modal.is_some();
    let close_button = props
        .close_button
        .unwrap_or(is_modal || props.onclose.is_some());
    let title_id = use_id();
    use_name_warning(
        props.aria_label.is_some() || props.title.is_some() || names_itself(&props.attributes),
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
        (Some(modal), _) => modal.close(),
        (None, Some(onclose)) => onclose.call(()),
        (None, None) => {}
    });
    let variables: Input<Variables> = dialog_variables(&props)
        .merge(props.variables.unwrap_or_default())
        .into();

    // Pushed after the caller's, to win a duplicate name.
    let mut attributes = props.attributes;
    attributes.push(attr("role", "dialog"));
    if is_modal {
        attributes.push(attr("aria-modal", "true"));
        // Focusable by script and by a click on its text, so focus and the
        // modal's keys stay inside when nothing in it takes focus (APG).
        attributes.push(attr("tabindex", "-1"));
    }
    if props.aria_label.is_none() && props.title.is_some() {
        attributes.push(attr("aria-labelledby", title_id()));
    }
    if let Some(aria_label) = props.aria_label.clone() {
        attributes.push(attr("aria-label", aria_label));
    }

    let mut children = Vec::with_capacity(2);
    if props.title.is_some() || close_button {
        children.push(rsx! {
            DialogHeader {
                title: props.title,
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
        .states(&props.states)
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
                    aria_label: close_label.unwrap_or_else(|| current_localization().common.close.to_string()),
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
}
