use std::{cell::Cell, rc::Rc};

use dioxus::html::FileData;
use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    CssLayer,
    components::{
        common::{Glyph, HtmlTag, Input, States, input_from_str},
        feedback::Loader,
        form::{
            CropOptions, CropRect, SelectionArgs, field_parts_enum, field_props, slot_icon_size,
            use_bound, use_chip_announcer, use_field,
        },
        layout::{BoxStyle, use_box},
    },
    context::IconSlot,
    hooks::{
        ElementHandle, current_localization, use_css, use_element, use_local_state, use_theme,
    },
    platform::{self, ElementApi},
    sx::{Sx, ThemeAwareValue, sx},
    theme::{FileFieldVariant, Size},
    utils::warn,
};

use super::accept::accept_hint;
use super::crop::CropGate;
use super::files::Files;
use super::intake::{Intake, Taking, use_file_intake, use_input_mirror};
use super::rows::{ChipKeys, FileRows, FocusDebt, use_chip_cursor, use_focus_debt};
use super::styles::{FILE_CARD_LIST_SX, FILE_CARD_SX, FILE_CHIP_SX, FILE_INPUT_SX};
use super::surface::{
    Cards, Chips, Surface, file_dropzone_variant, file_input_variant, trailing_slot,
};

input_from_str!(FileFieldVariant);

field_parts_enum! {
    /// [`FileField`]'s inner parts, for its `parts` prop. The frame parts are
    /// the `Input` variant's, the cards the `Dropzone`'s.
    pub enum FileFieldPart {
        /// The `Input` variant's bordered box.
        Frame = "frame" => "& > [data-slot='frame']",
        /// The group holding the Browse button: inside the frame, or the
        /// dropzone's surface.
        Control = "control" => "& > [data-slot='frame'] > [data-slot='control'], & > [data-slot='control']",
        /// The Browse button: the rest of the line, or the whole surface.
        Browse = "browse" => "& > [data-slot='frame'] > [data-slot='control'] > [data-slot='browse'], & > [data-slot='control'] > [data-slot='browse']",
        /// One picked file's chip, `Input` variant.
        Chip = "chip" => "& > [data-slot='frame'] > [data-slot='control'] > [data-slot='value'] > [data-slot='chip']",
        /// The loader and clear button after the control, `Input` variant.
        Trailing = "trailing" => "& > [data-slot='frame'] > [data-slot='trailing']",
        /// One picked file's card under the surface, `Dropzone` variant.
        Card = "card" => "& > [role='list'] > [data-slot='card']",
    }
}

field_props! {
    parts(FileFieldPart);
    pub struct FileFieldProps {
        /// Controlled: pair it with `onchange`. Takes a `FileData`, an
        /// `Option` or a `Vec` of them.
        #[props(default, into)]
        value: Files,
        /// Takes more than one file; otherwise the field keeps the first.
        #[props(default)]
        multiple: bool,
        /// The `accept` attribute (`.pdf`, `image/*`, ...), applied to drops too.
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
        /// An upload is in flight: draws a `Loader` and sets `aria-busy`. Blocks
        /// nothing; `disabled` does.
        #[props(default)]
        loading: Option<bool>,
        /// Which control to draw: a one-line input, or a drop surface.
        #[props(default, into)]
        variant: Input<FileFieldVariant>,
        /// Draws one picked file, remove control included (`args.remove`).
        #[props(default)]
        selection: Option<Callback<SelectionArgs<FileData>, Element>>,
        /// What the files post as, kept equal to `value`. A path also binds it
        /// to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<Files>,
        /// Fires with the next files: a pick, a drop, a removal or a clear.
        #[props(default)]
        onchange: Option<EventHandler<Files>>,
        /// Crops a single picked PNG, JPEG, WebP, BMP or AVIF in a dialog before
        /// `onchange` gets it, cut on every platform but an AVIF on Blitz, which
        /// stays whole. Cancel drops the pick. Ignored with `multiple`.
        #[props(default)]
        crop: Option<CropOptions>,
        /// The crop the dialog applied, on every platform.
        #[props(default)]
        oncrop: Option<EventHandler<CropRect>>,
        /// Rules over the files, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<Files>,
        /// The `Dropzone` variant's prompt; `Input` shows `placeholder`.
        #[props(default)]
        children: Element,
    }
}

/// Files picked from the system dialog or dropped on the control.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{FileField, Files};
/// # fn app() -> Element {
/// let mut resume = use_signal(|| None::<dioxus::html::FileData>);
/// rsx! {
///     FileField {
///         label: "Resume",
///         accept: ".pdf",
///         value: resume(),
///         onchange: move |files: Files| resume.set(files.one()),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/file-field>
#[component]
pub fn FileField(props: FileFieldProps) -> Element {
    let theme = use_theme();
    let input_element = use_element();
    // Where the focus goes when a row is removed: the row that took its
    // place, or the Browse button when none is left.
    let browse_element = use_element();
    // The chips or the cards, whichever the variant draws.
    let list_element = use_element();
    // What the input holds, and which input held it; `use_input_mirror`
    // writes Rust-side edits back.
    let mirrored = use_signal(|| (None::<usize>, Files::default()));
    let opening = use_hook(|| Rc::new(Cell::new(false)));

    let size = props.size.copied_or(theme.file_field.size);
    let radius = props.radius.copied_or(theme.file_field.radius);
    let variant = props.variant.copied_or(theme.file_field.variant);
    let clearable = props.clearable.unwrap_or(theme.file_field.clearable);
    let required = props.required.unwrap_or(false);
    let multiple = props.multiple;
    let loading = props.loading.unwrap_or(false);
    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let interactive = (props.onchange.is_some() || bound.is_bound()) && !disabled;
    // A read-only field stays reachable and posts (`interactive`); `editable`
    // refuses the picker, drops, clear and removes.
    let editable = interactive && !props.readonly.unwrap_or(false);

    if props.onchange.is_none() && !bound.is_bound() && !disabled {
        warn("FileField: `value` without `onchange` can never change.");
    }

    let value = bound.value().unwrap_or_else(|| props.value.clone());
    let announcer = use_chip_announcer(value.iter().map(FileData::name).collect());
    let dragging = use_local_state(|| false);

    let crop = props.crop.filter(|_| !multiple);
    let pending = use_signal(|| None::<FileData>);

    let cards = variant == FileFieldVariant::Dropzone;
    // A full single-file dropzone swaps its surface for the card.
    let surface = !cards || multiple || value.is_empty();

    let Intake {
        emit,
        mut owed,
        take,
    } = use_file_intake(Taking {
        onchange: props.onchange,
        setter: bound.setter(),
        accept: props.accept.clone().unwrap_or_default(),
        multiple,
        editable,
        crop: crop.map(|_| pending),
    });

    use_input_mirror(input_element, mirrored, value.clone());

    // The input's own picker, or the system dialog where the input opens none.
    let accept = props.accept.clone().unwrap_or_default();
    let capture = props.capture.clone();
    let open = use_callback(move |()| {
        platform::pick_files(
            &accept,
            multiple,
            capture.as_deref(),
            || {
                let _ = input_element.click();
            },
            move |picked| take.call(picked),
        );
    });

    let field_sx = full_width(&props.sx);

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
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let has_files = !value.is_empty();
    // Chips ride inside the control, so they sit one step down the field's
    // own scale. The loader takes the same step, for the same reason.
    let chip_size = size.step_down();
    let trailing = trailing_slot(
        loading.then_some(chip_size),
        clearable && has_files && editable,
        size,
        browse_element,
        emit,
    );
    let chip_class = use_css(Some(&FILE_CHIP_SX), CssLayer::Framework);
    let card_class = use_css(Some(&FILE_CARD_SX), CssLayer::Framework);

    let count = value.len();
    let (cursor, chip_cursor) = use_chip_cursor(variant, count, cards);
    let id_prefix = format!("{}-file", field.id());

    // Prepared unconditionally, the way every `use_box` must be, and used
    // only by the dropzone variant.
    let card_list_style = use_box()
        .framework_sx(&FILE_CARD_LIST_SX)
        .states(field.states())
        .prepare();

    use_focus_debt(
        owed,
        count,
        surface,
        list_element,
        browse_element,
        // Each row's tab stop: a card's x, or the chip itself.
        match cards {
            true => format!("{}-remove", field.id()),
            false => id_prefix.clone(),
        },
    );

    // Read off the field before the closures below, which outlive the borrow
    // they would otherwise hold while `field.render` consumes it.
    let (labelledby, describedby, invalid) =
        (field.label_id(), field.describedby(), field.invalid());

    let files = value.clone();
    // Guarded here as well as by hiding the x: a caller's own `selection`
    // gets `remove` too.
    let remove_at = use_callback(move |index: usize| {
        if !editable {
            return;
        }
        owed.set(Some(FocusDebt::Removed(index)));
        emit.call(files.without(index));
    });
    let files = value.clone();
    // Backspace on the Browse button: the focus is not on the row, so it stays.
    let drop_at = use_callback(move |index: usize| {
        if editable {
            emit.call(files.without(index));
        }
    });

    let keys = ChipKeys {
        interactive,
        editable,
        count,
        remove_at,
        drop_at,
        list: list_element,
        browse: browse_element,
        id_prefix: id_prefix.clone(),
        cursor,
    };

    let rows = FileRows {
        draw: props.selection,
        remove_at,
        cards,
        multiple,
        editable,
        icon_size: ThemeAwareValue::Size(slot_icon_size(size)).into(),
        size,
        // Only once the surface is gone: otherwise the loader is on the
        // surface, and one is enough.
        card_loader: (loading && !surface).then_some(chip_size),
        field_id: field.id().to_string(),
        chip_cursor,
        keys: keys.clone(),
        chip_class,
        card_class,
    };
    let drawn = rows.all(&value);

    let input = file_input(
        use_box().framework_sx(&FILE_INPUT_SX).prepare(),
        input_element,
        &props,
        bound.name().map(str::to_string),
        interactive,
        take,
    );

    let states: Input<States> = field
        .states()
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("multiple", multiple)
        .with("dragging", dragging.get())
        .into();

    let control = Surface {
        browse: browse_element,
        id: field.id().to_string(),
        labelledby: labelledby.clone(),
        describedby,
        invalid,
        required,
        interactive,
        editable,
        loading,
        open,
        // A dropzone has no chips - its cards are ordinary tab stops outside
        // the group - so its button only ever opens.
        keys: (!cards).then_some(keys),
        take,
        dragging,
        opening,
        attributes: props.attributes.clone(),
    };

    let gate = crop.map(|options| {
        rsx! { CropGate { pending, options, emit, oncrop: props.oncrop } }
    });
    let field = match variant {
        // A scope per variant: their hooks differ, so a `variant` switch must
        // remount rather than reuse the other's slots.
        FileFieldVariant::Input => file_input_variant(
            field,
            control,
            states,
            Chips {
                trailing,
                drawn,
                list_element,
                placeholder: props.placeholder.clone().unwrap_or_default(),
            },
            input,
            announcer,
        ),
        FileFieldVariant::Dropzone => file_dropzone_variant(
            field,
            control,
            states,
            Cards {
                prompt: dropzone_prompt(&props, loading.then_some(size)),
                drawn,
                style: card_list_style,
                list_element,
                labelledby,
                surface,
                loading,
                has_files,
            },
            input,
            announcer,
        ),
    };
    match gate {
        Some(gate) => rsx! { {field} {gate} },
        None => field,
    }
}

/// The caller's `sx` behind a definite width: `min-width: 0` and the rest let a
/// long filename push the frame out of its parent (measured in the browser).
fn full_width(caller: &Input<Sx>) -> Input<Sx> {
    match caller.as_ref() {
        // The caller's own declarations come second, so they still win.
        Some(caller) => sx().width("100%").and(caller.clone()).into(),
        None => sx().width("100%").into(),
    }
}

/// The real `input[type="file"]`: the picker, and what a form posts.
fn file_input(
    style: BoxStyle,
    element: ElementHandle,
    props: &FileFieldProps,
    name: Option<String>,
    interactive: bool,
    take: Callback<Vec<FileData>>,
) -> Element {
    style
        .attr_default("type", "file")
        .attr("multiple", props.multiple)
        .attr("accept", props.accept.clone())
        .attr("capture", props.capture.clone())
        .attr("name", name)
        .attr("disabled", !interactive)
        // The native half of `required`: a form's own validation reads it.
        .attr("required", props.required.unwrap_or(false))
        // It is the Browse button that is focusable and named; this is plumbing.
        .attr("tabindex", "-1")
        .attr("aria-hidden", "true")
        .element(&element)
        .event("onchange", move |event: FormEvent| take.call(event.files()))
        .render(HtmlTag::Input, Vec::new(), ())
}

/// The surface: an icon or `loader`, the prompt (`children`, `placeholder`,
/// then the localization's), and a hint read off `accept`.
fn dropzone_prompt(props: &FileFieldProps, loader: Option<Size>) -> Element {
    let written = props
        .children
        .as_ref()
        .is_ok_and(|children| *children != VNode::default());
    let labels = current_localization().file_field;
    let prompt = match (written, props.placeholder.clone()) {
        (true, _) => props.children.clone(),
        (false, Some(placeholder)) => rsx! { span { "{placeholder}" } },
        (false, None) => {
            let text = match props.multiple {
                true => labels.drop_files,
                false => labels.drop_file,
            };
            rsx! { span { "{text}" } }
        }
    };
    let accept = props.accept.as_deref().unwrap_or_default();
    let hint = accept_hint(accept, labels.any_of).map(|hint| {
        rsx! {
            span { "data-slot": "hint", "{hint}" }
        }
    });
    rsx! {
        if let Some(size) = loader {
            Loader { size }
        } else {
            Glyph { slot: IconSlot::Upload, icon: lucide::upload::outlined }
        }
        {prompt}
        {hint}
    }
}
