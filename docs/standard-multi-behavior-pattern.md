# Standard Multi-Behavior Pattern

The Standard Multi-Behavior Pattern is a powerful feature of UdonSharp-Rust that allows you to define multiple `UdonBehaviour` structs in a single Rust module. This pattern enables clean organization of complex VRChat world logic while maintaining type safety and generating optimized UdonSharp C# code.

## Table of Contents

- [Overview](#overview)
- [Getting Started](#getting-started)
- [Basic Usage](#basic-usage)
- [Advanced Features](#advanced-features)
- [Inter-Behavior Communication](#inter-behavior-communication)
- [Network Synchronization](#network-synchronization)
- [SharedRuntime](#sharedruntime)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)
- [API Reference](#api-reference)

## Overview

### What is the Standard Multi-Behavior Pattern?

The Standard Multi-Behavior Pattern allows you to:

- Define multiple `#[derive(UdonBehaviour)]` structs in a single Rust file
- Automatically generate separate UdonSharp C# classes for each behavior
- Share common functionality through an automatically generated `SharedRuntime` class
- Enable type-safe communication between behaviors
- Maintain proper dependency management and initialization order

### Benefits

- **Clean Organization**: Group related behaviors together in logical modules
- **Code Reuse**: Shared functions are automatically extracted to `SharedRuntime`
- **Type Safety**: Full Rust type checking for all behavior interactions
- **Performance**: Optimized C# code generation with minimal overhead
- **Maintainability**: Clear separation of concerns with automatic dependency management

## Getting Started

### Prerequisites

- UdonSharp-Rust compiler installed
- Basic understanding of Rust and UdonSharp
- VRChat SDK3 with UdonSharp

### Project Setup

1. Create a new Rust project or use an existing one:

```bash
cargo new my-vrchat-world
cd my-vrchat-world
```

2. Add UdonSharp-Rust dependencies to `Cargo.toml`:

```toml
[dependencies]
udonsharp-core = "0.1"
udonsharp-macros = "0.1"

[lib]
crate-type = ["cdylib"]
```

3. Configure multi-behavior compilation in `udonsharp.toml`:

```toml
[multi_behavior]
enabled = true
generate_shared_runtime = true
naming_convention = "PascalCase"
min_behaviors_threshold = 2

[multi_behavior.prefab_settings]
generate_individual_prefabs = true
generate_master_prefab = true
auto_setup_references = true
```

## Basic Usage

### Defining Multiple Behaviors

Create multiple UdonBehaviour structs in your `src/lib.rs`:

```rust
use udonsharp_core::prelude::*;

#[derive(UdonBehaviour)]
pub struct PlayerManager {
    #[udon_public]
    max_players: i32,
    
    #[udon_sync]
    current_player_count: i32,
    
    ui_controller: Option<unity::GameObject>,
}

impl UdonBehaviour for PlayerManager {
    fn start(&mut self) {
        self.max_players = 20;
        self.current_player_count = 0;
        
        // Find the UI controller
        self.ui_controller = unity::GameObject::find("UIController");
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.current_player_count += 1;
        self.update_ui();
        
        if networking::is_master() {
            networking::request_serialization();
        }
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.current_player_count -= 1;
        self.update_ui();
        
        if networking::is_master() {
            networking::request_serialization();
        }
    }
}

impl PlayerManager {
    fn update_ui(&self) {
        if let Some(ui_obj) = &self.ui_controller {
            ui_obj.send_custom_event("UpdatePlayerCount");
        }
    }
}

#[derive(UdonBehaviour)]
pub struct UIController {
    #[udon_public]
    player_count_text: Option<unity::ui::Text>,
    
    player_manager: Option<unity::GameObject>,
}

impl UdonBehaviour for UIController {
    fn start(&mut self) {
        // Find the player manager
        self.player_manager = unity::GameObject::find("PlayerManager");
    }
}

impl UIController {
    #[udon_event("UpdatePlayerCount")]
    pub fn update_player_count(&mut self) {
        if let Some(pm_obj) = &self.player_manager {
            if let Some(pm) = pm_obj.get_component::<PlayerManager>() {
                let count = pm.current_player_count;
                
                if let Some(text) = &self.player_count_text {
                    text.set_text(&format!("Players: {}/{}", count, pm.max_players));
                }
            }
        }
    }
}

// Shared utility functions (automatically moved to SharedRuntime)
pub fn format_time(seconds: f32) -> String {
    let minutes = (seconds / 60.0) as i32;
    let secs = (seconds % 60.0) as i32;
    format!("{}:{:02}", minutes, secs)
}

pub fn clamp_value(value: f32, min: f32, max: f32) -> f32 {
    if value < min { min }
    else if value > max { max }
    else { value }
}
```

### Compilation

Compile your project using the UdonSharp-Rust compiler:

```bash
cargo udonsharp build
```

This will generate:
- `PlayerManager.cs` - The player management behavior
- `UIController.cs` - The UI management behavior  
- `SharedRuntime.cs` - Shared utility functions
- Unity prefab files (if enabled)

## Advanced Features

### Field Attributes

Control how struct fields are converted to C# properties:

```rust
#[derive(UdonBehaviour)]
pub struct AdvancedBehavior {
    // Public field visible in Unity Inspector
    #[udon_public]
    #[header("Configuration")]
    #[tooltip("Maximum number of items")]
    max_items: i32,
    
    // Synchronized field for networking
    #[udon_public]
    #[udon_sync]
    shared_data: f32,
    
    // Private field (not visible in Inspector)
    internal_state: bool,
    
    // GameObject reference for inter-behavior communication
    other_behavior: Option<unity::GameObject>,
}
```

### Method Attributes

Create custom event handlers for inter-behavior communication:

```rust
impl AdvancedBehavior {
    // Custom event handler
    #[udon_event("OnDataChanged")]
    pub fn handle_data_change(&mut self, new_value: f32) {
        self.shared_data = new_value;
        
        if networking::is_master() {
            networking::request_serialization();
        }
    }
    
    // Unity event methods
    fn update(&mut self) {
        // Called every frame
    }
    
    fn on_trigger_enter(&mut self, other: unity::Collider) {
        // Handle trigger events
    }
}
```

### Synchronization Modes

Configure network synchronization behavior:

```rust
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]  // Manual synchronization
pub struct NetworkedBehavior {
    #[udon_sync]
    synchronized_value: i32,
}

impl UdonBehaviour for NetworkedBehavior {
    fn on_post_deserialization(&mut self) {
        // Called when network data is received
        self.handle_sync_update();
    }
}

impl NetworkedBehavior {
    fn update_value(&mut self, new_value: i32) {
        if networking::is_master() {
            self.synchronized_value = new_value;
            networking::request_serialization();
        }
    }
    
    fn handle_sync_update(&self) {
        // React to synchronized data changes
    }
}
```

## Inter-Behavior Communication

### GameObject References

Behaviors can reference each other through GameObject fields:

```rust
#[derive(UdonBehaviour)]
pub struct BehaviorA {
    behavior_b_ref: Option<unity::GameObject>,
}

impl UdonBehaviour for BehaviorA {
    fn start(&mut self) {
        // Find other behavior by GameObject name
        self.behavior_b_ref = unity::GameObject::find("BehaviorB");
    }
}

impl BehaviorA {
    fn communicate_with_b(&self) {
        if let Some(b_obj) = &self.behavior_b_ref {
            // Send custom event
            b_obj.send_custom_event("HandleMessage");
            
            // Or access component directly (if available)
            if let Some(b_component) = b_obj.get_component::<BehaviorB>() {
                // Direct method call (compile-time checked)
                b_component.receive_message("Hello from A!");
            }
        }
    }
}
```

### Custom Events

Use custom events for loose coupling between behaviors:

```rust
#[derive(UdonBehaviour)]
pub struct EventSender {
    receivers: Vec<unity::GameObject>,
}

impl EventSender {
    fn broadcast_event(&self, event_name: &str) {
        for receiver in &self.receivers {
            receiver.send_custom_event(event_name);
        }
    }
}

#[derive(UdonBehaviour)]
pub struct EventReceiver {
    // Event handlers are automatically made public in generated C#
    #[udon_event("OnGameStart")]
    pub fn handle_game_start(&mut self) {
        // Handle the event
    }
    
    #[udon_event("OnGameEnd")]
    pub fn handle_game_end(&mut self) {
        // Handle the event
    }
}
```

## Network Synchronization

### Synchronized Fields

Fields marked with `#[udon_sync]` are automatically synchronized across the network:

```rust
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct GameState {
    #[udon_public]
    #[udon_sync]
    game_time: f32,
    
    #[udon_public]
    #[udon_sync]
    current_round: i32,
    
    #[udon_public]
    #[udon_sync]
    is_game_active: bool,
}

impl GameState {
    fn update_game_time(&mut self, delta_time: f32) {
        if networking::is_master() {
            self.game_time += delta_time;
            
            // Request synchronization when data changes
            networking::request_serialization();
        }
    }
}
```

### Master Client Validation

The compiler automatically generates master client checks:

```rust
impl GameState {
    fn start_new_round(&mut self) {
        // This will be wrapped with master client check in generated C#
        if networking::is_master() {
            self.current_round += 1;
            self.is_game_active = true;
            networking::request_serialization();
        }
    }
}
```

### Deserialization Handling

Handle incoming network data:

```rust
impl UdonBehaviour for GameState {
    fn on_post_deserialization(&mut self) {
        // Called when synchronized data is received
        self.update_ui_elements();
        self.notify_other_behaviors();
    }
}
```

## SharedRuntime

### Automatic Function Extraction

Functions used by multiple behaviors are automatically moved to `SharedRuntime`:

```rust
// This function is used by multiple behaviors
pub fn calculate_distance(pos1: Vector3, pos2: Vector3) -> f32 {
    let diff = pos1 - pos2;
    (diff.x * diff.x + diff.y * diff.y + diff.z * diff.z).sqrt()
}

// This function is used by multiple behaviors
pub fn format_score(score: i32) -> String {
    if score >= 1000000 {
        format!("{}M", score / 1000000)
    } else if score >= 1000 {
        format!("{}K", score / 1000)
    } else {
        score.to_string()
    }
}

#[derive(UdonBehaviour)]
pub struct BehaviorA {
    position: Vector3,
}

impl BehaviorA {
    fn check_distance_to_origin(&self) -> f32 {
        // This call will be converted to SharedRuntime.CalculateDistance()
        calculate_distance(self.position, Vector3::zero())
    }
}

#[derive(UdonBehaviour)]
pub struct BehaviorB {
    target_position: Vector3,
}

impl BehaviorB {
    fn get_distance_to_target(&self, current_pos: Vector3) -> f32 {
        // This call will also use SharedRuntime.CalculateDistance()
        calculate_distance(current_pos, self.target_position)
    }
}
```

### Generated SharedRuntime.cs

The compiler generates a SharedRuntime class:

```csharp
using UnityEngine;
using UdonSharp;

public class SharedRuntime : UdonSharpBehaviour
{
    public static float CalculateDistance(Vector3 pos1, Vector3 pos2)
    {
        Vector3 diff = pos1 - pos2;
        return Mathf.Sqrt(diff.x * diff.x + diff.y * diff.y + diff.z * diff.z);
    }
    
    public static string FormatScore(int score)
    {
        if (score >= 1000000)
            return $"{score / 1000000}M";
        else if (score >= 1000)
            return $"{score / 1000}K";
        else
            return score.ToString();
    }
}
```

### Shared Data Types

Enums and structs used by multiple behaviors are also moved to SharedRuntime:

```rust
// Shared enum
pub enum GameMode {
    Lobby,
    Playing,
    Finished,
}

// Shared struct
pub struct PlayerStats {
    pub score: i32,
    pub kills: i32,
    pub deaths: i32,
}

// Multiple behaviors can use these types
#[derive(UdonBehaviour)]
pub struct GameManager {
    current_mode: GameMode,
    player_stats: Vec<PlayerStats>,
}

#[derive(UdonBehaviour)]
pub struct UIManager {
    displayed_mode: GameMode,
}
```

## Best Practices

### Project Organization

1. **Group Related Behaviors**: Keep behaviors that work together in the same module
2. **Use Descriptive Names**: Behavior names become C# class names, so use clear, descriptive names
3. **Minimize Dependencies**: Reduce coupling between behaviors where possible
4. **Document Public APIs**: Add comments to public methods and fields

### Performance Optimization

1. **Minimize Network Sync**: Only synchronize data that truly needs to be shared
2. **Batch Updates**: Group related changes and call `request_serialization()` once
3. **Use SharedRuntime**: Let the compiler extract shared functions automatically
4. **Avoid Circular Dependencies**: Design behaviors with clear hierarchies

### Error Handling

1. **Validate GameObject References**: Always check if GameObject references are valid
2. **Handle Network Edge Cases**: Consider what happens when players join/leave
3. **Use Option Types**: Leverage Rust's Option type for nullable references
4. **Test Compilation**: Regularly compile to catch errors early

### Code Style

```rust
// Good: Clear, descriptive names
#[derive(UdonBehaviour)]
pub struct PlayerInventoryManager {
    #[udon_public]
    max_inventory_slots: i32,
    
    #[udon_sync]
    current_item_count: i32,
}

// Good: Proper error handling
impl PlayerInventoryManager {
    fn add_item(&mut self, item_id: i32) -> bool {
        if self.current_item_count < self.max_inventory_slots {
            self.current_item_count += 1;
            
            if networking::is_master() {
                networking::request_serialization();
            }
            
            true
        } else {
            false // Inventory full
        }
    }
}

// Good: Clear event handlers
impl PlayerInventoryManager {
    #[udon_event("OnItemPickup")]
    pub fn handle_item_pickup(&mut self, item_id: i32) {
        if self.add_item(item_id) {
            self.update_inventory_ui();
        } else {
            self.show_inventory_full_message();
        }
    }
}
```

## Troubleshooting

### Common Issues

#### "No UdonBehaviour structs found"

**Problem**: The compiler can't find any structs with `#[derive(UdonBehaviour)]`

**Solution**:
- Ensure you have `#[derive(UdonBehaviour)]` on your structs
- Check that `udonsharp-core` is properly imported
- Verify the source file is being read correctly

#### "Circular dependency detected"

**Problem**: Behaviors have circular references to each other

**Solution**:
- Redesign the dependency structure
- Use events instead of direct references
- Consider extracting shared logic to SharedRuntime

#### "Field attribute validation failed"

**Problem**: Invalid combination of field attributes

**Solution**:
- Ensure `#[udon_sync]` fields are also `#[udon_public]`
- Check that attribute syntax is correct
- Verify field types are UdonSharp-compatible

#### "Generated code validation failed"

**Problem**: The generated C# code has syntax errors

**Solution**:
- Check for invalid C# identifiers in field/method names
- Ensure all types are UdonSharp-compatible
- Report the issue if it's a compiler bug

### Debug Mode

Enable debug mode for more detailed error messages:

```toml
[build]
generate_debug_info = true

[multi_behavior]
enabled = true
# ... other settings
```

### Compilation Reports

The compiler generates detailed reports:

```bash
cargo udonsharp build --verbose
```

Example output:
```
=== Standard Multi-Behavior Compilation Report ===

Compilation Status: SUCCESS
Total Behaviors: 3
Total Files Generated: 4
Shared Functions: 2
Inter-Behavior Calls: 5
Has Networking: true
Dependencies: 2

--- Generated Files ---
  • PlayerManager.cs
  • UIController.cs
  • GameState.cs
  • SharedRuntime.cs

=== End Report ===
```

## API Reference

### Attributes

#### Struct Attributes

- `#[derive(UdonBehaviour)]` - Marks a struct as a UdonBehaviour
- `#[udon_sync_mode(Manual|Continuous|None)]` - Sets synchronization mode

#### Field Attributes

- `#[udon_public]` - Makes field public and serializable in Unity
- `#[udon_sync]` - Synchronizes field across network
- `#[header("text")]` - Adds header in Unity Inspector
- `#[tooltip("text")]` - Adds tooltip in Unity Inspector

#### Method Attributes

- `#[udon_event("EventName")]` - Creates custom event handler

### Traits

#### UdonBehaviour

Required trait for all UdonBehaviour structs:

```rust
pub trait UdonBehaviour {
    // Required
    fn start(&mut self) {}
    
    // Optional Unity events
    fn update(&mut self) {}
    fn fixed_update(&mut self) {}
    fn late_update(&mut self) {}
    fn on_enable(&mut self) {}
    fn on_disable(&mut self) {}
    fn on_destroy(&mut self) {}
    
    // VRChat events
    fn on_player_joined(&mut self, player: VRCPlayerApi) {}
    fn on_player_left(&mut self, player: VRCPlayerApi) {}
    
    // Networking events
    fn on_post_deserialization(&mut self) {}
    
    // Interaction events
    fn on_pickup(&mut self) {}
    fn on_drop(&mut self) {}
    fn on_pickup_use_down(&mut self) {}
    fn on_pickup_use_up(&mut self) {}
    
    // Trigger/Collision events
    fn on_trigger_enter(&mut self, other: unity::Collider) {}
    fn on_trigger_exit(&mut self, other: unity::Collider) {}
    fn on_trigger_stay(&mut self, other: unity::Collider) {}
    fn on_collision_enter(&mut self, collision: unity::Collision) {}
    fn on_collision_exit(&mut self, collision: unity::Collision) {}
    fn on_collision_stay(&mut self, collision: unity::Collision) {}
}
```

### Configuration

#### udonsharp.toml

```toml
[multi_behavior]
# Enable multi-behavior compilation
enabled = true

# Generate SharedRuntime for shared functions
generate_shared_runtime = true

# Naming convention for generated classes
naming_convention = "PascalCase"  # or "PascalCaseWithSuffix" or "Custom"

# Minimum behaviors to trigger multi-behavior mode
min_behaviors_threshold = 2

# Generate Unity prefab files
generate_prefabs = true

[multi_behavior.prefab_settings]
generate_individual_prefabs = true
generate_master_prefab = true
auto_setup_references = true
include_example_scene = false
output_directory = "Generated/Prefabs"

[multi_behavior.initialization_order]
auto_determine_order = true
generate_coordinator = true
coordinator_class_name = "BehaviorCoordinator"
use_script_execution_order = true
```

### Supported Types

#### Primitive Types
- `bool` → `bool`
- `i32` → `int`
- `f32` → `float`
- `String` → `string`

#### Unity Types
- `Vector2` → `Vector2`
- `Vector3` → `Vector3`
- `Vector4` → `Vector4`
- `Quaternion` → `Quaternion`
- `Color` → `Color`
- `GameObject` → `GameObject`
- `Transform` → `Transform`

#### VRChat Types
- `VRCPlayerApi` → `VRCPlayerApi`

#### Container Types
- `Option<T>` → `T` (nullable)
- `Vec<T>` → `T[]`
- `HashMap<K,V>` → `Dictionary<K,V>` (limited support)

For the most up-to-date information and advanced usage examples, see the [UdonSharp-Rust GitHub repository](https://github.com/example/udonsharp-rust).