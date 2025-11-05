# Player Counter Example

An intermediate example demonstrating networking, synchronization, and UI management in Rust UdonSharp.

## What This Example Demonstrates

- Network synchronization with `#[udon_sync]`
- Manual sync mode for controlled networking
- UI system integration
- Event-driven architecture
- Performance optimization techniques
- Error handling and validation

## Features

- Real-time player count display
- Network-synchronized player statistics
- Join/leave notifications with timestamps
- Player list management
- UI updates with smooth animations
- Master client responsibilities

## Code Overview

This example creates a comprehensive player tracking system that:
- Synchronizes player count across all clients
- Maintains a list of active players with join times
- Updates UI elements in real-time
- Handles master client changes gracefully
- Provides detailed logging and statistics

## Files

- `src/lib.rs` - Main player counter behavior
- `src/player_data.rs` - Player data structures
- `src/ui_manager.rs` - UI management system
- `Cargo.toml` - Project configuration
- `udonsharp.toml` - UdonSharp settings with networking enabled

## Building

```bash
cd examples/player-counter
cargo udonsharp build
```

## Unity Setup

1. Copy generated C# files to your Unity Assets folder
2. Create a GameObject with the `PlayerCounter` behavior
3. Set up UI elements:
   - Create a Canvas with a Text component named "PlayerCountText"
   - Create a Text component named "PlayerListText" for the player list
   - Optionally create "JoinNotification" and "LeaveNotification" text objects
4. Configure the public fields in the Inspector
5. Build and test with multiple clients

## Key Concepts

### Network Synchronization

```rust
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]  // Manual control over when to sync
pub struct PlayerCounter {
    #[udon_sync]  // This field is synchronized across network
    pub total_players: i32,
    
    #[udon_sync]
    pub master_player_name: String,
}
```

### Manual Sync Control

```rust
impl PlayerCounter {
    fn update_player_count(&mut self) {
        if self.is_master() {
            self.total_players = self.get_current_player_count();
            self.request_serialization();  // Trigger network sync
        }
    }
}
```

### UI Management

```rust
impl PlayerCounter {
    fn update_ui(&self) {
        if let Some(text) = self.find_ui_text("PlayerCountText") {
            text.set_text(&format!("Players: {}", self.total_players));
        }
    }
}
```

## Advanced Features

### Player Data Tracking

The example includes a comprehensive player data system:

```rust
pub struct PlayerData {
    pub player_api: VRCPlayerApi,
    pub display_name: String,
    pub join_time: f32,
    pub is_vr_user: bool,
    pub session_duration: f32,
}
```

### Event System

Uses an event-driven architecture for loose coupling:

```rust
pub enum PlayerEvent {
    Joined(PlayerData),
    Left(PlayerData),
    MasterChanged(VRCPlayerApi),
}
```

### Performance Optimization

- Batched UI updates
- Efficient player list management
- Minimal network traffic
- Smart update intervals

## Testing

The example includes comprehensive tests:

```bash
cargo test
```

Tests cover:
- Player data management
- Network synchronization logic
- UI update mechanisms
- Error handling scenarios

## Next Steps

After mastering this example, try:
- [interactive-button](../interactive-button/) - Learn about user interactions
- [game-manager](../game-manager/) - Explore complex state management
- [networking-demo](../networking-demo/) - Advanced networking patterns