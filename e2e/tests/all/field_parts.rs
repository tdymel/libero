//! The `parts` prop reaches each form component's inner parts by its
//! selectors, and a field nested in a slot keeps its own look.

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

#[test]
fn parts_style_the_inner_parts() {
    block_on(async {
        let fixture = Fixture::open("/field-parts", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#file").await.unwrap();
        let read = |selector: &'static str, property: &'static str| async move {
            page.evaluate(format!(
                "(() => {{ const el = document.querySelector({selector:?}); \
                 return el ? getComputedStyle(el).{property} : 'missing'; }})()"
            ))
            .await
            .unwrap()
            .into_value::<String>()
            .unwrap()
        };

        // Each id lands on the control, so the part is found from the wrapper.
        let cases = [
            ("label[for=text]", "fontStyle", "italic"),
            (
                "label[for=text] > [data-slot=required]",
                "letterSpacing",
                "3px",
            ),
            ("#text", "letterSpacing", "5px"),
            ("#nested", "letterSpacing", "normal"),
            ("label[for=nested]", "fontStyle", "normal"),
            ("#textarea ~ [data-slot=counter]", "letterSpacing", "3px"),
            ("#select > [data-slot=value]", "fontStyle", "italic"),
            ("[data-slot=chip]", "paddingLeft", "6px"),
            ("#checkbox + [data-slot=box]", "borderTopWidth", "3px"),
            ("[data-slot=thumb]", "borderTopWidth", "3px"),
            ("#file", "letterSpacing", "3px"),
            ("#chip > [data-slot=chip-icon]", "paddingLeft", "6px"),
            ("#fieldset > [data-slot=legend]", "fontStyle", "italic"),
        ];
        for (selector, property, expected) in cases {
            assert_eq!(
                read(selector, property).await,
                expected,
                "{selector} {property}"
            );
        }

        // The frame and helper of `#text`, reached from its own wrapper.
        let frame = "#text";
        let border = page
            .evaluate(format!(
                "getComputedStyle(document.querySelector({frame:?}).closest('[data-slot=frame]')).borderTopWidth"
            ))
            .await
            .unwrap()
            .into_value::<String>()
            .unwrap();
        assert_eq!(border, "3px");
        let nested_border = page
            .evaluate(
                "getComputedStyle(document.querySelector('#nested').closest('[data-slot=frame]')).borderTopWidth",
            )
            .await
            .unwrap()
            .into_value::<String>()
            .unwrap();
        assert_eq!(
            nested_border, "1px",
            "the nested field took the outer one's frame"
        );

        fixture.console.assert_clean("field parts").unwrap();
        fixture.close().await.unwrap();
    });
}
