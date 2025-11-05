use udonsharp_core::prelude::*;
use udonsharp_bindings::{vrchat, unity};

/// A simple Hello World UdonSharp behavior that demonstrates basic concepts.
/// 
/// This behavior shows:
/// - Basic UdonSharp trait implementation
/// - Public fields for Unity Inspector
/// - Player event handling
/// - Logging and debug output
#[derive(UdonBehaviour)]
pub struct HelloWorld {
    /// Welcome message displayed when the world starts
    #[udon_public]
    pub welcome_message: String,
    
    /// Maximum number of players to track
    #[udon_public]
    pub max_players: i32,
    
    /// Whether to show detailed logs
    #[udon_public]
    pub verbose_logging: bool,
    
    // Private fields
    player_count: i32,
    world_start_time: f32,
    initialized: bool,
}

impl HelloWorld {
    /// Create a new HelloWorld behavior with default values
    pub fn new() -> Self {
        Self {
            welcome_message: "Welcome to my Rust UdonSharp world!".to_string(),
            max_players: 80,
            verbose_logging: true,
            player_count: 0,
            world_start_time: 0.0,
            initialized: false,
        }
    }
    
    /// Initialize the world with custom settings
    fn initialize_world(&mut self) {
        self.world_start_time = unity::Time::time();
        self.initialized = true;
        
        log::info!("🌍 World initialized at time: {:.2}s", self.world_start_time);
        log::info!("📝 Welcome message: {}", self.welcome_message);
        log::info!("👥 Max players: {}", self.max_players);
        
        // Find and setup UI elements if they exist
        if let Some(welcome_text) = unity::GameObject::find("WelcomeText") {
            if let Some(text_component) = welcome_text.get_component::<unity::UI::Text>() {
                text_component.set_text(&self.welcome_message);
                log::info!("✅ Welcome text UI updated");
            }
        }
        
        // Setup player counter UI
        self.update_player_counter_ui();
    }
    
    /// Update the player counter display
    fn update_player_counter_ui(&self) {
        if let Some(counter_obj) = unity::GameObject::find("PlayerCounter") {
            if let Some(text_component) = counter_obj.get_component::<unity::UI::Text>() {
                let counter_text = format!("Players: {} / {}", self.player_count, self.max_players);
                text_component.set_text(&counter_text);
                
                if self.verbose_logging {
                    log::debug!("🔢 Updated player counter: {}", counter_text);
                }
            }
        }
    }
    
    /// Handle a new player joining
    fn handle_player_joined(&mut self, player: &VRCPlayerApi) {
        let player_name = player.get_display_name();
        let join_time = unity::Time::time() - self.world_start_time;
        
        log::info!("👋 Player joined: {} (at {:.1}s)", player_name, join_time);
        
        if self.verbose_logging {
            log::debug!("🔍 Player details:");
            log::debug!("  - ID: {}", player.player_id());
            log::debug!("  - Is Local: {}", player.is_local());
            log::debug!("  - Is VR: {}", player.is_user_in_vr());
        }
        
        // Check if we're at capacity
        if self.player_count >= self.max_players {
            log::warn!("⚠️ World at maximum capacity! ({} players)", self.max_players);
        }
        
        // Send welcome message to the player (if they're local)
        if player.is_local() {
            log::info!("🎉 {}", self.welcome_message);
        }
    }
    
    /// Handle a player leaving
    fn handle_player_left(&mut self, player: &VRCPlayerApi) {
        let player_name = player.get_display_name();
        let session_time = unity::Time::time() - self.world_start_time;
        
        log::info!("👋 Player left: {} (after {:.1}s)", player_name, session_time);
        
        if self.verbose_logging {
            log::debug!("📊 Session stats for {}:", player_name);
            log::debug!("  - Time in world: {:.1}s", session_time);
            log::debug!("  - Remaining players: {}", self.player_count - 1);
        }
    }
    
    /// Get current world statistics
    pub fn get_world_stats(&self) -> WorldStats {
        WorldStats {
            uptime: unity::Time::time() - self.world_start_time,
            player_count: self.player_count,
            max_players: self.max_players,
            initialized: self.initialized,
        }
    }
}

impl UdonBehaviour for HelloWorld {
    fn start(&mut self) {
        log::info!("🚀 HelloWorld behavior starting...");
        
        // Validate configuration
        if self.welcome_message.is_empty() {
            self.welcome_message = "Welcome to VRChat!".to_string();
            log::warn!("⚠️ Empty welcome message, using default");
        }
        
        if self.max_players <= 0 {
            self.max_players = 80;
            log::warn!("⚠️ Invalid max_players, using default: 80");
        }
        
        // Initialize the world
        self.initialize_world();
        
        // Get initial player count
        let all_players = vrchat::Networking::get_players();
        self.player_count = all_players.len() as i32;
        
        log::info!("✅ HelloWorld initialized with {} players", self.player_count);
        self.update_player_counter_ui();
    }
    
    fn update(&mut self) {
        // Only run updates if initialized
        if !self.initialized {
            return;
        }
        
        // Update player counter UI periodically (every 60 frames ≈ 1 second at 60fps)
        let frame_count = unity::Time::frame_count();
        if frame_count % 60 == 0 {
            self.update_player_counter_ui();
        }
        
        // Log world stats every 5 minutes in verbose mode
        if self.verbose_logging && frame_count % (60 * 60 * 5) == 0 {
            let stats = self.get_world_stats();
            log::info!("📊 World Stats - Uptime: {:.1}s, Players: {}/{}", 
                stats.uptime, stats.player_count, stats.max_players);
        }
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.player_count += 1;
        self.handle_player_joined(&player);
        self.update_player_counter_ui();
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.player_count -= 1;
        self.handle_player_left(&player);
        self.update_player_counter_ui();
    }
}

/// Statistics about the current world state
#[derive(Debug, Clone)]
pub struct WorldStats {
    pub uptime: f32,
    pub player_count: i32,
    pub max_players: i32,
    pub initialized: bool,
}

impl Default for HelloWorld {
    fn default() -> Self {
        Self::new()
    }
}

// Export the main behavior for UdonSharp compilation
pub use HelloWorld as MainBehaviour;