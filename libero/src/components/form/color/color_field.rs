use dioxus::prelude::*;

use super::{ColorCode, ColorFormat, ColorPicker, ColorSwatch, Swatches};
use crate::{
    components::{
        ActionIcon, FOCUSABLE_SELECTOR, HtmlTag, Input, States,
        common::{EyeDropperIcon, NavigationChord, field_props, navigation_chord},
        form::{
            FIELD_CONTROL_SX, FieldStatus, SliderChangeEvent, slot_icon_size, use_announcer,
            use_bound, use_field, use_field_frame,
        },
        layout::{paper_sx, use_box},
    },
    hooks::{
        PopoverOptions, use_element, use_field_list_layer, use_focus_within, use_localization,
        use_popover_on, use_theme,
    },
    platform::{ElementApi, eye_dropper, next_task},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{Size, SizeCss, Z_INDEX_POPOVER},
};

static COLOR_FIELD_DROPDOWN_SX: StaticSx = StaticSx::new(|| {
    // A surface, so the background and the `bordered` border are
    // `paper_sx()`'s. Everything positional comes from `use_popover` as an
    // inline style.
    paper_sx()
        .z_index(Z_INDEX_POPOVER.value())
        .padding(SizeCss::SPACING.value(Size::Sm))
        // The field's own corner rather than the surface default, and a
        // dropdown floats over the page, where that default rests.
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .box_shadow(SizeCss::SHADOW.value(Size::Lg))
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
        /// Strictly controlled - pair it with `oninput`. Inside a `Form`, a
        /// path `name` can supply it instead.
        #[props(default)]
        value: ColorCode,
        /// A drag in the dropdown brackets its moves with `Start`/`End`.
        /// Everything else settles at once and emits `Change` then `End`:
        /// a key press or a swatch in the dropdown, typed text each time it
        /// parses, and the eyedropper.
        #[props(default)]
        oninput: Option<EventHandler<SliderChangeEvent<ColorCode>>>,
        /// Rules over the color, shown once the field loses focus or its form
        /// is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<ColorCode>,
        /// How the text shows the color, and so what `name` posts. Hex by
        /// default, hexa `with_alpha`. Typing accepts every form either way.
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
        /// `false` leaves only the swatches in the dropdown, and no dropdown
        /// at all without them.
        #[props(default)]
        with_picker: Option<bool>,
        /// The swatch in the leading slot.
        #[props(default)]
        with_preview: Option<bool>,
        /// The eyedropper button in the trailing slot, where the platform has
        /// one - Chromium today.
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
        /// What the field posts as. A path - `Theme::FIELDS.accent()` - also
        /// binds it to the surrounding `Form`'s value when it has no `oninput`.
        #[props(default, into)]
        name: crate::components::FieldName<ColorCode>,
        #[props(default, into)]
        placeholder: Option<String>,
    }
}

/// A text field holding a color, with a preview swatch, an eyedropper and a
/// `ColorPicker` in a dropdown.
///
/// Controlled: it renders `value` and asks for a new one through `oninput`.
/// Typed text is kept as typed until it blurs; each time it parses, the color
/// is emitted.
#[component]
pub fn ColorField(props: ColorFieldProps) -> Element {
    let theme = use_theme();
    let defaults = &theme.color_field;
    let size = props.size.copied_or(defaults.size);
    let radius = props.radius.copied_or(defaults.radius);
    let required = props.required.unwrap_or(false);
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
    let bound = use_bound(&props.name, props.oninput.is_some());
    let disabled = bound.disabled(props.disabled);
    // Like the other dropdown fields: the text takes the native `readonly`,
    // the dropdown refuses to open and the eyedropper stands down.
    let readonly = props.readonly.unwrap_or(false);
    let has_dropdown = (with_picker || !props.swatches.is_empty()) && !disabled;

    let mut opened = use_signal(|| false);
    // The text as typed, while it differs from `value`'s own spelling. `None`
    // shows `value` in `format`.
    let mut draft = use_signal(|| Option::<String>::None);
    // Focus is coming back to the text input from the dropdown, which closed:
    // that focus must not open it again.
    let mut returning = use_signal(|| false);
    // Arrow Down asked for focus in the picker, once the dropdown is drawn.
    let mut entering = use_signal(|| false);
    // Asked after mount: a server render has no eyedropper, and a client that
    // answered otherwise while hydrating would not match it.
    let mut has_eye_dropper = use_signal(|| false);
    use_effect(move || has_eye_dropper.set(eye_dropper().is_some()));

    let value = bound.value().unwrap_or(props.value);
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
    let dropper_emit = emit.clone();

    let labels = use_localization().color;
    // Typed text that is no color, found on Enter or on a blur that keeps it;
    // cleared by the next keystroke, as `DateField` does.
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
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
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
                            emit(color.with_alpha(preview.alpha()));
                        }
                    });
                },
                EyeDropperIcon {}
            }
        }
    });

    let frame = use_field_frame()
        .leading(&leading)
        .trailing(&trailing)
        .states(field.states())
        .prepare();

    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    // Every one of these is a hook, so all of them run before anything
    // branches on `opened`.
    let anchor = use_element();
    let showing = opened() && has_dropdown && !readonly;
    // On the Escape stack exactly while the key handler below would take
    // Escape, so a `HoverCard` around this field leaves the press to it.
    use_field_list_layer(showing);
    let popover = use_popover_on(
        anchor,
        use_element(),
        showing,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding),
    );
    let floating = *popover.floating();
    let dropdown_states: Input<States> = States::new().active("bordered").into();
    let dropdown = use_box()
        .framework_sx(&COLOR_FIELD_DROPDOWN_SX)
        .states(&dropdown_states)
        .style(popover.style())
        .prepare();
    // Waits for placement too: Arrow Down on a closed field opens it, and the
    // picker is not drawn until the box has been measured.
    use_effect(move || {
        if !entering() || !popover.placed() {
            return;
        }
        entering.set(false);
        let _ = floating
            .query_selector(DROPDOWN_ENTRY)
            .and_then(|element| element.focus());
    });

    // After the platform's next task focus has landed, so this can tell
    // whether it stayed in the field - the text input or the dropdown - or
    // left. Without a platform answer it counts as left.
    let settle = move || {
        spawn(async move {
            next_task().await;
            let inside = anchor.query_selector(":focus").is_ok()
                || floating.query_selector(":focus").is_ok();
            if !inside {
                opened.set(false);
            }
        });
    };
    let mut focus_input = move || {
        returning.set(true);
        if anchor
            .query_selector("input[data-controlled]")
            .and_then(|input| input.focus())
            .is_err()
        {
            returning.set(false);
        }
    };
    let focused = move || match returning() {
        true => returning.set(false),
        false => opened.set(true),
    };
    // Text that parsed was already emitted, so it goes back to the value's own
    // spelling; text that did not stays only if asked.
    let unparsable = move || {
        draft
            .peek()
            .as_ref()
            .is_some_and(|text| text.parse::<ColorCode>().is_err())
    };
    let blurred = move || {
        if fix_on_blur || !unparsable() {
            draft.set(None);
        } else if !*rejected.peek() {
            // Said once, so a blur after Enter does not repeat it.
            rejected.set(true);
            announcer.say(labels.invalid.to_string());
        }
    };
    // Element 0 is the text input, 1 the dropdown; the box closes once focus
    // is in neither. Only the dropdown's `focusout` waits to see where it went.
    let focus = use_focus_within(
        move || vec![anchor.mounted(), floating.mounted()],
        move |change| {
            let mut opened = opened;
            match (change.element, change.within) {
                (0, true) => {
                    let mut focused = focused;
                    focused();
                }
                (0, false) => {
                    let mut blurred = blurred;
                    blurred();
                }
                _ => {}
            }
            match change.in_group {
                Some(false) => opened.set(false),
                None if change.element == 1 => settle(),
                _ => {}
            }
        },
    );

    let text = draft().unwrap_or_else(|| value.to_format(format));
    let dialog_id = format!("{}-dialog", field.id());
    let input = field
        .aria(control)
        .attr_default("type", "text")
        .attr("name", bound.name().map(str::to_string))
        .attr("value", text)
        .attr("data-controlled", true)
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("required", required)
        .attr("readonly", disallow_input || readonly)
        .attr("autocomplete", "off")
        .attr("spellcheck", "false")
        // APG Date Picker Combobox: a textbox may not carry `aria-expanded`.
        .attr("role", has_dropdown.then_some("combobox"))
        .attr("aria-haspopup", has_dropdown.then_some("dialog"))
        .attr("aria-expanded", has_dropdown.then(|| showing.to_string()))
        .attr("aria-controls", showing.then(|| dialog_id.clone()))
        .event("oninput", move |event: FormEvent| {
            let text = event.value();
            if let Ok(color) = text.parse::<ColorCode>() {
                emit(color);
            }
            draft.set(Some(text));
            if *rejected.peek() {
                rejected.set(false);
            }
            announcer.clear();
        })
        .event("onfocus", focus.focusin(0))
        .event("onclick", move |_: MouseEvent| opened.set(true))
        .event("onblur", focus.focusout(0))
        .event("onkeydown", move |event: KeyboardEvent| match event.key() {
            // Focus stays, and the error line is no live region.
            Key::Enter if unparsable() => {
                rejected.set(true);
                announcer.say(labels.invalid.to_string());
            }
            // APG: Alt+ArrowDown enters like ArrowDown; Ctrl/Meta is the caret's.
            Key::ArrowDown
                if has_dropdown
                    && !readonly
                    && navigation_chord(&event) != Some(NavigationChord::Browser) =>
            {
                event.prevent_default();
                opened.set(true);
                entering.set(true);
            }
            Key::Escape if showing => {
                event.prevent_default();
                opened.set(false);
            }
            _ => {}
        })
        .render(HtmlTag::Input, props.attributes, ());

    // Portaled, so no `overflow: hidden` ancestor clips it. A mousedown in the
    // box is cancelled, so a click keeps focus on the text input; the keyboard
    // enters the picker with Arrow Down and leaves it with Escape or Tab. The box
    // closes once focus is in neither.
    let picker_setter = bound.setter();
    popover.show(showing.then(|| {
        dropdown
            .element(popover.floating())
            .attr("id", dialog_id)
            .attr("role", "dialog")
            .attr("aria-label", labels.choose)
            .event("onmousedown", move |event: MouseEvent| {
                event.prevent_default()
            })
            .event("onfocusout", focus.focusout(1))
            .event("onkeydown", move |event: KeyboardEvent| match event.key() {
                Key::Escape => {
                    event.prevent_default();
                    focus_input();
                    opened.set(false);
                }
                // Portaled after the page: Tab past either end goes back
                // through the text input, then on as from the field.
                Key::Tab => {
                    let Ok(stops) = floating.query_selector_all(FOCUSABLE_SELECTOR) else {
                        return;
                    };
                    let backwards = event.modifiers().shift();
                    let edge = if backwards {
                        stops.first()
                    } else {
                        stops.last()
                    };
                    if edge.is_some_and(|stop| stop.is_focused()) {
                        if backwards {
                            event.prevent_default();
                        }
                        focus_input();
                    }
                }
                _ => {}
            })
            .render(
                HtmlTag::Div,
                Vec::new(),
                rsx! {
                    ColorPicker {
                        value,
                        format: Input::Value(format),
                        with_alpha,
                        with_picker,
                        swatches: props.swatches.clone(),
                        swatches_per_row: props.swatches_per_row,
                        size,
                        oninput: move |event: SliderChangeEvent<ColorCode>| {
                            draft.set(None);
                            match (&oninput, &picker_setter) {
                                (Some(oninput), _) => oninput.call(event),
                                (None, Some(setter)) => {
                                    if let SliderChangeEvent::Change(color) = event {
                                        setter.set(color);
                                    }
                                }
                                (None, None) => {}
                            }
                        },
                        onswatchclick: move |_| {
                            if close_on_swatch_click {
                                // The focused swatch is about to go; focus goes back first.
                                if floating.query_selector(":focus").is_ok() {
                                    focus_input();
                                }
                                opened.set(false);
                            }
                        },
                    }
                },
            )
    }));

    let control = rsx! {
        // On the wrapper, not the input: focus can also leave from the eyedropper.
        div { onmounted: anchor.mount(), onfocusout: move |_| settle(),
            {frame.render(input)}
            {announcer.render()}
        }
    };
    field.render(control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// The dropdown is a `Paper` surface with two overrides. The `bordered`
    /// token it is rendered with only exists in a browser - the box opens on
    /// an event.
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
}
