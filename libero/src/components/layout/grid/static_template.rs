use std::ops::Deref;
use std::sync::OnceLock;

use super::{GridArea, GridTemplate, GridTemplateBuilder};

/// A [`GridTemplate`] built once, at a `static` - the shape of a layout is a
/// constant, and `build`'s `Result` cannot travel through `fn() -> Element`.
///
/// ```ignore
/// static PAGE: StaticGridTemplate<PageArea> = StaticGridTemplate::new(|template| {
///     template
///         .row(|row| row.cell(PageArea::Header))
///         .row(|row| row.cell(PageArea::Sidebar).cells(PageArea::Content, 3))
/// });
///
/// rsx! { Grid { template: PAGE.clone(), .. } }
/// ```
///
/// The type parameter stays here, on the caller's own `static`; `Grid` still
/// takes the erased [`GridTemplate`], so no component is monomorphized per
/// area enum.
///
/// `OnceLock` rather than `LazyLock` (which is what [`StaticSx`] uses) only
/// because the builder function has to be stored and applied later.
///
/// [`StaticSx`]: crate::sx::StaticSx
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

    /// Panics on a template CSS cannot express. A `static` shape is a
    /// programmer error, not a runtime condition - build it with
    /// [`GridTemplate::new`] where the `Result` matters.
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
