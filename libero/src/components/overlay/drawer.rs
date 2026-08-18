use dioxus::prelude::*;

use crate::{
    components::{
        Box, Dialog, Float, Input, Modal, Placement, ScrollArea, States, Variables,
        common::{base_props, input_from_str},
        variables,
    },
    hooks::use_css,
    hooks::use_portal,
    str_enum::str_enum,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ColorCss, ColorShade, CssVar, DRAWER_SIZE, Size},
};

const DRAWER_Z_INDEX_VAR: CssVar = CssVar::new("--lsx-drawer-z-index");

// `Static` is documented as usable for a sidebar, which is almost always a
// flex item that needs to not get squeezed and to scroll its own content
// rather than growing past its allotted space - the actual scrolling is
// `ScrollArea`'s job (see the component below), this only sizes the panel
// itself.
static DRAWER_STATIC_BASE_SX: StaticSx = StaticSx::new(|| {
    let size = DRAWER_SIZE.override_var().value_or("auto");
    let border = format!("1px solid {}", ColorCss::GREY.value(ColorShade::S4));

    sx().flex_shrink("0")
        .min_height("0")
        .z_index(DRAWER_Z_INDEX_VAR.value_or("auto"))
        .when(
            "anchor-left",
            sx().width(size.clone()).border_right(border.clone()),
        )
        .when(
            "anchor-right",
            sx().width(size.clone()).border_left(border.clone()),
        )
        .when(
            "anchor-top",
            sx().height(size.clone()).border_bottom(border.clone()),
        )
        .when(
            "anchor-bottom",
            sx().height(size.clone()).border_top(border.clone()),
        )
});

// `Temporary`'s surface, layered onto `Dialog`'s own class/vars (Dialog owns
// its own `framework_sx`, so this rides along as an extra class rather than
// replacing it). Edge-docking itself is `Float`'s job (see `Drawer`) - this
// only sizes the panel to fill the slot `Float` anchors it into.
static DRAWER_TEMPORARY_SX: StaticSx = StaticSx::new(|| {
    let size = DRAWER_SIZE.overridable(Size::Md);

    sx().margin("0")
        .border_radius("0")
        .max_width("none")
        .when("anchor-left", sx().height("100%").width(size.clone()))
        .when("anchor-right", sx().height("100%").width(size.clone()))
        .when("anchor-top", sx().width("100%").height(size.clone()))
        .when("anchor-bottom", sx().width("100%").height(size.clone()))
});

/// Maps a `DrawerAnchor` to a corner `Float` placement plus the one extra
/// offset that stretches it to a full edge - a corner placement anchors two
/// adjacent sides with no `transform` involved, so this only adds the third.
fn drawer_float_placement(anchor: DrawerAnchor) -> (Placement, Sx) {
    match anchor {
        DrawerAnchor::Left => (Placement::TopStart, sx().bottom("0")),
        DrawerAnchor::Right => (Placement::TopEnd, sx().bottom("0")),
        DrawerAnchor::Top => (Placement::TopStart, sx().right("0")),
        DrawerAnchor::Bottom => (Placement::BottomStart, sx().right("0")),
    }
}

str_enum! {
    pub enum DrawerVariant {
        /// Portaled, dimmed, focus-trapped, closes on Escape/backdrop - built on
        /// `Modal`.
        #[default]
        Temporary = "temporary",
        /// In-place plain panel, no portal/backdrop/focus-trap, e.g. a sidebar.
        Static = "static",
    }
}

input_from_str!(DrawerVariant);

str_enum! {
    pub enum DrawerAnchor {
        #[default]
        Left = "left",
        Right = "right",
        Top = "top",
        Bottom = "bottom",
    }
}

input_from_str!(DrawerAnchor);

// TODO: `anchor` here only picks the border/axis, not real placement - callers
// must still order `Drawer` correctly themselves. Consider an API that also
// positions it (e.g. order/margin-auto) so `anchor` is authoritative here too.

fn drawer_variables(props: &DrawerProps) -> Variables {
    variables()
        .with(
            DRAWER_SIZE.override_var(),
            props.size.resolve(Some(DRAWER_SIZE)),
        )
        .with(DRAWER_Z_INDEX_VAR, props.z_index.resolve(None))
}

base_props! {
    pub struct DrawerProps {
        #[props(default, into)]
        variant: Input<DrawerVariant>,
        /// On `Static`, only picks the border side/size axis, not placement -
        /// position it yourself in your own layout. Full effect on `Temporary`.
        #[props(default, into)]
        anchor: Input<DrawerAnchor>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        /// Requested by Escape/backdrop on `Temporary`; on `Static` it's only
        /// for your own close button - `Drawer` tracks no open/closed state.
        #[props(default)]
        onclose: EventHandler<()>,
        children: Element,
    }
}

/// A panel anchored to one edge - `Temporary` (default) is a modal drawer;
/// `Static` is a plain in-flow panel (e.g. a sidebar). Compose both, hidden
/// via CSS on opposite breakpoints, for a responsive sidebar-becomes-drawer.
#[component]
pub fn Drawer(props: DrawerProps) -> Element {
    let variant = props.variant.copied_or_default();
    let anchor = props.anchor.copied_or_default();
    let variables = drawer_variables(&props);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("anchor-left", anchor == DrawerAnchor::Left)
        .with("anchor-right", anchor == DrawerAnchor::Right)
        .with("anchor-top", anchor == DrawerAnchor::Top)
        .with("anchor-bottom", anchor == DrawerAnchor::Bottom);

    // Registered for both variants, and paired with the `use_portal(None)`
    // below: hook slots are positional, and `variant` is a caller prop that
    // can flip (the docs compose a Static and a Temporary drawer on
    // opposite breakpoints), so neither hook may sit behind this return.
    let drawer_class = use_css(Some(&DRAWER_TEMPORARY_SX), crate::CssLayer::Framework);

    if variant == DrawerVariant::Static {
        use_portal(None);
        return rsx! {
            Box {
                class: props.class,
                sx: props.sx,
                states,
                variables,
                framework_sx: &DRAWER_STATIC_BASE_SX,
                attributes: props.attributes,
                ScrollArea {
                    sx: sx().padding("lg"),
                    {props.children}
                }
            }
        };
    }

    let class = props.class.clone().unwrap_or_default().with(drawer_class);
    let (placement, float_sx) = drawer_float_placement(anchor);

    let onclose = props.onclose;
    let z_index = props.z_index.clone();
    let sx = props.sx.clone();
    let attributes = props.attributes.clone();
    let children = props.children.clone();

    use_portal(Some(rsx! {
        Modal {
            onclose: move |_| onclose.call(()),
            Float {
                placement: Input::Value(placement),
                z_index: z_index.clone(),
                sx: float_sx.clone(),
                Dialog {
                    class: class.clone(),
                    sx: sx.clone(),
                    states: states.clone(),
                    variables: variables.clone(),
                    attributes: attributes.clone(),
                    {children.clone()}
                }
            }
        }
    }));

    rsx! {}
}
