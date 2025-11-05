//! Auto-generated bindings for VRChat, Unity, and C# APIs
//! 
//! This crate provides Rust bindings for UdonSharp-compatible APIs from:
//! - VRChat SDK (VRC.SDK3, VRC.Udon)
//! - Unity Engine (UnityEngine.*)
//! - C# System Libraries (System.*)

pub mod asmdef;
pub mod analyzer;
pub mod generator;
pub mod compatibility;

// Re-export core types
pub use udonsharp_core::*;

// VRChat/Udon API bindings
pub mod vrchat;

// Unity Engine API bindings
pub mod unity;

// C# System Library bindings
pub mod csharp;

// Re-export commonly used types for convenience
pub use generator::*;
pub use compatibility::*;
pub use asmdef::*;

// Re-export API bindings for easy access
pub use vrchat::*;
pub use unity::*;
pub use csharp::*;