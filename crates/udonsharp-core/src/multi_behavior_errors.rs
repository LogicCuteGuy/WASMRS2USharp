//! Specialized error handling for multi-behavior compilation
//! 
//! This module provides detailed error reporting and user-friendly suggestions
//! for common issues encountered during multi-behavior compilation.

use crate::error::{UdonSharpError, Diagnostic, DiagnosticLevel};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Multi-behavior error handler with contextual suggestions
pub struct MultiBehaviorErrorHandler {
    error_patterns: HashMap<String, ErrorPattern>,
    suggestion_database: SuggestionDatabase,
}

impl MultiBehaviorErrorHandler {
    /// Create a new multi-behavior error handler
    pub fn new() -> Self {
        let mut handler = Self {
            error_patterns: HashMap::new(),
            suggestion_database: SuggestionDatabase::new(),
        };
        
        handler.initialize_error_patterns();
        handler
    }
    
    /// Initialize common error patterns and their solutions
    fn initialize_error_patterns(&mut self) {
        // Circular dependency patterns
        self.error_patterns.insert(
            "circular_dependency".to_string(),
            ErrorPattern {
                pattern: "circular dependency".to_string(),
                category: ErrorCategory::CircularDependency,
                severity: DiagnosticLevel::Error,
                suggestions: vec![
                    "Move shared functionality to a common utility class".to_string(),
                    "Use events or interfaces to decouple behaviors".to_string(),
                    "Consider restructuring the dependency hierarchy".to_string(),
                ],
            }
        );
        
        // Invalid attribute patterns
        self.error_patterns.insert(
            "invalid_udon_behaviour_attribute".to_string(),
            ErrorPattern {
                pattern: "invalid.*udon_behaviour".to_string(),
                category: ErrorCategory::InvalidAttribute,
                severity: DiagnosticLevel::Error,
                suggestions: vec![
                    "Ensure the function signature is compatible with UdonSharp".to_string(),
                    "Check that the attribute parameters are valid".to_string(),
                    "Verify the function is public and has the correct return type".to_string(),
                ],
            }
        );
        
        // Missing behavior reference patterns
        self.error_patterns.insert(
            "missing_behavior_reference".to_string(),
            ErrorPattern {
                pattern: "behavior.*not found".to_string(),
                category: ErrorCategory::MissingReference,
                severity: DiagnosticLevel::Error,
                suggestions: vec![
                    "Ensure all referenced behaviors are properly defined".to_string(),
                    "Check that GameObject names match the behavior names".to_string(),
                    "Verify the behavior is included in the compilation".to_string(),
                ],
            }
        );
        
        // Inter-behavior communication patterns
        self.error_patterns.insert(
            "communication_failure".to_string(),
            ErrorPattern {
                pattern: "communication.*failed".to_string(),
                category: ErrorCategory::CommunicationError,
                severity: DiagnosticLevel::Error,
                suggestions: vec![
                    "Ensure target behavior GameObject exists in the scene".to_string(),
                    "Check that the target method is public and properly exposed".to_string(),
                    "Verify network synchronization settings if using networked calls".to_string(),
                ],
            }
        );
        
        // Behavior splitting warnings
        self.error_patterns.insert(
            "splitting_warning".to_string(),
            ErrorPattern {
                pattern: "splitting.*performance".to_string(),
                category: ErrorCategory::PerformanceWarning,
                severity: DiagnosticLevel::Warning,
                suggestions: vec![
                    "Consider consolidating small behaviors to reduce overhead".to_string(),
                    "Review the splitting strategy for optimal performance".to_string(),
                    "Use shared runtime for common functionality".to_string(),
                ],
            }
        );
    }
    
    /// Analyze an error and provide contextual suggestions
    pub fn analyze_error(&self, error: &UdonSharpError) -> EnhancedErrorReport {
        match error {
            UdonSharpError::CircularDependency { cycle, behaviors, .. } => {
                self.handle_circular_dependency_error(cycle, behaviors)
            }
            UdonSharpError::InvalidAttribute { message, attribute_name, function_name, .. } => {
                self.handle_invalid_attribute_error(message, attribute_name, function_name)
            }
            UdonSharpError::BehaviorSplitting { message, source_function, target_behavior, .. } => {
                self.handle_behavior_splitting_error(message, source_function, target_behavior)
            }
            UdonSharpError::InterBehaviorCommunication { message, source_behavior, target_behavior, .. } => {
                self.handle_communication_error(message, source_behavior, target_behavior)
            }
            UdonSharpError::MultiBehavior { message, behavior_name, .. } => {
                self.handle_general_multi_behavior_error(message, behavior_name)
            }
            _ => {
                // Handle other error types with generic suggestions
                EnhancedErrorReport {
                    original_error: error.to_string(),
                    category: ErrorCategory::General,
                    severity: DiagnosticLevel::Error,
                    suggestions: vec!["Check the compilation logs for more details".to_string()],
                    code_examples: Vec::new(),
                    related_documentation: Vec::new(),
                }
            }
        }
    }
    
    /// Handle circular dependency errors with detailed analysis
    fn handle_circular_dependency_error(&self, cycle: &str, behaviors: &[String]) -> EnhancedErrorReport {
        let mut suggestions = vec![
            format!("Circular dependency detected in behaviors: {}", behaviors.join(" -> ")),
            "To resolve this issue, consider one of the following approaches:".to_string(),
        ];
        
        // Provide specific suggestions based on the number of behaviors involved
        if behaviors.len() == 2 {
            suggestions.push("1. Extract shared functionality into a common utility class".to_string());
            suggestions.push("2. Use events to communicate between behaviors instead of direct references".to_string());
        } else {
            suggestions.push("1. Identify the core functionality and create a central coordinator behavior".to_string());
            suggestions.push("2. Use a hierarchical structure with clear parent-child relationships".to_string());
        }
        
        let code_examples = vec![
            CodeExample {
                title: "Using Events for Communication".to_string(),
                code: r#"
// Instead of direct references:
// behavior_a.CallMethodOnB(); // Creates circular dependency

// Use events:
[udon_behaviour(name = "BehaviorA")]
pub fn behavior_a_start() {
    // Send event to BehaviorB
    send_custom_event("BehaviorB", "HandleEventFromA");
}

[udon_behaviour(name = "BehaviorB")]  
pub fn behavior_b_start() {
    // Handle event from BehaviorA
}

pub fn handle_event_from_a() {
    // Process the event without direct dependency
}
"#.to_string(),
            },
        ];
        
        EnhancedErrorReport {
            original_error: cycle.to_string(),
            category: ErrorCategory::CircularDependency,
            severity: DiagnosticLevel::Error,
            suggestions,
            code_examples,
            related_documentation: vec![
                "docs/best-practices.md#avoiding-circular-dependencies".to_string(),
                "docs/api-reference.md#inter-behavior-communication".to_string(),
            ],
        }
    }
    
    /// Handle invalid attribute errors
    fn handle_invalid_attribute_error(&self, message: &str, attribute_name: &str, function_name: &Option<String>) -> EnhancedErrorReport {
        let mut suggestions = vec![
            format!("Invalid attribute '{}' detected", attribute_name),
        ];
        
        if let Some(func_name) = function_name {
            suggestions.push(format!("Function '{}' has an invalid attribute configuration", func_name));
        }
        
        suggestions.extend(vec![
            "Common issues with #[udon_behaviour] attributes:".to_string(),
            "1. Function must be public and have a compatible signature".to_string(),
            "2. Attribute parameters must be valid (name, events, dependencies)".to_string(),
            "3. Function should not have complex parameter types".to_string(),
        ]);
        
        let code_examples = vec![
            CodeExample {
                title: "Correct UdonBehaviour Attribute Usage".to_string(),
                code: r#"
// Correct usage:
#[udon_behaviour(name = "PlayerManager")]
pub fn player_manager_start() {
    // Entry point for PlayerManager UdonBehaviour
}

// With events:
#[udon_behaviour(name = "UIController", events = ["Update", "OnTriggerEnter"])]
pub fn ui_controller_start() {
    // Entry point with specific Unity events
}

// Incorrect usage (will cause errors):
#[udon_behaviour(name = "Invalid")]
fn private_function() { } // Must be public

#[udon_behaviour(name = "")]  // Empty name not allowed
pub fn invalid_name() { }
"#.to_string(),
            },
        ];
        
        EnhancedErrorReport {
            original_error: message.to_string(),
            category: ErrorCategory::InvalidAttribute,
            severity: DiagnosticLevel::Error,
            suggestions,
            code_examples,
            related_documentation: vec![
                "docs/api-reference.md#udon-behaviour-attributes".to_string(),
                "docs/best-practices.md#attribute-usage".to_string(),
            ],
        }
    }
    
    /// Handle behavior splitting errors
    fn handle_behavior_splitting_error(&self, message: &str, source_function: &Option<String>, target_behavior: &Option<String>) -> EnhancedErrorReport {
        let mut suggestions = vec![
            "Behavior splitting encountered an issue:".to_string(),
        ];
        
        if let (Some(func), Some(behavior)) = (source_function, target_behavior) {
            suggestions.push(format!("Function '{}' could not be assigned to behavior '{}'", func, behavior));
        }
        
        suggestions.extend(vec![
            "Common causes of splitting errors:".to_string(),
            "1. Function dependencies span multiple behaviors incorrectly".to_string(),
            "2. Shared functions are not properly identified".to_string(),
            "3. Behavior boundaries are not clearly defined".to_string(),
        ]);
        
        let code_examples = vec![
            CodeExample {
                title: "Proper Behavior Organization".to_string(),
                code: r#"
// Clear behavior separation:
#[udon_behaviour(name = "PlayerTracker")]
pub fn player_tracker_start() {
    // Only player-related functionality
}

#[udon_behaviour(name = "UIManager")]
pub fn ui_manager_start() {
    // Only UI-related functionality
}

// Shared utilities (no attribute - goes to SharedRuntime):
pub fn calculate_distance(a: Vector3, b: Vector3) -> f32 {
    // Available to all behaviors
}
"#.to_string(),
            },
        ];
        
        EnhancedErrorReport {
            original_error: message.to_string(),
            category: ErrorCategory::BehaviorSplitting,
            severity: DiagnosticLevel::Error,
            suggestions,
            code_examples,
            related_documentation: vec![
                "docs/compilation-pipeline.md#behavior-splitting".to_string(),
                "docs/best-practices.md#organizing-behaviors".to_string(),
            ],
        }
    }
    
    /// Handle inter-behavior communication errors
    fn handle_communication_error(&self, message: &str, source_behavior: &Option<String>, target_behavior: &Option<String>) -> EnhancedErrorReport {
        let mut suggestions = vec![
            "Inter-behavior communication failed:".to_string(),
        ];
        
        if let (Some(source), Some(target)) = (source_behavior, target_behavior) {
            suggestions.push(format!("Communication from '{}' to '{}' failed", source, target));
        }
        
        suggestions.extend(vec![
            "Common communication issues:".to_string(),
            "1. Target behavior GameObject not found in scene".to_string(),
            "2. Target method is not public or properly exposed".to_string(),
            "3. Network synchronization issues in multiplayer scenarios".to_string(),
            "4. Timing issues - target behavior not initialized yet".to_string(),
        ]);
        
        let code_examples = vec![
            CodeExample {
                title: "Safe Inter-Behavior Communication".to_string(),
                code: r#"
// Safe communication pattern:
impl PlayerTracker {
    fn notify_ui_update(&self) {
        // Check if UI behavior exists before calling
        if let Some(ui_obj) = unity::GameObject::find("UIController") {
            ui_obj.send_custom_event("UpdatePlayerDisplay");
        } else {
            log::warn!("UIController not found - skipping UI update");
        }
    }
}

// In UIController:
#[udon_event("UpdatePlayerDisplay")]
pub fn update_player_display(&mut self) {
    // Handle the update request
}
"#.to_string(),
            },
        ];
        
        EnhancedErrorReport {
            original_error: message.to_string(),
            category: ErrorCategory::CommunicationError,
            severity: DiagnosticLevel::Error,
            suggestions,
            code_examples,
            related_documentation: vec![
                "docs/api-reference.md#inter-behavior-communication".to_string(),
                "docs/best-practices.md#communication-patterns".to_string(),
            ],
        }
    }
    
    /// Handle general multi-behavior errors
    fn handle_general_multi_behavior_error(&self, message: &str, behavior_name: &Option<String>) -> EnhancedErrorReport {
        let mut suggestions = vec![
            "Multi-behavior compilation issue:".to_string(),
        ];
        
        if let Some(name) = behavior_name {
            suggestions.push(format!("Issue with behavior: {}", name));
        }
        
        suggestions.extend(vec![
            "General troubleshooting steps:".to_string(),
            "1. Check that all #[udon_behaviour] attributes are correctly formatted".to_string(),
            "2. Ensure behavior names are unique and valid".to_string(),
            "3. Verify that dependencies between behaviors are properly defined".to_string(),
            "4. Check for any circular dependencies in the behavior graph".to_string(),
        ]);
        
        EnhancedErrorReport {
            original_error: message.to_string(),
            category: ErrorCategory::General,
            severity: DiagnosticLevel::Error,
            suggestions,
            code_examples: Vec::new(),
            related_documentation: vec![
                "docs/getting-started.md#multi-behavior-basics".to_string(),
                "docs/best-practices.md".to_string(),
            ],
        }
    }
    
    /// Generate a user-friendly error report
    pub fn generate_user_report(&self, error: &UdonSharpError) -> String {
        let report = self.analyze_error(error);
        let mut output = String::new();
        
        // Error header
        output.push_str(&format!("🚨 {} Error\n", report.category));
        output.push_str(&format!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"));
        output.push_str(&format!("Error: {}\n\n", report.original_error));
        
        // Suggestions
        if !report.suggestions.is_empty() {
            output.push_str("💡 Suggestions:\n");
            for (i, suggestion) in report.suggestions.iter().enumerate() {
                output.push_str(&format!("   {}. {}\n", i + 1, suggestion));
            }
            output.push_str("\n");
        }
        
        // Code examples
        if !report.code_examples.is_empty() {
            output.push_str("📝 Code Examples:\n");
            for example in &report.code_examples {
                output.push_str(&format!("   {}\n", example.title));
                output.push_str(&format!("   ```rust{}\n   ```\n\n", example.code));
            }
        }
        
        // Documentation links
        if !report.related_documentation.is_empty() {
            output.push_str("📚 Related Documentation:\n");
            for doc in &report.related_documentation {
                output.push_str(&format!("   • {}\n", doc));
            }
            output.push_str("\n");
        }
        
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        output
    }
}

impl Default for MultiBehaviorErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Error pattern for matching and categorizing errors
#[derive(Debug, Clone)]
struct ErrorPattern {
    pattern: String,
    category: ErrorCategory,
    severity: DiagnosticLevel,
    suggestions: Vec<String>,
}

/// Enhanced error report with contextual information
#[derive(Debug, Clone)]
pub struct EnhancedErrorReport {
    pub original_error: String,
    pub category: ErrorCategory,
    pub severity: DiagnosticLevel,
    pub suggestions: Vec<String>,
    pub code_examples: Vec<CodeExample>,
    pub related_documentation: Vec<String>,
}

/// Code example for error resolution
#[derive(Debug, Clone)]
pub struct CodeExample {
    pub title: String,
    pub code: String,
}

/// Error categories for multi-behavior compilation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    CircularDependency,
    InvalidAttribute,
    MissingReference,
    CommunicationError,
    BehaviorSplitting,
    PerformanceWarning,
    General,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::CircularDependency => write!(f, "Circular Dependency"),
            ErrorCategory::InvalidAttribute => write!(f, "Invalid Attribute"),
            ErrorCategory::MissingReference => write!(f, "Missing Reference"),
            ErrorCategory::CommunicationError => write!(f, "Communication"),
            ErrorCategory::BehaviorSplitting => write!(f, "Behavior Splitting"),
            ErrorCategory::PerformanceWarning => write!(f, "Performance"),
            ErrorCategory::General => write!(f, "General"),
        }
    }
}

/// Database of suggestions for common issues
struct SuggestionDatabase {
    suggestions: HashMap<String, Vec<String>>,
}

impl SuggestionDatabase {
    fn new() -> Self {
        let mut db = Self {
            suggestions: HashMap::new(),
        };
        
        db.initialize_suggestions();
        db
    }
    
    fn initialize_suggestions(&mut self) {
        // Add common suggestions for various error types
        self.suggestions.insert(
            "circular_dependency".to_string(),
            vec![
                "Extract shared functionality into a utility class".to_string(),
                "Use events for loose coupling between behaviors".to_string(),
                "Implement a mediator pattern for complex interactions".to_string(),
            ]
        );
        
        self.suggestions.insert(
            "invalid_attribute".to_string(),
            vec![
                "Check attribute syntax and parameters".to_string(),
                "Ensure function signature is UdonSharp compatible".to_string(),
                "Verify function visibility (must be public)".to_string(),
            ]
        );
    }
    
    fn get_suggestions(&self, category: &str) -> Vec<String> {
        self.suggestions.get(category).cloned().unwrap_or_default()
    }
}