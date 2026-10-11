/// Two names for one animation: only a name change replays a running one, so
/// consecutive clicks alternate.
pub const RIPPLE_ANIMATION: [&str; 2] = ["lsx-ripple-a", "lsx-ripple-b"];
pub const RIPPLE_KEYFRAMES: &str = concat!(
    "@keyframes lsx-ripple-a{from{opacity:0.3;transform:translate(-50%, -50%) scale(0);}",
    "to{transform:translate(-50%, -50%) scale(1);opacity:0;}}",
    "@keyframes lsx-ripple-b{from{opacity:0.3;transform:translate(-50%, -50%) scale(0);}",
    "to{transform:translate(-50%, -50%) scale(1);opacity:0;}}"
);

/// The `data-state` that runs each one.
pub const RIPPLE_STATE: [&str; 2] = ["ripple-a", "ripple-b"];
