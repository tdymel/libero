# Performance

Crate: `libero`
Index: [index.md](index.md) - every other component's markdown page
Description: What a libero component actually costs per render, measured by ablation - scope and dynamic-node counts, memoization boundaries, and how to measure it yourself.

Every number on this page was measured by ablation - removing a thing and taking
the delta - rather than reasoned about or read off a timer. Timers inside a
render lie in both directions: attributes are diffed after `rsx!` returns, so a
probe around a render body misses them, and a dozen nested probes inflated one
component by 25%.

The short version: styling is not what costs, and neither is the size of a props
struct. What costs is the shape of the tree - how many component scopes and
dynamic nodes there are, and how much of it re-renders when something changes.

## What a component costs

A Libero component is one Dioxus component scope plus one styling pass. Measured
against a component that renders a bare `span` and nothing else, the cheap ones -
`Box`, `Text`, `Center` - land around 1.5x, and that cluster is the floor.
Anything above it is paying for markup of its own: `Select` renders a label, a
trigger and a list; `Sidebar` renders a bordered panel around a scroll area. The
spread across the library:

| Component | Re-render cost |
|---|---|
| `Box` | 1.5x |
| `Text` | 1.6x |
| `Flex` | 2.1x |
| `Button` | 2.4x |
| `Select` | 6.1x |
| `Splitter` | 8.2x |
| `Sidebar` | 4.7x |

Ratios travel between machines; absolute nanoseconds do not. So these are an
ordering - which components to look at first when a screen is slow - and not a
budget.

## Styling is not the expensive part

An `Sx` hashes to a class by its content, so a thousand elements styled alike
share one class and one CSS rule, built once. Per render, an unchanged `sx` costs
a hash comparison and nothing else - no CSS is re-rendered and no style attribute
is diffed.

Three things keep it that way, and all three are [styling.md](styling.md)'s
advice for a different reason: hoist a constant style into a `StaticSx` so it is
built once per process and compared by pointer; switch variants with `states` so
every variant shares one class instead of minting one each; and for a genuinely
unbounded value - a drag offset, a percentage - use `Variables`, which writes a
CSS custom property into the style attribute rather than a new class per distinct
value.

## Memoization boundaries

A component scope is not just overhead - it is a boundary Dioxus stops at. While
a component's props compare equal, its whole subtree is skipped. So a block of
markup that only changes with one piece of state, sitting inside something that
re-renders for other reasons, is often cheaper as its own component: one scope
bought, a subtree saved. Extracting the divider out of `Splitter` that way cost
one scope and saved 27% of the component.

It only works if every prop compares by value or by a stable identity. Plain
values, `Signal`s and a `use_callback` do; a bare closure never does, and neither
does `children` - two `Element`s compare by pointer, so a component that takes
children can never memoize.

```rust
// Re-renders every time the list does: a bare closure has a
// fresh identity each render, so the props never compare equal.
Row { onselect: move |_| selected.set(id), .. }

// Skipped while nothing about it changed.
let onselect = use_callback(move |_| selected.set(id));
Row { onselect, .. }
```

## What actually costs

Per re-rendered instance, all by ablation. The surprise in this table is usually
the second row: a `{..}` hole in `rsx!` is paid for on every render even when the
branch inside it renders nothing, while static markup around it is free.

| What | Cost | Why it is worth knowing |
|---|---|---|
| A component scope | ~300 ns | The floor for anything that is its own component - and the price of a memoization boundary. |
| A dynamic node - any `{..}` hole in `rsx!` | ~400-600 ns | Paid whether or not it renders anything. Static markup is free: templates are const. |
| An `EventHandler` props field | ~300 ns | Even when the caller never sets it - the default still allocates. `Option<EventHandler<T>>` costs ~95 ns. |
| An attribute through the builder | ~135 ns | Including a `false` or `None` that renders nothing. |
| `use_signal` plus one read | ~290 ns | The subscription bookkeeping, not the hook slot. |
| A hook slot | ~51 ns | `use_context`, `use_hook` and `use_drop` are one each. Props field count, on the other hand, is free. |

Props field count is free: nine extra optional fields on a component that does
nothing else measured at zero. Only the `EventHandler` is expensive, and only
because its default allocates.

## Measuring it yourself

The library's own harness covers every component, one row each, and prints
nanoseconds per re-rendered instance. It is ignored by default, because the
numbers only mean anything in release:

```bash
cargo test --release -p libero --test render_cost -- --ignored --nocapture
```

Four rules make a run trustworthy, in the order of how much grief each one saves:
price a thing by removing it, not by wrapping it in a timer; keep a control row in
every run, so machine drift is visible; interleave the variants and keep a
per-variant minimum, because sequential runs drift enough to invert a sign; and
compare against a baseline you ran on the same machine, never against a number
someone wrote down.
