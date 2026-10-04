use dioxus::prelude::*;
use libero::{
    components::Box,
    hooks::{DragMove, DragOptions, DragStart, drag_handle_sx, use_drag, use_element},
    sx::sx,
};

#[component]
pub fn Knob() -> Element {
    let track = use_element();
    let mut x = use_signal(|| 0.0);
    let mut from = use_signal(|| 0.0);
    let drag = use_drag(DragOptions {
        capture: track,
        onstart: Callback::new(move |_: DragStart| from.set(x())),
        onmove: Callback::new(move |step: DragMove| {
            x.set((from() + step.delta().x).clamp(0.0, 200.0));
        }),
        onend: Callback::new(|()| {}),
    });

    rsx! {
        Box {
            onmounted: track.mount(),
            onpointermove: move |event| drag.onpointermove.call(event),
            onpointerup: move |event| drag.onpointerup.call(event),
            onpointercancel: move |event| drag.onpointercancel.call(event),
            sx: sx().position("relative").width("232px").height("32px")
                .background("muted.1").border_radius("16px"),
            Box {
                onpointerdown: move |event| drag.onpointerdown.call(event),
                tabindex: 0,
                role: "slider",
                aria_label: "Knob position",
                aria_valuemin: 0,
                aria_valuemax: 200,
                aria_valuenow: "{x}",
                onkeydown: move |event: KeyboardEvent| {
                    let next = match event.key() {
                        Key::ArrowLeft | Key::ArrowDown => x() - 10.0,
                        Key::ArrowRight | Key::ArrowUp => x() + 10.0,
                        Key::Home => 0.0,
                        Key::End => 200.0,
                        _ => return,
                    };
                    event.prevent_default();
                    x.set(next.clamp(0.0, 200.0));
                },
                sx: drag_handle_sx().position("absolute").width("32px").height("32px")
                    .border_radius("16px").background("primary.6").cursor("grab")
                    .focus_visible(sx().outline_offset("-4px")),
                style: "left: {x}px",
            }
        }
    }
}
