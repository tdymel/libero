//! One happy-path render per component: what it renders as, and the class /
//! `data-state` / custom-property triple that carries its styling.

mod common;

use common::{attributes_of, body, classes_of, has_rule_for, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ActionIcon, Anchor, AspectRatio, Autocomplete, Box, Button, Carousel, Center, Chip, Code,
        CodeBlock, Combobox, ComboboxOption, ComboboxOptionArgs, ComboboxState, Container,
        DataList, DataListItem, Dialog, Divider, FileField, Flex, Float, FocusTrap, Grid, GridArea,
        GridItem, GridSpan, GridTemplate, GridZone, Header, Icon, Image, Kbd, List, ListItem, Mark,
        MultiSelect, NativeSelect, NavLink, OptionLabel, Options, Overlay, Paper, QrCode,
        RangeSlider, ScrollArea, SegmentedControl, Select, SelectionArgs, Sidebar, Slider,
        SliderMark, SliderValue, Splitter, Table, Tabs, Text, Title, Tooltip, Tree, TreeItem,
        TreeNode, TreeNodeRenderArgs, VisuallyHidden, column, sp,
    },
    hooks::{DrawerOptions, ModalScope, use_drawer, use_modal},
    theme::{Color, Size},
};

/// The hidden checkbox is `position: absolute`, so the chip has to be its
/// containing block - the same defect `SegmentedControl` had, where a focused
/// input laid out against the viewport scrolls the whole document.
#[test]
fn a_selectable_chip_contains_its_hidden_checkbox() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { checked: true, onchange: move |_| {}, "tag" }
            }
        }
    }

    let html = render(app);
    let input = classes_of(&html, "input");
    assert!(
        input
            .iter()
            .any(|class| html.contains(&format!(".{class}{{position:absolute"))),
        "{input:?}"
    );

    let positioned = classes_of(&html, "span").iter().any(|class| {
        let rule = format!(".{class}{{");
        html.find(&rule).is_some_and(|at| {
            let base = &html[at..];
            base[..base.find('}').unwrap()].contains("position:relative")
        })
    });
    assert!(positioned, "no positioned class on the chip root");
}

#[test]
fn a_selectable_chip_renders_a_checkbox_its_label_points_at() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { checked: true, onchange: move |_| {}, "tag" }
            }
        }
    }

    let html = render(app);
    let input = attributes_of(&html, "input");

    assert_eq!(input["type"], "checkbox");
    assert!(html.contains("checked=true"));
    // The label points at the input, so clicking the text toggles it.
    assert_eq!(attributes_of(&html, "label")["for"], input["id"]);

    let span = attributes_of(&html, "span");
    assert_eq!(span["data-state"], "filled size-md radius-xl checked");
    // M3's selected filter chip is a tonal container, and one step past
    // `Tonal`'s own resting tint - so selecting an already-tonal chip still
    // reads as a change.
    assert!(span["style"].contains("--lsx-chip-container:var(--lsx-primary-2);"));
    assert!(span["style"].contains("--lsx-chip-on-container:var(--lsx-primary-contrast-2);"));
    assert!(body(&html).contains(">tag<"));
}

#[test]
fn a_clickable_chip_is_a_button_and_a_linked_one_an_anchor() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { onclick: move |_| {}, "act" }
                Chip { to: "https://example.com", target: "_blank", "go" }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "button")["type"], "button");
    assert_eq!(attributes_of(&html, "a")["href"], "https://example.com");
    assert_eq!(attributes_of(&html, "a")["target"], "_blank");
    // The pointer state both roots need, and no `<span>` root gets.
    assert!(attributes_of(&html, "button")["data-state"].contains("clickable"));
}

#[derive(Clone, PartialEq, Options)]
enum Emphasis {
    Bold,
    Italic,
}

#[test]
fn a_segmented_control_checks_only_the_selected_radio() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl {
                    value: Emphasis::Bold,
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    // The first `div` is the field's wrapper; the group is the one inside it.
    let body = &body[body[1..].find("<div").expect("the group") + 1..];
    let root = attributes_of(body, "div");

    // A radio group, not a toolbar: exactly one segment is ever selected, so
    // the semantics are the browser's rather than `aria-pressed`.
    assert_eq!(root["role"], "radiogroup");
    assert_eq!(root["data-state"], "horizontal filled collapsed");

    let checked: Vec<bool> = body
        .match_indices("<input")
        .map(|(at, _)| {
            let tag = &body[at..at + body[at..].find('>').expect("an unterminated tag")];
            tag.contains("checked")
        })
        .collect();
    assert_eq!(checked, [true, false]);

    // A rich label is an icon beside text, so the segment separates its own
    // children - no caller `sx` should be needed for that.
    assert!(html.contains(&format!(
        ".{} > label{{",
        classes_of(&body, "div").first().expect("a framework class")
    )));
    assert!(html.contains("gap:var(--lsx-spacing-xs)"));

    // The derive names the segments, and the radio carries that name because
    // a rich label's text is not it.
    assert!(body.contains("aria-label=\"Bold\""));
    assert!(body.contains("aria-label=\"Italic\""));

    // The selected look is a `data-state`, so one class serves both segments -
    // `classes_of` only ever reads the first tag, hence the split.
    let first_at = body.find("<label").expect("a label");
    let (first, second) =
        body.split_at(first_at + body[first_at + 1..].find("<label").expect("two labels") + 1);
    assert_eq!(classes_of(first, "label"), classes_of(second, "label"));
    assert!(attributes_of(first, "label")["data-state"].contains("checked"));
    assert!(!attributes_of(second, "label")["data-state"].contains("checked"));

    let root_class = classes_of(&body, "div");
    let root_class = root_class.first().expect("a framework class");
    assert!(has_rule_for(&html, root_class));
    // The selected block ties the variant's own `:hover` on specificity, so it
    // has to be emitted after it - a swap would silently lose the selected
    // background under the pointer.
    let selected =
        format!(".{root_class}[data-state~=\"outlined\"] > label[data-state~=\"checked\"]");
    let hover = format!(".{root_class}[data-state~=\"outlined\"] > label:hover");
    assert!(
        html.find(&selected).expect("no selected rule") > html.find(&hover).expect("no hover rule")
    );

    // The inner corners have to beat the segment's own radius rule, which is
    // one `when` shallower - so the attribute selectors are load-bearing.
    assert!(html.contains(&format!(
        ".{root_class}[data-state~=\"collapsed\"][data-state~=\"horizontal\"] > label:not(:first-of-type)"
    )));
}

#[test]
fn a_gapped_segmented_control_keeps_every_segment_s_own_corners() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl {
                    gap: "xs",
                    value: Emphasis::Bold,
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let body = &body[body[1..].find("<div").expect("the group") + 1..];
    let root = attributes_of(body, "div");

    // No `collapsed`, so the corner-squashing rules below cannot match.
    assert_eq!(root["data-state"], "horizontal filled size-xs");

    let root_class = classes_of(&body, "div");
    let root_class = root_class.first().expect("a framework class");
    assert!(html.contains(&format!(
        ".{root_class}[data-state~=\"size-xs\"]{{gap:var(--lsx-spacing-xs)"
    )));
}
#[test]
fn slider_renders_a_thumb_with_the_value_and_its_marks() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    value: 25.0,
                    aria_label: "Volume",
                    marks: vec![SliderMark::labeled(50.0, "half")],
                    format: Callback::new(|value: f64| format!("{value}%")),
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    // The first div is the field wrapper; the slider's own root is the second.
    let root = attributes_of(&html[html.find("<div").unwrap() + 4..], "div");

    assert_eq!(root["data-state"], "size-md marks-labeled");
    assert!(root["style"].contains("--lsx-slider-filled:0.25;"));

    // The thumb carries the a11y contract; the mark carries its position.
    assert!(html.contains(r#"role="slider""#));
    assert!(html.contains(r#"aria-label="Volume""#));
    assert!(html.contains("aria-valuenow=25"));
    assert!(html.contains("--lsx-slider-mark-at:0.5"));
    assert!(body(&html).contains(">half<"));
    // The value bubble is a `Tooltip`.
    assert!(html.contains(r#"role="tooltip""#));
    assert!(body(&html).contains(">25%<"));
}

#[test]
fn a_range_slider_renders_two_thumbs_and_posts_both_values() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                RangeSlider {
                    label: "Price",
                    value: (20.0, 80.0),
                    min_range: 10.0,
                    name: "price",
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let root = attributes_of(&html[html.find("<div").unwrap() + 4..], "div");

    // The bar spans between the thumbs rather than from the track's start.
    assert!(
        root["style"].contains("--lsx-slider-filled-from:0.2;"),
        "{root:?}"
    );
    assert!(
        root["style"].contains("--lsx-slider-filled-span:0.6"),
        "{root:?}"
    );

    assert_eq!(html.matches(r#"role="slider""#).count(), 2);
    assert!(html.contains("--lsx-slider-thumb-at:0.2"));
    assert!(html.contains("--lsx-slider-thumb-at:0.8"));
    assert!(html.contains(r#"aria-label="Minimum""#));
    assert!(html.contains(r#"aria-label="Maximum""#));

    // Each thumb is bounded by its neighbour, `min_range` short of it.
    assert!(html.contains("aria-valuenow=20"));
    assert!(html.contains("aria-valuemax=70"));
    assert!(html.contains("aria-valuenow=80"));
    assert!(html.contains("aria-valuemin=30"));

    // Both ends post under one name, in track order.
    let inputs: Vec<_> = html.match_indices(r#"type="hidden""#).collect();
    assert_eq!(inputs.len(), 2, "{html}");
    assert_eq!(html.matches(r#"name="price""#).count(), 2, "{html}");
    let first = html.find(r#"value="20""#).expect("the lower value");
    let second = html.find(r#"value="80""#).expect("the upper value");
    assert!(first < second, "{html}");
}

#[test]
fn a_file_field_draws_a_hidden_input_and_names_its_control() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    label: "Attachment",
                    placeholder: "No file picked",
                    accept: "image/*",
                    name: "avatar",
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    // The control is named by the label, not by `for` - it is a div with a
    // role, which `for` cannot name.
    let label = attributes_of(&body, "label");
    assert!(!label.contains_key("for"), "{label:?}");
    let id = label["id"].trim_end_matches("-label").to_string();
    assert!(
        body.contains(&format!(r#"aria-labelledby="{id}-label""#)),
        "{body}"
    );
    assert!(body.contains(r#"role="button""#), "{body}");

    // The real input carries everything a form needs, and nothing else does.
    let input = attributes_of(&body, "input");
    assert_eq!(input["type"], "file");
    assert_eq!(input["name"], "avatar");
    assert_eq!(input["accept"], "image/*");
    assert_eq!(input["aria-hidden"], "true");

    assert!(body.contains("No file picked"), "{body}");
}

#[test]
fn a_dropzone_file_field_renders_its_prompt_instead_of_a_frame() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    label: "Attachments",
                    variant: "dropzone",
                    accept: "image/*",
                    multiple: true,
                    onchange: move |_| {},
                    "Drop files here"
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("Drop files here"), "{body}");
    assert!(body.contains(r#"role="button""#), "{body}");
    // A `false` bool is never pushed, so the attribute's presence is the test.
    assert!(body.contains("multiple=true"), "{body}");
    assert!(body.contains(r#"type="file""#), "{body}");
    // The prompt says what the attribute enforces, read off the attribute.
    assert!(body.contains(r#"data-slot="hint""#), "{body}");
    assert!(body.contains(">images<"), "{body}");
}

/// A hand-written `SliderValue`, which is what a caller with a foreign enum
/// writes - and what covers the trait's default `position`/`at`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Tier {
    Free,
    Pro,
    Team,
    Enterprise,
}

impl SliderValue for Tier {
    type Step = usize;

    fn options() -> Option<&'static [Self]> {
        Some(&[Self::Free, Self::Pro, Self::Team, Self::Enterprise])
    }

    fn label(&self) -> String {
        format!("{self:?}")
    }
}

/// An ordered enum makes the slider discrete: range, step grid, marks and
/// every caption come from `SliderValue::options`.
#[test]
fn a_discrete_slider_derives_its_scale_from_the_value_type() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    value: Tier::Pro,
                    aria_label: "Tier",
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let root = attributes_of(&html[html.find("<div").unwrap() + 4..], "div");

    // `Pro` is the second of four options, so the scale is 0..=3.
    assert!(root["style"].contains("--lsx-slider-filled:0.3333333333333333;"));
    assert!(html.contains("aria-valuenow=1"));
    assert!(html.contains("aria-valuemax=3"));
    assert!(html.contains(r#"aria-valuetext="Pro""#));
    assert!(root["data-state"].contains("marks-labeled"));
    for tier in ["Free", "Pro", "Team", "Enterprise"] {
        assert!(body(&html).contains(&format!(">{tier}<")));
    }
}

/// `min`/`max` are written in the value's own type, and `step` is a stride
/// over the options - `step: 1.5` on a `Tier` slider does not compile.
#[test]
fn a_discrete_sliders_bounds_are_typed_and_its_step_counts_options() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    value: Tier::Team,
                    min: Tier::Pro,
                    max: Tier::Enterprise,
                    step: 2,
                    aria_label: "Tier",
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);

    assert!(html.contains("aria-valuemin=1"));
    assert!(html.contains("aria-valuemax=3"));
    // Marks at `Pro` and `Enterprise` only - every second option from `min`.
    assert!(body(&html).contains(">Pro<"));
    assert!(body(&html).contains(">Enterprise<"));
    assert!(!body(&html).contains(">Free<"));
    assert!(!body(&html).contains(">Team<"));
}

/// `#[derive(SliderValue)]` is the whole discrete impl: variants in
/// declaration order are the options, their names are the labels.
#[derive(Clone, Copy, Debug, PartialEq, SliderValue)]
enum Quality {
    Low,
    #[slider(label = "Med")]
    Medium,
    High,
}

#[test]
fn a_derived_slider_value_names_and_orders_its_own_options() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    value: Quality::Medium,
                    aria_label: "Quality",
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);

    assert!(html.contains("aria-valuemax=2"));
    assert!(html.contains("aria-valuenow=1"));
    // The overridden label reaches both the bubble and `aria-valuetext`.
    assert!(html.contains(r#"aria-valuetext="Med""#));
    for label in [">Low<", ">Med<", ">High<"] {
        assert!(body(&html).contains(label));
    }
}

/// The tint is the lightest shade and its label the *contrast* of that shade,
/// not a darker tone of the hue - which our ramp cannot make legible.
#[test]
fn a_tonal_button_labels_its_container_with_that_shade_s_contrast() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { variant: "tonal", color: Color::Warning, "Tint" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "button");

    assert_eq!(attributes["data-state"], "tonal size-md radius-md");
    assert!(attributes["style"].contains("--lsx-button-container:var(--lsx-warning-1);"));
    assert!(
        attributes["style"].contains("--lsx-button-on-container:var(--lsx-warning-contrast-1);")
    );
}

/// Elevated sits on the surface and is separated by its shadow alone - so it
/// paints no container, and the shadow comes from the shared scale.
#[test]
fn an_elevated_button_keeps_the_surface_and_reads_the_elevation_scale() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { variant: "elevated", color: Color::Error, "Lift" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "button");

    assert_eq!(attributes["data-state"], "elevated size-md radius-md");
    assert!(!attributes["style"].contains("--lsx-button-container:"));
    assert!(html.contains("box-shadow:var(--lsx-shadow-xs)"));
    assert!(html.contains("--lsx-shadow-xs:"));
}

#[test]
fn button_renders_its_class_state_and_variables() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { color: Color::Error, "Save" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "button");

    assert_eq!(attributes["type"], "button");
    assert_eq!(attributes["data-state"], "filled size-md radius-md");
    assert!(attributes["style"].contains("--lsx-button-color:var(--lsx-error-6);"));
    assert!(body(&html).contains(">Save<"));

    for class in classes_of(&html, "button") {
        assert!(has_rule_for(&html, &class), "{class} has no CSS rule");
    }
}

#[test]
fn flex_renders_its_children_in_order() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Flex {
                    span { "first" }
                    span { "second" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let first = body.find("first").expect("the first child");
    let second = body.find("second").expect("the second child");

    assert!(first < second);
    assert!(!classes_of(&html, "div").is_empty());
}

#[test]
fn box_renders_as_the_element_it_was_asked_for() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Box { component: "section", "content" }
            }
        }
    }

    let html = render(app);

    assert!(html.contains("<section"));
    assert!(attributes_of(&html, "section").contains_key("class"));
}

#[test]
fn text_carries_its_size_as_a_data_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Text { size: Size::Lg, "body copy" }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "p")["data-state"], "size-lg");
    assert!(body(&html).contains("body copy"));
}

#[test]
fn title_renders_as_its_heading_level() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Title { component: "h2", "Heading" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<h2"));
    assert!(attributes_of(&html, "h2").contains_key("data-state"));
}

#[test]
fn icon_renders_its_variant_and_colour_variables() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Icon { color: Color::Success, "★" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "span");

    assert_eq!(attributes["data-state"], "filled");
    assert!(attributes["style"].contains("--lsx-icon-color:var(--lsx-success-6);"));
}

#[test]
fn kbd_renders_its_key() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Kbd { "Ctrl" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<kbd"));
    assert!(body(&html).contains("Ctrl"));
}

#[test]
fn mark_highlights_with_its_background_variable() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Mark { color: Color::Warning, "highlighted" }
            }
        }
    }

    let html = render(app);

    assert!(attributes_of(&html, "mark")["style"].contains("--lsx-mark-background"));
}

#[test]
fn visually_hidden_stays_in_the_accessibility_tree() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                VisuallyHidden { "screen reader only" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("screen reader only"));
}

#[test]
fn list_renders_its_items() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                List {
                    ListItem { "one" }
                    ListItem { "two" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("<ul"));
    assert_eq!(body.matches("<li").count(), 2);
}

#[test]
fn data_list_pairs_a_label_with_its_value() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DataList {
                    DataListItem { label: rsx! { "Status" }, "Active" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("Status"));
    assert!(body.contains("Active"));
}

#[test]
fn a_table_marks_only_sortable_headers_and_aligns_by_cell_type() {
    #[derive(Clone, PartialEq)]
    struct Row {
        name: &'static str,
        age: u32,
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: vec![
                        Row { name: "Ada", age: 36 },
                        Row { name: "Grace", age: 45 },
                    ],
                    columns: vec![
                        column("Name").value(|row: &Row| row.name.to_string()),
                        column("Age").value(|row: &Row| row.age).sortable(),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    // Two rows of data, plus the header row.
    assert_eq!(body.matches("<tr").count(), 3);
    assert_eq!(body.matches("<td").count(), 4);
    assert!(body.contains("Ada"));
    assert!(body.contains("36"));

    // Only the sortable column advertises a sort affordance.
    assert_eq!(body.matches("aria-sort=\"none\"").count(), 1);
    assert_eq!(body.matches("<button").count(), 1);
    // The arrow is always in the markup, so sorting can't resize the header.
    assert_eq!(body.matches("<svg").count(), 1);

    // The numeric column aligns itself; the text column doesn't say anything.
    assert_eq!(body.matches("data-align=\"end\"").count(), 3);
    assert!(!body.contains("data-align=\"start\""));
    assert_eq!(attributes_of(&html, "table")["aria-label"], "People");
    assert_eq!(body.matches("<th scope=\"col\"").count(), 2);
}

#[test]
fn a_custom_render_replaces_the_cell_body() {
    #[derive(Clone, PartialEq)]
    struct Row {
        score: u32,
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    data: vec![Row { score: 7 }],
                    columns: vec![
                        column("Score")
                            .value(|row: &Row| row.score)
                            .render(|row: &Row| rsx! { Mark { "{row.score} pts" } }),
                    ],
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("7 pts"));
    assert!(body.contains("<mark"));
}

#[derive(Clone, PartialEq, Options)]
enum Pick {
    First,
    Second,
}

#[test]
fn select_renders_its_options_and_marks_the_current_one() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NativeSelect { value: Pick::Second, onchange: move |_| {} }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("<select"));
    assert_eq!(body.matches("<option").count(), 2);
    // The selection is on the `<option>`, not a `value` on the `<select>`:
    // that is what SSR can express, and it needs no second render to appear.
    assert!(body.contains("<option value=\"1\" selected"));
    assert!(!body.contains("<select value="));
}

/// `value: None` is a real state - the field has not been filled in yet - so
/// it selects an entry no one can pick rather than silently taking the first.
#[test]
fn a_select_without_a_value_shows_its_placeholder() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NativeSelect {
                    value: None::<Pick>,
                    placeholder: "Choose one",
                    // Annotated: with `value: None` there is nothing else for
                    // `T` to be inferred from.
                    onchange: move |_: Pick| {},
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("Choose one"));
    assert_eq!(body.matches("<option").count(), 3);
    // Disabled and hidden, so it cannot be picked back once a value is set.
    assert!(body.contains("hidden"));
    assert!(!body.contains("<option value=\"0\" selected"));
}

#[test]
fn a_disabled_select_renders_the_attribute() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NativeSelect {
                    value: Pick::First,
                    disabled: true,
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("disabled"));
}

/// The row hands its content the tab stop and the `disabled` flag through
/// context, so a `TreeItem` needs neither at the call site.
#[test]
fn a_tree_item_takes_its_tabindex_from_the_row() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tree {
                    aria_label: "Files",
                    data: vec![TreeNode::new("a", "Alpha".to_string())],
                    render_node: move |_: TreeNodeRenderArgs<String>| rsx! {
                        TreeItem { "Alpha" }
                    },
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("<button"));
    assert!(body.contains(r#"tabindex="-1""#));
}

#[test]
fn divider_renders_as_a_separator() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Divider { color: Color::Grey }
            }
        }
    }

    let html = render(app);

    assert!(attributes_of(&html, "div")["style"].contains("--lsx-divider-color"));
}

#[test]
fn a_dividers_size_reaches_its_data_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Divider { size: Size::Lg }
            }
        }
    }

    let html = render(app);

    assert!(attributes_of(&html, "div")["data-state"].contains("size-lg"));
}

#[test]
fn container_and_center_render_their_children() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Container {
                    Center { inline: true, "centred" }
                }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("centred"));
    assert!(body(&html).contains("--lsx-center-display-override:inline-flex;"));
}

#[test]
fn aspect_ratio_sets_its_ratio_variable() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                AspectRatio { ratio: 1.5, "boxed" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("--lsx-aspect-ratio-override:1.5;"));
}

#[test]
fn action_icon_labels_itself_for_assistive_technology() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ActionIcon { aria_label: "Close", "×" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "button");

    assert_eq!(attributes["aria-label"], "Close");
    assert_eq!(attributes["type"], "button");
}

#[test]
fn image_renders_its_source_and_alt_text() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Image { src: "/logo.png", alt: "The logo" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "img");

    assert_eq!(attributes["src"], "/logo.png");
    assert_eq!(attributes["alt"], "The logo");
}

#[test]
fn qr_code_renders_an_svg_labelled_for_assistive_technology() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                QrCode { data: "https://example.com", aria_label: "Scan me" }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "div")["aria-label"], "Scan me");
}

#[test]
fn header_renders_as_a_banner_landmark() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Header { "site header" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<header"));
    assert!(body(&html).contains("site header"));
}

#[test]
fn splitter_renders_both_panes_around_a_divider() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Splitter {
                    initial_size: 50.0,
                    panel_a: rsx! { div { "left" } },
                    panel_b: rsx! { div { "right" } },
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("left"));
    assert!(body.contains("right"));
    assert!(body.contains("--lsx-splitter-a:50%;"));
}

/// `touch-action: none` on the drag target is what lets a touch drag start
/// at all - without it the browser claims the gesture for scrolling and no
/// `pointermove` ever arrives. Nothing visible regresses if it goes missing,
/// so assert it reaches the rendered CSS. The drag itself needs real pointer
/// input, which SSR cannot produce.
#[test]
fn splitter_hit_target_opts_out_of_browser_touch_gestures() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Splitter {
                    initial_size: 50.0,
                    panel_a: rsx! { div { "left" } },
                    panel_b: rsx! { div { "right" } },
                }
            }
        }
    }

    assert!(render(app).contains("touch-action:none"));
}

#[test]
fn scroll_area_renders_its_content() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { "scrollable content" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("scrollable content"));
}

/// The default opts the viewport *out* of Chromium's implicit tab stop, which
/// is what every existing call site relies on.
#[test]
fn a_scroll_area_is_not_a_tab_stop_by_default() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { "scrollable content" }
            }
        }
    }

    let html = body(&render(app));

    assert!(html.contains(r#"tabindex="-1""#));
    assert!(!html.contains(r#"tabindex="0""#));
}

#[test]
fn a_focusable_scroll_area_is_a_tab_stop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { focusable: true, "scrollable content" }
            }
        }
    }

    let html = body(&render(app));

    assert!(html.contains(r#"tabindex="0""#));
    assert!(!html.contains(r#"tabindex="-1""#));
}

#[test]
fn float_renders_its_child_with_placement_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Float { placement: "top-end", "floating" }
            }
        }
    }

    let html = render(app);

    let state = &attributes_of(&html, "div")["data-state"];

    assert!(body(&html).contains("floating"));
    assert!(state.contains("vertical-top"), "got {state}");
    assert!(state.contains("horizontal-end"), "got {state}");
}

#[test]
fn a_paper_is_a_surface_with_no_semantics_of_its_own() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Paper { radius: "lg", bordered: true, "sheet" }
            }
        }
    }

    let html = render(app);
    let paper = attributes_of(&body(&html), "div");

    assert_eq!(paper["data-state"], "radius-lg bordered");
    assert!(
        !paper.contains_key("role"),
        "a surface names nothing: {paper:?}"
    );
    for class in classes_of(&body(&html), "div") {
        assert!(
            has_rule_for(&html, &class),
            "{class} is referenced but never emitted"
        );
    }
    assert!(
        html.contains("background:var(--lsx-paper-background);"),
        "{html}"
    );
}

/// The themed default arrives as a plain declaration, not as a token - which
/// is what lets `Dialog` override it in its own base static.
#[test]
fn a_paper_that_names_no_step_carries_no_data_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Paper { "sheet" }
            }
        }
    }

    let paper = attributes_of(&body(&render(app)), "div");

    assert!(!paper.contains_key("data-state"), "{paper:?}");
}

/// `Dialog` is a `Paper`, so the surface has to reach it - and the dialog's
/// own chrome has to survive the composition.
#[test]
fn a_dialog_renders_the_paper_surface_under_its_own_chrome() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Dialog { aria_label: "Inline", "content" }
            }
        }
    }

    let html = render(app);
    let dialog = attributes_of(&body(&html), "div");

    assert_eq!(dialog["role"], "dialog");
    assert!(
        html.contains("background:var(--lsx-paper-background);"),
        "{html}"
    );
    assert!(
        html.contains("border-radius:var(--lsx-dialog-radius, var(--lsx-paper-radius));"),
        "{html}"
    );
    assert!(html.contains("box-shadow:var(--lsx-shadow-xl);"), "{html}");
}

/// The five-part DOM and the a11y wiring in one pass, because every part of it
/// is a projection of the same `(count, current)` pair.
#[test]
fn a_carousel_names_its_slides_and_points_its_controls_at_the_track() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Photos",
                    indicators: true,
                    per_view: 3.0,
                    slides: (0..6).map(|i| rsx! { div { "slide {i}" } }).collect(),
                }
            }
        }
    }

    let html = body(&render(app));
    let root = attributes_of(&html, "section");

    assert_eq!(root["role"], "region");
    assert_eq!(root["aria-roledescription"], "carousel");
    assert_eq!(root["aria-label"], "Photos");

    // Every slide is a named group, and none is hidden: in a real scroll
    // container an offscreen slide is still reachable.
    assert_eq!(html.matches(r#"aria-roledescription="slide""#).count(), 6);
    assert!(html.contains(r#"aria-label="1 of 6""#), "{html}");
    assert!(html.contains(r#"aria-label="6 of 6""#), "{html}");
    assert!(!html.contains("aria-hidden"), "{html}");

    // The controls name the element they scroll, and the track is the tab stop.
    // The track is the only element here that carries an id, and the controls
    // have to name that one rather than whatever `attributes_of` finds first.
    let track_id = html
        .split_once(r#" id=""#)
        .map(|(_, rest)| rest.split('"').next().unwrap_or_default().to_string())
        .expect("the track carries an id");
    assert_eq!(
        html.matches(&format!(r#"aria-controls="{track_id}""#))
            .count(),
        2
    );
    assert!(html.contains(r#"tabindex="0""#), "{html}");

    // Six slides three-up stop at index 3, so four dots, not six.
    assert_eq!(html.matches(r#"aria-label="Go to slide"#).count(), 4);
    // At the first slide the previous control is disabled but keeps its place
    // in the tab order.
    assert!(html.contains(r#"aria-disabled="true""#), "{html}");
    assert!(!html.contains("disabled=true"), "{html}");

    // The live region reads the settled slide, politely, as one utterance.
    assert!(html.contains(r#"role="status""#), "{html}");
    assert!(html.contains(r#"aria-live="polite""#), "{html}");
    assert!(html.contains("Slide 1 of 6"), "{html}");
}

/// Centred three-up over six slides reaches indices 1-4, not 0-3, so an index
/// seeded at 0 sits outside the window. Left there it strands the keyboard:
/// four dots and no tab stop among them, nothing marked current, and no scroll
/// at rest to correct any of it.
#[test]
fn an_index_outside_the_reachable_window_is_pulled_into_it() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Photos",
                    indicators: true,
                    per_view: 3.0,
                    align: "center",
                    slides: (0..6).map(|i| rsx! { div { "slide {i}" } }).collect(),
                }
            }
        }
    }

    let html = body(&render(app));

    // The indicator strip is reachable at all.
    assert!(
        html.contains(r#"tabindex="0""#),
        "no tab stop in the strip: {html}"
    );
    assert!(html.contains(r#"aria-current="true""#), "{html}");
    // Slide 1 is the one actually centred at rest, so it is the current one.
    let slide_one = html.find("slide 1").expect("slide 1");
    let current = html
        .find(r#"data-current="true""#)
        .expect("a current slide");
    let slide_two = html.find("slide 2").expect("slide 2");
    assert!(
        current < slide_one && slide_one < slide_two,
        "the current slide should be slide 1: {html}"
    );
}

/// The mount-time clamp speaks only to a controlled caller, which is the only
/// party that can be holding an index the component disagrees with.
/// `onindexchange` documents itself as a scroll, control, key, indicator or
/// autoplay event, and a clamp is none of those.
#[test]
fn an_uncontrolled_carousel_reports_no_index_change_on_mount() {
    #[component]
    fn Counted(controlled: bool) -> Element {
        let mut calls = use_signal(|| 0usize);

        rsx! {
            div { "calls: {calls}" }
            Carousel {
                aria_label: "Photos",
                per_view: 3.0,
                align: "center",
                index: controlled.then_some(0),
                onindexchange: move |_| calls += 1,
                slides: (0..6).map(|i| rsx! { div { "slide {i}" } }).collect(),
            }
        }
    }

    fn uncontrolled() -> Element {
        rsx! { LiberoProvider { Counted { controlled: false } } }
    }
    fn controlled() -> Element {
        rsx! { LiberoProvider { Counted { controlled: true } } }
    }

    // Both clamp - the window is 1..=4 either way.
    assert!(body(&render(uncontrolled)).contains("calls: 0"));
    assert!(body(&render(controlled)).contains("calls: 1"));
}

/// A controlled index that leaves the window *after* mount is clamped like one
/// that starts outside it, and the caller has to hear about it the same way -
/// or it goes on holding 5 while the carousel shows 4.
#[test]
fn a_controlled_index_pushed_out_of_the_window_later_is_reported_back() {
    #[component]
    fn Driven() -> Element {
        // In the window (1..=4) at mount, so the mount-time clamp is not what
        // this sees.
        let mut index = use_signal(|| 3usize);
        let mut reported = use_signal(Vec::<usize>::new);
        use_effect(move || index.set(5));

        rsx! {
            div { "holding: {index}, reported: {reported:?}" }
            Carousel {
                aria_label: "Photos",
                per_view: 3.0,
                align: "center",
                index: index(),
                onindexchange: move |next| {
                    reported.write().push(next);
                    index.set(next);
                },
                slides: (0..6).map(|i| rsx! { div { "slide {i}" } }).collect(),
            }
        }
    }

    fn app() -> Element {
        rsx! { LiberoProvider { Driven {} } }
    }

    // Its own dom rather than `render`, which stops after one pass: this
    // needs the effect, the re-render it causes, the carousel's answer and the
    // re-render after that.
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..6 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    let html = body(&dioxus_ssr::render(&dom));

    assert!(html.contains("holding: 4, reported: [4]"), "{html}");
}

/// The roving `tabindex` moves focus by looking the new dot up by id, so the
/// ids have to be on the dots and have to match what the lookup builds. If they
/// were not, focus would silently never move and only the comment would say
/// otherwise.
#[test]
fn every_indicator_carries_the_id_its_focus_lookup_targets() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Photos",
                    indicators: true,
                    slides: (0..3).map(|i| rsx! { div { "slide {i}" } }).collect(),
                }
            }
        }
    }

    let html = body(&render(app));
    let track_id = html
        .split_once(r#" id=""#)
        .map(|(_, rest)| rest.split('"').next().unwrap_or_default().to_string())
        .expect("the track carries an id");

    for index in 0..3 {
        let id = format!("{track_id}-indicator-{index}");
        assert!(html.contains(&format!(r#"id="{id}""#)), "no {id} in {html}");
    }
}

/// The clones are what a looping strip scrolls into past either edge. They
/// carry the same content, so they are hidden rather than announced twice.
#[test]
fn a_looping_carousel_clones_its_ends_and_hides_the_copies() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Photos",
                    r#loop: true,
                    slides: (0..4).map(|i| rsx! { div { "slide {i}" } }).collect(),
                }
            }
        }
    }

    let html = body(&render(app));

    // One clone at each end at one-up: six positions for four slides.
    assert_eq!(html.matches("<div>slide").count(), 6);
    assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 2);
    // Only the four real slides are named and grouped.
    assert_eq!(html.matches(r#"aria-roledescription="slide""#).count(), 4);
    assert_eq!(html.matches(r#"aria-label="1 of 4""#).count(), 1);

    // The strip opens on the last slide (the leading clone) and the first
    // slide (the trailing one), in that order.
    let first = html.find("slide 3").expect("the leading clone");
    let second = html.find("slide 0").expect("the first real slide");
    assert!(first < second, "{html}");

    // No end to be at, so neither control is disabled and there is one dot per
    // real slide.
    assert!(!html.contains(r#"aria-disabled="true""#), "{html}");
    assert_eq!(html.matches(r#"aria-label="Go to slide"#).count(), 0);
}

/// A generic name beats none at all, so the theme's stands in - the warning is
/// what says to do better.
#[test]
fn an_unnamed_carousel_falls_back_to_the_theme_label() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel { slides: vec![rsx! { div { "one" } }] }
            }
        }
    }

    let root = attributes_of(&body(&render(app)), "section");

    assert_eq!(root["aria-label"], "Carousel");
}

#[test]
fn a_modal_renders_through_the_portal_outlet() {
    #[component]
    fn Opener() -> Element {
        let modal = use_modal(|s: ModalScope<String>| {
            rsx! {
                Dialog { title: "{s.args()}", "modal content" }
            }
        });
        use_hook(move || modal.open_with("Titled"));

        rsx! {
            div { id: "in-place" }
        }
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider { Opener {} }
        }
    }

    let html = render(app);
    let body = body(&html);
    let in_place = body.find("in-place").expect("the in-place wrapper");
    let content = body.find("modal content").expect("the modal content");

    assert!(
        content > in_place,
        "portalled content should render at the outlet, after where it was written"
    );
    assert!(body.contains("Titled"), "the args reach the render closure");
}

#[test]
fn a_superseded_opening_no_longer_closes_the_live_modal() {
    #[component]
    fn Opener() -> Element {
        let modal = use_modal(|s: ModalScope<String>| rsx! { Dialog { "{s.args()}" } });
        use_hook(move || {
            let first = modal.open_with("first");
            modal.open_with("second");
            first.close();
        });

        rsx! {}
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider { Opener {} }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("second"), "got {body}");
    assert!(!body.contains("first"), "got {body}");
}

#[test]
fn dismissing_a_modal_settles_its_opening_with_no_result() {
    #[component]
    fn Opener(outcome: Signal<::std::option::Option<::std::option::Option<bool>>>) -> Element {
        let modal = use_modal(|_: ModalScope<(), bool>| rsx! { Dialog { "asking" } });
        use_hook(move || {
            let mut outcome = outcome;
            modal
                .open()
                .on_result(move |result| outcome.set(Some(result)));
            modal.close();
        });

        rsx! {}
    }

    fn app() -> Element {
        let outcome = use_signal(|| ::std::option::Option::None);

        rsx! {
            LiberoProvider { Opener { outcome } }
            "{outcome:?}"
        }
    }

    let body = body(&render(app));

    assert!(body.contains("Some(None)"), "got {body}");
    assert!(!body.contains("asking"), "the modal should be gone: {body}");
}

#[test]
fn a_drawer_renders_through_the_portal_outlet() {
    #[component]
    fn Opener() -> Element {
        let options = DrawerOptions {
            anchor: "right".into(),
            ..Default::default()
        };
        let nav = use_drawer(options, |_: ModalScope<()>| rsx! { "drawer content" });
        use_hook(move || nav.open());

        rsx! {}
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider { Opener {} }
        }
    }

    let html = render(app);

    assert_eq!(html.matches("drawer content").count(), 1);
    assert!(
        html.contains(r#"data-state="size-md anchor-right""#),
        "got {html}"
    );
}

#[test]
fn a_sidebar_renders_in_place() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sidebar { "sidebar content" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("sidebar content"));
}

#[test]
fn a_sidebar_names_its_side_in_data_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sidebar { side: "right", "sidebar content" }
            }
        }
    }

    let html = render(app);

    let state = &attributes_of(&html, "div")["data-state"];
    assert!(state.contains("side-right"), "got {state}");
}

#[test]
fn overlay_renders_its_backdrop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Overlay {}
            }
        }
    }

    let html = render(app);
    let class = classes_of(&html, "div")
        .into_iter()
        .next()
        .expect("a class on the backdrop");

    assert!(has_rule_for(&html, &class));
}

#[test]
fn focus_trap_renders_its_children() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FocusTrap { "trapped" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("trapped"));
}

/// Two components with the same styling share one class and one emitted
/// rule - the registry is keyed by content, not by call site.
#[test]
fn identical_styling_is_registered_once() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Text { size: Size::Sm, "first" }
                Text { size: Size::Sm, "second" }
            }
        }
    }

    let html = render(app);
    let class = classes_of(&html, "p")
        .into_iter()
        .next()
        .expect("a class on the first Text");

    assert_eq!(body(&html).matches(&class).count(), 2);
    assert_eq!(html.matches(&format!(".{class}{{")).count(), 1);
}

#[test]
fn an_external_anchor_renders_a_plain_link() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", "Example" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "a");

    assert_eq!(attributes["href"], "https://example.com");
    assert!(body(&html).contains("Example"));
}

#[test]
fn code_renders_its_source() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Code { source: "let x = 1;" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<code"));
    assert!(body(&html).contains("let x = 1;"));
}

#[test]
fn code_block_renders_its_source_in_a_pre() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                CodeBlock { source: "let x = 1;" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<pre"));
    assert!(body(&html).contains("let x = 1;"));
}

#[test]
fn tree_renders_a_labelled_row_per_node() {
    fn app() -> Element {
        let data = vec![
            TreeNode::new("a", "Alpha".to_string()),
            TreeNode::new("b", "Beta".to_string()),
        ];

        rsx! {
            LiberoProvider {
                Tree { aria_label: "Files", data }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert_eq!(attributes_of(&html, "ul")["aria-label"], "Files");
    assert!(body.contains("Alpha"));
    assert!(body.contains("Beta"));
}

#[test]
fn nav_link_renders_a_link_with_its_active_background() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NavLink { to: "https://example.com", active: true, "Docs" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "a");

    assert_eq!(attributes["href"], "https://example.com");
    assert!(attributes["style"].contains("--lsx-nav-link-active-background"));
    assert!(attributes["data-state"].contains("active"));
}

/// Registering CSS used to re-render `LiberoProvider` itself - the component
/// owning `{children}` - which discarded the post-effect state of everything
/// under it. `NativeSelect` lost the `mounted` flag its `value` depends on that
/// way. The registered sheets now live in a `StyleOutlet` leaf instead, so a
/// registration dirties only that.
#[test]
fn effect_state_survives_a_stylesheet_registration() {
    #[component]
    fn Stateful() -> Element {
        let mut effect_ran = use_signal(|| false);
        use_effect(move || effect_ran.set(true));
        rsx! { Text { "effect_ran={effect_ran()}" } }
    }

    fn app() -> Element {
        rsx! { LiberoProvider { Stateful {} } }
    }

    assert!(body(&render(app)).contains("effect_ran=true"));
}

/// `StyleOutlet` renders after `{children}`, and Dioxus renders child scopes
/// eagerly in tree order, so the creating render already carries every sheet
/// its descendants registered - no second pass needed for CSS.
#[test]
fn the_first_render_already_carries_the_component_css() {
    fn app() -> Element {
        rsx! { LiberoProvider { Button { "Press" } } }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    let class = classes_of(&html, "button")
        .into_iter()
        .find(|class| class.starts_with("lsx-"))
        .expect("the button carries a generated class");
    assert!(
        html.contains(&format!(".{class}")),
        "the class's own CSS is missing from the first render"
    );
}

/// These four reach their own root element, which they used to do by
/// generating an id and querying the document for it - rendering a second
/// `id` beside a caller's, which browsers resolve to the first, breaking every
/// lookup. They go through a mounted handle now, so they emit no `id` at all
/// and a caller's is simply passed through.
#[test]
fn a_root_element_component_leaves_the_id_to_its_caller() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { id: "mine", "scrollable content" }
                Splitter {
                    id: "mine",
                    initial_size: 50.0,
                    panel_a: rsx! { div { "left" } },
                    panel_b: rsx! { div { "right" } },
                }
                Tree { id: "mine", aria_label: "Files", data: vec![TreeNode::new("a", "Alpha".to_string())] }
                FocusTrap { id: "mine", "trapped" }
            }
        }
    }

    let body = body(&render(app));

    assert_eq!(body.matches("id=\"mine\"").count(), 4);
    assert!(
        !body.contains("id=\"lsx-"),
        "a generated id shadowed the caller's:\n{body}"
    );
}

/// `min_size` floors *both* panes, so anything past 50 leaves `f64::clamp`
/// with `min > max` and panicked the whole subtree.
#[test]
fn splitter_survives_a_min_size_past_the_midpoint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Splitter {
                    initial_size: 50.0,
                    min_size: 70.0,
                    panel_a: rsx! { div { "left" } },
                    panel_b: rsx! { div { "right" } },
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("left") && body.contains("right"));
    assert!(body.contains("aria-valuemin=\"50\""));
    assert!(body.contains("--lsx-splitter-a:50%;"));
}

/// `<button>`'s HTML default is `submit`; ours is `button`, but only as a
/// fallback - a caller asking for a submit button has to get one.
#[test]
fn a_button_defaults_to_type_button_and_yields_to_the_caller() {
    fn plain() -> Element {
        rsx! { LiberoProvider { Button { "Save" } } }
    }
    fn submit() -> Element {
        rsx! { LiberoProvider { Button { r#type: "submit", "Save" } } }
    }

    assert_eq!(
        attributes_of(&render(plain), "button")
            .get("type")
            .map(String::as_str),
        Some("button")
    );
    assert_eq!(
        attributes_of(&render(submit), "button")
            .get("type")
            .map(String::as_str),
        Some("submit")
    );
}

#[test]
fn tooltip_wraps_its_trigger_and_labels_it() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tooltip {
                    label: rsx! { "Copy" },
                    label_id: "copy-tip",
                    open_delay: 300,
                    gap: Size::Sm,
                    Button { "C" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let wrapper = attributes_of(&body, "span");
    // The wrapper is the first `<span>`, the bubble the last.
    let bubble = attributes_of(&body[body.rfind("<span").expect("the bubble")..], "span");

    assert_eq!(bubble["role"], "tooltip");
    assert_eq!(bubble["id"], "copy-tip");
    assert_eq!(bubble["data-state"], "placement-top size-sm");
    assert!(wrapper["style"].contains("--lsx-tooltip-open-delay:300ms;"));
    assert!(wrapper["style"].contains("--lsx-tooltip-gap:var(--lsx-spacing-sm);"));
    // The trigger stays a real button inside the wrapper.
    assert!(body.contains(">C<"));

    // A stretched wrapper centres the bubble on the container, not the
    // trigger - which is what a `Flex` column parent does by default.
    let wrapper_class = classes_of(&html, "span")
        .into_iter()
        .find(|class| html.contains(&format!(".{class}:hover")))
        .expect("the wrapper class");
    assert!(html.contains(&format!(
        ".{wrapper_class}{{position:relative;display:inline-block;width:max-content;"
    )));
}

/// The whole component is these two rules: hover/focus opens the bubble, and
/// an explicit `opened` has to be able to beat them at equal specificity - so
/// it must come *later* in the sheet.
#[test]
fn tooltip_opens_on_hover_and_a_controlled_state_wins_by_source_order() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tooltip { label: rsx! { "t" }, opened: true, "x" }
            }
        }
    }

    let html = render(app);
    let wrapper = classes_of(&html, "span")
        .into_iter()
        .find(|class| html.contains(&format!(".{class}:hover")))
        .expect("the wrapper class");

    let hover = html
        .find(&format!(".{wrapper}:hover > [role=\"tooltip\"]"))
        .expect("a hover rule");
    let focus = html
        .find(&format!(
            ".{wrapper}:has(:focus-visible) > [role=\"tooltip\"]"
        ))
        .expect("a keyboard-focus rule");
    let opened = html
        .find(&format!(
            ".{wrapper}[data-state~=\"opened\"] > [role=\"tooltip\"]"
        ))
        .expect("an opened rule");

    assert!(
        opened > hover && opened > focus,
        "the override must sort last"
    );
    assert_eq!(attributes_of(&html, "span")["data-state"], "opened");
}

/// `attributes_of` reads the first matching tag, and a grid nests three deep.
fn nth_div(html: &str, skip: usize) -> String {
    let body = body(html);
    let mut rest = body.as_str();
    for _ in 0..skip {
        let start = rest.find("<div").expect("another nested div") + 4;
        rest = &rest[start..];
    }
    rest[rest.find("<div").expect("a div")..].to_string()
}

#[derive(Clone, Copy, PartialEq)]
enum TestArea {
    Header,
    Body,
}

impl GridArea for TestArea {
    fn name(&self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Body => "body",
        }
    }
}

fn test_template() -> GridTemplate {
    GridTemplate::new()
        .row(|row| row.cell(TestArea::Header))
        .row(|row| row.cells(TestArea::Body, 4))
        .build()
        .expect("a rectangular template")
}

#[test]
fn a_grid_publishes_its_areas_and_column_count_as_custom_properties() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Grid { template: test_template(), span { "zone" } }
            }
        }
    }

    let html = render(app);
    let style = attributes_of(&html, "div")
        .get("style")
        .expect("the areas variable")
        .clone();

    // The quotes `grid-template-areas` needs are HTML-escaped on the way into
    // the `style` attribute.
    assert!(style.contains("--lsx-grid-areas:&#34;header header header header&#34;"));
    assert!(style.contains("--lsx-grid-columns:4"));
}

#[test]
fn a_zone_carries_its_area_name_and_its_packing_states() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Grid { template: test_template(),
                    GridZone { area: TestArea::Body, dense: true, masonry: true, span { "item" } }
                }
            }
        }
    }

    let html = render(app);
    let zone = attributes_of(&nth_div(&html, 1), "div");

    assert!(
        zone.get("style")
            .expect("the zone area variable")
            .contains("--lsx-grid-zone-area:body")
    );
    let state = zone.get("data-state").expect("the zone states");
    assert!(state.contains("dense"));
    assert!(state.contains("masonry"));
}

#[test]
fn a_zone_works_without_a_grid_and_then_names_no_area() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { masonry: true, GridItem { span: GridSpan::Half, "card" } }
            }
        }
    }

    let html = render(app);
    let zone = attributes_of(&nth_div(&html, 0), "div");

    // Every zone publishes its resolved gap, but only one naming an area
    // declares the area var.
    let style = zone.get("style").expect("the resolved gap");
    assert!(!style.contains("--lsx-grid-zone-area"));
    assert!(style.contains("--lsx-grid-zone-gap"));
    assert!(body(&html).contains("card"));
}

#[test]
fn an_item_writes_the_grid_item_token_the_zone_selects_on() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { GridItem { span: GridSpan::TwoThirds, "card" } }
            }
        }
    }

    let html = render(app);
    let item = attributes_of(&nth_div(&html, 1), "div");
    let state = item.get("data-state").expect("the item states");

    assert!(state.contains("grid-item"));
    assert!(state.contains("span-two-thirds"));
    // Nothing measures without a browser, so no span is claimed.
    assert!(!state.contains("measured"));
}

#[test]
fn a_responsive_span_queries_its_zone_not_the_viewport() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Grid { template: test_template(),
                    GridZone { area: TestArea::Body,
                        GridItem { span: sp().base(GridSpan::Full).md(GridSpan::Half), "card" }
                    }
                }
            }
        }
    }

    let html = render(app);
    let zone = attributes_of(&nth_div(&html, 1), "div");

    // The zone names itself so an item has something to query.
    assert!(
        zone.get("style")
            .expect("the zone container variable")
            .contains("--lsx-grid-zone-container:lsx-zone-body")
    );

    // The base span still rides the recycled framework class; only the
    // breakpoint needs a rule of its own, and it is a container query.
    let item = attributes_of(&nth_div(&html, 2), "div");
    assert!(
        item.get("data-state")
            .expect("the item states")
            .contains("span-full")
    );
    assert!(html.contains("@container lsx-zone-body (min-width: 62rem){"));
    assert!(html.contains("grid-column:span 6;"));
    // A viewport query would be the bug this replaces.
    assert!(!html.contains("@media (min-width: 62rem){"));
}

/// Regression: `container-type: inline-size` zeroes an element's intrinsic
/// contribution, so on a standalone zone - a shrink-to-fit flex item, as in
/// the docs preview - it collapsed the zone to zero width and stacked every
/// item at x=0. A zone with no area has no container name to be queried by
/// either, so it must not be a container at all.
#[test]
fn only_a_zone_filling_an_area_is_a_query_container() {
    fn standalone() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { GridItem { span: GridSpan::Half, "loose" } }
            }
        }
    }

    fn placed() -> Element {
        rsx! {
            LiberoProvider {
                Grid { template: test_template(),
                    GridZone { area: TestArea::Body, GridItem { span: GridSpan::Half, "placed" } }
                }
            }
        }
    }

    let loose = render(standalone);
    assert!(
        !attributes_of(&nth_div(&loose, 0), "div")
            .get("data-state")
            .is_some_and(|state| state.contains("container"))
    );

    let html = render(placed);
    assert!(
        attributes_of(&nth_div(&html, 1), "div")
            .get("data-state")
            .expect("the zone states")
            .contains("container")
    );
    assert!(has_rule_for(&html, "lsx-",));
    assert!(html.contains("[data-state~=\"container\"]{container-type:inline-size;}"));
}

#[test]
fn a_plain_span_emits_no_container_query() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { GridItem { span: GridSpan::Half, "card" } }
            }
        }
    }

    assert!(!render(app).contains("@container"));
}

#[test]
fn an_orphan_item_still_renders_its_children() {
    fn app() -> Element {
        rsx! { LiberoProvider { GridItem { "loose" } } }
    }

    assert!(body(&render(app)).contains("loose"));
}

/// The zone places its children from *its* stylesheet while the item sets its
/// own `grid-column` from another. Both are on the framework layer, which the
/// registry emits in hash order, so only specificity settles the two - (0,3,0)
/// against (0,2,0). Loosening the zone selector to `& > *` would invert it.
#[test]
fn the_zone_places_items_at_a_higher_specificity_than_the_item_styles_itself() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { masonry: true, GridItem { span: GridSpan::Half, "card" } }
            }
        }
    }

    let html = render(app);

    assert!(html.contains(
        "[data-state~=\"masonry\"] > [data-state~=\"grid-item\"][data-state~=\"measured\"]"
    ));
    assert!(html.contains("[data-state~=\"span-half\"]"));
}

/// `gap` has to reach the item's `margin-bottom` and the zone's cancelling
/// margin, not just the `gap` shorthand - in masonry the row gap is zero and
/// the vertical spacing is entirely that margin. A per-size class could only
/// change the shorthand, so the resolved value is published as the var itself.
#[test]
fn a_zones_gap_reaches_the_vertical_spacing_masonry_actually_uses() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { masonry: true, gap: Size::Xl, GridItem { "card" } }
            }
        }
    }

    let html = render(app);
    let zone = attributes_of(&nth_div(&html, 0), "div");

    assert!(
        zone.get("style")
            .expect("the resolved gap")
            .contains("--lsx-grid-zone-gap:var(--lsx-spacing-xl)")
    );
    assert!(html.contains("margin-bottom:var(--lsx-grid-zone-gap)"));
    assert!(html.contains("margin-bottom:calc(-1 * var(--lsx-grid-zone-gap))"));
}

/// Twelve tracks carry eleven gaps whatever spans them, and a length gap does
/// not shrink - so a zone narrower than `11 * gap` would overflow its area.
/// The percentage cap resolves against the zone's own width and removes the
/// floor; `min-width: 0` is what lets the zone shrink at all, since a grid
/// item defaults to `min-width: auto`.
#[test]
fn a_zone_can_shrink_below_the_width_its_twelve_tracks_would_demand() {
    fn app() -> Element {
        rsx! { LiberoProvider { GridZone { GridItem { "card" } } } }
    }

    let html = render(app);

    assert!(html.contains("min-width:0"));
    assert!(html.contains("column-gap:min(var(--lsx-grid-zone-gap), 4%)"));
    assert!(html.contains("row-gap:var(--lsx-grid-zone-gap)"));
}

#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    #[option(label = "Admin area")]
    Admin,
    Billing,
}

#[test]
fn tabs_wire_every_tab_to_its_panel_and_render_only_the_selected_one() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    value: Section::Admin,
                    onchange: move |_| {},
                    disabled: vec![Section::Billing],
                    panel: |section: Section| match section {
                        Section::Account => rsx! { "account body" },
                        Section::Admin => rsx! { "admin body" },
                        Section::Billing => rsx! { "billing body" },
                    },
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    // One button per variant, in declaration order, labelled by the derive.
    assert_eq!(body.matches("role=\"tab\"").count(), 3);
    assert!(body.contains("Account"));
    assert!(body.contains("Admin area"));

    // Only the selected panel is rendered at all.
    assert!(body.contains("admin body"));
    assert!(!body.contains("account body"));
    assert!(!body.contains("billing body"));

    // Exactly one selected tab, and it owns the roving tabindex.
    assert_eq!(body.matches("aria-selected=\"true\"").count(), 1);
    assert_eq!(body.matches("tabindex=\"0\"").count(), 2); // the tab and its panel
    assert_eq!(body.matches("aria-disabled=\"true\"").count(), 1);

    // The panel points back at the tab that controls it.
    let panel = attributes_of(&body, "div role=\"tabpanel\"");
    let tab_id = panel["aria-labelledby"].clone();
    assert!(body.contains(&format!("id=\"{tab_id}\"")));
    assert!(body.contains(&format!("aria-controls=\"{}\"", panel["id"])));
}

#[test]
fn a_rich_tab_label_draws_its_content_and_still_names_the_tab() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    value: Section::Account,
                    onchange: move |_| {},
                    tabs: vec![Section::Account, Section::Billing],
                    label: |section: Section| OptionLabel::rich(
                        format!("t:{}", section.label()),
                        rsx! { span { "rich" } },
                    ),
                    panel: |_: Section| rsx! { "body" },
                }
            }
        }
    }

    let body = body(&render(app));

    // `tabs` narrows the strip; `label` names them; `render_label` fills them.
    assert_eq!(body.matches("role=\"tab\"").count(), 2);
    assert!(body.contains("aria-label=\"t:Account\""));
    assert!(body.contains("aria-label=\"t:Billing\""));
    assert_eq!(body.matches("rich").count(), 2);
    assert!(!body.contains(">Account<"));
}

#[derive(Clone, PartialEq, SliderValue)]
enum Grade {
    Low,
    High,
}

#[test]
fn a_sliders_format_prop_renames_its_mark_captions_too() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    value: Grade::Low,
                    oninput: move |_| {},
                    format: |grade: Grade| match grade {
                        Grade::Low => "Niedrig".to_string(),
                        Grade::High => "Hoch".to_string(),
                    },
                }
            }
        }
    }

    let body = body(&render(app));

    // The bubble, `aria-valuetext` and both captions speak one language.
    assert!(body.contains("Niedrig"));
    assert!(body.contains("Hoch"));
    assert!(!body.contains("Low"));
    assert!(!body.contains("High"));
}

/// The regression that made the arrow keys look dead: the rows were behind a
/// memoized subtree, so moving the highlight - or filtering the options -
/// changed state nothing redrew.
mod combobox_highlight {
    use super::*;
    use libero::components::use_combobox;
    use std::cell::RefCell;

    thread_local! {
        /// The rendered app's state, so the test can move the highlight.
        static STATE: RefCell<Option<ComboboxState>> = const { RefCell::new(None) };
        /// Its `options`, so the test can filter them the way typing does.
        static OPTIONS: RefCell<Option<Signal<Vec<&'static str>>>> = const { RefCell::new(None) };
    }

    #[component]
    fn App() -> Element {
        let fruit = use_combobox();
        let options = use_signal(|| vec!["apple", "banana", "grape"]);
        use_hook(|| fruit.open());
        STATE.with(|handle| *handle.borrow_mut() = Some(fruit));
        OPTIONS.with(|handle| *handle.borrow_mut() = Some(options));

        rsx! {
            LiberoProvider {
                Combobox {
                    state: fruit,
                    options: options(),
                    option: move |o: ComboboxOptionArgs<&'static str>| rsx! {
                        ComboboxOption { onpick: move |_| {}, "{o.value}" }
                    },
                    Button { attributes: fruit.a11y_attributes(), "pick" }
                }
            }
        }
    }

    fn state() -> ComboboxState {
        STATE
            .with(|handle| *handle.borrow())
            .expect("the app rendered")
    }

    /// One row, opening tag through closing tag, found by the `id`
    /// `ComboboxOption` takes from the `Combobox`.
    ///
    /// Matched together with `role="option"`, the attribute that follows it:
    /// the trigger's own `aria-activedescendant` holds the very same id, and
    /// it comes first in the document.
    fn row_of(html: &str, index: usize) -> String {
        let id = format!(r#"-option-{index}" role="option""#);
        let at = html
            .find(&id)
            .unwrap_or_else(|| panic!("no row {index} in:\n{html}"));
        let open = html[..at].rfind('<').expect("an unterminated tag");
        let close = at + html[at..].find("</div>").expect("an unclosed row");
        html[open..close].to_string()
    }

    fn render_pass(dom: &mut VirtualDom) -> String {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        body(&dioxus_ssr::render(dom))
    }

    #[test]
    fn moving_the_active_row_redraws_the_rows_it_touches() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();

        let html = render_pass(&mut dom);
        assert!(row_of(&html, 0).contains("active"));
        assert!(!row_of(&html, 2).contains("active"));

        dom.in_runtime(|| state().set_active(Some(2)));

        let html = render_pass(&mut dom);
        assert!(!row_of(&html, 0).contains("active"), "row 0 stayed active");
        assert!(row_of(&html, 2).contains("active"), "row 2 never lit up");
    }

    /// The same trap one level up: a stable row `Callback` compared equal, so
    /// filtering the options left the old ones on screen.
    #[test]
    fn filtering_the_options_redraws_the_rows() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        assert!(render_pass(&mut dom).contains("banana"));

        let options = OPTIONS
            .with(|handle| *handle.borrow())
            .expect("the app rendered");
        // What typing "ap" leaves: a shorter list whose second row is a
        // different option at the same index.
        dom.in_runtime(|| options.clone().set(vec!["apple", "grape"]));
        assert!(
            row_of(&render_pass(&mut dom), 1).contains("grape"),
            "row 1 stayed stale"
        );

        // The harder case: same length, different options, same highlight - so
        // every prop a memoizing subtree could compare is unchanged.
        dom.in_runtime(|| options.clone().set(vec!["apricot", "plum"]));

        let html = render_pass(&mut dom);
        assert!(row_of(&html, 0).contains("apricot"), "row 0 stayed stale");
        assert!(row_of(&html, 1).contains("plum"), "row 1 stayed stale");
    }

    /// The dropdown's chrome is `Paper`'s, and both halves have to agree: the
    /// class reads the themed surface, and the root carries the `bordered`
    /// token that class's border fold answers.
    #[test]
    fn the_dropdown_is_a_bordered_paper_surface() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);

        let markup = body(&html);
        let listbox = markup
            .find(r#"role="listbox""#)
            .expect("the dropdown rendered");
        let root = markup[..listbox]
            .rfind("position:fixed")
            .and_then(|style| markup[..style].rfind("<div"))
            .expect("the portaled dropdown root");
        let root = attributes_of(&markup[root..], "div");

        assert!(
            root["data-state"]
                .split(' ')
                .any(|token| token == "bordered"),
            "{root:?}"
        );
        let class = root["class"].split(' ').next().expect("a framework class");
        assert!(
            html.contains(&format!(
                ".{class}{{background:var(--lsx-paper-background);--lsx-focus-contrast:var(--lsx-paper-contrast);"
            )),
            "the dropdown's base no longer starts from `paper_sx()`"
        );
        assert!(
            html.contains(&format!(
                r#".{class}[data-state~="bordered"]{{border:1px solid var(--lsx-paper-border-color);}}"#
            )),
            "nothing answers the `bordered` token"
        );
    }

    /// The port to `use_popover`: the dropdown leaves the wrapper entirely, so
    /// no `overflow: hidden` ancestor can clip it.
    #[test]
    fn the_dropdown_is_portaled_out_of_the_wrapper() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        let html = render_pass(&mut dom);

        let trigger = html.find("<button").expect("the trigger rendered");
        let wrapper_closes = trigger + html[trigger..].find("</div>").expect("an open wrapper");
        let listbox = html
            .find(r#"role="listbox""#)
            .expect("the dropdown rendered");

        assert!(
            listbox > wrapper_closes,
            "the dropdown is still nested inside the trigger's wrapper"
        );
        assert!(
            row_of(&html, 0).contains("apple"),
            "the portaled rows lost their content"
        );
    }

    /// `ComboboxOption` reads its id and its highlight from a context, and a
    /// portaled subtree mounts under `PortalOutlet` rather than under
    /// `ComboboxCore` - so the context has to be handed across as a prop. The
    /// lookup is a `try_consume_context`, so getting this wrong fails silently.
    #[test]
    fn the_rows_keep_their_context_across_the_portal() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        dom.in_runtime(|| state().set_active(Some(2)));

        let html = render_pass(&mut dom);
        let row = row_of(&html, 2);

        assert!(row.contains("active"), "the row lost the highlight context");
        assert!(
            row.contains(&format!(r#"id="{}-option-2""#, state().id())),
            "the row lost the id context"
        );
    }

    /// Fixed to the viewport, not absolute to a wrapper that is no longer
    /// positioned - and hidden until the first measurement lands, which under
    /// SSR never does.
    #[test]
    fn the_dropdown_is_hidden_until_it_has_been_measured() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        let html = render_pass(&mut dom);

        assert!(
            html.contains(
                r#"style="position:fixed;left:0px;top:0px;width:auto;min-width:auto;visibility:hidden;""#
            ),
            "the dropdown is not laid out fixed and hidden before its first \
             measurement, which under SSR never lands:\n{html}"
        );
    }

    /// The whole reason the state is a handle: the trigger has to be able to
    /// name the row the arrows are on.
    #[test]
    fn the_trigger_points_at_the_active_row() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        dom.in_runtime(|| state().set_active(Some(1)));

        let html = render_pass(&mut dom);
        let button = attributes_of(&html, "button");

        assert_eq!(button["role"], "combobox");
        assert_eq!(button["aria-expanded"], "true");
        assert!(html.contains(&format!(r#"id="{}""#, button["aria-activedescendant"])));
        assert!(
            row_of(&html, 1).contains(&button["aria-activedescendant"]),
            "the trigger names a row other than the active one"
        );
    }
}

/// `Select` and `MultiSelect` as SSR sees them: closed, since the open state
/// lives in the component and no test can click. The open list is the
/// Combobox's, and `combobox_highlight` covers it.
mod select_listbox {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Debug, Options)]
    enum Fruit {
        Apple,
        Banana,
        Cherry,
    }

    #[test]
    fn the_trigger_is_a_combobox_named_by_the_label_and_shows_the_selection() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Select { label: "Fruit", value: Fruit::Banana, onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains(r#"role="combobox""#), "{html}");
        assert!(html.contains(r#"aria-haspopup="listbox""#), "{html}");
        assert!(html.contains(r#"aria-expanded="false""#), "{html}");
        assert!(html.contains(r#"tabindex="0""#), "{html}");
        assert!(
            html.contains("Banana"),
            "the selection is not drawn:\n{html}"
        );
        assert!(
            !html.contains("Apple"),
            "a closed list rendered its rows:\n{html}"
        );

        let label_id = html
            .split(r#"<label id=""#)
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("the label carries an id");
        assert!(
            html.contains(&format!(r#"aria-labelledby="{label_id}""#)),
            "the trigger is not named by its label:\n{html}"
        );
    }

    #[test]
    fn nothing_selected_shows_the_placeholder() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Select {
                        value: None::<Fruit>,
                        placeholder: "Pick a fruit",
                        onchange: move |_: Option<Fruit>| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains("data-placeholder"), "{html}");
        assert!(html.contains("Pick a fruit"), "{html}");
    }

    #[test]
    fn clearable_offers_the_x_only_while_something_is_selected() {
        fn picked() -> Element {
            rsx! {
                LiberoProvider {
                    Select { value: Fruit::Apple, clearable: true, onchange: move |_| {} }
                }
            }
        }
        fn empty() -> Element {
            rsx! {
                LiberoProvider {
                    Select {
                        value: None::<Fruit>,
                        clearable: true,
                        onchange: move |_: Option<Fruit>| {},
                    }
                }
            }
        }

        assert!(body(&render(picked)).contains(r#"aria-label="Clear""#));
        assert!(!body(&render(empty)).contains(r#"aria-label="Clear""#));
    }

    #[test]
    fn a_disabled_select_leaves_the_tab_order() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Select { value: Fruit::Apple, disabled: true, onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains(r#"aria-disabled="true""#), "{html}");
        assert!(!html.contains(r#"tabindex="0""#), "{html}");
    }

    #[test]
    fn a_multi_select_draws_each_value_in_the_order_it_was_picked() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect { value: vec![Fruit::Cherry, Fruit::Apple], onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        let cherry = html.find("Cherry").expect("Cherry is drawn");
        let apple = html.find("Apple").expect("Apple is drawn");
        assert!(cherry < apple, "the chips lost the pick order:\n{html}");
        assert!(
            !html.contains("Banana"),
            "an unpicked value is drawn:\n{html}"
        );
        assert!(
            html.contains("multiple"),
            "the trigger lost its `multiple` state:\n{html}"
        );
    }

    #[test]
    fn every_chip_carries_a_labelled_remove_button_and_an_id_to_point_at() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect { value: vec![Fruit::Cherry, Fruit::Apple], onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        assert!(
            html.contains(r#"aria-label="Remove Cherry""#)
                && html.contains(r#"aria-label="Remove Apple""#),
            "a chip lost its remove button:\n{html}"
        );
        assert!(
            html.matches(r#"tabindex="-1""#).count() == 2,
            "the remove buttons are tab stops, or are missing:\n{html}"
        );
        // The ids `aria-activedescendant` points at once the cursor moves.
        assert!(
            html.matches(r#"data-slot="chip""#).count() == 2,
            "a chip lost its wrapper:\n{html}"
        );
        assert!(
            !html.contains("aria-activedescendant"),
            "a closed select with no cursor still names a descendant:\n{html}"
        );
    }

    #[test]
    fn the_chips_follow_the_field_one_step_down_the_size_scale() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect { size: "lg", value: vec![Fruit::Cherry], onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        assert!(
            html.contains("size-md"),
            "an `lg` field did not draw an `md` chip:\n{html}"
        );
    }

    /// The chip's inner design is the caller's, remove control included - the
    /// component adds nothing of its own around what `selection` returns.
    #[test]
    fn a_custom_selection_draws_only_what_the_caller_drew() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect {
                        value: vec![Fruit::Cherry],
                        onchange: move |_| {},
                        selection: move |args: SelectionArgs<Fruit>| rsx! {
                            span { onclick: move |_| args.remove.call(()), "{args.value.label()}!" }
                        },
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains("Cherry!"), "{html}");
        assert!(
            !html.contains("aria-label=\"Remove Cherry\""),
            "the default x survived a custom selection:\n{html}"
        );
    }

    /// `searchable` must not leak a dropdown into a closed select. The core
    /// keeps an *open* list alive through a query that matches nothing - that
    /// is what the header slot is for - and a closed one draws neither.
    #[test]
    fn a_closed_searchable_select_draws_no_search_box() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Select {
                        value: Fruit::Apple,
                        searchable: true,
                        search_placeholder: "Find a fruit",
                        onchange: move |_| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(
            !html.contains("Find a fruit"),
            "a closed select drew its search box:\n{html}"
        );
        assert!(!html.contains(r#"role="listbox""#), "{html}");
        // Closed, the trigger is still the combobox - the role only moves to
        // the search box while one exists.
        assert!(html.contains(r#"role="combobox""#), "{html}");
    }
}

/// `Autocomplete` as SSR sees it: closed. The open state lives in the
/// component - there is no `state` prop to force it open with, the same limit
/// `select_listbox` documents - so the suggestions themselves are out of
/// reach here. What is testable is the control, which is where this component
/// differs from `Select`: an `<input>` that keeps its own text.
mod autocomplete_suggestions {
    use super::*;

    const CITIES: [&str; 3] = ["Amsterdam", "Berlin", "Copenhagen"];

    fn cities() -> Vec<String> {
        CITIES.iter().map(|city| city.to_string()).collect()
    }

    #[test]
    fn the_control_is_an_input_that_announces_a_list_of_suggestions() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Autocomplete {
                        label: "City",
                        value: "Ber",
                        options: cities(),
                        oninput: move |_| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains("<input"), "{html}");
        assert!(html.contains(r#"role="combobox""#), "{html}");
        assert!(html.contains(r#"aria-autocomplete="list""#), "{html}");
        assert!(html.contains(r#"aria-expanded="false""#), "{html}");
        // The browser's own dropdown would sit on top of ours.
        assert!(html.contains(r#"autocomplete="off""#), "{html}");
        assert!(
            html.contains(r#"value="Ber""#),
            "the text is not drawn:\n{html}"
        );
    }

    /// The whole point of the component: the value is the text, so a closed
    /// field draws no rows at all - not even the ones that match.
    #[test]
    fn a_closed_field_draws_none_of_its_suggestions() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Autocomplete {
                        value: "Ber",
                        placeholder: "Where to?",
                        options: cities(),
                        oninput: move |_| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains(r#"placeholder="Where to?""#), "{html}");
        for city in CITIES {
            assert!(
                !html.contains(&format!(">{city}<")),
                "a closed list rendered {city}:\n{html}"
            );
        }
    }

    #[test]
    fn clearable_offers_the_x_only_while_the_field_holds_text() {
        fn typed() -> Element {
            rsx! {
                LiberoProvider {
                    Autocomplete {
                        value: "Ber",
                        clearable: true,
                        options: cities(),
                        oninput: move |_| {},
                    }
                }
            }
        }
        fn blank() -> Element {
            rsx! {
                LiberoProvider {
                    Autocomplete {
                        value: "",
                        clearable: true,
                        options: cities(),
                        oninput: move |_| {},
                    }
                }
            }
        }

        assert!(body(&render(typed)).contains(r#"aria-label="Clear""#));
        assert!(!body(&render(blank)).contains(r#"aria-label="Clear""#));
    }

    /// The caller's own trailing content keeps its place when the x joins it.
    #[test]
    fn a_disabled_field_offers_no_clear_button() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Autocomplete {
                        value: "Ber",
                        clearable: true,
                        disabled: true,
                        options: cities(),
                        oninput: move |_| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains("disabled"), "{html}");
        assert!(!html.contains(r#"aria-label="Clear""#), "{html}");
    }
}
