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
pub mod multi_behavior;
pub mod struct_analyzer;
pub mod trait_validator;
pub mod behavior_dependency_analyzer;
pub mod code_generator;
pub mod inter_behavior_communication;
pub mod shared_runtime;
pub mod error_detection;
pub mod error_reporting;
pub mod runtime_validation;
pub mod comprehensive_error_system;
pub mod standard_multi_behavior_integration;
pub mod debug_info_generator;
pub mod dependency_analyzer_tool;
pub mod compilation_reporter;

pub use config::*;
pub use pipeline::*;
pub use prefab_generator::*;
pub use initialization_coordinator::*;
pub use multi_behavior::*;
pub use struct_analyzer::*;
pub use behavior_dependency_analyzer::*;
pub use code_generator::*;
pub use inter_behavior_communication::*;
pub use shared_runtime::*;
pub use error_detection::*;
pub use error_reporting::*;
pub use runtime_validation::*;
pub use comprehensive_error_system::*;
pub use standard_multi_behavior_integration::*;
pub use debug_info_generator::*;
pub use dependency_analyzer_tool::*;
pub use compilation_reporter::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod code_generator_test;

#[cfg(test)]
mod pipeline_tests;

#[cfg(test)]
mod multi_behavior_tests;

#[cfg(test)]
mod performance_validation_tests;