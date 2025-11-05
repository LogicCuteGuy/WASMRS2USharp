//! UdonSharp testing framework
//! 
//! This module provides testing utilities for UdonSharp development, including
//! mock VRChat and Unity environments, assertion systems, and test runners.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Once};
use crate::types::*;
// use crate::error::UdonSharpError;

static INIT: Once = Once::new();
static mut TEST_ENVIRONMENT: Option<Arc<Mutex<MockEnvironment>>> = None;

/// UdonSharp test environment that provides mock VRChat and Unity systems
pub struct UdonTestEnvironment {
    _private: (),
}

impl UdonTestEnvironment {
    /// Create a new test environment
    pub fn new() -> Self {
        setup_mock_environment();
        Self { _private: () }
    }
}

impl Drop for UdonTestEnvironment {
    fn drop(&mut self) {
        cleanup_mock_environment();
    }
}

/// Mock environment that simulates VRChat and Unity systems
#[derive(Debug)]
pub struct MockEnvironment {
    pub players: Vec<MockVRCPlayerApi>,
    pub game_objects: HashMap<String, MockGameObject>,
    pub networking_state: MockNetworkingState,
    pub time_state: MockTimeState,
    pub physics_state: MockPhysicsState,
    pub audio_state: MockAudioState,
}

impl MockEnvironment {
    pub fn new() -> Self {
        Self {
            players: vec![MockVRCPlayerApi::local_player()],
            game_objects: HashMap::new(),
            networking_state: MockNetworkingState::new(),
            time_state: MockTimeState::new(),
            physics_state: MockPhysicsState::new(),
            audio_state: MockAudioState::new(),
        }
    }
    
    pub fn add_player(&mut self, display_name: &str) -> MockVRCPlayerApi {
        let player = MockVRCPlayerApi::new(self.players.len() as u32, display_name);
        self.players.push(player.clone());
        player
    }
    
    pub fn create_game_object(&mut self, name: &str) -> MockGameObject {
        let game_object = MockGameObject::new(name);
        self.game_objects.insert(name.to_string(), game_object.clone());
        game_object
    }
    
    pub fn find_game_object(&self, name: &str) -> Option<&MockGameObject> {
        self.game_objects.get(name)
    }
}

/// Mock VRCPlayerApi for testing
#[derive(Debug, Clone)]
pub struct MockVRCPlayerApi {
    pub player_id: u32,
    pub display_name: String,
    pub is_local: bool,
    pub is_master: bool,
    pub position: Vector3,
    pub rotation: Quaternion,
    pub is_in_vr: bool,
}

impl MockVRCPlayerApi {
    pub fn new(player_id: u32, display_name: &str) -> Self {
        Self {
            player_id,
            display_name: display_name.to_string(),
            is_local: false,
            is_master: player_id == 0,
            position: Vector3::zero(),
            rotation: Quaternion::identity(),
            is_in_vr: false,
        }
    }
    
    pub fn local_player() -> Self {
        let mut player = Self::new(0, "LocalPlayer");
        player.is_local = true;
        player.is_master = true;
        player
    }
    
    pub fn get_display_name(&self) -> String {
        self.display_name.clone()
    }
    
    pub fn is_local_player(&self) -> bool {
        self.is_local
    }
    
    pub fn is_master_player(&self) -> bool {
        self.is_master
    }
    
    pub fn get_position(&self) -> Vector3 {
        self.position
    }
    
    pub fn set_position(&mut self, position: Vector3) {
        self.position = position;
    }
    
    pub fn teleport_to(&mut self, position: Vector3, rotation: Quaternion) {
        self.position = position;
        self.rotation = rotation;
    }
    
    pub fn respawn(&mut self) {
        self.position = Vector3::zero();
        self.rotation = Quaternion::identity();
    }
}

/// Mock GameObject for testing
#[derive(Debug)]
pub struct MockGameObject {
    pub name: String,
    pub active: bool,
    pub transform: MockTransform,
    pub components: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl Clone for MockGameObject {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            active: self.active,
            transform: self.transform.clone(),
            components: HashMap::new(), // Skip cloning complex components
        }
    }
}

impl MockGameObject {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            active: true,
            transform: MockTransform::new(),
            components: HashMap::new(),
        }
    }
    
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
    
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    pub fn get_transform(&self) -> &MockTransform {
        &self.transform
    }
    
    pub fn get_transform_mut(&mut self) -> &mut MockTransform {
        &mut self.transform
    }
}

/// Mock Transform component for testing
#[derive(Debug, Clone)]
pub struct MockTransform {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub parent: Option<String>,
    pub children: Vec<String>,
}

impl MockTransform {
    pub fn new() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::identity(),
            scale: Vector3::one(),
            parent: None,
            children: Vec::new(),
        }
    }
    
    pub fn translate(&mut self, translation: Vector3) {
        self.position = self.position + translation;
    }
    
    pub fn rotate(&mut self, rotation: Vector3) {
        // Simple rotation implementation for testing
        let euler_rotation = Quaternion::from_euler(rotation.x, rotation.y, rotation.z);
        self.rotation = self.rotation * euler_rotation;
    }
    
    pub fn look_at(&mut self, target: Vector3) {
        let direction = (target - self.position).normalized();
        self.rotation = Quaternion::look_rotation(direction, Vector3::up());
    }
}

/// Mock networking state for testing
#[derive(Debug)]
pub struct MockNetworkingState {
    pub is_master: bool,
    pub owner_map: HashMap<String, u32>,
    pub synced_variables: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl MockNetworkingState {
    pub fn new() -> Self {
        Self {
            is_master: true,
            owner_map: HashMap::new(),
            synced_variables: HashMap::new(),
        }
    }
    
    pub fn set_owner(&mut self, object_name: &str, player_id: u32) {
        self.owner_map.insert(object_name.to_string(), player_id);
    }
    
    pub fn get_owner(&self, object_name: &str) -> Option<u32> {
        self.owner_map.get(object_name).copied()
    }
    
    pub fn is_owner(&self, object_name: &str, player_id: u32) -> bool {
        self.owner_map.get(object_name).map_or(false, |&owner| owner == player_id)
    }
}

/// Mock time state for testing
#[derive(Debug)]
pub struct MockTimeState {
    pub time: f32,
    pub delta_time: f32,
    pub fixed_time: f32,
    pub time_scale: f32,
}

impl MockTimeState {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            delta_time: 0.016667, // 60 FPS
            fixed_time: 0.0,
            time_scale: 1.0,
        }
    }
    
    pub fn advance_time(&mut self, delta: f32) {
        self.time += delta * self.time_scale;
        self.fixed_time += delta * self.time_scale;
        self.delta_time = delta * self.time_scale;
    }
    
    pub fn reset_time(&mut self) {
        self.time = 0.0;
        self.fixed_time = 0.0;
        self.delta_time = 0.016667;
    }
}

/// Mock physics state for testing
#[derive(Debug)]
pub struct MockPhysicsState {
    pub gravity: Vector3,
    pub physics_enabled: bool,
}

impl MockPhysicsState {
    pub fn new() -> Self {
        Self {
            gravity: Vector3::new(0.0, -9.81, 0.0),
            physics_enabled: true,
        }
    }
}

/// Mock audio state for testing
#[derive(Debug)]
pub struct MockAudioState {
    pub master_volume: f32,
    pub playing_sounds: HashMap<String, MockAudioSource>,
}

impl MockAudioState {
    pub fn new() -> Self {
        Self {
            master_volume: 1.0,
            playing_sounds: HashMap::new(),
        }
    }
}

/// Mock audio source for testing
#[derive(Debug, Clone)]
pub struct MockAudioSource {
    pub clip_name: String,
    pub volume: f32,
    pub pitch: f32,
    pub is_playing: bool,
    pub is_looping: bool,
}

impl MockAudioSource {
    pub fn new(clip_name: &str) -> Self {
        Self {
            clip_name: clip_name.to_string(),
            volume: 1.0,
            pitch: 1.0,
            is_playing: false,
            is_looping: false,
        }
    }
    
    pub fn play(&mut self) {
        self.is_playing = true;
    }
    
    pub fn stop(&mut self) {
        self.is_playing = false;
    }
    
    pub fn pause(&mut self) {
        self.is_playing = false;
    }
}

/// UdonSharp-specific assertion functions
pub mod assertions {
    use super::*;
    
    /// Assert that a VRCPlayerApi is valid
    pub fn assert_player_valid(player: &MockVRCPlayerApi) {
        assert!(!player.display_name.is_empty(), "Player display name should not be empty");
        assert!(player.player_id < 1000, "Player ID should be reasonable");
    }
    
    /// Assert that a GameObject is in the expected state
    pub fn assert_game_object_state(game_object: &MockGameObject, expected_active: bool) {
        assert_eq!(game_object.active, expected_active, 
                  "GameObject '{}' active state mismatch", game_object.name);
    }
    
    /// Assert that a transform has the expected position within tolerance
    pub fn assert_transform_position(transform: &MockTransform, expected: Vector3, tolerance: f32) {
        let distance = (transform.position - expected).magnitude();
        assert!(distance <= tolerance, 
               "Transform position mismatch: expected {:?}, got {:?}, distance: {}", 
               expected, transform.position, distance);
    }
    
    /// Assert that networking ownership is correct
    pub fn assert_ownership(object_name: &str, expected_owner: u32) {
        with_mock_environment(|env| {
            let actual_owner = env.networking_state.get_owner(object_name);
            assert_eq!(actual_owner, Some(expected_owner), 
                      "Ownership mismatch for '{}': expected {}, got {:?}", 
                      object_name, expected_owner, actual_owner);
        });
    }
    
    /// Assert that time has advanced as expected
    pub fn assert_time_advanced(expected_min_time: f32) {
        with_mock_environment(|env| {
            assert!(env.time_state.time >= expected_min_time, 
                   "Time should have advanced to at least {}, but is {}", 
                   expected_min_time, env.time_state.time);
        });
    }
    
    /// Assert that an audio source is playing
    pub fn assert_audio_playing(source_name: &str) {
        with_mock_environment(|env| {
            if let Some(source) = env.audio_state.playing_sounds.get(source_name) {
                assert!(source.is_playing, "Audio source '{}' should be playing", source_name);
            } else {
                panic!("Audio source '{}' not found", source_name);
            }
        });
    }
}

/// Test utilities for common UdonSharp testing scenarios
pub mod test_utils {
    use super::*;
    
    /// Create a test scenario with multiple players
    pub fn create_multiplayer_scenario(player_count: usize) -> Vec<MockVRCPlayerApi> {
        let mut players = Vec::new();
        
        with_mock_environment_mut(|env| {
            env.players.clear();
            
            // Add local player
            let local_player = MockVRCPlayerApi::local_player();
            env.players.push(local_player.clone());
            players.push(local_player);
            
            // Add remote players
            for i in 1..player_count {
                let player = env.add_player(&format!("Player{}", i));
                players.push(player);
            }
        });
        
        players
    }
    
    /// Simulate time passing in the test environment
    pub fn simulate_time_passage(seconds: f32) {
        with_mock_environment_mut(|env| {
            env.time_state.advance_time(seconds);
        });
    }
    
    /// Create a test world with common objects
    pub fn create_test_world() -> HashMap<String, MockGameObject> {
        let mut objects = HashMap::new();
        
        with_mock_environment_mut(|env| {
            // Create common world objects
            let spawn_point = env.create_game_object("SpawnPoint");
            objects.insert("SpawnPoint".to_string(), spawn_point);
            
            let world_controller = env.create_game_object("WorldController");
            objects.insert("WorldController".to_string(), world_controller);
            
            let ui_canvas = env.create_game_object("UICanvas");
            objects.insert("UICanvas".to_string(), ui_canvas);
        });
        
        objects
    }
    
    /// Simulate player joining the world
    pub fn simulate_player_join(display_name: &str) -> MockVRCPlayerApi {
        with_mock_environment_mut(|env| {
            env.add_player(display_name)
        })
    }
    
    /// Simulate player leaving the world
    pub fn simulate_player_leave(player_id: u32) {
        with_mock_environment_mut(|env| {
            env.players.retain(|p| p.player_id != player_id);
        });
    }
    
    /// Reset the test environment to initial state
    pub fn reset_test_environment() {
        with_mock_environment_mut(|env| {
            env.players.clear();
            env.players.push(MockVRCPlayerApi::local_player());
            env.game_objects.clear();
            env.networking_state = MockNetworkingState::new();
            env.time_state.reset_time();
            env.physics_state = MockPhysicsState::new();
            env.audio_state = MockAudioState::new();
        });
    }
}

/// Setup mock environment for testing
pub fn setup_mock_environment() {
    unsafe {
        INIT.call_once(|| {
            TEST_ENVIRONMENT = Some(Arc::new(Mutex::new(MockEnvironment::new())));
        });
    }
}

/// Cleanup mock environment after testing
pub fn cleanup_mock_environment() {
    // Environment is automatically cleaned up when the test ends
    // This function is here for explicit cleanup if needed
}

/// Execute a function with access to the mock environment
pub fn with_mock_environment<F, R>(f: F) -> R
where
    F: FnOnce(&MockEnvironment) -> R,
{
    unsafe {
        if let Some(ref env) = TEST_ENVIRONMENT {
            let env_guard = env.lock().unwrap();
            f(&*env_guard)
        } else {
            panic!("Mock environment not initialized. Call setup_mock_environment() first.");
        }
    }
}

/// Execute a function with mutable access to the mock environment
pub fn with_mock_environment_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut MockEnvironment) -> R,
{
    unsafe {
        if let Some(ref env) = TEST_ENVIRONMENT {
            let mut env_guard = env.lock().unwrap();
            f(&mut *env_guard)
        } else {
            panic!("Mock environment not initialized. Call setup_mock_environment() first.");
        }
    }
}

/// Get the current mock environment state for inspection
pub fn get_mock_environment_state() -> MockEnvironment {
    with_mock_environment(|env| {
        // Create a simplified clone for inspection
        MockEnvironment {
            players: env.players.clone(),
            game_objects: env.game_objects.clone(),
            networking_state: MockNetworkingState {
                is_master: env.networking_state.is_master,
                owner_map: env.networking_state.owner_map.clone(),
                synced_variables: HashMap::new(), // Skip complex cloning for now
            },
            time_state: MockTimeState {
                time: env.time_state.time,
                delta_time: env.time_state.delta_time,
                fixed_time: env.time_state.fixed_time,
                time_scale: env.time_state.time_scale,
            },
            physics_state: MockPhysicsState {
                gravity: env.physics_state.gravity,
                physics_enabled: env.physics_state.physics_enabled,
            },
            audio_state: MockAudioState {
                master_volume: env.audio_state.master_volume,
                playing_sounds: env.audio_state.playing_sounds.clone(),
            },
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_environment_creation() {
        let env = MockEnvironment::new();
        assert_eq!(env.players.len(), 1);
        assert!(env.players[0].is_local);
        assert!(env.game_objects.is_empty());
    }
    
    #[test]
    fn test_mock_player_creation() {
        let player = MockVRCPlayerApi::new(1, "TestPlayer");
        assert_eq!(player.player_id, 1);
        assert_eq!(player.display_name, "TestPlayer");
        assert!(!player.is_local);
        assert!(!player.is_master);
    }
    
    #[test]
    fn test_mock_game_object_creation() {
        let game_object = MockGameObject::new("TestObject");
        assert_eq!(game_object.name, "TestObject");
        assert!(game_object.active);
        assert_eq!(game_object.transform.position, Vector3::zero());
    }
    
    #[test]
    fn test_mock_transform_operations() {
        let mut transform = MockTransform::new();
        
        transform.translate(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(transform.position, Vector3::new(1.0, 2.0, 3.0));
        
        transform.rotate(Vector3::new(0.0, 90.0, 0.0));
        // Basic rotation test - just ensure it doesn't panic
        assert_ne!(transform.rotation, Quaternion::identity());
    }
    
    #[test]
    fn test_networking_state() {
        let mut networking = MockNetworkingState::new();
        
        networking.set_owner("TestObject", 1);
        assert_eq!(networking.get_owner("TestObject"), Some(1));
        assert!(networking.is_owner("TestObject", 1));
        assert!(!networking.is_owner("TestObject", 2));
    }
    
    #[test]
    fn test_time_state() {
        let mut time_state = MockTimeState::new();
        let initial_time = time_state.time;
        
        time_state.advance_time(1.0);
        assert!(time_state.time > initial_time);
        
        time_state.reset_time();
        assert_eq!(time_state.time, 0.0);
    }
    
    #[test]
    fn test_audio_source() {
        let mut audio_source = MockAudioSource::new("TestClip");
        assert!(!audio_source.is_playing);
        
        audio_source.play();
        assert!(audio_source.is_playing);
        
        audio_source.stop();
        assert!(!audio_source.is_playing);
    }
}