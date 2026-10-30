use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::{use_interval, use_timeout},
};
use web_time::Instant;

#[component]
pub fn Stopwatch() -> Element {
    let mut since = use_signal(|| None::<Instant>);
    let mut banked = use_signal(|| 0.0);
    let mut seconds = use_signal(|| 0);
    let mut splits = use_signal(Vec::<f64>::new);
    let mut saved = use_signal(|| false);
    let elapsed = move || banked() + since().map_or(0.0, |at: Instant| at.elapsed().as_secs_f64());
    let interval = use_interval(move || seconds.set(elapsed() as u64), 1000);
    let flash = use_timeout(move || saved.set(false), 2000);
    let list = splits
        .read()
        .iter()
        .map(|split| format!("{split:.1} s"))
        .collect::<Vec<_>>()
        .join(", ");

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Text { "{seconds} s" }
            Flex { gap: "sm",
                Button {
                    onclick: move |_| {
                        match since() {
                            Some(at) => {
                                banked.set(banked() + at.elapsed().as_secs_f64());
                                since.set(None);
                                seconds.set(banked() as u64);
                            }
                            None => since.set(Some(Instant::now())),
                        }
                        interval.toggle();
                    },
                    if interval.active() {
                        "Stop"
                    } else {
                        "Start"
                    }
                }
                Button {
                    variant: "outlined",
                    disabled: since().is_none(),
                    focusable_when_disabled: true,
                    onclick: move |_| {
                        splits.push(elapsed());
                        saved.set(true);
                        flash.start();
                    },
                    "Save split"
                }
            }
            div { role: "status",
                if saved() {
                    "Split {splits.len()} saved"
                }
            }
            if !list.is_empty() {
                Text { "Splits: {list}" }
            }
        }
    }
}
