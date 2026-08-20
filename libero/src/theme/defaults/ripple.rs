/// Two names for one animation. Restarting a CSS animation on an element that
/// is already running it needs the *name* to change - a class or custom
/// property does not replay it - so consecutive clicks alternate between these.
pub const RIPPLE_ANIMATION: [&str; 2] = ["lsx-ripple-a", "lsx-ripple-b"];
pub const RIPPLE_KEYFRAMES: &str = concat!(
    "@keyframes lsx-ripple-a{from{opacity:0.3;transform:translate(-50%, -50%) scale(0);}",
    "to{transform:translate(-50%, -50%) scale(1);opacity:0;}}",
    "@keyframes lsx-ripple-b{from{opacity:0.3;transform:translate(-50%, -50%) scale(0);}",
    "to{transform:translate(-50%, -50%) scale(1);opacity:0;}}"
);

/// `ripple-a` / `ripple-b`, the `data-state` that runs each one.
pub const RIPPLE_STATE: [&str; 2] = ["ripple-a", "ripple-b"];
