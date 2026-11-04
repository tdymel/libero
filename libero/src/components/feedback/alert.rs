use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{
            COARSE_POINTER, COARSE_TARGET, Glyph, HtmlTag, Input, Part, ScaleOrCss, States,
            Variables, Variant, VariantVars, base_color, base_props, contrast_color, fill_color,
            label_color, literal_contrast, parts_enum, responsive_sx, variables, variant_chrome_sx,
            variant_container_colors, with_own,
        },
        layout::{paper_sx, use_box},
    },
    context::IconSlot,
    hooks::{use_localization, use_range_css, use_root_id, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        ALERT_BODY_GAP, ALERT_GAP, ALERT_ICON_SIZE, ALERT_RADIUS, ANCHOR_COLOR, AlertDefaults,
        AnchorDefaults, BUTTON_HEIGHT, Color, CssVar, FOCUS_RING_HALO, NamedColorCss, Responsive,
        SURFACE_LABEL, Size, SizeCss,
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
        // Chained on the same `Sx` as `paper_sx()`'s `display: block`, so the
        // class carries one `display`, not two racing in the cascade.
        .display("flex")
        // A wide `actions` row drops under the body, not beside a one-character message.
        .flex_wrap("wrap")
        // Icon and close button sit beside the first line of a wrapped message.
        .align_items("flex-start")
        // `Elevated` puts its own shadow back, from inside its fold.
        .box_shadow("none")
        // `variant_chrome_sx` sets only `border-color`.
        .border_style("solid")
        .border_width("1px")
        .overflow("hidden")
        .and(AlertDefaults::theme_vars())
        .selector(
            AlertPart::Icon.selector(),
            sx().display("flex")
                .flex_shrink("0")
                .align_items("center")
                .justify_content("center")
                .width(ALERT_ICON_SIZE.value())
                // At least one line box tall, so the icon centres on the first line (todo 5360).
                .height(format!("max({}, 1lh)", ALERT_ICON_SIZE.value()))
                .selector(
                    "& svg",
                    sx().width(ALERT_ICON_SIZE.value())
                        .height(ALERT_ICON_SIZE.value()),
                ),
        )
        // An empty icon would still take its size and the `gap`. `:empty`
        // ignores dioxus' placeholder, so an `if` that rendered nothing counts.
        .selector(
            format!("{}:empty", AlertPart::Icon.selector()),
            sx().display("none"),
        )
        .selector(
            AlertPart::Body.selector(),
            sx().display("flex")
                .flex_direction("column")
                .gap(ALERT_BODY_GAP.value())
                // 60% of the row at least, else `actions` wraps (106px wide message
                // at 320px otherwise); `min-width: 0` lets a long title wrap.
                .flex("1 1 60%")
                .min_width("0")
                // Wraps, never an ellipsis: cut text is unreadable (WCAG 1.4.10).
                .with("overflow-wrap", "anywhere"),
        )
        .selector(
            AlertPart::Actions.selector(),
            sx().display("flex")
                .flex_shrink("0")
                .align_items("center")
                .align_self("center")
                .gap(ALERT_GAP.value())
                .max_width("100%")
                // A sized Button wraps a long label and grows past its step, not clipped.
                .selector(
                    "& > *",
                    sx().white_space("normal")
                        .overflow_wrap("anywhere")
                        .per_size(|size| {
                            sx().height("auto")
                                .min_height(format!(
                                    "max({}, calc(1.5em + 2px))",
                                    BUTTON_HEIGHT.value(size)
                                ))
                                // Same selector as the step's height, which outranks `Button`'s own.
                                .media(
                                    COARSE_POINTER,
                                    sx().min_height(format!(
                                        "max({COARSE_TARGET}, calc(1.5em + 2px))"
                                    )),
                                )
                        }),
                ),
        )
        // On a wrapped row the close button keeps the end edge.
        .selector(
            AlertPart::Close.selector(),
            sx().margin_inline_start("auto"),
        )
        .selector(AlertPart::Title.selector(), sx().font_weight("600"));

    // Chrome only, no `:hover`: an alert is not a click target.
    Variant::ALL.iter().fold(base, |base, &variant| {
        base.when(variant.state_name(), variant_sx(variant))
    })
});

/// One variant's surface, shared by the static sheet and a responsive range's copy.
fn variant_sx(variant: Variant) -> Sx {
    let chrome = variant_chrome_sx(variant, &ALERT_VARS);
    // On a tint or fill the page's link colour falls to about 1.5:1: links take
    // the text colour, as on `Mark` (todo 1576), underlined at rest (todo 1647);
    // so does an uncoloured standard Button (todo 1663).
    let links = sx()
        .var(ANCHOR_COLOR, "currentColor")
        .var(SURFACE_LABEL, "currentColor");
    // `Filled`: children's focus ring takes the text colour, haloed by the fill
    // (todo 630). `currentColor` fallback, as an unset var erases the ring.
    match variant {
        Variant::Filled => chrome.and(AnchorDefaults::underline_at_rest()).selector(
            "& > *",
            links
                .var(
                    CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
                    ALERT_CONTRAST_VAR.value_or("currentColor"),
                )
                .var(
                    FOCUS_RING_HALO,
                    ALERT_FILL_VAR.value_or(ALERT_COLOR_VAR.value()),
                ),
        ),
        Variant::Tonal | Variant::Gradient => chrome
            .and(AnchorDefaults::underline_at_rest())
            .selector("& > *", links),
        _ => chrome,
    }
}

/// A range copy of one radius step: the base var that `border-radius` falls back to.
fn radius_copy_sx(size: Size) -> Sx {
    sx().var(ALERT_RADIUS, SizeCss::RADIUS.value(size))
}

/// `variant` is the one whose container colours are written: only `Tonal` reads them.
fn alert_variables(base: &ThemeAwareValue, variant: Variant, radius: Option<String>) -> Variables {
    let contrast = contrast_color(base);
    // Not `variant_colors`: its hover and selected shades are unused here.
    let (container, on_container) = variant_container_colors(variant, base);

    variables()
        .with(ALERT_COLOR_VAR, label_color(base))
        .with(ALERT_FILL_VAR, fill_color(base))
        .with(
            ALERT_CONTRAST_VAR,
            contrast
                .and_then(|c| c.resolve(None))
                .or_else(|| literal_contrast(base)),
        )
        .with(ALERT_CONTAINER_VAR, container)
        .with(ALERT_ON_CONTAINER_VAR, on_container)
        .with(ALERT_RADIUS.override_var(), radius)
}

/// The palette colour `color` names, `None` for any other CSS colour.
fn palette_color(color: &ThemeAwareValue) -> Option<Color> {
    match color {
        ThemeAwareValue::Color(color) => Some(*color),
        ThemeAwareValue::ColorValue(value) => Some(value.color()),
        _ => None,
    }
}

/// `alert` interrupts the reader: only `error` and `warning` earn it.
fn alert_role(color: &ThemeAwareValue) -> &'static str {
    match palette_color(color) {
        Some(Color::Error | Color::Warning) => "alert",
        _ => "status",
    }
}

/// The default glyph of a severity colour; none for the brand and neutral ones.
fn severity_icon(color: &ThemeAwareValue) -> Option<Element> {
    let (slot, icon) = match palette_color(color)? {
        Color::Info => (IconSlot::Info, lucide::info::outlined),
        Color::Success => (IconSlot::Success, lucide::circle_check::outlined),
        Color::Warning => (IconSlot::Warning, lucide::triangle_alert::outlined),
        Color::Error => (IconSlot::Error, lucide::circle_alert::outlined),
        _ => return None,
    };
    Some(rsx! { Glyph { slot, icon } })
}

parts_enum! {
    /// [`Alert`]'s inner parts, for its `parts` prop. Each is a direct child,
    /// so a nested `Alert` in the message keeps its own styles.
    pub enum AlertPart {
        /// The leading glyph's wrapper.
        Icon = "icon" => "& > [data-slot='icon']",
        /// Title and message, beside the icon.
        Body = "body" => "& > [data-slot='body']",
        Title = "title" => "& > [data-slot='body'] > [data-slot='title']",
        Message = "message" => "& > [data-slot='body'] > [data-slot='message']",
        /// The `actions` row, between the body and the close button.
        Actions = "actions" => "& > [data-slot='actions']",
        /// The close button, when `onclose` is set.
        Close = "close" => "& > [data-slot='close']",
    }
}

base_props! {
    parts(AlertPart);
    pub struct AlertProps {
        /// The title, and the accessible name.
        #[props(default, into)]
        title: Option<String>,
        /// A leading glyph, `aria-hidden`. Unset, a `color` set to `info`, `success`, `warning`
        /// or `error` draws its severity's glyph.
        #[props(default)]
        icon: Option<Element>,
        /// `false` drops the severity glyph, `true` draws it for the theme's colour too.
        /// Unset, `theme.alert.severity_icons` for a `color` the call site sets.
        #[props(default)]
        with_icon: Option<bool>,
        /// Theme colour or CSS colour; `error` and `warning` make it `role="alert"`.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Surface style, without a hover response.
        /// Per breakpoint too: `responsive(Variant::Outlined).md(Variant::Filled)`.
        #[props(default, into)]
        variant: Input<Responsive<Variant>>,
        /// A size step from `xs` to `xxl`, or any CSS, as `radius: "0"`.
        /// Per breakpoint too: `responsive(Size::Sm).md(Size::Xl)`.
        #[props(default, into)]
        radius: Input<Responsive<ThemeAwareValue>>,
        /// Buttons or links beside the message, before the close button.
        #[props(default)]
        actions: Option<Element>,
        /// Set, shows the close button. The caller unmounts the alert and moves focus.
        #[props(default)]
        onclose: Option<EventHandler<()>>,
        /// The close button's accessible name.
        #[props(default, into)]
        close_label: Option<String>,
        /// The message.
        children: Element,
    }
}

/// A tinted surface for something the reader has to know: an error, a warning, a note.
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
///
/// Docs: <https://libero-ui.dev/feedback/alert>
#[component]
pub fn Alert(props: AlertProps) -> Element {
    let theme = use_theme();
    let common = use_localization().common;
    let id = use_root_id(&props.attributes);

    let variants = props.variant.copied_or(theme.alert.variant.into());
    let color = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ThemeAwareValue::from(theme.alert.color));
    let base = base_color("Alert", Some(&color));
    let role = alert_role(&color);

    // A plain radius writes its inline override; a responsive one rides the range copies.
    let radii = props.radius.as_ref().filter(|radii| !radii.is_plain());
    let radius_steps = radii
        .map(|radii| radii.map(|value| ScaleOrCss::new(Some(&value), theme.alert.radius).size));
    let radius = match radii {
        Some(_) => None,
        None => props
            .radius
            .as_ref()
            .and_then(|radius| radius.base_ref().resolve(Some(SizeCss::RADIUS))),
    };
    // Custom CSS per breakpoint rides the override var in the user layer; a size resets it.
    let radius_sx = radii.and_then(|radii| {
        responsive_sx(radii, |sx, value| {
            let css = ScaleOrCss::new(Some(&value), theme.alert.radius)
                .custom_css(SizeCss::RADIUS)
                .unwrap_or_else(|| "initial".into());
            sx.var(ALERT_RADIUS.override_var(), css)
        })
    });
    let own_sx = radius_sx.map(|radius_sx| with_own(Some(radius_sx), &props.sx));
    let user_sx = own_sx.as_ref().unwrap_or(&props.sx);

    // Only `Tonal` reads the container colours: written if any range is tonal.
    let container = match variants
        .breakpoints()
        .any(|(_, variant)| variant == Variant::Tonal)
    {
        true => Variant::Tonal,
        false => variants.base(),
    };
    let variables: Input<Variables> = alert_variables(&base, container, radius).into();
    let states = props
        .states
        .clone()
        .unwrap_or_default()
        .responsive(&variants, Variant::state_name);
    let states: Input<States> = match &radius_steps {
        Some(steps) => states.responsive(steps, |size| size.radius_state_name()),
        None => states,
    }
    .into();

    let mut copies = variants.range_copies(Variant::state_name, variant_sx);
    if let Some(steps) = &radius_steps {
        copies.extend(steps.range_copies(|size| size.radius_state_name(), radius_copy_sx));
    }
    use_range_css(&ALERT_BASE_SX, copies);

    // A blank title draws no span, so nothing is left for `aria-labelledby` to point at.
    let title = props.title.clone().filter(|title| !title.trim().is_empty());
    let title_id = title.as_ref().map(|_| format!("{}-title", id()));
    // No `aria-describedby` pointing at an empty message.
    let message = (props.children != VNode::empty()).then(|| props.children.clone());
    let body_id = message.as_ref().map(|_| format!("{}-body", id()));
    let close_label = props
        .close_label
        .clone()
        .filter(|label| !label.trim().is_empty())
        .unwrap_or_else(|| common.close.to_string());

    let onclose = props.onclose;
    // Severity is the caller's to state: the theme's default colour draws no glyph.
    let severity_glyph = props
        .with_icon
        .unwrap_or(theme.alert.severity_icons && props.color.as_ref().is_some());
    let icon = props
        .icon
        .clone()
        .or_else(|| severity_glyph.then(|| severity_icon(&color)).flatten());
    let actions = props.actions.clone();
    let children = rsx! {
        if let Some(icon) = icon {
            span { "data-slot": AlertPart::Icon.slot(), "aria-hidden": "true", {icon} }
        }
        div { "data-slot": AlertPart::Body.slot(),
            if let Some(title) = title {
                span { "data-slot": AlertPart::Title.slot(), id: "{id}-title", "{title}" }
            }
            if let Some(message) = message {
                div { "data-slot": AlertPart::Message.slot(), id: "{id}-body", {message} }
            }
        }
        if let Some(actions) = actions {
            div { "data-slot": AlertPart::Actions.slot(), {actions} }
        }
        if let Some(onclose) = onclose {
            ActionIcon {
                "data-slot": AlertPart::Close.slot(),
                // `currentColor` reads on every variant's ground, `Filled` too.
                variant: "standard",
                color: "currentColor",
                size: "xs",
                aria_label: close_label,
                onclick: move |_| onclose.call(()),
                Glyph { slot: IconSlot::Close, icon: lucide::x::outlined }
            }
        }
    };

    use_box()
        .framework_sx(&ALERT_BASE_SX)
        .class(&props.class)
        .sx(user_sx)
        .visibility(&props.hidden_from, &props.visible_from)
        .parts(&props.parts)
        .states(&states)
        .variables(&variables)
        .prepare()
        // `attr_default`: a caller's own `role` (`Form`'s summary) wins. A duplicate
        // `role` is dropped non-deterministically, SSR keeping the first, the DOM the last.
        .attr_default("role", role)
        .attr("id", id())
        .attr("aria-labelledby", title_id)
        .attr("aria-describedby", body_id)
        .render(HtmlTag::Div, props.attributes, children)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        let table: Vec<_> = AlertPart::ALL
            .iter()
            .map(|part| (part.slot(), part.selector()))
            .collect();

        assert_eq!(
            table,
            [
                ("icon", "& > [data-slot='icon']"),
                ("body", "& > [data-slot='body']"),
                ("title", "& > [data-slot='body'] > [data-slot='title']"),
                ("message", "& > [data-slot='body'] > [data-slot='message']"),
                ("actions", "& > [data-slot='actions']"),
                ("close", "& > [data-slot='close']"),
            ]
        );
    }

    /// Measured at 320px and 200% text: nowrap left the message 35px wide beside "Update card".
    #[test]
    fn the_actions_row_wraps_under_a_body_that_keeps_most_of_the_row() {
        let css = crate::css::Stylesheet::from(&*ALERT_BASE_SX);
        let css = css.as_str();

        assert!(css.contains("flex-wrap:wrap;"), "{css}");
        assert!(css.contains("flex:1 1 60%;"), "{css}");
    }

    /// The page's link colour is about 1.5:1 on an `error` fill, 3.5:1 on its tint (todo 1576).
    #[test]
    fn a_filled_or_tinted_alert_gives_its_links_the_text_colour() {
        let css = crate::css::Stylesheet::from(&*ALERT_BASE_SX);
        let css = css.as_str();

        for variant in [Variant::Filled, Variant::Tonal, Variant::Gradient] {
            let state = format!("[data-state~=\"{}\"] > *{{", variant.state_name());
            let rule = css.find(&state).unwrap_or_else(|| panic!("{state}: {css}"));
            assert!(
                css[rule..].contains("--lsx-anchor-color:currentColor;"),
                "{state}: {css}"
            );
        }
        assert!(!css.contains("[data-state~=\"outlined\"] > *{"), "{css}");
    }
}
