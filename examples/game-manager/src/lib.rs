use udonsharp_core::prelude::*;
use udonsharp_bindings::{vrchat, unity};
use std::collections::HashMap;

mod game_state;
mod player_roles;
mod scoring_system;
mod event_system;
mod timer_system;

pub use game_state::*;
pub use player_roles::*;
pub use scoring_system::*;
pub use event_system::*;
pub use timer_system::*;

/// Core game state manager - handles main game flow and state transitions.
/// 
/// This behavior demonstrates:
/// - Sophisticated state machine implementation
/// - Advanced networking and synchronization
/// - Master client responsibilities
/// - Inter-behavior coordination
#[udon_behaviour(name = "GameStateManager")]
pub fn game_state_manager_start() {
    // Entry point for GameStateManager UdonBehaviour
}

#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct GameStateManager {
    // Public configuration
    #[udon_public]
    pub game_duration: f32,
    
    #[udon_public]
    pub min_players: i32,
    
    #[udon_public]
    pub max_players: i32,
    
    #[udon_public]
    pub auto_start_delay: f32,
    
    #[udon_public]
    pub enable_spectators: bool,
    
    // Synchronized state
    #[udon_sync]
    pub current_state: GameState,
    
    #[udon_sync]
    pub game_timer: f32,
    
    #[udon_sync]
    pub round_number: i32,
    
    #[udon_sync]
    pub host_player_id: i32,
    
    // Private systems
    player_roles: PlayerRoleManager,
    scoring_system: ScoringSystem,
    event_system: EventSystem,
    timer_system: TimerSystem,
    
    // State management
    state_enter_time: f32,
    last_state_update: f32,
    pending_state_change: Option<GameState>,
    
    // Performance monitoring
    frame_times: Vec<f32>,
    performance_threshold: f32,
    last_performance_check: f32,
    
    // Error recovery
    last_sync_time: f32,
    sync_timeout: f32,
    recovery_attempts: i32,
    max_recovery_attempts: i32,
    
    initialized: bool,
}

impl GameStateManager {
    pub fn new() -> Self {
        Self {
            game_duration: 300.0, // 5 minutes
            min_players: 2,
            max_players: 20,
            auto_start_delay: 10.0,
            enable_spectators: true,
            current_state: GameState::Lobby,
            game_timer: 0.0,
            round_number: 0,
            host_player_id: 0,
            player_roles: PlayerRoleManager::new(),
            scoring_system: ScoringSystem::new(),
            event_system: EventSystem::new(),
            timer_system: TimerSystem::new(),
            state_enter_time: 0.0,
            last_state_update: 0.0,
            pending_state_change: None,
            frame_times: Vec::with_capacity(60),
            performance_threshold: 0.02, // 20ms frame time threshold
            last_performance_check: 0.0,
            last_sync_time: 0.0,
            sync_timeout: 5.0,
            recovery_attempts: 0,
            max_recovery_attempts: 3,
            initialized: false,
        }
    }
    
    /// Initialize the game manager system
    fn initialize(&mut self) {
        let current_time = unity::Time::time();
        
        // Initialize subsystems
        self.player_roles.initialize();
        self.scoring_system.initialize();
        self.event_system.initialize();
        self.timer_system.initialize();
        
        // Set up event listeners
        self.setup_event_listeners();
        
        // Determine host
        self.determine_host();
        
        // Initialize state
        self.enter_state(GameState::Lobby, current_time);
        
        self.initialized = true;
        
        log::info!("🎮 GameManager initialized - Host: {}, State: {:?}", 
            self.host_player_id, self.current_state);
        
        // Emit initialization event
        self.event_system.emit("game_manager_initialized", &GameManagerInitializedEvent {
            host_id: self.host_player_id,
            initial_state: self.current_state.clone(),
            timestamp: current_time,
        });
    }
    
    /// Set up event system listeners
    fn setup_event_listeners(&mut self) {
        // Player events
        self.event_system.subscribe("player_joined", |event_data| {
            log::info!("📥 Event: Player joined - {:?}", event_data);
        });
        
        self.event_system.subscribe("player_left", |event_data| {
            log::info!("📥 Event: Player left - {:?}", event_data);
        });
        
        // Game state events
        self.event_system.subscribe("game_started", |event_data| {
            log::info!("📥 Event: Game started - {:?}", event_data);
        });
        
        self.event_system.subscribe("game_ended", |event_data| {
            log::info!("📥 Event: Game ended - {:?}", event_data);
        });
        
        // Score events
        self.event_system.subscribe("score_updated", |event_data| {
            log::debug!("📥 Event: Score updated - {:?}", event_data);
        });
    }
    
    /// Determine who should be the host
    fn determine_host(&mut self) {
        let master_player = vrchat::Networking::get_master();
        self.host_player_id = master_player.player_id();
        
        // Assign host role
        self.player_roles.set_player_role(self.host_player_id, PlayerRole::Host);
        
        log::info!("👑 Host determined: {} ({})", 
            master_player.get_display_name(), 
            self.host_player_id
        );
    }
    
    /// Enter a new game state
    fn enter_state(&mut self, new_state: GameState, current_time: f32) {
        let old_state = self.current_state.clone();
        
        // Exit current state
        self.exit_state(&old_state, current_time);
        
        // Update state
        self.current_state = new_state.clone();
        self.state_enter_time = current_time;
        self.last_state_update = current_time;
        
        // Enter new state
        self.enter_state_internal(&new_state, current_time);
        
        // Log transition
        log::info!("🔄 State transition: {:?} → {:?} (at {:.2}s)", 
            old_state, new_state, current_time);
        
        // Emit state change event
        self.event_system.emit("state_changed", &StateChangedEvent {
            from_state: old_state,
            to_state: new_state.clone(),
            timestamp: current_time,
            round_number: self.round_number,
        });
        
        // Request synchronization if we're the host
        if self.is_host() {
            self.request_serialization();
        }
    }
    
    /// Exit current state
    fn exit_state(&mut self, state: &GameState, current_time: f32) {
        match state {
            GameState::Lobby => {
                log::debug!("🚪 Exiting Lobby state");
            }
            GameState::Starting => {
                log::debug!("🚪 Exiting Starting state");
                self.timer_system.stop_countdown();
            }
            GameState::Playing => {
                log::debug!("🚪 Exiting Playing state");
                self.timer_system.pause();
                
                // Emit game paused/ended event
                self.event_system.emit("game_paused", &GamePausedEvent {
                    elapsed_time: current_time - self.state_enter_time,
                    remaining_time: self.game_duration - self.game_timer,
                    timestamp: current_time,
                });
            }
            GameState::Paused => {
                log::debug!("🚪 Exiting Paused state");
            }
            GameState::Ending => {
                log::debug!("🚪 Exiting Ending state");
            }
            GameState::Ended => {
                log::debug!("🚪 Exiting Ended state");
            }
        }
    }
    
    /// Enter new state
    fn enter_state_internal(&mut self, state: &GameState, current_time: f32) {
        match state {
            GameState::Lobby => {
                log::info!("🏠 Entered Lobby state");
                self.game_timer = 0.0;
                self.timer_system.reset();
                
                // Reset player roles (except host)
                self.reset_player_roles_for_lobby();
            }
            GameState::Starting => {
                log::info!("⏳ Entered Starting state");
                self.timer_system.start_countdown(self.auto_start_delay);
                
                // Emit game starting event
                self.event_system.emit("game_starting", &GameStartingEvent {
                    countdown_duration: self.auto_start_delay,
                    player_count: self.get_active_player_count(),
                    timestamp: current_time,
                });
            }
            GameState::Playing => {
                log::info!("🎮 Entered Playing state");
                self.round_number += 1;
                self.timer_system.start_game_timer(self.game_duration);
                
                // Initialize scoring for this round
                self.scoring_system.start_new_round(self.round_number);
                
                // Emit game started event
                self.event_system.emit("game_started", &GameStartedEvent {
                    round_number: self.round_number,
                    duration: self.game_duration,
                    player_count: self.get_active_player_count(),
                    timestamp: current_time,
                });
            }
            GameState::Paused => {
                log::info!("⏸️ Entered Paused state");
                self.timer_system.pause();
            }
            GameState::Ending => {
                log::info!("🏁 Entered Ending state");
                self.timer_system.stop();
                
                // Calculate final scores
                let final_scores = self.scoring_system.calculate_final_scores();
                
                // Emit game ending event
                self.event_system.emit("game_ending", &GameEndingEvent {
                    final_scores: final_scores.clone(),
                    round_number: self.round_number,
                    total_duration: self.game_timer,
                    timestamp: current_time,
                });
            }
            GameState::Ended => {
                log::info!("🎯 Entered Ended state");
                
                // Emit game ended event
                self.event_system.emit("game_ended", &GameEndedEvent {
                    round_number: self.round_number,
                    final_scores: self.scoring_system.get_leaderboard(),
                    timestamp: current_time,
                });
            }
        }
    }
    
    /// Check if current player is the host
    fn is_host(&self) -> bool {
        let local_player = vrchat::Networking::local_player();
        local_player.player_id() == self.host_player_id
    }
    
    /// Get the number of active players
    fn get_active_player_count(&self) -> i32 {
        self.player_roles.get_player_count(PlayerRole::Player)
    }
    
    /// Reset player roles for lobby
    fn reset_player_roles_for_lobby(&mut self) {
        let all_players = vrchat::Networking::get_players();
        for player in all_players {
            if player.player_id() != self.host_player_id {
                self.player_roles.set_player_role(player.player_id(), PlayerRole::Player);
            }
        }
    }
    
    /// Update game logic based on current state
    fn update_game_logic(&mut self, current_time: f32, delta_time: f32) {
        match &self.current_state {
            GameState::Lobby => {
                self.update_lobby_logic(current_time);
            }
            GameState::Starting => {
                self.update_starting_logic(current_time);
            }
            GameState::Playing => {
                self.update_playing_logic(current_time, delta_time);
            }
            GameState::Paused => {
                self.update_paused_logic(current_time);
            }
            GameState::Ending => {
                self.update_ending_logic(current_time);
            }
            GameState::Ended => {
                self.update_ended_logic(current_time);
            }
        }
    }
    
    /// Update lobby logic
    fn update_lobby_logic(&mut self, current_time: f32) {
        if self.is_host() {
            let active_players = self.get_active_player_count();
            
            // Auto-start if we have enough players
            if active_players >= self.min_players {
                let time_in_lobby = current_time - self.state_enter_time;
                if time_in_lobby > 5.0 { // Wait 5 seconds before auto-starting
                    self.pending_state_change = Some(GameState::Starting);
                }
            }
        }
    }
    
    /// Update starting logic
    fn update_starting_logic(&mut self, current_time: f32) {
        if self.timer_system.is_countdown_finished() {
            self.pending_state_change = Some(GameState::Playing);
        }
    }
    
    /// Update playing logic
    fn update_playing_logic(&mut self, current_time: f32, delta_time: f32) {
        // Update game timer
        self.game_timer += delta_time;
        
        // Check if game should end
        if self.game_timer >= self.game_duration {
            self.pending_state_change = Some(GameState::Ending);
        }
        
        // Update scoring system
        self.scoring_system.update(delta_time);
    }
    
    /// Update paused logic
    fn update_paused_logic(&mut self, _current_time: f32) {
        // Game is paused, minimal updates
    }
    
    /// Update ending logic
    fn update_ending_logic(&mut self, current_time: f32) {
        let time_in_ending = current_time - self.state_enter_time;
        
        // Transition to ended after showing results
        if time_in_ending > 5.0 {
            self.pending_state_change = Some(GameState::Ended);
        }
    }
    
    /// Update ended logic
    fn update_ended_logic(&mut self, current_time: f32) {
        let time_in_ended = current_time - self.state_enter_time;
        
        // Return to lobby after showing final results
        if time_in_ended > 10.0 {
            self.pending_state_change = Some(GameState::Lobby);
        }
    }
    
    /// Monitor performance and optimize if needed
    fn monitor_performance(&mut self, delta_time: f32, current_time: f32) {
        // Track frame times
        self.frame_times.push(delta_time);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
        
        // Check performance periodically
        if current_time - self.last_performance_check > 1.0 {
            let average_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            
            if average_frame_time > self.performance_threshold {
                log::warn!("⚠️ Performance issue detected: {:.3}ms average frame time", 
                    average_frame_time * 1000.0);
                self.optimize_for_performance();
            }
            
            self.last_performance_check = current_time;
        }
    }
    
    /// Optimize performance when issues are detected
    fn optimize_for_performance(&mut self) {
        log::info!("🔧 Optimizing performance...");
        
        // Reduce update frequencies
        self.scoring_system.set_update_frequency(0.5); // Update every 500ms instead of every frame
        self.event_system.set_batch_size(10); // Batch more events together
        
        // Clear old data
        self.frame_times.clear();
        
        log::info!("✅ Performance optimization applied");
    }
}

impl UdonBehaviour for GameStateManager {
    fn start(&mut self) {
        log::info!("🚀 GameManager starting...");
        
        // Validate configuration
        if self.game_duration <= 0.0 {
            self.game_duration = 300.0;
            log::warn!("⚠️ Invalid game_duration, using default: 300s");
        }
        
        if self.min_players <= 0 {
            self.min_players = 2;
            log::warn!("⚠️ Invalid min_players, using default: 2");
        }
        
        if self.max_players <= self.min_players {
            self.max_players = self.min_players * 2;
            log::warn!("⚠️ Invalid max_players, using: {}", self.max_players);
        }
        
        self.initialize();
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        let current_time = unity::Time::time();
        let delta_time = unity::Time::delta_time();
        
        // Update subsystems
        self.timer_system.update(delta_time);
        self.scoring_system.update(delta_time);
        self.event_system.update();
        
        // Update game logic
        self.update_game_logic(current_time, delta_time);
        
        // Handle pending state changes
        if let Some(new_state) = self.pending_state_change.take() {
            self.enter_state(new_state, current_time);
        }
        
        // Monitor performance
        self.monitor_performance(delta_time, current_time);
        
        // Check for network issues and recovery
        if current_time - self.last_sync_time > self.sync_timeout {
            self.handle_sync_timeout(current_time);
        }
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        let player_id = player.player_id();
        let player_name = player.get_display_name();
        
        log::info!("👋 Player joined: {} (ID: {})", player_name, player_id);
        
        // Assign default role
        let role = if self.enable_spectators && self.get_active_player_count() >= self.max_players {
            PlayerRole::Spectator
        } else {
            PlayerRole::Player
        };
        
        self.player_roles.set_player_role(player_id, role.clone());
        
        // Emit player joined event
        self.event_system.emit("player_joined", &PlayerJoinedEvent {
            player_id,
            player_name: player_name.clone(),
            assigned_role: role,
            timestamp: unity::Time::time(),
        });
        
        // Update game state if needed
        if self.is_host() && matches!(self.current_state, GameState::Lobby) {
            let active_players = self.get_active_player_count();
            if active_players >= self.min_players {
                log::info!("🎯 Minimum players reached, game can start");
            }
        }
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        let player_id = player.player_id();
        let player_name = player.get_display_name();
        
        log::info!("👋 Player left: {} (ID: {})", player_name, player_id);
        
        // Remove from role system
        self.player_roles.remove_player(player_id);
        
        // Remove from scoring system
        self.scoring_system.remove_player(player_id);
        
        // Emit player left event
        self.event_system.emit("player_left", &PlayerLeftEvent {
            player_id,
            player_name: player_name.clone(),
            timestamp: unity::Time::time(),
        });
        
        // Handle host leaving
        if player_id == self.host_player_id {
            log::warn!("👑 Host left, determining new host...");
            self.determine_host();
        }
        
        // Check if we need to pause/end the game
        if self.is_host() && matches!(self.current_state, GameState::Playing) {
            let active_players = self.get_active_player_count();
            if active_players < self.min_players {
                log::warn!("⚠️ Not enough players, pausing game");
                self.pending_state_change = Some(GameState::Paused);
            }
        }
    }
    
    fn on_post_deserialization(&mut self) {
        self.last_sync_time = unity::Time::time();
        log::debug!("📥 Received sync data - State: {:?}, Timer: {:.1}s", 
            self.current_state, self.game_timer);
    }
}

// Additional helper methods
impl GameStateManager {
    /// Handle sync timeout (potential network issues)
    fn handle_sync_timeout(&mut self, current_time: f32) {
        if self.is_host() {
            // As host, try to re-sync
            self.request_serialization();
            log::warn!("🔄 Host re-syncing due to timeout");
        } else {
            // As client, increment recovery attempts
            self.recovery_attempts += 1;
            
            if self.recovery_attempts >= self.max_recovery_attempts {
                log::error!("❌ Max recovery attempts reached, requesting full sync");
                // In a real implementation, this might trigger a full state recovery
                self.recovery_attempts = 0;
            }
        }
        
        self.last_sync_time = current_time;
    }
    
    /// Public API for starting the game (called by UI buttons, etc.)
    #[udon_event("StartGame")]
    pub fn start_game(&mut self) {
        if self.is_host() && matches!(self.current_state, GameState::Lobby) {
            let active_players = self.get_active_player_count();
            if active_players >= self.min_players {
                self.pending_state_change = Some(GameState::Starting);
                log::info!("🎮 Game start requested by host");
            } else {
                log::warn!("⚠️ Cannot start game: need {} players, have {}", 
                    self.min_players, active_players);
            }
        }
    }
    
    /// Public API for pausing the game
    #[udon_event("PauseGame")]
    pub fn pause_game(&mut self) {
        if self.is_host() && matches!(self.current_state, GameState::Playing) {
            self.pending_state_change = Some(GameState::Paused);
            log::info!("⏸️ Game pause requested by host");
        }
    }
    
    /// Public API for resuming the game
    #[udon_event("ResumeGame")]
    pub fn resume_game(&mut self) {
        if self.is_host() && matches!(self.current_state, GameState::Paused) {
            self.pending_state_change = Some(GameState::Playing);
            log::info!("▶️ Game resume requested by host");
        }
    }
    
    /// Public API for ending the game
    #[udon_event("EndGame")]
    pub fn end_game(&mut self) {
        if self.is_host() && matches!(self.current_state, GameState::Playing | GameState::Paused) {
            self.pending_state_change = Some(GameState::Ending);
            log::info!("🏁 Game end requested by host");
        }
    }
}

impl Default for GameStateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Player role management behavior - handles player assignments and permissions.
#[udon_behaviour(name = "PlayerRoleManager")]
pub fn player_role_manager_start() {
    // Entry point for PlayerRoleManager UdonBehaviour
}

#[derive(UdonBehaviour)]
pub struct PlayerRoleManagerBehaviour {
    role_manager: PlayerRoleManager,
    game_state_ref: Option<unity::GameObject>,
    initialized: bool,
}

impl PlayerRoleManagerBehaviour {
    pub fn new() -> Self {
        Self {
            role_manager: PlayerRoleManager::new(),
            game_state_ref: None,
            initialized: false,
        }
    }
    
    fn find_game_state_manager(&mut self) {
        if let Some(manager_obj) = unity::GameObject::find("GameStateManager") {
            self.game_state_ref = Some(manager_obj);
            log::info!("✅ Found GameStateManager reference");
        }
    }
    
    pub fn assign_player_role(&mut self, player_id: i32, role: PlayerRole) {
        self.role_manager.set_player_role(player_id, role);
        // Notify game state manager of role change
        if let Some(game_state_obj) = &self.game_state_ref {
            // Send custom event to game state manager
            game_state_obj.send_custom_event("OnPlayerRoleChanged");
        }
    }
}

impl UdonBehaviour for PlayerRoleManagerBehaviour {
    fn start(&mut self) {
        log::info!("👥 PlayerRoleManager starting...");
        self.role_manager.initialize();
        self.find_game_state_manager();
        self.initialized = true;
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        let player_id = player.player_id();
        self.role_manager.set_player_role(player_id, PlayerRole::Player);
        log::info!("👋 Assigned Player role to: {}", player.get_display_name());
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        let player_id = player.player_id();
        self.role_manager.remove_player(player_id);
        log::info!("👋 Removed player role for: {}", player.get_display_name());
    }
}

impl Default for PlayerRoleManagerBehaviour {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoring system behavior - handles all score tracking and leaderboards.
#[udon_behaviour(name = "ScoringSystem")]
pub fn scoring_system_start() {
    // Entry point for ScoringSystem UdonBehaviour
}

#[derive(UdonBehaviour)]
pub struct ScoringSystemBehaviour {
    scoring_system: ScoringSystem,
    game_state_ref: Option<unity::GameObject>,
    last_score_update: f32,
    score_update_interval: f32,
    initialized: bool,
}

impl ScoringSystemBehaviour {
    pub fn new() -> Self {
        Self {
            scoring_system: ScoringSystem::new(),
            game_state_ref: None,
            last_score_update: 0.0,
            score_update_interval: 1.0,
            initialized: false,
        }
    }
    
    fn find_game_state_manager(&mut self) {
        if let Some(manager_obj) = unity::GameObject::find("GameStateManager") {
            self.game_state_ref = Some(manager_obj);
        }
    }
    
    pub fn add_score(&mut self, player_id: i32, points: i32) {
        self.scoring_system.add_score(player_id, points);
        log::info!("🎯 Added {} points for player {}", points, player_id);
    }
    
    pub fn get_leaderboard(&self) -> Vec<(i32, i32)> {
        self.scoring_system.get_leaderboard()
    }
}

impl UdonBehaviour for ScoringSystemBehaviour {
    fn start(&mut self) {
        log::info!("🏆 ScoringSystem starting...");
        self.scoring_system.initialize();
        self.find_game_state_manager();
        self.initialized = true;
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        let current_time = unity::Time::time();
        let delta_time = unity::Time::delta_time();
        
        self.scoring_system.update(delta_time);
        
        // Periodic score updates
        if current_time - self.last_score_update >= self.score_update_interval {
            // Update leaderboard display or notify other systems
            self.last_score_update = current_time;
        }
    }
}

impl Default for ScoringSystemBehaviour {
    fn default() -> Self {
        Self::new()
    }
}

/// Event coordination behavior - handles inter-behavior communication.
#[udon_behaviour(name = "EventCoordinator")]
pub fn event_coordinator_start() {
    // Entry point for EventCoordinator UdonBehaviour
}

#[derive(UdonBehaviour)]
pub struct EventCoordinator {
    event_system: EventSystem,
    behavior_references: HashMap<String, unity::GameObject>,
    initialized: bool,
}

impl EventCoordinator {
    pub fn new() -> Self {
        Self {
            event_system: EventSystem::new(),
            behavior_references: HashMap::new(),
            initialized: false,
        }
    }
    
    fn find_all_behaviors(&mut self) {
        // Find all related behavior GameObjects
        if let Some(obj) = unity::GameObject::find("GameStateManager") {
            self.behavior_references.insert("GameStateManager".to_string(), obj);
        }
        if let Some(obj) = unity::GameObject::find("PlayerRoleManager") {
            self.behavior_references.insert("PlayerRoleManager".to_string(), obj);
        }
        if let Some(obj) = unity::GameObject::find("ScoringSystem") {
            self.behavior_references.insert("ScoringSystem".to_string(), obj);
        }
    }
    
    pub fn broadcast_event(&mut self, event_name: &str, target_behaviors: &[&str]) {
        for behavior_name in target_behaviors {
            if let Some(behavior_obj) = self.behavior_references.get(*behavior_name) {
                behavior_obj.send_custom_event(event_name);
            }
        }
    }
}

impl UdonBehaviour for EventCoordinator {
    fn start(&mut self) {
        log::info!("📡 EventCoordinator starting...");
        self.event_system.initialize();
        self.find_all_behaviors();
        self.initialized = true;
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        self.event_system.update();
    }
}

impl Default for EventCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

// Shared utility functions available to all behaviors
pub fn validate_player_count(count: i32, min: i32, max: i32) -> bool {
    count >= min && count <= max
}

pub fn format_game_time(seconds: f32) -> String {
    let minutes = (seconds / 60.0) as i32;
    let secs = (seconds % 60.0) as i32;
    format!("{}:{:02}", minutes, secs)
}