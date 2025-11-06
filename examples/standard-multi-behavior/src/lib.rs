//! Standard Multi-Behavior Pattern Example
//! 
//! This example demonstrates a complete game system using multiple UdonBehaviour structs
//! that work together to create a simple multiplayer game in VRChat.
//! 
//! The example includes:
//! - GameManager: Manages overall game state and rounds
//! - PlayerTracker: Tracks player statistics and positions  
//! - UIController: Manages the user interface
//! - ScoreSystem: Handles scoring and leaderboards
//! - SharedRuntime: Automatically generated shared utilities

use udonsharp_core::prelude::*;

/// Manages the overall game state, rounds, and timing
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct GameManager {
    /// Maximum number of players allowed in the game
    #[udon_public]
    #[header("Game Configuration")]
    #[tooltip("Maximum number of players that can participate")]
    max_players: i32,
    
    /// Duration of each game round in seconds
    #[udon_public]
    #[tooltip("How long each round lasts in seconds")]
    round_duration: f32,
    
    /// Current game state synchronized across all clients
    #[udon_sync]
    current_game_state: i32, // 0 = Lobby, 1 = Playing, 2 = Finished
    
    /// Current round number
    #[udon_sync]
    current_round: i32,
    
    /// Time remaining in current round
    #[udon_sync]
    time_remaining: f32,
    
    /// Reference to the UI controller for updates
    ui_controller: Option<unity::GameObject>,
    
    /// Reference to the player tracker
    player_tracker: Option<unity::GameObject>,
    
    /// Reference to the score system
    score_system: Option<unity::GameObject>,
    
    /// Internal timer for round management
    round_timer: f32,
}

impl UdonBehaviour for GameManager {
    fn start(&mut self) {
        // Initialize game settings
        self.max_players = 20;
        self.round_duration = 300.0; // 5 minutes
        self.current_game_state = 0; // Start in lobby
        self.current_round = 0;
        self.time_remaining = 0.0;
        self.round_timer = 0.0;
        
        // Find other behavior components
        self.ui_controller = unity::GameObject::find("UIController");
        self.player_tracker = unity::GameObject::find("PlayerTracker");
        self.score_system = unity::GameObject::find("ScoreSystem");
        
        // Initialize UI
        self.update_ui();
        
        log_info("GameManager initialized");
    }
    
    fn update(&mut self) {
        if self.current_game_state == 1 && networking::is_master() {
            // Update round timer
            self.round_timer += unity::time::delta_time();
            self.time_remaining = self.round_duration - self.round_timer;
            
            // Check if round should end
            if self.time_remaining <= 0.0 {
                self.end_round();
            }
            
            // Update UI every second
            if (self.round_timer * 10.0) as i32 != ((self.round_timer - unity::time::delta_time()) * 10.0) as i32 {
                networking::request_serialization();
            }
        }
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        log_info(&format!("Player joined: {}", player.display_name()));
        
        // Notify other systems
        if let Some(tracker) = &self.player_tracker {
            tracker.send_custom_event("OnPlayerJoined");
        }
        
        self.update_ui();
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        log_info(&format!("Player left: {}", player.display_name()));
        
        // Notify other systems
        if let Some(tracker) = &self.player_tracker {
            tracker.send_custom_event("OnPlayerLeft");
        }
        
        self.update_ui();
    }
    
    fn on_post_deserialization(&mut self) {
        // Update UI when synchronized data changes
        self.update_ui();
    }
}

impl GameManager {
    /// Start a new game round (master client only)
    pub fn start_round(&mut self) {
        if !networking::is_master() {
            return;
        }
        
        self.current_round += 1;
        self.current_game_state = 1; // Playing
        self.time_remaining = self.round_duration;
        self.round_timer = 0.0;
        
        // Notify other systems
        if let Some(tracker) = &self.player_tracker {
            tracker.send_custom_event("OnRoundStart");
        }
        
        if let Some(scores) = &self.score_system {
            scores.send_custom_event("OnRoundStart");
        }
        
        networking::request_serialization();
        self.update_ui();
        
        log_info(&format!("Round {} started", self.current_round));
    }
    
    /// End the current round (master client only)
    fn end_round(&mut self) {
        if !networking::is_master() {
            return;
        }
        
        self.current_game_state = 2; // Finished
        self.time_remaining = 0.0;
        
        // Notify other systems
        if let Some(scores) = &self.score_system {
            scores.send_custom_event("OnRoundEnd");
        }
        
        networking::request_serialization();
        self.update_ui();
        
        log_info(&format!("Round {} ended", self.current_round));
    }
    
    /// Return to lobby (master client only)
    #[udon_event("ReturnToLobby")]
    pub fn return_to_lobby(&mut self) {
        if !networking::is_master() {
            return;
        }
        
        self.current_game_state = 0; // Lobby
        self.time_remaining = 0.0;
        self.round_timer = 0.0;
        
        networking::request_serialization();
        self.update_ui();
        
        log_info("Returned to lobby");
    }
    
    /// Update the UI with current game state
    fn update_ui(&self) {
        if let Some(ui) = &self.ui_controller {
            ui.send_custom_event("UpdateGameState");
        }
    }
    
    /// Get current game state as string
    pub fn get_game_state_string(&self) -> String {
        match self.current_game_state {
            0 => "Lobby".to_string(),
            1 => "Playing".to_string(),
            2 => "Finished".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

/// Tracks player statistics and positions
#[derive(UdonBehaviour)]
pub struct PlayerTracker {
    /// Maximum number of players to track
    #[udon_public]
    #[header("Tracking Configuration")]
    max_tracked_players: i32,
    
    /// How often to update player positions (in seconds)
    #[udon_public]
    update_interval: f32,
    
    /// Current number of active players
    #[udon_sync]
    active_player_count: i32,
    
    /// Reference to game manager
    game_manager: Option<unity::GameObject>,
    
    /// Reference to score system
    score_system: Option<unity::GameObject>,
    
    /// Internal update timer
    update_timer: f32,
    
    /// Player positions (simplified for example)
    player_positions: Vec<Vector3>,
}

impl UdonBehaviour for PlayerTracker {
    fn start(&mut self) {
        self.max_tracked_players = 20;
        self.update_interval = 1.0; // Update every second
        self.active_player_count = 0;
        self.update_timer = 0.0;
        self.player_positions = Vec::new();
        
        // Find other components
        self.game_manager = unity::GameObject::find("GameManager");
        self.score_system = unity::GameObject::find("ScoreSystem");
        
        log_info("PlayerTracker initialized");
    }
    
    fn update(&mut self) {
        self.update_timer += unity::time::delta_time();
        
        if self.update_timer >= self.update_interval {
            self.update_player_tracking();
            self.update_timer = 0.0;
        }
    }
}

impl PlayerTracker {
    /// Update player tracking information
    fn update_player_tracking(&mut self) {
        // Get current player count
        let current_count = networking::get_player_count();
        
        if current_count != self.active_player_count && networking::is_master() {
            self.active_player_count = current_count;
            networking::request_serialization();
        }
        
        // Update player positions (simplified)
        self.player_positions.clear();
        let local_player = networking::get_local_player();
        if let Some(player) = local_player {
            let position = player.get_position();
            self.player_positions.push(position);
        }
    }
    
    /// Handle player joined event
    #[udon_event("OnPlayerJoined")]
    pub fn handle_player_joined(&mut self) {
        log_info("PlayerTracker: Player joined");
        self.update_player_tracking();
    }
    
    /// Handle player left event
    #[udon_event("OnPlayerLeft")]
    pub fn handle_player_left(&mut self) {
        log_info("PlayerTracker: Player left");
        self.update_player_tracking();
    }
    
    /// Handle round start event
    #[udon_event("OnRoundStart")]
    pub fn handle_round_start(&mut self) {
        log_info("PlayerTracker: Round started");
        // Reset tracking data for new round
        self.player_positions.clear();
    }
    
    /// Get the current active player count
    pub fn get_active_player_count(&self) -> i32 {
        self.active_player_count
    }
    
    /// Check if a position is within the play area
    pub fn is_position_in_play_area(&self, position: Vector3) -> bool {
        // Simple bounds check (customize for your world)
        let bounds_size = 50.0;
        position.x.abs() <= bounds_size && 
        position.z.abs() <= bounds_size && 
        position.y >= -10.0 && position.y <= 100.0
    }
}

/// Manages the user interface and displays
#[derive(UdonBehaviour)]
pub struct UIController {
    /// Main game state text display
    #[udon_public]
    #[header("UI References")]
    #[tooltip("Text component showing current game state")]
    game_state_text: Option<unity::ui::Text>,
    
    /// Player count display
    #[udon_public]
    #[tooltip("Text component showing player count")]
    player_count_text: Option<unity::ui::Text>,
    
    /// Round timer display
    #[udon_public]
    #[tooltip("Text component showing time remaining")]
    timer_text: Option<unity::ui::Text>,
    
    /// Score display
    #[udon_public]
    #[tooltip("Text component showing current scores")]
    score_text: Option<unity::ui::Text>,
    
    /// Start game button
    #[udon_public]
    #[tooltip("Button to start a new game round")]
    start_button: Option<unity::ui::Button>,
    
    /// Reference to game manager
    game_manager: Option<unity::GameObject>,
    
    /// Reference to player tracker
    player_tracker: Option<unity::GameObject>,
    
    /// Reference to score system
    score_system: Option<unity::GameObject>,
}

impl UdonBehaviour for UIController {
    fn start(&mut self) {
        // Find other components
        self.game_manager = unity::GameObject::find("GameManager");
        self.player_tracker = unity::GameObject::find("PlayerTracker");
        self.score_system = unity::GameObject::find("ScoreSystem");
        
        // Set up button listeners (simplified)
        if let Some(button) = &self.start_button {
            // In real implementation, this would set up the button click handler
            button.set_interactable(networking::is_master());
        }
        
        // Initial UI update
        self.update_all_ui();
        
        log_info("UIController initialized");
    }
}

impl UIController {
    /// Update all UI elements
    #[udon_event("UpdateGameState")]
    pub fn update_all_ui(&mut self) {
        self.update_game_state_display();
        self.update_player_count_display();
        self.update_timer_display();
        self.update_score_display();
        self.update_button_states();
    }
    
    /// Update game state text
    fn update_game_state_display(&self) {
        if let Some(text) = &self.game_state_text {
            if let Some(gm_obj) = &self.game_manager {
                if let Some(gm) = gm_obj.get_component::<GameManager>() {
                    let state_text = format!("Game State: {} (Round {})", 
                        gm.get_game_state_string(), 
                        gm.current_round
                    );
                    text.set_text(&state_text);
                }
            }
        }
    }
    
    /// Update player count display
    fn update_player_count_display(&self) {
        if let Some(text) = &self.player_count_text {
            if let Some(tracker_obj) = &self.player_tracker {
                if let Some(tracker) = tracker_obj.get_component::<PlayerTracker>() {
                    let count_text = format!("Players: {}", 
                        tracker.get_active_player_count()
                    );
                    text.set_text(&count_text);
                }
            }
        }
    }
    
    /// Update timer display
    fn update_timer_display(&self) {
        if let Some(text) = &self.timer_text {
            if let Some(gm_obj) = &self.game_manager {
                if let Some(gm) = gm_obj.get_component::<GameManager>() {
                    if gm.current_game_state == 1 {
                        let time_text = format_time(gm.time_remaining);
                        text.set_text(&format!("Time: {}", time_text));
                    } else {
                        text.set_text("Time: --:--");
                    }
                }
            }
        }
    }
    
    /// Update score display
    fn update_score_display(&self) {
        if let Some(text) = &self.score_text {
            if let Some(score_obj) = &self.score_system {
                if let Some(scores) = score_obj.get_component::<ScoreSystem>() {
                    let score_text = format!("High Score: {}", 
                        format_score(scores.get_high_score())
                    );
                    text.set_text(&score_text);
                }
            }
        }
    }
    
    /// Update button states based on game state and permissions
    fn update_button_states(&self) {
        if let Some(button) = &self.start_button {
            let can_start = networking::is_master();
            let should_show = if let Some(gm_obj) = &self.game_manager {
                if let Some(gm) = gm_obj.get_component::<GameManager>() {
                    gm.current_game_state == 0 // Only show in lobby
                } else {
                    false
                }
            } else {
                false
            };
            
            button.set_interactable(can_start && should_show);
        }
    }
    
    /// Handle start button click
    #[udon_event("OnStartButtonClick")]
    pub fn handle_start_button_click(&self) {
        if networking::is_master() {
            if let Some(gm_obj) = &self.game_manager {
                gm_obj.send_custom_event("StartRound");
            }
        }
    }
}

/// Manages scoring and leaderboards
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct ScoreSystem {
    /// Points awarded for basic actions
    #[udon_public]
    #[header("Scoring Configuration")]
    #[tooltip("Points awarded for basic actions")]
    base_points: i32,
    
    /// Multiplier for bonus actions
    #[udon_public]
    #[tooltip("Multiplier for bonus actions")]
    bonus_multiplier: f32,
    
    /// Current high score across all players
    #[udon_sync]
    high_score: i32,
    
    /// Current round high score
    #[udon_sync]
    round_high_score: i32,
    
    /// Total games played
    #[udon_sync]
    total_games_played: i32,
    
    /// Reference to UI controller for updates
    ui_controller: Option<unity::GameObject>,
    
    /// Local player score (not synchronized)
    local_player_score: i32,
}

impl UdonBehaviour for ScoreSystem {
    fn start(&mut self) {
        self.base_points = 100;
        self.bonus_multiplier = 1.5;
        self.high_score = 0;
        self.round_high_score = 0;
        self.total_games_played = 0;
        self.local_player_score = 0;
        
        // Find UI controller
        self.ui_controller = unity::GameObject::find("UIController");
        
        log_info("ScoreSystem initialized");
    }
    
    fn on_post_deserialization(&mut self) {
        // Update UI when scores change
        self.update_ui();
    }
}

impl ScoreSystem {
    /// Add points to local player score
    pub fn add_points(&mut self, points: i32) {
        self.local_player_score += points;
        
        // Check for new high score
        if self.local_player_score > self.round_high_score && networking::is_master() {
            self.round_high_score = self.local_player_score;
            
            if self.round_high_score > self.high_score {
                self.high_score = self.round_high_score;
            }
            
            networking::request_serialization();
        }
        
        self.update_ui();
        
        log_info(&format!("Points added: {}. Total: {}", points, self.local_player_score));
    }
    
    /// Award bonus points with multiplier
    pub fn add_bonus_points(&mut self, base_points: i32) {
        let bonus_points = (base_points as f32 * self.bonus_multiplier) as i32;
        self.add_points(bonus_points);
    }
    
    /// Handle round start event
    #[udon_event("OnRoundStart")]
    pub fn handle_round_start(&mut self) {
        // Reset round scores
        self.local_player_score = 0;
        
        if networking::is_master() {
            self.round_high_score = 0;
            networking::request_serialization();
        }
        
        self.update_ui();
        log_info("ScoreSystem: Round started, scores reset");
    }
    
    /// Handle round end event
    #[udon_event("OnRoundEnd")]
    pub fn handle_round_end(&mut self) {
        if networking::is_master() {
            self.total_games_played += 1;
            networking::request_serialization();
        }
        
        self.update_ui();
        log_info(&format!("ScoreSystem: Round ended. Final score: {}", self.local_player_score));
    }
    
    /// Get the current high score
    pub fn get_high_score(&self) -> i32 {
        self.high_score
    }
    
    /// Get the current round high score
    pub fn get_round_high_score(&self) -> i32 {
        self.round_high_score
    }
    
    /// Get local player score
    pub fn get_local_score(&self) -> i32 {
        self.local_player_score
    }
    
    /// Update UI display
    fn update_ui(&self) {
        if let Some(ui) = &self.ui_controller {
            ui.send_custom_event("UpdateGameState");
        }
    }
}

// Shared utility functions (automatically moved to SharedRuntime)

/// Format time in seconds to MM:SS format
pub fn format_time(seconds: f32) -> String {
    let total_seconds = seconds.max(0.0) as i32;
    let minutes = total_seconds / 60;
    let secs = total_seconds % 60;
    format!("{}:{:02}", minutes, secs)
}

/// Format score with appropriate suffixes (K, M, etc.)
pub fn format_score(score: i32) -> String {
    if score >= 1_000_000 {
        format!("{:.1}M", score as f32 / 1_000_000.0)
    } else if score >= 1_000 {
        format!("{:.1}K", score as f32 / 1_000.0)
    } else {
        score.to_string()
    }
}

/// Calculate distance between two points
pub fn calculate_distance(pos1: Vector3, pos2: Vector3) -> f32 {
    let diff = pos1 - pos2;
    (diff.x * diff.x + diff.y * diff.y + diff.z * diff.z).sqrt()
}

/// Clamp a value between min and max
pub fn clamp_value(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Log an info message (wrapper for UdonSharp logging)
pub fn log_info(message: &str) {
    // In real implementation, this would use UdonSharp's Debug.Log
    println!("[INFO] {}", message);
}

/// Log a warning message
pub fn log_warning(message: &str) {
    // In real implementation, this would use UdonSharp's Debug.LogWarning
    println!("[WARNING] {}", message);
}

/// Log an error message
pub fn log_error(message: &str) {
    // In real implementation, this would use UdonSharp's Debug.LogError
    println!("[ERROR] {}", message);
}