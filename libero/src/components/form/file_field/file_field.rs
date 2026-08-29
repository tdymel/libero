use dioxus::html::{FileData, HasFileData};
use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        ActionIcon, Chip, HtmlTag, Input, States,
        common::{field_props, focus_ring_sx, input_from_str},
        form::{
            SelectionArgs, field_control_sx,
            glyphs::{CloseIcon, UploadIcon},
            use_bound, use_field, use_field_frame,
        },
        layout::use_box,
    },
    hooks::{use_css, use_element, use_local_state, use_theme},
    platform::ElementApi,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        CHIP_HEIGHT, FILE_FIELD_DROPZONE_HEIGHT, FILE_FIELD_PADDING, FILE_FIELD_RADIUS,
        FileFieldDefaults, FileFieldVariant, Size,
    },
    utils::warn,
};

use super::accept::{accept_hint, accepts};
use super::files::{Files, format_size};

input_from_str!(FileFieldVariant);

/// What the next render owes the keyboard, once the control focus was on has
/// gone away. See [[principles/focus-after-removal]] in the project brain.
#[derive(Clone, Copy, PartialEq)]
enum FocusDebt {
    /// This row was removed; focus the one that took its place.
    Removed(usize),
    /// Files arrived; focus the first row when the surface stood down.
    Took,
}

/// The one-line control, the `Select` trigger's shape: the value or the
/// placeholder, and whatever the frame's trailing slot holds beside it.
///
/// A `div` with `role="button"`, not a `<button>`: the chips carry their own
/// remove buttons, and an interactive descendant of a button is invalid -
/// the same reason `MultiSelect`'s trigger is a `div`.
static FILE_CONTROL_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .display("flex")
        .align_items("center")
        // Lets the control shrink inside the frame, which is what makes the
        // value slot's own ellipsis take effect instead of the frame growing.
        .min_width("0")
        .gap("4px")
        .cursor("pointer")
        .user_select("none")
        .selector(
            "& > [data-slot='value']",
            sx().flex("1 1 auto")
                .min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
        .selector("& [data-placeholder]", sx().color("grey.6"))
        // A filename has no spaces to break on, so without this one long one
        // pushes the frame past whatever width its parent allows.
        .selector(
            "& [data-slot='name']",
            sx().display("block")
                .min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
        .selector("& [data-slot='chip'] > *", sx().max_width("100%"))
        // The chip the keyboard is on. The ring goes on what was drawn, not on
        // the wrapper, so it follows that element's own radius.
        .selector(
            "& [data-slot='chip'][data-cursor='true'] > *",
            focus_ring_sx(),
        )
        .when(
            "multiple",
            sx().selector(
                "& > [data-slot='value']",
                sx().display("flex")
                    .flex_wrap("wrap")
                    .gap("4px")
                    // Chips wrap, and the single-line clip would cut the
                    // second row off at the slot's edge.
                    .overflow("visible"),
            ),
        )
        .when("disabled", sx().cursor("not-allowed"))
});

/// The tall surface. Same value, same input, same picker - only the thing the
/// user aims at differs, which is why this is a variant and not a component.
static FILE_DROPZONE_SX: StaticSx = StaticSx::new(|| {
    // The surface carries the size and radius states, so the per-size vars
    // resolve here and its children inherit them.
    FileFieldDefaults::theme_vars()
        .display("flex")
        .flex_direction("column")
        .align_items("center")
        .justify_content("center")
        .gap("4px")
        .width("100%")
        // Height, padding, font and radius all come from the field's own
        // scale, so a dropzone and a text field at one `size` read as a family.
        .min_height(FILE_FIELD_DROPZONE_HEIGHT.value())
        .padding(FILE_FIELD_PADDING.value())
        .border("2px dashed")
        .border_color("grey.4")
        .border_radius(FILE_FIELD_RADIUS.value())
        .background("transparent")
        .cursor("pointer")
        .text_align("center")
        .transition("border-color 150ms, background 150ms")
        .selector("& [data-slot='hint']", sx().color("grey.6"))
        // Sized against the text, which is the one thing on the surface that
        // already scales.
        .selector("& > svg", sx().width("2em").height("2em").color("grey.6"))
        .when(
            "dragging",
            sx().border_color("primary")
                .background("color-mix(in srgb, var(--lsx-primary) 8%, transparent)"),
        )
        .when(
            "disabled",
            sx().cursor("not-allowed").border_color("grey.3"),
        )
});

/// The picked files, under the surface rather than inside it: a dropzone that
/// grows with its own contents stops being a target to aim at.
static FILE_CARD_LIST_SX: StaticSx = StaticSx::new(|| {
    // The list is a sibling of the surface, not a descendant, so it resolves
    // the same vars for its own cards.
    FileFieldDefaults::theme_vars()
        .display("flex")
        .flex_direction("column")
        .gap("4px")
        .width("100%")
        .margin_top("8px")
        .list_style("none")
        .padding("0")
});

static FILE_CARD_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .gap("8px")
        .width("100%")
        // The row must be allowed to shrink, or a long name pushes the x out
        // of the card instead of being clipped.
        .min_width("0")
        .padding(format!(
            "calc({} / 2) {}",
            FILE_FIELD_PADDING.value(),
            FILE_FIELD_PADDING.value()
        ))
        .border("1px solid")
        .border_color("grey.3")
        .border_radius(FILE_FIELD_RADIUS.value())
        .selector(
            "& [data-slot='name']",
            sx().flex("1 1 auto")
                .min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
        .selector(
            "& [data-slot='size']",
            sx().flex("0 0 auto").color("grey.6").font_size("0.85em"),
        )
        // `inline-flex`, or the button sits on the text's baseline instead of
        // the row's centre line - the same fix the chip's x needed.
        .selector(
            "& [data-slot='remove']",
            sx().flex("0 0 auto")
                .display("inline-flex")
                .align_items("center"),
        )
});

/// A flex line of its own, or the remove button hangs off the label's baseline.
static FILE_CHIP_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex").max_width("100%").selector(
        "& [data-slot='remove']",
        sx().display("inline-flex").align_items("center"),
    )
});

/// The real input, and the only thing a form posts. Hidden rather than absent:
/// its `FileList` is what `set_files` keeps equal to `value`.
static FILE_INPUT_SX: StaticSx = StaticSx::new(|| sx().display("none"));

field_props! {
    pub struct FileFieldProps {
        /// Strictly controlled - pair it with `onchange`. Takes a
        /// `FileData`, an `Option<FileData>` or a `Vec<FileData>`.
        #[props(default, into)]
        value: Files,
        /// Lets the user pick and drop more than one file. A single-file
        /// field keeps the first of whatever it is given.
        #[props(default)]
        multiple: bool,
        /// The `accept` attribute: `.pdf`, `image/png`, `image/*`, or a
        /// comma-separated list. The picker applies it, and so does a drop.
        #[props(default, into)]
        accept: Option<String>,
        /// Asks a phone for a fresh capture - `user` or `environment`.
        #[props(default, into)]
        capture: Option<String>,
        /// Shown while nothing is picked.
        #[props(default, into)]
        placeholder: Option<String>,
        /// Shows an x that empties the field.
        #[props(default)]
        clearable: Option<bool>,
        /// Which control to draw: a one-line input, or a drop surface.
        #[props(default, into)]
        variant: Input<FileFieldVariant>,
        /// Draws one picked file. Defaults to a `Chip` with an x when
        /// `multiple`, and to the bare filename when not. A caller who
        /// overrides it draws the whole thing, remove control included -
        /// `args.remove` is the wiring.
        #[props(default)]
        selection: Option<Callback<SelectionArgs<FileData>, Element>>,
        /// The hidden `input[type="file"]`'s name, so the files post with a
        /// form. The list is kept equal to `value`, removals included.
        /// A path - `Claim::FIELDS.receipts()` - also binds the files to the
        /// surrounding `Form`'s value when there is no `onchange`.
        #[props(default, into)]
        name: crate::components::FieldName<Files>,
        /// Fires with the files the field should hold next - a pick, a drop,
        /// a removal or a clear.
        #[props(default)]
        onchange: Option<EventHandler<Files>>,
        /// Rules over the files, shown once the field loses focus or its form
        /// is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<Files>,
        /// The `Dropzone` variant's prompt, inside the surface. Ignored by
        /// the `Input` variant, which shows `placeholder` instead.
        #[props(default)]
        children: Element,
    }
}

/// Files picked from the system dialog or dropped on the control, wearing the
/// field slots. Controlled: it renders `value` and asks for the next set
/// through `onchange`.
///
/// Both variants drive one hidden `input[type="file"]`, which is also what a
/// `name` posts - the component writes the caller's own list back into it, so
/// a removed file stops posting.
#[component]
pub fn FileField(props: FileFieldProps) -> Element {
    let theme = use_theme();
    let input_element = use_element();
    // Where the focus goes when a card is removed: the card that took its
    // place, or the surface when none is left.
    let surface_element = use_element();
    let list_element = use_element();
    // What the input itself holds, and *which* input held it. A pick fills
    // the list; every other edit happens in Rust, and the effect below writes
    // the difference back. A `Signal` rather than a `use_local_state`, which
    // needs `Copy`.
    let mut mirrored = use_signal(|| (None::<usize>, Files::default()));

    let size = props.size.copied_or(theme.file_field.size);
    let radius = props.radius.copied_or(theme.file_field.radius);
    let variant = props.variant.copied_or(theme.file_field.variant);
    let clearable = props.clearable.unwrap_or(theme.file_field.clearable);
    let required = props.required.unwrap_or(false);
    let multiple = props.multiple;
    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let interactive = (props.onchange.is_some() || bound.is_bound()) && !disabled;

    if props.onchange.is_none() && !bound.is_bound() && !disabled {
        warn("FileField: `value` without `onchange` can never change.");
    }

    let value = bound.value().unwrap_or_else(|| props.value.clone());
    let accept = props.accept.clone().unwrap_or_default();
    let dragging = use_local_state(|| false);

    let onchange = props.onchange;
    let setter = bound.setter();
    let emit = use_callback(move |files: Files| match (&onchange, &setter) {
        (Some(onchange), _) => onchange.call(files),
        (None, Some(setter)) => setter.set(files),
        (None, None) => {}
    });

    // A drop bypasses the picker, which is the only place `accept` applies by
    // itself - so the component applies it, and says so rather than dropping
    // files in silence.
    let cards = variant == FileFieldVariant::Dropzone;
    // A single-file dropzone has nothing left to ask for once it holds its
    // file: the card replaces the surface, and removing the file brings it
    // back. A `multiple` one keeps taking files, so it keeps its surface.
    let surface = !cards || multiple || value.is_empty();

    // What the next render owes the keyboard. A removal destroys the button
    // focus was on; a pick that stands the surface down destroys the
    // *surface* focus was on. Either way focus would fall to the body.
    let mut owed = use_signal(|| None::<FocusDebt>);

    let take = use_callback(move |files: Vec<FileData>| {
        let picked = files.len();
        let kept: Files = files
            .into_iter()
            .filter(|file| accepts(&accept, &file.name(), file.content_type().as_deref()))
            .collect::<Files>()
            .truncated(multiple);
        if kept.len() < picked {
            warn("FileField: dropped files that `accept` or `multiple` excludes.");
        }
        if !kept.is_empty() {
            owed.set(Some(FocusDebt::Took));
            emit.call(kept);
        }
    });

    // The input's `FileList` is the only thing a form posts, and it cannot be
    // edited - so whenever the caller's value and the input disagree (a
    // removal, a clear, a drop), the list is written back.
    let synced = value.clone();
    // The token, not merely `is_mounted`: switching `variant` unmounts the
    // input and mounts a fresh one, whose `FileList` starts empty. Without
    // this the mirror still claimed the old list and nothing rewrote it, so a
    // field holding a file posted nothing.
    let mount = input_element.mount_token();
    use_effect(use_reactive!(|(synced, mount)| {
        let (mirrored_mount, mirrored_files) = mirrored.peek().clone();
        if mirrored_mount == mount && mirrored_files == synced {
            return;
        }
        if input_element.set_files(&synced).is_ok() {
            mirrored.set((mount, synced));
        }
    }));

    // The wrapper needs a **definite** width, not merely `min-width: 0`: a
    // stretched flex item is floored at its own min-content width, and a long
    // filename is one unbreakable word. Measured in the browser against every
    // other candidate - `min-width: 0` up the chain, `overflow: hidden`,
    // `contain: inline-size`, a breakable filename - and this is the only one
    // that keeps the frame inside its parent.
    let field_sx: Input<Sx> = match props.sx.as_ref() {
        // The caller's own declarations come second, so they still win.
        Some(caller) => sx().width("100%").and(caller.clone()).into(),
        None => sx().width("100%").into(),
    };

    let field = use_field()
        .labelled_by()
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
        .sx(&field_sx)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let icon_size: Input<ThemeAwareValue> = ThemeAwareValue::Size(size).into();
    let has_files = !value.is_empty();
    let clear = (clearable && has_files && interactive).then(|| {
        rsx! {
            ActionIcon {
                aria_label: "Clear",
                size: icon_size.clone(),
                onclick: move |event: MouseEvent| {
                    // Clearing is not a click on the control, which would open
                    // the picker straight after emptying the field.
                    event.stop_propagation();
                    emit.call(Files::default());
                },
                CloseIcon {}
            }
        }
    });

    // Chips ride inside the control, so they sit one step down the field's
    // own scale - `xs` has nowhere lower to go.
    let chip_size = Size::ALL[size.index().saturating_sub(1)];
    let chip_class = use_css(Some(&FILE_CHIP_SX), CssLayer::Framework);
    let card_class = use_css(Some(&FILE_CARD_SX), CssLayer::Framework);

    // Which chip the keyboard is on. Only the `Input` variant has one: the
    // dropzone's cards sit outside the control, so their remove buttons are
    // ordinary tab stops and need no cursor at all.
    let mut cursor = use_signal(|| None::<usize>);
    let count = value.len();
    // Removing the chip under the cursor leaves the index pointing at the one
    // that took its place, and past the end it clamps.
    let chip_cursor = match count {
        0 => None,
        count => cursor().map(|index| index.min(count - 1)),
    };
    let id_prefix = format!("{}-file", field.id());

    // Prepared unconditionally, the way every `use_box` must be, and used
    // only by the dropzone variant.
    let card_list_style = use_box()
        .framework_sx(&FILE_CARD_LIST_SX)
        .states(field.states())
        .prepare();

    // Removing a row destroys the button the keyboard was on, and focus would
    // otherwise fall to the body. It moves to the row that took this one's
    // place, to the new last row when the removed one was last, and to the
    // surface when the list is empty - which is also the only control left
    // there.
    let remaining = value.len();
    let focus_prefix = format!("{}-remove", field.id());
    let surface_survives = surface;
    use_effect(use_reactive!(|remaining| {
        let Some(debt) = *owed.peek() else {
            return;
        };
        owed.set(None);
        let target = match debt {
            // Nothing left to remove: the surface is the only control there.
            FocusDebt::Removed(_) if remaining == 0 => None,
            // The row that took this one's place, clamped to the new last.
            FocusDebt::Removed(index) => Some(index.min(remaining - 1)),
            // A single-file dropzone puts its surface away once it holds a
            // file, so the x that replaced it is what the keyboard needs.
            FocusDebt::Took if !surface_survives && remaining > 0 => Some(0),
            // The surface is still there, and still focused.
            FocusDebt::Took => return,
        };
        match target {
            Some(index) => {
                if let Ok(button) = list_element.query_selector(&format!("#{focus_prefix}-{index}"))
                {
                    let _ = button.focus();
                }
            }
            None => {
                let _ = surface_element.focus();
            }
        }
    }));

    // Read off the field before the closures below, which outlive the borrow
    // they would otherwise hold while `field.render` consumes it.
    let (labelledby, describedby, invalid) =
        (field.label_id(), field.describedby(), field.invalid());

    let draw = props.selection;
    let files = value.clone();
    let remove_at = use_callback(move |index: usize| {
        owed.set(Some(FocusDebt::Removed(index)));
        emit.call(files.without(index));
    });

    let drawn_files = value.clone();
    let drawn = drawn_files
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, file)| {
            let remove = Callback::new(move |_: ()| remove_at.call(index));
            let content = match &draw {
                Some(draw) => draw.call(SelectionArgs {
                    value: file.clone(),
                    remove,
                }),
                None => match cards {
                    true => default_card(
                        &file,
                        remove,
                        icon_size.clone(),
                        interactive,
                        format!("{}-remove-{index}", field.id()),
                    ),
                    false => default_chip(&file, remove, chip_size, multiple, interactive),
                },
            };
            match cards {
                true => rsx! {
                    li { key: "{index}", class: card_class.clone(), {content} }
                },
                false => rsx! {
                    span {
                        key: "{index}",
                        class: chip_class.clone(),
                        "data-slot": "chip",
                        id: "{id_prefix}-{index}",
                        "data-cursor": (chip_cursor == Some(index)).then_some("true"),
                        {content}
                    }
                },
            }
        })
        .collect::<Vec<_>>();

    let placeholder = props.placeholder.clone().unwrap_or_default();
    // The dropzone's files are a list under the surface; the input's are the
    // control's own contents.
    let (value_slot, card_list) = match cards {
        true => (
            rsx! {},
            has_files.then(|| {
                card_list_style
                    // With no surface left, the list is what the field's
                    // label names.
                    .attr(
                        "aria-labelledby",
                        (!surface).then(|| labelledby.clone()).flatten(),
                    )
                    .element(&list_element)
                    .render(HtmlTag::Ul, Vec::new(), rsx! { {drawn.into_iter()} })
            }),
        ),
        false => (
            match has_files {
                true => rsx! {
                    span { "data-slot": "value", {drawn.into_iter()} }
                },
                false => rsx! {
                    span { "data-slot": "value",
                        span { "data-placeholder": "true", "{placeholder}" }
                    }
                },
            },
            None,
        ),
    };

    // What the surface says, when the caller supplies no prompt of its own.
    // `children` when the caller wrote one, `placeholder` next, and an
    // English default last - the shape `Dialog`'s `close_label` set.
    let written = props
        .children
        .as_ref()
        .is_ok_and(|children| *children != VNode::default());
    let prompt = match (written, props.placeholder.clone()) {
        (true, _) => rsx! { {props.children.clone()} },
        (false, Some(placeholder)) => rsx! { span { "{placeholder}" } },
        (false, None) => {
            let text = match multiple {
                true => "Drop files here, or click to pick",
                false => "Drop a file here, or click to pick",
            };
            rsx! { span { "{text}" } }
        }
    };
    // Read off `accept` rather than written beside it, so the prompt cannot
    // claim something the picker would refuse.
    let hint = accept_hint(&props.accept.clone().unwrap_or_default()).map(|hint| {
        rsx! {
            span { "data-slot": "hint", "{hint}" }
        }
    });

    let name = bound.name().map(str::to_string);
    let input = use_box()
        .framework_sx(&FILE_INPUT_SX)
        .prepare()
        .attr_default("type", "file")
        .attr("multiple", multiple)
        .attr("accept", props.accept.clone())
        .attr("capture", props.capture.clone())
        .attr("name", name)
        .attr("disabled", !interactive)
        // It is the control that is focusable and named; this is plumbing.
        .attr("tabindex", "-1")
        .attr("aria-hidden", "true")
        .element(&input_element)
        .event("onchange", move |event: FormEvent| take.call(event.files()))
        .render(HtmlTag::Input, Vec::new(), ());

    let states: Input<States> = field
        .states()
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("multiple", multiple)
        .with("dragging", dragging.get())
        .into();

    let open = move |_: MouseEvent| {
        if interactive {
            let _ = input_element.click();
        }
    };
    // Two keyboards on one element, the way `MultiSelect`'s trigger has them:
    // the chips answer Left, Right, Backspace and Delete, and everything else
    // opens the picker. A dropzone has no chips - its cards are ordinary tab
    // stops outside the control - so it only ever opens.
    let chips = !cards && interactive;
    let on_key = move |event: Event<KeyboardData>| {
        if !interactive {
            return;
        }
        match event.key() {
            Key::ArrowLeft if chips && count > 0 => {
                event.prevent_default();
                cursor.set(Some(match chip_cursor {
                    Some(index) => index.saturating_sub(1),
                    // From no cursor, the last chip - the one Backspace would
                    // have taken.
                    None => count - 1,
                }));
            }
            Key::ArrowRight if chips && count > 0 => {
                event.prevent_default();
                cursor.set(match chip_cursor {
                    Some(index) if index + 1 < count => Some(index + 1),
                    // Past the last chip is back to no cursor, not a wrap.
                    _ => None,
                });
            }
            Key::Backspace | Key::Delete if chips && count > 0 => {
                event.prevent_default();
                remove_at.call(chip_cursor.unwrap_or(count - 1));
            }
            // A `role="button"` answers Enter and Space itself - the browser
            // does that only for a real `<button>`, which the chips rule out.
            Key::Enter => {
                event.prevent_default();
                let _ = input_element.click();
            }
            Key::Character(ref character) if character == " " => {
                event.prevent_default();
                let _ = input_element.click();
            }
            _ => {}
        }
    };

    let dragging_over = dragging.clone();
    let dragging_off = dragging.clone();
    let dragging_drop = dragging.clone();
    let control = move |style: crate::components::layout::BoxStyle, children: Element| {
        style
            .element(&surface_element)
            .attr("role", "button")
            .attr("tabindex", if interactive { "0" } else { "-1" })
            .attr("aria-labelledby", labelledby)
            .attr("aria-describedby", describedby)
            .attr("aria-invalid", invalid.then_some("true"))
            .attr("aria-required", required.then_some("true"))
            .attr("aria-disabled", !interactive)
            .attr(
                "aria-activedescendant",
                chip_cursor.map(|index| format!("{id_prefix}-{index}")),
            )
            .event("onclick", open)
            .event("onkeydown", on_key)
            .event("ondragover", move |event: DragEvent| {
                if interactive {
                    // Without this the browser opens the file instead.
                    event.prevent_default();
                    dragging_over.set(true);
                }
            })
            .event("ondragleave", move |_: DragEvent| dragging_off.set(false))
            .event("ondrop", move |event: DragEvent| {
                event.prevent_default();
                dragging_drop.set(false);
                if interactive {
                    take.call(event.files());
                }
            })
            .render(HtmlTag::Div, props.attributes.clone(), children)
    };

    match variant {
        FileFieldVariant::Input => {
            let frame = use_field_frame()
                .trailing(&clear)
                .states(field.states())
                .prepare();
            // The frame draws the ring, so the control must not draw a second.
            let style = use_box()
                .framework_sx(&FILE_CONTROL_SX)
                .focus_ring(false)
                .states(&states)
                .prepare();
            field.render(frame.render(control(
                style,
                rsx! {
                    {value_slot}
                    {input}
                },
            )))
        }
        FileFieldVariant::Dropzone => {
            let style = use_box()
                .framework_sx(&FILE_DROPZONE_SX)
                .states(&states)
                .prepare();
            // The cards sit under the surface, not in it: a dropzone that
            // grows with its own contents stops being a target to aim at.
            // The input stays mounted either way - it is what posts.
            let drop_target = surface.then(|| {
                control(
                    style,
                    rsx! {
                        UploadIcon {}
                        {prompt}
                        {hint}
                    },
                )
            });
            field.render(rsx! {
                {drop_target}
                {card_list}
                {input}
            })
        }
    }
}

/// A chip with an x when the field takes several files, the bare filename
/// when it takes one - a chip's x beside the frame's clear button would be
/// two removes for one file.
fn default_chip(
    file: &FileData,
    remove: Callback<()>,
    size: Size,
    multiple: bool,
    interactive: bool,
) -> Element {
    let name = file.name();
    if !multiple {
        // `data-slot` so the control can clip it: one long filename would
        // otherwise push the frame past its parent's width.
        return rsx! {
            span { "data-slot": "name", "{name}" }
        };
    }
    // A fraction of the chip's own height, not of its font: the two do not
    // scale at the same rate, so an `em` x shrinks against its chip as the
    // field grows.
    let icon_size: Input<ThemeAwareValue> =
        ThemeAwareValue::String(format!("calc({} * 0.6)", CHIP_HEIGHT.value(size))).into();
    rsx! {
        Chip { size,
            span { "data-slot": "name", "{name}" }
            if interactive {
                span {
                    "data-slot": "remove",
                    // A remove is not a click on the control, which would open
                    // the picker under the chip that just went away.
                    onclick: move |event: MouseEvent| {
                        event.stop_propagation();
                        remove.call(());
                    },
                    ActionIcon {
                        aria_label: "Remove {name}",
                        size: icon_size,
                        // A native button inherits neither `color` nor
                        // `font-size` - it takes the UA's `buttontext`.
                        sx: sx()
                            .color("inherit")
                            .font_size("inherit")
                            .border_radius("50%")
                            .selector(
                                "&:hover",
                                sx().background("color-mix(in srgb, currentColor 20%, transparent)"),
                            ),
                        // The control is one tab stop.
                        tabindex: "-1",
                        CloseIcon {}
                    }
                }
            }
        }
    }
}

/// One picked file as a row under the dropzone: the name, its size, and an x.
/// The card is outside the control, so its remove button is an ordinary tab
/// stop rather than something the control's own keyboard has to reach.
fn default_card(
    file: &FileData,
    remove: Callback<()>,
    icon_size: Input<ThemeAwareValue>,
    interactive: bool,
    id: String,
) -> Element {
    let name = file.name();
    let size = format_size(file.size());
    rsx! {
        span { "data-slot": "name", "{name}" }
        span { "data-slot": "size", "{size}" }
        if interactive {
            span { "data-slot": "remove",
                ActionIcon {
                    // The id is how the focus finds the row that takes this
                    // one's place: a list cannot hold a hook per row.
                    id,
                    aria_label: "Remove {name}",
                    size: icon_size,
                    sx: sx()
                        .color("inherit")
                        .border_radius("50%")
                        .selector(
                            "&:hover",
                            sx().background("color-mix(in srgb, currentColor 12%, transparent)"),
                        ),
                    onclick: move |_: MouseEvent| remove.call(()),
                    CloseIcon {}
                }
            }
        }
    }
}
