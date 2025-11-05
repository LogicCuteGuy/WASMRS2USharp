pub mod traits;
pub mod types;
pub mod attributes;
pub mod macros;
pub mod error;
pub mod diagnostics;
pub mod multi_behavior_errors;
pub mod prelude;
pub mod testing;

#[cfg(test)]
pub mod integration_tests;

#[cfg(test)]
pub mod test_runner;

pub use traits::*;
pub use types::{
    Vector2, Vector3, Vector4, Quaternion, Color, Color32,
    VRCPlayerApi, GameObject, Transform, Collider, Collision,
    UdonSyncMode, NetworkEventTarget, UdonFieldInfo, Rigidbody, 
    Animator, AudioSource, Light, VRCStation, VRCPickup, 
    VRCObjectPool, Text, Button, Slider, Toggle, UdonSharpUtility, 
    Networking, Mathf, Random, Time
};
pub use attributes::{
    UdonPublic, UdonSync, UdonEvent, UdonNetworkEvent,
    UdonHeader, UdonTooltip, UdonRange, UdonTextArea,
    UdonSpace, UdonPropertyDrawer, SyncMode
};
pub use error::*;
pub use diagnostics::*;

// Re-export procedural macros
pub use udonsharp_macros::*;