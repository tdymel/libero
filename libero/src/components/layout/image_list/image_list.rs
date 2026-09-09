use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, LinkedImageScope, States, Variables,
        common::{base_props, input_from_str, variables},
        layout::use_box,
        navigation::InternalAnchor,
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::{
        ASPECT_RATIO, BarPosition, IMAGE_LIST_BAR_BACKGROUND, IMAGE_LIST_BAR_BACKGROUND_TOP,
        IMAGE_LIST_BAR_COLOR, IMAGE_LIST_BAR_PADDING, IMAGE_LIST_RADIUS, ImageListDefaults,
        ImageListVariant, Size, SizeCss,
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

/// A quilted cell's aspect ratio: the list's, scaled by how much bigger the
/// cell is than an ordinary one.
///
/// This is the arithmetic MUI does with `rowHeight * rows + gap * (rows - 1)`,
/// done from `ratio` instead - so a 2x2 cell is exactly twice the size of a
/// 1x1 one and every row lands on the same height without anyone naming a
/// pixel. A fixed pixel height does not survive a responsive column count;
/// an aspect ratio does.
fn quilt_ratio(ratio: f32, columns: u8, default_columns: u8, rows: u8) -> f32 {
    let widths = f32::from(columns) / f32::from(default_columns.max(1));
    ratio * widths / f32::from(rows.max(1))
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

/// Every variant but `masonry` gives the cell a height of its own, from the
/// list's ratio. A token rather than a list of variant names, so a new variant
/// says which family it is in at the one place it is decided.
const RATIO_BOX_STATE: &str = "ratio-box";

/// The `<ul>`. `GridZone` brings the grid, so this is the list reset it has no
/// reason to carry - plus `woven`, which is the only variant whose rules are
/// about the *relationship* between neighbouring cells and so cannot live on
/// a cell's own class.
static IMAGE_LIST_SX: StaticSx = StaticSx::new(|| {
    sx().list_style("none")
        .margin("0")
        .padding("0")
        // Equal rows, with nothing naming a pixel. A cell's own ratio already
        // scales with its width and height (see `quilt_ratio`), so the rows
        // come out near-equal on their own; `1fr` absorbs the residue the gap
        // terms leave, which an aspect ratio cannot see. This is the whole of
        // what MUI needs `rowHeight` for.
        .when(
            ImageListVariant::Quilted.state_name(),
            sx().grid_auto_rows("1fr"),
        )
        .when(
            ImageListVariant::Woven.state_name(),
            // MUI's woven, mechanism included: every cell fills its row, every
            // second one takes 70% of it, and centring is what turns that into
            // an alternating rhythm rather than a ragged bottom edge. The
            // percentages resolve against the row, which the ratio box sizes -
            // so this is `standard` with one cell in two cropped.
            sx().align_items("center")
                .selector("& > li", sx().height("100%"))
                .selector("& > li:nth-of-type(even)", sx().height("70%")),
        )
});

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
            RATIO_BOX_STATE,
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
/// Stretched rather than wrapping the cell: the bar holds whatever the caller
/// rendered, and a `<button>` inside an anchor is invalid HTML - so the anchor
/// is the picture and the tile is covered by a pseudo-element instead.
fn link_base() -> Sx {
    sx().color("inherit").text_decoration("none").selector(
        "&::after",
        sx().content("\"\"").position("absolute").inset("0"),
    )
}

static IMAGE_LIST_MEDIA_SX: StaticSx = StaticSx::new(media_base);
static IMAGE_LIST_MEDIA_LINK_SX: StaticSx = StaticSx::new(|| media_base().and(link_base()));

/// The scrim's `data-state` token, per position: the gradient fades *away*
/// from the edge it sits on, so which one is a function of both the position
/// and whether the caller kept the scrim at all. Encoded as one token rather
/// than two, because `when` matches a single one.
fn scrim_state(position: BarPosition) -> &'static str {
    match position {
        BarPosition::Bottom => "bar-scrim-bottom",
        BarPosition::Top => "bar-scrim-top",
        // In flow on the page's own background - there is nothing to scrim.
        BarPosition::Below => "",
    }
}

static IMAGE_LIST_BAR_SX: StaticSx = StaticSx::new(|| {
    // The picture's own grid cell, aligned to one edge of it - **never**
    // `position: absolute`. See `IMAGE_LIST_CELL_SX`.
    let overlay = sx().grid_row("1").grid_column("1");

    sx().display("flex")
        .align_items("center")
        .gap(SizeCss::SPACING.value(Size::Sm))
        .padding(IMAGE_LIST_BAR_PADDING.value())
        // Above the stretched link, so a control the caller put in the bar is
        // clickable on a cell with a `to`. `z-index` **without** `position`: a
        // grid item takes one either way, and positioning the bar would make
        // it the containing block for the anchor's `::after` - the defect the
        // browser pass found. Nothing inside a cell may be positioned.
        .z_index("1")
        .when(
            BarPosition::Bottom.state_name(),
            overlay.clone().align_self("end"),
        )
        .when(BarPosition::Top.state_name(), overlay.align_self("start"))
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
        // The colour rides the scrim rather than the position: with the scrim
        // off there is no dark backdrop to be light against, and the caller
        // owns the design from there.
        .when(
            scrim_state(BarPosition::Bottom),
            sx().background(IMAGE_LIST_BAR_BACKGROUND.value())
                .color(IMAGE_LIST_BAR_COLOR.value()),
        )
        .when(
            scrim_state(BarPosition::Top),
            sx().background(IMAGE_LIST_BAR_BACKGROUND_TOP.value())
                .color(IMAGE_LIST_BAR_COLOR.value()),
        )
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
    let quilted = variant == ImageListVariant::Quilted;
    let cols = snap_cols(props.cols.copied_or(defaults.cols));
    let gap = props.gap.copied_or(defaults.gap);
    let radius = props.radius.copied_or(defaults.radius);

    if masonry && props.ratio.as_ref().is_some() {
        warn(
            "ImageList: ratio is ignored by the masonry variant - a packed cell takes its height from its own picture.",
        );
    }

    let default_span = span_for_cols(cols);

    let media_states: Input<States> = States::default()
        .with(variant.state_name(), true)
        .with(RATIO_BOX_STATE, !masonry)
        .into();
    // The list's ratio, published once and inherited by every cell, over
    // `AspectRatio`'s own `-override` twin so the theme default still shows
    // through when no caller names one.
    //
    // `quilted` is the exception: there the ratio differs per cell, so it is
    // published on each `<li>` and inherits down instead - a variable here
    // would shadow it.
    let media_variables: Input<Variables> = variables()
        .with(
            ASPECT_RATIO.override_var(),
            props
                .ratio
                .as_ref()
                .filter(|_| !masonry && !quilted)
                .map(f32::to_string),
        )
        .into();
    let base_ratio = props
        .ratio
        .as_ref()
        .copied()
        .unwrap_or(theme.aspect_ratio.ratio);
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

    let cells = props.items.iter().enumerate().map(|(index, item)| {
        // One link path, always on the picture: the bar's content is the
        // caller's, so there is no title element for the anchor to be.
        let media = match &item.to {
            Some(to) => rsx! {
                InternalAnchor {
                    to: to.clone(),
                    framework_sx: Some(&IMAGE_LIST_MEDIA_LINK_SX),
                    states: media_states.clone(),
                    variables: media_variables.clone(),
                    LinkedImageScope { {item.content.clone()} }
                }
            },
            None => media_style
                .clone()
                .render(HtmlTag::Div, Vec::new(), item.content.clone()),
        };

        let bar = item.bar.as_ref().map(|bar| {
            let position = bar.position.unwrap_or(defaults.bar_position);
            let scrim = scrim_state(position);

            bar_style
                .clone()
                .attr(
                    "data-state",
                    States::default()
                        .with(position.state_name(), true)
                        .with(scrim, bar.scrim && !scrim.is_empty())
                        .data_state(),
                )
                .render(HtmlTag::Div, Vec::new(), bar.content.clone())
        });

        let span = item.span.unwrap_or(default_span);
        if item.rows.is_some() && !quilted {
            warn(&format!(
                "ImageList: ImageItem::rows is ignored by the {} variant - only quilted places a \
                 cell over more than one row.",
                variant.as_str()
            ));
        }
        let rows = item.rows.filter(|_| quilted);

        // Every quilted cell, not only a spanning one: the list-level ratio is
        // suppressed under `quilted`, so an ordinary cell has to be told its
        // own or it would fall back to the theme's and ignore the `ratio`
        // prop. One class per distinct shape, not per cell - the registry
        // recycles by content, and a quilt has a handful of shapes.
        let cell_sx: Input<Sx> = match quilted.then(|| {
            quilt_ratio(
                base_ratio,
                span.columns(),
                default_span.columns(),
                rows.unwrap_or(1),
            )
        }) {
            None => Input::Static(&IMAGE_LIST_CELL_SX),
            Some(ratio) => Input::Value(
                Sx::clone(&IMAGE_LIST_CELL_SX).var(ASPECT_RATIO.override_var(), ratio.to_string()),
            ),
        };

        rsx! {
            GridItem {
                key: "{index}",
                component: HtmlTag::Li,
                span,
                rows,
                sx: cell_sx,
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

    // The variant reaches the list itself, not only the cells: `woven`'s rules
    // are about neighbouring cells and key off the root.
    let list_states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(variant.state_name(), true)
        .into();

    rsx! {
        GridZone {
            masonry,
            gap,
            component: HtmlTag::Ul,
            class: props.class.clone(),
            sx: root_sx,
            states: list_states,
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

    /// An ordinary quilted cell keeps the list's ratio, and a cell twice as
    /// wide and twice as tall keeps it too - which is the property that makes
    /// every row the same height without a `rowHeight` in sight.
    #[test]
    fn a_proportional_quilt_cell_keeps_the_lists_ratio() {
        // Three columns, so an ordinary cell spans four tracks.
        assert_eq!(quilt_ratio(1.5, 4, 4, 1), 1.5);
        assert_eq!(quilt_ratio(1.5, 8, 4, 2), 1.5);
    }

    /// A cell that is not proportional is the case the arithmetic exists for:
    /// twice as wide over one row is twice as wide a picture, and twice as
    /// tall over one column is half.
    #[test]
    fn an_out_of_proportion_quilt_cell_scales_on_both_axes() {
        assert_eq!(quilt_ratio(1.0, 8, 4, 1), 2.0);
        assert_eq!(quilt_ratio(1.0, 4, 4, 2), 0.5);
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
