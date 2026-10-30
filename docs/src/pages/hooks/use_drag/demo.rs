use dioxus::prelude::*;
use libero::{
    components::Box,
    hooks::{
        DragMove, DragOptions, DragStart, drag_handle_sx, use_direction, use_drag, use_element,
    },
    sx::sx,
};

#[component]
pub fn Knob() -> Element {
    let track = use_element();
    let rtl = use_direction().is_rtl();
    // Under RTL the knob starts on the right, so a drag or arrow to the left adds.
    let forward = if rtl { -1.0 } else { 1.0 };
    let mut x = use_signal(|| 0.0);
    let mut from = use_signal(|| 0.0);
    let drag = use_drag(DragOptions {
        capture: track,
        onstart: Callback::new(move |_: DragStart| from.set(x())),
        onmove: Callback::new(move |step: DragMove| {
            x.set((from() + forward * step.delta().x).clamp(0.0, 200.0));
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
            // A 44 px hit area round the 32 px knob, as a touch target.
            Box {
                onpointerdown: move |event| drag.onpointerdown.call(event),
                tabindex: 0,
                role: "slider",
                aria_label: "Knob position",
                aria_valuemin: 0,
                aria_valuemax: 200,
                aria_valuenow: "{x}",
                aria_valuetext: "{x} px",
                onkeydown: move |event: KeyboardEvent| {
                    let step = match event.key() {
                        Key::Named(NamedKey::ArrowLeft) => -10.0 * forward,
                        Key::Named(NamedKey::ArrowRight) => 10.0 * forward,
                        Key::Named(NamedKey::ArrowDown) => -10.0,
                        Key::Named(NamedKey::ArrowUp) => 10.0,
                        Key::Named(NamedKey::Home) => -200.0,
                        Key::Named(NamedKey::End) => 200.0,
                        _ => return,
                    };
                    event.prevent_default();
                    x.set((x() + step).clamp(0.0, 200.0));
                },
                sx: drag_handle_sx().position("absolute").top("-6px").width("44px").height("44px")
                    .display("grid").place_items("center").border_radius("22px").cursor("grab")
                    .focus_visible(sx().outline_offset("-10px")),
                style: "inset-inline-start: calc({x}px - 6px)",
                Box {
                    aria_hidden: "true",
                    sx: sx().width("32px").height("32px").border_radius("16px")
                        .background("primary.6"),
                }
            }
        }
    }
}
