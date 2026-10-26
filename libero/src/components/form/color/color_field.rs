use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{ColorCode, ColorFormat, ColorPicker, ColorSwatch, Swatches, color_code::names_alpha};
use crate::{
    components::{
        accessibility::use_announcer,
        buttons::ActionIcon,
        common::{
            Glyph, HtmlTag, Input, NavigationChord, Part, Parts, ScaleOrCss, States,
            navigation_chord,
        },
        form::{
            ColorDropdownPart, FIELD_CONTROL_SX, FieldStatus, FormScope, SLOT_BUTTON_SX,
            SliderChangeEvent, field_props,
            keyboardless::{PickerInput, use_keyboardless, use_picker_focus},
            phone_sheet::{
                SheetSwipe, narrow_query, phone_sheet_sx, sheet_clearance_style,
                use_sheet_clearance, use_sheet_room, use_sheet_swipe,
            },
            slot_icon_size, use_bound, use_field, use_field_frame, use_prefill,
        },
        layout::{paper_sx, use_box},
    },
    context::IconSlot,
    hooks::{
        POPOVER_AVAILABLE_HEIGHT, PopoverOptions, placement_sx, use_element, use_field_list_layer,
        use_focus_within, use_form_owner, use_localization, use_media_query, use_popover_on,
        use_theme,
    },
    platform::{eye_dropper, is_tab},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        COLOR_PICKER_SPACING, FIELD_LABEL_FONT_SIZE, SLIDER_THUMB, SLIDER_TRACK, Size, SizeCss,
        Z_INDEX_POPOVER,
    },
};

static COLOR_FIELD_DROPDOWN_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        .and(placement_sx(true))
        .z_index(Z_INDEX_POPOVER.value())
        .padding(SizeCss::SPACING.value(Size::Sm))
        // The field's corner, and a shadow: the dropdown floats over the page.
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .box_shadow(SizeCss::SHADOW.value(Size::Lg))
        // Never past the room on its side: it scrolls instead (WCAG 1.4.10).
        .max_height(POPOVER_AVAILABLE_HEIGHT.value_or("none"))
        .overflow_y("auto")
        .selector(
            "& > [data-slot='text-input']",
            sx().margin_top(SizeCss::SPACING.value(Size::Xs)),
        )
        .selector(
            "& > [data-slot='recent-label']",
            sx().margin_top(SizeCss::SPACING.value(Size::Sm)),
        )
        .selector(
            "& > [data-recent]",
            sx().margin_top(SizeCss::SPACING.value(Size::Xs)),
        )
        // Below `sm` the phone sheet, its picker spread over the width (todos 3900, 3923).
        .and(phone_sheet_sx())
        .media(
            narrow_query(),
            sx().selector(
                "& [data-slot='saturation']",
                sx().height("clamp(160px, 50vw, 30dvh)"),
            )
            // 44px tracks need no overhang, so no wider pitch; `div` outranks the picker's rule.
            .selector(
                "& [data-slot='body'] > div[data-slot='sliders']",
                sx().gap(COLOR_PICKER_SPACING.value()).padding_block("0"),
            )
            // On the track, below the slider root whose `sx` layer sets them.
            .selector(
                "& [data-slot='sliders'] [data-slot='track']",
                sx().var(SLIDER_TRACK, "44px").var(SLIDER_THUMB, "44px"),
            )
            // Finger-sized presets that wrap, so none is cut at 360px (4206); `div` outranks the picker's rule.
            .selector(
                "& div > [data-slot='swatches']",
                sx().max_width("none").padding_block("2px"),
            )
            .selector(
                "& [data-slot='swatches'] > [data-slot='swatch']",
                sx().width("44px").height("44px").min_width("44px"),
            ),
        )
});

/// Where Arrow Down in the text input puts focus: the saturation area, or the
/// first swatch when the dropdown holds only swatches.
const DROPDOWN_ENTRY: &str = "[tabindex='0'], button:not([tabindex='-1'])";

/// The leading preview, sized to the text rather than to a size step.
static COLOR_FIELD_PREVIEW_SX: StaticSx =
    StaticSx::new(|| sx().width("1.25em").height("1.25em").min_width("1.25em"));

field_props! {
    extends(input);
    pub struct ColorFieldProps {
        /// Controlled: pair it with `oninput`, or bind a path `name` in a `Form`.
        #[props(default)]
        value: ColorCode,
        /// A drag brackets its moves with `Start`/`End`; typed text sends `Change`
        /// per parse and `End` on Enter or blur; anything else `Change` then `End`.
        #[props(default)]
        oninput: Option<EventHandler<SliderChangeEvent<ColorCode>>>,
        /// Rules over the color, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<ColorCode>,
        /// How the text shows the color after a blur. Typing accepts every form.
        #[props(default, into)]
        format: Input<ColorFormat>,
        /// Shows the alpha slider in the dropdown, and keeps typed alpha.
        #[props(default)]
        with_alpha: Option<bool>,
        /// Preset colors in the dropdown. Takes `ColorCode`s or CSS strings.
        #[props(default, into)]
        swatches: Swatches,
        /// Caps how many swatches share a row in the dropdown.
        #[props(default)]
        swatches_per_row: Option<usize>,
        /// Keeps up to this many recently picked colors, newest first, in a row under
        /// the swatches. In memory, lost on unmount: persist them through `swatches`.
        ///
        /// ```no_run
        /// # use dioxus::prelude::*;
        /// # use libero::components::{ColorCode, ColorField};
        /// # fn app() -> Element {
        /// # let color = use_signal(ColorCode::default);
        /// # rsx! {
        /// ColorField { label: "Accent", value: color(), recent_colors: 6 }
        /// # } }
        /// ```
        #[props(default)]
        recent_colors: Option<usize>,
        /// `false` leaves only the swatches, and no dropdown without them.
        #[props(default)]
        with_picker: Option<bool>,
        /// The swatch in the leading slot.
        #[props(default)]
        with_preview: Option<bool>,
        /// The eyedropper button, where the platform has one (Chromium today).
        #[props(default)]
        with_eye_dropper: Option<bool>,
        /// The text is read-only: a color comes from the dropdown alone.
        #[props(default)]
        disallow_input: Option<bool>,
        /// Text that does not parse goes back to the last valid color on blur.
        #[props(default)]
        fix_on_blur: Option<bool>,
        /// Picking a swatch closes the dropdown.
        #[props(default)]
        close_on_swatch_click: Option<bool>,
        /// What the field posts its color as, in `format`, from a hidden input. A path
        /// also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<ColorCode>,
        #[props(default, into)]
        placeholder: Option<String>,
        /// Styles the portaled dropdown and the picker in it.
        #[props(default, into)]
        dropdown_parts: Input<Parts<ColorDropdownPart>>,
    }
}

/// A text field holding a color, with a preview swatch, an eyedropper and a
/// `ColorPicker` dropdown.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{ColorCode, ColorField, SliderChangeEvent};
/// # fn app() -> Element {
/// let mut color = use_signal(|| ColorCode::rgba(34, 139, 230, 1.0));
/// rsx! {
///     ColorField {
///         label: "Accent",
///         value: color(),
///         oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/color-field>
#[component]
pub fn ColorField(props: ColorFieldProps) -> Element {
    let theme = use_theme();
    let defaults = &theme.color_field;
    let size = props.size.copied_or(defaults.size);
    let radius = ScaleOrCss::new(props.radius.as_ref(), defaults.radius);
    let required = crate::components::form::fieldset_required(props.required);
    let with_alpha = props.with_alpha.unwrap_or(false);
    let with_preview = props.with_preview.unwrap_or(defaults.with_preview);
    let with_eye_dropper = props.with_eye_dropper.unwrap_or(defaults.with_eye_dropper);
    let disallow_input = props.disallow_input.unwrap_or(false);
    let fix_on_blur = props.fix_on_blur.unwrap_or(defaults.fix_on_blur);
    let close_on_swatch_click = props
        .close_on_swatch_click
        .unwrap_or(defaults.close_on_swatch_click);
    let format = props.format.copied_or(match with_alpha {
        true => ColorFormat::Hexa,
        false => ColorFormat::Hex,
    });
    let with_picker = props.with_picker.unwrap_or(true);
    let recent_cap = props.recent_colors.unwrap_or(0);
    let recents = use_signal(Vec::<ColorCode>::new);
    // Newest first, once each: a repeat moves to the front (todo 3951).
    let remember = move |color: ColorCode| {
        if recent_cap == 0 {
            return;
        }
        let color = match with_alpha {
            true => color,
            false => color.opaque(),
        };
        let mut recents = recents;
        let mut list = recents.write();
        list.retain(|kept| kept.to_rgba() != color.to_rgba());
        list.insert(0, color);
        list.truncate(recent_cap);
    };
    let bound = use_bound(&props.name, props.oninput.is_some());
    let disabled = bound.disabled(props.disabled);
    // Like the other dropdown fields: the text takes the native `readonly`,
    // the dropdown refuses to open and the eyedropper stands down.
    let readonly = props.readonly.unwrap_or(false);
    let has_dropdown = (with_picker || !props.swatches.is_empty() || recent_cap > 0) && !disabled;
    let has_palette = with_picker || !props.swatches.is_empty();

    let mut opened = use_signal(|| false);
    // The text as typed, while it differs from `value`'s own spelling. `None`
    // shows `value` in `format`.
    let mut draft = use_signal(|| Option::<String>::None);
    // Arrow Down asked for focus in the picker, once the dropdown is drawn.
    let mut entering = use_signal(|| false);
    // Asked after mount: a server render has no eyedropper, and a client that
    // answered otherwise while hydrating would not match it.
    let mut has_eye_dropper = use_signal(|| false);
    use_effect(move || has_eye_dropper.set(eye_dropper().is_some()));

    let value = bound.value().unwrap_or(props.value);
    let keep_alpha = with_alpha && !format.has_alpha();
    let typed = move |text: &str| typed_color(text, value, keep_alpha);
    let oninput = props.oninput;
    let setter = bound.setter();
    // Without alpha the field has no way to show or change it, so a typed or
    // dropped translucent color arrives opaque.
    let emit = move |color: ColorCode| {
        let color = match with_alpha {
            true => color,
            false => color.opaque(),
        };
        match (&oninput, &setter) {
            // Settled the moment it lands, so a caller that commits on `End`
            // hears it too - as a key press in the dropdown does (todo 291).
            (Some(oninput), _) => {
                oninput.call(SliderChangeEvent::Change(color));
                oninput.call(SliderChangeEvent::End(color));
            }
            (None, Some(setter)) => setter.set(color),
            (None, None) => {}
        }
    };
    let dropper_emit = emit;
    // Typed text sends `Change` per parse and `End` once, on Enter or blur:
    // `#fa5252` passes through `#fa5` and `#fa52` (todo 2296).
    let mut unsettled = use_signal(|| Option::<ColorCode>::None);
    let typed_setter = bound.setter();
    let type_in = move |color: ColorCode| {
        let color = match with_alpha {
            true => color,
            false => color.opaque(),
        };
        match (&oninput, &typed_setter) {
            (Some(oninput), _) => {
                oninput.call(SliderChangeEvent::Change(color));
                unsettled.set(Some(color));
            }
            (None, Some(setter)) => setter.set(color),
            (None, None) => {}
        }
    };
    let settle_typed = move || {
        if unsettled.peek().is_none() {
            return;
        }
        if let (Some(color), Some(oninput)) = (unsettled.take(), oninput) {
            oninput.call(SliderChangeEvent::End(color));
        }
    };

    let labels = use_localization().color;
    // Typed text that is no color, found on Enter or on a blur that keeps it;
    // cleared by the next keystroke, as `ChronoField` does.
    let mut rejected = use_signal(|| false);
    let announcer = use_announcer();
    // A color from the dropdown or the eyedropper drops the draft, and the error with it.
    let status: Input<FieldStatus> = match rejected() && draft.read().is_some() {
        true => Input::Value(FieldStatus::Error(labels.invalid.to_string())),
        false => props.status.clone(),
    };
    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&status)
        .rules(props.validate.check(&value))
        .bound(&bound)
        .required(required)
        // A color is always held: only text cleared to blank leaves the field empty.
        .empty(
            draft
                .read()
                .as_deref()
                .is_some_and(|text| text.trim().is_empty()),
        )
        .readonly(readonly)
        .disabled(disabled)
        .size(size)
        .radius(radius.clone())
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let preview = match with_alpha {
        true => value,
        false => value.opaque(),
    };
    let leading = with_preview.then(|| {
        rsx! {
            ColorSwatch { color: preview, sx: Input::Static(&*COLOR_FIELD_PREVIEW_SX) }
        }
    });

    let icon_size: Input<ThemeAwareValue> = ThemeAwareValue::Size(slot_icon_size(size)).into();
    let trailing = (with_eye_dropper && has_eye_dropper() && !disabled && !readonly).then(|| {
        rsx! {
            ActionIcon {
                aria_label: labels.eye_dropper,
                size: icon_size,
                sx: &SLOT_BUTTON_SX,
                onclick: move |_| {
                    let Some(api) = eye_dropper() else {
                        return;
                    };
                    let pick = api.pick();
                    let emit = dropper_emit.clone();
                    spawn(async move {
                        // A dismissed pick is no answer, not an error to show.
                        let Ok(hex) = pick.await else {
                            return;
                        };
                        if let Ok(color) = hex.parse::<ColorCode>() {
                            draft.set(None);
                            unsettled.set(None);
                            emit(color.with_alpha(preview.alpha()));
                            remember(color.with_alpha(preview.alpha()));
                        }
                    });
                },
                Glyph { slot: IconSlot::EyeDropper, icon: lucide::pipette::outlined }
            }
        }
    });

    let frame = use_field_frame()
        .leading(&leading)
        .trailing(&trailing)
        .states(field.states())
        .placeholder(props.placeholder.as_deref())
        .prepare();

    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    // Every one of these is a hook, so all of them run before anything
    // branches on `opened`.
    let anchor = use_element();
    // A form reset drops the typed text and its refusal, as in `ChronoField`.
    let scope = try_use_context::<FormScope>();
    let owner = use_form_owner(anchor, scope.is_none());
    let resets = scope.map_or(0, |form| form.resets()) + owner().unwrap_or(0);
    use_effect(use_reactive!(|resets| {
        let _ = resets;
        if draft.peek().is_some() {
            draft.set(None);
        }
        if *rejected.peek() {
            rejected.set(false);
        }
    }));
    let showing = opened() && has_dropdown && !readonly;
    // The color the dropdown opened on; a different one at its close is remembered.
    let mut opened_on = use_signal(|| Option::<ColorCode>::None);
    use_effect(use_reactive!(|showing, preview| match showing {
        true if opened_on.peek().is_none() => opened_on.set(Some(preview)),
        false if opened_on.peek().is_some() => {
            let start = opened_on.take();
            if start.is_some_and(|start| start.to_rgba() != preview.to_rgba()) {
                remember(preview);
            }
        }
        _ => {}
    }));
    let popover = use_popover_on(
        anchor,
        use_element(),
        showing,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding),
    );
    let floating = *popover.floating();
    let narrow = use_media_query(&narrow_query());
    let room = use_sheet_room(floating, narrow() && popover.placed());
    let clearance = use_sheet_clearance(anchor, field.root(), popover, showing, theme.popover.gap);
    let close = use_callback(move |()| opened.set(false));
    let SheetSwipe {
        strip,
        listeners,
        style: drag,
    } = use_sheet_swipe(floating, close);
    let dropdown_states: Input<States> = States::new().active("bordered").into();
    let dropdown = use_box()
        .framework_sx(&COLOR_FIELD_DROPDOWN_SX)
        .parts(&props.dropdown_parts)
        .states(&dropdown_states)
        .style(Some(format!("{}{room}{drag}", popover.vars())))
        .prepare();
    let keyboardless = use_keyboardless();
    let picker_focus = use_picker_focus(anchor, floating, field.id());
    let dialog_id = format!("{}-dialog", field.id());
    let entry_id = dialog_id.clone();
    let tab_id = dialog_id.clone();
    // Waits for placement too: Arrow Down on a closed field opens it, and the
    // picker is not drawn until the box has been measured.
    use_effect(move || {
        if !entering() || !popover.placed() {
            return;
        }
        entering.set(false);
        picker_focus.enter(&entry_id, DROPDOWN_ENTRY);
    });

    // Waits for focus to land, then closes unless it stayed in the input or the dropdown.
    let press = keyboardless.press();
    let settle = move || {
        // A tap into the picker's text box stays, the blur's `focusin` aside (3905).
        if press.taken() {
            return;
        }
        spawn(async move {
            if !picker_focus.settled_inside().await {
                opened.set(false);
            }
        });
    };
    let focus_input = move || picker_focus.focus_input();
    // On the Escape stack exactly while the key handler below would take
    // Escape, so a `HoverCard` around this field leaves the press to it.
    use_field_list_layer(
        showing,
        use_callback(move |()| {
            if picker_focus.in_dropdown() {
                focus_input();
            }
            opened.set(false);
        }),
    );
    // Text that parsed was already emitted, so it goes back to the value's own
    // spelling; text that did not stays only if asked. Blank text is empty, not invalid.
    let blank = move || {
        draft
            .peek()
            .as_deref()
            .is_some_and(|text| text.trim().is_empty())
    };
    let unparsable = move || {
        draft
            .peek()
            .as_ref()
            .is_some_and(|text| text.parse::<ColorCode>().is_err())
            && !blank()
    };
    // Text typed with the dropdown shut is remembered as it settles; an open one, at its close.
    let remember_typed = move || {
        if showing {
            return;
        }
        if let Some(color) = draft.peek().as_deref().and_then(typed) {
            remember(color);
        }
    };
    let blurred = move || {
        remember_typed();
        let mut settle_typed = settle_typed;
        settle_typed();
        let unparsable = unparsable();
        // Said once, so a blur after Enter does not repeat it; a revert says it too (todo 1548).
        if unparsable && !*rejected.peek() {
            rejected.set(true);
            announcer.say(labels.invalid.to_string());
        }
        if fix_on_blur || !unparsable && !blank() {
            draft.set(None);
        }
    };
    // Element 0 is the text input, 1 the dropdown; the box closes once focus
    // is in neither. Focus alone does not open it (WCAG 3.2.1): Arrow Down or a click does.
    let focus = use_focus_within(
        move || vec![anchor.mounted(), floating.mounted()],
        move |change| {
            picker_focus.track(change.element, change.within);
            let mut opened = opened;
            if let (0, false) = (change.element, change.within) {
                let mut blurred = blurred;
                blurred();
            }
            match change.in_group {
                Some(false) => opened.set(false),
                None if change.element == 1 => settle(),
                _ => {}
            }
        },
    );

    let text = draft().unwrap_or_else(|| value.to_format(format));
    let picker_text = text.clone();
    let prefill_type_in = type_in.clone();
    // Each parse is sent; the text stays as typed until the blur.
    let on_text = move |event: FormEvent| {
        let text = event.value();
        if let Some(color) = typed(&text) {
            let mut type_in = type_in.clone();
            type_in(color);
        }
        draft.set(Some(text));
        if *rejected.peek() {
            rejected.set(false);
        }
        announcer.clear();
    };
    // Typed and left, as a blur would settle it: no focus came first.
    let input_element = use_element();
    use_prefill(
        input_element,
        Some(text.clone()),
        Some(move |text: String| {
            if let Some(color) = typed(&text) {
                let mut type_in = prefill_type_in.clone();
                type_in(color);
            }
            let mut draft = draft;
            draft.set(Some(text));
            let mut blurred = blurred;
            blurred();
        }),
        disabled || readonly || disallow_input,
    );
    let input = field
        .aria(control)
        .element(&input_element)
        .attr_default("type", "text")
        .attr("value", text)
        .attr("data-controlled", true)
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("required", required)
        .attr("readonly", disallow_input || readonly)
        .attr("autocomplete", "off")
        .attr("spellcheck", "false")
        // A tap opens only the picker: a phone's keyboard would cover it (todo 3923).
        .attr(
            "inputmode",
            has_dropdown.then(|| keyboardless.inputmode()).flatten(),
        )
        // APG Date Picker Combobox: a textbox may not carry `aria-expanded`.
        .attr("role", has_dropdown.then_some("combobox"))
        .attr("aria-haspopup", has_dropdown.then_some("dialog"))
        .attr("aria-expanded", has_dropdown.then(|| showing.to_string()))
        .attr("aria-controls", showing.then(|| dialog_id.clone()))
        .event("oninput", on_text.clone())
        .event("onfocus", focus.focusin(0))
        .event("onclick", move |_: MouseEvent| opened.set(true))
        .event("onblur", focus.focusout(0))
        .event("onkeydown", move |event: KeyboardEvent| match event.key() {
            // A composing key picks an IME candidate, it is not a command.
            _ if event.is_composing() => {}
            // Focus stays, and the error line is no live region.
            Key::Named(NamedKey::Enter) if unparsable() => {
                rejected.set(true);
                announcer.say(labels.invalid.to_string());
            }
            Key::Named(NamedKey::Enter) => {
                remember_typed();
                let mut settle_typed = settle_typed;
                settle_typed();
            }
            // APG: Alt+ArrowDown enters like ArrowDown; Ctrl/Meta is the caret's.
            Key::Named(NamedKey::ArrowDown)
                if has_dropdown
                    && !readonly
                    && navigation_chord(&event) != Some(NavigationChord::Browser) =>
            {
                event.prevent_default();
                opened.set(true);
                entering.set(true);
            }
            Key::Named(NamedKey::Escape) if showing => {
                event.prevent_default();
                opened.set(false);
            }
            _ => {}
        })
        .render(HtmlTag::Input, props.attributes, ());

    // Portaled, so no `overflow: hidden` ancestor clips it. A cancelled
    // mousedown keeps focus on the text input.
    let picker_setter = bound.setter();
    // The hex line under the picker, where a finger types (3923): Enter settles and hands
    // focus back, a blur settles as the field's own does.
    let text_input = (!disallow_input)
        .then(|| {
            keyboardless.input(PickerInput {
                field_id: field.id().to_string(),
                labelled: !props.label.is_none(),
                fallback_label: labels.choose,
                value: picker_text,
                invalid: rejected() && draft.read().is_some(),
                oninput: on_text,
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() != Key::Named(NamedKey::Enter) || event.is_composing() {
                        return;
                    }
                    if unparsable() {
                        rejected.set(true);
                        announcer.say(labels.invalid.to_string());
                        return;
                    }
                    let mut settle_typed = settle_typed;
                    settle_typed();
                    focus_input();
                    opened.set(false);
                },
                onblur: move |_: FocusEvent| {
                    let mut blurred = blurred;
                    blurred();
                },
            })
        })
        .flatten();
    let picked = use_callback(move |event: SliderChangeEvent<ColorCode>| {
        draft.set(None);
        unsettled.set(None);
        match (&oninput, &picker_setter) {
            (Some(oninput), _) => oninput.call(event),
            (None, Some(setter)) => {
                if let SliderChangeEvent::Change(color) = event {
                    setter.set(color);
                }
            }
            (None, None) => {}
        }
    });
    let swatch_clicked = use_callback(move |_: ColorCode| {
        if close_on_swatch_click {
            // The focused swatch is about to go; focus goes back first.
            if picker_focus.in_dropdown() {
                focus_input();
            }
            opened.set(false);
        }
    });
    // A palette-only picker, so the row sizes and wraps as the presets do.
    let recent_row = (!recents.read().is_empty()).then(|| {
        rsx! {
            span {
                "data-slot": "recent-label",
                "aria-hidden": "true",
                style: "font-size: {FIELD_LABEL_FONT_SIZE.value(size)}",
                "{labels.recent}"
            }
            ColorPicker {
                value,
                with_alpha,
                with_picker: false,
                swatches: Swatches::from(recents()),
                swatches_per_row: props.swatches_per_row,
                full_width: narrow(),
                size,
                oninput: picked,
                onswatchclick: swatch_clicked,
                role: "group",
                aria_label: labels.recent,
                "data-recent": true,
            }
        }
    });
    popover.show(showing.then(|| {
        dropdown
            .element(popover.floating())
            .attr("id", dialog_id)
            .attr("data-slot", ColorDropdownPart::Panel.slot())
            .attr("role", "dialog")
            .attr("aria-label", labels.choose)
            .event("onmousedown", move |event: MouseEvent| {
                event.prevent_default()
            })
            .event("onfocusin", focus.focusin(1))
            .event("onfocusout", focus.focusout(1))
            .event("onkeydown", move |event: KeyboardEvent| match event.key() {
                Key::Named(NamedKey::Escape) => {
                    event.prevent_default();
                    focus_input();
                    opened.set(false);
                }
                _ if is_tab(&event) => picker_focus.tab(&tab_id, &event),
                _ => {}
            })
            .render(
                HtmlTag::Div,
                listeners,
                rsx! {
                    {strip}
                    if has_palette {
                        ColorPicker {
                            value,
                            format: Input::Value(format),
                            with_alpha,
                            with_picker,
                            swatches: props.swatches.clone(),
                            swatches_per_row: props.swatches_per_row,
                            full_width: narrow(),
                            size,
                            oninput: picked,
                            onswatchclick: swatch_clicked,
                        }
                    }
                    {recent_row}
                    {text_input}
                },
            )
    }));

    // Posts the value in `format`, as `ColorPicker` does, never the text being typed (todo 2315).
    let hidden = bound.name().map(|name| {
        rsx! {
            input {
                r#type: "hidden",
                name,
                value: value.to_format(format),
                disabled,
            }
        }
    });
    let control = rsx! {
        // On the wrapper, not the input: focus can also leave from the eyedropper.
        div {
            onmounted: anchor.mount(),
            onfocusout: move |_| settle(),
            style: sheet_clearance_style(clearance),
            {frame.render(input)}
            {hidden}
            {announcer.render()}
        }
    };
    field.render(control)
}

/// Typed text as a color. With `keep_alpha` (alpha on, an alpha-less `format`), text
/// that names no alpha keeps the current one, as the eyedropper does (todo 4241).
fn typed_color(text: &str, current: ColorCode, keep_alpha: bool) -> Option<ColorCode> {
    let color = text.parse::<ColorCode>().ok()?;
    match keep_alpha && !names_alpha(text) {
        true => Some(color.with_alpha(current.alpha())),
        false => Some(color),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// The dropdown is a `Paper` surface with two overrides. Its `bordered`
    /// token only shows in a browser, where the box opens.
    #[test]
    fn the_dropdown_starts_from_the_paper_surface() {
        let css = Stylesheet::from(&*COLOR_FIELD_DROPDOWN_SX);
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

    /// Todo 4247: recents alone are a dropdown to open, with or without a picker or swatches.
    #[test]
    fn recents_without_a_picker_or_swatches_still_open_a_dropdown() {
        #[component]
        fn Field(recent_colors: Option<usize>) -> Element {
            rsx! {
                ColorField {
                    label: "Accent",
                    value: ColorCode::default(),
                    oninput: move |_| {},
                    with_picker: false,
                    recent_colors,
                }
            }
        }
        let render = |recent_colors| {
            dioxus_ssr::render_element(rsx! {
                crate::LiberoProvider { Field { recent_colors } }
            })
        };
        assert!(render(Some(5)).contains(r#"role="combobox""#));
        assert!(!render(None).contains(r#"role="combobox""#));
    }

    #[test]
    fn alpha_less_text_keeps_the_current_alpha() {
        let current: ColorCode = "#228be680".parse().unwrap();
        let kept = typed_color("#228be7", current, true).unwrap();
        assert_eq!(kept.to_hexa(), "#228be780");
        let kept = typed_color("rgb(34, 139, 231)", current, true).unwrap();
        assert_eq!(kept.to_hexa(), "#228be780");
        // Text that spells alpha wins, and so does a format that shows it.
        let named = typed_color("#228be7ff", current, true).unwrap();
        assert_eq!(named.to_hexa(), "#228be7ff");
        let named = typed_color("hsl(208 80% 52% / 0.25)", current, true).unwrap();
        assert_eq!(named.alpha(), 0.25);
        let opaque = typed_color("#228be7", current, false).unwrap();
        assert_eq!(opaque.to_hexa(), "#228be7ff");
        assert!(typed_color("", current, true).is_none());
    }
}
