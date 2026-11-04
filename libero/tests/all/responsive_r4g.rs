//! Responsive leftovers of todo 5586 (bundle r4g): Timeline align beside an opposite (5673),
//! AvatarGroup size, radius and variant (5672), Table size (5671), ButtonGroup size and
//! variant (5667), a line_clamp's 0 range falling back to truncate (5659).

use crate::common::{body, render, tag_with, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        AvatarGroup, AvatarSpec, Button, ButtonGroup, Table, Text, Timeline, TimelineEvent, column,
    },
    sx::ThemeAwareValue,
    theme::{Size, TimelineAlign, Variant, responsive},
};

/// Whether the page holds a run word.
fn has_run_word(html: &str) -> bool {
    html.split(r#"[data-state~=""#)
        .skip(1)
        .chain(html.split(r#"data-state=""#).skip(1))
        .any(|rest| rest[..rest.find('"').unwrap_or(0)].contains(':'))
}

fn opposed_events() -> Vec<TimelineEvent> {
    ["Ordered", "Shipped"]
        .into_iter()
        .map(|title| TimelineEvent::new(title).opposite(rsx! { "09:00" }))
        .collect()
}

/// The slot order inside each event, as `opposite,body`.
fn slot_orders(html: &str) -> Vec<String> {
    body(html)
        .split(r#"data-slot="item""#)
        .skip(1)
        .map(|item| {
            let body = item.find(r#"data-slot="body""#).unwrap_or(usize::MAX);
            let opposite = item.find(r#"data-slot="opposite""#).unwrap_or(usize::MAX);
            match opposite < body {
                true => "opposite,body".into(),
                false => "body,opposite".into(),
            }
        })
        .collect()
}

#[test]
fn a_responsive_timeline_align_orders_its_events_by_the_base_on_the_server() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Timeline {
                    items: opposed_events(),
                    align: responsive(TimelineAlign::Start).md(TimelineAlign::Alternate),
                }
            }
        }
    }
    fn plain() -> Element {
        rsx! {
            LiberoProvider { Timeline { items: opposed_events(), align: TimelineAlign::Alternate } }
        }
    }
    // `start` puts every body at the end of the rail, so the opposite reads first.
    assert_eq!(
        slot_orders(&render(app)),
        ["opposite,body", "opposite,body"]
    );
    assert_eq!(
        slot_orders(&render(plain)),
        ["opposite,body", "body,opposite"]
    );
}

fn people() -> Vec<AvatarSpec> {
    ["Ada", "Alan", "Grace", "Edsger"]
        .into_iter()
        .map(AvatarSpec::from)
        .collect()
}

#[test]
fn a_responsive_avatar_group_hands_its_axes_to_every_member_and_the_chip() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                AvatarGroup {
                    people: people(),
                    max: 3,
                    size: responsive(Size::Xs).md(Size::Lg),
                    radius: responsive(ThemeAwareValue::from(Size::Xs)).md(Size::Xxl.into()),
                    variant: responsive(Variant::Outlined).md(Variant::Filled),
                }
            }
        }
    }
    fn plain() -> Element {
        rsx! {
            LiberoProvider {
                AvatarGroup { people: people(), max: 3, size: "lg", radius: "0", variant: "filled" }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    let members = tags_with(&page, r#"role="img""#);
    // Two members and the chip.
    assert_eq!(members.len(), 3, "{page}");
    for member in &members {
        let words: Vec<&str> = member["data-state"].split(' ').collect();
        for word in [
            "md-:size-lg",
            "base-md:size-xs",
            "md-:filled",
            "base-md:outlined",
            "md-:radius-xxl",
        ] {
            assert!(words.contains(&word), "{word}: {member:?}");
        }
    }
    let chip = tag_with(&page, r#"aria-label="2 more""#);
    let chip_class = chip["class"].split(' ').next().unwrap().to_string();
    assert!(
        html.contains(&format!(r#".{chip_class}[data-state~="md-:size-lg"]"#)),
        "no size copy on the chip's sheet"
    );

    let plain_html = render(plain);
    assert!(!has_run_word(&plain_html), "run words on a plain group");
    let chip = tag_with(&body(&plain_html), r#"aria-label="2 more""#);
    assert_eq!(chip["data-state"], "grouped size-lg filled", "{chip:?}");
}

#[derive(Clone, PartialEq)]
struct Row {
    name: &'static str,
}

fn rows() -> Vec<Row> {
    vec![Row { name: "Ada" }, Row { name: "Alan" }]
}

#[test]
fn a_responsive_table_size_rides_run_words_and_copies_of_the_table_sheet() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    id: "table",
                    caption: "People",
                    data: rows(),
                    columns: vec![column("Name").value(|row: &Row| row.name.to_string())],
                    size: responsive(Size::Xs).md(Size::Lg),
                }
            }
        }
    }
    fn plain() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    id: "table",
                    caption: "People",
                    data: rows(),
                    columns: vec![column("Name").value(|row: &Row| row.name.to_string())],
                    size: "lg",
                }
            }
        }
    }

    let html = render(app);
    let table = tag_with(&body(&html), r#"id="table""#);
    let words: Vec<&str> = table["data-state"].split(' ').collect();
    assert!(
        words.contains(&"base-md:size-xs") && words.contains(&"md-:size-lg"),
        "{table:?}"
    );
    let class = table["class"].split(' ').next().unwrap().to_string();
    for word in ["base-md:size-xs", "md-:size-lg"] {
        assert!(
            html.contains(&format!(r#".{class}[data-state~="{word}"]"#)),
            "no {word} copy of the table sheet"
        );
    }

    let plain_html = render(plain);
    assert!(!has_run_word(&plain_html), "run words on a plain table");
    let table = tag_with(&body(&plain_html), r#"id="table""#);
    assert!(
        table["data-state"].split(' ').any(|word| word == "size-lg"),
        "{table:?}"
    );
}

#[test]
fn a_responsive_button_group_hands_its_run_words_to_the_buttons_and_copies_their_seams() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ButtonGroup {
                    id: "group",
                    "aria-label": "Style",
                    size: responsive(Size::Xs).md(Size::Md),
                    variant: responsive(Variant::Filled).md(Variant::Outlined),
                    Button { id: "bold", "Bold" }
                    Button { id: "italic", "Italic" }
                }
            }
        }
    }
    fn plain() -> Element {
        rsx! {
            LiberoProvider {
                ButtonGroup { "aria-label": "Style", size: "xs", variant: "outlined",
                    Button { "Bold" }
                    Button { "Italic" }
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    let group = tag_with(&page, r#"id="group""#);
    let group_words: Vec<&str> = group["data-state"].split(' ').collect();
    // The outlined run and the compact run, which its buttons carry too.
    for word in ["md-:outlined", "base-md:compact"] {
        assert!(group_words.contains(&word), "{word}: {group:?}");
    }
    let italic = tag_with(&page, r#"id="italic""#);
    let words: Vec<&str> = italic["data-state"].split(' ').collect();
    for word in [
        "md-:outlined",
        "base-md:filled",
        "md-:size-md",
        "base-md:size-xs",
    ] {
        assert!(words.contains(&word), "{word}: {italic:?}");
    }
    let class = group["class"].split(' ').next().unwrap().to_string();
    assert!(
        html.contains(&format!(r#".{class}[data-state~="md-:outlined"]"#)),
        "no outlined seam copy"
    );

    let plain_html = render(plain);
    assert!(!has_run_word(&plain_html), "run words on a plain group");
}

#[test]
fn a_zero_clamp_range_with_a_truncate_carries_the_truncate_run_word() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Text { id: "text", truncate: "end", line_clamp: responsive(0).md(3), "Long text" }
            }
        }
    }
    let html = render(app);
    let text = tag_with(&body(&html), r#"id="text""#);
    let words: Vec<&str> = text["data-state"].split(' ').collect();
    assert!(words.contains(&"base-md:truncate-end"), "{text:?}");
    assert!(words.contains(&"md-:line-clamp"), "{text:?}");
    assert!(!words.contains(&"line-clamp"), "{text:?}");
    let class = text["class"].split(' ').next().unwrap().to_string();
    assert!(
        html.contains(&format!(r#".{class}[data-state~="base-md:truncate-end"]"#)),
        "no truncate copy"
    );
}
