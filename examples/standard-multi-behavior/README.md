# Standard Multi-Behavior Pattern Example

This example demonstrates the Standard Multi-Behavior Pattern in UdonSharp-Rust, showing how to create a complete multiplayer game system using multiple interconnected UdonBehaviour structs.

## Overview

This example implements a simple multiplayer game system with the following components:

- **GameManager**: Manages overall game state, rounds, and timing
- **PlayerTracker**: Tracks player statistics and positions
- **UIController**: Manages the user interface and displays
- **ScoreSystem**: Handles scoring and leaderboards
- **SharedRuntime**: Automatically generated shared utilities

## Features Demonstrated

### Multi-Behavior Architecture
- Multiple `#[derive(UdonBehaviour)]` structs in a single module
- Automatic generation of separate C# classes for each behavior
- Clean separation of concerns with well-defined responsibilities

### Inter-Behavior Communication
- GameObject references between behaviors
- Custom event system for loose coupling
- Type-safe method calls between behaviors

### Network Synchronization
- Synchronized game state across all clients
- Master client validation for state changes
- Proper handling of player join/leave events

### Shared Functionality
- Automatic extraction of shared functions to SharedRuntime
- Common utility functions used across multiple behaviors
- Optimized code generation with minimal duplication

### Unity Integration
- Proper field attributes for Unity Inspector
- UI component references and updates
- Event handling for user interactions

## Project Structure

```
examples/standard-multi-behavior/
├── Cargo.toml              # Rust project configuration
├── udonsharp.toml          # UdonSharp-Rust configuration
├── build.rs                # Build script
├── src/
│   └── lib.rs              # Main source file with all behaviors
└── README.md               # This file
```

## Building the Example

1. Navigate to the example directory:
```bash
cd examples/standard-multi-behavior
```

2. Build the project:
```bash
cargo udonsharp build
```

This will generate the following C# files:
- `GameManager.cs`
- `PlayerTracker.cs`
- `UIController.cs`
- `ScoreSystem.cs`
- `SharedRuntime.cs`
- `GameSystemCoordinator.cs` (initialization coordinator)

## Generated Code Structure

### GameManager.cs
Manages the overall game state with synchronized fields:
- `currentGameState` - Current phase of the game (Lobby/Playing/Finished)
- `currentRound` - Current round number
- `timeRemaining` - Time left in the current round

### PlayerTracker.cs
Tracks player information:
- `activePlayerCount` - Number of active players (synchronized)
- Player position tracking and validation
- Event handlers for player join/leave

### UIController.cs
Manages all UI elements:
- Game state display
- Player count display
- Timer display
- Score display
- Button state management

### ScoreSystem.cs
Handles scoring mechanics:
- `highScore` - All-time high score (synchronized)
- `roundHighScore` - Current round high score (synchronized)
- `totalGamesPlayed` - Total number of games (synchronized)
- Local player score tracking

### SharedRuntime.cs
Contains shared utility functions:
- `FormatTime()` - Format seconds to MM:SS
- `FormatScore()` - Format scores with K/M suffixes
- `CalculateDistance()` - Calculate distance between points
- `ClampValue()` - Clamp values between min/max
- Logging utilities

## Usage in Unity

1. **Import Generated Files**: Copy the generated C# files to your Unity project's Assets folder.

2. **Set Up GameObjects**: Create GameObjects for each behavior:
   ```
   GameSystem (Empty GameObject)
   ├── GameManager (with GameManager.cs)
   ├── PlayerTracker (with PlayerTracker.cs)
   ├── UIController (with UIController.cs)
   ├── ScoreSystem (with ScoreSystem.cs)
   └── SharedRuntime (with SharedRuntime.cs)
   ```

3. **Configure UI References**: In the UIController component, assign the UI Text and Button references:
   - Game State Text
   - Player Count Text
   - Timer Text
   - Score Text
   - Start Button

4. **Set Up Prefabs**: Use the generated prefab files for easy setup, or create your own prefabs with the configured GameObjects.

## Key Concepts Demonstrated

### 1. Automatic Dependency Resolution
The behaviors automatically find each other using `GameObject::find()`:
```rust
self.ui_controller = unity::GameObject::find("UIController");
self.player_tracker = unity::GameObject::find("PlayerTracker");
```

### 2. Event-Driven Communication
Behaviors communicate through custom events:
```rust
// GameManager notifies other systems
if let Some(tracker) = &self.player_tracker {
    tracker.send_custom_event("OnRoundStart");
}

// PlayerTracker handles the event
#[udon_event("OnRoundStart")]
pub fn handle_round_start(&mut self) {
    // Reset tracking data for new round
}
```

### 3. Network Synchronization
Synchronized fields automatically handle networking:
```rust
#[udon_sync]
current_game_state: i32,

// Master client updates
if networking::is_master() {
    self.current_game_state = 1; // Playing
    networking::request_serialization();
}
```

### 4. Shared Function Extraction
Functions used by multiple behaviors are automatically moved to SharedRuntime:
```rust
// Used by both GameManager and UIController
pub fn format_time(seconds: f32) -> String {
    let total_seconds = seconds.max(0.0) as i32;
    let minutes = total_seconds / 60;
    let secs = total_seconds % 60;
    format!("{}:{:02}", minutes, secs)
}
```

## Customization

### Adding New Behaviors
To add a new behavior to the system:

1. Define a new struct with `#[derive(UdonBehaviour)]`
2. Implement the `UdonBehaviour` trait
3. Add references to other behaviors as needed
4. Implement custom event handlers for communication

### Modifying Game Logic
The example provides a foundation that can be extended:
- Add new game modes by extending the GameManager
- Implement different scoring systems in ScoreSystem
- Add new UI elements through UIController
- Track additional player statistics in PlayerTracker

### Configuration Options
Modify `udonsharp.toml` to change compilation behavior:
- Enable/disable SharedRuntime generation
- Change naming conventions
- Configure prefab generation
- Adjust initialization order

## Best Practices Demonstrated

1. **Clear Separation of Concerns**: Each behavior has a specific responsibility
2. **Event-Driven Architecture**: Loose coupling through custom events
3. **Master Client Pattern**: Proper network authority handling
4. **UI Synchronization**: Consistent UI updates across all clients
5. **Error Handling**: Null checks and validation throughout
6. **Performance Optimization**: Efficient update patterns and batched synchronization

## Troubleshooting

### Common Issues

**"GameObject not found" errors**:
- Ensure GameObject names match the `find()` calls exactly
- Check that GameObjects are active in the scene

**Synchronization not working**:
- Verify that only the master client is modifying synchronized fields
- Ensure `request_serialization()` is called after changes

**UI not updating**:
- Check that UI component references are properly assigned
- Verify that custom events are being sent correctly

### Debug Mode
Enable debug mode in `udonsharp.toml` for more detailed compilation information:
```toml
[build]
generate_debug_info = true
```

## Further Reading

- [Standard Multi-Behavior Pattern Documentation](../../docs/standard-multi-behavior-pattern.md)
- [UdonSharp-Rust API Reference](../../docs/api-reference.md)
- [Best Practices Guide](../../docs/best-practices.md)