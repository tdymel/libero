use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{ChevronDownIcon, ComboboxState, HtmlTag, Input, use_combobox},
        form::{
            CaretKeys, ComboboxCore, ComboboxOption, FIELD_CONTROL_SX, field_props, use_bound,
            use_field, use_field_frame, with_drawn_placeholder,
        },
        layout::{BoxStyle, use_box},
    },
    hooks::{
        ElementHandle, PopoverWidth, current_localization, use_element, use_localization, use_theme,
    },
    localization::fill,
    platform::ElementApi,
    sx::{StaticSx, sx},
    theme::Size,
};

use super::countries::{self, COUNTRIES, Country};

/// The picker in the leading slot: a real `<button>` and tab stop, the only
/// way to reach what it does.
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
        // 2.5.8's 24px, which a line of `xs`-`md` text falls short of (todo 763).
        .min_height("24px")
        .padding("0")
        .margin("0")
        .cursor("pointer")
        .white_space("nowrap")
        .selector(
            "& > svg",
            sx().flex("0 0 auto")
                .width("1em")
                .height("1em")
                .color("muted.6"),
        )
        .selector("& > [data-slot='dial']", sx().color("text-dimmed"))
        .when("disabled", sx().cursor("not-allowed"))
});

/// The static prefix `country_select: false` draws instead of the picker -
/// the same `+49`, with no tab stop and nothing to open.
static PREFIX_SX: StaticSx = StaticSx::new(|| sx().white_space("nowrap"));

/// The search box above the rows, copied from `SelectCore`'s. It sits outside
/// the field frame, so it carries its own chrome.
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
        // The box's only boundary: 3:1, as a field frame (WCAG 1.4.11, todo 490).
        .border_color("muted.6")
        .selector("::placeholder", sx().color("text-dimmed"))
});

/// [`SEARCH_SX`]'s padding and bottom border, where a drawn placeholder sits.
const SEARCH_INSET: &str = "4px 8px 5px";

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
        // Dimmed text misses 4.5:1 on the selected tint; the row's own colour reads.
        .selector(
            ":where([aria-selected='true']) & > [data-slot='dial']",
            sx().color("inherit"),
        )
});

field_props! {
    extends(input);
    pub struct PhoneFieldProps {
        /// The number in E.164 (`"+12133734253"`); `None` leaves it uncontrolled.
        #[props(default, into)]
        value: Option<String>,
        /// Fires per keystroke with the next E.164, or `""` once nothing is typed.
        #[props(default)]
        oninput: Option<EventHandler<String>>,
        /// The starting country, ISO alpha-2 (`"DE"`). A `value`'s own dial code
        /// wins; a change re-emits the number under the new code.
        #[props(default, into)]
        country: Option<String>,
        /// The user picked another country; `oninput` re-emits the value too.
        #[props(default)]
        oncountrychange: Option<EventHandler<String>>,
        /// `false` pins the country and draws a static `+49` instead.
        #[props(default)]
        country_select: Option<bool>,
        /// Overrides the name a country is offered under, during render.
        #[props(default)]
        country_label: Option<Callback<String, String>>,
        /// Narrows the list to these ISO codes, in the order given.
        #[props(default)]
        countries: Option<Vec<String>>,
        /// Draws a flag beside a country; none ship. Keep it `aria-hidden`.
        #[props(default)]
        flag: Option<Callback<String, Element>>,
        /// Rules over the E.164, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<String>,
        /// What the E.164 posts as. A path also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<String>,
        #[props(default, into)]
        placeholder: Option<String>,
    }
}

/// A country picker and a `tel` input, holding one E.164 string. Validates
/// nothing on its own.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::PhoneField;
/// # fn app() -> Element {
/// let mut phone = use_signal(String::new);
/// rsx! {
///     PhoneField {
///         label: "Phone",
///         country: "DE",
///         value: phone(),
///         oninput: move |next| phone.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/phone-field>
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

    // Only the picker writes it; the prop is the initial country.
    let initial = props
        .country
        .as_deref()
        .and_then(countries::find)
        .or_else(|| countries::find(theme.phone_field.country))
        .unwrap_or(&COUNTRIES[0]);
    let picked = use_signal(|| initial);
    let country = country_for(current.as_deref(), picked());

    // The edit buffer, shown while it still assembles into the caller's E.164;
    // otherwise the caller's value wins.
    let text = use_signal(String::new);
    let display = match &current {
        None => text(),
        Some(value) if countries::to_e164(country, &text()) == *value => text(),
        Some(value) => countries::national_of(value, country)
            .map(|national| countries::group(country, &national).unwrap_or(national))
            .unwrap_or_default(),
    };
    // A pick or a blur re-assembles from the digits on screen, not the buffer.
    let typed = countries::digits_of(&display);
    // The pick closure below outlives `display`, which the input takes.
    let shown = display.clone();
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

    let localization = use_localization();
    let label_of = props.country_label;
    let names = localization.phone_field;
    let name_of = move |country: &'static Country| match &label_of {
        Some(label) => label.call(country.iso.to_string()),
        None => names
            .country_name(country.iso)
            .unwrap_or(country.name)
            .to_string(),
    };

    let oncountrychange = props.oncountrychange;
    let pick_emit = emit.clone();
    // An `Rc`, not a `use_callback`: it writes signals read here from a click
    // ([[codebase/reentrant-handlers]]). The `bool` is "the user did this".
    let pick: Rc<dyn Fn(&'static Country, bool)> =
        Rc::new(move |next: &'static Country, by_user: bool| {
            let mut picked = picked;
            let mut text = text;
            picked.set(next);
            // Only the dial code changed; without a fixed shape the typed text
            // keeps its spacing (todo 87b).
            text.set(countries::group(next, &typed).unwrap_or_else(|| shown.clone()));
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
            // The list and its search box are gone; focus returns to the picker.
            let _ = picker_element.focus();
        });

    // A changed `country` prop moves the number like a pick would. Guarded on
    // a real change, so the first render emits nothing.
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

    let opened = state.is_open() && !disabled && !readonly;
    // The list opens on the current country, like a native `<select>`.
    let home_row = offered(&props.countries)
        .iter()
        .position(|row| row.iso == country.iso);
    let rows = phone_rows(
        RowList {
            opened,
            codes: props.countries.clone(),
            needle: query().trim().to_lowercase(),
            country,
            row_box,
            flag,
        },
        name_of,
        &pick,
    );

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
        .attr("placeholder", localization.common.search)
        .attr("aria-label", localization.phone_field.search)
        // Ours is the list underneath; the browser's would cover it.
        .attr("autocomplete", "off")
        .attr("aria-autocomplete", "list")
        .event("oninput", move |event: FormEvent| {
            query.set(event.value());
            // The list under the highlight just changed; arm its top row.
            state.set_active(Some(0));
        })
        // The list cancels `mousedown`, so only an outside click blurs this.
        .event("onblur", move |_: FocusEvent| state.close())
        .render(HtmlTag::Input, state.a11y_attributes(), ());
    let search = with_drawn_placeholder(Some(localization.common.search), SEARCH_INSET, search);

    let picker = phone_picker(
        picker_box,
        PickerButton {
            element: picker_element,
            state,
            country,
            picker_name: name_of(country),
            flag,
            opened,
            home_row,
            disabled,
            readonly,
        },
    );

    let with_select = props
        .country_select
        .unwrap_or(theme.phone_field.country_select);
    let dial_id = format!("{}-dial", field.id());
    let leading = phone_leading(
        with_select,
        Picker {
            picker,
            prefix_box,
            search,
            search_element,
            picker_element,
            state,
            query,
            opened,
            home_row,
            size,
            radius,
            disabled: disabled || readonly,
            dial: country.dial,
            dial_id: dial_id.clone(),
        },
        rows,
    );

    let frame = use_field_frame()
        .leading(&leading)
        .states(field.states())
        .placeholder(props.placeholder.as_deref())
        .prepare();

    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();
    // `field.aria()` by hand: the bare dial code is visible text the input's
    // name lacks, so it joins the description (2.5.3).
    let describedby = match (with_select, field.describedby()) {
        (true, captions) => captions,
        (false, None) => Some(dial_id),
        (false, Some(captions)) => Some(format!("{dial_id} {captions}")),
    };
    let control = control
        .attr("id", field.id().to_string())
        .attr("aria-describedby", describedby)
        .attr("aria-invalid", field.invalid().then_some("true"))
        .attr("aria-required", required.then_some("true"));

    let input = phone_input(
        control,
        Entry {
            display,
            text,
            country,
            placeholder: props.placeholder,
            disabled,
            readonly,
            required,
        },
        emit.clone(),
        props.attributes,
    );

    let hidden = phone_hidden(bound.name().map(str::to_string), &e164, disabled);

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

/// The field's country: the pick while the value fits it, so `242` typed in a
/// US field stays US; else the value's own dial code.
pub(crate) fn country_for(value: Option<&str>, picked: &'static Country) -> &'static Country {
    let digits = countries::digits_of(value.unwrap_or_default());
    if digits.is_empty() || digits.starts_with(picked.dial) {
        return picked;
    }
    countries::by_dial(&digits).unwrap_or(picked)
}

/// What the country list is drawn from.
struct RowList {
    opened: bool,
    codes: Option<Vec<String>>,
    needle: String,
    country: &'static Country,
    row_box: BoxStyle,
    flag: Option<Callback<String, Element>>,
}

/// The country rows.
fn phone_rows(
    list: RowList,
    name_of: impl Fn(&'static Country) -> String,
    pick: &Rc<dyn Fn(&'static Country, bool)>,
) -> Vec<Element> {
    let RowList {
        opened,
        codes,
        needle,
        country,
        row_box,
        flag,
    } = list;
    let codes = &codes;
    // Nothing built while closed: 240 rows and name lookups per render (todo 29).
    if !opened {
        return Vec::new();
    }
    let dial = needle.trim_start_matches('+');
    // Rank 0 starts with the query, 1 only contains it, so "fr" tops with France
    // and Enter picks it, not the Central African Republic.
    let rank = |country: &Country, name: &str| -> Option<u8> {
        let lower = name.to_lowercase();
        if needle.is_empty()
            || lower.starts_with(&needle)
            || country.iso.to_lowercase() == needle
            || country.dial.starts_with(dial)
        {
            Some(0)
        } else if lower.contains(&needle) || country.iso.to_lowercase().starts_with(&needle) {
            Some(1)
        } else {
            None
        }
    };
    let mut matched: Vec<(u8, &'static Country, String)> = offered(codes)
        .into_iter()
        .filter_map(|country| {
            let name = name_of(country);
            rank(country, &name).map(|rank| (rank, country, name))
        })
        .collect();
    // The caller's order stands; the full list follows the shown names.
    match codes {
        Some(_) => matched.sort_by_key(|(rank, _, _)| *rank),
        None => matched.sort_by_cached_key(|(rank, _, name)| (*rank, sort_key(name))),
    }
    matched
        .into_iter()
        .map(|(_, row, name)| {
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
        .collect()
}

/// A name's place in the list: lowercase, Latin accents folded to their base
/// letter, so "Österreich" sorts among the O's rather than after "Zypern".
fn sort_key(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| match c {
            'à'..='å' => 'a',
            'ç' => 'c',
            'è'..='ë' => 'e',
            'ì'..='ï' => 'i',
            'ñ' => 'n',
            'ò'..='ö' | 'ø' => 'o',
            'ù'..='ü' => 'u',
            'ý' | 'ÿ' => 'y',
            c => c,
        })
        .collect()
}

/// What the country button shows and what its click toggles.
struct PickerButton {
    element: ElementHandle,
    state: ComboboxState,
    country: &'static Country,
    picker_name: String,
    flag: Option<Callback<String, Element>>,
    /// Whether the list is drawn, which a disabled or read-only field never is.
    opened: bool,
    /// The current country's row, which a click opens on.
    home_row: Option<usize>,
    disabled: bool,
    readonly: bool,
}

/// The country button in the frame's leading slot: a real `<button>` and a
/// real tab stop, because it is the only way to reach what it does.
fn phone_picker(picker_box: BoxStyle, button: PickerButton) -> Element {
    let PickerButton {
        element,
        state,
        country,
        picker_name,
        flag,
        opened,
        home_row,
        disabled,
        readonly,
    } = button;
    let picker_flag = flag.map(|flag| flag.call(country.iso.to_string()));
    picker_box
        .element(&element)
        // A button inside a form submits it unless it says otherwise.
        .attr_default("type", "button")
        // What names the listbox.
        .attr("id", picker_id(state))
        // The open list's search box is the combobox; the button only says a
        // list hangs off it.
        .attr("aria-haspopup", "listbox")
        .attr("aria-expanded", opened.to_string())
        // The content reads `DE +49`, which names a code and not a country; the
        // name keeps it too, for speech input (WCAG 2.5.3).
        .attr(
            "aria-label",
            fill(
                current_localization().phone_field.country,
                &[
                    ("name", &picker_name),
                    ("iso", &country.iso),
                    ("dial", &country.dial),
                ],
            ),
        )
        .attr("disabled", disabled)
        // Read-only keeps the tab stop but says the button does nothing.
        .attr("aria-disabled", (readonly && !disabled).then_some("true"))
        // Else the press blurs the search box, closing the list, and the click
        // reopens it.
        .event("onmousedown", move |event: MouseEvent| {
            event.prevent_default()
        })
        .event("onclick", move |_: MouseEvent| {
            if !disabled && !readonly {
                if !state.is_open() {
                    state.set_active(home_row);
                }
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
                ChevronDownIcon {}
            },
        )
}

/// The frame's leading slot: the button, the open list's search box, and the
/// two handles focus moves between.
struct Picker {
    picker: Element,
    prefix_box: BoxStyle,
    search: Element,
    search_element: ElementHandle,
    picker_element: ElementHandle,
    state: ComboboxState,
    query: Signal<String>,
    opened: bool,
    home_row: Option<usize>,
    size: Size,
    radius: Size,
    disabled: bool,
    dial: &'static str,
    dial_id: String,
}

/// The frame's leading slot: the country picker with its list, or - with
/// `country_select` off - the bare dial code.
fn phone_leading(with_select: bool, parts: Picker, rows: Vec<Element>) -> Option<Element> {
    let Picker {
        picker,
        prefix_box,
        search,
        search_element,
        picker_element,
        state,
        mut query,
        opened,
        home_row,
        size,
        radius,
        disabled,
        dial,
        dial_id,
    } = parts;

    match with_select {
        true => Some(rsx! {
            ComboboxCore {
                rows,
                active: state.active(),
                onactive: move |row| state.set_active(Some(row)),
                opened,
                onopened: move |opened: bool| {
                    // Over the row the opening arrow armed, as `Select` does.
                    if opened && !state.is_open() {
                        state.set_active(home_row);
                    }
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
                caret_keys: CaretKeys::Always,
                labelled_by: Some(picker_id(state)),
                {picker}
            }
        }),
        false => Some(prefix_box.attr("id", dial_id).render(
            HtmlTag::Span,
            Vec::new(),
            rsx! { "+{dial}" },
        )),
    }
}

fn picker_id(state: ComboboxState) -> String {
    format!("{}-picker", state.id())
}

/// What the national-number input shows and writes. The text is the
/// component's edit buffer, not the value.
struct Entry {
    display: String,
    text: Signal<String>,
    country: &'static Country,
    placeholder: Option<String>,
    disabled: bool,
    readonly: bool,
    required: bool,
}

/// The national number. The E.164 posts from the hidden input beside it; this
/// one only ever holds the text.
fn phone_input<F: Fn(String) + Clone + 'static>(
    control: BoxStyle,
    entry: Entry,
    emit: Option<F>,
    attributes: Vec<Attribute>,
) -> Element {
    let Entry {
        display,
        mut text,
        country,
        placeholder,
        disabled,
        readonly,
        required,
    } = entry;

    let input_emit = emit;
    let blur_country = country;
    control
        .attr_default("type", "tel")
        .attr_default("inputmode", "tel")
        // The dial code is the picker's, so the browser should offer the
        // national part alone.
        .attr_default("autocomplete", "tel-national")
        // The hidden input below is what posts - this one holds the text.
        .attr("value", display)
        .attr("data-controlled", true)
        .attr("placeholder", placeholder)
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
        // Grouped on blur: regrouping per keystroke moves the caret, and
        // `ElementApi` cannot put it back.
        .event("onblur", move |_: FocusEvent| {
            // Only where the plan has a fixed shape (todo 87b).
            if let Some(grouped) = countries::group(blur_country, &text())
                && grouped != text()
            {
                text.set(grouped);
            }
        })
        .render(HtmlTag::Input, attributes, ())
}

/// The E.164 is what posts. The visible input holds the display text and would
/// otherwise post that - the `Slider`/`PinField`/`TagsField` shape.
fn phone_hidden(name: Option<String>, e164: &str, disabled: bool) -> Option<Element> {
    let e164 = e164.to_string();
    // The E.164 is what posts. The visible input holds the display text and
    // would otherwise post that - the `Slider`/`PinField`/`TagsField` shape.
    name.map(|name| {
        rsx! {
            input {
                r#type: "hidden",
                name,
                value: "{e164}",
                disabled: disabled.then_some(true),
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn country(iso: &str) -> &'static Country {
        countries::find(iso).expect("a country in the table")
    }

    /// The list sorts by the shown name, an umlaut with its base letter.
    #[test]
    fn localized_names_sort_with_their_base_letter() {
        let mut names = ["Zypern", "Österreich", "Ägypten", "Deutschland", "Oman"];
        names.sort_by_cached_key(|name| sort_key(name));
        assert_eq!(
            names,
            ["Ägypten", "Deutschland", "Oman", "Österreich", "Zypern"]
        );
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

    /// A shared dial code belongs to the main country, not to whichever row
    /// sorts last by name.
    #[test]
    fn a_shared_dial_code_falls_to_its_main_country() {
        for (value, iso) in [
            ("+61212345678", "AU"),
            ("+390612345678", "IT"),
            ("+6491234567", "NZ"),
            ("+4712345678", "NO"),
            ("+590590123456", "GP"),
            ("+12135550123", "US"),
            ("+18095550123", "US"),
            ("+13405550123", "VI"),
        ] {
            assert_eq!(country_for(Some(value), country("DE")).iso, iso, "{value}");
        }
        // A pick that shares the code keeps the number, area code and all.
        assert_eq!(country_for(Some("+18295550123"), country("DO")).iso, "DO");
        assert_eq!(
            countries::to_e164(country("DO"), "829 555 0123"),
            "+18295550123"
        );
    }

    /// Every shared dial code has exactly one main country, so the tie-break
    /// never falls back to table order.
    #[test]
    fn every_shared_dial_code_has_one_main_country() {
        for entry in COUNTRIES {
            let sharers: Vec<&Country> = COUNTRIES
                .iter()
                .filter(|other| other.dial == entry.dial)
                .collect();
            let main = sharers
                .iter()
                .filter(|other| !countries::SHARING.contains(&other.iso))
                .count();
            assert_eq!(main, 1, "+{} has {main} main countries", entry.dial);
        }
    }

    /// An empty value never drags the field off the country it is on.
    #[test]
    fn an_empty_value_leaves_the_country_alone() {
        assert_eq!(country_for(None, country("DE")).iso, "DE");
        assert_eq!(country_for(Some(""), country("DE")).iso, "DE");
    }

    /// Only a fixed-shape plan groups; elsewhere the typed text stays (todo 87b).
    #[test]
    fn blur_groups_only_where_the_plan_is_fixed() {
        assert_eq!(
            countries::group(country("US"), "2133734253").as_deref(),
            Some("213 373 4253")
        );
        assert_eq!(
            countries::group(country("FR"), "612345678").as_deref(),
            Some("6 12 34 56 78")
        );
        assert_eq!(countries::group(country("DE"), "30 123456"), None);
        // A number of another length is not a number we know how to group.
        assert_eq!(countries::group(country("US"), "213373"), None);
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

    /// The German names cover exactly the table's codes, each once.
    #[test]
    fn the_german_names_cover_the_table() {
        let names = crate::localization::PhoneFieldLabels::GERMAN.country_names;
        assert_eq!(names.len(), COUNTRIES.len());
        for entry in COUNTRIES {
            let hits = names.iter().filter(|(iso, _)| *iso == entry.iso).count();
            assert_eq!(hits, 1, "{} named {hits} times", entry.iso);
        }
    }
}
