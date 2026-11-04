//! Responsive `size` and `radius` on the form family (todo 5652): every element that carries the
//! field's states, and whose sheet sizes it, gets a range copy of that sheet's block.

use std::collections::BTreeSet;

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Checkbox, NativeSelect, NumberField, Options, PasswordField, PinField, Radio, Rating,
        SegmentedControl, Slider, Switch, TagsField, TextField, Textarea, Variant,
    },
    sx::ThemeAwareValue,
    theme::{ChoiceVariant, Size, responsive},
};

/// The classes of every tag whose `data-state` holds `word`.
fn classes_carrying(html: &str, word: &str) -> BTreeSet<String> {
    let mut classes = BTreeSet::new();
    for tag in body(html).split('<').skip(1) {
        let tag = &tag[..tag.find('>').unwrap_or(tag.len())];
        let value = |name: &str| {
            let at = tag.find(&format!(" {name}=\""))? + name.len() + 3;
            Some(&tag[at..at + tag[at..].find('"')?])
        };
        if value("data-state").is_some_and(|state| state.split(' ').any(|w| w == word)) {
            classes.extend(
                value("class")
                    .into_iter()
                    .flat_map(|class| class.split(' ').map(str::to_string)),
            );
        }
    }
    classes
}

/// The classes a rule keys on `word` directly: `.c[data-state~="word"]`.
fn classes_keyed(html: &str, word: &str) -> BTreeSet<String> {
    let needle = format!(r#"[data-state~="{word}"]"#);
    html.match_indices(&needle)
        .filter_map(|(at, _)| {
            let head = &html[..at];
            let start = head.rfind('.')?;
            let class = &head[start + 1..];
            class
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                .then(|| class.to_string())
        })
        .collect()
}

/// Every sheet that sizes a plain `lg` field has its `md-:..-lg` copy on the responsive page.
fn assert_follows(plain: fn() -> Element, responsive: fn() -> Element, name: &str) {
    let (plain, responsive) = (render(plain), render(responsive));
    for (word, run) in [("size-lg", "md-:size-lg"), ("radius-xl", "md-:radius-xl")] {
        let sized: BTreeSet<_> = classes_carrying(&plain, word)
            .intersection(&classes_keyed(&plain, word))
            .cloned()
            .collect();
        assert!(
            !sized.is_empty() || word.starts_with("radius"),
            "{name}: nothing sized by {word}"
        );
        let copied = classes_keyed(&responsive, run);
        assert!(
            !classes_carrying(&responsive, run).is_empty() || word.starts_with("radius"),
            "{name}: no {run} word"
        );
        let missing: Vec<_> = sized.difference(&copied).collect();
        assert!(missing.is_empty(), "{name}: no {run} copy for {missing:?}");
    }
}

macro_rules! follows {
    ($test:ident, $name:literal, $component:ident { $($rest:tt)* }) => {
        #[test]
        fn $test() {
            fn plain() -> Element {
                rsx! { LiberoProvider { $component { size: "lg", radius: "xl", $($rest)* } } }
            }
            fn responsive_one() -> Element {
                rsx! {
                    LiberoProvider {
                        $component {
                            size: responsive(Size::Sm).md(Size::Lg),
                            radius: responsive(ThemeAwareValue::Size(Size::Xs)).md(ThemeAwareValue::Size(Size::Xl)),
                            $($rest)*
                        }
                    }
                }
            }
            assert_follows(plain, responsive_one, $name);
        }
    };
}

follows!(
    a_text_field_follows_the_viewport,
    "TextField",
    TextField {
        label: "Name",
        leading: rsx! { "@" }
    }
);
follows!(
    a_textarea_follows_the_viewport,
    "Textarea",
    Textarea { label: "Note" }
);
follows!(
    a_checkbox_follows_the_viewport,
    "Checkbox",
    Checkbox { label: "Agree" }
);
follows!(
    a_switch_follows_the_viewport,
    "Switch",
    Switch { label: "On" }
);
follows!(
    a_pin_field_follows_the_viewport,
    "PinField",
    PinField { label: "Code" }
);
follows!(
    a_tags_field_follows_the_viewport,
    "TagsField",
    TagsField { label: "Tags" }
);
follows!(
    a_native_select_follows_the_viewport,
    "NativeSelect",
    NativeSelect {
        label: "Pick",
        options: vec!["A".to_string(), "B".to_string()]
    }
);

#[test]
fn a_radio_and_a_slider_follow_the_viewport() {
    fn plain() -> Element {
        rsx! { LiberoProvider { Radio { label: "A", size: "lg" } Slider { label: "Volume", value: 5.0, size: "lg" } Rating { label: "Stars", size: "lg" } } }
    }
    fn responsive_one() -> Element {
        rsx! {
            LiberoProvider {
                Radio { label: "A", size: responsive(Size::Sm).md(Size::Lg) }
                Slider { label: "Volume", value: 5.0, size: responsive(Size::Sm).md(Size::Lg) }
                Rating { label: "Stars", size: responsive(Size::Sm).md(Size::Lg) }
            }
        }
    }
    assert_follows(plain, responsive_one, "Radio, Slider");
}

#[test]
fn a_plain_field_carries_no_range_copy() {
    fn app() -> Element {
        rsx! { LiberoProvider { TextField { label: "Name", size: "lg", radius: "0" } Checkbox { label: "A", size: "sm" } } }
    }
    let html = render(app);
    assert!(
        !html.contains(":size-") && !html.contains(":radius-"),
        "{html}"
    );
}

/// A custom radius per breakpoint rides the override var in the user layer.
#[test]
fn a_custom_radius_per_breakpoint_sets_the_override_var() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { label: "Name", radius: responsive(ThemeAwareValue::from("0")).md(ThemeAwareValue::Size(Size::Xl)) }
            }
        }
    }
    let html = render(app);
    assert!(html.contains("--lsx-radius-override:0"), "{html}");
    assert!(html.contains("md-:radius-xl"), "{html}");
}

/// A card from `md` up: the card look and its ring are copied for the run; no plain `card` word.
#[test]
fn a_choice_turns_into_a_card_from_a_breakpoint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Checkbox { label: "A", variant: responsive(ChoiceVariant::Plain).md(ChoiceVariant::Card) }
                Switch { label: "B", variant: responsive(ChoiceVariant::Plain).md(ChoiceVariant::Card) }
            }
        }
    }
    let html = render(app);
    let page = body(&html);
    assert!(
        page.contains("md-:card") && !page.contains(" card "),
        "{page}"
    );
    assert!(
        html.contains(r#"[data-state~="md-:card"] > [data-ring]"#),
        "{html}"
    );
    assert!(
        html.contains(r#"[data-state~="md-:card"][data-state~="checked"]"#),
        "{html}"
    );
}

#[derive(Clone, PartialEq, Options)]
enum View {
    List,
    Grid,
}

/// The segments' size and radius, the variant and the gap per breakpoint: the strip carries the
/// run words, and its copies reach the segments.
#[test]
fn a_segmented_control_follows_the_viewport() {
    fn plain() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl { label: "View", value: View::List, onchange: move |_| {}, size: "lg", radius: "xl", variant: Variant::Outlined, gap: "xs" }
            }
        }
    }
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl {
                    label: "View",
                    value: View::List,
                    onchange: move |_| {},
                    size: responsive(Size::Sm).md(Size::Lg),
                    radius: responsive(ThemeAwareValue::Size(Size::Xs)).md(ThemeAwareValue::Size(Size::Xl)),
                    variant: responsive(Variant::Filled).md(Variant::Outlined),
                    gap: responsive(ThemeAwareValue::Size(Size::Xs)).md("0.5rem".into()),
                }
            }
        }
    }
    let plain = render(plain);
    assert!(
        !plain.contains(":size-") && !plain.contains("-v-"),
        "{plain}"
    );
    let html = render(app);
    let page = body(&html);
    for word in [
        "md-:size-lg",
        "md-:radius-xl",
        "base-md:filled",
        "md-:outlined",
    ] {
        assert!(page.contains(word), "no {word} in {page}");
    }
    // The segments carry no word of a responsive axis.
    assert!(
        !page.contains("\"size-lg") && !page.contains(" size-sm"),
        "{page}"
    );
    for copy in [
        r#"[data-state~="md-:size-lg"] > label"#,
        r#"[data-state~="base-md:radius-xs"] > label"#,
        r#"[data-state~="md-:outlined"] > label"#,
    ] {
        assert!(html.contains(copy), "no copy {copy}");
    }
    // Each variant's colours under its own names; the copy maps them back.
    assert!(
        page.contains("-v-tonal") && page.contains("-v-outlined"),
        "{page}"
    );
    assert!(html.contains("--lsx-spacing-override:0.5rem"), "{html}");
}

/// The slot buttons step with the field: the reveal and stepper buttons carry run words too.
#[test]
fn slot_buttons_follow_a_responsive_field() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                PasswordField { label: "Secret", size: responsive(Size::Sm).md(Size::Xl) }
                NumberField::<i64> { label: "Count", steppers: true, size: responsive(Size::Sm).md(Size::Xl) }
            }
        }
    }
    let html = render(app);
    let page = body(&html);
    let buttons: Vec<&str> = page
        .split('<')
        .filter(|tag| tag.starts_with("button"))
        .map(|tag| &tag[..tag.find('>').unwrap_or(tag.len())])
        .collect();
    assert_eq!(buttons.len(), 3, "{buttons:?}");
    for button in buttons {
        assert!(
            button.contains("base-md:") && button.contains("md-:"),
            "{button}"
        );
    }
}
