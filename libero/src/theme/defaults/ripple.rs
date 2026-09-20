/// Two names for one animation: only a name change replays a running one, so
/// consecutive clicks alternate.
pub const RIPPLE_ANIMATION: [&str; 2] = ["lsx-ripple-a", "lsx-ripple-b"];
pub const RIPPLE_KEYFRAMES: &str = concat!(
    "@keyframes lsx-ripple-a{from{opacity:0.3;transform:translate(-50%, -50%) scale(0);}",
    "to{transform:translate(-50%, -50%) scale(1);opacity:0;}}",
    "@keyframes lsx-ripple-b{from{opacity:0.3;transform:translate(-50%, -50%) scale(0);}",
    "to{transform:translate(-50%, -50%) scale(1);opacity:0;}}",
    "@keyframes lsx-ripple-clip-a{from{opacity:0.3;clip-path:circle(0 at var(--lsx-ripple-x, 50%) var(--lsx-ripple-y, 50%));}",
    "to{clip-path:circle(150% at var(--lsx-ripple-x, 50%) var(--lsx-ripple-y, 50%));opacity:0;}}",
    "@keyframes lsx-ripple-clip-b{from{opacity:0.3;clip-path:circle(0 at var(--lsx-ripple-x, 50%) var(--lsx-ripple-y, 50%));}",
    "to{clip-path:circle(150% at var(--lsx-ripple-x, 50%) var(--lsx-ripple-y, 50%));opacity:0;}}"
);

/// Grown by `clip-path`, so the host needs no `overflow: hidden` (`ActionIcon`'s hit area).
pub const RIPPLE_CLIP_ANIMATION: [&str; 2] = ["lsx-ripple-clip-a", "lsx-ripple-clip-b"];

/// The `data-state` that runs each one.
pub const RIPPLE_STATE: [&str; 2] = ["ripple-a", "ripple-b"];
