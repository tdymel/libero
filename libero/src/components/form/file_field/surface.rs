use std::{cell::Cell, rc::Rc};

use dioxus::dioxus_core::AttributeValue;
use dioxus::html::{FileData, HasFileData};
use dioxus::prelude::*;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::{HtmlTag, Input, Part, States, ring_overlay},
        feedback::Loader,
        form::{PreparedField, clear_button, use_field_frame},
        layout::{BoxStyle, use_box},
    },
    hooks::{ElementHandle, LocalState, current_localization},
    platform::nested_interactive,
    theme::Size,
};

use super::file_field::FileFieldPart;
use super::files::Files;
use super::rows::{ChipKeys, StandIn, card_list, chip_list};
use super::styles::{FILE_BROWSE_SX, FILE_CONTROL_SX, FILE_DROPZONE_BROWSE_SX, FILE_DROPZONE_SX};

/// What both variants draw: a labelled group with the chips and a Browse
/// button. A click opens the picker; a drop takes the files.
#[derive(Clone)]
pub(super) struct Surface {
    pub(super) browse: ElementHandle,
    /// The field's id, which the Browse button carries.
    pub(super) id: String,
    pub(super) labelledby: Option<String>,
    pub(super) describedby: Option<String>,
    pub(super) invalid: bool,
    pub(super) required: bool,
    pub(super) interactive: bool,
    pub(super) editable: bool,
    pub(super) loading: bool,
    pub(super) open: Callback<()>,
    /// The chips' keys, from the Browse button. `None` on a dropzone.
    pub(super) keys: Option<ChipKeys>,
    pub(super) take: Callback<Vec<FileData>>,
    pub(super) dragging: LocalState<bool>,
    /// Set while the button clicks its input.
    pub(super) opening: Rc<Cell<bool>>,
    pub(super) attributes: Vec<Attribute>,
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

    /// The frame's drag cue, for a drag over its padding as well. `None` when disabled.
    fn frame_drag(&self) -> Option<impl Fn(bool) + 'static> {
        let (dragging, editable) = (self.dragging.clone(), self.editable);
        self.interactive.then_some(move |over: bool| {
            let over = over && editable;
            if dragging.get() != over {
                dragging.set(over);
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
        // Without a label the caller's `aria_label` names the group as well as Browse.
        let group_name = attributes
            .iter()
            .filter(|_| labelledby.is_none())
            .find_map(|attribute| match &attribute.value {
                AttributeValue::Text(text) if attribute.name == "aria-label" => Some(text.clone()),
                _ => None,
            });
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
            .attr("data-slot", FileFieldPart::Browse.slot())
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
            .attr("data-slot", FileFieldPart::Control.slot())
            .attr("role", "group")
            .attr("aria-labelledby", labelledby)
            .attr("aria-label", group_name)
            .attr("aria-busy", loading.then_some("true"))
            .event("onclick", move |event: MouseEvent| {
                // The button, a chip and its x answer their own clicks.
                if !nested_interactive(&event, "[role=group]") {
                    press();
                }
            })
            .event("ondragover", move |event: DragEvent| {
                if interactive {
                    // Else the browser opens the file and leaves the form, so
                    // a read-only field takes the drop and refuses it.
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
        .ondrag(control.frame_drag())
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

/// What the `Input` variant draws inside its group.
pub(super) struct Chips {
    pub(super) trailing: Option<Element>,
    pub(super) drawn: Vec<Element>,
    pub(super) list_element: ElementHandle,
    pub(super) placeholder: String,
}

/// The `Input` variant: the files are chips inside the field's own group,
/// beside the Browse button, and the native input sits in there too.
pub(super) fn file_input_variant(
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
    let frame_states: Input<States> = field
        .states()
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("dragging", control.dragging.get())
        .into();

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
pub(super) struct Cards {
    pub(super) prompt: Element,
    pub(super) drawn: Vec<Element>,
    pub(super) style: BoxStyle,
    pub(super) list_element: ElementHandle,
    pub(super) labelledby: Option<String>,
    pub(super) describedby: Option<String>,
    pub(super) invalid: bool,
    pub(super) surface: bool,
    pub(super) loading: bool,
    pub(super) has_files: bool,
}

/// The `Dropzone` variant. The cards sit under the surface so it keeps its
/// size; the input stays mounted, since it posts.
pub(super) fn file_dropzone_variant(
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
        describedby,
        invalid,
        surface,
        loading,
        has_files,
    } = cards;

    let focusable = control.interactive && !control.editable;
    let drop_target = rsx! {
        FileDropzoneControl { control, states, shown: surface, {prompt} }
    };
    let stand_in = (!surface).then_some(StandIn {
        labelledby,
        describedby,
        invalid,
        loading,
        focusable,
    });
    let card_list = has_files.then(|| card_list(style, list_element, stand_in, drawn));

    field.render(rsx! {
        {drop_target}
        {card_list}
        {input}
        {announcer}
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

/// The frame's trailing slot: the loader, then the clear button. The loader is
/// silent; the group's `aria-busy` says it is waiting.
pub(super) fn trailing_slot(
    loader: Option<Size>,
    clearable: bool,
    size: Size,
    browse_element: ElementHandle,
    emit: Callback<Files>,
) -> Option<Element> {
    let spinner = loader.map(|size| rsx! { Loader { size } });
    // In the field's labelled group already, so "Clear" alone.
    let clear = clear_button(
        clearable,
        size,
        browse_element,
        None,
        move |event: MouseEvent| {
            // Clearing is not a click on the control, which would open the picker
            // straight after emptying the field.
            event.stop_propagation();
            emit.call(Files::default());
        },
    )
    .map(|button| {
        rsx! {
            {spinner.clone()}
            {button}
        }
    });
    clear.or(spinner)
}
