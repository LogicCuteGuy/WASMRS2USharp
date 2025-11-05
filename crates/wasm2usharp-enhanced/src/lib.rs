//! Enhanced wasm2usharp with OOP behavior analysis
//! 
//! This crate extends the existing wasm2usharp functionality with object-oriented
//! programming pattern detection and transformation capabilities.

pub mod analyzer;
pub mod transformer;
pub mod splitter;
pub mod file_generator;
pub mod dependency_analyzer;

#[cfg(test)]
mod tests;

pub use analyzer::*;
pub use transformer::{EnhancedWasm2USharp, ConversionConfig, ConversionResult};
pub use splitter::*;
pub use file_generator::*;
pub use dependency_analyzer::*;

use anyhow::Result;

/// Main entry point for enhanced WASM to UdonSharp conversion
pub struct EnhancedWasm2USharpPipeline {
    analyzer: OopBehaviorAnalyzer,
    converter: EnhancedWasm2USharp,
}

impl EnhancedWasm2USharpPipeline {
    /// Create a new enhanced conversion pipeline
    pub fn new() -> Self {
        Self {
            analyzer: OopBehaviorAnalyzer::new(),
            converter: EnhancedWasm2USharp::new(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: ConversionConfig) -> Self {
        Self {
            analyzer: OopBehaviorAnalyzer::new(),
            converter: EnhancedWasm2USharp::with_config(config),
        }
    }
    
    /// Convert WASM to UdonSharp with full OOP analysis and transformation
    pub fn convert(&mut self, wasm_bytes: &[u8]) -> Result<ConversionResult> {
        // Step 1: Analyze WASM for OOP patterns
        let analysis = self.analyzer.analyze(wasm_bytes)?;
        
        // Step 2: Convert with OOP transformations
        let result = self.converter.convert_with_oop(wasm_bytes, &analysis)?;
        
        Ok(result)
    }
}

impl Default for EnhancedWasm2USharpPipeline {
    fn default() -> Self {
        Self::new()
    }
}