use std::ops::Deref;
use std::sync::OnceLock;

use super::{GridArea, GridTemplate, GridTemplateBuilder};

/// A [`GridTemplate`] built on first use, at a `static`. Panics if CSS can't
/// express the shape; use [`GridTemplate::new`] where the `Result` matters.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Grid, GridArea, StaticGridTemplate};
/// # fn app() -> Element {
/// # #[derive(Clone, Copy, PartialEq)] enum PageArea { Header, Sidebar, Content }
/// # impl GridArea for PageArea { fn name(&self) -> &'static str { "a" } }
/// static PAGE: StaticGridTemplate<PageArea> = StaticGridTemplate::new(|template| {
///     template
///         .row(|row| row.cell(PageArea::Header))
///         .row(|row| row.cell(PageArea::Sidebar).cells(PageArea::Content, 3))
/// });
///
/// rsx! { Grid { template: PAGE.clone() } }
/// # }
/// ```
// `OnceLock`, not `LazyLock`: the builder function is stored and applied later.
pub struct StaticGridTemplate<A: GridArea> {
    build: fn(GridTemplateBuilder<A>) -> GridTemplateBuilder<A>,
    template: OnceLock<GridTemplate>,
}

impl<A: GridArea> StaticGridTemplate<A> {
    pub const fn new(build: fn(GridTemplateBuilder<A>) -> GridTemplateBuilder<A>) -> Self {
        Self {
            build,
            template: OnceLock::new(),
        }
    }
}

impl<A: GridArea> std::fmt::Debug for StaticGridTemplate<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StaticGridTemplate").finish()
    }
}

impl<A: GridArea> Deref for StaticGridTemplate<A> {
    type Target = GridTemplate;

    /// Panics on a template CSS cannot express: a bad `static` shape is a programmer error.
    fn deref(&self) -> &Self::Target {
        self.template.get_or_init(|| {
            (self.build)(GridTemplate::new())
                .build()
                .unwrap_or_else(|error| panic!("StaticGridTemplate: {error}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq)]
    enum Area {
        Header,
        Body,
    }

    impl GridArea for Area {
        fn name(&self) -> &'static str {
            match self {
                Self::Header => "header",
                Self::Body => "body",
            }
        }
    }

    static PAGE: StaticGridTemplate<Area> = StaticGridTemplate::new(|template| {
        template
            .row(|row| row.cell(Area::Header))
            .row(|row| row.cell(Area::Body))
    });

    #[test]
    fn a_static_template_builds_on_first_use_and_stays_the_same_value() {
        assert_eq!(PAGE.0.areas, "\"header\" \"body\"");
        // The `Arc` is shared, so cloning it at a call site is a refcount bump.
        assert!(std::sync::Arc::ptr_eq(&PAGE.clone().0, &PAGE.0));
    }
}
