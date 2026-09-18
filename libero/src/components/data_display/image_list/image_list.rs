use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, LinkedImageScope, States, Variables,
        common::{base_props, input_from_str, inset_focus_ring_sx, variables},
        layout::use_box,
        navigation::InternalAnchor,
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::{
        ASPECT_RATIO, BarPosition, CssVar, FOCUS_RING_WIDTH, GRID_ZONE_GAP,
        IMAGE_LIST_BAR_BACKGROUND, IMAGE_LIST_BAR_BACKGROUND_TOP, IMAGE_LIST_BAR_COLOR,
        IMAGE_LIST_BAR_PADDING, IMAGE_LIST_RADIUS, ImageListDefaults, ImageListVariant, Responsive,
        Size, SizeCss,
    },
    utils::warn,
};

use super::item::ImageItem;
use crate::components::layout::{GridItem, GridSpan, GridZone};

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

impl From<u8> for Input<Responsive<u8>> {
    fn from(value: u8) -> Self {
        Self::Value(value.into())
    }
}

impl From<Option<u8>> for Input<Responsive<u8>> {
    fn from(value: Option<u8>) -> Self {
        match value {
            Some(value) => Self::Value(value.into()),
            None => Self::None,
        }
    }
}

impl From<Responsive<u8>> for Input<Responsive<u8>> {
    fn from(value: Responsive<u8>) -> Self {
        Self::Value(value)
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

/// A quilted cell's height, from its own width (`100%` in a `padding-top`,
/// which resolves against the `<li>`'s width; Blitz has no `cqi`):
/// `rowHeight * rows + gap * (rows - 1)`, with the row height being an
/// ordinary cell's width over `ratio`.
///
/// An aspect ratio per cell cannot carry the gap terms: a 2x1 cell's ratio
/// asked for `w + g/2` and `1fr` gave every row that (todo 451). The zone's
/// themed gap stands in for its column gap, which `min(gap, 4%)` can only
/// shrink, so a wide cell asks at most an ordinary cell's height.
fn quilt_height(ratio: f32, columns: u8, default_columns: u8, rows: u8) -> String {
    let widths = f32::from(columns) / f32::from(default_columns.max(1));
    let rows = rows.max(1);
    let gap = GRID_ZONE_GAP.value();
    format!(
        "calc({rows} * (100% - {} * {gap}) / {} + {} * {gap})",
        widths - 1.0,
        widths * ratio,
        rows - 1
    )
}

/// Holds a quilted cell's first row open at [`quilt_height`]: a percentage
/// padding resolves against the grid area's width, the cell's.
fn quilt_strut_sx() -> Sx {
    sx().content("\"\"")
        .grid_row("1")
        .grid_column("1")
        .padding_top(QUILT_HEIGHT_VAR.value())
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

/// The spans above the base, as viewport queries on the cell. User layer, as
/// `GridItem`'s own `sp()` rules, or the base span's recycled class wins.
fn span_breakpoints(spans: Responsive<GridSpan>) -> Sx {
    spans.breakpoints().fold(sx(), |cell, (size, span)| {
        cell.breakpoint(size, sx().grid_column(format!("span {}", span.columns())))
    })
}

/// Every variant but `masonry` gives the cell a height of its own, from the
/// list's ratio. A token rather than a list of variant names, so a new variant
/// says which family it is in at the one place it is decided.
const RATIO_BOX_STATE: &str = "ratio-box";

/// A quilted cell's [`quilt_height`], published on its `<li>`.
const QUILT_HEIGHT_VAR: CssVar = CssVar::new("--lsx-image-list-quilt-height");

/// The `<ul>`. `GridZone` brings the grid, so this is the list reset it has no
/// reason to carry - plus `woven`, which is the only variant whose rules are
/// about the *relationship* between neighbouring cells and so cannot live on
/// a cell's own class.
static IMAGE_LIST_SX: StaticSx = StaticSx::new(|| {
    sx().list_style("none")
        .margin("0")
        .padding("0")
        // Equal rows, with nothing naming a pixel: every cell asks for its
        // `quilt_height`, which is whole rows plus their gaps, and `1fr`
        // keeps the rows even where a `Below` bar makes one taller.
        .when(
            ImageListVariant::Quilted.state_name(),
            sx().grid_auto_rows("1fr"),
        )
        .when(
            ImageListVariant::Woven.state_name(),
            // Every cell fills its row, every
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
        // After the ratio box, which it overrides: the cell's `::before` sizes
        // the row, and `height: 100%` fills it, or a row `1fr` grew.
        .when(
            ImageListVariant::Quilted.state_name(),
            sx().aspect_ratio("auto"),
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
    sx().color("inherit")
        .text_decoration("none")
        .selector(
            "&::after",
            sx().content("\"\"").position("absolute").inset("0"),
        )
        // The link fills the clipped cell, so an outset ring is cut away (todo 618).
        .focus_visible(inset_focus_ring_sx(&format!(
            "calc(-1 * {})",
            FOCUS_RING_WIDTH.value()
        )))
}

static IMAGE_LIST_MEDIA_SX: StaticSx = StaticSx::new(media_base);
/// A captioned cell's `figure`, boxless: picture and bar stay the `<li>`'s grid
/// items, so the one-grid layout above is untouched.
static IMAGE_LIST_FIGURE_SX: StaticSx = StaticSx::new(|| sx().display("contents").margin("0"));
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
        /// Columns, each snapped to a divisor of twelve, since a cell is a span
        /// of a `GridZone`'s twelve tracks.
        ///
        /// `cols: 3`, or per viewport breakpoint: `cols: responsive(1).sm(2).lg(3)`.
        #[props(default, into)]
        cols: Input<Responsive<u8>>,
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
/// one. A cell with a bar is a `figure` and the bar its `figcaption`.
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
    let cols = props.cols.copied_or(defaults.cols).map(snap_cols);
    let gap = props.gap.copied_or(defaults.gap);
    let radius = props.radius.copied_or(defaults.radius);

    if masonry && props.ratio.as_ref().is_some() {
        warn(
            "ImageList: ratio is ignored by the masonry variant - a packed cell takes its height from its own picture.",
        );
    }

    let spans = cols.map(span_for_cols);
    let default_span = spans.base();
    // Shared by every cell `cols` sizes; one class, however many cells.
    let ordinary_cell_sx: Input<Sx> = match spans.breakpoints().next() {
        None => Input::Static(&IMAGE_LIST_CELL_SX),
        Some(_) => Input::Value(Sx::clone(&IMAGE_LIST_CELL_SX).and(span_breakpoints(spans))),
    };

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
    let figure_style = use_box()
        .framework_sx(&IMAGE_LIST_FIGURE_SX)
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
                .render(HtmlTag::Figcaption, Vec::new(), bar.content.clone())
        });
        // A bar captions its picture: `figure` + `figcaption`, else the bare picture.
        let content = match bar {
            Some(bar) => figure_style.clone().render(
                HtmlTag::Figure,
                Vec::new(),
                rsx! {
                    {media}
                    {bar}
                },
            ),
            None => media,
        };

        let span = item.span.unwrap_or(default_span);
        let cell_spans = item.span.map_or(spans, Responsive::new);
        if item.rows.is_some() && !quilted {
            warn(&format!(
                "ImageList: ImageItem::rows is ignored by the {} variant - only quilted places a \
                 cell over more than one row.",
                variant.as_str()
            ));
        }
        let rows = item.rows.filter(|_| quilted);

        // Every quilted cell, not only a spanning one: an ordinary cell's
        // height is the row height the rest are built from. One class per
        // distinct shape, not per cell - the registry recycles by content, and
        // a quilt has a handful of shapes.
        let height_at = |size: Option<Size>| {
            let at = |spans: Responsive<GridSpan>| size.map_or(spans.base(), |size| spans.at(size));
            quilt_height(
                base_ratio,
                at(cell_spans).columns(),
                at(spans).columns(),
                rows.unwrap_or(1),
            )
        };
        let cell_sx: Input<Sx> = match (quilted, item.span) {
            (false, None) => ordinary_cell_sx.clone(),
            (false, Some(_)) => Input::Static(&IMAGE_LIST_CELL_SX),
            // A spanning cell's width relative to an ordinary one moves with
            // `cols`, so its height is re-published at each breakpoint.
            (true, _) => Input::Value(
                spans.breakpoints().fold(
                    Sx::clone(&IMAGE_LIST_CELL_SX)
                        .var(QUILT_HEIGHT_VAR, height_at(None))
                        .selector("&::before", quilt_strut_sx())
                        .and(span_breakpoints(cell_spans)),
                    |cell, (size, _)| {
                        cell.breakpoint(size, sx().var(QUILT_HEIGHT_VAR, height_at(Some(size))))
                    },
                ),
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
                {content}
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

    /// An ordinary cell is its width over the ratio, with no gap term: that
    /// is the row height every other shape is built from.
    #[test]
    fn an_ordinary_quilt_cell_is_its_width_over_the_ratio() {
        let gap = GRID_ZONE_GAP.value();
        // Three columns, so an ordinary cell spans four tracks.
        assert_eq!(
            quilt_height(1.5, 4, 4, 1),
            format!("calc(1 * (100% - 0 * {gap}) / 1.5 + 0 * {gap})")
        );
    }

    /// A spanning cell takes its column gaps off its width and adds its row
    /// gaps to its height, so it lands on whole rows at any gap.
    #[test]
    fn a_spanning_quilt_cell_adds_up_its_gaps() {
        let gap = GRID_ZONE_GAP.value();
        assert_eq!(
            quilt_height(1.0, 8, 4, 2),
            format!("calc(2 * (100% - 1 * {gap}) / 2 + 1 * {gap})")
        );
        assert_eq!(
            quilt_height(1.0, 4, 4, 2),
            format!("calc(2 * (100% - 0 * {gap}) / 1 + 1 * {gap})")
        );
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
