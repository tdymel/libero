use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_props, input_from_str, variables},
        layout::use_box,
        navigation::InternalAnchor,
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::{
        ASPECT_RATIO, BarPosition, IMAGE_LIST_BAR_BACKGROUND, IMAGE_LIST_BAR_BACKGROUND_TOP,
        IMAGE_LIST_BAR_COLOR, IMAGE_LIST_BAR_PADDING, IMAGE_LIST_BAR_SUBTITLE_OPACITY,
        IMAGE_LIST_RADIUS, ImageListDefaults, ImageListVariant, Size, SizeCss, TEXT_FONT_SIZE,
        TEXT_LINE_HEIGHT,
    },
    utils::warn,
};

use super::super::grid::{GridItem, GridSpan, GridZone};
use super::item::ImageItem;

input_from_str!(ImageListVariant);
input_from_str!(BarPosition);

/// `#[props(into)]` cannot chain `u8 -> Input<u8>` on its own, and `Input<u8>`
/// is a local type, so the impls may live here rather than in `input.rs`.
impl From<u8> for Input<u8> {
    fn from(value: u8) -> Self {
        Self::Value(value)
    }
}

impl From<Option<u8>> for Input<u8> {
    fn from(value: Option<u8>) -> Self {
        match value {
            Some(value) => Self::Value(value),
            None => Self::None,
        }
    }
}

/// The column counts a twelve-track zone can express exactly. `ImageList` is
/// projected onto `GridZone` rather than growing a grid of its own, so these
/// are the counts, and anything else snaps to the nearest.
const COLUMN_COUNTS: [u8; 6] = [1, 2, 3, 4, 6, 12];

/// A tie snaps **down**, to the wider cell: 5 becomes 4 rather than 6. Fewer,
/// larger pictures is the failure a caller can live with on a phone.
fn snap_cols(cols: u8) -> u8 {
    if COLUMN_COUNTS.contains(&cols) {
        return cols;
    }

    let snapped = COLUMN_COUNTS
        .into_iter()
        .min_by_key(|count| count.abs_diff(cols))
        .unwrap_or(1);
    warn(&format!(
        "ImageList: cols {cols} is not a divisor of twelve, snapped to {snapped}. A cell spans \
         whole tracks of a twelve-track GridZone."
    ));
    snapped
}

/// The span one cell takes when `cols` cells share a row.
fn span_for_cols(cols: u8) -> GridSpan {
    match cols {
        1 => GridSpan::Full,
        2 => GridSpan::Half,
        3 => GridSpan::Third,
        4 => GridSpan::Quarter,
        6 => GridSpan::Sixth,
        _ => GridSpan::Twelfth,
    }
}

/// The `<ul>`. `GridZone` brings the grid, so this is the list reset it has no
/// reason to carry.
static IMAGE_LIST_SX: StaticSx = StaticSx::new(|| sx().list_style("none").margin("0").padding("0"));

/// The `<li>`. **A one-column grid, and the only positioned box in the cell.**
///
/// An overlay bar is a second item in the same grid cell rather than a
/// `position: absolute` strip, and that is load-bearing rather than a matter of
/// taste: a positioned bar is itself a containing block, so a `to` cell's
/// stretched `::after` - which lives on the anchor inside it - resolved against
/// the *bar* and the hit area stopped at the caption. Found in a browser by
/// hit-testing the tile's four corners; SSR sees the correct `inset: 0` either
/// way. Nothing here may become positioned again.
static IMAGE_LIST_CELL_SX: StaticSx = StaticSx::new(|| {
    ImageListDefaults::theme_vars()
        .position("relative")
        .display("grid")
        .grid_template_columns("minmax(0, 1fr)")
        .overflow("hidden")
        .border_radius(IMAGE_LIST_RADIUS.value())
});

/// Always the cell's first row. An overlay bar shares it; a `Below` bar takes
/// the implicit second one.
fn media_base() -> Sx {
    sx().grid_row("1")
        .grid_column("1")
        .min_height("0")
        .overflow("hidden")
        .selector("& > *", sx().width("100%").display("block"))
        .when(
            ImageListVariant::Standard.state_name(),
            // `AspectRatio`'s var pair, not one of our own: it is the same
            // concept, it already carries a themed default, and a caller who
            // retunes `theme.aspect_ratio` gets galleries that match.
            //
            // `height: 100%` **beside** the ratio, and the pair is what makes
            // a mixed-span row work. A box with a preferred aspect ratio is
            // not stretched by `align-self`, so in a row where one cell spans
            // wider - and is therefore taller - every other picture stayed at
            // its own ratio and left dead space below it: 256px of white under
            // a 252px picture in a 508px cell, measured on the docs page's own
            // Spans section. A percentage against the *indefinite* height of an
            // unstretched cell computes to `auto`, so the ratio still decides
            // the ordinary case; against a stretched cell it computes to the
            // cell and the picture covers it.
            // `width: 100%` for the same reason on the other axis: a ratio box
            // is not stretched by `justify-self` either, so with only the
            // height pinned the picture grew to `height * ratio` - 508px wide
            // in a 252px cell - and `overflow: hidden` cropped it off-centre.
            // With both percentages resolved the ratio steps aside, the box is
            // the cell, and `object-fit: cover` does the cropping.
            sx().aspect_ratio(ASPECT_RATIO.overridable())
                .width("100%")
                .height("100%")
                .selector("& > *", sx().height("100%").object_fit("cover")),
        )
        .when(
            // The whole point of the variant: a cell keeps the picture's own
            // height, which is what the packing engine then measures.
            ImageListVariant::Masonry.state_name(),
            sx().selector("& > *", sx().height("auto")),
        )
}

/// A link that is a link and nothing else, plus the hit area over the tile.
/// Stretched rather than wrapping the cell: the bar's action is a `<button>`,
/// and a button inside an anchor is invalid HTML.
fn link_base() -> Sx {
    sx().color("inherit").text_decoration("none").selector(
        "&::after",
        sx().content("\"\"").position("absolute").inset("0"),
    )
}

fn title_base() -> Sx {
    sx().display("block")
        .font_weight("500")
        .overflow("hidden")
        .text_overflow("ellipsis")
        .white_space("nowrap")
}

static IMAGE_LIST_MEDIA_SX: StaticSx = StaticSx::new(media_base);
static IMAGE_LIST_MEDIA_LINK_SX: StaticSx = StaticSx::new(|| media_base().and(link_base()));
static IMAGE_LIST_TITLE_SX: StaticSx = StaticSx::new(title_base);
static IMAGE_LIST_TITLE_LINK_SX: StaticSx = StaticSx::new(|| title_base().and(link_base()));

static IMAGE_LIST_BAR_SX: StaticSx = StaticSx::new(|| {
    // The picture's own grid cell, aligned to one edge of it - **never**
    // `position: absolute`. See `IMAGE_LIST_CELL_SX`.
    let overlay = sx()
        .grid_row("1")
        .grid_column("1")
        .color(IMAGE_LIST_BAR_COLOR.value());

    sx().display("flex")
        .align_items("center")
        .gap(SizeCss::SPACING.value(Size::Sm))
        .padding(IMAGE_LIST_BAR_PADDING.value())
        .when(
            BarPosition::Bottom.state_name(),
            overlay
                .clone()
                .align_self("end")
                .background(IMAGE_LIST_BAR_BACKGROUND.value()),
        )
        .when(
            BarPosition::Top.state_name(),
            overlay
                .align_self("start")
                .background(IMAGE_LIST_BAR_BACKGROUND_TOP.value()),
        )
        // The implicit second row: in flow under the picture, on the page's own
        // background - so it takes the page's text colour and only its vertical
        // padding.
        .when(
            BarPosition::Below.state_name(),
            sx().grid_row("2")
                .grid_column("1")
                .padding_left("0")
                .padding_right("0")
                .padding_bottom("0"),
        )
});

static IMAGE_LIST_TEXT_SX: StaticSx = StaticSx::new(|| sx().flex("1 1 auto").min_width("0"));

static IMAGE_LIST_SUBTITLE_SX: StaticSx = StaticSx::new(|| {
    // A concrete step, never `0.85em`: the bar is a sibling of the picture and
    // sits on the cell, so a relative size would resolve against whatever the
    // caller's page happens to be - the trap `Blockquote`'s caption fell into.
    sx().font_size(TEXT_FONT_SIZE.value(Size::Sm))
        .line_height(TEXT_LINE_HEIGHT.value(Size::Sm))
        .opacity(IMAGE_LIST_BAR_SUBTITLE_OPACITY.value())
        .overflow("hidden")
        .text_overflow("ellipsis")
        .white_space("nowrap")
});

/// Above the stretched link, or a `to` cell would swallow every click on it.
static IMAGE_LIST_ACTION_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .z_index("1")
        .flex("0 0 auto")
        .display("flex")
        .align_items("center")
});

base_props! {
    pub struct ImageListProps {
        /// One cell each, in render order.
        #[props(default)]
        items: Vec<ImageItem>,
        /// Columns. Snapped to a divisor of twelve, since a cell is a span of
        /// a `GridZone`'s twelve tracks.
        ///
        /// One value, not one per breakpoint: `sx().breakpoint(..)` is the
        /// way to change it with the viewport until per-breakpoint props land.
        #[props(default, into)]
        cols: Input<u8>,
        /// `"standard"` - every cell the same height - or `"masonry"`.
        #[props(default, into)]
        variant: Input<ImageListVariant>,
        #[props(default, into)]
        gap: Input<Size>,
        /// Each cell's corner radius.
        #[props(default, into)]
        radius: Input<Size>,
        /// Cell aspect ratio, e.g. `16.0 / 9.0`. Ignored by `masonry`, where
        /// the picture's own height is the point.
        #[props(default, into)]
        ratio: Input<f32>,
    }
}

/// A grid of pictures, each with an optional caption bar.
///
/// Renders a `<ul role="list">` of `<li>`s over a [`GridZone`], so a gallery
/// is announced with a count and packs on the library's own twelve-track grid
/// - `masonry` is that zone's measuring engine, not a CSS multi-column, so
/// reading order and visual order agree.
///
/// Each picture's accessible name is its own `alt`; `ImageList` never invents
/// one, and a bar is sibling content rather than a label for the image.
///
/// Not a composite widget: no roving focus and no arrow keys. A cell with a
/// `to` or an action contributes native tab stops in document order.
#[component]
pub fn ImageList(props: ImageListProps) -> Element {
    let theme = use_theme();
    let defaults = &theme.image_list;

    let variant = props.variant.copied_or(defaults.variant);
    let masonry = variant == ImageListVariant::Masonry;
    let cols = snap_cols(props.cols.copied_or(defaults.cols));
    let gap = props.gap.copied_or(defaults.gap);
    let radius = props.radius.copied_or(defaults.radius);

    if masonry && props.ratio.as_ref().is_some() {
        warn(
            "ImageList: ratio is ignored by the masonry variant - a packed cell takes its height from its own picture.",
        );
    }

    let default_span = span_for_cols(cols);

    let media_states: Input<States> = States::default().with(variant.state_name(), true).into();
    // The list's ratio, published once and inherited by every cell, over
    // `AspectRatio`'s own `-override` twin so the theme default still shows
    // through when no caller names one.
    let media_variables: Input<Variables> = variables()
        .with(
            ASPECT_RATIO.override_var(),
            props
                .ratio
                .as_ref()
                .filter(|_| !masonry)
                .map(f32::to_string),
        )
        .into();
    let cell_states: Input<States> = States::default()
        .with(radius.radius_state_name(), true)
        .into();

    // Prepared once each, above the loop - `use_box` is a hook, so it can
    // never run per item. Every cell shares one class per element and differs
    // only in its `data-state`, which is a plain attribute. `PinField`'s
    // one-prepared-frame-cloned-per-cell.
    let media_style = use_box()
        .framework_sx(&IMAGE_LIST_MEDIA_SX)
        .states(&media_states)
        .variables(&media_variables)
        .focus_ring(false)
        .prepare();
    let bar_style = use_box()
        .framework_sx(&IMAGE_LIST_BAR_SX)
        .focus_ring(false)
        .prepare();
    let text_style = use_box()
        .framework_sx(&IMAGE_LIST_TEXT_SX)
        .focus_ring(false)
        .prepare();
    let title_style = use_box()
        .framework_sx(&IMAGE_LIST_TITLE_SX)
        .focus_ring(false)
        .prepare();
    let subtitle_style = use_box()
        .framework_sx(&IMAGE_LIST_SUBTITLE_SX)
        .focus_ring(false)
        .prepare();
    let action_style = use_box()
        .framework_sx(&IMAGE_LIST_ACTION_SX)
        .focus_ring(false)
        .prepare();

    let cells = props.items.iter().enumerate().map(|(index, item)| {
        // The anchor goes on the title where there is one, because a title is
        // text and an image's `alt` may legitimately be empty.
        let link_on_title = item.to.is_some() && item.bar.is_some();

        let media = match (&item.to, link_on_title) {
            (Some(to), false) => rsx! {
                InternalAnchor {
                    to: to.clone(),
                    framework_sx: Some(&IMAGE_LIST_MEDIA_LINK_SX),
                    states: media_states.clone(),
                    variables: media_variables.clone(),
                    {item.content.clone()}
                }
            },
            _ => media_style
                .clone()
                .render(HtmlTag::Div, Vec::new(), item.content.clone()),
        };

        let bar = item.bar.as_ref().map(|bar| {
            let position = bar.position.unwrap_or(defaults.bar_position);

            let title = match (&item.to, link_on_title) {
                (Some(to), true) => rsx! {
                    InternalAnchor {
                        to: to.clone(),
                        framework_sx: Some(&IMAGE_LIST_TITLE_LINK_SX),
                        {bar.title.render()}
                    }
                },
                _ => title_style
                    .clone()
                    .render(HtmlTag::Div, Vec::new(), bar.title.render()),
            };
            let subtitle = bar.subtitle.as_ref().map(|subtitle| {
                subtitle_style
                    .clone()
                    .render(HtmlTag::Div, Vec::new(), subtitle.render())
            });
            let text = text_style.clone().render(
                HtmlTag::Div,
                Vec::new(),
                [Some(title), subtitle]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>(),
            );
            let action = bar.action.as_ref().map(|action| {
                action_style
                    .clone()
                    .render(HtmlTag::Div, Vec::new(), action.clone())
            });

            bar_style
                .clone()
                .attr(
                    "data-state",
                    States::default()
                        .with(position.state_name(), true)
                        .data_state(),
                )
                .render(
                    HtmlTag::Div,
                    Vec::new(),
                    [Some(text), action]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>(),
                )
        });

        let span = item.span.unwrap_or(default_span);

        rsx! {
            GridItem {
                key: "{index}",
                component: HtmlTag::Li,
                span,
                sx: &IMAGE_LIST_CELL_SX,
                states: cell_states.clone(),
                {media}
                {bar}
            }
        }
    });
    let cells = cells.collect::<Vec<_>>();

    let root_sx: Input<Sx> = match props.sx.as_ref() {
        None => Input::Static(&IMAGE_LIST_SX),
        Some(caller) => Input::Value(Sx::clone(&IMAGE_LIST_SX).and(caller.clone())),
    };
    // `list-style: none` drops list semantics in Safari with VoiceOver, and a
    // gallery's count is half of why this is a `<ul>`. Not `attr`: the caller
    // may have a better role for their own list.
    let names_role = props
        .attributes
        .iter()
        .any(|attribute| attribute.name == "role");
    let mut attributes = props.attributes.clone();
    if !names_role {
        attributes.push(crate::components::common::attr("role", "list"));
    }

    rsx! {
        GridZone {
            masonry,
            gap,
            component: HtmlTag::Ul,
            class: props.class.clone(),
            sx: root_sx,
            states: props.states.clone(),
            attributes,
            {cells.into_iter()}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_divisor_of_twelve_is_left_alone() {
        for cols in COLUMN_COUNTS {
            assert_eq!(snap_cols(cols), cols);
        }
    }

    /// Both ties in the range - 5 and 9 - go to the wider cell, and nothing
    /// ever snaps to zero.
    #[test]
    fn everything_else_snaps_to_the_nearest_divisor() {
        assert_eq!(snap_cols(0), 1);
        assert_eq!(snap_cols(5), 4);
        assert_eq!(snap_cols(7), 6);
        assert_eq!(snap_cols(8), 6);
        assert_eq!(snap_cols(9), 6);
        assert_eq!(snap_cols(10), 12);
        assert_eq!(snap_cols(200), 12);
    }

    /// A row of `cols` cells fills the zone exactly - the property the whole
    /// snap exists to keep.
    #[test]
    fn a_row_of_cols_cells_fills_the_twelve_tracks() {
        for cols in COLUMN_COUNTS {
            assert_eq!(
                u16::from(span_for_cols(cols).columns()) * u16::from(cols),
                12
            );
        }
    }
}
