use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct OptionProps {
    #[props(into)]
    value: String,
    children: Element,
}

#[component]
pub fn Option(props: OptionProps) -> Element {
    rsx! {
        option {
            value: props.value,
            {props.children}
        }
    }
}
