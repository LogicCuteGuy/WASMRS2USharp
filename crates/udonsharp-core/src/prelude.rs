//! Prelude module for common UdonSharp types and traits
//! 
//! This module re-exports commonly used types and traits for convenience.

pub use crate::traits::*;
pub use crate::types::{
    Vector2, Vector3, Vector4, Quaternion, Color, Color32,
    VRCPlayerApi, GameObject, Transform, Collider, Collision,
    UdonSyncMode, UdonFieldInfo, Rigidbody, Animator, AudioSource,
    Light, VRCStation, VRCPickup, VRCObjectPool, Text, Button,
    Slider, Toggle, UdonSharpUtility, Networking, Mathf, Random, Time
};
pub use crate::attributes::{
    UdonPublic, UdonSync, UdonEvent, UdonNetworkEvent,
    UdonHeader, UdonTooltip, UdonRange, UdonTextArea,
    UdonSpace, UdonPropertyDrawer, SyncMode
};
// Re-export NetworkEventTarget from types only to avoid ambiguity
pub use crate::types::NetworkEventTarget;
pub use crate::error::*;
pub use crate::diagnostics::*;