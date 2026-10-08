<p align="center">
  <img src="https://raw.githubusercontent.com/tdymel/libero/main/docs/assets/logo.svg" alt="Libero" width="240">
</p>

<p align="center"><strong>Focus on your game, while Libero has your back!</strong></p>

Libero is a component library for Rust and [Dioxus](https://dioxuslabs.com).
Like the libero on a volleyball court, it covers the defence: accessibility,
keyboard support and theming are handled, so you can focus on your app.

- 100+ components and hooks, accessible and keyboard-ready
- A typed `sx` builder for theme-aware styling, all in Rust
- 20+ ready-made themes, or your own
- One codebase for the web, the desktop and Android (iOS builds, untested)

## A taste

```rust
#[derive(Clone, PartialEq, Default, Fields)]
struct Booking {
    name: String,
    day: Option<NaiveDate>,
    terrace: bool,
}

#[component]
fn BookingForm() -> Element {
    let booking = use_store(Booking::default);
    let notify = use_notifications();

    rsx! {
        Paper {
            shadow: "lg",
            // Typed, theme-aware styling: spacing tokens, palette colours, breakpoints.
            sx: sx()
                .padding("md")
                .border_top("4px solid")
                .border_color("primary.6")
                .breakpoint(Size::Md, sx().padding("xl").max_width("440px")),
            Form {
                value: booking,
                onsubmit: move |_| notify.show("Table booked."),
                TextField {
                    label: "Name",
                    name: Booking::FIELDS.name(),
                    validate: not_empty.error("Enter a name."),
                }
                DateField { label: "Day", name: Booking::FIELDS.day() }
                Switch { label: "On the terrace", name: Booking::FIELDS.terrace() }
                Button { r#type: "submit", "Book" }
            }
        }
    }
}
```

See it running, with every component and its live demos, at
**[libero-ui.dev](https://libero-ui.dev)**.

## Installation

Libero is not on crates.io yet. Until the first release, install it from this
repository, with dioxus from git too (libero tracks Dioxus `main`):

```shell
cargo add dioxus --git https://github.com/DioxusLabs/dioxus
cargo add libero --git https://github.com/tdymel/libero
```

[Getting started](https://libero-ui.dev/about/getting-started) covers the
setup, each platform and the feature flags.

## AI disclaimer

Libero's implementation is heavily driven by AI coding agents. The
architectural decisions and the code reviews are made by humans, and an
extensive test suite backs every change, but expect the rough edges of a
pre-release. Libero is a work in progress, and APIs may change
between `0.x` releases.

## License

MIT OR Apache-2.0
