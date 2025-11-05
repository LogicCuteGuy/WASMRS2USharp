//! UdonSharp attribute system for controlling code generation
//! 
//! This module provides attributes that can be used to control how Rust code
//! is transformed into UdonSharp-compatible C# code.
//! 
//! # Usage
//! 
//! ```rust
//! use udonsharp_core::prelude::*;
//! 
//! #[derive(UdonBehaviour)]
//! #[udon_sync_mode("Manual")]
//! pub struct MyBehaviour {
//!     #[udon_public]
//!     #[udon_header("Player Settings")]
//!     pub player_name: String,
//!     
//!     #[udon_sync("Continuous")]
//!     #[udon_tooltip("Current player score")]
//!     pub score: i32,
//!     
//!     #[udon_range(0.0, 100.0)]
//!     pub health: f32,
//! }
//! 
//! impl UdonBehaviour for MyBehaviour {
//!     fn start(&mut self) {
//!         // Initialization code
//!     }
//!     
//!     #[udon_event]
//!     fn on_player_scored(&mut self) {
//!         self.score += 1;
//!     }
//! }
//! ```

/// Marks a field as public in the generated UdonSharp code
/// This allows the field to be visible in the Unity Inspector
pub struct UdonPublic;

/// Marks a field for network synchronization
/// The field will be automatically synchronized across all clients
pub struct UdonSync {
    /// Synchronization mode (Manual or Continuous)
    pub mode: SyncMode,
}

/// Marks a method as a UdonSharp event handler
/// The method will be callable via SendCustomEvent
pub struct UdonEvent {
    /// Optional custom event name (defaults to method name)
    pub name: Option<String>,
}

/// Marks a method as a network event handler
/// The method will be callable via SendCustomNetworkEvent
pub struct UdonNetworkEvent {
    /// Network event target (All, Others, Owner, etc.)
    pub target: NetworkEventTarget,
    /// Optional custom event name (defaults to method name)
    pub name: Option<String>,
}

/// Marks a field as a UdonSharp header in the Inspector
pub struct UdonHeader {
    /// Header text to display
    pub text: String,
}

/// Marks a field with a tooltip in the Inspector
pub struct UdonTooltip {
    /// Tooltip text to display
    pub text: String,
}

/// Marks a field with a range constraint in the Inspector
pub struct UdonRange {
    /// Minimum value
    pub min: f32,
    /// Maximum value
    pub max: f32,
}

/// Marks a field as a text area in the Inspector
pub struct UdonTextArea {
    /// Minimum number of lines
    pub min_lines: Option<u32>,
    /// Maximum number of lines
    pub max_lines: Option<u32>,
}

/// Marks a field as a space in the Inspector
pub struct UdonSpace {
    /// Amount of space in pixels
    pub pixels: Option<f32>,
}

/// Marks a field with a custom property drawer
pub struct UdonPropertyDrawer {
    /// Name of the property drawer class
    pub drawer_name: String,
}

/// Synchronization modes for UdonSync
#[derive(Debug, Clone, Copy)]
pub enum SyncMode {
    /// Manual synchronization - requires explicit RequestSerialization calls
    Manual,
    /// Continuous synchronization - automatically syncs when values change
    Continuous,
}

/// Network event targets for UdonNetworkEvent
#[derive(Debug, Clone, Copy)]
pub enum NetworkEventTarget {
    /// Send to all clients including sender
    All,
    /// Send to all clients except sender
    Others,
    /// Send to the owner of the object
    Owner,
    /// Send to the master client
    Master,
}

// Placeholder implementations for the attribute structs
// These will be replaced with actual procedural macros later

impl UdonPublic {
    pub fn new() -> Self {
        Self
    }
}

impl UdonSync {
    pub fn new(mode: SyncMode) -> Self {
        Self { mode }
    }
    
    pub fn manual() -> Self {
        Self { mode: SyncMode::Manual }
    }
    
    pub fn continuous() -> Self {
        Self { mode: SyncMode::Continuous }
    }
}

impl UdonEvent {
    pub fn new() -> Self {
        Self { name: None }
    }
    
    pub fn with_name(name: String) -> Self {
        Self { name: Some(name) }
    }
}

impl UdonNetworkEvent {
    pub fn new(target: NetworkEventTarget) -> Self {
        Self { target, name: None }
    }
    
    pub fn with_name(target: NetworkEventTarget, name: String) -> Self {
        Self { target, name: Some(name) }
    }
}

impl UdonHeader {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl UdonTooltip {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl UdonRange {
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }
}

impl UdonTextArea {
    pub fn new() -> Self {
        Self {
            min_lines: None,
            max_lines: None,
        }
    }
    
    pub fn with_lines(min_lines: u32, max_lines: u32) -> Self {
        Self {
            min_lines: Some(min_lines),
            max_lines: Some(max_lines),
        }
    }
}

impl UdonSpace {
    pub fn new() -> Self {
        Self { pixels: None }
    }
    
    pub fn with_pixels(pixels: f32) -> Self {
        Self { pixels: Some(pixels) }
    }
}

impl UdonPropertyDrawer {
    pub fn new(drawer_name: String) -> Self {
        Self { drawer_name }
    }
}

/// Marks a function as a UdonBehaviour entry point
/// This attribute is used to split WASM into multiple UdonBehaviour classes
#[derive(Debug, Clone)]
pub struct UdonBehaviourMarker {
    /// Name of the generated UdonBehaviour class
    pub name: Option<String>,
    /// Unity events this behaviour should handle
    pub events: Vec<String>,
    /// Dependencies on other UdonBehaviour classes
    pub dependencies: Vec<String>,
    /// Whether to automatically sync this behaviour
    pub auto_sync: bool,
}

impl UdonBehaviourMarker {
    pub fn new() -> Self {
        Self {
            name: None,
            events: vec!["Start".to_string()],
            dependencies: Vec::new(),
            auto_sync: false,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_events(mut self, events: Vec<String>) -> Self {
        self.events = events;
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn with_auto_sync(mut self) -> Self {
        self.auto_sync = true;
        self
    }
}

/// Validation functions for UdonBehaviour attributes
impl UdonBehaviourMarker {
    /// Validate the UdonBehaviour configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate name
        if let Some(ref name) = self.name {
            if name.is_empty() {
                return Err("UdonBehaviour name cannot be empty".to_string());
            }
            
            if !name.chars().next().unwrap_or('a').is_ascii_uppercase() {
                return Err("UdonBehaviour name must start with an uppercase letter".to_string());
            }
            
            if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return Err("UdonBehaviour name can only contain alphanumeric characters and underscores".to_string());
            }
        }
        
        // Validate events
        for event in &self.events {
            if !is_valid_unity_event(event) {
                return Err(format!("Invalid Unity event: '{}'. Must be a valid Unity lifecycle method or custom event", event));
            }
        }
        
        // Validate dependencies
        for dep in &self.dependencies {
            if dep.is_empty() {
                return Err("Dependency name cannot be empty".to_string());
            }
            
            if !dep.chars().next().unwrap_or('a').is_ascii_uppercase() {
                return Err(format!("Dependency '{}' must start with an uppercase letter", dep));
            }
        }
        
        // Check for circular dependencies (basic check - full check happens during compilation)
        if self.dependencies.contains(&self.name.clone().unwrap_or_default()) {
            return Err("UdonBehaviour cannot depend on itself".to_string());
        }
        
        Ok(())
    }
}

/// Check if a string represents a valid Unity event
fn is_valid_unity_event(event: &str) -> bool {
    const VALID_UNITY_EVENTS: &[&str] = &[
        // Unity lifecycle events
        "Awake", "Start", "Update", "LateUpdate", "FixedUpdate",
        "OnEnable", "OnDisable", "OnDestroy",
        
        // Unity collision events
        "OnTriggerEnter", "OnTriggerExit", "OnTriggerStay",
        "OnCollisionEnter", "OnCollisionExit", "OnCollisionStay",
        
        // Unity UI events
        "OnPointerClick", "OnPointerDown", "OnPointerUp", "OnPointerEnter", "OnPointerExit",
        "OnDrag", "OnBeginDrag", "OnEndDrag", "OnDrop",
        
        // VRChat specific events
        "OnPlayerJoined", "OnPlayerLeft", "OnPlayerRespawn",
        "OnStationEntered", "OnStationExited",
        "OnOwnershipTransferred", "OnDeserialization",
        "OnPreSerialization", "OnPostSerialization",
        
        // UdonSharp events
        "OnPickup", "OnDrop", "OnPickupUseDown", "OnPickupUseUp",
        "OnVideoStart", "OnVideoEnd", "OnVideoError", "OnVideoReady",
        "OnVideoPlay", "OnVideoPause",
    ];
    
    // Check if it's a known Unity/VRChat event
    if VALID_UNITY_EVENTS.contains(&event) {
        return true;
    }
    
    // Allow custom events (must be valid C# method names)
    if event.is_empty() {
        return false;
    }
    
    // Must start with letter or underscore
    let first_char = event.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }
    
    // Rest must be alphanumeric or underscore
    event.chars().skip(1).all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Attribute configuration parser and validator
pub struct AttributeParser;

impl AttributeParser {
    /// Parse UdonBehaviour attribute from string representation
    pub fn parse_udon_behaviour_from_metadata(metadata: &str) -> Result<UdonBehaviourMarker, String> {
        let mut marker = UdonBehaviourMarker::new();
        
        for pair in metadata.split(';') {
            if pair.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = pair.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }
            
            match parts[0] {
                "name" => {
                    if !parts[1].is_empty() {
                        marker.name = Some(parts[1].to_string());
                    }
                }
                "events" => {
                    if !parts[1].is_empty() {
                        marker.events = parts[1]
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
                "dependencies" => {
                    if !parts[1].is_empty() {
                        marker.dependencies = parts[1]
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
                "auto_sync" => {
                    marker.auto_sync = parts[1] == "true";
                }
                _ => {} // Ignore unknown fields
            }
        }
        
        // Validate the parsed configuration
        marker.validate()?;
        
        Ok(marker)
    }
    
    /// Validate a UdonBehaviour attribute configuration
    pub fn validate_udon_behaviour(marker: &UdonBehaviourMarker) -> Result<(), String> {
        marker.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_unity_events() {
        assert!(is_valid_unity_event("Start"));
        assert!(is_valid_unity_event("Update"));
        assert!(is_valid_unity_event("OnTriggerEnter"));
        assert!(is_valid_unity_event("OnPlayerJoined"));
        assert!(is_valid_unity_event("CustomEvent"));
        assert!(is_valid_unity_event("_PrivateEvent"));
    }

    #[test]
    fn test_invalid_unity_events() {
        assert!(!is_valid_unity_event(""));
        assert!(!is_valid_unity_event("123Invalid"));
        assert!(!is_valid_unity_event("Invalid-Event"));
        assert!(!is_valid_unity_event("Invalid Event"));
    }

    #[test]
    fn test_udon_behaviour_validation() {
        let mut marker = UdonBehaviourMarker::new();
        marker.name = Some("ValidName".to_string());
        assert!(marker.validate().is_ok());

        marker.name = Some("invalidName".to_string());
        assert!(marker.validate().is_err());

        marker.name = Some("Invalid-Name".to_string());
        assert!(marker.validate().is_err());

        marker.name = Some("".to_string());
        assert!(marker.validate().is_err());
    }

    #[test]
    fn test_attribute_parser() {
        let metadata = "name:TestBehaviour;events:Start,Update;dependencies:PlayerManager;auto_sync:true;";
        let result = AttributeParser::parse_udon_behaviour_from_metadata(metadata);
        
        assert!(result.is_ok());
        let marker = result.unwrap();
        assert_eq!(marker.name, Some("TestBehaviour".to_string()));
        assert_eq!(marker.events, vec!["Start".to_string(), "Update".to_string()]);
        assert_eq!(marker.dependencies, vec!["PlayerManager".to_string()]);
        assert!(marker.auto_sync);
    }
}