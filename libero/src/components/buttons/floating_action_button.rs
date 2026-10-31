use dioxus::{core::AttributeValue, prelude::*};

use crate::{
    components::{
        buttons::{ActionIcon, Button},
        common::{HtmlTag, Input, States, Variant, attr, base_props},
        layout::{Float, Placement, use_box},
    },
    hooks::{DismissOptions, listener, use_dismiss, use_element, use_id, use_theme},
    sx::{REDUCED_MOTION, Sx, ThemeAwareValue, sx},
    theme::{
        CssVar, Gradient, ICON_SIZE, OWN_SHADOW, Size, SizeCss, TRANSITION_DURATION,
        TRANSITION_EASING, Z_INDEX_HEADER,
    },
};

/// What a docked `NavigationBar` publishes, the bottom safe-area inset included.
const NAVIGATION_BAR_HEIGHT_VAR: CssVar = CssVar::new("--lsx-navigation-bar-height");

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
        /// Unset, the theme's `filled`.
        #[props(default, into)]
        variant: Input<Variant>,
        #[props(default, into)]
        gradient: Option<Gradient>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// A step of `Button`'s heights; unset, `xl` (60px). With `children`, only a size word applies.
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        /// Unset, `xxl`: a circle, or a pill with `children`.
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
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
        #[props(default)]
        hidden: bool,
        /// `FloatingActionButtonAction`s: a press opens them as a speed dial above the button,
        /// or below a top one. `to` is ignored then.
        #[props(default)]
        actions: Option<Element>,
        /// Whether the `actions` show, for a controlled speed dial; unset, the button owns it.
        #[props(default)]
        open: Option<Signal<bool>>,
    }
}

base_props! {
    pub struct FloatingActionButtonActionProps {
        /// The glyph before the label.
        icon: Element,
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Renders a router link instead of a `<button>`.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        #[props(default)]
        disabled: Option<bool>,
        /// The visible label, which names the action.
        #[props(into)]
        label: String,
    }
}

/// Closes the speed dial from a pressed action, handing focus back to the button.
#[derive(Clone, Copy)]
struct SpeedDial {
    close: Callback<()>,
}

/// Where the actions open: away from the placed edge, aligned to the button's inline side.
fn actions_sx(placement: Placement) -> Sx {
    use Placement::*;
    let gap = SizeCss::SPACING.value(Size::Sm);
    let mut list = sx()
        .position("absolute")
        .display("flex")
        .gap(gap.clone())
        .width("max-content");
    list = match placement {
        TopStart | TopCenter | TopEnd => list
            .top("100%")
            .flex_direction("column")
            .padding_block_start(gap),
        _ => list
            .bottom("100%")
            .flex_direction("column-reverse")
            .padding_block_end(gap),
    };
    // Logical, so the column hugs the button's end edge under `dir="rtl"` too.
    match placement {
        TopStart | CenterStart | BottomStart => {
            list.inset_inline_start("0").align_items("flex-start")
        }
        TopEnd | CenterEnd | BottomEnd => list.inset_inline_end("0").align_items("flex-end"),
        TopCenter | CenterCenter | BottomCenter => list
            .left("50%")
            .transform("translateX(-50%)")
            .align_items("center"),
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
    let mut edges = sx().display("flex");
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
/// or extended with a label in `children`. `class`, `sx` and attributes land on the button.
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
    let variant = props.variant.copied_or(defaults.variant);
    let size = props
        .size
        .clone()
        .into_option()
        .unwrap_or(ThemeAwareValue::Size(defaults.size));
    let radius = props
        .radius
        .clone()
        .into_option()
        .unwrap_or(ThemeAwareValue::Size(defaults.radius));
    let extended = props.children.is_some();
    let states: Input<States> = props
        .states
        .clone()
        .unwrap_or_default()
        .with("extended", extended)
        .into();
    let button_size = match &size {
        ThemeAwareValue::Size(size) => *size,
        _ => defaults.size,
    };
    let mut button_sx = elevation_sx(SizeCss::SHADOW.value(defaults.elevation));
    if extended {
        // As the icon-only glyph: `Icon`'s size at the step, where `Button` leaves an unsized svg at 0.
        let glyph = ICON_SIZE.value(button_size);
        button_sx = button_sx.selector(
            "& :where([data-slot='button-icon'] > svg)",
            sx().width(glyph.clone()).height(glyph),
        );
    }
    if let Some(own) = props.sx.as_ref() {
        button_sx = button_sx.and(own.clone());
    }
    let aria_label = props
        .aria_label
        .clone()
        .filter(|label| !label.trim().is_empty());

    let own_open = use_signal(|| false);
    let mut open = props.open.unwrap_or(own_open);
    let dial = props.actions.is_some();
    let is_open = dial && open() && !props.hidden;
    let anchor = use_element();
    let floating = use_element();
    let close = use_callback(move |()| open.set(false));
    let dismiss = use_dismiss(
        anchor,
        floating,
        is_open,
        true,
        Some(close),
        DismissOptions::default(),
    );
    let focus_return = dismiss.focus_return();
    use_context_provider(|| SpeedDial {
        close: Callback::new(move |()| dismiss.dismiss()),
    });
    let list_id = use_id();
    let own_id = use_id();
    let list_sx: Input<Sx> = actions_sx(placement).into();
    let list = use_box().sx(&list_sx).prepare();

    let mut attributes = props.attributes.clone();
    // The group is named by the button, through the caller's `id` or one of its own.
    let caller_id = attributes
        .iter()
        .rev()
        .find_map(|attribute| match &attribute.value {
            AttributeValue::Text(id) if attribute.name == "id" && !id.trim().is_empty() => {
                Some(id.clone())
            }
            _ => None,
        });
    let trigger_id = caller_id.clone().unwrap_or_else(|| own_id.cloned());
    if dial {
        if caller_id.is_none() {
            attributes.push(attr("id", trigger_id.clone()));
        }
        // The trigger outlives every opening, so it is remembered once, on mount.
        attributes.extend(anchor.attributes());
        attributes.push(listener("onmounted", move |event: Event<MountedData>| {
            anchor.mount()(event.clone());
            focus_return.remember(event);
        }));
        attributes.push(attr("aria-expanded", is_open.to_string()));
        if is_open {
            attributes.push(attr("aria-controls", list_id()));
        }
        attributes.extend(dismiss.anchor_events());
    }
    let user_onclick = props.onclick;
    let onclick = (dial || user_onclick.is_some()).then(|| {
        EventHandler::new(move |event: MouseEvent| {
            if dial {
                let next = !*open.peek();
                open.set(next);
            }
            if let Some(onclick) = user_onclick {
                onclick.call(event);
            }
        })
    });
    let to = if dial { Input::None } else { props.to.clone() };
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
                    disabled: props.disabled,
                    loading: props.loading,
                    loader: props.loader.clone(),
                    ripple: props.ripple,
                    onclick,
                    to: to.clone(),
                    target: props.target.clone(),
                    aria_label,
                    class: props.class.clone(),
                    sx: button_sx.clone(),
                    states: states.clone(),
                    attributes: attributes.clone(),
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
                    disabled: props.disabled,
                    loading: props.loading,
                    loader: props.loader.clone(),
                    ripple: props.ripple,
                    onclick,
                    to,
                    target: props.target.clone(),
                    class: props.class.clone(),
                    sx: button_sx,
                    states,
                    attributes,
                    {props.icon.clone()}
                }
            }
            // After the button: a disclosure's content follows its trigger in the reading order.
            if is_open {
                {
                    list.element(&floating)
                        .attr("id", list_id())
                        .attr("role", "group")
                        .attr("aria-labelledby", trigger_id)
                        .render(HtmlTag::Div, dismiss.floating_events(), props.actions.clone().unwrap_or(VNode::empty()))
                }
            }
        }
    }
}

/// One action of a `FloatingActionButton`'s speed dial: a labelled pill that closes
/// the dial when pressed. `class`, `sx` and attributes land on the button.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{FloatingActionButton, FloatingActionButtonAction, Pictogram};
/// # use pictogram_icons_lucide as lucide;
/// # fn app() -> Element {
/// rsx! {
///     FloatingActionButton {
///         aria_label: "Create",
///         icon: rsx! { Pictogram { icon: lucide::plus::outlined } },
///         actions: rsx! {
///             FloatingActionButtonAction { label: "Message", icon: rsx! { Pictogram { icon: lucide::mail::outlined } } }
///             FloatingActionButtonAction { label: "Event", icon: rsx! { Pictogram { icon: lucide::calendar::outlined } } }
///         },
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/floating-action-button>
#[component]
pub fn FloatingActionButtonAction(props: FloatingActionButtonActionProps) -> Element {
    let dial = try_use_context::<SpeedDial>();
    let user_onclick = props.onclick;
    let onclick = move |event: MouseEvent| {
        if let Some(onclick) = user_onclick {
            onclick.call(event);
        }
        if let Some(dial) = dial {
            dial.close.call(());
        }
    };
    let glyph = ICON_SIZE.value(Size::Md);
    let mut action_sx = elevation_sx(SizeCss::SHADOW.value(Size::Sm)).selector(
        "& :where([data-slot='button-icon'] > svg)",
        sx().width(glyph.clone()).height(glyph),
    );
    if let Some(own) = props.sx.as_ref() {
        action_sx = action_sx.and(own.clone());
    }

    rsx! {
        Button {
            variant: Variant::Tonal,
            size: Size::Md,
            radius: ThemeAwareValue::Size(Size::Xxl),
            disabled: props.disabled,
            onclick,
            to: props.to.clone(),
            target: props.target.clone(),
            class: props.class.clone(),
            sx: action_sx,
            states: props.states.clone(),
            attributes: props.attributes.clone(),
            icon: props.icon.clone(),
            "{props.label}"
        }
    }
}
