use std::{cell::Cell, rc::Rc};

use dioxus::html::{FileData, HasFileData};
use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        ActionIcon, HtmlTag, Input, States, VisuallyHidden,
        common::{
            CloseIcon, UploadIcon, field_props, focus_ring_sx, input_from_str, navigation_chord,
            ring_overlay, ring_overlay_sx,
        },
        feedback::Loader,
        form::{
            PreparedField, SelectionArgs, Setter, clear_button, field_control_sx, removable_chip,
            use_bound, use_chip_announcer, use_field, use_field_frame,
        },
        layout::{BoxStyle, use_box},
    },
    hooks::{
        ElementHandle, LocalState, current_localization, id_selector, use_css, use_element,
        use_local_state, use_theme,
    },
    localization::fill,
    platform::{self, ElementApi, nested_interactive},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        FILE_FIELD_DROPZONE_HEIGHT, FILE_FIELD_PADDING, FILE_FIELD_RADIUS, FileFieldDefaults,
        FileFieldVariant, Size,
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

/// The `Input` variant's group, the `Select` trigger's shape: the chips, then
/// the Browse button over the rest of the line.
static FILE_CONTROL_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .display("flex")
        .align_items("center")
        // The frame's height, not its contents': an empty placeholder left a
        // 0px control that no click or drop could reach (todo 520).
        .align_self("stretch")
        // Lets the control shrink inside the frame, which is what makes the
        // value slot's own ellipsis take effect instead of the frame growing.
        .min_width("0")
        .gap("4px")
        .cursor("pointer")
        .user_select("none")
        .selector(
            "& > [data-slot='value']",
            sx().display("flex")
                .flex("0 1 auto")
                .min_width("0")
                .gap("4px")
                .margin("0")
                .padding("0")
                .list_style("none")
                .overflow("hidden"),
        )
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
        // The ring goes on what was drawn, not on the item, so it follows that
        // element's own radius.
        .selector("& [data-slot='chip']:focus-visible", sx().outline("none"))
        .selector("& [data-slot='chip']:focus-visible > *", focus_ring_sx())
        .when(
            "multiple",
            sx().selector(
                "& > [data-slot='value']",
                // Chips wrap, and the single-line clip would cut the second
                // row off at the slot's edge.
                sx().flex_wrap("wrap").overflow("visible"),
            ),
        )
        .when("disabled", sx().cursor("not-allowed"))
});

/// The `Input` variant's Browse button: the rest of the line, showing the
/// placeholder while nothing is picked. The frame draws its ring.
static FILE_BROWSE_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .display("flex")
        .align_items("center")
        .align_self("stretch")
        .flex("1 1 0")
        // Room to aim at beside a full row of chips.
        .min_width("2em")
        .text_align("start")
        .cursor("pointer")
        .selector(
            "& [data-placeholder]",
            sx().color("text-dimmed")
                .min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
        .selector("&:disabled", sx().cursor("not-allowed"))
});

/// The dropzone's Browse button: the whole surface inside its padding.
static FILE_DROPZONE_BROWSE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .align_items("center")
        .justify_content("center")
        .gap("4px")
        .flex("1 1 auto")
        .align_self("stretch")
        .border("none")
        .outline("none")
        .background("transparent")
        .padding("0")
        .color("inherit")
        .font_family("inherit")
        .font_size("inherit")
        .text_align("center")
        .cursor("pointer")
        .selector("&:disabled", sx().cursor("not-allowed"))
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
        // 3:1 on the page, as a field frame (WCAG 1.4.11, todo 490).
        .border_color("muted.6")
        .border_radius(FILE_FIELD_RADIUS.value())
        .background("transparent")
        .cursor("pointer")
        .text_align("center")
        .transition("border-color 150ms, background 150ms")
        // The Browse button's ring is the surface's, out by the dashed border.
        .position("relative")
        .selector(
            "& > [data-ring]",
            ring_overlay_sx()
                .inset("-2px")
                .border_radius(FILE_FIELD_RADIUS.value()),
        )
        .selector("& :focus-visible ~ [data-ring]", focus_ring_sx())
        .selector("& [data-slot='hint']", sx().color("text-dimmed"))
        // Sized against the text, which is the one thing on the surface that
        // already scales.
        .selector(
            "& [data-slot='browse'] > svg",
            sx().width("2em").height("2em").color("muted.6"),
        )
        .when(
            "dragging",
            sx().border_color("primary")
                .background("color-mix(in srgb, var(--lsx-primary) 8%, transparent)"),
        )
        .when(
            "disabled",
            sx().cursor("not-allowed").border_color("muted.3"),
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
        .border_color("muted.3")
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
            sx().flex("0 0 auto")
                .color("text-dimmed")
                .font_size("0.85em"),
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
        /// An upload is in flight. Draws a `Loader` in the control - beside
        /// the selection in the `Input` variant, in place of the icon on a
        /// dropzone, on the card once a single-file dropzone has put its
        /// surface away - and marks the field's group `aria-busy`.
        ///
        /// It blocks nothing: a `multiple` field can take more files while
        /// the first ones upload. `disabled` is the switch for that.
        #[props(default)]
        loading: Option<bool>,
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
    // Where the focus goes when a row is removed: the row that took its
    // place, or the Browse button when none is left.
    let browse_element = use_element();
    // The chips or the cards, whichever the variant draws.
    let list_element = use_element();
    // What the input itself holds, and *which* input held it. A pick fills
    // the list; every other edit happens in Rust, and `use_input_mirror`
    // writes the difference back. A `Signal` rather than a `use_local_state`,
    // which needs `Copy`.
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
    // Two gates, because `interactive` also drives the tab stop and the
    // input's `disabled`, and a read-only field keeps both - it is reached
    // and it posts. `editable` is what refuses the picker, the drop, the
    // clear button and every remove.
    let editable = interactive && !props.readonly.unwrap_or(false);

    if props.onchange.is_none() && !bound.is_bound() && !disabled {
        warn("FileField: `value` without `onchange` can never change.");
    }

    let value = bound.value().unwrap_or_else(|| props.value.clone());
    let announcer = use_chip_announcer(value.iter().map(FileData::name).collect());
    let dragging = use_local_state(|| false);

    // A drop bypasses the picker, which is the only place `accept` applies by
    // itself - so the component applies it, and says so rather than dropping
    // files in silence.
    let cards = variant == FileFieldVariant::Dropzone;
    // A single-file dropzone has nothing left to ask for once it holds its
    // file: the card replaces the surface, and removing the file brings it
    // back. A `multiple` one keeps taking files, so it keeps its surface.
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
    });

    use_input_mirror(input_element, mirrored, value.clone());

    // The input's own picker, or the system dialog where the input opens none.
    let accept = props.accept.clone().unwrap_or_default();
    let open = use_callback(move |()| {
        platform::pick_files(
            &accept,
            multiple,
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
        icon_size: ThemeAwareValue::Size(size).into(),
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

    match variant {
        // Each variant draws its control in a scope of its own: the two
        // prepare different boxes, and hooks in one scope are positional, so
        // a `variant` switch must remount rather than reuse the other's slots.
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
    }
}

/// The files a pick or a drop leaves once `accept` and `multiple` have had
/// their say. A drop bypasses the picker, which is the only place `accept`
/// applies by itself - so the component applies it, and says so rather than
/// dropping files in silence.
fn keep_accepted(files: Vec<FileData>, accept: &str, multiple: bool) -> Files {
    let picked = files.len();
    let kept: Files = files
        .into_iter()
        .filter(|file| accepts(accept, &file.name(), file.content_type().as_deref()))
        .collect::<Files>()
        .truncated(multiple);
    if kept.len() < picked {
        warn("FileField: dropped files that `accept` or `multiple` excludes.");
    }
    kept
}

/// The caller's `sx` behind a full width. The wrapper needs a **definite**
/// width, not merely `min-width: 0`: a stretched flex item is floored at its
/// own min-content width, and a long filename is one unbreakable word.
/// Measured in the browser against every other candidate - `min-width: 0` up
/// the chain, `overflow: hidden`, `contain: inline-size`, a breakable filename
/// - and this is the only one that keeps the frame inside its parent.
fn full_width(caller: &Input<Sx>) -> Input<Sx> {
    match caller.as_ref() {
        // The caller's own declarations come second, so they still win.
        Some(caller) => sx().width("100%").and(caller.clone()).into(),
        None => sx().width("100%").into(),
    }
}

/// The frame's trailing slot: the clear button, with the loader ahead of it -
/// or the loader alone when there is nothing to clear. The loader is silent:
/// it sits beside a group that the field's label already names, and
/// `aria-busy` on that group is what says it is waiting.
fn trailing_slot(
    loader: Option<Size>,
    clearable: bool,
    size: Size,
    browse_element: ElementHandle,
    emit: Callback<Files>,
) -> Option<Element> {
    let spinner = loader.map(|size| rsx! { Loader { size } });
    let clear = clear_button(clearable, size, browse_element, move |event: MouseEvent| {
        // Clearing is not a click on the control, which would open the picker
        // straight after emptying the field.
        event.stop_propagation();
        emit.call(Files::default());
    })
    .map(|button| {
        rsx! {
            {spinner.clone()}
            {button}
        }
    });
    clear.or(spinner)
}

/// The dropzone's cards, a list under the surface.
fn card_list(
    style: BoxStyle,
    list_element: ElementHandle,
    labelledby: Option<String>,
    surface: bool,
    loading: bool,
    drawn: Vec<Element>,
) -> Element {
    style
        // With no surface left, the list is what the field's label names.
        .attr(
            "aria-labelledby",
            (!surface).then_some(labelledby).flatten(),
        )
        .attr("aria-busy", (loading && !surface).then_some("true"))
        .element(&list_element)
        // Safari with VoiceOver drops list semantics from a `list-style: none`
        // list.
        .attr("role", "list")
        .render(HtmlTag::Ul, Vec::new(), drawn)
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

/// The input's `FileList` is the only thing a form posts, and it cannot be
/// edited - so whenever the caller's value and the input disagree (a removal,
/// a clear, a drop), the list is written back.
fn use_input_mirror(
    input_element: ElementHandle,
    mut mirrored: Signal<(Option<usize>, Files)>,
    synced: Files,
) {
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
}

/// Removing a row destroys the element the keyboard was on, and focus would
/// otherwise fall to the body. It moves to the row that took this one's
/// place, to the new last row when the removed one was last, and to the
/// Browse button when the list is empty. `focus_prefix` plus a row index is
/// that row's tab stop's id: a card's x, or a chip.
fn use_focus_debt(
    mut owed: Signal<Option<FocusDebt>>,
    remaining: usize,
    surface_survives: bool,
    list_element: ElementHandle,
    browse_element: ElementHandle,
    focus_prefix: String,
) {
    use_effect(use_reactive!(|remaining| {
        let Some(debt) = *owed.peek() else {
            return;
        };
        owed.set(None);
        let target = match debt {
            // Nothing left to remove: the Browse button is what is left.
            FocusDebt::Removed(_) if remaining == 0 => None,
            // The row that took this one's place, clamped to the new last.
            FocusDebt::Removed(index) => Some(index.min(remaining - 1)),
            // A single-file dropzone puts its surface away once it holds a
            // file, so the x that replaced it is what the keyboard needs.
            FocusDebt::Took if !surface_survives && remaining > 0 => Some(0),
            // The Browse button is still there, and still focused.
            FocusDebt::Took => return,
        };
        focus_row(list_element, browse_element, &focus_prefix, target);
    }));
}

/// Focuses row `target` by its tab stop's id under `list`, or the Browse
/// button for `None`.
fn focus_row(list: ElementHandle, browse: ElementHandle, prefix: &str, target: Option<usize>) {
    match target {
        Some(index) => {
            if let Ok(row) = list.query_selector(&id_selector(&format!("{prefix}-{index}"))) {
                let _ = row.focus();
            }
        }
        None => {
            let _ = browse.focus();
        }
    }
}

/// Draws the picked files, for one render: chips in the `Input` variant,
/// cards under a dropzone, or the caller's `selection`.
struct FileRows {
    draw: Option<Callback<SelectionArgs<FileData>, Element>>,
    remove_at: Callback<usize>,
    cards: bool,
    multiple: bool,
    /// Draws the x at all. Off while disabled or read-only.
    editable: bool,
    icon_size: Input<ThemeAwareValue>,
    /// The field's; the chips step down from it themselves.
    size: Size,
    /// The loader a card carries, once no surface is left to carry it.
    card_loader: Option<Size>,
    field_id: String,
    /// The chip that holds the list's one tab stop.
    chip_cursor: Option<usize>,
    keys: ChipKeys,
    chip_class: Option<String>,
    card_class: Option<String>,
}

impl FileRows {
    fn all(&self, files: &Files) -> Vec<Element> {
        files
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, file)| self.row(index, file))
            .collect()
    }

    fn row(&self, index: usize, file: FileData) -> Element {
        let remove_at = self.remove_at;
        let remove = Callback::new(move |_: ()| remove_at.call(index));
        let content = match &self.draw {
            Some(draw) => draw.call(SelectionArgs {
                value: file.clone(),
                remove,
            }),
            None => match self.cards {
                true => default_card(
                    &file,
                    remove,
                    self.icon_size.clone(),
                    self.editable,
                    self.card_loader,
                    format!("{}-remove-{index}", self.field_id),
                ),
                false => default_chip(&file, remove, self.size, self.multiple, self.editable),
            },
        };
        match self.cards {
            true => rsx! {
                li { key: "{index}", class: self.card_class.clone(), {content} }
            },
            false => {
                let keys = self.keys.clone();
                let mut cursor = keys.cursor;
                let id = format!("{}-{index}", keys.id_prefix);
                // One tab stop for the whole list; the arrows walk the rest.
                let stop =
                    keys.interactive
                        .then_some(match self.chip_cursor.unwrap_or(0) == index {
                            true => "0",
                            false => "-1",
                        });
                rsx! {
                    li {
                        key: "{index}",
                        class: self.chip_class.clone(),
                        "data-slot": "chip",
                        id,
                        tabindex: stop,
                        // Keeps a press on an x (a caller's too) from taking
                        // the focus the removal hands on.
                        onmousedown: move |event: MouseEvent| event.prevent_default(),
                        onfocus: move |_| cursor.set(Some(index)),
                        onkeydown: move |event: KeyboardEvent| keys.handle(event, Some(index)),
                        {content}
                    }
                }
            }
        }
    }
}

/// The `Input` variant's chips, a list inside the group. `None` while empty.
fn chip_list(drawn: Vec<Element>, list_element: ElementHandle) -> Option<Element> {
    (!drawn.is_empty()).then(|| {
        rsx! {
            // Safari with VoiceOver drops list semantics from a
            // `list-style: none` list.
            ul {
                "data-slot": "value",
                role: "list",
                onmounted: list_element.mount(),
                {drawn.into_iter()}
            }
        }
    })
}

/// The `Input` variant's Browse button: the placeholder while nothing is
/// picked, and the hidden text that says what the button does.
fn browse_content(placeholder: &str, empty: bool) -> Element {
    let browse = current_localization().file_field.browse;
    rsx! {
        if empty && !placeholder.is_empty() {
            span { "data-placeholder": "true", "{placeholder}" }
        }
        VisuallyHidden { "{browse}" }
    }
}

/// What the surface holds: an icon, or `loader` while an upload is in flight,
/// then the prompt. `children` when the caller wrote one, `placeholder` next,
/// and the localization's last - the shape `Dialog`'s `close_label` set. Then
/// a hint read off `accept` rather than written beside it, so the prompt
/// cannot claim something the picker would refuse.
fn dropzone_prompt(props: &FileFieldProps, loader: Option<Size>) -> Element {
    let written = props
        .children
        .as_ref()
        .is_ok_and(|children| *children != VNode::default());
    let prompt = match (written, props.placeholder.clone()) {
        (true, _) => props.children.clone(),
        (false, Some(placeholder)) => rsx! { span { "{placeholder}" } },
        (false, None) => {
            let labels = current_localization().file_field;
            let text = match props.multiple {
                true => labels.drop_files,
                false => labels.drop_file,
            };
            rsx! { span { "{text}" } }
        }
    };
    let hint = accept_hint(props.accept.as_deref().unwrap_or_default()).map(|hint| {
        rsx! {
            span { "data-slot": "hint", "{hint}" }
        }
    });
    rsx! {
        if let Some(size) = loader {
            Loader { size }
        } else {
            UploadIcon {}
        }
        {prompt}
        {hint}
    }
}

/// The chip the keyboard is on, of `count`. Removing the chip under the
/// cursor leaves the index pointing at the one that took its place, and past
/// the end it clamps. A dropzone has no chips, so no cursor.
fn chip_cursor(cursor: Option<usize>, count: usize, cards: bool) -> Option<usize> {
    match count {
        _ if cards => None,
        0 => None,
        count => cursor.map(|index| index.min(count - 1)),
    }
}

/// The `Input` variant's keys, on each chip and on the Browse button: the
/// arrows move the focus along the chips, Backspace and Delete remove.
#[derive(Clone)]
struct ChipKeys {
    interactive: bool,
    /// The keys that remove. The arrows only read, so a read-only field still
    /// walks the chips.
    editable: bool,
    count: usize,
    /// Removes a chip the focus is on, and owes the focus a new place.
    remove_at: Callback<usize>,
    /// Removes a chip from the Browse button, where the focus stays.
    drop_at: Callback<usize>,
    list: ElementHandle,
    browse: ElementHandle,
    id_prefix: String,
    /// The chip that last had the focus, and so holds the tab stop.
    cursor: Signal<Option<usize>>,
}

impl ChipKeys {
    /// `at` is the chip the focus is on, `None` for the Browse button.
    fn handle(&self, event: KeyboardEvent, at: Option<usize>) {
        // A chord is the browser's (Alt+ArrowLeft is Back).
        if !self.interactive || self.count == 0 || navigation_chord(&event).is_some() {
            return;
        }
        let last = self.count - 1;
        let target = match (event.key(), at) {
            (Key::ArrowLeft, Some(index)) => Some(index.saturating_sub(1)),
            // From the Browse button, the last chip - the one Backspace takes.
            (Key::ArrowLeft, None) => Some(last),
            // Past the last chip is the Browse button, not a wrap.
            (Key::ArrowRight, Some(index)) => (index < last).then_some(index + 1),
            (Key::Home, Some(_)) => Some(0),
            (Key::End, Some(_)) => Some(last),
            (Key::Backspace | Key::Delete, Some(index)) if self.editable => {
                event.prevent_default();
                self.remove_at.call(index);
                return;
            }
            (Key::Backspace | Key::Delete, None) if self.editable => {
                event.prevent_default();
                self.drop_at.call(last);
                return;
            }
            _ => return,
        };
        event.prevent_default();
        focus_row(self.list, self.browse, &self.id_prefix, target);
    }
}

/// What both variants draw: a group named by the label - the frame's control,
/// or the drop surface - holding the chips and a real Browse button. A click
/// anywhere on it opens the picker; a drop takes the files.
#[derive(Clone)]
struct Surface {
    browse: ElementHandle,
    /// The field's id, which the Browse button carries.
    id: String,
    labelledby: Option<String>,
    describedby: Option<String>,
    invalid: bool,
    required: bool,
    interactive: bool,
    editable: bool,
    loading: bool,
    open: Callback<()>,
    /// The chips' keys, from the Browse button. `None` on a dropzone.
    keys: Option<ChipKeys>,
    take: Callback<Vec<FileData>>,
    dragging: LocalState<bool>,
    /// Set while the button clicks its input.
    opening: Rc<Cell<bool>>,
    attributes: Vec<Attribute>,
}

impl Surface {
    /// The group's own drop, for the frame around it: the padding takes a file
    /// too (todo 531). `None` when disabled, as the group's dragover is.
    fn frame_drop(&self) -> Option<impl Fn(DragEvent) + 'static> {
        let (take, dragging, editable) = (self.take, self.dragging.clone(), self.editable);
        self.interactive.then_some(move |event: DragEvent| {
            event.prevent_default();
            dragging.set(false);
            if editable {
                take.call(event.files());
            }
        })
    }

    /// `chips` come before the button, `after` after its ring.
    fn render(
        self,
        group: BoxStyle,
        browse: BoxStyle,
        chips: Option<Element>,
        content: Element,
        after: Option<Element>,
    ) -> Element {
        let Surface {
            browse: element,
            id,
            labelledby,
            describedby,
            invalid,
            required,
            interactive,
            editable,
            loading,
            open,
            keys,
            take,
            dragging,
            opening,
            attributes,
        } = self;
        let required_id = format!("{id}-required");
        // Neither a group nor a button may carry `aria-required`.
        let required_label = current_localization().file_field.required;
        // The label, then the button's own text: "Receipt, Browse files".
        let named = labelledby.as_ref().map(|label| format!("{label} {id}"));
        let described = [
            required.then_some(required_id.as_str()),
            describedby.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
        let press = move || {
            // A press on the input opens it natively; its own click bubbling
            // back here while `opening` is not a second press.
            if !editable || opening.get() {
                return;
            }
            let opening = opening.clone();
            // After this dispatch: a synchronous click would re-enter this listener.
            spawn(async move {
                opening.set(true);
                open.call(());
                opening.set(false);
            });
        };
        let button = browse
            .element(&element)
            .attr_default("type", "button")
            .attr("id", id)
            // What a click on the label looks for.
            .attr("tabindex", "0")
            .attr("data-slot", "browse")
            .attr("aria-labelledby", named)
            .attr(
                "aria-describedby",
                (!described.is_empty()).then_some(described),
            )
            .attr("aria-invalid", invalid.then_some("true"))
            // Read-only stays a tab stop that refuses; disabled leaves the order.
            .attr(
                "aria-disabled",
                (interactive && !editable).then_some("true"),
            )
            .attr("disabled", !interactive)
            .event("onclick", {
                let press = press.clone();
                move |_: MouseEvent| press()
            })
            .event("onkeydown", move |event: KeyboardEvent| {
                if let Some(keys) = &keys {
                    keys.handle(event, None);
                }
            })
            .render(HtmlTag::Button, attributes, content);
        let dragging_over = dragging.clone();
        let dragging_off = dragging.clone();
        group
            .attr("role", "group")
            .attr("aria-labelledby", labelledby)
            .attr("aria-busy", loading.then_some("true"))
            .event("onclick", move |event: MouseEvent| {
                // The button, a chip and its x answer their own clicks.
                if !nested_interactive(&event, "[role=group]") {
                    press();
                }
            })
            .event("ondragover", move |event: DragEvent| {
                if interactive {
                    // Without this the browser opens the file instead - which
                    // on a read-only field would navigate away from the form
                    // being reviewed, so it is taken and then refused below.
                    event.prevent_default();
                    // `dragover` fires every few ms; `set` re-renders even on an equal value.
                    if dragging_over.get() != editable {
                        dragging_over.set(editable);
                    }
                }
            })
            .event("ondragleave", move |_: DragEvent| dragging_off.set(false))
            .event("ondrop", move |event: DragEvent| {
                event.prevent_default();
                dragging.set(false);
                if editable {
                    take.call(event.files());
                }
            })
            .render(
                HtmlTag::Div,
                Vec::new(),
                rsx! {
                    {chips}
                    {button}
                    {ring_overlay()}
                    if required {
                        span { id: required_id, hidden: true, "{required_label}" }
                    }
                    {after}
                },
            )
    }
}

// The variants' props carry `children`, which never compares equal, so they
// redraw with `FileField` whatever this says.
impl PartialEq for Surface {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

/// The `Input` variant's group inside its frame. `children` is the Browse
/// button's content.
#[component]
fn FileInputControl(
    control: Surface,
    trailing: Option<Element>,
    frame_states: Input<States>,
    states: Input<States>,
    chips: Option<Element>,
    input: Element,
    children: Element,
) -> Element {
    let frame = use_field_frame()
        .trailing(&trailing)
        .states(&frame_states)
        .ondrop(control.frame_drop())
        .prepare();
    // The frame draws the ring, so neither box may draw a second.
    let group = use_box()
        .framework_sx(&FILE_CONTROL_SX)
        .focus_ring(false)
        .states(&states)
        .prepare();
    let browse = use_box()
        .framework_sx(&FILE_BROWSE_SX)
        .focus_ring(false)
        .prepare();
    frame.render(control.render(group, browse, chips, children, Some(input)))
}

/// The `Dropzone` variant's surface. `shown` is false once a single-file
/// dropzone holds its file; the boxes are prepared either way.
#[component]
fn FileDropzoneControl(
    control: Surface,
    states: Input<States>,
    shown: bool,
    children: Element,
) -> Element {
    // The surface draws the ring, around the whole of itself.
    let group = use_box()
        .framework_sx(&FILE_DROPZONE_SX)
        .focus_ring(false)
        .states(&states)
        .prepare();
    let browse = use_box()
        .framework_sx(&FILE_DROPZONE_BROWSE_SX)
        .focus_ring(false)
        .prepare();
    match shown {
        true => control.render(group, browse, None, children, None),
        false => rsx! {},
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
    editable: bool,
) -> Element {
    let name = file.name();
    if !multiple {
        // `data-slot` so the control can clip it: one long filename would
        // otherwise push the frame past its parent's width.
        return rsx! {
            span { "data-slot": "name", "{name}" }
        };
    }
    // The chip `MultiSelect` and `TagsField` draw. Its press guard keeps the
    // focus on the control, so a mouse removal needs no repair.
    removable_chip(name, remove, size, !editable)
}

/// One picked file as a row under the dropzone: the name, its size, and an x.
/// The card is outside the control, so its remove button is an ordinary tab
/// stop rather than something the control's own keyboard has to reach.
fn default_card(
    file: &FileData,
    remove: Callback<()>,
    icon_size: Input<ThemeAwareValue>,
    editable: bool,
    loading: Option<Size>,
    id: String,
) -> Element {
    let name = file.name();
    let size = format_size(file.size());
    let remove_label = fill(current_localization().common.remove, &[("label", &name)]);
    rsx! {
        span { "data-slot": "name", "{name}" }
        span { "data-slot": "size", "{size}" }
        if let Some(size) = loading {
            Loader { size }
        }
        if editable {
            span { "data-slot": "remove",
                ActionIcon {
                    // The id is how the focus finds the row that takes this
                    // one's place: a list cannot hold a hook per row.
                    id,
                    aria_label: remove_label,
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

/// What the `Input` variant draws inside its group.
struct Chips {
    trailing: Option<Element>,
    drawn: Vec<Element>,
    list_element: ElementHandle,
    placeholder: String,
}

/// The `Input` variant: the files are chips inside the field's own group,
/// beside the Browse button, and the native input sits in there too.
fn file_input_variant(
    field: PreparedField,
    control: Surface,
    states: Input<States>,
    chips: Chips,
    input: Element,
    announcer: Element,
) -> Element {
    let Chips {
        trailing,
        drawn,
        list_element,
        placeholder,
    } = chips;
    let browse = browse_content(&placeholder, drawn.is_empty());
    let chips = chip_list(drawn, list_element);
    let frame_states = field.states().clone();

    field.render(rsx! {
        FileInputControl {
            control,
            trailing,
            frame_states,
            states,
            chips,
            input,
            {browse}
        }
        {announcer}
    })
}

/// What the dropzone draws under its surface.
struct Cards {
    prompt: Element,
    drawn: Vec<Element>,
    style: BoxStyle,
    list_element: ElementHandle,
    labelledby: Option<String>,
    surface: bool,
    loading: bool,
    has_files: bool,
}

/// The `Dropzone` variant. The cards sit under the surface, not in it: a
/// dropzone that grows with its own contents stops being a target to aim at.
/// The input stays mounted either way - it is what posts.
fn file_dropzone_variant(
    field: PreparedField,
    control: Surface,
    states: Input<States>,
    cards: Cards,
    input: Element,
    announcer: Element,
) -> Element {
    let Cards {
        prompt,
        drawn,
        style,
        list_element,
        labelledby,
        surface,
        loading,
        has_files,
    } = cards;

    let drop_target = rsx! {
        FileDropzoneControl { control, states, shown: surface, {prompt} }
    };
    let card_list =
        has_files.then(|| card_list(style, list_element, labelledby, surface, loading, drawn));

    field.render(rsx! {
        {drop_target}
        {card_list}
        {input}
        {announcer}
    })
}

/// What the two paths that add files need. `accept` applies to a drop as well
/// as to the picker, which is the only place the browser applies it itself.
struct Taking {
    onchange: Option<EventHandler<Files>>,
    setter: Option<Setter<Files>>,
    accept: String,
    multiple: bool,
    editable: bool,
}

/// The one writer, and the focus debt every edit through it leaves.
#[derive(Clone, Copy)]
struct Intake {
    emit: Callback<Files>,
    /// What the next render owes the keyboard. A removal destroys the button
    /// focus was on; a pick that stands the surface down destroys the
    /// *surface* focus was on. Either way focus would fall to the body.
    owed: Signal<Option<FocusDebt>>,
    take: Callback<Vec<FileData>>,
}

fn use_file_intake(taking: Taking) -> Intake {
    let Taking {
        onchange,
        setter,
        accept,
        multiple,
        editable,
    } = taking;

    let emit = use_callback(move |files: Files| match (&onchange, &setter) {
        (Some(onchange), _) => onchange.call(files),
        (None, Some(setter)) => setter.set(files),
        (None, None) => {}
    });

    let mut owed = use_signal(|| None::<FocusDebt>);

    let take = use_callback(move |files: Vec<FileData>| {
        if !editable {
            return;
        }
        let kept = keep_accepted(files, &accept, multiple);
        if !kept.is_empty() {
            owed.set(Some(FocusDebt::Took));
            emit.call(kept);
        }
    });

    Intake { emit, owed, take }
}

/// Which chip the keyboard is on. Only the `Input` variant has one: the
/// dropzone's cards sit outside the control, so their remove buttons are
/// ordinary tab stops and need no cursor at all.
fn use_chip_cursor(
    variant: FileFieldVariant,
    count: usize,
    cards: bool,
) -> (Signal<Option<usize>>, Option<usize>) {
    let mut cursor = use_signal(|| None::<usize>);
    // A cursor set in the `Input` variant means nothing to the dropzone, and
    // coming back must not show a chip the keyboard never picked there. The
    // dropzone ignores it above, so clearing it one render late is unseen.
    use_effect(use_reactive!(|(variant,)| {
        let _ = variant;
        if cursor.peek().is_some() {
            cursor.set(None);
        }
    }));

    (cursor, chip_cursor(cursor(), count, cards))
}

#[cfg(test)]
mod tests {
    use super::chip_cursor;

    #[test]
    fn the_cursor_clamps_to_the_chips_left() {
        assert_eq!(chip_cursor(Some(1), 3, false), Some(1));
        assert_eq!(chip_cursor(Some(4), 3, false), Some(2));
        assert_eq!(chip_cursor(Some(0), 0, false), None);
        assert_eq!(chip_cursor(None, 3, false), None);
    }

    /// Todo 245: a cursor left over from the `Input` variant named a chip id
    /// the dropzone never draws, through `aria-activedescendant`.
    #[test]
    fn a_dropzone_has_no_cursor_whatever_is_left_over() {
        assert_eq!(chip_cursor(Some(1), 3, true), None);
    }
}
