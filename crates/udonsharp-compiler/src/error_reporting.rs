//! User-friendly error reporting system with contextual suggestions
//! 
//! This module provides comprehensive error reporting with clear messages,
//! code examples, and actionable suggestions for developers.

use crate::error_detection::{CompilationError, CompilationErrorReport, ErrorType, ErrorCategory, SourceLocation};
use udonsharp_core::error::{DiagnosticLevel, Diagnostic};
use std::collections::HashMap;
use std::path::PathBuf;

/// User-friendly error reporter with contextual suggestions and examples
pub struct ErrorReporter {
    /// Error message templates
    message_templates: HashMap<ErrorType, MessageTemplate>,
    /// Color support for terminal output
    use_colors: bool,
    /// Verbose mode for detailed output
    verbose: bool,
}

impl ErrorReporter {
    /// Create a new error reporter
    pub fn new() -> Self {
        let mut reporter = Self {
            message_templates: HashMap::new(),
            use_colors: true,
            verbose: false,
        };
        
        reporter.initialize_message_templates();
        reporter
    }
    
    /// Create a reporter with specific configuration
    pub fn with_config(use_colors: bool, verbose: bool) -> Self {
        let mut reporter = Self::new();
        reporter.use_colors = use_colors;
        reporter.verbose = verbose;
        reporter
    }
    
    /// Initialize error message templates
    fn initialize_message_templates(&mut self) {
        self.message_templates.insert(
            ErrorType::MissingTraitImplementation,
            MessageTemplate {
                title: "Missing UdonBehaviour Trait Implementation".to_string(),
                description: "Your struct needs to implement the UdonBehaviour trait to work with UdonSharp".to_string(),
                icon: "🚫".to_string(),
                color: TerminalColor::Red,
                quick_fix: Some("Add impl UdonBehaviour for YourStruct { ... }".to_string()),
                help_url: Some("https://docs.vrchat.com/docs/udonsharp-getting-started".to_string()),
            }
        );
        
        self.message_templates.insert(
            ErrorType::MissingRequiredMethods,
            MessageTemplate {
                title: "Missing Required Methods".to_string(),
                description: "The UdonBehaviour trait requires certain methods to be implemented".to_string(),
                icon: "⚠️".to_string(),
                color: TerminalColor::Yellow,
                quick_fix: Some("Add the missing methods to your impl block".to_string()),
                help_url: Some("https://docs.vrchat.com/docs/udonsharp-behaviour-methods".to_string()),
            }
        );
        
        self.message_templates.insert(
            ErrorType::InvalidAttributeUsage,
            MessageTemplate {
                title: "Invalid Attribute Usage".to_string(),
                description: "The attribute you're using is not valid in this context or has incorrect parameters".to_string(),
                icon: "🏷️".to_string(),
                color: TerminalColor::Red,
                quick_fix: Some("Check the attribute documentation for correct usage".to_string()),
                help_url: Some("https://docs.vrchat.com/docs/udonsharp-attributes".to_string()),
            }
        );
        
        self.message_templates.insert(
            ErrorType::DuplicateAttribute,
            MessageTemplate {
                title: "Duplicate Attribute".to_string(),
                description: "You have the same attribute applied multiple times to the same field".to_string(),
                icon: "🔄".to_string(),
                color: TerminalColor::Yellow,
                quick_fix: Some("Remove the duplicate attribute".to_string()),
                help_url: None,
            }
        );
        
        self.message_templates.insert(
            ErrorType::UnsupportedType,
            MessageTemplate {
                title: "Unsupported Type".to_string(),
                description: "This Rust type is not supported by UdonSharp and cannot be compiled".to_string(),
                icon: "🚫".to_string(),
                color: TerminalColor::Red,
                quick_fix: Some("Use a supported type or wrap in a compatible container".to_string()),
                help_url: Some("https://docs.vrchat.com/docs/udonsharp-supported-types".to_string()),
            }
        );
        
        self.message_templates.insert(
            ErrorType::UnsupportedFeature,
            MessageTemplate {
                title: "Unsupported Feature".to_string(),
                description: "This Rust feature is not supported in UdonSharp compilation".to_string(),
                icon: "🚫".to_string(),
                color: TerminalColor::Red,
                quick_fix: Some("Use alternative approaches supported by UdonSharp".to_string()),
                help_url: Some("https://docs.vrchat.com/docs/udonsharp-limitations".to_string()),
            }
        );
    }
    
    /// Generate a comprehensive error report
    pub fn generate_report(&self, error_report: &CompilationErrorReport) -> String {
        let mut output = Vec::new();
        
        // Header
        output.push(self.format_header(error_report));
        output.push("".to_string());
        
        // Summary
        output.push(self.format_summary(error_report));
        output.push("".to_string());
        
        // Errors by category
        if !error_report.errors.is_empty() {
            output.push(self.format_errors_by_category(error_report));
            output.push("".to_string());
        }
        
        // Detailed error list
        if self.verbose || error_report.error_count <= 5 {
            output.push(self.format_detailed_errors(&error_report.errors));
            output.push("".to_string());
        }
        
        // Footer with next steps
        output.push(self.format_footer(error_report));
        
        output.join("\n")
    }
    
    /// Generate a single error message with context
    pub fn format_error(&self, error: &CompilationError) -> String {
        let template = self.message_templates.get(&error.error_type)
            .cloned()
            .unwrap_or_else(|| self.get_default_template());
        
        let mut output = Vec::new();
        
        // Error header with icon and title
        let header = if self.use_colors {
            format!("{} {} {}", 
                template.icon,
                self.colorize(&template.title, template.color),
                self.colorize(&format!("[{}]", self.format_severity(&error.severity)), TerminalColor::Gray)
            )
        } else {
            format!("{} {} [{}]", template.icon, template.title, self.format_severity(&error.severity))
        };
        output.push(header);
        
        // Error message
        output.push(format!("   {}", error.message));
        
        // Location information
        if let Some(location) = &error.source_location {
            let location_str = if self.use_colors {
                self.colorize(&format!("   --> {}:{}:{}", location.file, location.line, location.column), TerminalColor::Blue)
            } else {
                format!("   --> {}:{}:{}", location.file, location.line, location.column)
            };
            output.push(location_str);
        }
        
        // Context information
        if let Some(struct_name) = &error.struct_name {
            output.push(format!("   in struct: {}", struct_name));
        }
        if let Some(field_name) = &error.field_name {
            output.push(format!("   on field: {}", field_name));
        }
        if let Some(method_name) = &error.method_name {
            output.push(format!("   in method: {}", method_name));
        }
        
        // Suggestion
        if let Some(suggestion) = &error.suggestion {
            output.push("".to_string());
            let suggestion_header = if self.use_colors {
                self.colorize("💡 Suggestion:", TerminalColor::Green)
            } else {
                "💡 Suggestion:".to_string()
            };
            output.push(suggestion_header);
            
            // Format multi-line suggestions with proper indentation
            for line in suggestion.lines() {
                output.push(format!("   {}", line));
            }
        }
        
        // Code example
        if let Some(code_example) = &error.code_example {
            output.push("".to_string());
            let example_header = if self.use_colors {
                self.colorize("📝 Example:", TerminalColor::Cyan)
            } else {
                "📝 Example:".to_string()
            };
            output.push(example_header);
            
            // Format code with syntax highlighting (basic)
            for line in code_example.lines() {
                let formatted_line = if self.use_colors {
                    self.format_code_line(line)
                } else {
                    format!("   {}", line)
                };
                output.push(formatted_line);
            }
        }
        
        // Help URL
        if let Some(help_url) = &template.help_url {
            output.push("".to_string());
            let help_text = if self.use_colors {
                format!("📚 Learn more: {}", self.colorize(help_url, TerminalColor::Blue))
            } else {
                format!("📚 Learn more: {}", help_url)
            };
            output.push(help_text);
        }
        
        output.join("\n")
    }
    
    /// Format error header
    fn format_header(&self, error_report: &CompilationErrorReport) -> String {
        let title = "UdonSharp Compilation Analysis";
        let separator = "═".repeat(title.len() + 4);
        
        if self.use_colors {
            format!("{}\n  {}\n{}", 
                self.colorize(&separator, TerminalColor::Blue),
                self.colorize(title, TerminalColor::Blue),
                self.colorize(&separator, TerminalColor::Blue)
            )
        } else {
            format!("{}\n  {}\n{}", separator, title, separator)
        }
    }
    
    /// Format error summary
    fn format_summary(&self, error_report: &CompilationErrorReport) -> String {
        let mut summary = Vec::new();
        
        let status_icon = if error_report.has_blocking_errors { "❌" } else { "✅" };
        let status_text = if error_report.has_blocking_errors {
            "Compilation blocked by errors"
        } else {
            "Analysis completed successfully"
        };
        
        summary.push(format!("{} {}", status_icon, status_text));
        summary.push("".to_string());
        
        // Statistics
        summary.push(format!("📊 Analysis Results:"));
        summary.push(format!("   • Structs analyzed: {}", error_report.structs_analyzed));
        summary.push(format!("   • Errors found: {}", error_report.error_count));
        summary.push(format!("   • Warnings: {}", error_report.warning_count));
        
        if error_report.has_blocking_errors {
            summary.push("".to_string());
            let blocking_msg = if self.use_colors {
                self.colorize("⚠️  Compilation cannot proceed until errors are fixed", TerminalColor::Red)
            } else {
                "⚠️  Compilation cannot proceed until errors are fixed".to_string()
            };
            summary.push(blocking_msg);
        }
        
        summary.join("\n")
    }
    
    /// Format errors grouped by category
    fn format_errors_by_category(&self, error_report: &CompilationErrorReport) -> String {
        let mut output = Vec::new();
        
        output.push("📋 Issues by Category:".to_string());
        
        for (category, errors) in &error_report.errors_by_category {
            let category_name = self.format_category_name(category);
            let error_count = errors.iter().filter(|e| e.severity == DiagnosticLevel::Error).count();
            let warning_count = errors.iter().filter(|e| e.severity == DiagnosticLevel::Warning).count();
            
            let category_line = if error_count > 0 {
                format!("   {} {} ({} errors, {} warnings)", 
                    self.get_category_icon(category), category_name, error_count, warning_count)
            } else {
                format!("   {} {} ({} warnings)", 
                    self.get_category_icon(category), category_name, warning_count)
            };
            
            output.push(category_line);
            
            // Show top issues in each category
            let top_issues: Vec<_> = errors.iter().take(3).collect();
            for error in top_issues {
                let issue_line = format!("     • {}", self.truncate_message(&error.message, 60));
                output.push(issue_line);
            }
            
            if errors.len() > 3 {
                output.push(format!("     ... and {} more", errors.len() - 3));
            }
        }
        
        output.join("\n")
    }
    
    /// Format detailed error list
    fn format_detailed_errors(&self, errors: &[CompilationError]) -> String {
        let mut output = Vec::new();
        
        output.push("🔍 Detailed Issues:".to_string());
        output.push("".to_string());
        
        for (index, error) in errors.iter().enumerate() {
            output.push(format!("{}. {}", index + 1, self.format_error(error)));
            output.push("".to_string());
            
            // Add separator between errors
            if index < errors.len() - 1 {
                let separator = if self.use_colors {
                    self.colorize("─".repeat(50).as_str(), TerminalColor::Gray)
                } else {
                    "─".repeat(50)
                };
                output.push(separator);
                output.push("".to_string());
            }
        }
        
        output.join("\n")
    }
    
    /// Format footer with next steps
    fn format_footer(&self, error_report: &CompilationErrorReport) -> String {
        let mut footer = Vec::new();
        
        if error_report.has_blocking_errors {
            footer.push("🛠️  Next Steps:".to_string());
            footer.push("   1. Fix the errors listed above".to_string());
            footer.push("   2. Run the compilation again".to_string());
            footer.push("   3. Check the documentation links for detailed guidance".to_string());
        } else {
            footer.push("🎉 Great! Your code is ready for compilation.".to_string());
            if error_report.warning_count > 0 {
                footer.push("   Consider addressing the warnings for better code quality.".to_string());
            }
        }
        
        footer.push("".to_string());
        footer.push("📚 Additional Resources:".to_string());
        footer.push("   • UdonSharp Documentation: https://docs.vrchat.com/docs/udonsharp".to_string());
        footer.push("   • VRChat Creator Companion: https://vcc.docs.vrchat.com/".to_string());
        footer.push("   • Community Discord: https://discord.gg/vrchat".to_string());
        
        footer.join("\n")
    }
    
    /// Generate quick fix suggestions
    pub fn generate_quick_fixes(&self, errors: &[CompilationError]) -> Vec<QuickFix> {
        let mut fixes = Vec::new();
        
        for error in errors {
            if let Some(template) = self.message_templates.get(&error.error_type) {
                if let Some(quick_fix) = &template.quick_fix {
                    fixes.push(QuickFix {
                        title: quick_fix.clone(),
                        description: template.description.clone(),
                        error_type: error.error_type.clone(),
                        applicable_to: error.struct_name.clone(),
                        code_action: self.generate_code_action(error),
                    });
                }
            }
        }
        
        fixes
    }
    
    /// Generate IDE-compatible diagnostics
    pub fn generate_diagnostics(&self, errors: &[CompilationError]) -> Vec<Diagnostic> {
        errors.iter().map(|error| {
            let mut diagnostic = Diagnostic::error(&error.message);
            
            if let Some(location) = &error.source_location {
                diagnostic = diagnostic.with_location(
                    PathBuf::from(&location.file),
                    location.line,
                    location.column
                );
            }
            
            if let Some(suggestion) = &error.suggestion {
                diagnostic = diagnostic.with_help(suggestion);
            }
            
            diagnostic = diagnostic.with_code(format!("E{:04}", self.get_error_code(&error.error_type)));
            
            diagnostic
        }).collect()
    }
    
    // Helper methods
    
    fn get_default_template(&self) -> MessageTemplate {
        MessageTemplate {
            title: "Compilation Error".to_string(),
            description: "An error occurred during compilation".to_string(),
            icon: "❌".to_string(),
            color: TerminalColor::Red,
            quick_fix: None,
            help_url: None,
        }
    }
    
    fn colorize(&self, text: &str, color: TerminalColor) -> String {
        if !self.use_colors {
            return text.to_string();
        }
        
        let color_code = match color {
            TerminalColor::Red => "\x1b[31m",
            TerminalColor::Green => "\x1b[32m",
            TerminalColor::Yellow => "\x1b[33m",
            TerminalColor::Blue => "\x1b[34m",
            TerminalColor::Cyan => "\x1b[36m",
            TerminalColor::Gray => "\x1b[90m",
        };
        
        format!("{}{}\x1b[0m", color_code, text)
    }
    
    fn format_code_line(&self, line: &str) -> String {
        // Basic syntax highlighting for Rust code
        let mut formatted = format!("   {}", line);
        
        if self.use_colors {
            // Highlight keywords
            formatted = formatted.replace("fn ", &format!("{}fn{} ", "\x1b[35m", "\x1b[0m"));
            formatted = formatted.replace("impl ", &format!("{}impl{} ", "\x1b[35m", "\x1b[0m"));
            formatted = formatted.replace("pub ", &format!("{}pub{} ", "\x1b[35m", "\x1b[0m"));
            formatted = formatted.replace("struct ", &format!("{}struct{} ", "\x1b[35m", "\x1b[0m"));
            
            // Highlight strings
            if line.contains('"') {
                // Simple string highlighting (not perfect but good enough)
                formatted = formatted.replace("\"", &format!("{}\"{}", "\x1b[32m", "\x1b[0m"));
            }
        }
        
        formatted
    }
    
    fn format_severity(&self, severity: &DiagnosticLevel) -> String {
        match severity {
            DiagnosticLevel::Error => "ERROR".to_string(),
            DiagnosticLevel::Warning => "WARNING".to_string(),
            DiagnosticLevel::Info => "INFO".to_string(),
            DiagnosticLevel::Hint => "HINT".to_string(),
        }
    }
    
    fn format_category_name(&self, category: &ErrorCategory) -> String {
        match category {
            ErrorCategory::TraitImplementation => "Trait Implementation".to_string(),
            ErrorCategory::AttributeUsage => "Attribute Usage".to_string(),
            ErrorCategory::TypeValidation => "Type Validation".to_string(),
        }
    }
    
    fn get_category_icon(&self, category: &ErrorCategory) -> &str {
        match category {
            ErrorCategory::TraitImplementation => "🔧",
            ErrorCategory::AttributeUsage => "🏷️",
            ErrorCategory::TypeValidation => "🔍",
        }
    }
    
    fn truncate_message(&self, message: &str, max_length: usize) -> String {
        if message.len() <= max_length {
            message.to_string()
        } else {
            format!("{}...", &message[..max_length - 3])
        }
    }
    
    fn generate_code_action(&self, error: &CompilationError) -> Option<CodeAction> {
        match error.error_type {
            ErrorType::MissingTraitImplementation => {
                if let Some(struct_name) = &error.struct_name {
                    Some(CodeAction {
                        title: "Add UdonBehaviour implementation".to_string(),
                        kind: CodeActionKind::QuickFix,
                        edit: format!(
                            "impl UdonBehaviour for {} {{\n    fn start(&mut self) {{\n        // TODO: Initialize behavior\n    }}\n}}",
                            struct_name
                        ),
                    })
                } else {
                    None
                }
            }
            ErrorType::DuplicateAttribute => {
                Some(CodeAction {
                    title: "Remove duplicate attribute".to_string(),
                    kind: CodeActionKind::QuickFix,
                    edit: "// Remove the duplicate attribute line".to_string(),
                })
            }
            _ => None,
        }
    }
    
    fn get_error_code(&self, error_type: &ErrorType) -> u16 {
        match error_type {
            ErrorType::MissingTraitImplementation => 1001,
            ErrorType::MissingRequiredMethods => 1002,
            ErrorType::InvalidAttributeUsage => 2001,
            ErrorType::DuplicateAttribute => 2002,
            ErrorType::UnsupportedType => 3001,
            ErrorType::UnsupportedFeature => 3002,
        }
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Message template for error formatting
#[derive(Debug, Clone)]
struct MessageTemplate {
    title: String,
    description: String,
    icon: String,
    color: TerminalColor,
    quick_fix: Option<String>,
    help_url: Option<String>,
}

/// Terminal colors for output formatting
#[derive(Debug, Clone, Copy)]
enum TerminalColor {
    Red,
    Green,
    Yellow,
    Blue,
    Cyan,
    Gray,
}

/// Quick fix suggestion
#[derive(Debug, Clone)]
pub struct QuickFix {
    pub title: String,
    pub description: String,
    pub error_type: ErrorType,
    pub applicable_to: Option<String>,
    pub code_action: Option<CodeAction>,
}

/// Code action for IDE integration
#[derive(Debug, Clone)]
pub struct CodeAction {
    pub title: String,
    pub kind: CodeActionKind,
    pub edit: String,
}

/// Types of code actions
#[derive(Debug, Clone)]
pub enum CodeActionKind {
    QuickFix,
    Refactor,
    Source,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_detection::*;

    fn create_test_error() -> CompilationError {
        CompilationError {
            error_type: ErrorType::MissingTraitImplementation,
            struct_name: Some("TestBehavior".to_string()),
            field_name: None,
            method_name: None,
            message: "Test error message".to_string(),
            suggestion: Some("Test suggestion".to_string()),
            code_example: Some("fn test() {}".to_string()),
            severity: DiagnosticLevel::Error,
            source_location: Some(SourceLocation {
                file: "test.rs".to_string(),
                line: 10,
                column: 5,
            }),
        }
    }

    #[test]
    fn test_reporter_creation() {
        let reporter = ErrorReporter::new();
        assert!(!reporter.message_templates.is_empty());
    }

    #[test]
    fn test_error_formatting() {
        let reporter = ErrorReporter::with_config(false, false); // No colors for testing
        let error = create_test_error();
        
        let formatted = reporter.format_error(&error);
        assert!(formatted.contains("Missing UdonBehaviour Trait Implementation"));
        assert!(formatted.contains("Test error message"));
        assert!(formatted.contains("Test suggestion"));
        assert!(formatted.contains("fn test() {}"));
    }

    #[test]
    fn test_report_generation() {
        let reporter = ErrorReporter::with_config(false, false);
        let error = create_test_error();
        let report = CompilationErrorReport {
            errors: vec![error],
            errors_by_category: HashMap::new(),
            error_count: 1,
            warning_count: 0,
            structs_analyzed: 1,
            has_blocking_errors: true,
        };
        
        let formatted_report = reporter.generate_report(&report);
        assert!(formatted_report.contains("UdonSharp Compilation Analysis"));
        assert!(formatted_report.contains("Compilation blocked by errors"));
    }

    #[test]
    fn test_quick_fix_generation() {
        let reporter = ErrorReporter::new();
        let error = create_test_error();
        
        let fixes = reporter.generate_quick_fixes(&[error]);
        assert!(!fixes.is_empty());
        assert!(fixes[0].title.contains("impl UdonBehaviour"));
    }

    #[test]
    fn test_diagnostic_generation() {
        let reporter = ErrorReporter::new();
        let error = create_test_error();
        
        let diagnostics = reporter.generate_diagnostics(&[error]);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].level, DiagnosticLevel::Error);
    }
}