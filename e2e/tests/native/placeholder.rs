//! Blitz draws no `placeholder`, so a framed field draws its own: shown while
//! the control holds no text, hidden once it does, back after clearing.

use dioxus::prelude::*;
use e2e::native::{Key, Page, mount};
use libero::components::{
    Cascader, CascaderOption, MultiSelect, NumberField, PhoneField, Select, TagsField, TextField,
    Textarea,
};

const SHOWN: &str = "[data-lsx-placeholder-shown]";

fn typing(page: &mut Page, text: &str) {
    for c in text.chars() {
        page.press(Key::Character(c.to_string()));
    }
}

/// Pixels along the control's first line that differ from its background.
fn inked(page: &Page, control: &str) -> usize {
    let (x, y, width, _) = page.rect(control);
    let row = (y + 10.0) as u32;
    let points: Vec<(u32, u32)> = (0..60).map(|i| ((x + 2.0) as u32 + i, row)).collect();
    let background = page.painted_pixels(&[((x + width - 4.0) as u32, row)])[0];
    page.painted_pixels(&points)
        .into_iter()
        .filter(|pixel| *pixel != background)
        .count()
}

fn round_trip(mut page: Page, control: &str) {
    assert!(page.exists(SHOWN), "no placeholder shown: {}", page.tree());
    assert!(inked(&page, control) > 0, "the placeholder painted nothing");
    page.click(control);
    typing(&mut page, "a");
    assert!(
        !page.exists(SHOWN),
        "still shown after typing: {}",
        page.tree()
    );
    page.press(Key::Backspace);
    assert!(
        page.exists(SHOWN),
        "not back after clearing: {}",
        page.tree()
    );
}

#[test]
fn a_text_field_draws_its_placeholder_while_empty() {
    fn app() -> Element {
        let mut value = use_signal(String::new);
        rsx! {
            TextField {
                label: "Name",
                placeholder: "Ada Lovelace",
                value: value(),
                oninput: move |next| value.set(next),
            }
        }
    }
    let page = mount(app);
    let (span, input) = (page.rect("[data-lsx-placeholder]"), page.rect("input"));
    let middle = |(_, y, _, height): (f64, f64, f64, f64)| y + height / 2.0;
    assert!(
        (span.0 - input.0).abs() < 1.0 && (middle(span) - middle(input)).abs() <= 1.0,
        "placeholder at {span:?}, input at {input:?}"
    );
    round_trip(page, "input");
}

#[test]
fn an_uncontrolled_text_field_draws_its_placeholder_while_empty() {
    fn app() -> Element {
        rsx! {
            TextField { label: "Name", placeholder: "Ada Lovelace" }
        }
    }
    round_trip(mount(app), "input");
}

#[test]
fn a_placeholder_leaves_the_input_in_place() {
    fn with() -> Element {
        rsx! {
            TextField { label: "Name", placeholder: "Ada Lovelace" }
        }
    }
    fn without() -> Element {
        rsx! {
            TextField { label: "Name" }
        }
    }
    let (with, without) = (mount(with).rect("input"), mount(without).rect("input"));
    assert_eq!(with, without, "the placeholder moved the input");
}

#[test]
fn a_number_field_draws_its_placeholder_while_empty() {
    fn app() -> Element {
        rsx! {
            NumberField::<i32> { label: "Count", placeholder: "0" }
        }
    }
    let mut page = mount(app);
    assert!(page.exists(SHOWN), "no placeholder shown: {}", page.tree());
    page.click("input");
    typing(&mut page, "4");
    assert!(
        !page.exists(SHOWN),
        "still shown after typing: {}",
        page.tree()
    );
}

#[test]
fn a_textarea_draws_its_placeholder_at_the_top() {
    fn app() -> Element {
        let mut value = use_signal(String::new);
        rsx! {
            Textarea {
                label: "Notes",
                placeholder: "Start typing",
                value: value(),
                oninput: move |next| value.set(next),
            }
        }
    }
    let page = mount(app);
    let (span, area) = (page.rect("[data-lsx-placeholder]"), page.rect("textarea"));
    assert!(
        (span.1 - area.1).abs() < 1.0,
        "placeholder at {span:?}, textarea at {area:?}"
    );
    round_trip(page, "textarea");
}

#[test]
fn a_tags_field_draws_its_placeholder_until_a_tag_is_held() {
    fn app() -> Element {
        let mut tags = use_signal(Vec::<String>::new);
        rsx! {
            TagsField {
                label: "Topics",
                placeholder: "Add a topic",
                value: tags(),
                onchange: move |next| tags.set(next),
            }
        }
    }
    let mut page = mount(app);
    assert!(page.exists(SHOWN), "no placeholder shown: {}", page.tree());
    page.click("input");
    typing(&mut page, "rust");
    assert!(!page.exists(SHOWN), "still shown after typing");
    page.press(Key::Enter);
    assert!(
        !page.exists("[data-lsx-placeholder]"),
        "drawn beside a held tag: {}",
        page.tree()
    );
}

/// The leftmost and rightmost inked x inside the frame's border.
fn frame_ink(page: &Page) -> Option<(u32, u32)> {
    let (x, y, width, height) = page.rect("[data-frame]");
    ink_span(page, (x + 2.0, y + 2.0, width - 4.0, height - 4.0))
}

/// The leftmost and rightmost inked x in `rect`, its top and bottom 4px off.
fn ink_span(page: &Page, (x, y, width, height): (f64, f64, f64, f64)) -> Option<(u32, u32)> {
    let rows = (y + 4.0) as u32..(y + height - 4.0) as u32;
    let points: Vec<(u32, u32)> = rows
        .flat_map(|row| (x as u32..(x + width) as u32).map(move |x| (x, row)))
        .collect();
    let background = page.painted_pixels(&[((x + 1.0) as u32, (y + 1.0) as u32)])[0];
    let inked: Vec<u32> = page
        .painted_pixels(&points)
        .into_iter()
        .zip(&points)
        .filter(|(pixel, _)| *pixel != background)
        .map(|(_, (x, _))| *x)
        .collect();
    Some((*inked.iter().min()?, *inked.iter().max()?))
}

/// An open list's search box, after its drawn placeholder.
const SEARCH: &str = "[data-lsx-placeholder] + input[aria-autocomplete=list]";

/// An open list's search box draws its placeholder inside itself until typed in.
fn searches(mut page: Page, text: &str) {
    let shown = format!("{SHOWN} + input[aria-autocomplete=list]");
    assert!(page.exists(&shown), "no placeholder shown: {}", page.tree());
    let drawn: Vec<String> = {
        let doc = page.doc.inner.borrow();
        page.query_all(SHOWN)
            .into_iter()
            .filter_map(|id| Some(doc.get_node(id)?.text_content()))
            .collect()
    };
    assert!(drawn.iter().any(|drawn| drawn == text), "drew {drawn:?}");
    let (x, y, width, height) = page.rect(SEARCH);
    let (left, right) = ink_span(&page, (x + 1.0, y, width - 2.0, height))
        .expect("the search placeholder painted nothing");
    assert!(
        f64::from(left) >= x + 4.0 && f64::from(right) <= x + width,
        "search placeholder ink {left}..{right}, box {x}..{}",
        x + width
    );
    typing(&mut page, "a");
    assert!(!page.exists(&shown), "still shown after typing");
}

#[test]
fn a_select_search_box_draws_its_placeholder() {
    fn app() -> Element {
        rsx! {
            Select {
                label: "Page",
                options: vec!["Theming".to_string(), "Select".to_string()],
                searchable: true,
                search_placeholder: "Find a page",
                value: None::<String>,
                onchange: |_| {},
            }
        }
    }
    let mut page = mount(app);
    page.click("[role=combobox]");
    searches(page, "Find a page");
}

#[test]
fn a_multi_select_search_box_draws_its_placeholder() {
    fn app() -> Element {
        rsx! {
            MultiSelect {
                label: "Pages",
                options: vec!["Theming".to_string(), "Select".to_string()],
                searchable: true,
                search_placeholder: "Find pages",
                value: Vec::<String>::new(),
                onchange: |_| {},
            }
        }
    }
    let mut page = mount(app);
    page.click("[role=combobox]");
    searches(page, "Find pages");
}

#[test]
fn a_cascader_search_box_draws_its_placeholder() {
    fn app() -> Element {
        rsx! {
            Cascader {
                label: "Place",
                searchable: true,
                search_placeholder: "Find a place",
                data: vec![
                    CascaderOption::new("europe", "Europe")
                        .children(vec![CascaderOption::new("paris", "Paris")]),
                ],
                value: None::<String>,
                onchange: |_: Option<String>| {},
            }
        }
    }
    let mut page = mount(app);
    page.click("[role=combobox]");
    searches(page, "Find a place");
}

#[test]
fn a_phone_field_country_search_draws_its_placeholder() {
    fn app() -> Element {
        rsx! {
            PhoneField { label: "Phone" }
        }
    }
    let mut page = mount(app);
    page.click("[data-slot=leading] button");
    searches(page, "Search");
}

const LONG: &str = "A placeholder far longer than the narrow field it sits in, on and on";

#[test]
fn a_long_placeholder_is_clipped_at_the_input_edge() {
    fn app() -> Element {
        rsx! {
            div { width: "200px",
                TextField { label: "Name", placeholder: LONG }
            }
        }
    }
    let page = mount(app);
    let (x, _, width, _) = page.rect("input");
    let (left, right) = frame_ink(&page).expect("the placeholder painted nothing");
    assert!(
        f64::from(left) >= x - 1.0 && f64::from(right) <= x + width + 1.0,
        "placeholder ink {left}..{right} outside the input {x}..{}",
        x + width
    );
}

#[test]
fn a_placeholder_starts_at_the_right_under_rtl() {
    fn app() -> Element {
        rsx! {
            div { dir: "rtl", width: "300px",
                TextField { label: "Name", placeholder: "Ada" }
            }
        }
    }
    let page = mount(app);
    let (x, _, width, _) = page.rect("input");
    let (left, right) = frame_ink(&page).expect("the placeholder painted nothing");
    assert!(
        f64::from(left) >= x + width / 2.0 && f64::from(right) <= x + width + 1.0,
        "placeholder ink {left}..{right}, input {x}..{}",
        x + width
    );
}

/// The typed text sits where the placeholder was, as on the web. Blitz's
/// editor takes no `text-align` (computed `right` here, drawn at the left).
#[test]
#[ignore = "needs Blitz: an input's editor ignores text-align, RTL text starts at the left (todo 904)"]
fn typed_text_starts_at_the_right_under_rtl() {
    fn app() -> Element {
        rsx! {
            div { dir: "rtl", width: "300px",
                TextField { label: "Name", value: "Ada" }
            }
        }
    }
    let page = mount(app);
    let (x, _, width, _) = page.rect("input");
    let (left, right) = frame_ink(&page).expect("the value painted nothing");
    assert!(
        f64::from(left) >= x + width / 2.0 && f64::from(right) <= x + width + 1.0,
        "value ink {left}..{right}, input {x}..{}",
        x + width
    );
}

#[test]
fn a_textarea_placeholder_wraps_inside_it() {
    fn app() -> Element {
        rsx! {
            div { width: "200px",
                Textarea { label: "Notes", placeholder: LONG }
            }
        }
    }
    let page = mount(app);
    assert!(
        !page.wrapped_text("[data-lsx-placeholder]").is_empty(),
        "the placeholder did not wrap"
    );
    let (span, area) = (page.rect("[data-lsx-placeholder]"), page.rect("textarea"));
    assert!(
        span.2 <= area.2 + 0.5,
        "placeholder {span:?} wider than the textarea {area:?}"
    );
}

#[test]
fn the_frame_still_rings_a_focused_input() {
    fn app() -> Element {
        rsx! {
            TextField { label: "Name", placeholder: "Ada" }
        }
    }
    let mut page = mount(app);
    page.click("input");
    let (frame, ring) = (page.rect("[data-frame]"), page.rect("[data-ring]"));
    assert!(
        (ring.0 - frame.0).abs() <= 1.0 && (ring.2 - frame.2).abs() <= 2.0,
        "ring {ring:?} not round the frame {frame:?}"
    );
    assert_ne!(
        page.computed("[data-ring]", "box-shadow"),
        "none",
        "no ring on the focused input"
    );
}

#[test]
fn a_value_set_by_the_app_hides_the_placeholder() {
    fn app() -> Element {
        let mut value = use_signal(String::new);
        // No press, key or input: the value arrives from the app alone.
        use_effect(move || value.set("Ada".into()));
        rsx! {
            TextField { label: "Name", placeholder: "Name", value: value() }
        }
    }
    let page = mount(app);
    assert!(
        !page.exists(SHOWN),
        "still shown after the app set a value: {}",
        page.tree()
    );
}

#[test]
fn a_value_cleared_by_a_timer_shows_the_placeholder() {
    fn app() -> Element {
        let mut value = use_signal(|| "Ada".to_string());
        use_hook(move || {
            let timer = libero::platform::timer().expect("a timer");
            std::rc::Rc::new(timer.after(
                std::time::Duration::from_millis(300),
                Box::new(move || value.set(String::new())),
            ))
        });
        rsx! {
            TextField { label: "Name", placeholder: "Name", value: value() }
            span { id: "cleared", "{value().is_empty()}" }
        }
    }
    let mut page = mount(app);
    // A loaded machine may take the timer's 300ms to mount.
    if page.text("#cleared") == "false" {
        assert!(!page.exists(SHOWN), "shown over a value: {}", page.tree());
    }
    page.wait_for(|page| page.text("#cleared") == "true");
    assert_eq!(page.text("#cleared"), "true", "the timer never fired");
    assert!(
        page.exists(SHOWN),
        "not shown after a timer cleared the value: {}",
        page.tree()
    );
}
