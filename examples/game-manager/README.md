# Game Manager Example

An advanced example demonstrating complex state management, event systems, and game logic in Rust UdonSharp.

## What This Example Demonstrates

- Complex state machine implementation
- Event-driven architecture with custom events
- Advanced networking patterns
- Timer and scoring systems
- Player role management
- Comprehensive error handling and recovery
- Performance optimization techniques

## Features

- Multi-state game management (Lobby, Playing, Paused, Ended)
- Player role system (Host, Player, Spectator)
- Real-time scoring and leaderboards
- Game timer with pause/resume functionality
- Event broadcasting system
- Automatic game state recovery
- Comprehensive logging and debugging

## Code Overview

This example creates a sophisticated game management system that:
- Manages complex game states with proper transitions
- Handles player roles and permissions
- Implements a robust event system for loose coupling
- Provides real-time scoring and statistics
- Includes comprehensive error handling and recovery mechanisms
- Demonstrates advanced networking patterns

## Files

- `src/lib.rs` - Main game manager behavior
- `src/game_state.rs` - Game state management
- `src/player_roles.rs` - Player role system
- `src/scoring_system.rs` - Scoring and leaderboards
- `src/event_system.rs` - Custom event system
- `src/timer_system.rs` - Game timing functionality
- `Cargo.toml` - Project configuration
- `udonsharp.toml` - Advanced UdonSharp configuration

## Building

```bash
cd examples/game-manager
cargo udonsharp build
```

## Unity Setup

This example requires more complex Unity setup:

1. **Game Manager Object**: Create a GameObject with the `GameManager` behavior
2. **UI Elements**: Set up comprehensive UI system:
   - Game state display
   - Player list with roles
   - Scoreboard
   - Timer display
   - Control buttons (Start, Pause, Reset)
3. **Game Objects**: Create game-specific objects referenced by the manager
4. **Audio Sources**: Set up audio for game events
5. **Spawn Points**: Configure player spawn locations

## Key Concepts

### State Machine

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Lobby,
    Starting,
    Playing,
    Paused,
    Ending,
    Ended,
}

impl GameManager {
    fn transition_to_state(&mut self, new_state: GameState) {
        if self.can_transition_to(&new_state) {
            self.exit_current_state();
            self.game_state = new_state;
            self.enter_new_state();
        }
    }
}
```

### Event System

```rust
pub struct EventSystem {
    listeners: HashMap<String, Vec<Box<dyn Fn(&EventData)>>>,
}

// Usage
self.event_system.emit("game_started", &GameStartedEvent {
    duration: self.game_duration,
    players: self.get_active_players(),
});
```

### Player Roles

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerRole {
    Host,      // Can control game state
    Player,    // Can participate in game
    Spectator, // Can only observe
}
```

## Advanced Features

### Automatic Recovery

The game manager includes automatic recovery from network issues:

```rust
impl GameManager {
    fn recover_from_network_issue(&mut self) {
        // Detect and recover from common network problems
        if self.detect_desync() {
            self.request_full_sync();
        }
    }
}
```

### Performance Monitoring

Built-in performance monitoring and optimization:

```rust
impl GameManager {
    fn monitor_performance(&mut self) {
        let frame_time = unity::Time::delta_time();
        if frame_time > self.performance_threshold {
            self.optimize_for_performance();
        }
    }
}
```

### Comprehensive Logging

Detailed logging system for debugging:

```rust
impl GameManager {
    fn log_state_transition(&self, from: &GameState, to: &GameState) {
        log::info!("🎮 State transition: {:?} → {:?}", from, to);
        log::debug!("  - Players: {}", self.get_player_count());
        log::debug!("  - Time: {:.2}s", unity::Time::time());
    }
}
```

## Testing

Comprehensive test suite included:

```bash
cargo test
```

Tests cover:
- State machine transitions
- Event system functionality
- Scoring calculations
- Network synchronization
- Error recovery mechanisms

## Performance Considerations

This example demonstrates several performance optimization techniques:

- **Efficient State Updates**: Only update what's necessary
- **Event Batching**: Batch multiple events for network efficiency
- **Smart UI Updates**: Update UI elements only when needed
- **Memory Management**: Proper cleanup of temporary objects
- **Network Optimization**: Minimize network traffic

## Next Steps

This is an advanced example. After mastering it, you can:
- Implement your own game-specific logic
- Extend the event system for custom events
- Add more sophisticated player role systems
- Integrate with external services
- Build complex multiplayer game mechanics

## Related Examples

- [networking-demo](../networking-demo/) - Advanced networking patterns
- [player-counter](../player-counter/) - Basic networking and UI
- [world-portals](../world-portals/) - Complex world management