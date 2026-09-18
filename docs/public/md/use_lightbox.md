# use_lightbox

Crate: `libero`
Import: `use libero::hooks::use_lightbox;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/use_lightbox.rs>
Index: [index.md](index.md) lists every other page
Description: Registers a picture viewer over the page and returns the handle that opens it.

`use_lightbox(options) -> ModalHandle<LightboxOpening>` registers a picture
viewer over the page. Open it with the pictures and the index to start on, or
with one picture. [Lightbox](lightbox.md) covers zoom, thumbnails, captions and
swiping.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Flex, Image},
    hooks::{LightboxItem, LightboxOptions, use_lightbox},
    sx::sx,
};

#[derive(Clone, PartialEq)]
struct Photo {
    src: String,
    alt: String,
}

#[component]
fn Gallery(photos: Vec<Photo>) -> Element {
    let lightbox = use_lightbox(LightboxOptions::default());
    let items: Vec<LightboxItem> = photos.iter().map(|p| LightboxItem::new(&p.src, &p.alt)).collect();

    rsx! {
        Flex { direction: "row", gap: "xs",
            for (index, photo) in photos.iter().enumerate() {
                Box {
                    component: "button",
                    r#type: "button",
                    aria_label: "Open {photo.alt}",
                    sx: sx().width("96px").height("96px").padding("0").border_width("0"),
                    onclick: {
                        let items = items.clone();
                        move |_| { lightbox.open_with((items.clone(), index)); }
                    },
                    Image { src: "{photo.src}", alt: "", fit: "cover" }
                }
            }
        }
    }
}
```

## Accessibility

Each thumbnail is a button named for its picture, and the image inside has an
empty `alt`, so the name is read once. The viewer is a modal, so focus returns
to the thumbnail when it closes.

## API

```rust,ignore
pub fn use_lightbox(options: LightboxOptions) -> ModalHandle<LightboxOpening>
```

`open_with` takes a `LightboxItem`, a `Vec<LightboxItem>`, or
`(Vec<LightboxItem>, usize)` with the index to start on. The handle is the one
[use_modal](use_modal.md) returns. `LightboxOptions` is listed on
[Lightbox](lightbox.md).
