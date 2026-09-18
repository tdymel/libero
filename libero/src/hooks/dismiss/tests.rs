use std::rc::Rc;

use dioxus::core::{AttributeValue, ElementId, WriteMutations};
use dioxus::html::{HasKeyboardData, PlatformEventData};

use super::*;
use crate::{
    LiberoProvider,
    components::{HtmlTag, Modal, use_box},
    hooks::PopoverOptions,
    hooks::{ModalScope, use_element, use_modal, use_popover},
    use_theme,
};

/// Every element that registered a `keydown` listener, in the order the
/// renderer saw them - which is creation order, so outermost first.
#[derive(Default)]
struct FindKeydownListeners {
    last: Option<ElementId>,
    keydown: Vec<ElementId>,
    focusout: Vec<ElementId>,
    mounted: Vec<ElementId>,
    /// The element carrying `id="trigger"`, so a press can be dispatched at
    /// it even when it has no listener of its own - which is exactly the
    /// state `a_closed_box_does_not_swallow_escape_from_the_modal_around_it`
    /// is about.
    trigger: Option<ElementId>,
    /// The element carrying `id="dropdown"`, the field-dropdown stand-in.
    dropdown: Option<ElementId>,
    /// The first element carrying `role="combobox"`: a real `Select`'s
    /// trigger.
    combobox: Option<ElementId>,
}

impl WriteMutations for FindKeydownListeners {
    fn push_id(&mut self, id: ElementId) {
        self.last = Some(id);
    }
    fn set_id(&mut self, id: ElementId) {
        self.last = Some(id);
    }
    fn add_event_listener(&mut self, name: &str) {
        match (name, self.last) {
            ("keydown", Some(id)) => self.keydown.push(id),
            ("focusout", Some(id)) => self.focusout.push(id),
            ("mounted", Some(id)) => self.mounted.push(id),
            _ => {}
        }
    }
    fn child(&mut self, _index: usize) {}
    fn pop(&mut self) {}
    fn create_element(&mut self, _tag: &str, _ns: Option<&str>) {}
    fn create_text(&mut self, _value: &str) {}
    fn clone(&mut self) {}
    fn append_children(&mut self, _m: usize) {}
    fn replace_with(&mut self, _m: usize) {}
    fn insert_after(&mut self, _m: usize) {}
    fn insert_before(&mut self, _m: usize) {}
    fn set_attribute(&mut self, n: &str, _ns: Option<&str>, v: &AttributeValue) {
        if n == "id"
            && let AttributeValue::Text(value) = v
        {
            match value.as_str() {
                "trigger" => self.trigger = self.last,
                "dropdown" => self.dropdown = self.last,
                _ => {}
            }
        }
        if n == "role"
            && self.combobox.is_none()
            && let AttributeValue::Text(value) = v
            && value == "combobox"
        {
            self.combobox = self.last;
        }
    }
    fn set_text(&mut self, _value: &str) {}
    fn remove_event_listener(&mut self, _name: &str) {}
    fn remove(&mut self) {}
}

/// A stand-in for the renderer's key event: Escape, plain unless a test
/// asks for a held or a composing one - or ArrowDown, which is how a test
/// opens a real `Select`'s list.
#[derive(Clone, Copy, Default)]
struct FakeEscape {
    repeat: bool,
    composing: bool,
    arrow_down: bool,
}

impl HasKeyboardData for FakeEscape {
    fn key(&self) -> Key {
        match self.arrow_down {
            true => Key::ArrowDown,
            false => Key::Escape,
        }
    }
    fn code(&self) -> Code {
        match self.arrow_down {
            true => Code::ArrowDown,
            false => Code::Escape,
        }
    }
    fn location(&self) -> dioxus::html::input_data::keyboard_types::Location {
        dioxus::html::input_data::keyboard_types::Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        self.repeat
    }
    fn is_composing(&self) -> bool {
        self.composing
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl dioxus::html::point_interaction::ModifiersInteraction for FakeEscape {
    fn modifiers(&self) -> dioxus::html::keyboard_types::Modifiers {
        Default::default()
    }
}

struct EscapeConverter;

impl dioxus::html::HtmlEventConverter for EscapeConverter {
    fn convert_keyboard_data(&self, event: &PlatformEventData) -> dioxus::html::KeyboardData {
        let press = event.downcast::<FakeEscape>().copied().unwrap_or_default();
        dioxus::html::KeyboardData::new(press)
    }
    fn convert_animation_data(&self, _e: &PlatformEventData) -> dioxus::html::AnimationData {
        unimplemented!()
    }
    fn convert_before_input_data(&self, _e: &PlatformEventData) -> dioxus::html::BeforeInputData {
        unimplemented!()
    }
    fn convert_cancel_data(&self, _e: &PlatformEventData) -> dioxus::html::CancelData {
        unimplemented!()
    }
    fn convert_clipboard_data(&self, _e: &PlatformEventData) -> dioxus::html::ClipboardData {
        unimplemented!()
    }
    fn convert_composition_data(&self, _e: &PlatformEventData) -> dioxus::html::CompositionData {
        unimplemented!()
    }
    fn convert_drag_data(&self, _e: &PlatformEventData) -> dioxus::html::DragData {
        unimplemented!()
    }
    fn convert_focus_data(&self, _e: &PlatformEventData) -> dioxus::html::FocusData {
        dioxus::html::FocusData::new(FakeFocus)
    }
    fn convert_form_data(&self, _e: &PlatformEventData) -> dioxus::html::FormData {
        unimplemented!()
    }
    fn convert_image_data(&self, _e: &PlatformEventData) -> dioxus::html::ImageData {
        unimplemented!()
    }
    fn convert_media_data(&self, _e: &PlatformEventData) -> dioxus::html::MediaData {
        unimplemented!()
    }
    /// `()` backs no richer handle, so this is the mounted floor: every
    /// focus-containment question answers `Unsupported`.
    fn convert_mounted_data(&self, _e: &PlatformEventData) -> dioxus::html::MountedData {
        dioxus::html::MountedData::new(())
    }
    fn convert_mouse_data(&self, _e: &PlatformEventData) -> dioxus::html::MouseData {
        unimplemented!()
    }
    fn convert_pointer_data(&self, _e: &PlatformEventData) -> dioxus::html::PointerData {
        unimplemented!()
    }
    fn convert_resize_data(&self, _e: &PlatformEventData) -> dioxus::html::ResizeData {
        unimplemented!()
    }
    fn convert_scroll_data(&self, _e: &PlatformEventData) -> dioxus::html::ScrollData {
        unimplemented!()
    }
    fn convert_selection_data(&self, _e: &PlatformEventData) -> dioxus::html::SelectionData {
        unimplemented!()
    }
    fn convert_toggle_data(&self, _e: &PlatformEventData) -> dioxus::html::ToggleData {
        unimplemented!()
    }
    fn convert_touch_data(&self, _e: &PlatformEventData) -> dioxus::html::TouchData {
        unimplemented!()
    }
    fn convert_transition_data(&self, _e: &PlatformEventData) -> dioxus::html::TransitionData {
        unimplemented!()
    }
    fn convert_visible_data(&self, _e: &PlatformEventData) -> dioxus::html::VisibleData {
        unimplemented!()
    }
    fn convert_wheel_data(&self, _e: &PlatformEventData) -> dioxus::html::WheelData {
        unimplemented!()
    }
}

/// The focus payload carries nothing the hook reads.
struct FakeFocus;

impl dioxus::html::HasFocusData for FakeFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn escape() -> Rc<dyn std::any::Any> {
    Rc::new(PlatformEventData::new(Box::new(FakeEscape::default())))
}

/// Just the marker line the app prints, so a failure message is readable -
/// the rendered document is mostly the theme stylesheet. Cut at the next
/// tag, since a portal's outlet can render after it.
fn state(dom: &VirtualDom) -> String {
    let html = dioxus_ssr::render(dom);
    // The space keeps a `--lsx-*-on-state:` declaration from matching.
    let at = html.rfind("state: ").expect("the state marker");
    let marker = &html[at..];
    marker[..marker.find('<').unwrap_or(marker.len())].to_string()
}

/// Drains tasks and re-renders until nothing is left: a close travels
/// through a spawned task, then the consumer's signal, then the effect that
/// takes the layer off the stack.
fn settle(dom: &mut VirtualDom) {
    for _ in 0..4 {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
}

fn press(dom: &mut VirtualDom, target: ElementId) {
    dom.runtime()
        .handle_event("keydown", Event::new(escape(), true), target);
    settle(dom);
}

fn press_as(dom: &mut VirtualDom, target: ElementId, press: FakeEscape) {
    let data: Rc<dyn std::any::Any> = Rc::new(PlatformEventData::new(Box::new(press)));
    dom.runtime()
        .handle_event("keydown", Event::new(data, true), target);
    settle(dom);
}

const HELD: FakeEscape = FakeEscape {
    repeat: true,
    composing: false,
    arrow_down: false,
};
const COMPOSING: FakeEscape = FakeEscape {
    repeat: false,
    composing: true,
    arrow_down: false,
};
const ARROW_DOWN: FakeEscape = FakeEscape {
    repeat: false,
    composing: false,
    arrow_down: true,
};

/// How many `<div>`s are open at `at`, so two elements can be shown to be
/// siblings rather than nested. Nesting of other tags does not affect it.
fn div_depth_at(html: &str, at: usize) -> i32 {
    let mut depth = 0;
    let mut rest = &html[..at];
    while let Some(next) = rest.find("<div") {
        depth += 1;
        rest = &rest[next + 4..];
    }
    let mut closes = 0;
    let mut rest = &html[..at];
    while let Some(next) = rest.find("</div>") {
        closes += 1;
        rest = &rest[next + 6..];
    }
    depth - closes
}

/// Two modals, the inner opened while the outer was up. Both always push -
/// a modal's bubbled handler covers its whole subtree - so this is the
/// arbitration on every backend.
#[component]
fn TwoModals(outer: Signal<bool>, inner: Signal<bool>) -> Element {
    rsx! {
        if outer() {
            Modal { onclose: move |_| { let mut outer = outer; outer.set(false); },
                "outer body"
            }
        }
        if inner() {
            Modal { onclose: move |_| { let mut inner = inner; inner.set(false); },
                "inner body"
            }
        }
    }
}

fn two_modals() -> Element {
    let outer = use_signal(|| true);
    let inner = use_signal(|| true);

    rsx! {
        LiberoProvider { TwoModals { outer, inner } }
        "state: outer={outer} inner={inner}"
    }
}

#[test]
fn escape_acts_on_the_newest_modal_and_no_other() {
    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(two_modals);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);

    // Each modal registers one on its own root and one on its focus trap.
    assert_eq!(
        find.keydown.len(),
        4,
        "expected four keydown listeners, got {:?}",
        find.keydown
    );
    let outer_root = find.keydown[0];
    let inner_root = find.keydown[2];

    assert_eq!(state(&dom), "state: outer=true inner=true");

    // The older modal hears the press and declines: it is not the top.
    press(&mut dom, outer_root);
    assert_eq!(
        state(&dom),
        "state: outer=true inner=true",
        "a layer under the top one must not act"
    );

    press(&mut dom, inner_root);
    assert_eq!(state(&dom), "state: outer=true inner=false");

    // And with the newer one gone the older is top again, which is the
    // guard popping by identity rather than the stack being reset.
    press(&mut dom, outer_root);
    assert_eq!(state(&dom), "state: outer=false inner=false");
}

/// The discriminating test for popping by identity.
///
/// The older modal goes away on its own - a route change, a caller that
/// simply stops rendering it - so nothing runs a close handler and only the
/// guard's `Drop` takes it off the stack. It has to take *its own* entry. A
/// pop-the-last would take the newer modal's instead, and the newer modal
/// would then silently stop answering Escape with nothing to show for it.
#[test]
fn dropping_an_older_layer_leaves_the_newer_one_on_top() {
    /// Outside both modals, so its own Escape bubbles to the app root and
    /// through no layer's handler. A key rather than a click because the
    /// test converter only speaks keyboard.
    #[component]
    fn Closer(outer: Signal<bool>) -> Element {
        let style = use_box().prepare();
        style.render(
            HtmlTag::Div,
            vec![listener("onkeydown", move |_: Event<KeyboardData>| {
                let mut outer = outer;
                outer.set(false);
            })],
            rsx! {},
        )
    }

    fn app() -> Element {
        let outer = use_signal(|| true);
        let inner = use_signal(|| true);

        rsx! {
            LiberoProvider {
                Closer { outer }
                TwoModals { outer, inner }
            }
            "state: outer={outer} inner={inner}"
        }
    }

    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);

    // The closer, then each modal's root and focus trap.
    assert_eq!(
        find.keydown.len(),
        5,
        "expected five keydown listeners, got {:?}",
        find.keydown
    );
    let closer = find.keydown[0];
    let inner_root = find.keydown[3];

    press(&mut dom, closer);
    assert_eq!(state(&dom), "state: outer=false inner=true");

    press(&mut dom, inner_root);
    assert_eq!(
        state(&dom),
        "state: outer=false inner=false",
        "the newer layer should still have been on top"
    );
}

/// A dismissible region drawn *inside* the modal rather than portaled, so
/// an Escape on it bubbles through the modal's own handler. `outside` is
/// off because the focusout settle needs a platform, and `return_focus`
/// because there is no document to hand focus back to.
#[component]
fn Region(open: Signal<bool>) -> Element {
    let anchor = use_element();
    let floating = use_element();
    let close = use_callback(move |()| {
        let mut open = open;
        open.set(false);
    });
    let dismiss = use_dismiss(
        anchor,
        floating,
        open(),
        true,
        Some(close),
        DismissOptions {
            outside: false,
            return_focus: false,
            ..Default::default()
        },
    );
    let trigger = use_box().prepare();
    let style = use_box().prepare();

    // The trigger renders first, so the box is always the *last* keydown
    // listener and the trigger the one before it.
    rsx! {
        {
            trigger
                .element(&anchor)
                .attr("id", "trigger")
                .render(HtmlTag::Button, dismiss.anchor_events(), rsx! { "open" })
        }
        if open() {
            {style.element(&floating).render(HtmlTag::Div, dismiss.floating_events(), rsx! { "region" })}
        }
    }
}

fn region_in_modal() -> Element {
    let mut modal = use_signal(|| true);
    let region = use_signal(|| true);

    rsx! {
        LiberoProvider {
            if modal() {
                Modal { onclose: move |_| modal.set(false),
                    Region { open: region }
                }
            }
        }
        "state: modal={modal} region={region}"
    }
}

/// The layer consumes the press, so the `Modal` around it never hears it.
///
/// This is the assertion that discriminates. Before the bubble-phase
/// `stop_propagation`, both closed, and the doc comment called that "no
/// worse than what shipped" - true only because no non-portaled consumer
/// existed yet, which is a rationalisation rather than a reason. The old
/// test asserted only that the modal closed, so it passed either way and
/// made the defect look intentional.
#[test]
fn a_layers_own_handler_consumes_the_press_before_the_modal_hears_it() {
    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(region_in_modal);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);

    let region = *find.keydown.last().expect("no keydown listener");
    assert_eq!(state(&dom), "state: modal=true region=true");

    press(&mut dom, region);
    assert_eq!(
        state(&dom),
        "state: modal=true region=false",
        "the press should have stopped at the layer that consumed it"
    );
}

/// What `ComboboxCore`, `Cascader`, the date fields and `ColorField` do
/// with Escape: close the list from a handler inside the overlay, prevent
/// the default, and let the press bubble on. None of them is on the layer
/// stack, so only the prevented default can tell an enclosing overlay the
/// press was taken.
#[component]
fn FieldDropdown(open: Signal<bool>) -> Element {
    let style = use_box().prepare();
    style.attr("id", "dropdown").render(
        HtmlTag::Div,
        vec![listener("onkeydown", move |event: Event<KeyboardData>| {
            if event.key() == Key::Escape && open() {
                event.prevent_default();
                let mut open = open;
                open.set(false);
            }
        })],
        rsx! { "list" },
    )
}

fn dropdown_in_modal() -> Element {
    let mut modal = use_signal(|| true);
    let list = use_signal(|| true);

    rsx! {
        LiberoProvider {
            if modal() {
                Modal { onclose: move |_| modal.set(false),
                    FieldDropdown { open: list }
                }
            }
        }
        "state: modal={modal} list={list}"
    }
}

/// Todo 249, off the web: the list is not on the stack, so `is_top()`
/// answers yes for the modal, and before `key_taken` read the event's own
/// flag both closed on one press. The second press, with the list shut,
/// is the control: a dropdown that is not open takes nothing.
#[test]
fn a_field_dropdown_taking_escape_leaves_the_modal_open() {
    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(dropdown_in_modal);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);

    let dropdown = find.dropdown.expect("the dropdown stand-in");
    assert_eq!(state(&dom), "state: modal=true list=true");

    press(&mut dom, dropdown);
    assert_eq!(
        state(&dom),
        "state: modal=true list=false",
        "one Escape must close the list and not the modal around it"
    );

    press(&mut dom, dropdown);
    assert_eq!(state(&dom), "state: modal=false list=false");
}

#[component]
fn DropdownInWindow() -> Element {
    let list = use_signal(|| true);
    let window = crate::hooks::use_floating_window(
        crate::components::FloatingWindowOptions {
            title: Some("Inspector".into()),
            ..Default::default()
        },
        move |_| rsx! { FieldDropdown { open: list } },
    );
    use_hook(|| window.open());

    rsx! { "state: window={window.is_open()} list={list}" }
}

/// Todo 249's `FloatingWindow` half. The window closed on any Escape, so
/// with a `Select` open in it one press closed both, on every backend.
/// Same control as the modal's: once the list is shut, Escape reaches the
/// window.
#[test]
fn a_field_dropdown_taking_escape_leaves_the_window_open() {
    fn app() -> Element {
        rsx! { LiberoProvider { DropdownInWindow {} } }
    }

    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    // The window opens from a hook after the first render, so its markup
    // arrives in a later pass, and the finder has to see that one.
    for _ in 0..4 {
        dom.process_events();
        dom.render_immediate(&mut find);
    }

    let dropdown = find.dropdown.expect("the dropdown stand-in");
    assert_eq!(state(&dom), "state: window=true list=true");

    press(&mut dom, dropdown);
    assert_eq!(
        state(&dom),
        "state: window=true list=false",
        "one Escape must close the list and not the window around it"
    );

    press(&mut dom, dropdown);
    assert_eq!(state(&dom), "state: window=false list=false");
}

/// A `use_dismiss` box with a field dropdown in it: a `Select` in a
/// `HoverCard`'s content. Region's setup, with the stand-in as the box's
/// content.
#[component]
fn DropdownInRegion() -> Element {
    let mut open = use_signal(|| true);
    let list = use_signal(|| true);
    let anchor = use_element();
    let floating = use_element();
    let close = use_callback(move |()| open.set(false));
    let dismiss = use_dismiss(
        anchor,
        floating,
        open(),
        true,
        Some(close),
        DismissOptions {
            outside: false,
            return_focus: false,
            ..Default::default()
        },
    );
    let style = use_box().prepare();

    rsx! {
        if open() {
            {
                style
                    .element(&floating)
                    .render(HtmlTag::Div, dismiss.floating_events(), rsx! { FieldDropdown { open: list } })
            }
        }
        "state: region={open} list={list}"
    }
}

/// Todo 348, off the web: the box's element listener did not ask
/// `key_taken`, so the press the list took bubbled on and closed the box
/// too. The second press is the control: with the list shut, Escape
/// reaches the box. The web half, where the box hears Escape before the
/// field does, is `use_field_list_layer`, and this harness has no
/// keyboard capability to run it.
#[test]
fn a_field_dropdown_taking_escape_leaves_the_dismiss_box_open() {
    fn app() -> Element {
        rsx! { LiberoProvider { DropdownInRegion {} } }
    }

    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);

    let dropdown = find.dropdown.expect("the dropdown stand-in");
    assert_eq!(state(&dom), "state: region=true list=true");

    press(&mut dom, dropdown);
    assert_eq!(
        state(&dom),
        "state: region=true list=false",
        "one Escape must close the list and not the box around it"
    );

    press(&mut dom, dropdown);
    assert_eq!(state(&dom), "state: region=false list=false");
}

/// A real `Select`, for the regression tests below: the stand-in above
/// proves the rule, these prove the field that ships follows it.
#[component]
fn RealSelect() -> Element {
    let mut value = use_signal(|| None::<String>);
    rsx! {
        crate::components::Select::<String> {
            options: vec!["Apple".to_string(), "Banana".to_string()],
            value: value(),
            onchange: move |next| value.set(next),
        }
    }
}

/// Whether any combobox in the document says its list is open.
fn list_open(dom: &VirtualDom) -> bool {
    dioxus_ssr::render(dom).contains(r#"aria-expanded="true""#)
}

/// Opens the list with ArrowDown, then Escape twice: the first closes
/// only the list, the second the overlay. `overlay` reads the overlay's
/// open state from the marker.
fn escape_through_a_real_select(dom: &mut VirtualDom, find: &FindKeydownListeners, overlay: &str) {
    let combobox = find.combobox.expect("the Select's combobox");
    assert!(state(dom).contains(&format!("{overlay}=true")));
    assert!(!list_open(dom), "the list starts closed");

    press_as(dom, combobox, ARROW_DOWN);
    assert!(list_open(dom), "ArrowDown should open the list");

    press(dom, combobox);
    assert!(!list_open(dom), "the first Escape closes the list");
    assert!(
        state(dom).contains(&format!("{overlay}=true")),
        "the first Escape must leave the {overlay} open: {}",
        state(dom)
    );

    press(dom, combobox);
    assert!(
        state(dom).contains(&format!("{overlay}=false")),
        "the second Escape closes the {overlay}: {}",
        state(dom)
    );
}

/// Todo 348's regression guard for `Modal`: an open list on the stack
/// must not change what one Escape does in a modal.
#[test]
fn a_real_select_in_a_modal_takes_the_first_escape() {
    fn app() -> Element {
        let mut modal = use_signal(|| true);
        rsx! {
            LiberoProvider {
                if modal() {
                    Modal { onclose: move |_| modal.set(false), RealSelect {} }
                }
            }
            "state: modal={modal}"
        }
    }

    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);
    escape_through_a_real_select(&mut dom, &find, "modal");
}

/// The same for `FloatingWindow`.
#[test]
fn a_real_select_in_a_window_takes_the_first_escape() {
    #[component]
    fn SelectInWindow() -> Element {
        let window = crate::hooks::use_floating_window(
            crate::components::FloatingWindowOptions {
                title: Some("Inspector".into()),
                ..Default::default()
            },
            move |_| rsx! { RealSelect {} },
        );
        use_hook(|| window.open());
        rsx! { "state: window={window.is_open()}" }
    }
    fn app() -> Element {
        rsx! { LiberoProvider { SelectInWindow {} } }
    }

    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    for _ in 0..4 {
        dom.process_events();
        dom.render_immediate(&mut find);
    }
    escape_through_a_real_select(&mut dom, &find, "window");
}

/// Todo 318. Escape held down over an open list: the first press closes
/// the list, and its auto-repeats must not go on to close the modal. A
/// composing Escape cancels the composition and closes nothing. The last,
/// plain press is the control that the modal still answers.
#[test]
fn a_held_or_composing_escape_does_not_close_the_modal() {
    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(dropdown_in_modal);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);
    let dropdown = find.dropdown.expect("the dropdown stand-in");

    press(&mut dom, dropdown);
    press_as(&mut dom, dropdown, HELD);
    assert_eq!(
        state(&dom),
        "state: modal=true list=false",
        "a held Escape's repeat must not close the modal after the list"
    );

    press_as(&mut dom, dropdown, COMPOSING);
    assert_eq!(state(&dom), "state: modal=true list=false");

    press(&mut dom, dropdown);
    assert_eq!(state(&dom), "state: modal=false list=false");
}

/// Todo 318's `FloatingWindow` half, with the same control.
#[test]
fn a_held_or_composing_escape_does_not_close_the_window() {
    fn app() -> Element {
        rsx! { LiberoProvider { DropdownInWindow {} } }
    }

    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    for _ in 0..4 {
        dom.process_events();
        dom.render_immediate(&mut find);
    }
    let dropdown = find.dropdown.expect("the dropdown stand-in");

    press(&mut dom, dropdown);
    press_as(&mut dom, dropdown, HELD);
    assert_eq!(
        state(&dom),
        "state: window=true list=false",
        "a held Escape's repeat must not close the window after the list"
    );

    press_as(&mut dom, dropdown, COMPOSING);
    assert_eq!(state(&dom), "state: window=true list=false");

    press(&mut dom, dropdown);
    assert_eq!(state(&dom), "state: window=false list=false");
}

/// The other side, and the control property the amended push rule exists
/// for.
///
/// The press lands **elsewhere in the modal, not on the trigger** - focus on
/// some other control the modal holds - so neither of the region's handlers
/// fires. Focus on the trigger is the *neighbouring* test's scenario, and
/// since `anchor_events()` landed that case closes the region rather than
/// reaching the modal; a comment pointing at it here would claim this
/// control property covers something it does not.
///
/// `platform::keyboard()` is `None` here, so the region is not on the stack
/// either, and the `Modal` is therefore top and closes. If the region had
/// pushed without a transport that could hear this press, the modal would
/// have declined as not-top and Escape would have done nothing at all,
/// where today it closes the modal.
#[test]
fn a_layer_that_cannot_hear_escape_never_wedges_the_modal() {
    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(region_in_modal);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);

    let modal_root = find.keydown[0];
    assert_eq!(state(&dom), "state: modal=true region=true");

    press(&mut dom, modal_root);
    assert_eq!(
        state(&dom),
        "state: modal=false region=true",
        "the modal must answer a press the region cannot hear"
    );
}

/// Bob3's block: off the web, Escape has to reach a box whose trigger still
/// has focus.
///
/// This is the combobox-shaped consumer and every pointer-opened surface -
/// `initial_focus: None`, focus never moves into the box. The floating box
/// is portaled to the document root, so the trigger is not inside it and
/// the box's own handler never fires. Without `anchor_events()` on the
/// trigger the press reaches nothing at all and the surface cannot be
/// dismissed from the keyboard, which is an SC 1.4.13 failure on a shipped
/// target rather than a degradation.
#[test]
fn escape_on_the_trigger_closes_the_box_where_there_is_no_document_listener() {
    fn app() -> Element {
        let region = use_signal(|| true);

        rsx! {
            LiberoProvider { Region { open: region } }
            "state: region={region}"
        }
    }

    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);

    // **Load-bearing, not a sanity check.** With no listener on the
    // trigger there would be one listener instead of two, `keydown[0]`
    // would silently be the *box's*, and pressing that closes the region -
    // so the behaviour assertion below would pass for the wrong reason.
    // Checked by stubbing `anchor_events()` out: this is the assertion that
    // fails.
    assert_eq!(
        find.keydown.len(),
        2,
        "expected the trigger and the box, got {:?}",
        find.keydown
    );
    let trigger = find.keydown[0];
    assert_eq!(state(&dom), "state: region=true");

    press(&mut dom, trigger);
    assert_eq!(
        state(&dom),
        "state: region=false",
        "Escape on the trigger should close the box"
    );
}

/// Karen3's block 11: a **closed** popover must not swallow Escape for
/// whatever is open behind it.
///
/// The trigger outlives the box, so its listener has to ask whether the box
/// is open - the floating box never has to, because it is not rendered when
/// closed. Without the guard the trigger's `stop_propagation` eats the
/// press and the enclosing `Modal` never hears it, so Escape is dead for as
/// long as focus sits on a closed popover's trigger. A layer that is not
/// even open wedging one that is.
#[test]
fn a_closed_box_does_not_swallow_escape_from_the_modal_around_it() {
    #[component]
    fn Shut(modal: Signal<bool>) -> Element {
        let region = use_signal(|| false);
        rsx! {
            Modal { onclose: move |_| { let mut modal = modal; modal.set(false); },
                Region { open: region }
            }
        }
    }

    fn app() -> Element {
        let modal = use_signal(|| true);

        rsx! {
            LiberoProvider {
                if modal() {
                    Shut { modal }
                }
            }
            "state: modal={modal}"
        }
    }

    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);

    // The modal's root and its focus trap, and nothing else: a closed box
    // contributes no listener at all. This is the discriminating
    // assertion - before the fix there were three.
    assert_eq!(
        find.keydown.len(),
        2,
        "a closed box should contribute no keydown listener, got {:?}",
        find.keydown
    );
    // Dispatched at the trigger even though it has no listener, because
    // that is where focus is in the scenario and dioxus bubbles from there.
    let trigger = find.trigger.expect("the trigger element");
    assert_eq!(state(&dom), "state: modal=true");

    press(&mut dom, trigger);
    assert_eq!(
        state(&dom),
        "state: modal=false",
        "a closed box must let the press through to the modal"
    );
}

/// Todo 252, for `3203d7fd`: a `HoverCard` forced open cannot be
/// dismissed, so it takes no Escape. With `escape: true` its trigger's
/// listener consumed the press and closed nothing, and the `Modal` around
/// it never heard Escape while focus sat on the trigger. Checked by
/// putting `escape: true` back: this test then fails.
#[test]
fn a_hover_card_forced_open_lets_escape_through_to_the_modal() {
    #[component]
    fn Forced(modal: Signal<bool>) -> Element {
        // Through `use_box`, as `Region` does. With `button { id: .. }`
        // written in `rsx!`, the recorded trigger was not where the press
        // bubbled to the modal from, and even a bare button failed.
        let trigger = use_box().prepare();
        let anchor = use_element();
        rsx! {
            Modal { onclose: move |_| { let mut modal = modal; modal.set(false); },
                crate::components::HoverCard {
                    open: Some(true),
                    aria_label: "Details",
                    content: rsx! { "details" },
                    {trigger.element(&anchor).attr("id", "trigger").render(HtmlTag::Button, Vec::new(), rsx! { "Trigger" })}
                }
            }
        }
    }

    fn app() -> Element {
        let modal = use_signal(|| true);
        rsx! {
            LiberoProvider {
                if modal() {
                    Forced { modal }
                }
            }
            "state: modal={modal}"
        }
    }

    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);

    let trigger = find.trigger.expect("the trigger element");
    assert_eq!(state(&dom), "state: modal=true");
    press(&mut dom, trigger);
    assert_eq!(
        state(&dom),
        "state: modal=false",
        "a card that cannot close must not take the modal's Escape"
    );
}

/// The other half: with no modal over it, the floating box's own handler is
/// the transport and it closes the box.
#[test]
fn a_layer_with_only_its_own_handler_still_closes_on_escape() {
    fn app() -> Element {
        let region = use_signal(|| true);

        rsx! {
            LiberoProvider { Region { open: region } }
            "state: region={region}"
        }
    }

    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);

    let region = *find.keydown.last().expect("no keydown listener");
    assert_eq!(state(&dom), "state: region=true");

    press(&mut dom, region);
    assert_eq!(state(&dom), "state: region=false");
}

/// Two portaled modals are siblings at the outlet, not nested - which is
/// why no Escape bubbles between them today and why the `is_top` guard
/// changes nothing for the shipped case. Measured rather than reasoned.
#[test]
fn two_portaled_modals_are_siblings_at_the_outlet() {
    #[component]
    fn Opener() -> Element {
        let first = use_modal(|_: ModalScope<()>| rsx! { "first body" });
        let second = use_modal(|_: ModalScope<()>| rsx! { "second body" });
        use_hook(move || {
            first.open();
            second.open();
        });
        rsx! {}
    }

    fn app() -> Element {
        rsx! { LiberoProvider { Opener {} } }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = dioxus_ssr::render(&dom);

    let first = html.find("first body").expect("the first modal");
    let second = html.find("second body").expect("the second modal");

    assert_eq!(
        div_depth_at(&html, first),
        div_depth_at(&html, second),
        "the two modals should be siblings, not nested"
    );
}

/// The wiring the docs page prints in its `use_dismiss` sample, which is
/// not compiled anywhere else. Without this the sample can go stale and
/// nothing says so.
#[test]
fn the_documented_wiring_builds_and_renders() {
    #[component]
    fn Documented(opened: Signal<bool>) -> Element {
        let theme = use_theme();
        let anchor = use_element();
        let first_item = use_element();
        let close = use_callback(move |()| {
            let mut opened = opened;
            opened.set(false);
        });

        let popover = use_popover(
            anchor,
            opened(),
            PopoverOptions::new(theme.popover.gap, theme.popover.padding),
        );
        let dismiss = use_dismiss(
            anchor,
            *popover.floating(),
            opened(),
            popover.placed(),
            Some(close),
            DismissOptions {
                initial_focus: Some(first_item),
                ..Default::default()
            },
        );

        let dropdown = use_box().style(popover.style()).prepare();
        popover.show(opened().then(|| {
            dropdown.clone().element(popover.floating()).render(
                HtmlTag::Div,
                dismiss.floating_events(),
                rsx! { "dropdown body" },
            )
        }));

        let trigger = use_box().prepare();
        trigger
            .element(&anchor)
            .attr("type", "button")
            .event("onclick", move |_: Event<MouseData>| {
                let mut opened = opened;
                dismiss.focus_return().remember_active();
                let next = !opened();
                opened.set(next);
            })
            .render(HtmlTag::Button, vec![], rsx! { "Open" })
    }

    fn app() -> Element {
        let opened = use_signal(|| true);
        rsx! { LiberoProvider { Documented { opened } } }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("dropdown body"), "the box should render");
    assert!(html.contains("Open"), "the trigger should render");
}

/// A box with `outside` on, recording which of its two callbacks a close
/// called. With `focus_moved: false` it passes no `onfocusmoved` at all.
#[component]
fn Watched(open: Signal<bool>, focus_moved: bool, heard: Signal<Vec<&'static str>>) -> Element {
    let anchor = use_element();
    let floating = use_element();
    let onclose = use_callback(move |()| {
        let (mut open, mut heard) = (open, heard);
        heard.write().push("onclose");
        open.set(false);
    });
    let moved = use_callback(move |()| {
        let (mut open, mut heard) = (open, heard);
        heard.write().push("focus_moved");
        open.set(false);
    });
    let dismiss = use_dismiss(
        anchor,
        floating,
        open(),
        true,
        Some(onclose),
        DismissOptions {
            return_focus: false,
            onfocusmoved: focus_moved.then_some(moved),
            ..Default::default()
        },
    );
    let style = use_box().prepare();
    rsx! {
        if open() {
            {style.element(&floating).render(HtmlTag::Div, dismiss.floating_events(), rsx! { "box" })}
        }
    }
}

thread_local! {
    static WITH_FOCUS_MOVED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

fn watched() -> Element {
    let open = use_signal(|| true);
    let heard = use_signal(Vec::new);
    rsx! {
        LiberoProvider { Watched { open, focus_moved: WITH_FOCUS_MOVED.get(), heard } }
        "state: open={open} heard={heard.read().join(\",\")}"
    }
}

/// Mounts [`Watched`] and sends `name` to its box.
fn close_watched(with_focus_moved: bool, name: &str) -> String {
    send_watched(with_focus_moved, false, name)
}

/// With `floor`, the box's handle is mounted on the mounted floor first,
/// which cannot answer where focus is. Otherwise it stays unmounted.
fn send_watched(with_focus_moved: bool, floor: bool, name: &str) -> String {
    dioxus::html::set_event_converter(Box::new(EscapeConverter));
    WITH_FOCUS_MOVED.set(with_focus_moved);
    let mut dom = VirtualDom::new(watched);
    let mut find = FindKeydownListeners::default();
    dom.rebuild(&mut find);
    dom.render_immediate(&mut find);
    if floor {
        let target = *find.mounted.last().expect("the box has a handle");
        let data = Rc::new(PlatformEventData::new(Box::new(()))) as Rc<dyn std::any::Any>;
        dom.runtime()
            .handle_event("mounted", Event::new(data, false), target);
        settle(&mut dom);
    }
    let (target, data) = match name {
        "focusout" => (
            find.focusout.last(),
            Rc::new(PlatformEventData::new(Box::new(FakeFocus))) as Rc<dyn std::any::Any>,
        ),
        _ => (find.keydown.last(), escape()),
    };
    let target = *target.expect("the box listens");
    dom.runtime()
        .handle_event(name, Event::new(data, true), target);
    settle(&mut dom);
    state(&dom)
}

/// Focus leaving the box - a click or a Tab elsewhere, which is the only
/// way this hook hears an outside pointer - calls `onfocusmoved` in place
/// of `onclose`. Nothing is mounted here, so every focusout is focus
/// leaving.
#[test]
fn focus_leaving_calls_onfocusmoved_instead_of_onclose() {
    assert_eq!(
        close_watched(true, "focusout"),
        "state: open=false heard=focus_moved"
    );
}

#[test]
fn without_onfocusmoved_focus_leaving_calls_onclose() {
    assert_eq!(
        close_watched(false, "focusout"),
        "state: open=false heard=onclose"
    );
}

#[test]
fn escape_still_calls_onclose_when_onfocusmoved_is_set() {
    assert_eq!(
        close_watched(true, "keydown"),
        "state: open=false heard=onclose"
    );
}

/// Todo 46: a platform that cannot say where focus is closes nothing on a
/// focusout. The unmounted runs above are the control.
#[test]
fn a_platform_that_cannot_answer_ignores_focusout() {
    assert_eq!(
        send_watched(true, true, "focusout"),
        "state: open=true heard="
    );
    assert_eq!(
        send_watched(false, true, "focusout"),
        "state: open=true heard="
    );
}

#[test]
fn a_platform_that_cannot_answer_still_closes_on_escape() {
    assert_eq!(
        send_watched(true, true, "keydown"),
        "state: open=false heard=onclose"
    );
}

#[test]
fn one_yes_wins_and_one_unknown_blocks_a_no() {
    assert_eq!(focus_inside_of([Some(false), None, Some(true)]), Some(true));
    assert_eq!(focus_inside_of([Some(false), None]), None);
    assert_eq!(focus_inside_of([Some(false), Some(false)]), Some(false));
}
