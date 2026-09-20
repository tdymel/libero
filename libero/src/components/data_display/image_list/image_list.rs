use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, States, Variables, base_props, input_from_str, inset_focus_ring_sx,
            variables,
        },
        data_display::LinkedImageScope,
        layout::{InternalAnchor, use_box},
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

/// `#[props(into)]` cannot chain `u8 -> Input<u8>` on its own.
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

/// The column counts a twelve-track `GridZone` expresses exactly; others snap.
const COLUMN_COUNTS: [u8; 6] = [1, 2, 3, 4, 6, 12];

/// A tie snaps down, to the wider cell: 5 becomes 4.
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

/// A quilted cell's height, `rowHeight * rows + gap * (rows - 1)`, from its width
/// via `padding-top` (Blitz has no `cqi`). An aspect ratio can't carry the gaps (todo 451).
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

/// The spans above the base, as viewport queries. User layer, or the base
/// span's recycled class wins.
fn span_breakpoints(spans: Responsive<GridSpan>) -> Sx {
    spans.breakpoints().fold(sx(), |cell, (size, span)| {
        cell.breakpoint(size, sx().grid_column(format!("span {}", span.columns())))
    })
}

/// Every variant but `masonry` sizes the cell from the list's ratio.
const RATIO_BOX_STATE: &str = "ratio-box";

/// A quilted cell's [`quilt_height`], published on its `<li>`.
const QUILT_HEIGHT_VAR: CssVar = CssVar::new("--lsx-image-list-quilt-height");

/// The `<ul>`: the list reset, plus `woven`, whose rules span neighbouring cells.
static IMAGE_LIST_SX: StaticSx = StaticSx::new(|| {
    sx().list_style("none")
        .margin("0")
        .padding("0")
        // `1fr` keeps rows even where a `Below` bar makes one taller.
        .when(
            ImageListVariant::Quilted.state_name(),
            sx().grid_auto_rows("1fr"),
        )
        .when(
            ImageListVariant::Woven.state_name(),
            // Every second cell takes 70% of its row, centred: `standard` with
            // one cell in two cropped.
            sx().align_items("center")
                .selector("& > li", sx().height("100%"))
                .selector("& > li:nth-of-type(even)", sx().height("70%")),
        )
});

/// The `<li>`: a one-column grid, and the only positioned box in the cell. A
/// positioned bar would clip a `to` link's stretched `::after` to the caption.
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
            // `AspectRatio`'s themed var. The 100% pair fills a stretched cell in a
            // mixed-span row, which a ratio box alone would not.
            sx().aspect_ratio(ASPECT_RATIO.overridable())
                .width("100%")
                .height("100%")
                .selector("& > *", sx().height("100%").object_fit("cover")),
        )
        // Overrides the ratio box: the cell's `::before` sizes the row.
        .when(
            ImageListVariant::Quilted.state_name(),
            sx().aspect_ratio("auto"),
        )
        .when(
            // A cell keeps the picture's own height, which the packing measures.
            ImageListVariant::Masonry.state_name(),
            sx().selector("& > *", sx().height("auto")),
        )
}

/// The anchor is the picture, stretched over the tile by `::after`, so a
/// `<button>` in the bar is not nested in it.
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
/// A captioned cell's `figure`, boxless, so picture and bar stay the `<li>`'s grid items.
static IMAGE_LIST_FIGURE_SX: StaticSx = StaticSx::new(|| sx().display("contents").margin("0"));
static IMAGE_LIST_MEDIA_LINK_SX: StaticSx = StaticSx::new(|| media_base().and(link_base()));

/// The scrim's `data-state` token, one per position: `when` matches a single token.
fn scrim_state(position: BarPosition) -> &'static str {
    match position {
        BarPosition::Bottom => "bar-scrim-bottom",
        BarPosition::Top => "bar-scrim-top",
        // In flow on the page's own background - there is nothing to scrim.
        BarPosition::Below => "",
    }
}

static IMAGE_LIST_BAR_SX: StaticSx = StaticSx::new(|| {
    // The picture's grid cell, never `position: absolute` (see `IMAGE_LIST_CELL_SX`).
    let overlay = sx().grid_row("1").grid_column("1");

    sx().display("flex")
        .align_items("center")
        .gap(SizeCss::SPACING.value(Size::Sm))
        .padding(IMAGE_LIST_BAR_PADDING.value())
        // Above the stretched link, so bar controls click. No `position`: a
        // grid item takes `z-index` without it.
        .z_index("1")
        .when(
            BarPosition::Bottom.state_name(),
            overlay.clone().align_self("end"),
        )
        .when(BarPosition::Top.state_name(), overlay.align_self("start"))
        // The implicit second row, on the page's background: top padding only.
        .when(
            BarPosition::Below.state_name(),
            sx().grid_row("2")
                .grid_column("1")
                .padding_left("0")
                .padding_right("0")
                .padding_bottom("0"),
        )
        // The light text rides the scrim: with it off, the caller owns the colour.
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
        /// Columns, snapped to a divisor of twelve: `3`, or `responsive(1).sm(2).lg(3)`.
        #[props(default, into)]
        cols: Input<Responsive<u8>>,
        #[props(default, into)]
        variant: Input<ImageListVariant>,
        #[props(default, into)]
        gap: Input<Size>,
        /// Each cell's corner radius.
        #[props(default, into)]
        radius: Input<Size>,
        /// Cell aspect ratio, e.g. `16.0 / 9.0`. Ignored by `masonry`.
        #[props(default, into)]
        ratio: Input<f32>,
    }
}

/// A grid of pictures, each with an optional caption bar.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Image, ImageBar, ImageItem, ImageList};
/// # fn app() -> Element {
/// rsx! {
///     ImageList {
///         cols: 2u8,
///         items: vec![
///             ImageItem::new(rsx! { Image { src: "/a.jpg", alt: "A harbour", fit: "cover" } })
///                 .bar(ImageBar::new(rsx! { "Harbour" })),
///             ImageItem::new(rsx! { Image { src: "/b.jpg", alt: "A lighthouse", fit: "cover" } }),
///         ],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/image-list>
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
    // The list's ratio, over `AspectRatio`'s `-override` twin so the theme default
    // shows through. Not for `quilted`, which publishes per `<li>`.
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

    // Prepared once, above the loop (`use_box` is a hook), and cloned per cell.
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
        // The link is always the picture.
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

        // Every quilted cell: an ordinary one's height is the row height. The
        // registry recycles by content, so one class per shape.
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
            // Relative width moves with `cols`: re-published per breakpoint.
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
    // Safari/VoiceOver drops list semantics under `list-style: none`. The caller's role wins.
    let names_role = props
        .attributes
        .iter()
        .any(|attribute| attribute.name == "role");
    let mut attributes = props.attributes.clone();
    if !names_role {
        attributes.push(crate::components::common::attr("role", "list"));
    }

    // On the root too: `woven`'s rules key off it.
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

    /// Ties (5, 9) go to the wider cell; nothing snaps to zero.
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

    /// An ordinary cell is its width over the ratio, with no gap term.
    #[test]
    fn an_ordinary_quilt_cell_is_its_width_over_the_ratio() {
        let gap = GRID_ZONE_GAP.value();
        // Three columns, so an ordinary cell spans four tracks.
        assert_eq!(
            quilt_height(1.5, 4, 4, 1),
            format!("calc(1 * (100% - 0 * {gap}) / 1.5 + 0 * {gap})")
        );
    }

    /// A spanning cell nets out its column gaps and adds its row gaps.
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

    /// A row of `cols` cells fills the zone exactly.
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
