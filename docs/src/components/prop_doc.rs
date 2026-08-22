use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, Table, Text, Title, column},
    sx::sx,
};

/// One row of a docs page's Properties table.
#[derive(Clone, PartialEq)]
pub struct PropDoc {
    name: String,
    ty: String,
    default: String,
    description: String,
}

/// Starts a property row. `default` and `doc` fill the rest.
pub fn prop(name: impl Into<String>, ty: impl Into<String>) -> PropDoc {
    PropDoc {
        name: name.into(),
        ty: ty.into(),
        default: String::new(),
        description: String::new(),
    }
}

impl PropDoc {
    pub fn default(mut self, default: impl Into<String>) -> Self {
        self.default = default.into();
        self
    }

    pub fn doc(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

/// One component's props: a page documents its subcomponents as further groups.
#[derive(Clone, PartialEq)]
pub struct PropGroup {
    component: String,
    props: Vec<PropDoc>,
    base: bool,
}

/// Starts a Properties block for one component.
pub fn props(component: impl Into<String>, props: Vec<PropDoc>) -> PropGroup {
    PropGroup {
        component: component.into(),
        props,
        base: true,
    }
}

impl PropGroup {
    /// For a type that is not a `base_props!` component - a builder, say.
    #[allow(dead_code)]
    pub fn without_base_props(mut self) -> Self {
        self.base = false;
        self
    }
}

/// Appended to every component's own list: what `base_props!` gives all of them.
fn base_props() -> Vec<PropDoc> {
    vec![
        prop("class", "ClassList").doc("Extra class names on the root element."),
        prop("sx", "Sx").doc("Style overrides, applied after the theme's."),
        prop("states", "States").doc("`data-state` flags on the root, for styling and CSS hooks."),
        prop("attributes", "Vec<Attribute>")
            .doc("Any global HTML attribute or DOM event handler, passed through to the root."),
    ]
}

/// The Properties tab: one table per component, its own props then the shared ones.
///
/// Two columns, not four - the first reads as a signature (`size: Size`) with
/// the default beside it, so the table survives a phone without a horizontal
/// scrollbar and without a tall cell per row.
#[component]
pub fn PropertyTable(properties: Vec<PropGroup>) -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xl",
            for group in properties {
                Flex {
                    direction: "column",
                    gap: "sm",
                    Title { size: "lg", "{group.component}" }
                    PropRows { properties: group.props, base: group.base }
                }
            }
        }
    }
}

#[component]
fn PropRows(properties: Vec<PropDoc>, base: bool) -> Element {
    let mut rows = properties;
    if base {
        rows.extend(base_props());
    }

    rsx! {
        Table {
            // A wrapped name cell would float a middle-aligned description
            // away from the prop it describes.
            sx: sx().selector("& td", sx().vertical_align("top")),
            data: rows,
            columns: vec![
                column("Name")
                    .value(|p: &PropDoc| p.name.clone())
                    .render(|p: &PropDoc| rsx! {
                        Flex {
                            wrap: "wrap",
                            gap: "xs",
                            align: "baseline",
                            Code { source: "{p.name}: {p.ty}" }
                            if !p.default.is_empty() {
                                Text { size: "xs", color: "grey.6", "default: {p.default}" }
                            }
                        }
                    }),
                column("Description").value(|p: &PropDoc| p.description.clone()),
            ],
        }
    }
}
