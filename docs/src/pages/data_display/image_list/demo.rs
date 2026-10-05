use super::*;

/// Six pictures of **different intrinsic heights**, or `masonry` looks like `standard`.
pub(super) static GALLERY: [Asset; 6] = [
    asset!("/assets/gallery/1.svg"),
    asset!("/assets/gallery/2.svg"),
    asset!("/assets/gallery/3.svg"),
    asset!("/assets/gallery/4.svg"),
    asset!("/assets/gallery/5.svg"),
    asset!("/assets/gallery/6.svg"),
];

pub(super) const TITLES: [&str; 6] = ["Breakfast", "Burger", "Camera", "Coffee", "Hats", "Honey"];
/// Made up, as the pictures are placeholders: no real person's handle.
pub(super) const AUTHORS: [&str; 6] = [
    "@demo_cook",
    "@demo_grill",
    "@demo_lens",
    "@demo_brew",
    "@demo_hats",
    "@demo_bees",
];

/// Real pages, so a linked cell navigates rather than 404s.
pub(super) const LINK_TARGETS: [&str; 6] = [
    "/layout/grid",
    "/layout/aspect-ratio",
    "/data-display/image",
    "/layout/grid",
    "/layout/aspect-ratio",
    "/data-display/image",
];

/// The cells `span` widens and `rows` heightens - two of six, so the demo shows
/// the *mix*. Written as the snippet prints it.
pub(super) fn featured(index: usize) -> bool {
    index.is_multiple_of(3)
}

/// Twice an ordinary cell's width at every column count. A fixed `GridSpan::Half` is a
/// no-op at `cols: 2`.
pub(super) fn wide_span(cols: u8) -> Option<GridSpan> {
    match cols {
        2 => Some(GridSpan::Full),
        3 => Some(GridSpan::TwoThirds),
        4 => Some(GridSpan::Half),
        6 => Some(GridSpan::Third),
        // One column is already the whole zone; there is nothing wider.
        _ => None,
    }
}

pub(super) fn picture(index: usize) -> Element {
    rsx! {
        Image {
            src: GALLERY[index].to_string(),
            alt: format!("{} - a stylised landscape", TITLES[index]),
            fit: "cover",
        }
    }
}

/// A caller's caption. `min-width: 0` and ellipses keep a long title from pushing the control
/// out; `color: inherit`, since a `<button>` inherits none and the bar's colour is the scrim's.
pub(super) fn caption(index: usize, mut selected: Signal<[bool; 6]>) -> Element {
    let line = sx()
        .overflow("hidden")
        .text_overflow("ellipsis")
        .white_space("nowrap");
    let on = selected()[index];
    // Filled when selected; otherwise the bar's colour, the scrim's.
    let icon = match on {
        true => sx().flex("0 0 auto"),
        false => sx().color("inherit").flex("0 0 auto"),
    };

    rsx! {
        Box { sx: sx().flex("1 1 auto").min_width("0"),
            Box { sx: line.clone().font_weight("500"), "{TITLES[index]}" }
            Box { sx: line.font_size("0.75rem"), "{AUTHORS[index]}" }
        }
        ActionIcon {
            variant: if on { "filled" } else { "standard" },
            sx: icon,
            size: "sm",
            aria_label: format!("Select {}", TITLES[index]),
            aria_pressed: on,
            onclick: move |_| selected.write()[index] ^= true,
            Pictogram { icon: lucide::check::outlined }
        }
    }
}

pub(super) fn items(values: &DemoValues, selected: Signal<[bool; 6]>) -> Vec<ImageItem> {
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
                    ImageBar::new(caption(index, selected))
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
