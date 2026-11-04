use dioxus::prelude::*;

use crate::{
    components::{
        buttons::{ActionIcon, Button},
        common::{Input, NAVIGATION_BAR_HEIGHT_VAR, States, Variant, attr, base_props},
        layout::{FLOAT_SAFE_AREA_VAR, Float, Placement},
    },
    hooks::use_theme,
    sx::{REDUCED_MOTION, Sx, ThemeAwareValue, sx},
    theme::{
        Gradient, ICON_DEFAULT_SIZE, ICON_SIZE, OWN_SHADOW, Responsive, Size, SizeCss,
        TRANSITION_DURATION, TRANSITION_EASING, Z_INDEX_HEADER,
    },
};

base_props! {
    pub struct FloatingActionButtonProps {
        /// The glyph, e.g. `rsx! { Pictogram { icon: lucide::plus::outlined } }`.
        icon: Element,
        /// Required without `children`: an icon-only button has no text to announce.
        #[props(default, into)]
        aria_label: Option<String>,
        /// Shows `aria_label` in a `Tooltip` on hover and keyboard focus. Icon-only.
        #[props(default)]
        tooltip: Option<bool>,
        /// Unset, the theme's `filled`. Per breakpoint too, as on `ActionIcon`.
        #[props(default, into)]
        variant: Input<Responsive<Variant>>,
        #[props(default, into)]
        gradient: Option<Gradient>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// A step of `Button`'s heights; unset, `xl` (60px). With `children`, only a size word applies.
        /// Per breakpoint too, as on `ActionIcon`.
        #[props(default, into)]
        size: Input<Responsive<ThemeAwareValue>>,
        /// Unset, `xxl`: a circle, or a pill with `children`. Per breakpoint too.
        #[props(default, into)]
        radius: Input<Responsive<ThemeAwareValue>>,
        /// The corner or edge it sits at; unset, `bottom-end`. `start` and `end` follow the direction.
        #[props(default, into)]
        placement: Input<Placement>,
        /// Distance from the placed edges, a spacing word or any CSS length; unset, `md`.
        #[props(default, into)]
        offset: Input<ThemeAwareValue>,
        /// Against the viewport, clear of a docked `NavigationBar` and the safe area.
        /// `false` places it in the nearest `position: relative` ancestor instead.
        #[props(default = true)]
        fixed: bool,
        /// Unset, the header layer: above the page, below the skip link, windows and overlays.
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        #[props(default)]
        disabled: Option<bool>,
        /// Overlays a `Loader` and swallows clicks. Ignored in link mode.
        #[props(default)]
        loading: Option<bool>,
        #[props(default)]
        loader: Option<Element>,
        /// The press ripple. `false` drops it.
        #[props(default = true)]
        ripple: bool,
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Renders a router link instead of a `<button>`.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        /// A visible label: an extended button, the `icon` before it.
        #[props(default)]
        children: Option<Element>,
        /// Fades it out of view, the tab order and the accessibility tree, as for hide-on-scroll.
        /// Focus on it drops to the page, so the app moves focus first when it may hold focus.
        #[props(default)]
        hidden: bool,
    }
}

/// The fade of `hidden`; `visibility` flips at the end, so it stays drawn while fading.
/// `scale`, not `transform`, which holds `Float`'s placement.
fn hidden_sx(hidden: bool) -> Sx {
    let duration = TRANSITION_DURATION.value();
    let easing = TRANSITION_EASING.value();
    let motion = format!("opacity {duration} {easing}, scale {duration} {easing}");
    let fade = if hidden {
        sx().opacity("0")
            .with("scale", "0.6")
            .visibility("hidden")
            .pointer_events("none")
            .transition(format!("{motion}, visibility 0s linear {duration}"))
    } else {
        sx().opacity("1")
            .with("scale", "1")
            .transition(format!("{motion}, visibility 0s"))
    };
    fade.media(REDUCED_MOTION, sx().transition("none"))
}

/// The margins that hold the button off the placed edges; a fixed one also clears
/// a docked `NavigationBar` and the safe area. Logical, so `end` flips under `dir="rtl"`.
fn edge_sx(placement: Placement, offset: &str, fixed: bool) -> Sx {
    use Placement::*;
    // Flex, so no line box: Blitz put a 6px strut under the inline-flex button (todo 5062).
    // Its margins clear the safe area already, so `Float`'s own clearance is off.
    let mut edges = sx().display("flex").var(FLOAT_SAFE_AREA_VAR, "0");
    match placement {
        TopStart | TopCenter | TopEnd => {
            let top = if fixed {
                format!("calc(env(safe-area-inset-top, 0px) + {offset})")
            } else {
                offset.to_string()
            };
            edges = edges.margin_block_start(top);
        }
        BottomStart | BottomCenter | BottomEnd => {
            let bottom = if fixed {
                format!(
                    "calc({} + {offset})",
                    NAVIGATION_BAR_HEIGHT_VAR.value_or("env(safe-area-inset-bottom, 0px)")
                )
            } else {
                offset.to_string()
            };
            edges = edges.margin_block_end(bottom);
        }
        CenterStart | CenterCenter | CenterEnd => {}
    }
    match placement {
        TopStart | CenterStart | BottomStart => edges.margin_inline_start(offset.to_string()),
        TopEnd | CenterEnd | BottomEnd => edges.margin_inline_end(offset.to_string()),
        TopCenter | CenterCenter | BottomCenter => edges,
    }
}

/// The resting shadow, kept under the focus ring through `OWN_SHADOW`. Zero
/// specificity added, so a caller's `box_shadow` still wins.
fn elevation_sx(shadow: String) -> Sx {
    sx().var(OWN_SHADOW, shadow.clone())
        .selector("&:where(:not(:focus-visible))", sx().box_shadow(shadow))
}

/// A button that floats over the page at a corner, for its main action: icon-only,
/// or extended with a label in `children`. `class`, `sx` and attributes land on the button,
/// so for several actions it is a `Menu`'s trigger through `menu.a11y_attributes()`.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{FloatingActionButton, Pictogram};
/// # use pictogram_icons_lucide as lucide;
/// # fn app() -> Element {
/// rsx! {
///     FloatingActionButton {
///         aria_label: "New message",
///         icon: rsx! { Pictogram { icon: lucide::plus::outlined } },
///     }
///     FloatingActionButton { icon: rsx! { Pictogram { icon: lucide::pencil::outlined } }, "Compose" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/floating-action-button>
#[component]
pub fn FloatingActionButton(props: FloatingActionButtonProps) -> Element {
    let defaults = use_theme().floating_action_button;
    let placement = props.placement.copied_or(defaults.placement);
    let offset = props
        .offset
        .resolve(Some(SizeCss::SPACING))
        .unwrap_or_else(|| SizeCss::SPACING.value(defaults.offset));
    let z_index = props
        .z_index
        .clone()
        .into_option()
        .unwrap_or_else(|| ThemeAwareValue::from(Z_INDEX_HEADER.value()));
    let variant = props.variant.copied_or(defaults.variant.into());
    let size = props
        .size
        .clone()
        .into_option()
        .unwrap_or(ThemeAwareValue::Size(defaults.size).into());
    let radius = props
        .radius
        .clone()
        .into_option()
        .unwrap_or(ThemeAwareValue::Size(defaults.radius).into());
    let extended = props.children.is_some();
    let states: Input<States> = props
        .states
        .clone()
        .unwrap_or_default()
        .with("extended", extended)
        .into();
    let button_size = size.map(|size| match size {
        ThemeAwareValue::Size(size) => size,
        _ => defaults.size,
    });
    // An unsized `Icon` round the glyph follows the step, as a bare svg does.
    let icon_step = |rules: Sx, size: Size| rules.var(ICON_DEFAULT_SIZE, ICON_SIZE.value(size));
    let mut button_sx = elevation_sx(SizeCss::SHADOW.value(defaults.elevation));
    button_sx = button_size.breakpoints().fold(
        icon_step(button_sx, button_size.base()),
        |rules, (at, size)| rules.breakpoint(at, icon_step(sx(), size)),
    );
    if extended {
        // As the icon-only glyph: `Icon`'s size at the step, where `Button` leaves an unsized svg at 0.
        let glyph_sx = |rules: Sx, size: Size| {
            let glyph = ICON_SIZE.value(size);
            rules.selector(
                "& :where([data-slot='button-icon'] > svg)",
                sx().width(glyph.clone()).height(glyph),
            )
        };
        button_sx = button_size.breakpoints().fold(
            glyph_sx(button_sx, button_size.base()),
            |rules, (at, size)| rules.breakpoint(at, glyph_sx(sx(), size)),
        );
    }
    if let Some(own) = props.sx.as_ref() {
        button_sx = button_sx.and(own.clone());
    }
    let aria_label = props
        .aria_label
        .clone()
        .filter(|label| !label.trim().is_empty());
    let mut float_attributes = Vec::new();
    if props.hidden {
        float_attributes.push(attr("inert", "true"));
    }

    rsx! {
        Float {
            placement,
            fixed: props.fixed,
            z_index,
            sx: edge_sx(placement, &offset, props.fixed).and(hidden_sx(props.hidden)),
            attributes: float_attributes,
            if let Some(children) = props.children.clone() {
                Button {
                    variant,
                    gradient: props.gradient.clone(),
                    color: props.color.clone(),
                    size: button_size,
                    radius,
                    hidden_from: props.hidden_from,
                    visible_from: props.visible_from,
                    disabled: props.disabled,
                    loading: props.loading,
                    loader: props.loader.clone(),
                    ripple: props.ripple,
                    onclick: props.onclick,
                    to: props.to.clone(),
                    target: props.target.clone(),
                    aria_label,
                    class: props.class.clone(),
                    sx: button_sx.clone(),
                    states: states.clone(),
                    attributes: props.attributes.clone(),
                    icon: props.icon.clone(),
                    {children}
                }
            } else {
                ActionIcon {
                    aria_label: aria_label.unwrap_or_default(),
                    tooltip: props.tooltip,
                    variant,
                    gradient: props.gradient.clone(),
                    color: props.color.clone(),
                    size,
                    radius,
                    hidden_from: props.hidden_from,
                    visible_from: props.visible_from,
                    disabled: props.disabled,
                    loading: props.loading,
                    loader: props.loader.clone(),
                    ripple: props.ripple,
                    onclick: props.onclick,
                    to: props.to.clone(),
                    target: props.target.clone(),
                    class: props.class.clone(),
                    sx: button_sx,
                    states,
                    attributes: props.attributes.clone(),
                    {props.icon.clone()}
                }
            }
        }
    }
}
