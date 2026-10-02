use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent};
use crate::pages::form::date_locales::{Choice, FORMATS, LANGUAGES, options, picked};
use dioxus::prelude::*;
use libero::{
    chrono::{NaiveDate, NaiveDateTime, NaiveTime},
    components::{ChronoField, Code, CodeBlock, DatePicker, Flex, Table, Text, column},
    hooks::use_direction,
    sx::sx,
    theme::Direction,
    use_formats_handle, use_localization_handle,
};

/// What ships: the const, its first weekday, a day and a time.
const SHIPPED: [(&str, &str, &str, &str); 2] = [
    (
        "Formats::AMERICAN (default)",
        "Sunday",
        "September 14, 2026",
        "3:30 PM",
    ),
    ("Formats::GERMAN", "Monday", "14. September 2026", "15:30"),
];

const CUSTOM_FORMATS: &str = r#"static SWISS: Formats = Formats {
    decimal_separator: ".",
    ..Formats::GERMAN
};"#;

const OVERRIDE_LOCALIZATION: &str = r#"static WORDS: Localization = Localization {
    common: CommonLabels {
        close: "Zumachen",
        ..CommonLabels::GERMAN
    },
    pagination: PaginationLabels {
        page: "Gehe zu Seite {n}",
        ..PaginationLabels::GERMAN
    },
    ..Localization::GERMAN
};"#;

const SWITCH_LOCALIZATION: &str = r#"let localization = use_localization_handle();
let formats = use_formats_handle();

rsx! {
    Button {
        onclick: move |_| {
            localization.set(&Localization::GERMAN);
            formats.set(&Formats::GERMAN);
        },
        "Deutsch"
    }
}"#;

/// The prop for a picked const, unless it is libero's default (the first).
fn prop_of<T: 'static>(
    choices: &[Choice<T>; 2],
    values: &DemoValues,
    name: &str,
    ty: &str,
) -> Vec<String> {
    let (picked, _) = picked(choices, &values.str(name));
    match picked == choices[0].1 {
        true => vec![],
        false => vec![format!("{name}: &{ty}::{picked}")],
    }
}

/// Direction is an attribute round the provider, so its portals flip too.
fn wrap_direction(values: &DemoValues, code: &str) -> String {
    match values.str("direction").as_str() {
        "rtl" => format!("div {{ dir: \"rtl\",\n{}}}", indent(code)),
        _ => code.to_string(),
    }
}

/// The option naming `current`, the first if none does.
fn option_of<T: PartialEq + 'static>(choices: &[Choice<T>; 2], current: &T) -> String {
    let (label, ..) = choices
        .iter()
        .find(|(_, _, value)| *value == current)
        .unwrap_or(&choices[0]);
    label.to_string()
}

#[component]
pub fn LocalizationPage() -> Element {
    let localization = use_localization_handle();
    let formats = use_formats_handle();
    let direction = use_direction();
    // The controls start at what the site shows.
    let site = use_hook(|| {
        (
            option_of(&LANGUAGES, localization.get()),
            option_of(&FORMATS, formats.get()),
            direction.is_rtl(),
        )
    });

    rsx! {
        DocPage {
            title: "Localization",
            markdown: "/md/localization.md",
            lead: rsx! {
                Text {
                    "Every word a component says on its own, such as an accessible name or a "
                    "month, comes from "
                    Code { source: "localization" }
                    ". How dates and numbers are written depends on the region, so it is a "
                    "prop of its own, "
                    Code { source: "formats" }
                    ". Any language goes with any formats: this site is English in German formats."
                }
                Text {
                    "Direction comes from the document. Components follow a "
                    Code { source: "dir=\"rtl\"" }
                    " around them, and their directional props say start and end, not left and right."
                }
            },
            // snippet: item #[derive(Clone, PartialEq, Routable)] enum Route { #[route("/")] Home {} }
            // snippet: item #[component] fn Home() -> Element { rsx! {} }
            Demo {
                component: "LiberoProvider",
                children_text: "",
                children_code: "Router::<Route> {}".to_string(),
                wrap: Wrap(wrap_direction),
                controls: vec![
                    // Each swaps the site's own while the page is open.
                    Control::toggle("localization", options(&LANGUAGES))
                        .default(site.0.clone())
                        .code(|_, values| prop_of(&LANGUAGES, values, "localization", "Localization")),
                    Control::toggle("formats", options(&FORMATS))
                        .default(site.1.clone())
                        .code(|_, values| prop_of(&FORMATS, values, "formats", "Formats")),
                    Control::toggle("direction", ["ltr", "rtl"])
                        .labels(["Left to right", "Right to left"])
                        .default(if site.2 { "rtl" } else { "ltr" })
                        .code(|_, _| vec![]),
                ],
                render: move |values: DemoValues| rsx! {
                    LocalizationPreview { values }
                },
            }

            DocSection {
                title: "What ships",
                Text {
                    Code { source: "Localization::ENGLISH" }
                    " is the default language, and "
                    Code { source: "Localization::GERMAN" }
                    " the other. Two formats ship too:"
                }
                Table {
                    aria_label: "Shipped formats",
                    data: SHIPPED.to_vec(),
                    columns: vec![
                        column("Formats")
                            .value(|row: &(&str, &str, &str, &str)| row.0)
                            .render(|row: &(&str, &str, &str, &str)| rsx! { Code { source: row.0, sx: sx().white_space("nowrap") } }),
                        column("First weekday").value(|row: &(&str, &str, &str, &str)| row.1),
                        column("Day").value(|row: &(&str, &str, &str, &str)| row.2),
                        column("Time").value(|row: &(&str, &str, &str, &str)| row.3),
                    ],
                }
            }

            DocSection {
                title: "Your own words and formats",
                Text {
                    "Both are plain structs. Change what you need with struct update syntax in a "
                    Code { source: "static" }
                    ". The format patterns use dayjs tokens such as "
                    Code { source: "D. MMMM YYYY" }
                    "."
                }
                CodeBlock { source: CUSTOM_FORMATS, language: "rust" }
                Text {
                    Code { source: "Localization" }
                    " has a group per component, plus "
                    Code { source: "common" }
                    " for the words many share. A value inside a string is a named hole such as "
                    Code { source: "{{n}}" }
                    ", so each language puts it where its grammar wants it."
                }
                CodeBlock { source: OVERRIDE_LOCALIZATION, language: "rust" }
            }

            DocSection {
                title: "Switching at runtime",
                Text {
                    "The provider reads its props once. Switch later through "
                    Code { source: "use_localization_handle()" }
                    " and "
                    Code { source: "use_formats_handle()" }
                    ", and every component that reads them follows."
                }
                CodeBlock { source: SWITCH_LOCALIZATION, language: "rust" }
            }
        }
    }
}

/// A calendar and a date-time field in the site's language, formats and
/// direction, which it sets from the controls and puts back on leaving.
#[component]
fn LocalizationPreview(values: DemoValues) -> Element {
    let (language_name, language) = picked(&LANGUAGES, &values.str("localization"));
    // The page stays English: only the preview speaks the picked language (WCAG 3.1.2).
    let lang = if language_name == LANGUAGES[1].1 {
        "de"
    } else {
        "en"
    };
    let (_, conventions) = picked(&FORMATS, &values.str("formats"));
    let rtl = values.str("direction") == "rtl";
    let localization = use_localization_handle();
    let formats = use_formats_handle();
    let direction = use_direction();
    let site = use_hook(|| (localization.get(), formats.get(), direction.kept()));
    let turn_to = if rtl { Direction::Rtl } else { Direction::Ltr };
    {
        let direction = direction.clone();
        use_effect(use_reactive!(|language, conventions, turn_to| {
            localization.set(language);
            formats.set(conventions);
            if direction.get() != turn_to {
                direction.set(turn_to);
            }
        }));
    }
    use_drop(move || {
        localization.set(site.0);
        formats.set(site.1);
        // The choice found, or none: a visit alone should keep nothing.
        match site.2 {
            _ if direction.kept() == site.2 => {}
            Some(found) => direction.set(found),
            None => direction.clear(),
        }
    });

    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 14));
    let mut moment = use_signal(|| {
        NaiveDate::from_ymd_opt(2026, 9, 14)
            .zip(NaiveTime::from_hms_opt(15, 30, 0))
            .map(|(day, time)| NaiveDateTime::new(day, time))
    });
    rsx! {
        Flex { lang, gap: "md", wrap: "wrap", align: "start", justify: "center",
            DatePicker { value: day(), onchange: move |next| day.set(next) }
            ChronoField::<NaiveDateTime> {
                value: moment(),
                onchange: move |next| moment.set(next),
                label: if lang == "de" { "Wann" } else { "When" },
                sx: sx().width("260px"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libero::context::LiberoProvider;

    #[component]
    fn Preview(language: &'static str) -> Element {
        let values = DemoValues::defaults(&[
            Control::toggle("localization", options(&LANGUAGES)).default(language),
            Control::toggle("formats", options(&FORMATS)),
            Control::toggle("direction", ["ltr", "rtl"]),
        ]);
        rsx! {
            LiberoProvider { LocalizationPreview { values } }
        }
    }

    /// WCAG 3.1.2: German words in the preview sit under `lang="de"` (todo 1994).
    #[test]
    fn the_preview_speaks_its_language() {
        for (language, lang) in [("Deutsch", "de"), ("English", "en")] {
            let mut dom = VirtualDom::new_with_props(Preview, PreviewProps { language });
            dom.rebuild_in_place();
            let html = dioxus_ssr::render(&dom);
            assert!(
                html.contains(&format!("lang=\"{lang}\"")),
                "{language}: {html}"
            );
        }
    }
}
