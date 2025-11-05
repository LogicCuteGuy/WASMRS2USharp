# Hello World Example

A simple "Hello World" example demonstrating the basics of Rust UdonSharp development.

## What This Example Demonstrates

- Basic UdonSharp behavior implementation
- Using the `UdonBehaviour` trait
- Public fields visible in Unity Inspector
- Logging and debug output
- Player join/leave event handling

## Code Overview

This example creates a simple world controller that:
- Displays a welcome message when the world starts
- Logs when players join or leave
- Tracks the total number of players
- Shows basic UdonSharp attribute usage

## Files

- `src/lib.rs` - Main behavior implementation
- `Cargo.toml` - Project configuration
- `udonsharp.toml` - UdonSharp-specific settings
- `build.rs` - Build script

## Building

```bash
# Navigate to the example directory
cd examples/hello-world

# Build the project
cargo udonsharp build
```

## Unity Setup

1. Copy the generated C# files from `output/` to your Unity project's Assets folder
2. Create an empty GameObject in your scene
3. Add the `HelloWorld` UdonSharp behavior component
4. Configure the public fields in the Inspector
5. Build and test your world

## Key Concepts

### UdonBehaviour Trait

The main trait that all UdonSharp behaviors must implement:

```rust
impl UdonBehaviour for HelloWorld {
    fn start(&mut self) {
        // Called when the behavior starts
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        // Called when a player joins
    }
}
```

### Public Fields

Fields marked with `#[udon_public]` are visible in the Unity Inspector:

```rust
#[udon_public]
pub welcome_message: String,
```

### Logging

Use the logging macros for debug output:

```rust
log::info!("World started with message: {}", self.welcome_message);
```

## Next Steps

After understanding this example, try:
- [player-counter](../player-counter/) - Learn about networking and synchronization
- [interactive-button](../interactive-button/) - Explore user interaction systems