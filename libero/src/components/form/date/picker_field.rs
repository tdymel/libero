//! The engine every date and time field shares: a text input read on blur or Enter,
//! a dropdown holding a picker, and a hidden input posting ISO 8601.

use std::rc::Rc;

use chrono::NaiveDate;
use dioxus::prelude::*;

use super::{
    field_value::{FieldValue, Formats},
    range::Iso,
    today::use_today_on_open,
};
use crate::{
    components::{
        accessibility::use_announcer,
        buttons::Button,
        common::{
            ClassList, HtmlTag, Input, NavigationChord, Part, Parts, ScaleOrCss, States,
            navigation_chord,
        },
        form::{
            Caption, ChronoDropdownPart, FIELD_CONTROL_SX, FieldError, FieldName, FieldPart,
            FieldStatus, FormScope, Parse, Typed, Validators,
            keyboardless::{PickerInput, use_keyboardless, use_picker_focus},
            phone_sheet::{
                SheetSwipe, narrow_query, phone_sheet_sx, sheet_clearance_style,
                use_sheet_clearance, use_sheet_room, use_sheet_swipe,
            },
            use_bound, use_field, use_field_frame, word,
        },
        layout::{paper_sx, use_box},
    },
    hooks::{
        POPOVER_AVAILABLE_HEIGHT, PopoverOptions, placement_sx, use_element, use_field_list_layer,
        use_focus_within, use_form_owner, use_localization, use_media_query, use_popover_on,
        use_theme,
    },
    localization::Localization,
    platform::is_tab,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Size, SizeCss, Z_INDEX_POPOVER},
};

static PICKER_FIELD_DROPDOWN_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        .and(placement_sx(true))
        .z_index(Z_INDEX_POPOVER.value())
        .padding(SizeCss::SPACING.value(Size::Sm))
        // The field's corner, and a floating shadow over the surface's resting one.
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .box_shadow(SizeCss::SHADOW.value(Size::Lg))
        // Never past the room on its side: it scrolls instead (WCAG 1.4.10).
        .max_height(POPOVER_AVAILABLE_HEIGHT.value_or("none"))
        .overflow_y("auto")
        .selector(
            "& > :is([data-slot='text-input'], [data-slot='save'])",
            sx().margin_top(SizeCss::SPACING.value(Size::Sm)),
        )
        // Below `sm` the phone sheet, which a sticky header cannot cover (todo 3900).
        .and(phone_sheet_sx())
        .media(
            narrow_query(),
            // One centred column: a digital clock sat left on a wide phone. The swipe strip and
            // text box span it; a range's months shrink and scroll between them (3903).
            sx().display("flex")
                .flex_direction("column")
                .align_items("center")
                .selector(
                    "& > :is([data-slot='sheet-handle'], [data-slot='text-input'], [data-slot='save'])",
                    sx().align_self("stretch"),
                )
                // The clock face fills the sheet's width (3926); `div` outranks the picker's own rule.
                // Sized off the screen, not its parent: a parent's width would need `:has()` (3954).
                .selector(
                    "& div[data-slot='face']",
                    sx().width(format!(
                        "min(calc(100vw - 2 * {}), 50dvh)",
                        SizeCss::SPACING.value(Size::Sm)
                    ))
                    .height("auto")
                    .with("aspect-ratio", "1"),
                ),
        )
});

/// Where Arrow Down in the input puts focus: the picker's tab stop, not the navigation above it.
const DROPDOWN_ENTRY: &str = ":is([data-slot='months'], [data-slot='cells'], [data-slot='columns']) [tabindex='0'], [data-slot='face'][tabindex='0']";

/// Everything a field hands the engine from its `field_props!`.
pub(super) struct PickerField<'a, V: 'static> {
    pub value: Option<V>,
    pub onchange: Option<EventHandler<Option<V>>>,
    pub validate: &'a Validators<Option<V>>,
    pub name: &'a FieldName<Option<V>>,
    pub today: Option<NaiveDate>,
    pub placeholder: Option<String>,
    pub leading: Option<Element>,
    pub trailing: Option<Element>,
    pub label: &'a Caption,
    pub description: &'a Caption,
    pub helper: &'a Caption,
    pub status: &'a Input<FieldStatus>,
    pub size: Input<Size>,
    pub radius: Input<ThemeAwareValue>,
    pub required: Option<bool>,
    pub disabled: Option<bool>,
    pub readonly: Option<bool>,
    pub class: &'a Input<ClassList>,
    pub sx: &'a Input<Sx>,
    pub parts: &'a Input<Parts<FieldPart>>,
    pub dropdown_parts: &'a Input<Parts<ChronoDropdownPart>>,
    pub states: &'a Input<States>,
    pub attributes: Vec<Attribute>,
}

/// What the dropdown's picker is drawn from.
pub struct DropdownArgs<V: 'static> {
    pub value: Option<V>,
    pub today: Option<NaiveDate>,
    pub size: Size,
    /// Emits a picked value, closing the dropdown when the field's close rule says so.
    pub pick: Callback<Option<V>>,
}

/// Builds a [`PickerField`] from a field's props.
macro_rules! picker_field {
    ($props:ident, $today:expr) => {
        $crate::components::form::date::picker_field::PickerField {
            value: $props.value,
            onchange: $props.onchange,
            validate: &$props.validate,
            name: &$props.name,
            today: $today,
            placeholder: $props.placeholder.clone(),
            leading: $props.leading.clone(),
            trailing: $props.trailing.clone(),
            label: &$props.label,
            description: &$props.description,
            helper: &$props.helper,
            status: &$props.status,
            size: $props.size.clone(),
            radius: $props.radius.clone(),
            required: $props.required,
            disabled: $props.disabled,
            readonly: $props.readonly,
            class: &$props.class,
            sx: &$props.sx,
            parts: &$props.parts,
            dropdown_parts: &$props.dropdown_parts,
            states: &$props.states,
            attributes: $props.attributes,
        }
    };
}
pub(super) use picker_field;

/// Typed text stays until blur or Enter. Unreadable or refused text stays and shows an error.
pub(super) fn use_picker_field<V: FieldValue>(
    field: PickerField<'_, V>,
    formats: Formats,
    accepts: impl Fn(V) -> Result<(), String> + Clone + 'static,
    closes: impl Fn(Option<V>) -> bool + 'static,
    // The picker leaves the closing to a Save under it (3903).
    saves: bool,
    dropdown: impl FnOnce(DropdownArgs<V>) -> Element,
) -> Element {
    let theme = use_theme();
    let names = &use_localization().date;
    let defaults = &theme.chrono_field;
    let size = field.size.copied_or(defaults.size);
    let radius = ScaleOrCss::new(field.radius.as_ref(), defaults.radius);
    let required = crate::components::form::fieldset_required(field.required);
    let bound = use_bound(field.name, field.onchange.is_some());
    let disabled = bound.disabled(field.disabled);
    // Readonly: native on the text; the dropdown refuses to open, leaving focus and posting alone.
    let readonly = field.readonly.unwrap_or(false);
    let value = bound.value().unwrap_or(field.value);
    let opened = use_signal(|| false);
    let today = use_today_on_open(field.today, opened());
    // Peek-compared: focus then click both open, and an unchanged `set` still redraws.
    let open_to = move |next: bool| {
        let mut opened = opened;
        if *opened.peek() != next {
            opened.set(next);
        }
    };
    // Text the form kept refused for this path, from before an unmount.
    let scope = try_use_context::<FormScope>();
    let kept = use_hook(|| {
        let full = bound.name()?;
        scope?.typed(full).filter(|_| value.is_none())
    });
    // The text as typed, until it is committed. `None` shows `value`.
    let mut draft = use_signal(|| kept.as_ref().map(|typed| typed.text.clone()));
    // Why the last commit was refused, worded at render; cleared by the next keystroke.
    let mut rejected = use_signal(|| kept.map(|typed| typed.error));
    // Arrow Down asked for focus in the picker, once the dropdown is drawn.
    let mut entering = use_signal(|| false);
    let anchor = use_element();
    // A form reset drops the typed text and its refusal; a raw `<form>` is heard off the DOM.
    let owner = use_form_owner(anchor, scope.is_none());
    let resets = scope.map_or(0, |form| form.resets()) + owner().unwrap_or(0);
    // Not on mount, which may restore text the form kept.
    let seen = use_hook(|| Rc::new(std::cell::Cell::new(resets)));
    use_effect(use_reactive!(|resets| {
        if seen.replace(resets) == resets {
            return;
        }
        if draft.peek().is_some() {
            draft.set(None);
        }
        if rejected.peek().is_some() {
            rejected.set(None);
        }
    }));

    let onchange = field.onchange;
    let setter = bound.setter();
    // Refused text resets a bound value, as a mapped field's does; the form keeps the text.
    let full = bound.name().map(str::to_string);
    let keep = {
        let full = full.clone();
        let bound = setter.is_some() && onchange.is_none();
        move |typed: Option<Typed>| {
            let (Some(mut scope), Some(full)) = (scope, &full) else {
                return;
            };
            match typed {
                Some(typed) if bound => scope.keep_typed(full, typed),
                _ => scope.drop_typed(full),
            }
        }
    };
    let resets_value = setter.is_some() && onchange.is_none();
    let emit = move |next: Option<V>| match (&onchange, &setter) {
        (Some(onchange), _) => onchange.call(next),
        (None, Some(setter)) => setter.set(next),
        (None, None) => {}
    };
    // Below `sm` a range shows short while the input is not focused, so typed text never
    // reads it back (todo 3904).
    let narrow = use_media_query(&narrow_query());
    let focused = use_signal(|| false);
    let short = narrow() && !focused();
    let text = draft().unwrap_or_else(|| {
        value
            .map(|value| {
                short
                    .then(|| value.show_short(&formats))
                    .flatten()
                    .unwrap_or_else(|| value.show(&formats))
            })
            .unwrap_or_default()
    });
    // The picker's own text input always shows the whole text.
    let full_text =
        draft().unwrap_or_else(|| value.map(|value| value.show(&formats)).unwrap_or_default());
    // Says a refusal on Enter, where focus stays and the error line is not live.
    let announcer = use_announcer();
    // Returns the error when the text was refused.
    let commit = {
        let emit = emit.clone();
        let keep = keep.clone();
        move || {
            let text = draft.peek().clone()?;
            if text.trim().is_empty() {
                draft.set(None);
                keep(None);
                emit(None);
                return None;
            }
            let read = V::read(&text, &formats, value, today).map_err(|_| {
                Rc::new(|locale: &Localization| V::unreadable(&locale.date).to_string())
                    as Rc<dyn FieldError>
            });
            let refused = |error: String| Rc::new(error) as Rc<dyn FieldError>;
            match read.and_then(|next| accepts(next).map(|()| next).map_err(refused)) {
                Ok(next) => {
                    draft.set(None);
                    keep(None);
                    emit(Some(next));
                    None
                }
                Err(error) => {
                    let said = word(&*error);
                    rejected.set(Some(error.clone()));
                    keep(Some(Typed { text, error }));
                    if resets_value && value.is_some() {
                        emit(None);
                    }
                    Some(said)
                }
            }
        }
    };
    let (commit_on_blur, mut commit_on_enter) = (commit.clone(), commit.clone());
    let (mut commit_on_enter_picker, mut commit_in_picker) = (commit.clone(), commit.clone());
    let mut commit_on_picker_blur = commit;

    // Refused text outranks the caller's status: the field cannot hold what it shows.
    let status: Input<FieldStatus> = match rejected() {
        Some(error) => Input::Value(FieldStatus::Error(word(&*error))),
        None => field.status.clone(),
    };
    let parse = match (rejected(), value.is_some_and(|value| value.is_complete())) {
        (Some(error), _) => Parse::Invalid(error),
        (None, true) => Parse::Valid,
        (None, false) => Parse::Empty,
    };
    let field_box = use_field()
        .label(field.label)
        .description(field.description)
        .helper(field.helper)
        .status(&status)
        .rules(
            field
                .validate
                .check(&value.filter(|value| value.is_complete())),
        )
        .bound(&bound)
        .parse(parse)
        .required(required)
        .empty(!value.is_some_and(|value| value.is_complete()))
        .readonly(readonly)
        .disabled(disabled)
        .size(size)
        .radius(radius.clone())
        .class(field.class)
        .sx(field.sx)
        .parts(field.parts)
        .states(field.states)
        .attributes(&field.attributes)
        .prepare();

    let frame = use_field_frame()
        .leading(&field.leading)
        .trailing(&field.trailing)
        .states(field_box.states())
        .placeholder(field.placeholder.as_deref())
        .prepare();
    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    // Hooks: all run before anything branches on `opened`.
    let showing = opened() && !disabled && !readonly;
    let popover = use_popover_on(
        anchor,
        use_element(),
        showing,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding),
    );
    let floating = *popover.floating();
    let room = use_sheet_room(floating, narrow() && popover.placed());
    let clearance = use_sheet_clearance(
        anchor,
        field_box.root(),
        popover,
        showing,
        theme.popover.gap,
    );
    let close = use_callback(move |()| open_to(false));
    let SheetSwipe {
        strip,
        listeners,
        style: drag,
    } = use_sheet_swipe(floating, close);
    let dropdown_states: Input<States> = States::new().active("bordered").into();
    let dropdown_box = use_box()
        .framework_sx(&PICKER_FIELD_DROPDOWN_SX)
        .parts(field.dropdown_parts)
        .states(&dropdown_states)
        .style(Some(format!("{}{room}{drag}", popover.vars())))
        .prepare();
    let keyboardless = use_keyboardless();
    let picker_focus = use_picker_focus(anchor, floating, field_box.id());
    let dialog_id = format!("{}-dialog", field_box.id());
    let entry_id = dialog_id.clone();
    let tab_id = dialog_id.clone();
    // Waits for placement: the picker is not drawn until the box is measured.
    use_effect(move || {
        if !entering() || !popover.placed() {
            return;
        }
        entering.set(false);
        picker_focus.enter(&entry_id, DROPDOWN_ENTRY);
    });

    // After the next task focus has landed: closes unless it is in the input or dropdown.
    let press = keyboardless.press();
    let settle = move || {
        // A tap into the picker's text box stays, the blur's `focusin` aside (3905).
        if press.taken() {
            return;
        }
        spawn(async move {
            if !picker_focus.settled_inside().await {
                open_to(false);
            }
        });
    };
    let focus_input = move || picker_focus.focus_input();
    // On the Escape stack exactly while the handler below takes Escape, so a `HoverCard` leaves it.
    use_field_list_layer(
        opened() && !readonly,
        use_callback(move |()| {
            if picker_focus.in_dropdown() {
                focus_input();
            }
            open_to(false);
        }),
    );
    // Element 0 is the text input, 1 the dropdown; closes once focus is in neither.
    // Focus alone does not open (WCAG 3.2.1): Arrow Down or a click does.
    let focus = use_focus_within(
        move || vec![anchor.mounted(), floating.mounted()],
        move |change| {
            picker_focus.track(change.element, change.within);
            if change.element == 0 && *focused.peek() != change.within {
                let mut focused = focused;
                focused.set(change.within);
            }
            if let (0, false) = (change.element, change.within) {
                commit_on_blur.clone()();
            }
            match change.in_group {
                Some(true) => {}
                Some(false) => open_to(false),
                None => settle(),
            }
        },
    );

    let input = field_box
        .aria(control)
        .attr_default("type", "text")
        .attr("value", text)
        .attr("data-controlled", true)
        .attr("placeholder", field.placeholder)
        .attr("disabled", disabled)
        .attr("readonly", readonly)
        .attr("required", required)
        .attr("autocomplete", "off")
        // A phone's keyboard would cover the picker, which is the input there (todos 1012, 3905).
        .attr("inputmode", keyboardless.inputmode())
        // APG Date Picker Combobox: a textbox may not carry `aria-expanded`.
        .attr("role", "combobox")
        .attr("aria-haspopup", "dialog")
        .attr("aria-expanded", showing.to_string())
        .attr("aria-controls", showing.then(|| dialog_id.clone()))
        .event("oninput", move |event: FormEvent| {
            rejected.set(None);
            announcer.clear();
            draft.set(Some(event.value()));
        })
        .event("onfocus", focus.focusin(0))
        .event("onclick", move |_: MouseEvent| {
            if !readonly {
                open_to(true);
            }
        })
        .event("onblur", focus.focusout(0))
        .event("onkeydown", move |event: KeyboardEvent| match event.key() {
            // A composing Enter picks an IME candidate, it does not commit the draft.
            _ if readonly || event.is_composing() => {}
            Key::Named(NamedKey::Enter) => {
                if let Some(error) = commit_on_enter() {
                    announcer.say(error);
                }
            }
            // APG: Alt+ArrowDown enters like ArrowDown; Ctrl/Meta is the caret's.
            Key::Named(NamedKey::ArrowDown)
                if navigation_chord(&event) != Some(NavigationChord::Browser) =>
            {
                event.prevent_default();
                // The picker then enters on the typed day, not the one it showed.
                commit_on_enter_picker();
                open_to(true);
                entering.set(true);
            }
            Key::Named(NamedKey::Escape) if opened() => {
                event.prevent_default();
                open_to(false);
            }
            _ => {}
        })
        .render(HtmlTag::Input, field.attributes, ());

    // The text shows a format; the form gets ISO 8601.
    let hidden = bound.name().map(|name| {
        rsx! {
            input {
                r#type: "hidden",
                name: name.to_string(),
                // A disabled field posts nothing (todo 3873).
                disabled,
                value: value
                    .filter(|value| value.is_complete())
                    .map(Iso::iso)
                    .unwrap_or_default(),
            }
        }
    });

    // Portaled past `overflow: hidden`. Mousedown is cancelled so a click keeps focus on the input;
    // the keyboard enters with Arrow Down and leaves with Escape.
    // Stable across renders, so the picker memoizes while the field redraws.
    let pick = use_callback(move |next: Option<V>| {
        draft.set(None);
        rejected.set(None);
        keep(None);
        emit(next);
        if closes(next) {
            // The focused cell is about to go; focus goes back first.
            if picker_focus.in_dropdown() {
                focus_input();
            }
            open_to(false);
        }
    });
    // Closes as Escape does, the picks already committed (3903).
    let save = saves.then(|| {
        rsx! {
            div { "data-slot": "save",
                Button {
                    full_width: true,
                    onclick: move |_| {
                        if picker_focus.in_dropdown() {
                            focus_input();
                        }
                        open_to(false);
                    },
                    {names.save}
                }
            }
        }
    });
    // Shares the field's draft: Enter commits and hands focus back, a blur commits (3905).
    let picker_input = keyboardless.input(PickerInput {
        field_id: field_box.id().to_string(),
        labelled: !field.label.is_none(),
        fallback_label: V::dialog_label(names),
        value: full_text,
        invalid: rejected().is_some(),
        oninput: move |event: FormEvent| {
            rejected.set(None);
            announcer.clear();
            draft.set(Some(event.value()));
        },
        onkeydown: move |event: KeyboardEvent| {
            if event.key() != Key::Named(NamedKey::Enter) || event.is_composing() {
                return;
            }
            match commit_in_picker() {
                Some(error) => announcer.say(error),
                None => {
                    focus_input();
                    open_to(false);
                }
            }
        },
        onblur: move |_: FocusEvent| {
            commit_on_picker_blur();
        },
    });
    popover.show(showing.then(|| {
        dropdown_box
            .element(popover.floating())
            .attr("id", dialog_id)
            .attr("data-slot", ChronoDropdownPart::Panel.slot())
            .attr("role", "dialog")
            .attr("aria-label", V::dialog_label(names))
            .event("onmousedown", move |event: MouseEvent| {
                event.prevent_default()
            })
            .event("onfocusin", focus.focusin(1))
            .event("onfocusout", focus.focusout(1))
            .event("onkeydown", move |event: KeyboardEvent| match event.key() {
                Key::Named(NamedKey::Escape) => {
                    event.prevent_default();
                    focus_input();
                    open_to(false);
                }
                _ if is_tab(&event) => picker_focus.tab(&tab_id, &event),
                _ => {}
            })
            .render(
                HtmlTag::Div,
                listeners,
                rsx! {
                    {strip}
                    {dropdown(DropdownArgs {
                        value,
                        today,
                        size,
                        pick,
                    })}
                    {save}
                    {picker_input}
                },
            )
    }));

    let control = rsx! {
        div { onmounted: anchor.mount(), style: sheet_clearance_style(clearance),
            {frame.render(input)}
            {hidden}
            {announcer.render()}
        }
    };
    field_box.render(control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// The dropdown is a `Paper` surface with two overrides. Its `bordered` token needs a browser.
    #[test]
    fn the_dropdown_starts_from_the_paper_surface() {
        let css = Stylesheet::from(&*PICKER_FIELD_DROPDOWN_SX);
        let css = css.as_str();

        assert!(
            css.contains("background:var(--lsx-paper-background);"),
            "{css}"
        );
        assert!(
            css.contains("--lsx-focus-contrast:var(--lsx-paper-contrast);"),
            "{css}"
        );
        assert!(
            css.contains("border:1px solid var(--lsx-paper-border-color);"),
            "{css}"
        );
        // The overrides replace the surface defaults rather than race them.
        assert!(css.contains("border-radius:var(--lsx-radius-sm);"), "{css}");
        assert!(css.contains("box-shadow:var(--lsx-shadow-lg);"), "{css}");
        assert!(!css.contains("var(--lsx-paper-radius)"), "{css}");
        assert!(!css.contains("var(--lsx-paper-shadow)"), "{css}");
    }

    /// Below `sm` the dropdown is the phone sheet, and a clock face fills it (3900, 3926).
    #[test]
    fn the_dropdown_becomes_the_phone_sheet_below_sm() {
        let css = Stylesheet::from(&*PICKER_FIELD_DROPDOWN_SX);
        let css = css.as_str();
        let narrow = css
            .find(&format!("@media {}", narrow_query()))
            .unwrap_or_else(|| panic!("no narrow block: {css}"));
        assert!(
            css[narrow..].contains("bottom:var(--lsx-sheet-foot, 0px)"),
            "{css}"
        );
        assert!(css[narrow..].contains("div[data-slot='face']"), "{css}");
    }
}
