use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use crate::icons::CheckmarkIcon;
use dioxus::prelude::*;
use libero::{
    components::{
        ActionIcon, Box, Code, GridSpan, Image, ImageBar, ImageItem, ImageList, Input, Text,
    },
    sx::sx,
    use_theme,
};

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

/// Which cells the `span` switch widens, and which the `rows` switch heightens.
/// Two of six each, so the demo shows the *mix* - a gallery of equal cells is
/// what hid a whole class of defect from the first browser pass.
const WIDE: [usize; 2] = [0, 3];
const TALL: [usize; 2] = [0, 3];

/// Twice the width `cols` gives an ordinary cell, so the `span` control
/// actually changes something at every column count.
///
/// `GridSpan::Half` looked like the obvious choice and is a **no-op at the
/// default `cols: 2`**, where an ordinary cell is already half the zone -
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
    let span = values.str("span") == "true";
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
                && WIDE.contains(&index)
                && let Some(wide) = wide_span(cols)
            {
                item = item.span(wide);
            }
            if rows && TALL.contains(&index) {
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
    if values.str("span") == "true"
        && let Some(wide) = wide_span(values.str("cols").parse::<u8>().unwrap_or(2))
    {
        chain.push_str(&format!(
            "\n            // Two of the six, twice the width `cols` gives the rest.\n            .span(GridSpan::{})",
            match wide {
                GridSpan::Full => "Full",
                GridSpan::TwoThirds => "TwoThirds",
                GridSpan::Half => "Half",
                _ => "Third",
            }
        ));
    }
    if values.str("rows") == "true" && values.str("variant") == "quilted" {
        chain.push_str("\n            .rows(2)");
    }
    if values.str("link") == "true" {
        chain.push_str("\n            .to(Route::Photo { id: p.id })");
    }

    format!(
        "items: photos\n    .iter()\n    .map(|p| {{\n        ImageItem::new(rsx! {{\n            \
         Image {{ src: p.url.clone(), alt: p.alt.clone(), fit: \"cover\" }}\n        }}){chain}\n    }})\n    \
         .collect()"
    )
}

#[component]
pub fn ImageListPage() -> Element {
    let theme = use_theme();
    let defaults = &theme.image_list;

    rsx! {
        DocPage {
            title: "ImageList",
            source: "libero/src/components/layout/image_list/image_list.rs",
            markdown: "/md/image_list.md",
            properties: vec![
                props("ImageList", vec![
                    prop("items", "Vec<ImageItem>").doc("One cell each, in render order."),
                    prop("cols", "u8")
                        .default(defaults.cols.to_string())
                        .doc("Columns. Snapped to a divisor of twelve - 1, 2, 3, 4, 6 or 12 - because a cell is a span of a `GridZone`'s twelve tracks. One value, not one per breakpoint: use `sx().breakpoint(..)` until per-breakpoint props land."),
                    prop("variant", "ImageListVariant")
                        .default(defaults.variant.as_str())
                        .doc("`standard` gives every cell the same height; `masonry` keeps each picture's own and packs them with no dead space; `quilted` lets a cell take more than one row; `woven` shortens every second cell to 70%."),
                    prop("gap", "Size")
                        .default(defaults.gap.as_str())
                        .doc("Between cells."),
                    prop("radius", "Size")
                        .default(defaults.radius.as_str())
                        .doc("Each cell's corner radius."),
                    prop("ratio", "f32")
                        .default("1.0, from theme.aspect_ratio")
                        .doc("Cell aspect ratio, e.g. `16.0 / 9.0`. Ignored by `masonry`, where the picture's own height is the point. Under `quilted` it is the ratio of *one* cell of the quilt, and a taller or wider cell scales from it."),
                ]),
                props("ImageItem", vec![
                    prop("new(content)", "Element").doc("The cell's content - an `Image` with `fit: \"cover\"`, usually."),
                    prop("span(span)", "GridSpan").doc("This cell's width, overriding the one `cols` derives. The same twelfths a `GridItem` takes, so a span means here exactly what it means there."),
                    prop("rows(rows)", "u8").doc("This cell's height, in rows - `quilted`'s whole vocabulary. Ignored by every other variant: `standard` has one row per cell by definition and `masonry` derives the span from the measured height."),
                    prop("bar(bar)", "ImageBar").doc("The caption strip."),
                    prop("to(target)", "NavigationTarget").doc("Makes the cell a link. The anchor is the picture and a stretched `::after` extends the hit area over the tile, so the accessible name is the image's `alt` - a decorative image (`alt: \"\"`) leaves the link unnamed. The bar sits above the hit area, so a control in it still works."),
                ]),
                props("ImageBar", vec![
                    prop("new(content)", "Element").doc("The strip's content - anything. A flex row is all `ImageBar` adds, so a text block wants `flex: 1 1 auto; min-width: 0` and a `<button>` wants `color: inherit`. Sibling content, never a label for the image."),
                    prop("position(position)", "BarPosition")
                        .default(defaults.bar_position.as_str())
                        .doc("`bottom`, `top` or `below` - overriding the theme's, per cell. Where the strip sits is the cell's own layout, so it stays a prop."),
                    prop("scrim(on)", "bool")
                        .default("true")
                        .doc("The gradient behind an overlay bar, and the light text colour with it. Off hands you a bare transparent strip. Mind the contrast: white on the default scrim is 9.3:1 at the strip's bottom edge but 1.8:1 near its top, so a second line over a bright picture is the case that fails - a `text-shadow` is the cheap fix."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A grid of pictures, each with an optional caption bar. Renders a "
                    Code { source: "<ul role=\"list\">" }
                    " of "
                    Code { source: "<li>" }
                    "s, so a gallery is announced with a count - and every cell is a "
                    Code { source: "GridItem" }
                    " of a "
                    Code { source: "GridZone" }
                    ", so "
                    Code { source: "cols" }
                    " and "
                    Code { source: "ImageItem::span" }
                    " are spans of the library's own twelve tracks rather than a second "
                    "grid with its own track count."
                }
                Text {
                    "Three things the controls below show but cannot say. "
                    Code { source: "ImageItem::to" }
                    " makes the picture the anchor and stretches its hit area over the "
                    "whole tile, so the link's accessible name is the image's "
                    Code { source: "alt" }
                    " - a decorative image leaves it unnamed. The bar sits above that "
                    "hit area, so a control you put in it still works. And the scrim is "
                    "a gradient, so white text on it runs from 9.3:1 at the strip's "
                    "bottom edge to 1.8:1 near its top: a second line over a bright "
                    "picture is the case to check."
                }
            },
            Demo {
                component: "ImageList",
                children_text: "",
                controls: vec![
                    Control::slider("cols", ["1", "2", "3", "4", "6"])
                        .default(defaults.cols.to_string())
                        // Unquoted: `cols` is a `u8`, not a string.
                        .code(|control, values| {
                            let value = values.str("cols");
                            match value == control.default {
                                true => vec![],
                                false => vec![format!("cols: {value}u8")],
                            }
                        }),
                    Control::toggle("variant", ["standard", "masonry", "quilted", "woven"])
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
                    // Everything from here down is an `ImageItem` builder call
                    // rather than a prop, so `bar` prints the whole `items`
                    // block for all of them and the rest print nothing.
                    // Nothing is wider than a one-column cell, so the control
                    // would be a no-op there.
                    Control::switch("span")
                        .hidden_when(|values| values.str("cols") == "1")
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
                        cols: values.str("cols").parse::<u8>().unwrap_or(defaults.cols),
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
            }
        }
    }
}
