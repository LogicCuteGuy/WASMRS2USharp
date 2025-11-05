//! Rust to WASM to UdonSharp compilation pipeline
//! 
//! This crate provides the core compilation pipeline that transforms Rust code
//! into UdonSharp-compatible C# code through WebAssembly.

pub mod config;
pub mod pipeline;
pub mod wasm_compiler;
pub mod optimizer;
pub mod prefab_generator;
pub mod initialization_coordinator;

pub use config::*;
pub use pipeline::*;
pub use prefab_generator::*;
pub use initialization_coordinator::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod pipeline_tests;

#[cfg(test)]
mod multi_behavior_tests;