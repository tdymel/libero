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

## 1. Developer experience

An API that is small, clear and hard to misuse.

What we do:

- Let the compiler catch mistakes before a reviewer has to.
- Keep one way of doing things, the same across every component.
- Write docs that you and your AI assistant can copy from, with examples that
  build.

Why it matters: AI writes a lot of code now, and someone still has to read it,
review it and change it next month. Code that is hard to get wrong is easier
for a person to check and for a model to get right.

## 2. Accessibility

Everyone should be able to use what you build.

What we do:

- Follow WAI-ARIA, the APG patterns and WCAG. A deviation needs a reason.
- Aim for WCAG 2.2 AA.
- Never show a state by colour alone.
- Speak the user's language: the words components say on their own translate,
  with English and German built in.

A known limit: Windows High Contrast mode is supported only in part. The
[Accessibility](accessibility.md) page says what holds.

Why it matters: It helps more people than screen reader users. Keyboard support
helps anyone who would rather not reach for the mouse. A state that does not
rely on colour still reads on a dim screen in the sun. Motion that stops on
request helps anyone it makes dizzy.

## 3. Batteries included

What a typical app needs, in one place.

What we do:

- Cover the everyday needs, forms and overlays included.
- Work the same on the web and natively.
- Be honest where a platform falls short: a missing feature is absent, not
  broken.

Why it matters: Starting an app should not mean hunting for a date picker. Your
time goes into your app, not into stitching libraries together.

## 4. Simple yet modern

A clean look that stays out of the way.

What we do:

- Aim for a default that fits most apps as it is.
- Make changing the look ordinary Rust, not a fight with the library.
- Offer light and dark from the start.
- Let your own styles win over ours.

Why it matters: Your app should look like your app. A quiet default gets you
started, and when you want your own look, nothing stands in the way.
