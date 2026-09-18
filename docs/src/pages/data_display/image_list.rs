use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, prop, props};
use crate::icons::CheckmarkIcon;
use dioxus::prelude::*;
use libero::{
    components::{
        ActionIcon, Box, Code, GridSpan, Image, ImageBar, ImageItem, ImageList, Input, Text,
    },
    sx::sx,
    theme::{Responsive, Theme, responsive},
    use_theme,
};

/// Four, not the theme's two: at two a wide cell is full width, so no row mixes
/// widths - the case that found the stretched aspect-ratio box.
const DEMO_COLS: &str = "4";

/// The `responsive` switch's columns: one on a phone, two on a tablet, four
/// from a laptop up.
const RESPONSIVE_COLS: Responsive<u8> = responsive(1).sm(2).md(4);

fn cols(values: &DemoValues, fallback: Responsive<u8>) -> Responsive<u8> {
    match values.str("responsive") == "true" {
        true => RESPONSIVE_COLS,
        false => values
            .str("cols")
            .parse::<u8>()
            .map_or(fallback, Into::into),
    }
}

/// Six pictures with six **different intrinsic heights**. Equal ones would
/// make `masonry` look identical to `standard` and demonstrate nothing - the
/// mistake the `Grid` page made with `dense` and three equal cards.
static GALLERY: [Asset; 6] = [
    asset!("/assets/gallery/1.svg"),
    asset!("/assets/gallery/2.svg"),
    asset!("/assets/gallery/3.svg"),
    asset!("/assets/gallery/4.svg"),
    asset!("/assets/gallery/5.svg"),
    asset!("/assets/gallery/6.svg"),
];

const TITLES: [&str; 6] = ["Breakfast", "Burger", "Camera", "Coffee", "Hats", "Honey"];
const AUTHORS: [&str; 6] = [
    "@rgbagirl",
    "@amirali",
    "@bkristastucchio",
    "@nolanissac",
    "@hjrc33",
    "@arwinneil",
];

/// Real pages, so a linked cell navigates rather than 404s.
const LINK_TARGETS: [&str; 6] = [
    "/layout/grid",
    "/layout/aspect-ratio",
    "/data-display/image",
    "/layout/grid",
    "/layout/aspect-ratio",
    "/data-display/image",
];

/// The cells `span` widens and `rows` heightens - two of six, so the demo shows
/// the *mix*. Written as the snippet prints it.
fn featured(index: usize) -> bool {
    index.is_multiple_of(3)
}

/// Twice the width `cols` gives an ordinary cell, so the `span` control
/// actually changes something at every column count.
///
/// `GridSpan::Half` looked like the obvious choice and is a **no-op at
/// `cols: 2`**, where an ordinary cell is already half the zone -
/// caught in a browser, because the demo looked identical with the control on.
/// A span demo has to be relative to the count it sits in.
fn wide_span(cols: u8) -> Option<GridSpan> {
    match cols {
        2 => Some(GridSpan::Full),
        3 => Some(GridSpan::TwoThirds),
        4 => Some(GridSpan::Half),
        6 => Some(GridSpan::Third),
        // One column is already the whole zone; there is nothing wider.
        _ => None,
    }
}

fn picture(index: usize) -> Element {
    rsx! {
        Image {
            src: GALLERY[index].to_string(),
            alt: format!("{} - a stylised landscape", TITLES[index]),
            fit: "cover",
        }
    }
}

/// The bar holds an `Element`, so this is a *caller's* caption rather than the
/// component's: two lines and a control, laid out here.
///
/// Three things worth copying. The text block takes `flex: 1 1 auto` and
/// `min-width: 0`, or a long title pushes the control out of the strip. Every
/// line clips with an ellipsis for the same reason. And the `ActionIcon` is
/// told `color: inherit` - a `<button>` inherits none, and an overlay bar's
/// colour comes from the scrim.
fn caption(index: usize) -> Element {
    let line = sx()
        .overflow("hidden")
        .text_overflow("ellipsis")
        .white_space("nowrap");

    rsx! {
        Box { sx: sx().flex("1 1 auto").min_width("0"),
            Box { sx: line.clone().font_weight("500"), "{TITLES[index]}" }
            Box { sx: line.font_size("0.75rem").opacity("0.72"), "{AUTHORS[index]}" }
        }
        ActionIcon {
            variant: "standard",
            sx: sx().color("inherit").flex("0 0 auto"),
            size: "sm",
            aria_label: format!("Select {}", TITLES[index]),
            CheckmarkIcon {}
        }
    }
}

fn items(values: &DemoValues) -> Vec<ImageItem> {
    let bar = values.str("bar");
    let scrim = values.str("scrim") == "true";
    let cols = values.str("cols").parse::<u8>().unwrap_or(2);
    let span = values.str("span") == "true" && values.str("responsive") != "true";
    let link = values.str("link") == "true";
    let rows = values.str("rows") == "true" && values.str("variant") == "quilted";

    (0..GALLERY.len())
        .map(|index| {
            let mut item = ImageItem::new(picture(index));
            if bar != "none" {
                item = item.bar(
                    ImageBar::new(caption(index))
                        .position(bar.as_str().into())
                        .scrim(scrim),
                );
            }
            if span
                && featured(index)
                && let Some(wide) = wide_span(cols)
            {
                item = item.span(wide);
            }
            if rows && featured(index) {
                item = item.rows(2);
            }
            if link {
                item = item.to(LINK_TARGETS[index]);
            }
            item
        })
        .collect()
}

/// The `items` block, written out the way a caller would type it.
///
/// Five controls feed it and none of them is a prop on `ImageList` - they are
/// builder calls on each `ImageItem`. A control whose effect the block cannot
/// honestly print as `bar: "below"` has to print the thing it actually does.
fn items_code(values: &DemoValues) -> String {
    let mut chain = String::new();

    match values.str("bar").as_str() {
        "none" => {}
        position => {
            let mut bar = "ImageBar::new(caption(p))".to_string();
            if position != "bottom" {
                bar.push_str(&format!(
                    ".position(BarPosition::{})",
                    match position {
                        "top" => "Top",
                        _ => "Below",
                    }
                ));
            }
            if values.str("scrim") != "true" {
                bar.push_str(".scrim(false)");
            }
            chain.push_str(&format!("\n            .bar({bar})"));
        }
    }
    if values.str("link") == "true" {
        chain.push_str("\n            .to(Route::Photo { id: p.id })");
    }

    // Only the featured cells get these, so the block has to pick them out.
    let mut featured = String::new();
    if values.str("span") == "true"
        && values.str("responsive") != "true"
        && let Some(wide) = wide_span(values.str("cols").parse::<u8>().unwrap_or(2))
    {
        featured.push_str(&format!(
            ".span(GridSpan::{})",
            match wide {
                GridSpan::Full => "Full",
                GridSpan::TwoThirds => "TwoThirds",
                GridSpan::Half => "Half",
                _ => "Third",
            }
        ));
    }
    if values.str("rows") == "true" && values.str("variant") == "quilted" {
        featured.push_str(".rows(2)");
    }

    let item = format!(
        "ImageItem::new(rsx! {{\n            \
         Image {{ src: p.url.clone(), alt: p.alt.clone(), fit: \"cover\" }}\n        }}){chain}"
    );
    match featured.is_empty() {
        true => format!(
            "items: photos\n    .iter()\n    .map(|p| {{\n        {item}\n    }})\n    .collect()"
        ),
        false => format!(
            "items: photos\n    .iter()\n    .enumerate()\n    .map(|(i, p)| {{\n        let item = {item};\n        \
             // Every third cell is bigger than `cols` makes the rest.\n        \
             if i.is_multiple_of(3) {{ item{featured} }} else {{ item }}\n    }})\n    .collect()"
        ),
    }
}

/// Names the snippet's placeholders up front, so the pseudo-code is marked as such.
fn placeholder_header(values: &DemoValues, code: &str) -> String {
    let mut names = vec!["`photos` is your own list, each with a `url` and an `alt`".to_string()];
    if values.str("bar") != "none" {
        names.push("`caption(p)` your bar's content".to_string());
    }
    if values.str("link") == "true" {
        names.push("`Route::Photo { id }` your route".to_string());
    }
    format!("// Placeholders: {}.\n{code}", names.join("; "))
}

#[component]
pub fn ImageListPage() -> Element {
    let theme = use_theme();
    let defaults = &theme.image_list;

    rsx! {
        DocPage {
            title: "ImageList",
            source: "libero/src/components/data_display/image_list/image_list.rs",
            markdown: "/md/image_list.md",
            properties: vec![
                props("ImageList", vec![
                    prop("items", "Vec<ImageItem>")
                        .default("vec![]")
                        .doc("One cell each, in render order."),
                    prop("cols", "Responsive<u8>")
                        .default(defaults.cols.to_string())
                        .doc("Columns, as `cols: 3` or one count per breakpoint, `cols: responsive(1).sm(2).md(4)`. Each count snaps to 1, 2, 3, 4, 6 or 12, since a cell spans twelfths of a `GridZone`. An `ImageItem::span` stays the same at every width."),
                    prop("variant", "ImageListVariant")
                        .default(defaults.variant.as_str())
                        .doc("`standard` gives every cell the same height, `masonry` keeps each picture's own and packs them, `quilted` lets a cell take more than one row, and `woven` shortens every second cell to 70%."),
                    prop("gap", "Size")
                        .default(defaults.gap.as_str())
                        .doc("Between cells."),
                    prop("radius", "Size")
                        .default(defaults.radius.as_str())
                        .doc("Each cell's corner radius."),
                    prop("ratio", "f32")
                        .default("1.0, from theme.aspect_ratio")
                        .doc("Cell aspect ratio, such as `16.0 / 9.0`. Ignored by `masonry`. Under `quilted` it is the ratio of one cell, and a bigger cell scales from it."),
                ]),
                props("ImageItem", vec![
                    prop("new(content)", "Element").doc("The cell's content, usually an `Image` with `fit: \"cover\"`."),
                    prop("span(span)", "GridSpan").doc("This cell's width, in the twelfths a `GridItem` takes. Overrides the one `cols` gives."),
                    prop("rows(rows)", "u8").doc("This cell's height in rows. `quilted` only. Other variants ignore it with a warning."),
                    prop("bar(bar)", "ImageBar").doc("The caption strip."),
                    prop("to(target)", "NavigationTarget").doc("Makes the whole cell a link. A `zoomable` `Image` in it draws no zoom button and warns."),
                ]),
                props("ImageBar", vec![
                    prop("new(content)", "Element").doc("The strip's content, laid out as a flex row. Give a text block `flex: 1 1 auto; min-width: 0` and a `<button>` `color: inherit`."),
                    prop("position(position)", "BarPosition")
                        .default(defaults.bar_position.as_str())
                        .doc("`bottom`, `top` or `below`, for this cell."),
                    prop("scrim(on)", "bool")
                        .default("true")
                        .doc("The gradient behind an overlay bar and its light text color. Off leaves a bare transparent strip."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A grid of pictures, each with an optional caption bar. It renders a list, "
                    "so a screen reader announces the count. Each cell is a "
                    Code { source: "GridItem" }
                    " of a "
                    Code { source: "GridZone" }
                    ", so "
                    Code { source: "cols" }
                    " and "
                    Code { source: "ImageItem::span" }
                    " count twelfths, as the rest of the grid does."
                }
            },
            // snippet: item struct Picture { id: u32, url: String, alt: String }
            // snippet: let photos: Vec<Picture> = Vec::new();
            // snippet: item fn caption(_: &Picture) -> Element { rsx! {} }
            // snippet: item #[derive(Clone, PartialEq, Routable)] enum Route { #[route("/photo/:id")] Photo { id: u32 } }
            // snippet: item #[component] fn Photo(id: u32) -> Element { rsx! {} }
            Demo {
                component: "ImageList",
                children_text: "",
                controls: vec![
                    Control::slider("cols", ["1", "2", "3", "4", "6"])
                        .default(DEMO_COLS)
                        .hidden_when(|values| values.str("responsive") == "true")
                        // Omitted at the theme's default, not the demo's, which
                        // differs.
                        .code(|_, values| {
                            let value = values.str("cols");
                            match value == Theme::DEFAULT.image_list.cols.to_string() {
                                true => vec![],
                                false => vec![format!("cols: {value}")],
                            }
                        }),
                    Control::toggle("variant", ["standard", "masonry", "quilted", "woven"])
                        .labels(["Standard", "Masonry", "Quilted", "Woven"])
                        .default(defaults.variant.as_str()),
                    Control::slider("gap", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(defaults.gap.as_str()),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(defaults.radius.as_str()),
                    // `masonry` takes the picture's own height, so `ratio` does
                    // not exist in that state - an ignored prop must not print.
                    Control::toggle("ratio", ["1", "4:3", "16:9"])
                        .hidden_when(|values| values.str("variant") == "masonry")
                        .code(|_, values| match values.str("ratio").as_str() {
                            "4:3" => vec!["ratio: 4.0 / 3.0".to_string()],
                            "16:9" => vec!["ratio: 16.0 / 9.0".to_string()],
                            _ => vec![],
                        }),
                    // Replaces the `cols` slider, which is hidden and so prints
                    // nothing while this is on.
                    Control::switch("responsive").code(|_, values| {
                        match values.str("responsive") == "true" {
                            true => vec![format!("cols: {RESPONSIVE_COLS}")],
                            false => vec![],
                        }
                    }),
                    // Everything from here down is an `ImageItem` builder call
                    // rather than a prop, so `bar` prints the whole `items`
                    // block for all of them and the rest print nothing.
                    // Hidden at one column, where nothing is wider, and under
                    // `responsive`, where a fixed span would not follow `cols`.
                    Control::switch("span")
                        .default("true")
                        .hidden_when(|values| {
                            values.str("cols") == "1" || values.str("responsive") == "true"
                        })
                        .code(|_, _| vec![]),
                    // `rows` is quilted's vocabulary and nothing else's.
                    Control::switch("rows")
                        .default("true")
                        .hidden_when(|values| values.str("variant") != "quilted")
                        .code(|_, _| vec![]),
                    Control::switch("link").code(|_, _| vec![]),
                    Control::toggle("bar", ["none", "bottom", "top", "below"])
                        .default(defaults.bar_position.as_str())
                        .code(|_, values| vec![items_code(values)]),
                    Control::switch("scrim")
                        .default("true")
                        .hidden_when(|values| {
                            matches!(values.str("bar").as_str(), "none" | "below")
                        })
                        .code(|_, _| vec![]),
                ],
                render: move |values: DemoValues| rsx! {
                    ImageList {
                        cols: cols(&values, defaults.cols),
                        variant: values.str("variant"),
                        gap: values.str("gap"),
                        radius: values.str("radius"),
                        // Left unset under `masonry`, which warns on a ratio
                        // it is going to ignore - the control is hidden there,
                        // so the prop must not be set behind its back either.
                        ratio: match (
                            values.str("variant").as_str(),
                            values.str("ratio").as_str(),
                        ) {
                            ("masonry", _) => Input::None,
                            (_, "4:3") => Input::Value(4.0 / 3.0),
                            (_, "16:9") => Input::Value(16.0 / 9.0),
                            _ => Input::None,
                        },
                        items: items(&values),
                    }
                },
                wrap: Wrap(placeholder_header),
            }
            DocSection { title: "Accessibility",
                Text {
                    "Each picture's name is its own "
                    Code { source: "alt" }
                    ". A cell with a bar is a "
                    Code { source: "figure" }
                    ", and the bar is its caption. "
                    Code { source: "ImageItem::to" }
                    " makes the picture the link and stretches it over the tile, so the "
                    "link's name is the image's "
                    Code { source: "alt" }
                    ". A decorative image leaves the link unnamed. The bar sits above the "
                    "link, so a control in it still works."
                }
                Text {
                    "The scrim is a gradient. White text on it runs from 9.3:1 at the bottom "
                    "edge to 1.8:1 near the top, so check a second line over a bright "
                    "picture. A "
                    Code { source: "below" }
                    " bar has no scrim and always reads."
                }
            }
        }
    }
}
