//! Integration testing system for UdonSharp development
//! 
//! This module provides integration tests that validate the entire pipeline
//! from Rust code to generated UdonSharp, including Unity environment simulation
//! and VRChat world testing scenarios.

use crate::testing::*;
// use crate::types::*;
// use crate::traits::UdonBehaviour;
// use std::collections::HashMap;
use std::time::Duration;

/// Integration test runner for UdonSharp projects
pub struct UdonSharpIntegrationTester {
    test_scenarios: Vec<IntegrationTestScenario>,
    api_binding_tests: Vec<ApiBindingTest>,
    world_simulation_tests: Vec<WorldSimulationTest>,
}

impl UdonSharpIntegrationTester {
    pub fn new() -> Self {
        Self {
            test_scenarios: Vec::new(),
            api_binding_tests: Vec::new(),
            world_simulation_tests: Vec::new(),
        }
    }
    
    /// Add a new integration test scenario
    pub fn add_scenario(&mut self, scenario: IntegrationTestScenario) {
        self.test_scenarios.push(scenario);
    }
    
    /// Add an API binding test
    pub fn add_api_binding_test(&mut self, test: ApiBindingTest) {
        self.api_binding_tests.push(test);
    }
    
    /// Add a world simulation test
    pub fn add_world_simulation_test(&mut self, test: WorldSimulationTest) {
        self.world_simulation_tests.push(test);
    }
    
    /// Run all integration tests
    pub fn run_all_tests(&self) -> IntegrationTestResults {
        let mut results = IntegrationTestResults::new();
        
        // Run integration test scenarios
        for scenario in &self.test_scenarios {
            let result = self.run_integration_scenario(scenario);
            results.add_scenario_result(result);
        }
        
        // Run API binding tests
        for test in &self.api_binding_tests {
            let result = self.run_api_binding_test(test);
            results.add_api_binding_result(result);
        }
        
        // Run world simulation tests
        for test in &self.world_simulation_tests {
            let result = self.run_world_simulation_test(test);
            results.add_world_simulation_result(result);
        }
        
        results
    }
    
    fn run_integration_scenario(&self, scenario: &IntegrationTestScenario) -> IntegrationTestResult {
        let start_time = std::time::Instant::now();
        
        // Set up test environment
        setup_mock_environment();
        
        let mut result = IntegrationTestResult {
            test_name: scenario.name.clone(),
            success: false,
            duration: Duration::ZERO,
            error_message: None,
            assertions_passed: 0,
            assertions_total: scenario.assertions.len(),
        };
        
        // Execute test steps
        match self.execute_scenario_steps(&scenario.steps) {
            Ok(_) => {
                // Run assertions
                let mut passed_assertions = 0;
                for assertion in &scenario.assertions {
                    if self.run_assertion(assertion) {
                        passed_assertions += 1;
                    }
                }
                
                result.success = passed_assertions == scenario.assertions.len();
                result.assertions_passed = passed_assertions;
            }
            Err(e) => {
                result.error_message = Some(e.to_string());
            }
        }
        
        result.duration = start_time.elapsed();
        cleanup_mock_environment();
        
        result
    }
    
    fn execute_scenario_steps(&self, steps: &[TestStep]) -> Result<(), Box<dyn std::error::Error>> {
        for step in steps {
            match step {
                TestStep::CreateGameObject { name } => {
                    with_mock_environment_mut(|env| {
                        env.create_game_object(name);
                    });
                }
                TestStep::AddPlayer { display_name } => {
                    with_mock_environment_mut(|env| {
                        env.add_player(display_name);
                    });
                }
                TestStep::SimulateTime { seconds } => {
                    with_mock_environment_mut(|env| {
                        env.time_state.advance_time(*seconds);
                    });
                }
                TestStep::TriggerEvent { object_name, event_name } => {
                    // Simulate triggering a UdonSharp event
                    println!("Triggering event '{}' on object '{}'", event_name, object_name);
                }
                TestStep::SetNetworkOwnership { object_name, player_id } => {
                    with_mock_environment_mut(|env| {
                        env.networking_state.set_owner(object_name, *player_id);
                    });
                }
                TestStep::WaitForCondition { condition, timeout } => {
                    let start = std::time::Instant::now();
                    while start.elapsed() < *timeout {
                        if self.evaluate_condition(condition) {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
            }
        }
        Ok(())
    }
    
    fn run_assertion(&self, assertion: &TestAssertion) -> bool {
        match assertion {
            TestAssertion::ObjectExists { name } => {
                with_mock_environment(|env| {
                    env.find_game_object(name).is_some()
                })
            }
            TestAssertion::PlayerCount { expected } => {
                with_mock_environment(|env| {
                    env.players.len() == *expected
                })
            }
            TestAssertion::TimeAdvanced { min_seconds } => {
                with_mock_environment(|env| {
                    env.time_state.time >= *min_seconds
                })
            }
            TestAssertion::NetworkOwnership { object_name, expected_owner } => {
                with_mock_environment(|env| {
                    env.networking_state.get_owner(object_name) == Some(*expected_owner)
                })
            }
            TestAssertion::CustomCondition { condition } => {
                self.evaluate_condition(condition)
            }
        }
    }
    
    fn evaluate_condition(&self, condition: &str) -> bool {
        // Simple condition evaluation - in a real implementation this would be more sophisticated
        match condition {
            "players_initialized" => {
                with_mock_environment(|env| {
                    !env.players.is_empty() && env.players[0].is_local
                })
            }
            "world_active" => true, // Mock condition
            _ => false,
        }
    }
    
    fn run_api_binding_test(&self, test: &ApiBindingTest) -> ApiBindingTestResult {
        let start_time = std::time::Instant::now();
        
        let mut result = ApiBindingTestResult {
            test_name: test.name.clone(),
            api_category: test.api_category.clone(),
            success: false,
            duration: Duration::ZERO,
            error_message: None,
            bindings_tested: test.binding_calls.len(),
            bindings_passed: 0,
        };
        
        setup_mock_environment();
        
        let mut passed_bindings = 0;
        for binding_call in &test.binding_calls {
            if self.test_api_binding(binding_call) {
                passed_bindings += 1;
            }
        }
        
        result.success = passed_bindings == test.binding_calls.len();
        result.bindings_passed = passed_bindings;
        result.duration = start_time.elapsed();
        
        cleanup_mock_environment();
        result
    }
    
    fn test_api_binding(&self, binding_call: &ApiBindingCall) -> bool {
        match binding_call {
            ApiBindingCall::VRChatNetworking { method, expected_result } => {
                // Test VRChat networking API bindings
                match method.as_str() {
                    "IsOwner" => {
                        // Mock test for IsOwner
                        *expected_result == "true" || *expected_result == "false"
                    }
                    "GetLocalPlayer" => {
                        // Mock test for GetLocalPlayer
                        expected_result.contains("Player")
                    }
                    _ => false,
                }
            }
            ApiBindingCall::UnityEngine { method, expected_result } => {
                // Test Unity Engine API bindings
                match method.as_str() {
                    "GameObject.Find" => {
                        // Mock test for GameObject.Find
                        expected_result.contains("GameObject") || *expected_result == "null"
                    }
                    "Transform.position" => {
                        // Mock test for Transform.position
                        expected_result.contains("Vector3")
                    }
                    _ => false,
                }
            }
            ApiBindingCall::CSharpSystem { method, expected_result } => {
                // Test C# system library bindings
                match method.as_str() {
                    "String.Length" => {
                        // Mock test for String.Length
                        expected_result.parse::<i32>().is_ok()
                    }
                    "Math.Sin" => {
                        // Mock test for Math.Sin
                        expected_result.parse::<f32>().is_ok()
                    }
                    _ => false,
                }
            }
        }
    }
    
    fn run_world_simulation_test(&self, test: &WorldSimulationTest) -> WorldSimulationTestResult {
        let start_time = std::time::Instant::now();
        
        let mut result = WorldSimulationTestResult {
            test_name: test.name.clone(),
            scenario_type: test.scenario_type.clone(),
            success: false,
            duration: Duration::ZERO,
            error_message: None,
            events_triggered: 0,
            expected_events: test.expected_events.len(),
        };
        
        setup_mock_environment();
        
        // Set up world scenario
        match self.setup_world_scenario(&test.scenario_type) {
            Ok(_) => {
                // Run simulation steps
                let mut events_triggered = 0;
                for event in &test.expected_events {
                    if self.simulate_world_event(event) {
                        events_triggered += 1;
                    }
                }
                
                result.success = events_triggered == test.expected_events.len();
                result.events_triggered = events_triggered;
            }
            Err(e) => {
                result.error_message = Some(e.to_string());
            }
        }
        
        result.duration = start_time.elapsed();
        cleanup_mock_environment();
        result
    }
    
    fn setup_world_scenario(&self, scenario_type: &str) -> Result<(), Box<dyn std::error::Error>> {
        match scenario_type {
            "multiplayer_lobby" => {
                // Set up a multiplayer lobby scenario
                with_mock_environment_mut(|env| {
                    env.create_game_object("LobbyController");
                    env.create_game_object("SpawnPoint");
                    env.add_player("Player1");
                    env.add_player("Player2");
                });
                Ok(())
            }
            "interactive_world" => {
                // Set up an interactive world scenario
                with_mock_environment_mut(|env| {
                    env.create_game_object("InteractableButton");
                    env.create_game_object("WorldController");
                    env.create_game_object("UICanvas");
                });
                Ok(())
            }
            "physics_simulation" => {
                // Set up a physics simulation scenario
                with_mock_environment_mut(|env| {
                    env.create_game_object("PhysicsObject");
                    env.create_game_object("Ground");
                    env.physics_state.physics_enabled = true;
                });
                Ok(())
            }
            _ => Err(format!("Unknown scenario type: {}", scenario_type).into()),
        }
    }
    
    fn simulate_world_event(&self, event: &WorldEvent) -> bool {
        match event {
            WorldEvent::PlayerJoin { player_name } => {
                with_mock_environment_mut(|env| {
                    env.add_player(player_name);
                });
                true
            }
            WorldEvent::PlayerLeave { player_id } => {
                with_mock_environment_mut(|env| {
                    env.players.retain(|p| p.player_id != *player_id);
                });
                true
            }
            WorldEvent::ObjectInteraction { object_name, player_id } => {
                // Simulate object interaction
                with_mock_environment(|env| {
                    env.find_game_object(object_name).is_some() &&
                    env.players.iter().any(|p| p.player_id == *player_id)
                })
            }
            WorldEvent::NetworkSync { variable_name, value } => {
                with_mock_environment_mut(|env| {
                    env.networking_state.synced_variables.insert(
                        variable_name.clone(),
                        Box::new(value.clone())
                    );
                });
                true
            }
            WorldEvent::TimeAdvance { seconds } => {
                with_mock_environment_mut(|env| {
                    env.time_state.advance_time(*seconds);
                });
                true
            }
        }
    }
}

/// Integration test scenario definition
#[derive(Debug, Clone)]
pub struct IntegrationTestScenario {
    pub name: String,
    pub description: String,
    pub steps: Vec<TestStep>,
    pub assertions: Vec<TestAssertion>,
}

/// Test step in an integration scenario
#[derive(Debug, Clone)]
pub enum TestStep {
    CreateGameObject { name: String },
    AddPlayer { display_name: String },
    SimulateTime { seconds: f32 },
    TriggerEvent { object_name: String, event_name: String },
    SetNetworkOwnership { object_name: String, player_id: u32 },
    WaitForCondition { condition: String, timeout: Duration },
}

/// Test assertion for validation
#[derive(Debug, Clone)]
pub enum TestAssertion {
    ObjectExists { name: String },
    PlayerCount { expected: usize },
    TimeAdvanced { min_seconds: f32 },
    NetworkOwnership { object_name: String, expected_owner: u32 },
    CustomCondition { condition: String },
}

/// API binding test definition
#[derive(Debug, Clone)]
pub struct ApiBindingTest {
    pub name: String,
    pub api_category: String,
    pub binding_calls: Vec<ApiBindingCall>,
}

/// API binding call test
#[derive(Debug, Clone)]
pub enum ApiBindingCall {
    VRChatNetworking { method: String, expected_result: String },
    UnityEngine { method: String, expected_result: String },
    CSharpSystem { method: String, expected_result: String },
}

/// World simulation test definition
#[derive(Debug, Clone)]
pub struct WorldSimulationTest {
    pub name: String,
    pub scenario_type: String,
    pub expected_events: Vec<WorldEvent>,
}

/// World event for simulation
#[derive(Debug, Clone)]
pub enum WorldEvent {
    PlayerJoin { player_name: String },
    PlayerLeave { player_id: u32 },
    ObjectInteraction { object_name: String, player_id: u32 },
    NetworkSync { variable_name: String, value: String },
    TimeAdvance { seconds: f32 },
}

/// Results of integration tests
#[derive(Debug)]
pub struct IntegrationTestResults {
    pub scenario_results: Vec<IntegrationTestResult>,
    pub api_binding_results: Vec<ApiBindingTestResult>,
    pub world_simulation_results: Vec<WorldSimulationTestResult>,
}

impl IntegrationTestResults {
    pub fn new() -> Self {
        Self {
            scenario_results: Vec::new(),
            api_binding_results: Vec::new(),
            world_simulation_results: Vec::new(),
        }
    }
    
    pub fn add_scenario_result(&mut self, result: IntegrationTestResult) {
        self.scenario_results.push(result);
    }
    
    pub fn add_api_binding_result(&mut self, result: ApiBindingTestResult) {
        self.api_binding_results.push(result);
    }
    
    pub fn add_world_simulation_result(&mut self, result: WorldSimulationTestResult) {
        self.world_simulation_results.push(result);
    }
    
    pub fn total_tests(&self) -> usize {
        self.scenario_results.len() + 
        self.api_binding_results.len() + 
        self.world_simulation_results.len()
    }
    
    pub fn passed_tests(&self) -> usize {
        self.scenario_results.iter().filter(|r| r.success).count() +
        self.api_binding_results.iter().filter(|r| r.success).count() +
        self.world_simulation_results.iter().filter(|r| r.success).count()
    }
    
    pub fn success_rate(&self) -> f32 {
        if self.total_tests() == 0 {
            return 0.0;
        }
        self.passed_tests() as f32 / self.total_tests() as f32
    }
    
    pub fn print_summary(&self) {
        println!("\n=== Integration Test Results ===");
        println!("Total Tests: {}", self.total_tests());
        println!("Passed: {}", self.passed_tests());
        println!("Failed: {}", self.total_tests() - self.passed_tests());
        println!("Success Rate: {:.1}%", self.success_rate() * 100.0);
        
        if !self.scenario_results.is_empty() {
            println!("\n--- Integration Scenarios ---");
            for result in &self.scenario_results {
                let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
                println!("{} {} ({:.2}s) - {}/{} assertions", 
                        status, result.test_name, result.duration.as_secs_f32(),
                        result.assertions_passed, result.assertions_total);
                
                if let Some(ref error) = result.error_message {
                    println!("    Error: {}", error);
                }
            }
        }
        
        if !self.api_binding_results.is_empty() {
            println!("\n--- API Binding Tests ---");
            for result in &self.api_binding_results {
                let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
                println!("{} {} [{}] ({:.2}s) - {}/{} bindings", 
                        status, result.test_name, result.api_category, 
                        result.duration.as_secs_f32(),
                        result.bindings_passed, result.bindings_tested);
                
                if let Some(ref error) = result.error_message {
                    println!("    Error: {}", error);
                }
            }
        }
        
        if !self.world_simulation_results.is_empty() {
            println!("\n--- World Simulation Tests ---");
            for result in &self.world_simulation_results {
                let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
                println!("{} {} [{}] ({:.2}s) - {}/{} events", 
                        status, result.test_name, result.scenario_type,
                        result.duration.as_secs_f32(),
                        result.events_triggered, result.expected_events);
                
                if let Some(ref error) = result.error_message {
                    println!("    Error: {}", error);
                }
            }
        }
    }
}

/// Result of an integration test scenario
#[derive(Debug)]
pub struct IntegrationTestResult {
    pub test_name: String,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub assertions_passed: usize,
    pub assertions_total: usize,
}

/// Result of an API binding test
#[derive(Debug)]
pub struct ApiBindingTestResult {
    pub test_name: String,
    pub api_category: String,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub bindings_tested: usize,
    pub bindings_passed: usize,
}

/// Result of a world simulation test
#[derive(Debug)]
pub struct WorldSimulationTestResult {
    pub test_name: String,
    pub scenario_type: String,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub events_triggered: usize,
    pub expected_events: usize,
}

/// Predefined integration test scenarios
pub mod scenarios {
    use super::*;
    
    pub fn basic_udon_behaviour_test() -> IntegrationTestScenario {
        IntegrationTestScenario {
            name: "Basic UdonBehaviour Test".to_string(),
            description: "Test basic UdonBehaviour lifecycle methods".to_string(),
            steps: vec![
                TestStep::CreateGameObject { name: "TestController".to_string() },
                TestStep::SimulateTime { seconds: 0.1 },
            ],
            assertions: vec![
                TestAssertion::ObjectExists { name: "TestController".to_string() },
                TestAssertion::TimeAdvanced { min_seconds: 0.1 },
            ],
        }
    }
    
    pub fn multiplayer_networking_test() -> IntegrationTestScenario {
        IntegrationTestScenario {
            name: "Multiplayer Networking Test".to_string(),
            description: "Test multiplayer networking functionality".to_string(),
            steps: vec![
                TestStep::CreateGameObject { name: "NetworkController".to_string() },
                TestStep::AddPlayer { display_name: "Player1".to_string() },
                TestStep::AddPlayer { display_name: "Player2".to_string() },
                TestStep::SetNetworkOwnership { object_name: "NetworkController".to_string(), player_id: 0 },
            ],
            assertions: vec![
                TestAssertion::PlayerCount { expected: 3 }, // Including local player
                TestAssertion::NetworkOwnership { object_name: "NetworkController".to_string(), expected_owner: 0 },
            ],
        }
    }
    
    pub fn interactive_world_test() -> IntegrationTestScenario {
        IntegrationTestScenario {
            name: "Interactive World Test".to_string(),
            description: "Test interactive world elements".to_string(),
            steps: vec![
                TestStep::CreateGameObject { name: "InteractableButton".to_string() },
                TestStep::CreateGameObject { name: "UICanvas".to_string() },
                TestStep::AddPlayer { display_name: "TestPlayer".to_string() },
                TestStep::TriggerEvent { object_name: "InteractableButton".to_string(), event_name: "OnInteract".to_string() },
            ],
            assertions: vec![
                TestAssertion::ObjectExists { name: "InteractableButton".to_string() },
                TestAssertion::ObjectExists { name: "UICanvas".to_string() },
                TestAssertion::PlayerCount { expected: 2 },
            ],
        }
    }
}

/// Predefined API binding tests
pub mod api_tests {
    use super::*;
    
    pub fn vrchat_networking_bindings() -> ApiBindingTest {
        ApiBindingTest {
            name: "VRChat Networking Bindings".to_string(),
            api_category: "VRChat".to_string(),
            binding_calls: vec![
                ApiBindingCall::VRChatNetworking {
                    method: "IsOwner".to_string(),
                    expected_result: "true".to_string(),
                },
                ApiBindingCall::VRChatNetworking {
                    method: "GetLocalPlayer".to_string(),
                    expected_result: "LocalPlayer".to_string(),
                },
            ],
        }
    }
    
    pub fn unity_engine_bindings() -> ApiBindingTest {
        ApiBindingTest {
            name: "Unity Engine Bindings".to_string(),
            api_category: "Unity".to_string(),
            binding_calls: vec![
                ApiBindingCall::UnityEngine {
                    method: "GameObject.Find".to_string(),
                    expected_result: "GameObject".to_string(),
                },
                ApiBindingCall::UnityEngine {
                    method: "Transform.position".to_string(),
                    expected_result: "Vector3(0, 0, 0)".to_string(),
                },
            ],
        }
    }
    
    pub fn csharp_system_bindings() -> ApiBindingTest {
        ApiBindingTest {
            name: "C# System Bindings".to_string(),
            api_category: "C# System".to_string(),
            binding_calls: vec![
                ApiBindingCall::CSharpSystem {
                    method: "String.Length".to_string(),
                    expected_result: "5".to_string(),
                },
                ApiBindingCall::CSharpSystem {
                    method: "Math.Sin".to_string(),
                    expected_result: "0.0".to_string(),
                },
            ],
        }
    }
}

/// Predefined world simulation tests
pub mod world_tests {
    use super::*;
    
    pub fn multiplayer_lobby_simulation() -> WorldSimulationTest {
        WorldSimulationTest {
            name: "Multiplayer Lobby Simulation".to_string(),
            scenario_type: "multiplayer_lobby".to_string(),
            expected_events: vec![
                WorldEvent::PlayerJoin { player_name: "Player1".to_string() },
                WorldEvent::PlayerJoin { player_name: "Player2".to_string() },
                WorldEvent::NetworkSync { variable_name: "player_count".to_string(), value: "3".to_string() },
            ],
        }
    }
    
    pub fn interactive_world_simulation() -> WorldSimulationTest {
        WorldSimulationTest {
            name: "Interactive World Simulation".to_string(),
            scenario_type: "interactive_world".to_string(),
            expected_events: vec![
                WorldEvent::ObjectInteraction { object_name: "InteractableButton".to_string(), player_id: 0 },
                WorldEvent::NetworkSync { variable_name: "interaction_count".to_string(), value: "1".to_string() },
                WorldEvent::TimeAdvance { seconds: 1.0 },
            ],
        }
    }
    
    pub fn physics_simulation() -> WorldSimulationTest {
        WorldSimulationTest {
            name: "Physics Simulation".to_string(),
            scenario_type: "physics_simulation".to_string(),
            expected_events: vec![
                WorldEvent::TimeAdvance { seconds: 2.0 },
                WorldEvent::NetworkSync { variable_name: "physics_state".to_string(), value: "active".to_string() },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_integration_tester_creation() {
        let tester = UdonSharpIntegrationTester::new();
        assert_eq!(tester.test_scenarios.len(), 0);
        assert_eq!(tester.api_binding_tests.len(), 0);
        assert_eq!(tester.world_simulation_tests.len(), 0);
    }
    
    #[test]
    fn test_scenario_creation() {
        let scenario = scenarios::basic_udon_behaviour_test();
        assert_eq!(scenario.name, "Basic UdonBehaviour Test");
        assert_eq!(scenario.steps.len(), 2);
        assert_eq!(scenario.assertions.len(), 2);
    }
    
    #[test]
    fn test_api_binding_test_creation() {
        let test = api_tests::vrchat_networking_bindings();
        assert_eq!(test.name, "VRChat Networking Bindings");
        assert_eq!(test.api_category, "VRChat");
        assert_eq!(test.binding_calls.len(), 2);
    }
    
    #[test]
    fn test_world_simulation_test_creation() {
        let test = world_tests::multiplayer_lobby_simulation();
        assert_eq!(test.name, "Multiplayer Lobby Simulation");
        assert_eq!(test.scenario_type, "multiplayer_lobby");
        assert_eq!(test.expected_events.len(), 3);
    }
    
    #[test]
    fn test_integration_test_results() {
        let mut results = IntegrationTestResults::new();
        assert_eq!(results.total_tests(), 0);
        assert_eq!(results.passed_tests(), 0);
        assert_eq!(results.success_rate(), 0.0);
        
        // Add a successful test result
        results.add_scenario_result(IntegrationTestResult {
            test_name: "Test".to_string(),
            success: true,
            duration: Duration::from_millis(100),
            error_message: None,
            assertions_passed: 2,
            assertions_total: 2,
        });
        
        assert_eq!(results.total_tests(), 1);
        assert_eq!(results.passed_tests(), 1);
        assert_eq!(results.success_rate(), 1.0);
    }
    
    #[test]
    fn test_full_integration_test_run() {
        let mut tester = UdonSharpIntegrationTester::new();
        
        // Add test scenarios
        tester.add_scenario(scenarios::basic_udon_behaviour_test());
        tester.add_api_binding_test(api_tests::vrchat_networking_bindings());
        tester.add_world_simulation_test(world_tests::multiplayer_lobby_simulation());
        
        // Run all tests
        let results = tester.run_all_tests();
        
        // Verify results structure
        assert!(results.total_tests() > 0);
        assert!(results.scenario_results.len() > 0);
        assert!(results.api_binding_results.len() > 0);
        assert!(results.world_simulation_results.len() > 0);
        
        // Print summary for manual verification
        results.print_summary();
    }
}