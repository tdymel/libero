use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, HtmlTag, Input, States, Variables, Variant,
        common::{
            CloseIcon, base_color, base_props, contrast_color, fill_color, text_color, variables,
        },
        inputs::{VariantVars, variant_chrome_sx, variant_colors},
        layout::use_box,
        surface::paper_sx,
    },
    hooks::{use_root_id, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        ALERT_BODY_GAP, ALERT_ICON_SIZE, ALERT_RADIUS, AlertDefaults, CssVar, NamedColorCss, Size,
        SizeCss,
    },
};

const ALERT_COLOR_VAR: CssVar = CssVar::new("--lsx-alert-color");
const ALERT_FILL_VAR: CssVar = CssVar::new("--lsx-alert-fill");
const ALERT_CONTRAST_VAR: CssVar = CssVar::new("--lsx-alert-contrast");
const ALERT_CONTAINER_VAR: CssVar = CssVar::new("--lsx-alert-container");
const ALERT_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-alert-on-container");

const ALERT_VARS: VariantVars<'static> = VariantVars {
    color: &ALERT_COLOR_VAR,
    fill: &ALERT_FILL_VAR,
    contrast: &ALERT_CONTRAST_VAR,
    container: &ALERT_CONTAINER_VAR,
    on_container: &ALERT_ON_CONTAINER_VAR,
};

static ALERT_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = paper_sx()
        // `paper_sx()` opens with `display: block`; an alert is the flex row
        // `[icon][body][close]`. Chained on the same `Sx`, so the emitted
        // class carries one `display`, not two racing in the cascade.
        .display("flex")
        // Top-aligned: with a wrapped message the icon and the close button
        // belong beside the first line, not centred on the paragraph.
        .align_items("flex-start")
        // A surface that rests in the flow rather than floating over it.
        // `Elevated` puts its own shadow back, from inside its fold.
        .box_shadow("none")
        // `variant_chrome_sx` sets `border-color` only - the width and style
        // that make `Outlined`'s border visible join here.
        .border_style("solid")
        .border_width("1px")
        // Mantine's: a long unbroken title or message is cut at the radius
        // rather than escaping the tint.
        .overflow("hidden")
        .and(AlertDefaults::theme_vars())
        .selector(
            "& > [data-slot='icon']",
            sx().display("flex")
                .flex_shrink("0")
                .align_items("center")
                .justify_content("center")
                .width(ALERT_ICON_SIZE.value())
                .height(ALERT_ICON_SIZE.value())
                .selector("& svg", sx().width("100%").height("100%")),
        )
        // `icon: rsx! {}`, or an `if` that rendered nothing, is still a sized
        // flex item and still takes the `gap`: 32px of nothing before the
        // text. `:empty` ignores dioxus' placeholder, so this catches both.
        .selector("& > [data-slot='icon']:empty", sx().display("none"))
        .selector(
            "& > [data-slot='body']",
            sx().display("flex")
                .flex_direction("column")
                .gap(ALERT_BODY_GAP.value())
                // Takes the row, and `min-width: 0` is what lets it shrink
                // below its content so the ellipsis below can ever apply.
                .flex("1")
                .min_width("0"),
        )
        .selector(
            "& [data-slot='title']",
            sx().font_weight("600")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        );

    // Chrome only, no `:hover` - `Badge`'s rule. An alert is not a target, and
    // a tint that moved under the pointer would claim it is.
    Variant::ALL.iter().fold(base, |base, &variant| {
        let chrome = variant_chrome_sx(variant, &ALERT_VARS);
        // `paper_sx()`'s black focus ring is lost on a solid ground - 1.8:1 on
        // `neutral`. The text colour reads on it by construction, so the ring
        // takes that. `currentColor` rather than an unset var for a literal
        // colour: an unset var here would not fall back, it would erase the
        // ring (`paper.md`). Only on the children: the alert's own ring sits
        // outside it, on the page, where the text colour is often white on
        // white.
        let chrome = match variant {
            Variant::Filled => chrome.selector(
                "& > *",
                sx().var(
                    CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
                    ALERT_CONTRAST_VAR.value_or("currentColor"),
                ),
            ),
            _ => chrome,
        };
        base.when(variant.state_name(), chrome)
    })
});

fn alert_variables(props: &AlertProps, base: &ThemeAwareValue, variant: Variant) -> Variables {
    let contrast = contrast_color(base);
    let colors = variant_colors(variant, base);

    variables()
        .with(ALERT_COLOR_VAR, text_color(base))
        .with(ALERT_FILL_VAR, fill_color(base))
        .with(ALERT_CONTRAST_VAR, contrast.and_then(|c| c.resolve(None)))
        .with(ALERT_CONTAINER_VAR, colors.container)
        .with(ALERT_ON_CONTAINER_VAR, colors.on_container)
        .with(
            ALERT_RADIUS.override_var(),
            props
                .radius
                .as_ref()
                .map(|&radius| SizeCss::RADIUS.value(radius)),
        )
}

base_props! {
    pub struct AlertProps {
        /// The heading, and the region's accessible name via
        /// `aria-labelledby`. A `String` rather than an `Element`: a name is
        /// text, and markup in it would be dropped from the name silently -
        /// `Caption`'s rule.
        #[props(default, into)]
        title: Option<String>,
        /// A leading glyph, rendered `aria-hidden` - it repeats what the text
        /// already says. The library ships no icon set; this is the caller's
        /// own.
        #[props(default)]
        icon: Option<Element>,
        /// The tint base; a theme colour name or a literal CSS colour.
        /// Defaults to `theme.alert.color`, which is `info` - severity is the
        /// caller's to state.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// The five M3 arms, shared with `Button` and `Badge` - minus their
        /// hover response. `Outlined` is border-plus-`color: inherit` and
        /// carries no tint, deliberately.
        #[props(default, into)]
        variant: Input<Variant>,
        /// A size step or any CSS length.
        #[props(default, into)]
        radius: Input<Size>,
        /// **Its presence is what shows the close button.** There is no
        /// separate `with_close_button`: the pair would make "a close button
        /// that does nothing" representable, and a `Callback` prop can never
        /// be required anyway, so missing has to mean something - here it
        /// means no button.
        ///
        /// Closing is the caller unmounting the `Alert`. It does not hide
        /// itself.
        #[props(default)]
        onclose: Option<EventHandler<()>>,
        /// The close button's accessible name. Defaults to
        /// `theme.alert.close_label`.
        #[props(default, into)]
        close_label: Option<String>,
        /// The message.
        children: Element,
    }
}

/// A tinted surface that states something the reader has to know: an error
/// summary, a warning, a note.
///
/// Renders `role="alert"` **as a default the caller's own `role` wins over**,
/// so a caller that needs `role="status"` - or `Form`'s summary, which brings
/// its own `role` and `tabindex` - is not fighting the component.
///
/// It takes no focus, has no keyboard behaviour of its own and does not close
/// on `Escape`: it is not an overlay, and nothing gives it focus. The close
/// button is a real `<button>` inside an `ActionIcon`, so `Tab` reaches it and
/// `Enter`/`Space` activate it.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::Alert;
/// # fn app() -> Element {
/// # let mut dismissed = use_signal(|| false);
/// # rsx! {
/// Alert {
///     color: "warning",
///     title: "Card expiring",
///     icon: rsx! { WarningGlyph {} },
///     onclose: move |_| dismissed.set(true),
///     "Your card ends 09/26. Update it before the next invoice."
/// }
/// # } }
/// # #[component] fn WarningGlyph() -> Element { rsx! {} }
/// ```
#[component]
pub fn Alert(props: AlertProps) -> Element {
    let theme = use_theme();
    // Adopts a caller's own `id`, so the aria wiring below and the caller
    // never render two.
    let id = use_root_id(&props.attributes);

    let variant = props.variant.copied_or(theme.alert.variant);
    let color = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ThemeAwareValue::from(theme.alert.color));
    let base = base_color(Some(&color));

    let variables: Input<Variables> = alert_variables(&props, &base, variant).into();
    let states: Input<States> = props
        .states
        .clone()
        .unwrap_or_default()
        .with(variant.state_name(), true)
        .into();

    let title_id = props.title.as_ref().map(|_| format!("{}-title", id()));
    // An alert with only a title has no message to describe it with, and an
    // `aria-describedby` pointing at nothing is worse than none.
    let message = (props.children != VNode::empty()).then(|| props.children.clone());
    let body_id = message.as_ref().map(|_| format!("{}-body", id()));
    let close_label = props
        .close_label
        .clone()
        .unwrap_or_else(|| theme.alert.close_label.to_string());

    let onclose = props.onclose;
    let icon = props.icon.clone();
    let title = props.title.clone();
    let children = rsx! {
        if let Some(icon) = icon {
            span { "data-slot": "icon", "aria-hidden": "true", {icon} }
        }
        div { "data-slot": "body",
            if let Some(title) = title {
                span { "data-slot": "title", id: "{id}-title", "{title}" }
            }
            if let Some(message) = message {
                div { "data-slot": "message", id: "{id}-body", {message} }
            }
        }
        if let Some(onclose) = onclose {
            ActionIcon {
                "data-slot": "close",
                // `currentColor`, not the alert's colour: the glyph reads on
                // whatever the variant painted, `Filled`'s solid ground too.
                variant: "standard",
                color: "currentColor",
                size: "sm",
                aria_label: close_label,
                onclick: move |_| onclose.call(()),
                CloseIcon {}
            }
        }
    };

    use_box()
        .framework_sx(&ALERT_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        // `attr_default`, never `attr`: the component supplies the role only
        // where the caller supplied none. `Form`'s summary sets its own, and
        // a duplicate `role` attribute drops one of the two
        // non-deterministically - SSR keeps the first, the DOM the last.
        .attr_default("role", "alert")
        .attr("id", id())
        .attr("aria-labelledby", title_id)
        .attr("aria-describedby", body_id)
        .render(HtmlTag::Div, props.attributes, children)
}
