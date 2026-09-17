# Philosophy

Crate: `libero`
Index: [index.md](index.md) - every other component's markdown page
Description: The four principles behind libero in order of priority (developer experience, accessibility, batteries included, simple yet modern) and what we do about each.

Four principles decide what goes into libero and how it is shaped. They are
ranked: when two pull in different directions, the higher one wins.

1. Developer experience
2. Accessibility
3. Batteries included
4. Simple yet modern

## 1. Developer experience first

A small API that is hard to misuse.

What we do:

- Let the compiler catch the mistakes a convention would leave to code review.
- Offer one way to style and one way to set up, the same on every component.
- Give every docs page a plain markdown copy whose examples the tests compile.

Why it matters: AI writes a lot of code now. Someone still reviews it and
changes it next month, and an API that is hard to misuse is easy for a person
to check and for a model to get right.

## 2. Accessibility second

Not only for screen reader users.

What we do:

- Follow WAI-ARIA, the APG patterns and WCAG. A deviation needs a reason.
- Aim for WCAG 2.2 AA, contrast included.
- Never show a state by colour alone.
- Make every string a component says on its own translatable, with English and
  German built in.

A known limit: Windows High Contrast mode is supported only in part. The
[Accessibility](accessibility.md) page says what holds.

Why it matters: Keyboard support helps anyone who would rather not reach for
the mouse. A state that does not rely on colour still reads on a dim screen in
the sun. Motion that stops on request helps anyone it makes dizzy.

## 3. Batteries included

An app should not start with a hunt for a date picker.

What we do:

- Ship more than 100 components, forms and overlays included.
- Run the same components on the web and natively.
- Treat a feature the platform lacks as absent, not broken.

Why it matters: You spend your time on your app, not on stitching libraries
together.

## 4. Simple yet modern

A clean default look that stays out of the way.

What we do:

- Keep theming plain Rust: one struct, and you change only what you need.
- Build in light and dark, correct from the first paint.
- Use current CSS, with no CSS framework underneath.
- Let your own CSS win over libero's.

Why it matters: The default fits most apps as it is. When it does not, changing
it is ordinary Rust, not a fight with the library.
