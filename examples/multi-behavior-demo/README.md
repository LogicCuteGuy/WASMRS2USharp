# Multi-Behavior Demo

This example demonstrates the multi-behavior compilation feature of UdonSharp-Rust, showing how to structure a complex VRChat world with multiple interacting UdonBehaviour classes.

## Overview

This demo implements a simple multiplayer game system with the following behaviors:

- **PlayerManager**: Handles player join/leave events and player tracking
- **UIController**: Manages all user interface elements and updates
- **GameLogic**: Handles core game mechanics and state management
- **NetworkManager**: Manages synchronization and network events

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   PlayerManager │    │   UIController  │    │   GameLogic     │
│                 │    │                 │    │                 │
│ - Track players │    │ - Update UI     │    │ - Game state    │
│ - Join/Leave    │    │ - Display info  │    │ - Game rules    │
│ - Notify others │    │ - Handle events │    │ - Player mgmt   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │ NetworkManager  │
                    │                 │
                    │ - Sync data     │
                    │ - Network events│
                    │ - Master client │
                    └─────────────────┘
                                 │
                    ┌─────────────────┐
                    │ SharedRuntime   │
                    │                 │
                    │ - Common data   │
                    │ - Utilities     │
                    │ - Communication │
                    └─────────────────┘
```

## Key Features Demonstrated

### 1. Multi-Behavior Attributes

The example uses `#[udon_behaviour]` attributes to mark functions that should become separate UdonBehaviour classes:

```rust
#[udon_behaviour(name = "PlayerManager", events = ["Start", "OnPlayerJoined", "OnPlayerLeft"])]
pub fn player_manager_start() {
    // This becomes PlayerManager.cs with Start() method
}
```

### 2. Inter-Behavior Communication

Behaviors communicate through custom events:

```rust
pub fn notify_ui_player_count_changed() {
    send_custom_event("UIController", "OnPlayerCountChanged");
}
```

### 3. Shared Functionality

Common functions and data are automatically moved to SharedRuntime:

```rust
pub fn get_player_count() -> i32 {
    unsafe { PLAYER_COUNT }
}
```

### 4. Event Handling

Each behavior can handle specific Unity events:

```rust
#[udon_behaviour_event(behavior = "PlayerManager")]
pub fn on_player_joined(player: VRCPlayerApi) {
    // Handle player join event
}
```

## Configuration

The project uses `udonsharp.toml` to configure multi-behavior compilation:

```toml
[multi_behavior]
enabled = true
generate_shared_runtime = true
naming_convention = "PascalCase"
min_behaviors_threshold = 2
generate_prefabs = true

[multi_behavior.prefab_settings]
generate_individual_prefabs = true
generate_master_prefab = true
auto_setup_references = true
include_example_scene = true
```

## Generated Output

When compiled, this project generates:

### UdonBehaviour Classes
- `PlayerManager.cs` - Player management behavior
- `UIController.cs` - UI management behavior  
- `GameLogic.cs` - Game logic behavior
- `NetworkManager.cs` - Network management behavior
- `SharedRuntime.cs` - Shared utilities and data

### Unity Assets
- `PlayerManager.prefab` - Individual behavior prefab
- `UIController.prefab` - Individual behavior prefab
- `GameLogic.prefab` - Individual behavior prefab
- `NetworkManager.prefab` - Individual behavior prefab
- `MultiBehaviorSystem.prefab` - Master prefab with all behaviors
- `ExampleScene.unity` - Example scene setup
- `BehaviorCoordinator.cs` - Initialization coordinator

### Editor Scripts
- `Editor/MultiBehaviorExecutionOrderSetup.cs` - Script execution order setup

## Building

To build this example:

```bash
# Navigate to the example directory
cd examples/multi-behavior-demo

# Run the build script
cargo run --bin build

# Or build manually
cargo build --target wasm32-unknown-unknown --release
```

## Usage in Unity

1. Import the generated C# files into your Unity project
2. Import the generated prefabs
3. Add the `MultiBehaviorSystem.prefab` to your scene
4. The behaviors will automatically initialize in the correct order

## Inter-Behavior Dependencies

The example demonstrates proper dependency management:

```
PlayerManager → UIController (player count updates)
PlayerManager → GameLogic (player join/leave notifications)
GameLogic → UIController (game state updates)
GameLogic → NetworkManager (state synchronization)
NetworkManager → All (network updates)
```

## Best Practices Demonstrated

1. **Clear Separation of Concerns**: Each behavior has a specific responsibility
2. **Loose Coupling**: Behaviors communicate through events, not direct calls
3. **Shared State Management**: Common data is managed in SharedRuntime
4. **Proper Initialization**: Dependencies are initialized in the correct order
5. **Network Synchronization**: State changes are properly synchronized
6. **Error Handling**: Graceful handling of edge cases (insufficient players, etc.)

## Extending the Example

To add new behaviors:

1. Create new functions with `#[udon_behaviour]` attributes
2. Implement the required event handlers
3. Add inter-behavior communication as needed
4. Update the configuration if necessary
5. Rebuild the project

## Performance Considerations

- Shared functions minimize code duplication
- Event-based communication reduces coupling
- Proper initialization order prevents race conditions
- Network synchronization is optimized for VRChat constraints

This example serves as a comprehensive template for building complex VRChat worlds with multiple interacting systems while maintaining clean, maintainable code structure.