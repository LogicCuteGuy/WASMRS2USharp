use udonsharp_macros::udon_behaviour;

#[udon_behaviour]
pub fn simple_behaviour() {
    // Simple test function
}

#[udon_behaviour(name = "CustomBehaviour")]
pub fn custom_named_behaviour() {
    // Custom named behaviour
}

#[udon_behaviour(name = "GameManager", events = "Start,Update")]
pub fn game_manager() {
    // Game manager with multiple events
}

#[udon_behaviour(name = "NetworkManager", dependencies = "PlayerManager", auto_sync = true)]
pub fn network_manager() {
    // Network manager with dependencies and auto sync
}

#[test]
fn test_udon_behaviour_compilation() {
    // If this compiles, the macro is working
    simple_behaviour();
    custom_named_behaviour();
    game_manager();
    network_manager();
}