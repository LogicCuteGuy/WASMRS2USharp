//! Multi-Behavior Demo
//! 
//! This example demonstrates how to structure Rust code for multi-behavior
//! UdonSharp compilation. It shows:
//! - Multiple UdonBehaviour classes from a single Rust project
//! - Inter-behavior communication
//! - Shared functionality
//! - Proper dependency management

use udonsharp_core::prelude::*;

// ============================================================================
// PLAYER MANAGEMENT BEHAVIOR
// ============================================================================

/// Player management behavior handles player join/leave events and tracking
#[udon_behaviour(name = "PlayerManager", events = ["Start", "OnPlayerJoined", "OnPlayerLeft"])]
pub fn player_manager_start() {
    log_info("PlayerManager: Initializing player tracking system");
    initialize_player_data();
    setup_ui_references();
}

#[udon_behaviour_event(behavior = "PlayerManager")]
pub fn on_player_joined(player: VRCPlayerApi) {
    log_info(&format!("Player joined: {}", player.displayName));
    
    // Update player count
    let current_count = get_player_count();
    set_player_count(current_count + 1);
    
    // Notify UI to update
    notify_ui_player_count_changed();
    
    // Notify game logic about new player
    notify_game_logic_player_joined(player);
}

#[udon_behaviour_event(behavior = "PlayerManager")]
pub fn on_player_left(player: VRCPlayerApi) {
    log_info(&format!("Player left: {}", player.displayName));
    
    // Update player count
    let current_count = get_player_count();
    set_player_count(current_count - 1);
    
    // Notify UI to update
    notify_ui_player_count_changed();
    
    // Notify game logic about player leaving
    notify_game_logic_player_left(player);
}

// ============================================================================
// UI CONTROLLER BEHAVIOR
// ============================================================================

/// UI controller manages all user interface elements and updates
#[udon_behaviour(name = "UIController", events = ["Start", "Update"])]
pub fn ui_controller_start() {
    log_info("UIController: Initializing UI system");
    setup_ui_elements();
    update_initial_display();
}

#[udon_behaviour_event(behavior = "UIController")]
pub fn ui_controller_update() {
    // Update UI elements that need continuous updates
    update_timer_display();
    update_performance_metrics();
}

/// Handle player count updates from PlayerManager
#[udon_behaviour_method(behavior = "UIController")]
pub fn on_player_count_changed() {
    let count = get_player_count();
    update_player_count_display(count);
    
    // Update game status based on player count
    if count >= get_min_players_for_game() {
        show_game_ready_indicator();
    } else {
        show_waiting_for_players_indicator();
    }
}

/// Handle game state updates from GameLogic
#[udon_behaviour_method(behavior = "UIController")]
pub fn on_game_state_changed(new_state: GameState) {
    update_game_state_display(new_state);
    
    match new_state {
        GameState::Waiting => show_waiting_screen(),
        GameState::Starting => show_countdown_screen(),
        GameState::Playing => show_game_screen(),
        GameState::Finished => show_results_screen(),
    }
}

// ============================================================================
// GAME LOGIC BEHAVIOR
// ============================================================================

/// Game logic behavior handles the core game mechanics and state management
#[udon_behaviour(name = "GameLogic", events = ["Start", "FixedUpdate"])]
pub fn game_logic_start() {
    log_info("GameLogic: Initializing game systems");
    initialize_game_state();
    setup_game_parameters();
}

#[udon_behaviour_event(behavior = "GameLogic")]
pub fn game_logic_fixed_update() {
    match get_current_game_state() {
        GameState::Starting => update_countdown(),
        GameState::Playing => update_game_mechanics(),
        _ => {} // No updates needed for other states
    }
}

/// Handle player joined notification from PlayerManager
#[udon_behaviour_method(behavior = "GameLogic")]
pub fn on_player_joined_notification(player: VRCPlayerApi) {
    add_player_to_game(player);
    
    // Check if we can start the game
    if get_player_count() >= get_min_players_for_game() && get_current_game_state() == GameState::Waiting {
        start_game_countdown();
    }
}

/// Handle player left notification from PlayerManager
#[udon_behaviour_method(behavior = "GameLogic")]
pub fn on_player_left_notification(player: VRCPlayerApi) {
    remove_player_from_game(player);
    
    // Check if we need to pause/stop the game
    if get_player_count() < get_min_players_for_game() && get_current_game_state() == GameState::Playing {
        pause_game_insufficient_players();
    }
}

// ============================================================================
// NETWORKING BEHAVIOR
// ============================================================================

/// Networking behavior handles synchronization and network events
#[udon_behaviour(name = "NetworkManager", events = ["Start", "OnDeserialization"])]
pub fn network_manager_start() {
    log_info("NetworkManager: Initializing networking systems");
    setup_network_variables();
    register_network_callbacks();
}

#[udon_behaviour_event(behavior = "NetworkManager")]
pub fn on_deserialization() {
    // Handle incoming network data
    process_network_updates();
    
    // Notify other behaviors of network changes
    notify_behaviors_of_network_update();
}

/// Synchronize game state across all clients
#[udon_behaviour_method(behavior = "NetworkManager")]
pub fn sync_game_state(state: GameState) {
    if is_master_client() {
        set_networked_game_state(state);
        request_serialization();
    }
}

/// Synchronize player data across all clients
#[udon_behaviour_method(behavior = "NetworkManager")]
pub fn sync_player_data() {
    if is_master_client() {
        update_networked_player_data();
        request_serialization();
    }
}

// ============================================================================
// SHARED FUNCTIONALITY
// ============================================================================
// These functions will be moved to SharedRuntime by the compiler

/// Shared player data management
static mut PLAYER_COUNT: i32 = 0;
static mut MIN_PLAYERS_FOR_GAME: i32 = 2;

pub fn get_player_count() -> i32 {
    unsafe { PLAYER_COUNT }
}

pub fn set_player_count(count: i32) {
    unsafe { PLAYER_COUNT = count; }
}

pub fn get_min_players_for_game() -> i32 {
    unsafe { MIN_PLAYERS_FOR_GAME }
}

/// Game state management
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Waiting,
    Starting,
    Playing,
    Finished,
}

static mut CURRENT_GAME_STATE: GameState = GameState::Waiting;

pub fn get_current_game_state() -> GameState {
    unsafe { CURRENT_GAME_STATE }
}

pub fn set_current_game_state(state: GameState) {
    unsafe { CURRENT_GAME_STATE = state; }
}

/// Logging utilities
pub fn log_info(message: &str) {
    // In actual implementation, this would use UdonSharp's Debug.Log
    println!("[INFO] {}", message);
}

pub fn log_warning(message: &str) {
    println!("[WARNING] {}", message);
}

pub fn log_error(message: &str) {
    println!("[ERROR] {}", message);
}

/// Inter-behavior communication helpers
pub fn notify_ui_player_count_changed() {
    // This will be converted to SendCustomEvent("OnPlayerCountChanged") by the compiler
    send_custom_event("UIController", "OnPlayerCountChanged");
}

pub fn notify_game_logic_player_joined(player: VRCPlayerApi) {
    // Pass player data through custom event
    set_event_parameter("player", player);
    send_custom_event("GameLogic", "OnPlayerJoinedNotification");
}

pub fn notify_game_logic_player_left(player: VRCPlayerApi) {
    set_event_parameter("player", player);
    send_custom_event("GameLogic", "OnPlayerLeftNotification");
}

pub fn notify_ui_game_state_changed(state: GameState) {
    set_event_parameter("gameState", state);
    send_custom_event("UIController", "OnGameStateChanged");
}

pub fn notify_network_sync_game_state(state: GameState) {
    set_event_parameter("gameState", state);
    send_custom_event("NetworkManager", "SyncGameState");
}

/// Placeholder implementations for UdonSharp functionality
/// These would be replaced by actual UdonSharp bindings in real implementation

pub fn send_custom_event(behavior: &str, event: &str) {
    log_info(&format!("SendCustomEvent: {} -> {}", behavior, event));
}

pub fn set_event_parameter<T>(name: &str, value: T) {
    log_info(&format!("SetEventParameter: {} = {:?}", name, std::any::type_name::<T>()));
}

pub fn is_master_client() -> bool {
    true // Placeholder
}

pub fn request_serialization() {
    log_info("RequestSerialization called");
}

// ============================================================================
// BEHAVIOR-SPECIFIC IMPLEMENTATIONS
// ============================================================================

// PlayerManager specific functions
fn initialize_player_data() {
    set_player_count(1); // Start with local player
    log_info("Player data initialized");
}

fn setup_ui_references() {
    log_info("UI references set up for PlayerManager");
}

// UIController specific functions
fn setup_ui_elements() {
    log_info("UI elements initialized");
}

fn update_initial_display() {
    update_player_count_display(get_player_count());
    update_game_state_display(get_current_game_state());
}

fn update_timer_display() {
    // Update game timer UI
}

fn update_performance_metrics() {
    // Update FPS, ping, etc.
}

fn update_player_count_display(count: i32) {
    log_info(&format!("Updating player count display: {}", count));
}

fn show_game_ready_indicator() {
    log_info("Showing game ready indicator");
}

fn show_waiting_for_players_indicator() {
    log_info("Showing waiting for players indicator");
}

fn update_game_state_display(state: GameState) {
    log_info(&format!("Updating game state display: {:?}", state));
}

fn show_waiting_screen() {
    log_info("Showing waiting screen");
}

fn show_countdown_screen() {
    log_info("Showing countdown screen");
}

fn show_game_screen() {
    log_info("Showing game screen");
}

fn show_results_screen() {
    log_info("Showing results screen");
}

// GameLogic specific functions
fn initialize_game_state() {
    set_current_game_state(GameState::Waiting);
    log_info("Game state initialized");
}

fn setup_game_parameters() {
    log_info("Game parameters configured");
}

fn update_countdown() {
    // Handle countdown logic
    static mut COUNTDOWN_TIME: f32 = 5.0;
    
    unsafe {
        COUNTDOWN_TIME -= get_delta_time();
        if COUNTDOWN_TIME <= 0.0 {
            start_game();
        }
    }
}

fn update_game_mechanics() {
    // Handle core game logic updates
}

fn add_player_to_game(player: VRCPlayerApi) {
    log_info(&format!("Adding player to game: {}", player.displayName));
}

fn remove_player_from_game(player: VRCPlayerApi) {
    log_info(&format!("Removing player from game: {}", player.displayName));
}

fn start_game_countdown() {
    set_current_game_state(GameState::Starting);
    notify_ui_game_state_changed(GameState::Starting);
    notify_network_sync_game_state(GameState::Starting);
    log_info("Starting game countdown");
}

fn start_game() {
    set_current_game_state(GameState::Playing);
    notify_ui_game_state_changed(GameState::Playing);
    notify_network_sync_game_state(GameState::Playing);
    log_info("Game started!");
}

fn pause_game_insufficient_players() {
    set_current_game_state(GameState::Waiting);
    notify_ui_game_state_changed(GameState::Waiting);
    notify_network_sync_game_state(GameState::Waiting);
    log_info("Game paused: insufficient players");
}

// NetworkManager specific functions
fn setup_network_variables() {
    log_info("Network variables initialized");
}

fn register_network_callbacks() {
    log_info("Network callbacks registered");
}

fn process_network_updates() {
    log_info("Processing network updates");
}

fn notify_behaviors_of_network_update() {
    log_info("Notifying behaviors of network update");
}

fn set_networked_game_state(state: GameState) {
    log_info(&format!("Setting networked game state: {:?}", state));
}

fn update_networked_player_data() {
    log_info("Updating networked player data");
}

// Utility functions
fn get_delta_time() -> f32 {
    0.016 // Placeholder: 60 FPS
}

// Placeholder VRCPlayerApi for compilation
#[derive(Debug, Clone)]
pub struct VRCPlayerApi {
    pub displayName: String,
}

impl VRCPlayerApi {
    pub fn new(name: &str) -> Self {
        Self {
            displayName: name.to_string(),
        }
    }
}