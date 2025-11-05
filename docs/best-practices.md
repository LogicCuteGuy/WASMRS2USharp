# Best Practices for Rust UdonSharp Development

This guide covers best practices for developing VRChat worlds using the Rust UdonSharp framework.

## Project Organization

### Crate Structure

Organize your project using Rust's module system:

```rust
// src/lib.rs - Main entry point
pub mod behaviors;
pub mod systems;
pub mod utilities;
pub mod events;

pub use behaviors::*;
pub use systems::*;

// Re-export commonly used items
pub mod prelude {
    pub use crate::behaviors::*;
    pub use udonsharp::prelude::*;
}
```

```rust
// src/behaviors/mod.rs - UdonSharp behaviors
pub mod world_controller;
pub mod player_manager;
pub mod interactive_objects;

pub use world_controller::WorldController;
pub use player_manager::PlayerManager;
pub use interactive_objects::*;
```

```rust
// src/systems/mod.rs - Game systems
pub mod networking;
pub mod audio;
pub mod ui;

pub use networking::NetworkingSystem;
pub use audio::AudioSystem;
pub use ui::UISystem;
```

### File Naming Conventions

Follow Rust naming conventions:

- **Modules**: `snake_case` (e.g., `player_manager.rs`)
- **Structs**: `PascalCase` (e.g., `WorldController`)
- **Functions**: `snake_case` (e.g., `on_player_joined`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_PLAYERS`)

## UdonSharp Behavior Design

### Trait Implementation

Always implement the `UdonBehaviour` trait for your main behaviors:

```rust
use udonsharp::prelude::*;

#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct WorldController {
    #[udon_public]
    pub world_name: String,
    
    #[udon_sync]
    pub current_scene: i32,
    
    // Private fields don't need attributes
    initialized: bool,
    players: Vec<VRCPlayerApi>,
}

impl UdonBehaviour for WorldController {
    fn start(&mut self) {
        self.initialized = true;
        self.setup_world();
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        self.update_world_state();
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.players.push(player);
        self.welcome_player(&player);
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.players.retain(|p| p.player_id() != player.player_id());
        self.update_player_count();
    }
}
```

### Attribute Usage

Use UdonSharp attributes appropriately:

```rust
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]  // Use Manual for better control
pub struct GameManager {
    // Public fields visible in Unity Inspector
    #[udon_public]
    pub game_duration: f32,
    
    #[udon_public]
    pub max_players: i32,
    
    // Synchronized fields for networking
    #[udon_sync]
    pub game_state: GameState,
    
    #[udon_sync]
    pub current_round: i32,
    
    // Private fields (not synchronized)
    local_timer: f32,
    ui_elements: Vec<unity::GameObject>,
}
```

### Event Handling

Use the `#[udon_event]` attribute for UdonSharp events:

```rust
impl InteractableButton {
    #[udon_event("OnInteract")]
    pub fn on_interact(&mut self) {
        self.handle_button_press();
    }
    
    #[udon_event("OnTriggerEnter")]
    pub fn on_trigger_enter(&mut self, other: unity::Collider) {
        if let Some(player) = other.get_component::<VRCPlayerApi>() {
            self.handle_player_enter(player);
        }
    }
    
    #[udon_event("OnOwnershipTransferred")]
    pub fn on_ownership_transferred(&mut self, player: VRCPlayerApi) {
        log::info!("Ownership transferred to: {}", player.get_display_name());
    }
}
```

## Networking Best Practices

### Sync Mode Selection

Choose the appropriate sync mode:

```rust
// Manual sync - Best for most cases, gives you control
#[udon_sync_mode(Manual)]
pub struct ManualSyncBehavior {
    #[udon_sync]
    pub shared_data: String,
}

impl ManualSyncBehavior {
    pub fn update_shared_data(&mut self, new_data: String) {
        if vrchat::Networking::is_owner(&vrchat::Networking::local_player()) {
            self.shared_data = new_data;
            self.request_serialization(); // Manually trigger sync
        }
    }
}

// Continuous sync - Use sparingly, for frequently changing data
#[udon_sync_mode(Continuous)]
pub struct ContinuousSyncBehavior {
    #[udon_sync]
    pub position: unity::Vector3, // Continuously synced position
}
```

### Ownership Management

Handle ownership properly:

```rust
impl NetworkedObject {
    pub fn take_ownership(&mut self) {
        let local_player = vrchat::Networking::local_player();
        
        if !vrchat::Networking::is_owner(&local_player) {
            vrchat::Networking::set_owner(&local_player, &self.game_object());
        }
    }
    
    pub fn can_modify(&self) -> bool {
        let local_player = vrchat::Networking::local_player();
        vrchat::Networking::is_owner(&local_player)
    }
    
    pub fn modify_if_owner(&mut self, modification: impl FnOnce(&mut Self)) {
        if self.can_modify() {
            modification(self);
            self.request_serialization();
        }
    }
}
```

### Network Events

Use network events for important state changes:

```rust
impl GameManager {
    pub fn start_game(&mut self) {
        if self.can_modify() {
            self.game_state = GameState::Playing;
            self.send_custom_network_event(
                vrchat::NetworkEventTarget::All,
                "OnGameStarted"
            );
        }
    }
    
    #[udon_event("OnGameStarted")]
    pub fn on_game_started(&mut self) {
        // This runs on all clients
        self.initialize_game_ui();
        self.start_local_timer();
    }
}
```

## Performance Optimization

### Memory Management

Be mindful of memory usage in VRChat:

```rust
// Good: Use Vec for dynamic collections
pub struct EfficientManager {
    players: Vec<VRCPlayerApi>,           // Grows as needed
    active_objects: Vec<unity::GameObject>, // Reuse when possible
}

impl EfficientManager {
    pub fn add_player(&mut self, player: VRCPlayerApi) {
        // Check if already exists before adding
        if !self.players.iter().any(|p| p.player_id() == player.player_id()) {
            self.players.push(player);
        }
    }
    
    pub fn remove_player(&mut self, player: VRCPlayerApi) {
        // Use retain for efficient removal
        self.players.retain(|p| p.player_id() != player.player_id());
    }
    
    pub fn clear_inactive_objects(&mut self) {
        // Clean up periodically
        self.active_objects.retain(|obj| obj.active_in_hierarchy());
    }
}
```

### Avoid Expensive Operations

Minimize expensive operations in `update()`:

```rust
impl WorldController {
    fn update(&mut self) {
        // Good: Use timers for periodic operations
        self.update_timer += unity::Time::delta_time();
        
        if self.update_timer >= 1.0 {  // Update once per second
            self.update_timer = 0.0;
            self.expensive_operation();
        }
        
        // Good: Early returns for inactive states
        if !self.is_active {
            return;
        }
        
        // Good: Cache frequently accessed components
        self.update_cached_components();
    }
    
    fn expensive_operation(&mut self) {
        // Expensive operations here
        self.update_all_player_positions();
        self.check_game_conditions();
    }
}
```

### Object Pooling

Implement object pooling for frequently created/destroyed objects:

```rust
pub struct ObjectPool<T> {
    available: Vec<T>,
    in_use: Vec<T>,
    factory: Box<dyn Fn() -> T>,
}

impl<T> ObjectPool<T> {
    pub fn new(factory: Box<dyn Fn() -> T>) -> Self {
        Self {
            available: Vec::new(),
            in_use: Vec::new(),
            factory,
        }
    }
    
    pub fn get(&mut self) -> T {
        if let Some(obj) = self.available.pop() {
            self.in_use.push(obj);
            obj
        } else {
            let obj = (self.factory)();
            self.in_use.push(obj);
            obj
        }
    }
    
    pub fn return_object(&mut self, obj: T) {
        if let Some(pos) = self.in_use.iter().position(|x| /* compare */) {
            let obj = self.in_use.remove(pos);
            self.available.push(obj);
        }
    }
}
```

## Error Handling

### Graceful Degradation

Handle errors gracefully in VRChat environments:

```rust
impl WorldController {
    pub fn safe_find_object(&self, name: &str) -> Option<unity::GameObject> {
        match unity::GameObject::find(name) {
            Some(obj) => {
                log::info!("Found object: {}", name);
                Some(obj)
            }
            None => {
                log::warn!("Object not found: {}", name);
                None
            }
        }
    }
    
    pub fn safe_get_component<T>(&self, obj: &unity::GameObject) -> Option<T> {
        match obj.get_component::<T>() {
            Some(component) => Some(component),
            None => {
                log::warn!("Component not found on object: {}", obj.name());
                None
            }
        }
    }
    
    pub fn initialize_with_fallbacks(&mut self) {
        // Try to initialize with preferred settings
        if let Some(ui_canvas) = self.safe_find_object("MainCanvas") {
            self.setup_ui(ui_canvas);
        } else {
            // Fallback to basic UI
            log::warn!("Main canvas not found, using fallback UI");
            self.setup_fallback_ui();
        }
    }
}
```

### Validation

Validate inputs and state:

```rust
impl PlayerManager {
    pub fn add_player(&mut self, player: VRCPlayerApi) -> Result<(), String> {
        // Validate player
        if !player.is_valid() {
            return Err("Invalid player API".to_string());
        }
        
        // Check capacity
        if self.players.len() >= self.max_players as usize {
            return Err("Maximum players reached".to_string());
        }
        
        // Check for duplicates
        if self.players.iter().any(|p| p.player_id() == player.player_id()) {
            return Err("Player already exists".to_string());
        }
        
        self.players.push(player);
        Ok(())
    }
    
    pub fn set_player_score(&mut self, player_id: i32, score: i32) -> Result<(), String> {
        if score < 0 {
            return Err("Score cannot be negative".to_string());
        }
        
        if let Some(player_data) = self.player_scores.get_mut(&player_id) {
            player_data.score = score;
            Ok(())
        } else {
            Err("Player not found".to_string())
        }
    }
}
```

## Testing Strategies

### Unit Testing

Write unit tests for your logic:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use udonsharp::testing::*;
    
    #[udon_test]
    fn test_player_manager_add_player() {
        let mut manager = PlayerManager::new();
        let mock_player = create_mock_player("TestPlayer");
        
        let result = manager.add_player(mock_player);
        assert!(result.is_ok());
        assert_eq!(manager.player_count(), 1);
    }
    
    #[udon_test]
    fn test_score_calculation() {
        let mut game = GameLogic::new();
        game.add_score(100);
        game.add_multiplier(2.0);
        
        assert_eq!(game.calculate_final_score(), 200);
    }
}
```

### Integration Testing

Test UdonSharp behavior integration:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use udonsharp::testing::*;
    
    #[udon_integration_test]
    fn test_world_initialization() {
        let mut world = WorldController::new();
        let mock_environment = create_mock_unity_environment();
        
        world.start();
        
        assert!(world.is_initialized());
        assert_eq!(world.get_world_name(), "Test World");
    }
}
```

## Code Organization Patterns

### Component Pattern

Organize functionality into components:

```rust
// Core behavior
#[derive(UdonBehaviour)]
pub struct WorldController {
    audio_system: AudioSystem,
    ui_system: UISystem,
    networking_system: NetworkingSystem,
}

// Separate systems
pub struct AudioSystem {
    background_music: Option<unity::AudioSource>,
    sound_effects: Vec<unity::AudioClip>,
}

impl AudioSystem {
    pub fn play_sound(&mut self, sound_name: &str) {
        if let Some(clip) = self.find_sound_clip(sound_name) {
            // Play sound logic
        }
    }
}
```

### Event-Driven Architecture

Use events for loose coupling:

```rust
pub struct EventSystem {
    listeners: std::collections::HashMap<String, Vec<Box<dyn Fn(&EventData)>>>,
}

impl EventSystem {
    pub fn subscribe<F>(&mut self, event_name: &str, callback: F)
    where
        F: Fn(&EventData) + 'static,
    {
        self.listeners
            .entry(event_name.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(callback));
    }
    
    pub fn emit(&self, event_name: &str, data: &EventData) {
        if let Some(callbacks) = self.listeners.get(event_name) {
            for callback in callbacks {
                callback(data);
            }
        }
    }
}

// Usage
impl WorldController {
    fn setup_events(&mut self) {
        self.event_system.subscribe("player_joined", |data| {
            log::info!("Player joined: {:?}", data);
        });
        
        self.event_system.subscribe("game_started", |data| {
            log::info!("Game started with settings: {:?}", data);
        });
    }
}
```

## Documentation

### Code Documentation

Document your public APIs:

```rust
/// Manages player interactions and state in the VRChat world.
/// 
/// This struct handles player joining/leaving, score tracking,
/// and player-specific game state management.
/// 
/// # Examples
/// 
/// ```rust
/// let mut manager = PlayerManager::new();
/// manager.set_max_players(20);
/// 
/// // Handle player joining
/// manager.on_player_joined(player_api);
/// ```
#[derive(UdonBehaviour)]
pub struct PlayerManager {
    /// Maximum number of players allowed in the world
    #[udon_public]
    pub max_players: i32,
    
    /// Current list of active players
    players: Vec<VRCPlayerApi>,
}

impl PlayerManager {
    /// Creates a new PlayerManager with default settings.
    /// 
    /// # Returns
    /// 
    /// A new `PlayerManager` instance with max_players set to 80.
    pub fn new() -> Self {
        Self {
            max_players: 80,
            players: Vec::new(),
        }
    }
    
    /// Adds a player to the manager.
    /// 
    /// # Arguments
    /// 
    /// * `player` - The VRCPlayerApi instance to add
    /// 
    /// # Returns
    /// 
    /// `Ok(())` if successful, `Err(String)` with error message if failed.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if:
    /// - The player is invalid
    /// - Maximum player count is reached
    /// - The player is already in the list
    pub fn add_player(&mut self, player: VRCPlayerApi) -> Result<(), String> {
        // Implementation...
    }
}
```

## Deployment

### Build Configuration

Use appropriate build configurations:

```toml
# udonsharp.toml
[project]
name = "MyVRChatWorld"
namespace = "MyWorld"

[compilation.dev]
optimize_for_performance = false
generate_debug_info = true
include_debug_comments = true

[compilation.release]
optimize_for_performance = true
generate_debug_info = false
strip_debug_symbols = true
wasm_opt_level = 3
```

### Version Management

Tag your releases and maintain compatibility:

```rust
// src/version.rs
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_DATE: &str = env!("BUILD_DATE");

impl WorldController {
    fn start(&mut self) {
        log::info!("World version: {} (built: {})", VERSION, BUILD_DATE);
    }
}
```

## Common Pitfalls

### Avoid These Patterns

```rust
// ❌ Don't: Expensive operations in update()
impl BadBehavior {
    fn update(&mut self) {
        // This runs every frame!
        let all_objects = unity::GameObject::find_objects_of_type::<unity::GameObject>();
        for obj in all_objects {
            // Expensive operation
        }
    }
}

// ✅ Do: Cache and update periodically
impl GoodBehavior {
    fn update(&mut self) {
        self.frame_count += 1;
        if self.frame_count % 60 == 0 {  // Once per second at 60fps
            self.update_cached_objects();
        }
    }
}

// ❌ Don't: Ignore networking ownership
impl BadNetworking {
    fn update_shared_state(&mut self) {
        self.shared_data = "new value".to_string();  // Anyone can modify!
    }
}

// ✅ Do: Check ownership before modifying
impl GoodNetworking {
    fn update_shared_state(&mut self) {
        if vrchat::Networking::is_owner(&vrchat::Networking::local_player()) {
            self.shared_data = "new value".to_string();
            self.request_serialization();
        }
    }
}
```

## See Also

- [Getting Started](getting-started.md) - Basic setup and first project
- [API Reference](api-reference.md) - Complete API documentation
- [Compilation Pipeline](compilation-pipeline.md) - Understanding the build process
- [Performance Guide](performance.md) - Advanced optimization techniques