use dioxus::{dioxus_core::AttributeValue, prelude::*};
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{
            Glyph, HtmlTag, Input, Part, SAFE_AREA_INLINE_END_VAR, SAFE_AREA_INLINE_START_VAR,
            States, Variables, attr, base_props, input_from_str, names_itself, parts_enum,
            responsive_var_sx, safe_area_padding, use_name_warning, variables, with_own,
        },
        layout::{Box, paper_sx, use_box},
        typography::Title,
    },
    context::{Dismiss, IconSlot, ModalContext},
    hooks::{current_localization, use_id, use_range_css, use_theme},
    sx::{FORCED_COLORS, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        CssVar, DIALOG_DEFAULT_SIZE, DIALOG_SIZE, DialogFullScreen, PAPER_RADIUS, Responsive, Size,
        SizeCss, responsive,
    },
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
        // Forced colours drop the shadow, the panel's only edge over the page (5309).
        .media(FORCED_COLORS, sx().border("1px solid CanvasText"))
        .when("full-screen-always", full_screen_sx())
        .media(
            format!("not all and (min-width: {})", Size::Sm.breakpoint_value()),
            sx().when("full-screen-phone", full_screen_sx()),
        )
});

/// Edge to edge over `Modal`'s frame: its negative margins cancel the frame's `md`
/// padding, and `stretch` and `flex-grow` fill it. Plain CSS, so no centred first frame.
/// Its padding clears a notch and a home indicator on all four sides, so a `Sidebar` inside skips them (5531).
/// A column capped at the viewport: a [`DialogBody`] scrolls, its [`DialogActions`] stay at the foot,
/// and plain content scrolls the dialog itself, as it did the modal (5634).
fn full_screen_sx() -> Sx {
    let padding = SizeCss::SPACING.value(Size::Lg);
    sx().margin(format!("calc(-1 * {})", SizeCss::SPACING.value(Size::Md)))
        .align_self("stretch")
        .flex_grow("1")
        .max_width("none")
        .max_height("100dvh")
        .overflow_y("auto")
        .display("flex")
        .flex_direction("column")
        .border_radius("0")
        .box_shadow("none")
        .selector("& > *", sx().flex_shrink("0"))
        .selector(
            "& > [data-slot='body']",
            sx().flex("1 1 auto").min_height("0").overflow_y("auto"),
        )
        .selector("& > [data-slot='actions']", sx().margin_top("auto"))
        .with("padding-top", safe_area_padding(&padding, "top"))
        .with("padding-right", safe_area_padding(&padding, "right"))
        .with("padding-bottom", safe_area_padding(&padding, "bottom"))
        .with("padding-left", safe_area_padding(&padding, "left"))
        .var(SAFE_AREA_INLINE_START_VAR, "0")
        .var(SAFE_AREA_INLINE_END_VAR, "0")
}

input_from_str!(DialogFullScreen);

/// `Phone` is `responsive(true).sm(false)`.
impl From<DialogFullScreen> for Responsive<bool> {
    fn from(value: DialogFullScreen) -> Self {
        match value {
            DialogFullScreen::Never => responsive(false),
            DialogFullScreen::Phone => responsive(true).sm(false),
            DialogFullScreen::Always => responsive(true),
        }
    }
}

impl From<DialogFullScreen> for Input<Responsive<bool>> {
    fn from(value: DialogFullScreen) -> Self {
        Self::Value(value.into())
    }
}

impl From<&str> for Input<Responsive<bool>> {
    fn from(value: &str) -> Self {
        DialogFullScreen::from(value).into()
    }
}

impl From<String> for Input<Responsive<bool>> {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

/// The enum shape a value has, which writes today's word; `None` takes run words and copies.
fn full_screen_shape(value: &Responsive<bool>) -> Option<DialogFullScreen> {
    [
        DialogFullScreen::Never,
        DialogFullScreen::Phone,
        DialogFullScreen::Always,
    ]
    .into_iter()
    .find(|shape| Responsive::from(*shape).runs() == value.runs())
}

fn full_screen_word(on: bool) -> &'static str {
    if on { "full-screen" } else { "centred" }
}

/// `full_screen` as state words; a custom shape registers `block` per full-screen range on `base`.
pub(super) fn use_full_screen_states(
    states: States,
    value: &Responsive<bool>,
    base: &'static StaticSx,
    block: fn() -> Sx,
) -> States {
    let shape = full_screen_shape(value);
    let copies = match shape {
        Some(_) => Vec::new(),
        None => value.range_copies(full_screen_word, move |on| if on { block() } else { sx() }),
    };
    use_range_css(base, copies);
    match shape {
        Some(shape) => states
            .with("full-screen-phone", shape == DialogFullScreen::Phone)
            .with("full-screen-always", shape == DialogFullScreen::Always),
        None => states.responsive(value, full_screen_word),
    }
}

fn dialog_header_sx(margin_bottom: &str) -> Sx {
    sx().display("flex")
        .align_items("flex-start")
        // Not `space-between`: a lone close button would sit top-left.
        .justify_content("flex-end")
        .gap("sm")
        .margin_bottom(margin_bottom)
        // The 30px close box reaches into the padding, its glyph where a bare one sat.
        .selector(
            "& > button",
            sx().margin_top("-5px").margin_inline_end("-5px"),
        )
}

static DIALOG_HEADER_SX: StaticSx = StaticSx::new(|| dialog_header_sx("md"));

// A lone close button keeps its own row but adds no gap: on a phone the height counts (4847).
static DIALOG_BARE_HEADER_SX: StaticSx = StaticSx::new(|| dialog_header_sx("0"));

static DIALOG_HEADER_TITLE_SX: StaticSx = StaticSx::new(|| {
    sx().margin("0")
        .flex("1")
        // `overflow-wrap` leaves min-content alone, so a long word would push past the dialog.
        .min_width("0")
});

/// A responsive `radius` or `size` leaves its var to [`dialog_sx`].
fn dialog_variables(props: &DialogProps, default_radius: Option<Size>) -> Variables {
    fn plain(value: &Input<Responsive<ThemeAwareValue>>) -> Option<&ThemeAwareValue> {
        value
            .as_ref()
            .filter(|value| value.is_plain())
            .map(Responsive::base_ref)
    }
    let radius = match props.radius.as_ref() {
        Some(_) => plain(&props.radius).cloned(),
        None => default_radius.map(ThemeAwareValue::from),
    };
    variables()
        .with(
            DIALOG_RADIUS_VAR,
            radius.and_then(|radius| radius.resolve(Some(SizeCss::RADIUS))),
        )
        .with(
            DIALOG_SIZE.override_var(),
            plain(&props.size).and_then(|size| size.resolve(Some(DIALOG_SIZE))),
        )
}

/// `radius` and `size` per breakpoint, in the user layer under the caller's `sx`.
fn dialog_sx(props: &DialogProps) -> Option<Sx> {
    [
        props
            .radius
            .as_ref()
            .and_then(|radius| responsive_var_sx(radius, DIALOG_RADIUS_VAR, Some(SizeCss::RADIUS))),
        props.size.as_ref().and_then(|size| {
            responsive_var_sx(size, DIALOG_SIZE.override_var(), Some(DIALOG_SIZE))
        }),
    ]
    .into_iter()
    .flatten()
    .reduce(Sx::and)
}

parts_enum! {
    /// [`Dialog`]'s inner parts, for its `parts` prop. Matched as direct children,
    /// so a dialog in the content keeps its own styles.
    pub enum DialogPart {
        /// The row holding the title and the close button.
        Header = "header" => "& > [data-slot='header']",
        Title = "title" => "& > [data-slot='header'] > [data-slot='title']",
        Close = "close" => "& > [data-slot='header'] > [data-slot='close']",
        /// A [`DialogBody`]. Full screen, it fills the space between the header and the actions and scrolls.
        Body = "body" => "& > [data-slot='body']",
        /// A [`DialogActions`] row. Full screen, it sits at the foot.
        Actions = "actions" => "& > [data-slot='actions']",
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
        /// `theme.dialog.radius`, else `Paper`'s. Per breakpoint too: `responsive(Size::Sm).md(Size::Lg)`.
        #[props(default, into)]
        radius: Input<Responsive<ThemeAwareValue>>,
        /// Unset, `theme.dialog.size`. A full-screen dialog ignores it, as does
        /// `full_screen: "phone"` below `sm`. Per breakpoint too: `responsive(Size::Sm).md(Size::Lg)`.
        #[props(default, into)]
        size: Input<Responsive<ThemeAwareValue>>,
        /// Inside a modal, fills the screen edge to edge instead of centring: `"phone"`
        /// below the `sm` breakpoint, `"always"` at every width. Unset,
        /// `theme.dialog.full_screen`, else centred. Per breakpoint too:
        /// `responsive(true).md(false)`; `"phone"` is `responsive(true).sm(false)`.
        /// A [`DialogBody`] then scrolls and a [`DialogActions`] stays at the foot.
/// Its children then form a column, so a bare `Button` stretches: use those two parts.
        #[props(default, into)]
        full_screen: Input<Responsive<bool>>,
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
///     Dialog { title: "New note", full_screen: "phone", ondismiss: move |_| draft.read().is_empty(),
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
    let defaults = use_theme().dialog;
    let own_sx = dialog_sx(&props).map(|extra| with_own(Some(extra), &props.sx));
    let variables: Input<Variables> = dialog_variables(&props, defaults.radius)
        .merge(props.variables.unwrap_or_default())
        .into();
    // Outside a modal there is no frame to fill.
    let full_screen = match is_modal {
        true => props.full_screen.copied_or(defaults.full_screen.into()),
        false => responsive(false),
    };
    let states: Input<States> = use_full_screen_states(
        props.states.unwrap_or_default(),
        &full_screen,
        &DIALOG_BASE_SX,
        full_screen_sx,
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
        .visibility(&props.hidden_from, &props.visible_from)
        .sx(own_sx.as_ref().unwrap_or(&props.sx))
        .parts(&props.parts)
        .states(&states)
        .variables(&variables)
        .prepare()
        .render(HtmlTag::Div, attributes, children)
}

base_props! {
    pub struct DialogBodyProps {
        children: Element,
    }
}

/// The scrolling content of a [`Dialog`]. A direct child of it. Full screen, it fills the
/// space between the header and the [`DialogActions`] and scrolls; otherwise a plain block.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, Dialog, DialogActions, DialogBody};
/// # fn app() -> Element {
/// rsx! {
///     Dialog { title: "Terms", full_screen: "phone",
///         DialogBody { "A long text." }
///         DialogActions { Button { "Accept" } }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/overlay/dialog>
#[component]
pub fn DialogBody(mut props: DialogBodyProps) -> Element {
    props
        .attributes
        .push(attr("data-slot", DialogPart::Body.slot()));
    use_box()
        .class(&props.class)
        .visibility(&props.hidden_from, &props.visible_from)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(HtmlTag::Div, props.attributes, props.children)
}

// The gap is padding, so the foot's `margin-top: auto` keeps the room when the body fills.
static DIALOG_ACTIONS_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_wrap("wrap")
        .justify_content("flex-end")
        .gap("sm")
        .padding_top("md")
});

base_props! {
    pub struct DialogActionsProps {
        children: Element,
    }
}

/// The buttons row of a [`Dialog`], at its end. A direct child of it. Full screen, it sits
/// at the foot of the screen below a scrolling [`DialogBody`], above the home indicator.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, Dialog, DialogActions, DialogBody};
/// # fn app() -> Element {
/// rsx! {
///     Dialog { title: "Delete file?", full_screen: "phone",
///         DialogBody { "notes.md goes for good." }
///         DialogActions {
///             Button { variant: "outlined", "Cancel" }
///             Button { "Delete" }
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/overlay/dialog>
#[component]
pub fn DialogActions(mut props: DialogActionsProps) -> Element {
    props
        .attributes
        .push(attr("data-slot", DialogPart::Actions.slot()));
    use_box()
        .framework_sx(&DIALOG_ACTIONS_SX)
        .class(&props.class)
        .visibility(&props.hidden_from, &props.visible_from)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(HtmlTag::Div, props.attributes, props.children)
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
        Box {
            framework_sx: if title.is_some() { &DIALOG_HEADER_SX } else { &DIALOG_BARE_HEADER_SX },
            "data-slot": DialogPart::Header.slot(),
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
                ("body", "& > [data-slot='body']"),
                ("actions", "& > [data-slot='actions']"),
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

    /// 5657: a value shaped like an enum word keeps that word; any other shape takes runs.
    #[test]
    fn a_full_screen_value_finds_its_enum_shape() {
        let phone = responsive(true).xs(true).sm(false);
        assert_eq!(full_screen_shape(&phone), Some(DialogFullScreen::Phone));
        assert_eq!(
            full_screen_shape(&false.into()),
            Some(DialogFullScreen::Never)
        );
        assert_eq!(
            full_screen_shape(&true.into()),
            Some(DialogFullScreen::Always)
        );
        assert_eq!(full_screen_shape(&responsive(true).md(false)), None);
    }

    /// 5365: an unset `full_screen` takes `theme.dialog.full_screen`; a prop beats it.
    #[test]
    fn an_unset_full_screen_follows_the_theme() {
        static PHONE: crate::theme::Theme = crate::theme::Theme {
            dialog: crate::theme::DialogDefaults {
                full_screen: DialogFullScreen::Phone,
                ..crate::theme::Theme::DEFAULT.dialog
            },
            ..crate::theme::Theme::DEFAULT
        };
        #[component]
        fn InModal() -> Element {
            use_context_provider(|| ModalContext {
                onclose: Some(Callback::new(|()| {})),
                dismiss_guard: None,
            });
            rsx! {
                Dialog { title: "Themed", "one" }
                Dialog { title: "Own", full_screen: "never", "two" }
            }
        }
        fn app() -> Element {
            rsx! {
                crate::LiberoProvider { themes: &PHONE, InModal {} }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert_eq!(
            html.matches("data-state=\"full-screen-phone\"").count(),
            1,
            "{html}"
        );
    }

    /// Todo 4847: a header holding only the close button drops its margin below.
    #[test]
    fn an_untitled_header_drops_its_margin() {
        fn app() -> Element {
            rsx! {
                crate::LiberoProvider {
                    Dialog { title: "Notes", close_button: true, onclose: |_| {}, "one" }
                    Dialog { aria_label: "Notes", close_button: true, onclose: |_| {}, "two" }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let headers: Vec<_> = html
            .split("<div")
            .filter(|tag| tag.contains("data-slot=\"header\""))
            .map(|tag| tag.split('>').next().unwrap_or_default())
            .collect();

        assert_eq!(headers.len(), 2, "{html}");
        assert_ne!(headers[0], headers[1], "{html}");
        let bare = crate::css::Stylesheet::from(&DIALOG_BARE_HEADER_SX);
        assert!(
            bare.as_str().contains("margin-bottom:0"),
            "{}",
            bare.as_str()
        );
        let titled = crate::css::Stylesheet::from(&DIALOG_HEADER_SX);
        assert!(
            !titled.as_str().contains("margin-bottom:0"),
            "{}",
            titled.as_str()
        );
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
