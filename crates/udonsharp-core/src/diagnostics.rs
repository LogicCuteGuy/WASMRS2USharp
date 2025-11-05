//! Diagnostic and logging system for UdonSharp compilation
//! 
//! This module provides structured logging and diagnostic reporting
//! for the UdonSharp compilation pipeline.

use crate::error::{Diagnostic, DiagnosticLevel, DiagnosticCollector};
use log::{Level, Record};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Diagnostic reporter that integrates with the logging system
pub struct DiagnosticReporter {
    collector: Arc<Mutex<DiagnosticCollector>>,
    min_level: DiagnosticLevel,
}

impl DiagnosticReporter {
    /// Create a new diagnostic reporter
    pub fn new(min_level: DiagnosticLevel) -> Self {
        Self {
            collector: Arc::new(Mutex::new(DiagnosticCollector::new())),
            min_level,
        }
    }
    
    /// Report a diagnostic
    pub fn report(&self, diagnostic: Diagnostic) {
        if self.should_report(&diagnostic.level) {
            if let Ok(mut collector) = self.collector.lock() {
                collector.add(diagnostic);
            }
        }
    }
    
    /// Report an error
    pub fn error<S: Into<String>>(&self, message: S) {
        self.report(Diagnostic::error(message));
    }
    
    /// Report a warning
    pub fn warning<S: Into<String>>(&self, message: S) {
        self.report(Diagnostic::warning(message));
    }
    
    /// Report an info message
    pub fn info<S: Into<String>>(&self, message: S) {
        self.report(Diagnostic::info(message));
    }
    
    /// Get all collected diagnostics
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        if let Ok(collector) = self.collector.lock() {
            collector.diagnostics().to_vec()
        } else {
            Vec::new()
        }
    }
    
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        if let Ok(collector) = self.collector.lock() {
            collector.has_errors()
        } else {
            false
        }
    }
    
    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        if let Ok(collector) = self.collector.lock() {
            collector.has_warnings()
        } else {
            false
        }
    }
    
    /// Get error count
    pub fn error_count(&self) -> usize {
        if let Ok(collector) = self.collector.lock() {
            collector.error_count()
        } else {
            0
        }
    }
    
    /// Get warning count
    pub fn warning_count(&self) -> usize {
        if let Ok(collector) = self.collector.lock() {
            collector.warning_count()
        } else {
            0
        }
    }
    
    /// Clear all diagnostics
    pub fn clear(&self) {
        if let Ok(mut collector) = self.collector.lock() {
            collector.clear();
        }
    }
    
    fn should_report(&self, level: &DiagnosticLevel) -> bool {
        match (level, &self.min_level) {
            (DiagnosticLevel::Error, _) => true,
            (DiagnosticLevel::Warning, DiagnosticLevel::Error) => false,
            (DiagnosticLevel::Warning, _) => true,
            (DiagnosticLevel::Info, DiagnosticLevel::Error | DiagnosticLevel::Warning) => false,
            (DiagnosticLevel::Info, _) => true,
            (DiagnosticLevel::Hint, DiagnosticLevel::Hint) => true,
            (DiagnosticLevel::Hint, _) => false,
        }
    }
}

/// Custom log formatter for UdonSharp compilation
pub struct UdonSharpLogger {
    reporter: Option<Arc<DiagnosticReporter>>,
}

impl UdonSharpLogger {
    /// Create a new UdonSharp logger
    pub fn new() -> Self {
        Self { reporter: None }
    }
    
    /// Create a logger with diagnostic reporting
    pub fn with_reporter(reporter: Arc<DiagnosticReporter>) -> Self {
        Self {
            reporter: Some(reporter),
        }
    }
    
    /// Initialize the logger as the global logger
    pub fn init(self) -> Result<(), log::SetLoggerError> {
        log::set_boxed_logger(Box::new(self))?;
        log::set_max_level(log::LevelFilter::Info);
        Ok(())
    }
    
    /// Initialize with a specific log level
    pub fn init_with_level(self, level: log::LevelFilter) -> Result<(), log::SetLoggerError> {
        log::set_boxed_logger(Box::new(self))?;
        log::set_max_level(level);
        Ok(())
    }
}

impl Default for UdonSharpLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl log::Log for UdonSharpLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }
    
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_str = match record.level() {
                Level::Error => "ERROR",
                Level::Warn => "WARN ",
                Level::Info => "INFO ",
                Level::Debug => "DEBUG",
                Level::Trace => "TRACE",
            };
            
            let target = if record.target().is_empty() {
                "udonsharp"
            } else {
                record.target()
            };
            
            println!("[{}] {}: {}", level_str, target, record.args());
            
            // Report to diagnostic collector if available
            if let Some(reporter) = &self.reporter {
                let diagnostic_level = match record.level() {
                    Level::Error => DiagnosticLevel::Error,
                    Level::Warn => DiagnosticLevel::Warning,
                    Level::Info => DiagnosticLevel::Info,
                    Level::Debug | Level::Trace => DiagnosticLevel::Hint,
                };
                
                let diagnostic = Diagnostic {
                    level: diagnostic_level,
                    message: format!("{}", record.args()),
                    file: record.file().map(PathBuf::from),
                    line: record.line(),
                    column: None,
                    code: None,
                    help: None,
                };
                
                reporter.report(diagnostic);
            }
        }
    }
    
    fn flush(&self) {
        // Nothing to flush for stdout
    }
}

/// Compilation context that tracks diagnostics and configuration
pub struct CompilationContext {
    pub reporter: Arc<DiagnosticReporter>,
    pub verbose: bool,
    pub debug: bool,
}

impl CompilationContext {
    /// Create a new compilation context
    pub fn new() -> Self {
        Self {
            reporter: Arc::new(DiagnosticReporter::new(DiagnosticLevel::Warning)),
            verbose: false,
            debug: false,
        }
    }
    
    /// Create a context with verbose output
    pub fn verbose() -> Self {
        Self {
            reporter: Arc::new(DiagnosticReporter::new(DiagnosticLevel::Info)),
            verbose: true,
            debug: false,
        }
    }
    
    /// Create a context with debug output
    pub fn debug() -> Self {
        Self {
            reporter: Arc::new(DiagnosticReporter::new(DiagnosticLevel::Hint)),
            verbose: true,
            debug: true,
        }
    }
    
    /// Initialize logging for this context
    pub fn init_logging(&self) -> Result<(), log::SetLoggerError> {
        let level = if self.debug {
            log::LevelFilter::Debug
        } else if self.verbose {
            log::LevelFilter::Info
        } else {
            log::LevelFilter::Warn
        };
        
        UdonSharpLogger::with_reporter(self.reporter.clone()).init_with_level(level)
    }
    
    /// Report an error
    pub fn error<S: Into<String>>(&self, message: S) {
        self.reporter.error(message);
    }
    
    /// Report a warning
    pub fn warning<S: Into<String>>(&self, message: S) {
        self.reporter.warning(message);
    }
    
    /// Report an info message
    pub fn info<S: Into<String>>(&self, message: S) {
        self.reporter.info(message);
    }
    
    /// Check if compilation should continue (no errors)
    pub fn should_continue(&self) -> bool {
        !self.reporter.has_errors()
    }
    
    /// Print a summary of diagnostics
    pub fn print_summary(&self) {
        let error_count = self.reporter.error_count();
        let warning_count = self.reporter.warning_count();
        
        if error_count > 0 || warning_count > 0 {
            println!();
            if error_count > 0 {
                println!("Compilation failed with {} error(s)", error_count);
            }
            if warning_count > 0 {
                println!("Generated {} warning(s)", warning_count);
            }
            
            if self.verbose {
                println!("\nDiagnostics:");
                for diagnostic in self.reporter.diagnostics() {
                    println!("  {}", diagnostic);
                }
            }
        } else if self.verbose {
            println!("Compilation completed successfully with no issues");
        }
    }
}

impl Default for CompilationContext {
    fn default() -> Self {
        Self::new()
    }
}