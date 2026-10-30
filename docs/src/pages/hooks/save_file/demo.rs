use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    platform::{SaveOutcome, save_file},
};

fn outcome_text(outcome: &SaveOutcome) -> String {
    match outcome {
        SaveOutcome::Saved => "Saved team.csv".into(),
        SaveOutcome::Shared => "Shared team.csv".into(),
        SaveOutcome::Cancelled => "Save cancelled".into(),
        SaveOutcome::Failed(reason) => format!("Could not save: {reason}"),
    }
}

#[component]
pub fn SaveDemo() -> Element {
    let mut outcome = use_signal(|| None::<SaveOutcome>);
    rsx! {
        Flex { gap: "sm", align: "center", wrap: "wrap",
            Button {
                onclick: move |_| async move {
                    let csv = "name,role\nAda,Engineer\n";
                    let saved = save_file("team.csv", "text/csv", csv.into()).await;
                    outcome.set(Some(saved));
                },
                "Save team.csv"
            }
            div { role: "status",
                if let Some(outcome) = outcome() {
                    Text { size: "sm", {outcome_text(&outcome)} }
                }
            }
        }
    }
}
