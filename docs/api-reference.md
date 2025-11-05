# API Reference

Complete API documentation for the Rust UdonSharp framework.

## Core Traits and Types

### UdonBehaviour Trait

The main trait for creating UdonSharp behaviors.

```rust
pub trait UdonBehaviour {
    /// Called when the behavior starts (equivalent to UdonSharp Start())
    fn start(&mut self) {}
    
    /// Called every frame (equivalent to UdonSharp Update())
    fn update(&mut self) {}
    
    /// Called when a player joins the world
    fn on_player_joined(&mut self, player: VRCPlayerApi) {}
    
    /// Called when a player leaves the world
    fn on_player_left(&mut self, player: VRCPlayerApi) {}
    
    /// Called when ownership of this object is transferred
    fn on_ownership_transferred(&mut self, player: VRCPlayerApi) {}
    
    /// Called when serialization is requested (networking)
    fn on_pre_serialization(&mut self) {}
    
    /// Called after deserialization (networking)
    fn on_post_deserialization(&mut self) {}
}
```

### UdonBehaviour Derive Macro

```rust
#[derive(UdonBehaviour)]
#[udon_sync_mode(SyncMode)]  // Optional: None, Manual, Continuous
pub struct MyBehaviour {
    #[udon_public]           // Visible in Unity Inspector
    pub public_field: String,
    
    #[udon_sync]            // Synchronized across network
    pub synced_field: i32,
    
    private_field: bool,    // Private, not synchronized
}
```

**Sync Modes:**
- `None`: No networking synchronization
- `Manual`: Manual synchronization control (recommended)
- `Continuous`: Automatic continuous synchronization

### Core Types

```rust
// Unity math types
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

// VRChat types
pub struct VRCPlayerApi {
    // Opaque handle to VRChat player
}
```

## VRChat API Bindings

### Networking

```rust
pub mod vrchat {
    pub struct Networking;
    
    impl Networking {
        /// Get the local player
        pub fn local_player() -> VRCPlayerApi;
        
        /// Get all players in the world
        pub fn get_players() -> Vec<VRCPlayerApi>;
        
        /// Check if a player owns an object
        pub fn is_owner(player: &VRCPlayerApi) -> bool;
        
        /// Set ownership of an object to a player
        pub fn set_owner(player: &VRCPlayerApi, target: &unity::GameObject);
        
        /// Get the master player
        pub fn get_master() -> VRCPlayerApi;
        
        /// Check if the local player is master
        pub fn is_master() -> bool;
    }
}
```

### VRCPlayerApi

```rust
impl VRCPlayerApi {
    /// Get the player's display name
    pub fn get_display_name(&self) -> String;
    
    /// Get the player's unique ID
    pub fn player_id(&self) -> i32;
    
    /// Check if this is the local player
    pub fn is_local(&self) -> bool;
    
    /// Check if the player is valid
    pub fn is_valid(&self) -> bool;
    
    /// Get the player's position
    pub fn get_position(&self) -> Vector3;
    
    /// Get the player's rotation
    pub fn get_rotation(&self) -> Quaternion;
    
    /// Teleport the player to a position
    pub fn teleport_to(&self, position: Vector3, rotation: Quaternion);
    
    /// Respawn the player
    pub fn respawn(&self);
    
    /// Check if the player is in VR
    pub fn is_user_in_vr(&self) -> bool;
    
    /// Get the player's avatar height
    pub fn get_avatar_eye_height(&self) -> f32;
    
    /// Get the player's bone position
    pub fn get_bone_position(&self, bone: HumanBodyBones) -> Vector3;
    
    /// Get the player's bone rotation
    pub fn get_bone_rotation(&self, bone: HumanBodyBones) -> Quaternion;
}
```

### UdonBehaviour Extensions

```rust
pub trait UdonBehaviourExt {
    /// Send a custom event to this behavior
    fn send_custom_event(&self, event_name: &str);
    
    /// Send a custom network event
    fn send_custom_network_event(&self, target: NetworkEventTarget, event_name: &str);
    
    /// Request serialization for networking
    fn request_serialization(&self);
    
    /// Get the GameObject this behavior is attached to
    fn game_object(&self) -> unity::GameObject;
    
    /// Get the Transform component
    fn transform(&self) -> unity::Transform;
}

pub enum NetworkEventTarget {
    All,
    Others,
    Owner,
}
```

## Unity API Bindings

### GameObject

```rust
pub mod unity {
    pub struct GameObject {
        // Opaque handle
    }
    
    impl GameObject {
        /// Find a GameObject by name
        pub fn find(name: &str) -> Option<GameObject>;
        
        /// Find GameObjects by tag
        pub fn find_with_tag(tag: &str) -> Option<GameObject>;
        
        /// Find all GameObjects by tag
        pub fn find_objects_with_tag(tag: &str) -> Vec<GameObject>;
        
        /// Create a new GameObject
        pub fn new(name: &str) -> GameObject;
        
        /// Get a component of type T
        pub fn get_component<T>(&self) -> Option<T>;
        
        /// Get all components of type T
        pub fn get_components<T>(&self) -> Vec<T>;
        
        /// Add a component of type T
        pub fn add_component<T>(&mut self) -> T;
        
        /// Get the name of the GameObject
        pub fn name(&self) -> String;
        
        /// Set the name of the GameObject
        pub fn set_name(&mut self, name: &str);
        
        /// Get the tag of the GameObject
        pub fn tag(&self) -> String;
        
        /// Set the tag of the GameObject
        pub fn set_tag(&mut self, tag: &str);
        
        /// Check if the GameObject is active
        pub fn active_self(&self) -> bool;
        
        /// Check if the GameObject is active in hierarchy
        pub fn active_in_hierarchy(&self) -> bool;
        
        /// Set the GameObject active/inactive
        pub fn set_active(&mut self, active: bool);
        
        /// Get the Transform component
        pub fn transform(&self) -> Transform;
        
        /// Destroy the GameObject
        pub fn destroy(&self);
        
        /// Destroy the GameObject after delay
        pub fn destroy_delayed(&self, delay: f32);
    }
}
```

### Transform

```rust
impl Transform {
    /// Get the position
    pub fn position(&self) -> Vector3;
    
    /// Set the position
    pub fn set_position(&mut self, position: Vector3);
    
    /// Get the local position
    pub fn local_position(&self) -> Vector3;
    
    /// Set the local position
    pub fn set_local_position(&mut self, position: Vector3);
    
    /// Get the rotation
    pub fn rotation(&self) -> Quaternion;
    
    /// Set the rotation
    pub fn set_rotation(&mut self, rotation: Quaternion);
    
    /// Get the local rotation
    pub fn local_rotation(&self) -> Quaternion;
    
    /// Set the local rotation
    pub fn set_local_rotation(&mut self, rotation: Quaternion);
    
    /// Get the scale
    pub fn local_scale(&self) -> Vector3;
    
    /// Set the scale
    pub fn set_local_scale(&mut self, scale: Vector3);
    
    /// Translate by a vector
    pub fn translate(&mut self, translation: Vector3);
    
    /// Rotate by angles
    pub fn rotate(&mut self, angles: Vector3);
    
    /// Look at a target
    pub fn look_at(&mut self, target: Vector3);
    
    /// Get the parent transform
    pub fn parent(&self) -> Option<Transform>;
    
    /// Set the parent transform
    pub fn set_parent(&mut self, parent: Option<&Transform>);
    
    /// Get child count
    pub fn child_count(&self) -> i32;
    
    /// Get child by index
    pub fn get_child(&self, index: i32) -> Option<Transform>;
    
    /// Find child by name
    pub fn find(&self, name: &str) -> Option<Transform>;
}
```

### Physics

```rust
pub struct Rigidbody;

impl Rigidbody {
    /// Get the velocity
    pub fn velocity(&self) -> Vector3;
    
    /// Set the velocity
    pub fn set_velocity(&mut self, velocity: Vector3);
    
    /// Get the angular velocity
    pub fn angular_velocity(&self) -> Vector3;
    
    /// Set the angular velocity
    pub fn set_angular_velocity(&mut self, velocity: Vector3);
    
    /// Add force
    pub fn add_force(&mut self, force: Vector3);
    
    /// Add force at position
    pub fn add_force_at_position(&mut self, force: Vector3, position: Vector3);
    
    /// Add torque
    pub fn add_torque(&mut self, torque: Vector3);
    
    /// Get/set mass
    pub fn mass(&self) -> f32;
    pub fn set_mass(&mut self, mass: f32);
    
    /// Get/set drag
    pub fn drag(&self) -> f32;
    pub fn set_drag(&mut self, drag: f32);
    
    /// Get/set use gravity
    pub fn use_gravity(&self) -> bool;
    pub fn set_use_gravity(&mut self, use_gravity: bool);
    
    /// Get/set kinematic
    pub fn is_kinematic(&self) -> bool;
    pub fn set_kinematic(&mut self, kinematic: bool);
}

pub struct Collider;

impl Collider {
    /// Check if collider is trigger
    pub fn is_trigger(&self) -> bool;
    
    /// Set trigger mode
    pub fn set_trigger(&mut self, is_trigger: bool);
    
    /// Get bounds
    pub fn bounds(&self) -> Bounds;
    
    /// Get closest point on collider
    pub fn closest_point(&self, position: Vector3) -> Vector3;
}

pub struct Bounds {
    pub center: Vector3,
    pub size: Vector3,
    pub min: Vector3,
    pub max: Vector3,
}
```

### Audio

```rust
pub struct AudioSource;

impl AudioSource {
    /// Play the audio clip
    pub fn play(&mut self);
    
    /// Stop the audio
    pub fn stop(&mut self);
    
    /// Pause the audio
    pub fn pause(&mut self);
    
    /// Check if playing
    pub fn is_playing(&self) -> bool;
    
    /// Get/set the audio clip
    pub fn clip(&self) -> Option<AudioClip>;
    pub fn set_clip(&mut self, clip: Option<AudioClip>);
    
    /// Get/set volume
    pub fn volume(&self) -> f32;
    pub fn set_volume(&mut self, volume: f32);
    
    /// Get/set pitch
    pub fn pitch(&self) -> f32;
    pub fn set_pitch(&mut self, pitch: f32);
    
    /// Get/set loop
    pub fn is_loop(&self) -> bool;
    pub fn set_loop(&mut self, loop_audio: bool);
    
    /// Play one shot
    pub fn play_one_shot(&mut self, clip: &AudioClip);
}

pub struct AudioClip;

impl AudioClip {
    /// Get the length in seconds
    pub fn length(&self) -> f32;
    
    /// Get the name
    pub fn name(&self) -> String;
}
```

## C# System Bindings

### Collections

```rust
pub mod cs_sys {
    pub mod collections {
        pub struct List<T> {
            // Internal implementation
        }
        
        impl<T> List<T> {
            /// Create a new list
            pub fn new() -> Self;
            
            /// Add an item
            pub fn add(&mut self, item: T);
            
            /// Insert at index
            pub fn insert(&mut self, index: usize, item: T);
            
            /// Remove at index
            pub fn remove_at(&mut self, index: usize);
            
            /// Remove item
            pub fn remove(&mut self, item: &T) -> bool where T: PartialEq;
            
            /// Get item at index
            pub fn get(&self, index: usize) -> Option<&T>;
            
            /// Set item at index
            pub fn set(&mut self, index: usize, item: T);
            
            /// Get count
            pub fn count(&self) -> usize;
            
            /// Check if empty
            pub fn is_empty(&self) -> bool;
            
            /// Clear all items
            pub fn clear(&mut self);
            
            /// Check if contains item
            pub fn contains(&self, item: &T) -> bool where T: PartialEq;
            
            /// Find index of item
            pub fn index_of(&self, item: &T) -> Option<usize> where T: PartialEq;
        }
        
        pub struct Dictionary<K, V> {
            // Internal implementation
        }
        
        impl<K, V> Dictionary<K, V> 
        where 
            K: Eq + std::hash::Hash 
        {
            /// Create a new dictionary
            pub fn new() -> Self;
            
            /// Add key-value pair
            pub fn add(&mut self, key: K, value: V);
            
            /// Remove by key
            pub fn remove(&mut self, key: &K) -> bool;
            
            /// Get value by key
            pub fn get(&self, key: &K) -> Option<&V>;
            
            /// Set value by key
            pub fn set(&mut self, key: K, value: V);
            
            /// Check if key exists
            pub fn contains_key(&self, key: &K) -> bool;
            
            /// Get count
            pub fn count(&self) -> usize;
            
            /// Clear all items
            pub fn clear(&mut self);
            
            /// Get all keys
            pub fn keys(&self) -> Vec<&K>;
            
            /// Get all values
            pub fn values(&self) -> Vec<&V>;
        }
    }
}
```

### Math

```rust
pub mod math {
    pub struct Mathf;
    
    impl Mathf {
        /// Mathematical constants
        pub const PI: f32 = std::f32::consts::PI;
        pub const E: f32 = std::f32::consts::E;
        
        /// Absolute value
        pub fn abs(value: f32) -> f32;
        
        /// Sine function
        pub fn sin(angle: f32) -> f32;
        
        /// Cosine function
        pub fn cos(angle: f32) -> f32;
        
        /// Tangent function
        pub fn tan(angle: f32) -> f32;
        
        /// Arc sine
        pub fn asin(value: f32) -> f32;
        
        /// Arc cosine
        pub fn acos(value: f32) -> f32;
        
        /// Arc tangent
        pub fn atan(value: f32) -> f32;
        
        /// Arc tangent of y/x
        pub fn atan2(y: f32, x: f32) -> f32;
        
        /// Square root
        pub fn sqrt(value: f32) -> f32;
        
        /// Power function
        pub fn pow(base: f32, exponent: f32) -> f32;
        
        /// Natural logarithm
        pub fn log(value: f32) -> f32;
        
        /// Base 10 logarithm
        pub fn log10(value: f32) -> f32;
        
        /// Ceiling function
        pub fn ceil(value: f32) -> f32;
        
        /// Floor function
        pub fn floor(value: f32) -> f32;
        
        /// Round to nearest integer
        pub fn round(value: f32) -> f32;
        
        /// Minimum of two values
        pub fn min(a: f32, b: f32) -> f32;
        
        /// Maximum of two values
        pub fn max(a: f32, b: f32) -> f32;
        
        /// Clamp value between min and max
        pub fn clamp(value: f32, min: f32, max: f32) -> f32;
        
        /// Linear interpolation
        pub fn lerp(a: f32, b: f32, t: f32) -> f32;
        
        /// Inverse linear interpolation
        pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32;
        
        /// Smooth step interpolation
        pub fn smooth_step(from: f32, to: f32, t: f32) -> f32;
        
        /// Convert degrees to radians
        pub fn deg_to_rad(degrees: f32) -> f32;
        
        /// Convert radians to degrees
        pub fn rad_to_deg(radians: f32) -> f32;
        
        /// Sign of a number (-1, 0, or 1)
        pub fn sign(value: f32) -> f32;
    }
    
    pub struct Random;
    
    impl Random {
        /// Random value between 0.0 and 1.0
        pub fn value() -> f32;
        
        /// Random integer between min (inclusive) and max (exclusive)
        pub fn range_int(min: i32, max: i32) -> i32;
        
        /// Random float between min and max
        pub fn range_float(min: f32, max: f32) -> f32;
        
        /// Random point inside unit circle
        pub fn inside_unit_circle() -> Vector2;
        
        /// Random point inside unit sphere
        pub fn inside_unit_sphere() -> Vector3;
        
        /// Random point on unit sphere
        pub fn on_unit_sphere() -> Vector3;
        
        /// Random rotation
        pub fn rotation() -> Quaternion;
    }
}
```

### Time

```rust
pub mod time {
    pub struct Time;
    
    impl Time {
        /// Time since the start of the game
        pub fn time() -> f32;
        
        /// Time since the last frame
        pub fn delta_time() -> f32;
        
        /// Fixed timestep interval
        pub fn fixed_delta_time() -> f32;
        
        /// Time since the start of the game (unscaled)
        pub fn unscaled_time() -> f32;
        
        /// Unscaled delta time
        pub fn unscaled_delta_time() -> f32;
        
        /// Time scale factor
        pub fn time_scale() -> f32;
        
        /// Set time scale
        pub fn set_time_scale(scale: f32);
        
        /// Frame count since start
        pub fn frame_count() -> i32;
        
        /// Fixed update frame count
        pub fn fixed_frame_count() -> i32;
    }
}
```

## Attributes and Macros

### UdonSharp Attributes

```rust
/// Mark a struct as a UdonSharp behavior
#[derive(UdonBehaviour)]
#[udon_sync_mode(SyncMode)]  // Optional sync mode
pub struct MyBehaviour { }

/// Mark a field as public (visible in Unity Inspector)
#[udon_public]
pub field: Type,

/// Mark a field for network synchronization
#[udon_sync]
pub synced_field: Type,

/// Mark a method as a UdonSharp event handler
#[udon_event("EventName")]
pub fn event_handler(&mut self) { }

/// Mark a method as a custom network event
#[udon_network_event]
pub fn network_event(&mut self) { }
```

### Testing Attributes

```rust
/// Mark a function as a UdonSharp unit test
#[udon_test]
fn test_function() {
    // Test code
}

/// Mark a function as a UdonSharp integration test
#[udon_integration_test]
fn integration_test() {
    // Integration test code
}
```

## Error Types

```rust
/// Main error type for UdonSharp operations
#[derive(Debug)]
pub enum UdonSharpError {
    /// Compilation error
    CompilationError(String),
    
    /// Runtime error
    RuntimeError(String),
    
    /// Networking error
    NetworkingError(String),
    
    /// Unity integration error
    UnityError(String),
    
    /// Configuration error
    ConfigError(String),
}

impl std::fmt::Display for UdonSharpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UdonSharpError::CompilationError(msg) => write!(f, "Compilation error: {}", msg),
            UdonSharpError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            UdonSharpError::NetworkingError(msg) => write!(f, "Networking error: {}", msg),
            UdonSharpError::UnityError(msg) => write!(f, "Unity error: {}", msg),
            UdonSharpError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for UdonSharpError {}

pub type Result<T> = std::result::Result<T, UdonSharpError>;
```

## Logging

```rust
/// Logging macros for UdonSharp development
pub mod log {
    /// Log an info message
    pub fn info(message: &str);
    
    /// Log a warning message
    pub fn warn(message: &str);
    
    /// Log an error message
    pub fn error(message: &str);
    
    /// Log a debug message (only in debug builds)
    pub fn debug(message: &str);
}

// Usage
log::info("Player joined the world");
log::warn("Low performance detected");
log::error("Failed to initialize component");
log::debug("Debug information");
```

## Configuration

### Project Configuration

```rust
/// Configuration for UdonSharp projects
#[derive(Debug, Clone)]
pub struct UdonSharpConfig {
    /// Project name
    pub name: String,
    
    /// Target namespace for generated C#
    pub namespace: Option<String>,
    
    /// Sync mode for networking
    pub sync_mode: SyncMode,
    
    /// Whether to generate debug information
    pub generate_debug_info: bool,
    
    /// Whether to optimize for performance
    pub optimize_for_performance: bool,
    
    /// Target UdonSharp version
    pub target_udonsharp_version: String,
    
    /// Directories to scan for .asmdef files
    pub asmdef_scan_directories: Vec<String>,
    
    /// Path to custom binding rules
    pub custom_binding_rules: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SyncMode {
    None,
    Manual,
    Continuous,
}
```

## See Also

- [Getting Started](getting-started.md) - Basic setup and usage
- [Best Practices](best-practices.md) - Development guidelines
- [Compilation Pipeline](compilation-pipeline.md) - Build process details
- [Examples](../examples/) - Sample code and tutorials