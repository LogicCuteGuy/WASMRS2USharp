//! Error handling for UdonSharp compilation and runtime
//! 
//! This module provides comprehensive error types for all stages of the
//! Rust to UdonSharp compilation pipeline.

use thiserror::Error;
use std::path::PathBuf;

/// Main error type for UdonSharp operations
#[derive(Error, Debug)]
pub enum UdonSharpError {
    /// Compilation errors during Rust to WASM compilation
    #[error("Compilation error: {message}")]
    Compilation {
        message: String,
        file: Option<PathBuf>,
        line: Option<u32>,
        column: Option<u32>,
    },
    
    /// WASM processing errors
    #[error("WASM error: {message}")]
    Wasm {
        message: String,
        module: Option<String>,
    },
    
    /// UdonSharp generation errors
    #[error("UdonSharp generation error: {message}")]
    Generation {
        message: String,
        context: Option<String>,
    },
    
    /// Binding generation errors
    #[error("Binding error: {message}")]
    Binding {
        message: String,
        assembly: Option<String>,
        type_name: Option<String>,
    },
    
    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        field: Option<String>,
    },
    
    /// Multi-behavior compilation errors
    #[error("Multi-behavior error: {message}")]
    MultiBehavior {
        message: String,
        behavior_name: Option<String>,
        suggestion: Option<String>,
    },
    
    /// Circular dependency errors
    #[error("Circular dependency detected: {cycle}")]
    CircularDependency {
        cycle: String,
        behaviors: Vec<String>,
        suggestion: Option<String>,
    },
    
    /// Attribute validation errors
    #[error("Invalid attribute: {message}")]
    InvalidAttribute {
        message: String,
        attribute_name: String,
        function_name: Option<String>,
        suggestion: Option<String>,
    },
    
    /// Behavior splitting errors
    #[error("Behavior splitting error: {message}")]
    BehaviorSplitting {
        message: String,
        source_function: Option<String>,
        target_behavior: Option<String>,
        suggestion: Option<String>,
    },
    
    /// Inter-behavior communication errors
    #[error("Inter-behavior communication error: {message}")]
    InterBehaviorCommunication {
        message: String,
        source_behavior: Option<String>,
        target_behavior: Option<String>,
        suggestion: Option<String>,
    },
    
    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// JSON parsing errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// Generic errors
    #[error("Error: {0}")]
    Generic(#[from] anyhow::Error),
}

impl UdonSharpError {
    /// Create a new compilation error
    pub fn compilation<S: Into<String>>(message: S) -> Self {
        Self::Compilation {
            message: message.into(),
            file: None,
            line: None,
            column: None,
        }
    }
    
    /// Create a compilation error with location information
    pub fn compilation_with_location<S: Into<String>>(
        message: S,
        file: PathBuf,
        line: u32,
        column: u32,
    ) -> Self {
        Self::Compilation {
            message: message.into(),
            file: Some(file),
            line: Some(line),
            column: Some(column),
        }
    }
    
    /// Create a new WASM error
    pub fn wasm<S: Into<String>>(message: S) -> Self {
        Self::Wasm {
            message: message.into(),
            module: None,
        }
    }
    
    /// Create a WASM error with module information
    pub fn wasm_with_module<S: Into<String>, M: Into<String>>(message: S, module: M) -> Self {
        Self::Wasm {
            message: message.into(),
            module: Some(module.into()),
        }
    }
    
    /// Create a new generation error
    pub fn generation<S: Into<String>>(message: S) -> Self {
        Self::Generation {
            message: message.into(),
            context: None,
        }
    }
    
    /// Create a generation error with context
    pub fn generation_with_context<S: Into<String>, C: Into<String>>(message: S, context: C) -> Self {
        Self::Generation {
            message: message.into(),
            context: Some(context.into()),
        }
    }
    
    /// Create a new binding error
    pub fn binding<S: Into<String>>(message: S) -> Self {
        Self::Binding {
            message: message.into(),
            assembly: None,
            type_name: None,
        }
    }
    
    /// Create a binding error with assembly and type information
    pub fn binding_with_type<S: Into<String>, A: Into<String>, T: Into<String>>(
        message: S,
        assembly: A,
        type_name: T,
    ) -> Self {
        Self::Binding {
            message: message.into(),
            assembly: Some(assembly.into()),
            type_name: Some(type_name.into()),
        }
    }
    
    /// Create a new configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
            field: None,
        }
    }
    
    /// Create a configuration error with field information
    pub fn configuration_with_field<S: Into<String>, F: Into<String>>(message: S, field: F) -> Self {
        Self::Configuration {
            message: message.into(),
            field: Some(field.into()),
        }
    }
    
    /// Create a multi-behavior error
    pub fn multi_behavior<S: Into<String>>(message: S) -> Self {
        Self::MultiBehavior {
            message: message.into(),
            behavior_name: None,
            suggestion: None,
        }
    }
    
    /// Create a multi-behavior error with behavior name and suggestion
    pub fn multi_behavior_with_suggestion<S: Into<String>, B: Into<String>, G: Into<String>>(
        message: S, 
        behavior_name: B, 
        suggestion: G
    ) -> Self {
        Self::MultiBehavior {
            message: message.into(),
            behavior_name: Some(behavior_name.into()),
            suggestion: Some(suggestion.into()),
        }
    }
    
    /// Create a circular dependency error
    pub fn circular_dependency<S: Into<String>>(cycle: S, behaviors: Vec<String>) -> Self {
        let suggestion = format!(
            "Consider refactoring to break the dependency cycle. You can:\n\
            1. Move shared functionality to a common base class or utility\n\
            2. Use events or interfaces to decouple the behaviors\n\
            3. Restructure the code to eliminate circular references"
        );
        
        Self::CircularDependency {
            cycle: cycle.into(),
            behaviors,
            suggestion: Some(suggestion),
        }
    }
    
    /// Create an invalid attribute error
    pub fn invalid_attribute<S: Into<String>, A: Into<String>>(message: S, attribute_name: A) -> Self {
        Self::InvalidAttribute {
            message: message.into(),
            attribute_name: attribute_name.into(),
            function_name: None,
            suggestion: None,
        }
    }
    
    /// Create an invalid attribute error with function name and suggestion
    pub fn invalid_attribute_with_suggestion<S: Into<String>, A: Into<String>, F: Into<String>, G: Into<String>>(
        message: S, 
        attribute_name: A, 
        function_name: F, 
        suggestion: G
    ) -> Self {
        Self::InvalidAttribute {
            message: message.into(),
            attribute_name: attribute_name.into(),
            function_name: Some(function_name.into()),
            suggestion: Some(suggestion.into()),
        }
    }
    
    /// Create a behavior splitting error
    pub fn behavior_splitting<S: Into<String>>(message: S) -> Self {
        Self::BehaviorSplitting {
            message: message.into(),
            source_function: None,
            target_behavior: None,
            suggestion: None,
        }
    }
    
    /// Create a behavior splitting error with context and suggestion
    pub fn behavior_splitting_with_context<S: Into<String>, F: Into<String>, B: Into<String>, G: Into<String>>(
        message: S, 
        source_function: F, 
        target_behavior: B, 
        suggestion: G
    ) -> Self {
        Self::BehaviorSplitting {
            message: message.into(),
            source_function: Some(source_function.into()),
            target_behavior: Some(target_behavior.into()),
            suggestion: Some(suggestion.into()),
        }
    }
    
    /// Create an inter-behavior communication error
    pub fn inter_behavior_communication<S: Into<String>>(message: S) -> Self {
        Self::InterBehaviorCommunication {
            message: message.into(),
            source_behavior: None,
            target_behavior: None,
            suggestion: None,
        }
    }
    
    /// Create an inter-behavior communication error with context and suggestion
    pub fn inter_behavior_communication_with_context<S: Into<String>, S1: Into<String>, S2: Into<String>, G: Into<String>>(
        message: S, 
        source_behavior: S1, 
        target_behavior: S2, 
        suggestion: G
    ) -> Self {
        Self::InterBehaviorCommunication {
            message: message.into(),
            source_behavior: Some(source_behavior.into()),
            target_behavior: Some(target_behavior.into()),
            suggestion: Some(suggestion.into()),
        }
    }
}

/// Result type for UdonSharp operations
pub type UdonSharpResult<T> = Result<T, UdonSharpError>;

// Error conversion implementations
impl From<syn::Error> for UdonSharpError {
    fn from(error: syn::Error) -> Self {
        UdonSharpError::compilation(format!("Syntax error: {}", error))
    }
}

/// Diagnostic information for compilation issues
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub file: Option<PathBuf>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub code: Option<String>,
    pub help: Option<String>,
}

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
    Hint,
}

impl Diagnostic {
    /// Create a new error diagnostic
    pub fn error<S: Into<String>>(message: S) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: message.into(),
            file: None,
            line: None,
            column: None,
            code: None,
            help: None,
        }
    }
    
    /// Create a new warning diagnostic
    pub fn warning<S: Into<String>>(message: S) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message: message.into(),
            file: None,
            line: None,
            column: None,
            code: None,
            help: None,
        }
    }
    
    /// Create a new info diagnostic
    pub fn info<S: Into<String>>(message: S) -> Self {
        Self {
            level: DiagnosticLevel::Info,
            message: message.into(),
            file: None,
            line: None,
            column: None,
            code: None,
            help: None,
        }
    }
    
    /// Add location information to the diagnostic
    pub fn with_location(mut self, file: PathBuf, line: u32, column: u32) -> Self {
        self.file = Some(file);
        self.line = Some(line);
        self.column = Some(column);
        self
    }
    
    /// Add error code to the diagnostic
    pub fn with_code<S: Into<String>>(mut self, code: S) -> Self {
        self.code = Some(code.into());
        self
    }
    
    /// Add help text to the diagnostic
    pub fn with_help<S: Into<String>>(mut self, help: S) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// Diagnostic collector for gathering compilation issues
#[derive(Debug, Default, Clone)]
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    /// Create a new diagnostic collector
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a diagnostic to the collector
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
    
    /// Add an error diagnostic
    pub fn error<S: Into<String>>(&mut self, message: S) {
        self.add(Diagnostic::error(message));
    }
    
    /// Add a warning diagnostic
    pub fn warning<S: Into<String>>(&mut self, message: S) {
        self.add(Diagnostic::warning(message));
    }
    
    /// Add an info diagnostic
    pub fn info<S: Into<String>>(&mut self, message: S) {
        self.add(Diagnostic::info(message));
    }
    
    /// Get all diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.level == DiagnosticLevel::Error)
    }
    
    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter().any(|d| d.level == DiagnosticLevel::Warning)
    }
    
    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.level == DiagnosticLevel::Error).count()
    }
    
    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.level == DiagnosticLevel::Warning).count()
    }
    
    /// Clear all diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
    
    /// Convert to a result, returning an error if there are any error diagnostics
    pub fn into_result(self) -> UdonSharpResult<Vec<Diagnostic>> {
        if self.has_errors() {
            let error_messages: Vec<String> = self.diagnostics
                .iter()
                .filter(|d| d.level == DiagnosticLevel::Error)
                .map(|d| d.message.clone())
                .collect();
            
            Err(UdonSharpError::compilation(format!(
                "Compilation failed with {} error(s): {}",
                error_messages.len(),
                error_messages.join("; ")
            )))
        } else {
            Ok(self.diagnostics)
        }
    }
}

impl std::fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticLevel::Error => write!(f, "error"),
            DiagnosticLevel::Warning => write!(f, "warning"),
            DiagnosticLevel::Info => write!(f, "info"),
            DiagnosticLevel::Hint => write!(f, "hint"),
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.level, self.message)?;
        
        if let (Some(file), Some(line), Some(column)) = (&self.file, self.line, self.column) {
            write!(f, " at {}:{}:{}", file.display(), line, column)?;
        }
        
        if let Some(code) = &self.code {
            write!(f, " [{}]", code)?;
        }
        
        if let Some(help) = &self.help {
            write!(f, "\n  help: {}", help)?;
        }
        
        Ok(())
    }
}

/// Compilation context for tracking progress and diagnostics
#[derive(Debug, Clone)]
pub struct CompilationContext {
    pub reporter: DiagnosticCollector,
    debug_mode: bool,
    verbose: bool,
}

impl CompilationContext {
    /// Create a new compilation context
    pub fn new() -> Self {
        Self {
            reporter: DiagnosticCollector::new(),
            debug_mode: false,
            verbose: false,
        }
    }
    
    /// Create a debug compilation context
    pub fn debug() -> Self {
        Self {
            reporter: DiagnosticCollector::new(),
            debug_mode: true,
            verbose: true,
        }
    }
    
    /// Initialize logging for this context
    pub fn init_logging(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Logging is already initialized in main, so this is a no-op
        Ok(())
    }
    
    /// Log an info message
    pub fn info<S: Into<String>>(&self, message: S) {
        let msg = message.into();
        log::info!("{}", msg);
        if self.verbose {
            println!("ℹ️  {}", msg);
        }
    }
    
    /// Log a warning message
    pub fn warning<S: Into<String>>(&self, message: S) {
        let msg = message.into();
        log::warn!("{}", msg);
        println!("⚠️  {}", msg);
    }
    
    /// Log an error message
    pub fn error<S: Into<String>>(&self, message: S) {
        let msg = message.into();
        log::error!("{}", msg);
        println!("❌ {}", msg);
    }
    
    /// Check if compilation should continue
    pub fn should_continue(&self) -> bool {
        !self.reporter.has_errors()
    }
    
    /// Print a summary of the compilation
    pub fn print_summary(&self) {
        let error_count = self.reporter.error_count();
        let warning_count = self.reporter.warning_count();
        
        if error_count > 0 {
            println!("❌ Compilation failed with {} error(s)", error_count);
        } else {
            println!("✅ Compilation completed successfully");
        }
        
        if warning_count > 0 {
            println!("⚠️  {} warning(s)", warning_count);
        }
    }
    
    /// Print detailed diagnostics
    pub fn print_detailed_diagnostics(&self) {
        for diagnostic in self.reporter.diagnostics() {
            println!("{}", diagnostic);
        }
    }
    
    /// Print test summary
    pub fn print_test_summary(&self) {
        println!("🧪 Test summary:");
        // TODO: Implement test result tracking
        println!("   Tests: 0 passed, 0 failed");
    }
}

impl Default for CompilationContext {
    fn default() -> Self {
        Self::new()
    }
}