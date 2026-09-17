use crate::components::{DocPage, DocSection};
use crate::pages::form::date_locales::{Choice, FORMATS, LANGUAGES, options, picked};
use dioxus::prelude::*;
use libero::{
    chrono::{NaiveDate, NaiveDateTime, NaiveTime},
    components::{Code, CodeBlock, DateField, DayPicker, Flex, SegmentedControl, Text},
    sx::sx,
    use_formats_handle, use_localization_handle,
};

// snippet: item #[derive(Clone, PartialEq, Routable)] enum Route { #[route("/")] Home {} }
// snippet: item #[component] fn Home() -> Element { rsx! {} }
const LOCALIZATION: &str = r#"fn App() -> Element {
    rsx! {
        LiberoProvider {
            localization: &Localization::GERMAN,
            formats: &Formats::GERMAN,
            Router::<Route> {}
        }
    }
}"#;

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

#[component]
pub fn LocalizationPage() -> Element {
    rsx! {
        DocPage {
            title: "Localization",
            markdown: "/md/localization.md",
            lead: rsx! {
                Text {
                    "Every string a component says on its own, such as an accessible name, an "
                    "announcement or a month name, comes from "
                    Code { source: "LiberoProvider" }
                    "'s "
                    Code { source: "localization" }
                    ". Two languages ship: "
                    Code { source: "Localization::ENGLISH" }
                    ", the default, and "
                    Code { source: "Localization::GERMAN" }
                    "."
                }
                Text {
                    "How a date or a number is written depends on the region, not the language, "
                    "so it is a prop of its own, "
                    Code { source: "formats" }
                    ". Any language goes with any formats: this site is English in German formats."
                }
            },
            DocSection {
                title: "Language and formats",
                Text {
                    Code { source: "Formats::AMERICAN" }
                    " is the default: Sunday first, a 12-hour clock, "
                    Code { source: "September 14, 2026" }
                    ". "
                    Code { source: "Formats::GERMAN" }
                    " is Monday first, a 24-hour clock, "
                    Code { source: "14. September 2026" }
                    ". The switches below change this site's."
                }
                LocalizationPreview {}
                CodeBlock { source: LOCALIZATION, language: "rust" }
            }

            DocSection {
                title: "Formats",
                Text {
                    Code { source: "Formats" }
                    " holds the first weekday, a date pattern per level (day, month, year), the "
                    "month heading, the time pattern, and the range and decimal separators. The "
                    "patterns use dayjs tokens such as "
                    Code { source: "YYYY" }
                    ", "
                    Code { source: "MMMM" }
                    ", "
                    Code { source: "D" }
                    " and "
                    Code { source: "HH" }
                    ". An "
                    Code { source: "h" }
                    " or an "
                    Code { source: "A" }
                    " in the time pattern makes the pickers 12-hour."
                }
                CodeBlock { source: CUSTOM_FORMATS, language: "rust" }
            }

            DocSection {
                title: "Changing the words",
                Text {
                    Code { source: "Localization" }
                    " is shaped like a theme: one struct with a group per component, plus "
                    Code { source: "common" }
                    " for the words many components share. Change it with struct update "
                    "syntax in a "
                    Code { source: "static" }
                    "."
                }
                CodeBlock { source: OVERRIDE_LOCALIZATION, language: "rust" }
                Text {
                    "A string with a value in it is a template with named holes, such as "
                    Code { source: "\"Go to page {{n}}\"" }
                    ", so a language can put the value where its grammar wants it. "
                    Code { source: "fill(template, &[(\"n\", &3)])" }
                    " fills the holes. A prop that names what only the call site knows, such "
                    "as a dialog's close label, still wins over the localization."
                }
            }

            DocSection {
                title: "Switching at runtime",
                Text {
                    Code { source: "use_localization()" }
                    " and "
                    Code { source: "use_formats()" }
                    " read the active ones, and every component that reads them re-renders on "
                    "a switch. "
                    Code { source: "use_localization_handle()" }
                    " and "
                    Code { source: "use_formats_handle()" }
                    " add "
                    Code { source: "get()" }
                    " and "
                    Code { source: "set()" }
                    ", which a language or region picker is built on. The provider reads its "
                    "props once, at mount, so switch through the handles."
                }
                CodeBlock { source: SWITCH_LOCALIZATION, language: "rust" }
                Text {
                    "The handles take a "
                    Code { source: "&'static" }
                    " reference, so a catalogue loaded at runtime is leaked once per language "
                    "with "
                    Code { source: "Box::leak" }
                    "."
                }
            }
        }
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

/// A language and a formats switch over a calendar and a date-time field.
/// They set the site's own and put them back on leaving the page.
#[component]
fn LocalizationPreview() -> Element {
    let localization = use_localization_handle();
    let formats = use_formats_handle();
    let site = use_hook(|| (localization.get(), formats.get()));
    use_drop(move || {
        localization.set(site.0);
        formats.set(site.1);
    });
    let mut language = use_signal(|| option_of(&LANGUAGES, site.0));
    let mut conventions = use_signal(|| option_of(&FORMATS, site.1));
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 14));
    let mut moment = use_signal(|| {
        NaiveDate::from_ymd_opt(2026, 9, 14)
            .zip(NaiveTime::from_hms_opt(15, 30, 0))
            .map(|(day, time)| NaiveDateTime::new(day, time))
    });
    rsx! {
        Flex { direction: "column", gap: "md", align: "start",
            Flex { gap: "md", wrap: "wrap",
                SegmentedControl {
                    "aria-label": "Language",
                    value: language(),
                    options: options(&LANGUAGES).map(str::to_string).to_vec(),
                    onchange: move |next: String| {
                        localization.set(picked(&LANGUAGES, &next).1);
                        language.set(next);
                    },
                }
                SegmentedControl {
                    "aria-label": "Formats",
                    value: conventions(),
                    options: options(&FORMATS).map(str::to_string).to_vec(),
                    onchange: move |next: String| {
                        formats.set(picked(&FORMATS, &next).1);
                        conventions.set(next);
                    },
                }
            }
            Flex { gap: "md", wrap: "wrap", align: "start",
                DayPicker { value: day(), onchange: move |next| day.set(next) }
                DateField::<NaiveDateTime> {
                    value: moment(),
                    onchange: move |next| moment.set(next),
                    label: "When",
                    sx: sx().width("260px"),
                }
            }
        }
    }
}
