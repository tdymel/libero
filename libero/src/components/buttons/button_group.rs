use std::collections::HashSet;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            ButtonGroupContext, COARSE_POINTER, HtmlTag, Input, Orientation, States, Variant,
            base_props, names_itself, use_name_warning, use_provide_button_group,
        },
        layout::use_box,
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ColorCss, ColorShade, Size},
    utils::warn,
};

/// A control inside an item: the item itself, or one it wraps (`Repository`'s, `Copy`'s and
/// `Share`'s spans, `ThemeSwitcher`'s pair).
const CONTROL: &str = ":is(button, a)";

/// The theme's hairline, `Divider`'s colour: a borderless variant has no edge to share.
fn divider() -> String {
    format!("1px solid {}", ColorCss::MUTED.value(ColorShade::S4))
}

/// `rule` on each control of the items `item` matches, itself or inside it.
fn controls(item: &str, rule: Sx) -> Sx {
    sx().selector(format!("& > {item}{CONTROL}"), rule.clone())
        .selector(format!("& > {item} {CONTROL}"), rule)
}

/// The item's leading control, which carries the seam; `ThemeSwitcher`'s chevron has its own.
fn seam(overlap: Sx, line: Sx) -> Sx {
    let outlined = format!(":not([data-state~=\"{}\"])", Variant::Outlined.state_name());
    ["", " > :first-child"]
        .into_iter()
        .fold(sx(), |base, inner| {
            let at = format!("& > :not(:first-child){inner}{CONTROL}");
            base.selector(at.clone(), overlap.clone())
                .selector(format!("{at}{outlined}"), line.clone())
        })
}

/// `ThemeSwitcher`'s pair is a group of its own: its chevron's inner seam gets the divider too.
fn pair_seam(line: Sx) -> Sx {
    let borderless = format!(":not([data-state~=\"{}\"])", Variant::Outlined.state_name());
    sx().selector(
        format!("& > [role=\"group\"] > div > {CONTROL}{borderless}"),
        line,
    )
}

/// Under a finger an item's enlarged press box grows across the group, not along it, so it takes
/// no neighbour's end. `ends` are the two insets along the group.
fn coarse_across(ends: Sx) -> Sx {
    sx().media(
        COARSE_POINTER,
        // The items themselves, one wrapper in (`Repository`, `Copy`, `Share`), and the pair's two controls.
        sx().selector(format!("& > {CONTROL}::before"), ends.clone())
            .selector(format!("& > * > {CONTROL}::before"), ends.clone())
            .selector(
                format!("& > [role=\"group\"] > * > {CONTROL}::before"),
                ends,
            ),
    )
}

// One `when` deep, so these outrank the controls' own radius and variant rules.
static BUTTON_GROUP_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .max_width("100%")
        // The focused control's ring over its neighbours.
        .selector(format!("& {CONTROL}:focus-visible"), sx().z_index("1"))
        .when(
            Orientation::Horizontal.state_name(),
            // Logical sides: under RTL the previous item is on the right.
            controls(
                ":not(:first-child)",
                sx().border_start_start_radius("0")
                    .border_end_start_radius("0"),
            )
            .and(controls(
                ":not(:last-child)",
                sx().border_start_end_radius("0").border_end_end_radius("0"),
            ))
            .and(seam(
                sx().margin_inline_start("-1px"),
                sx().border_inline_start(divider()),
            ))
            .and(pair_seam(sx().border_inline_start(divider())))
            .and(coarse_across(
                sx().inset_inline_start("0").inset_inline_end("0"),
            )),
        )
        .when(
            Orientation::Vertical.state_name(),
            sx().flex_direction("column")
                .align_items("stretch")
                // `auto`, not `100%`, lets a set width (`ActionIcon`) stretch: Blitz resolves no percentage in the shrink-to-fit column (todo 2330).
                .selector(format!("& > {CONTROL}"), sx().width("auto"))
                .selector(
                    format!("& > :not([role=\"group\"]) > {CONTROL}"),
                    sx().width("100%"),
                )
                // A `max-content` wrapper (`Tooltip`'s) is stretched too (todo 2321).
                .selector(
                    format!("& > :not({CONTROL}, [role=\"group\"])"),
                    sx().width("auto"),
                )
                .selector(
                    format!("& > [role=\"group\"] > {CONTROL}"),
                    sx().flex_grow("1"),
                )
                .and(controls(
                    ":not(:first-child)",
                    sx().border_top_left_radius("0")
                        .border_top_right_radius("0"),
                ))
                .and(controls(
                    ":not(:last-child)",
                    sx().border_bottom_left_radius("0")
                        .border_bottom_right_radius("0"),
                ))
                .and(seam(
                    sx().margin_top("-1px"),
                    sx().border_block_start(divider()),
                ))
                // The pair stays a row in a column.
                .and(pair_seam(sx().border_inline_start(divider())))
                .and(coarse_across(sx().top("0").bottom("0"))),
        )
});

base_props! {
    pub struct ButtonGroupProps {
        /// Unset, `"horizontal"`.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Default `size` of the buttons inside.
        #[props(default, into)]
        size: Input<Size>,
        /// The group's outer corners, a size word or any CSS; the inner ones are always square.
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Default `variant` of the buttons inside. Borderless ones get a divider between them.
        #[props(default, into)]
        variant: Input<Variant>,
        /// Default `color` of the buttons inside.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Disables every button inside that does not set `disabled` itself.
        #[props(default)]
        disabled: Option<bool>,
        /// Default `focusable_when_disabled` of the buttons inside: a disabled one stays a Tab stop.
        #[props(default)]
        focusable_when_disabled: Option<bool>,
        /// The `value`s of the pressed buttons, strictly controlled. Set, a button with a `value` is a toggle
        /// (`aria-pressed`), unless it sets `selected` itself. Unset, a plain group.
        #[props(default)]
        value: Option<HashSet<String>>,
        /// With `value`: any number of buttons pressed. Off, pressing one releases the other.
        #[props(default)]
        multiple: bool,
        /// Asks for the next pressed set. Without it the selection can never change.
        #[props(default)]
        onchange: Option<EventHandler<HashSet<String>>>,
        /// `Button`s, `ActionIcon`s, and components built on them.
        children: Element,
    }
}

/// Buttons and action icons side by side as one control: shared seams, only the outer corners round.
/// Name it with an `aria-label`: it renders `role="group"`.
///
/// A child hidden with `display: none` still counts as first or last and squares its neighbour's
/// outer corners. Render a conditional control only when shown, or keep it outside the group.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, ButtonGroup};
/// # fn app() -> Element {
/// rsx! {
///     ButtonGroup { "aria-label": "Alignment", variant: "outlined",
///         Button { "Left" }
///         Button { "Center" }
///         Button { "Right" }
///     }
/// }
/// # }
/// ```
///
/// With `value` its buttons toggle: one at a time, or any number with `multiple`.
///
/// ```rust
/// # use std::collections::HashSet;
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, ButtonGroup};
/// # fn app() -> Element {
/// let mut pressed = use_signal(|| HashSet::from(["bold".to_string()]));
/// rsx! {
///     ButtonGroup {
///         "aria-label": "Text style",
///         multiple: true,
///         value: pressed(),
///         onchange: move |next| pressed.set(next),
///         Button { value: "bold", "Bold" }
///         Button { value: "italic", "Italic" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/button-group>
#[component]
pub fn ButtonGroup(props: ButtonGroupProps) -> Element {
    let orientation = props.orientation.copied_or(Orientation::Horizontal);
    use_name_warning(
        names_itself(&props.attributes),
        "ButtonGroup: no `aria-label` or `aria-labelledby`, so it is announced as just \"group\".",
    );
    use_provide_button_group(ButtonGroupContext {
        size: props.size.as_ref().copied(),
        radius: props.radius.as_ref().cloned(),
        variant: props.variant.as_ref().copied(),
        color: props.color.as_ref().cloned(),
        disabled: props.disabled,
        focusable_when_disabled: props.focusable_when_disabled,
        value: props.value.clone(),
        multiple: props.multiple,
        onchange: props.onchange,
    });
    if props.value.is_some() && props.onchange.is_none() {
        warn("ButtonGroup: without `onchange` the selection can never change.");
    }

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(orientation.state_name(), true)
        .into();

    use_box()
        .framework_sx(&BUTTON_GROUP_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .attr("role", "group")
        .render(HtmlTag::Div, props.attributes, props.children)
}
