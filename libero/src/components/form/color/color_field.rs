use dioxus::prelude::*;

use super::{ColorCode, ColorFormat, ColorPicker, ColorSwatch, Swatches};
use crate::{
    components::{
        ActionIcon, HtmlTag, Input, States,
        common::field_props,
        form::{
            FIELD_CONTROL_SX, SliderChangeEvent, glyphs::EyeDropperIcon, use_bound, use_field,
            use_field_frame,
        },
        layout::use_box,
        surface::paper_sx,
    },
    hooks::{PopoverOptions, use_element, use_field_list_layer, use_popover, use_theme},
    platform::eye_dropper,
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
/// `ColorPicker` in a dropdown - Mantine's `ColorInput`.
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

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
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

    let icon_size: Input<ThemeAwareValue> = ThemeAwareValue::Size(size).into();
    let trailing = (with_eye_dropper && has_eye_dropper() && !disabled && !readonly).then(|| {
        rsx! {
            ActionIcon {
                aria_label: "Pick a color from the screen",
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
    use_field_list_layer(opened());
    let popover = use_popover(
        anchor,
        showing,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding),
    );
    let dropdown_states: Input<States> = States::new().active("bordered").into();
    let dropdown = use_box()
        .framework_sx(&COLOR_FIELD_DROPDOWN_SX)
        .states(&dropdown_states)
        .style(popover.style())
        .prepare();

    let text = draft().unwrap_or_else(|| value.to_format(format));
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
        .attr("aria-haspopup", has_dropdown.then_some("dialog"))
        .attr("aria-expanded", has_dropdown.then(|| showing.to_string()))
        .event("oninput", move |event: FormEvent| {
            let text = event.value();
            if let Ok(color) = text.parse::<ColorCode>() {
                emit(color);
            }
            draft.set(Some(text));
        })
        .event("onfocus", move |_: FocusEvent| opened.set(true))
        .event("onclick", move |_: MouseEvent| opened.set(true))
        .event("onblur", move |_: FocusEvent| {
            opened.set(false);
            // Text that parsed was already emitted, so it goes back to the
            // value's own spelling; text that did not stays only if asked.
            let parses = draft
                .peek()
                .as_ref()
                .is_some_and(|text| text.parse::<ColorCode>().is_ok());
            if fix_on_blur || parses {
                draft.set(None);
            }
        })
        .event("onkeydown", move |event: KeyboardEvent| {
            if event.key() == Key::Escape && opened() {
                event.prevent_default();
                opened.set(false);
            }
        })
        .render(HtmlTag::Input, props.attributes, ());

    // Portaled, so no `overflow: hidden` ancestor clips it. The picker inside
    // is not focusable, and a mousedown anywhere in the box is cancelled: the
    // text input keeps focus throughout, and its blur is what closes the box.
    let picker_setter = bound.setter();
    popover.show(showing.then(|| {
        dropdown
            .element(popover.floating())
            .event("onmousedown", move |event: MouseEvent| {
                event.prevent_default()
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
                        focusable: false,
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
                                opened.set(false);
                            }
                        },
                    }
                },
            )
    }));

    let control = rsx! {
        div { onmounted: anchor.mount(), {frame.render(input)} }
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
