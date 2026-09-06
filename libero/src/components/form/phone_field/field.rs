use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        ComboboxCore, ComboboxOption, HtmlTag, Input,
        common::{field_props, ring_overlay},
        form::{FIELD_CONTROL_SX, glyphs::ChevronIcon, use_bound, use_field, use_field_frame},
        layout::use_box,
        use_combobox,
    },
    hooks::{PopoverWidth, use_element, use_theme},
    platform::ElementApi,
    sx::{StaticSx, sx},
};

use super::countries::{self, COUNTRIES, Country};

/// The picker in the frame's leading slot. It is a real `<button>` and a real
/// tab stop, because it is the only way to reach what it does
/// ([[codebase/components/field]]).
static PICKER_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .gap("4px")
        .border("none")
        .background("transparent")
        // The slot paints itself grey; the country is the field's own text.
        .color("inherit")
        .font_family("inherit")
        .font_size("inherit")
        .line_height("1.5")
        .padding("0")
        .margin("0")
        .cursor("pointer")
        .white_space("nowrap")
        .selector(
            "& > svg",
            sx().flex("0 0 auto")
                .width("1em")
                .height("1em")
                .color("grey.6"),
        )
        .selector("& > [data-slot='dial']", sx().color("text-dimmed"))
        .when("disabled", sx().cursor("not-allowed"))
});

/// The static prefix `country_select: false` draws instead of the picker -
/// the same `+49`, with no tab stop and nothing to open.
static PREFIX_SX: StaticSx = StaticSx::new(|| sx().white_space("nowrap"));

/// The search box at the top of the list, which is what makes 240 rows
/// reachable. Copied from `SelectCore`'s: it sits inside the dropdown, above
/// the rows and outside their scroll, so it carries its own chrome rather than
/// the field frame's.
static SEARCH_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .border("none")
        .outline("none")
        .background("transparent")
        .color("inherit")
        .font_family("inherit")
        .font_size("inherit")
        .line_height("1.5")
        .padding("4px 8px")
        .border_bottom("1px solid")
        .border_color("grey.3")
        .selector("::placeholder", sx().color("text-dimmed"))
});

/// One row: the caller's flag if there is one, the country's name, its dial
/// code.
static ROW_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .gap("8px")
        .width("100%")
        .selector(
            "& > [data-slot='name']",
            sx().flex("1 1 auto")
                .min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis"),
        )
        .selector("& > [data-slot='dial']", sx().color("text-dimmed"))
});

field_props! {
    extends(input);
    pub struct PhoneFieldProps {
        /// The number, **in E.164** - `"+12133734253"`. `None` leaves the
        /// field uncontrolled; it keeps its own text and needs no handler.
        ///
        /// The text on screen is a rendering of this, never the value itself:
        /// what a caller holds, posts and validates is always the E.164 string.
        #[props(default, into)]
        value: Option<String>,
        /// Fires per keystroke with the E.164 the field should hold next, or
        /// an empty string once nothing is typed - an empty field is worth no
        /// value at all, not a bare dial code.
        #[props(default)]
        oninput: Option<EventHandler<String>>,
        /// The country the field starts on, ISO 3166-1 alpha-2 - `"DE"`.
        /// Defaults to the theme's. A `value` whose dial code belongs to
        /// another country wins over it, which is what makes an existing
        /// number render under the right flag.
        ///
        /// The picker wins over it too - until the prop changes, which moves
        /// the field and re-emits the number under the new dial code.
        #[props(default, into)]
        country: Option<String>,
        /// The user picked another country. The value is re-emitted through
        /// `oninput` at the same time, under the new dial code.
        #[props(default)]
        oncountrychange: Option<EventHandler<String>>,
        /// Offers the picker at all. Off pins the country and draws a static
        /// `+49` in its place, which is also one tab stop fewer.
        #[props(default)]
        country_select: Option<bool>,
        /// Overrides the English name a country is offered under, during
        /// render - so it can read a locale out of context. The library ships
        /// no translations.
        #[props(default)]
        country_label: Option<Callback<String, String>>,
        /// Narrows the list to these ISO codes, in the order given. Anything
        /// the table does not know is dropped.
        #[props(default)]
        countries: Option<Vec<String>>,
        /// Draws a flag beside a country, in the picker and in the list. The
        /// library ships none: emoji flags render as two letters on Windows
        /// and an SVG sprite is ~44 KB gzipped, so a caller who wants flags
        /// brings their own.
        #[props(default)]
        flag: Option<Callback<String, Element>>,
        /// Rules over the E.164, shown once the field loses focus or its form
        /// is submitted. Nothing here validates a number on its own - the
        /// library ships no numbering-plan data.
        #[props(default, into)]
        validate: crate::components::Validators<String>,
        /// What the field posts as - the E.164, through a hidden input,
        /// because the visible one holds the text being edited. A path -
        /// `Signup::FIELDS.phone()` - also binds it to the surrounding
        /// `Form`'s value when the field has no `oninput`.
        #[props(default, into)]
        name: crate::components::FieldName<String>,
        #[props(default, into)]
        placeholder: Option<String>,
    }
}

/// A phone field: a country picker in the leading slot, a `tel` input beside
/// it, and one E.164 string as the value.
///
/// Two values at once is what makes this a component rather than a `TextField`
/// with a `leading` the caller fills: the text being edited is the national
/// number, and the value the caller holds, posts and validates is the E.164
/// string assembled from it - `NumberField`'s edit buffer in another shape.
///
/// The library ships the country list and the dial codes, and **no
/// validation**: nothing here knows that `+1 555` is short. Bring a
/// `Validator<String>` for that.
#[component]
pub fn PhoneField(props: PhoneFieldProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.phone_field.size);
    let radius = props.radius.copied_or(theme.phone_field.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.oninput.is_some());
    let disabled = bound.disabled(props.disabled);
    // Two editors, so two refusals: the national text (native `readonly`) and
    // the country picker, whose pick rewrites the E.164 value.
    let readonly = props.readonly.unwrap_or(false);
    let current = bound.value().or_else(|| props.value.clone());

    // The country the user picked, which only the picker writes. The prop is
    // the *initial* one, so a re-render never drags the field back to it.
    let initial = props
        .country
        .as_deref()
        .and_then(countries::find)
        .or_else(|| countries::find(theme.phone_field.country))
        .unwrap_or(&COUNTRIES[0]);
    let picked = use_signal(|| initial);
    let country = country_for(current.as_deref(), picked());

    // The text is the component's edit buffer, not the value: `NumberField`'s
    // shape. It is what the control shows for as long as it still assembles
    // into the E.164 the caller holds - otherwise the caller's value wins,
    // which is what makes the field controlled.
    let mut text = use_signal(String::new);
    let display = match &current {
        None => text(),
        Some(value) if countries::to_e164(country, &text()) == *value => text(),
        Some(value) => countries::national_of(value, country)
            .map(|national| countries::group(country, &national))
            .unwrap_or_default(),
    };
    // What a pick or a blur re-assembles from - the digits on screen, which is
    // not the same as the buffer when the caller's value is what won above.
    let typed = countries::digits_of(&display);
    let e164 = countries::to_e164(country, &typed);

    let state = use_combobox();
    let picker_element = use_element();
    let search_element = use_element();
    let mut query = use_signal(String::new);

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&e164))
        .bound(&bound)
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let emit = bound.emit(props.oninput);

    let label_of = props.country_label;
    let name_of = move |country: &'static Country| match &label_of {
        Some(label) => label.call(country.iso.to_string()),
        None => country.name.to_string(),
    };

    let oncountrychange = props.oncountrychange;
    let pick_emit = emit.clone();
    // An `Rc` rather than a `use_callback`: every row holds one, and it writes
    // signals this component reads from inside a click
    // ([[codebase/reentrant-handlers]]).
    // The `bool` is "the user did this": a pick closes the list, reports
    // through `oncountrychange` and takes the focus back, and a caller changing
    // `country` under the field does none of those things.
    let pick: Rc<dyn Fn(&'static Country, bool)> =
        Rc::new(move |next: &'static Country, by_user: bool| {
            let mut picked = picked;
            let mut text = text;
            picked.set(next);
            // The typed digits keep their meaning - only the dial code in front of
            // them changed.
            text.set(countries::group(next, &typed));
            if let Some(emit) = &pick_emit {
                emit(countries::to_e164(next, &typed));
            }
            if !by_user {
                return;
            }
            state.close();
            if let Some(oncountrychange) = &oncountrychange {
                oncountrychange.call(next.iso.to_string());
            }
            // The list is gone and the box it was typed in with it, so the focus
            // goes back to what opened it.
            let _ = picker_element.focus();
        });

    // `country` is the country the field starts on, but a caller who changes it
    // has changed their mind, and the number moves with it - the same work a
    // pick does, minus the list, the callback and the focus. Guarded on the
    // country actually differing, so the first render emits nothing.
    let requested = props.country.clone();
    let following = pick.clone();
    use_effect(use_reactive!(|requested| {
        if let Some(next) = requested.as_deref().and_then(countries::find)
            && next.iso != country.iso
        {
            following(next, false);
        }
    }));

    let flag = props.flag;
    // One prepared style, cloned per row - `PinField`'s cells, for the same
    // reason: `use_box` is a hook and a row is not a component.
    let row_box = use_box().framework_sx(&ROW_SX).prepare();

    // Nothing is built while the list is closed. 240 rows is a `Vec<Element>`
    // and 240 name lookups per render, and a closed `ComboboxCore` draws none
    // of them - which is exactly the cost [[todos]] 29 is about.
    let opened = state.is_open() && !disabled;
    let needle = query().trim().to_lowercase();
    let rows: Vec<Element> = match opened {
        false => Vec::new(),
        true => offered(&props.countries)
            .into_iter()
            .filter(|country| {
                needle.is_empty()
                    || name_of(country).to_lowercase().contains(&needle)
                    || country.iso.to_lowercase().starts_with(&needle)
                    || country.dial.starts_with(needle.trim_start_matches('+'))
            })
            .map(|row| {
                let name = name_of(row);
                let drawn = flag.map(|flag| flag.call(row.iso.to_string()));
                let pick = pick.clone();
                let content = row_box.clone().render(
                    HtmlTag::Div,
                    Vec::new(),
                    rsx! {
                        {drawn}
                        span { "data-slot": "name", "{name}" }
                        span { "data-slot": "dial", "+{row.dial}" }
                    },
                );
                rsx! {
                    ComboboxOption {
                        selected: row.iso == country.iso,
                        onpick: move |_| pick(row, true),
                        {content}
                    }
                }
            })
            .collect(),
    };

    let picker_box = use_box()
        .framework_sx(&PICKER_SX)
        .states(field.states())
        .prepare();
    let prefix_box = use_box().framework_sx(&PREFIX_SX).prepare();
    let search_box = use_box().framework_sx(&SEARCH_SX).prepare();

    // Built only while the list is open - it is the only time `ComboboxCore`
    // renders it, and it carries three event closures.
    let search = search_box
        .element(&search_element)
        .attr_default("type", "text")
        .attr("value", query())
        .attr("data-controlled", true)
        .attr("placeholder", "Search")
        .attr("aria-label", "Search countries")
        // Ours is the list underneath; the browser's would cover it.
        .attr("autocomplete", "off")
        .attr("aria-autocomplete", "list")
        .event("oninput", move |event: FormEvent| {
            query.set(event.value());
            // The list under the highlight just changed; arm its top row.
            state.set_active(Some(0));
        })
        // The rows and the dropdown cancel `mousedown`, so a click inside the
        // list never reaches this - which leaves an outside click, and that is
        // exactly what should close the picker.
        .event("onblur", move |_: FocusEvent| state.close())
        .render(HtmlTag::Input, state.a11y_attributes(), ());

    let picker_name = name_of(country);
    let picker_flag = flag.map(|flag| flag.call(country.iso.to_string()));
    let picker = picker_box
        .element(&picker_element)
        // A button inside a form submits it unless it says otherwise.
        .attr_default("type", "button")
        // Two elements cannot both be the combobox: while the list is open the
        // search box owns the role, `aria-controls` and the active descendant,
        // and the button keeps only what says a list hangs off it.
        .attr("aria-haspopup", "listbox")
        .attr("aria-expanded", state.is_open().to_string())
        // The content reads `DE +49`, which names a code and not a country.
        .attr("aria-label", format!("Country: {picker_name}"))
        .attr("disabled", disabled)
        // Or clicking the button while the list is open closes it twice over:
        // the press blurs the search box, which closes the list, and the click
        // that follows opens it again.
        .event("onmousedown", move |event: MouseEvent| {
            event.prevent_default()
        })
        .event("onclick", move |_: MouseEvent| {
            if !disabled && !readonly {
                state.toggle();
            }
        })
        .render(
            HtmlTag::Button,
            Vec::new(),
            rsx! {
                {picker_flag}
                span { "data-slot": "iso", "{country.iso}" }
                span { "data-slot": "dial", "+{country.dial}" }
                ChevronIcon {}
            },
        );

    let leading = match props
        .country_select
        .unwrap_or(theme.phone_field.country_select)
    {
        true => Some(rsx! {
            ComboboxCore {
                rows,
                active: state.active(),
                onactive: move |row| state.set_active(Some(row)),
                opened,
                onopened: move |opened: bool| {
                    state.set_open(opened);
                    if !opened {
                        query.set(String::new());
                        // Escape, Enter and Tab all close through here, and the
                        // box that held the focus has just unmounted.
                        let _ = picker_element.focus();
                    }
                },
                state,
                size,
                radius,
                disabled,
                width: PopoverWidth::Min,
                header: opened.then_some(search),
                autofocus: search_element,
                {picker}
                // The picker sits inside the combobox's wrapper, not straight
                // in the slot, so it brings the ring overlay it needs.
                {ring_overlay()}
            }
        }),
        false => Some(prefix_box.render(HtmlTag::Span, Vec::new(), rsx! { "+{country.dial}" })),
    };

    let frame = use_field_frame()
        .leading(&leading)
        .states(field.states())
        .prepare();

    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let input_emit = emit.clone();
    let blur_country = country;
    let input = field
        .aria(control)
        .attr_default("type", "tel")
        .attr_default("inputmode", "tel")
        // The dial code is the picker's, so the browser should offer the
        // national part alone.
        .attr_default("autocomplete", "tel-national")
        // The hidden input below is what posts - this one holds the text.
        .attr("value", display)
        .attr("data-controlled", true)
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("readonly", readonly)
        .attr("required", required)
        .event("oninput", move |event: FormEvent| {
            let raw = event.value();
            if let Some(emit) = &input_emit {
                emit(countries::to_e164(country, &raw));
            }
            text.set(raw);
        })
        // Grouped on the way out rather than on every keystroke: regrouping as
        // the text is typed moves the caret, and `ElementApi` has no
        // `selection_start` to put it back.
        .event("onblur", move |_: FocusEvent| {
            let grouped = countries::group(blur_country, &text());
            if grouped != text() {
                text.set(grouped);
            }
        })
        .render(HtmlTag::Input, props.attributes, ());

    // The E.164 is what posts. The visible input holds the display text and
    // would otherwise post that - the `Slider`/`PinField`/`TagsField` shape.
    let hidden = bound.name().map(str::to_string).map(|name| {
        rsx! {
            input {
                r#type: "hidden",
                name,
                value: "{e164}",
                disabled: disabled.then_some(true),
            }
        }
    });

    field.render(rsx! {
        {frame.render(input)}
        {hidden}
    })
}

/// The countries the picker offers, in the caller's order when there is one.
/// Only called while the list is open.
fn offered(codes: &Option<Vec<String>>) -> Vec<&'static Country> {
    match codes {
        Some(codes) => codes
            .iter()
            .filter_map(|code| countries::find(code))
            .collect(),
        None => COUNTRIES.iter().collect(),
    }
}

/// Which country a rendered field is on: the one the user picked, unless the
/// value it is holding belongs somewhere else.
///
/// The pick wins whenever the value still fits it, which is what settles the
/// twenty countries sharing `+1`: typing `242` into a US field does not silently
/// become the Bahamas, while a value that arrives as `+1242...` with nothing
/// picked does render as one.
pub(crate) fn country_for(value: Option<&str>, picked: &'static Country) -> &'static Country {
    let digits = countries::digits_of(value.unwrap_or_default());
    if digits.is_empty() || digits.starts_with(picked.dial) {
        return picked;
    }
    countries::by_dial(&digits).unwrap_or(picked)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn country(iso: &str) -> &'static Country {
        countries::find(iso).expect("a country in the table")
    }

    /// The value is E.164 and only E.164, whatever punctuation was typed.
    #[test]
    fn the_value_is_assembled_from_the_dial_code_and_the_digits() {
        assert_eq!(
            countries::to_e164(country("US"), "(213) 373-4253"),
            "+12133734253"
        );
        assert_eq!(
            countries::to_e164(country("DE"), "30 123456"),
            "+4930123456"
        );
        // Nothing typed is worth no value at all, not a bare dial code.
        assert_eq!(countries::to_e164(country("DE"), "  -"), "");
    }

    /// An existing number renders under its own country, not under whatever
    /// the field happened to start on.
    #[test]
    fn a_value_moves_the_field_to_the_country_its_dial_code_names() {
        assert_eq!(country_for(Some("+4930123456"), country("US")).iso, "DE");
        assert_eq!(
            countries::national_of("+4930123456", country("DE")),
            Some("30123456".to_string())
        );
    }

    /// Twenty countries share `+1`, so a longest-prefix match alone would move
    /// a US field to the Bahamas the moment an area code matched.
    #[test]
    fn the_pick_wins_over_the_dial_code_whenever_the_value_still_fits_it() {
        assert_eq!(country_for(Some("+12425550123"), country("US")).iso, "US");
        // With nothing of its own to go on, the longest dial code decides.
        assert_eq!(country_for(Some("+12425550123"), country("DE")).iso, "BS");
    }

    /// An empty value never drags the field off the country it is on.
    #[test]
    fn an_empty_value_leaves_the_country_alone() {
        assert_eq!(country_for(None, country("DE")).iso, "DE");
        assert_eq!(country_for(Some(""), country("DE")).iso, "DE");
    }

    /// Grouping is deliberately sparse: where a numbering plan has one fixed
    /// shape it is used, and everywhere else the digits are left alone rather
    /// than invented.
    #[test]
    fn blur_groups_only_where_the_plan_is_fixed() {
        assert_eq!(
            countries::group(country("US"), "2133734253"),
            "213 373 4253"
        );
        assert_eq!(
            countries::group(country("FR"), "612345678"),
            "6 12 34 56 78"
        );
        assert_eq!(countries::group(country("DE"), "30 123456"), "30123456");
        // A number of another length is not a number we know how to group.
        assert_eq!(countries::group(country("US"), "213373"), "213373");
    }

    /// Every dial code is digits, every ISO code is two upper-case letters, and
    /// no code appears twice - the table is hand-kept, so this is the guard.
    #[test]
    fn the_table_is_well_formed() {
        let mut seen: Vec<&str> = Vec::new();
        for entry in COUNTRIES {
            assert!(
                entry.iso.len() == 2 && entry.iso.chars().all(|c| c.is_ascii_uppercase()),
                "{}",
                entry.iso
            );
            assert!(
                !entry.dial.is_empty() && entry.dial.chars().all(|c| c.is_ascii_digit()),
                "{}",
                entry.iso
            );
            assert!(!seen.contains(&entry.iso), "{} twice", entry.iso);
            seen.push(entry.iso);
        }
    }
}
