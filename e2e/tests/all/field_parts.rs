//! The `parts` prop reaches each form component's inner parts by its
//! selectors, and a field nested in a slot keeps its own look.

use e2e::browser::block_on;
use e2e::passes::pointer;
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

/// `Form`'s summary, the portaled `Combobox` dropdown, `ColorPicker` and a lone
/// `HueSlider`, each styled through `parts`.
#[test]
fn parts_reach_the_summary_the_dropdown_and_the_color_parts() {
    block_on(async {
        let fixture = Fixture::open("/form-parts", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#hue").await.unwrap();

        pointer::click(page, "#submit").await.unwrap();
        wait::for_visible(page, "#form > [data-slot=summary]")
            .await
            .unwrap();
        pointer::click(page, "#trigger").await.unwrap();
        wait::for_visible(page, "[data-slot=option]").await.unwrap();

        let cases = [
            ("#form > [data-slot=summary]", "paddingTop", "13px"),
            ("#form [data-slot=title]", "fontStyle", "italic"),
            ("#form [data-slot=list]", "letterSpacing", "3px"),
            ("[data-slot=option]", "letterSpacing", "3px"),
            ("[data-slot=group-label]", "fontStyle", "italic"),
            (
                "#picker > [data-slot=saturation] > [data-slot=thumb]",
                "borderTopWidth",
                "3px",
            ),
            (
                "#picker [data-slot=hue] [data-slot=thumb]",
                "borderTopWidth",
                "3px",
            ),
            (
                "#picker [data-slot=alpha] [data-slot=thumb]",
                "borderTopWidth",
                "3px",
            ),
            ("#picker [data-slot=swatch]", "marginTop", "5px"),
            ("#hue > [data-slot=track]", "marginTop", "7px"),
            ("#picker [data-slot=track]", "marginTop", "0px"),
        ];
        for (selector, property, expected) in cases {
            let value = page
                .evaluate(format!(
                    "(() => {{ const el = document.querySelector({selector:?}); \
                     return el ? getComputedStyle(el).{property} : 'missing'; }})()"
                ))
                .await
                .unwrap()
                .into_value::<String>()
                .unwrap();
            assert_eq!(value, expected, "{selector} {property}");
        }

        fixture.console.assert_clean("form parts").unwrap();
        fixture.close().await.unwrap();
    });
}
