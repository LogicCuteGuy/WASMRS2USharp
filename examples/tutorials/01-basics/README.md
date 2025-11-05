# Tutorial 01: Getting Started with Rust UdonSharp

Welcome to the first tutorial in the Rust UdonSharp series! This tutorial will guide you through the basics of developing VRChat worlds using Rust and the UdonSharp framework.

## Learning Objectives

By the end of this tutorial, you will:
- Understand what Rust UdonSharp is and how it works
- Set up a complete development environment
- Create and build your first UdonSharp behavior
- Integrate generated C# code with Unity
- Understand the basic compilation pipeline

## What is Rust UdonSharp?

Rust UdonSharp is a framework that allows you to write VRChat world logic in Rust instead of C#. Your Rust code is compiled to WebAssembly, then converted to UdonSharp-compatible C# code that runs in VRChat.

### Benefits of Rust UdonSharp

- **Memory Safety**: Rust's ownership system prevents common bugs
- **Performance**: Rust's zero-cost abstractions and optimization
- **Type Safety**: Strong type system catches errors at compile time
- **Ecosystem**: Access to Rust's rich ecosystem of crates
- **Tooling**: Excellent development tools and IDE support

### The Compilation Pipeline

```
Rust Source Code → WebAssembly → Optimized WASM → UdonSharp C# → VRChat
```

## Prerequisites

### Required Software

1. **Rust 1.70+**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Add WebAssembly target
   rustup target add wasm32-unknown-unknown
   ```

2. **Unity 2022.3+**
   - Download from [Unity Hub](https://unity.com/download)
   - Install VRChat SDK3 Worlds from [VRChat Creator Companion](https://vcc.docs.vrchat.com/)

3. **wasm-opt** (for optimization)
   ```bash
   # macOS with Homebrew
   brew install binaryen
   
   # Ubuntu/Debian
   sudo apt install binaryen
   
   # Windows: Download from https://github.com/WebAssembly/binaryen/releases
   ```

4. **Rust UdonSharp Framework**
   ```bash
   cargo install cargo-udonsharp
   ```

### Recommended Tools

- **VS Code** with Rust Analyzer extension
- **Git** for version control
- **VRChat** account for testing worlds

## Chapter 1: Your First Project

### Step 1: Create a New Project

```bash
# Create a new Rust UdonSharp project
cargo udonsharp new my-first-world
cd my-first-world
```

This creates the following structure:
```
my-first-world/
├── Cargo.toml          # Rust project configuration
├── udonsharp.toml      # UdonSharp-specific settings
├── build.rs            # Build script
└── src/
    └── lib.rs          # Your main behavior code
```

### Step 2: Examine the Generated Code

Open `src/lib.rs` and examine the generated code:

```rust
use udonsharp_core::prelude::*;
use udonsharp_bindings::{vrchat, unity};

#[derive(UdonBehaviour)]
pub struct MyFirstWorld {
    #[udon_public]
    pub welcome_message: String,
    
    player_count: i32,
}

impl UdonBehaviour for MyFirstWorld {
    fn start(&mut self) {
        log::info!("World started!");
        self.welcome_message = "Welcome to my Rust world!".to_string();
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.player_count += 1;
        log::info!("Player joined: {}", player.get_display_name());
    }
}
```

### Step 3: Understanding the Code

Let's break down each part:

#### The UdonBehaviour Derive

```rust
#[derive(UdonBehaviour)]
pub struct MyFirstWorld {
    // ...
}
```

This macro generates the necessary code to make your struct work as a UdonSharp behavior.

#### Public Fields

```rust
#[udon_public]
pub welcome_message: String,
```

Fields marked with `#[udon_public]` are visible in the Unity Inspector, just like public fields in UdonSharp.

#### The UdonBehaviour Trait

```rust
impl UdonBehaviour for MyFirstWorld {
    fn start(&mut self) {
        // Called when the behavior starts
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        // Called when a player joins the world
    }
}
```

This trait provides the standard UdonSharp lifecycle methods.

### Step 4: Build Your Project

```bash
cargo udonsharp build
```

This command:
1. Compiles your Rust code to WebAssembly
2. Optimizes the WASM with wasm-opt
3. Converts WASM to UdonSharp C# code
4. Organizes the output into multiple files

You'll find the generated C# files in the `output/` directory.

## Chapter 2: Unity Integration

### Step 1: Set Up Unity Project

1. Create a new Unity project using the VRChat World template
2. Ensure VRChat SDK is properly installed
3. Create a basic scene with a ground plane

### Step 2: Import Generated Code

1. Copy the contents of `output/` to your Unity project's `Assets/Scripts/` folder
2. Wait for Unity to compile the scripts
3. Check the Console for any compilation errors

### Step 3: Add Behavior to Scene

1. Create an empty GameObject in your scene
2. Add the generated UdonSharp behavior component
3. Configure the public fields in the Inspector
4. Save your scene

### Step 4: Test in Unity

1. Enter Play mode in Unity
2. Check the Console for log messages
3. Test basic functionality

## Chapter 3: Understanding the Framework

### Core Concepts

#### Traits vs Classes

In Rust UdonSharp, you define behaviors using structs and traits instead of classes:

```rust
// Rust UdonSharp
#[derive(UdonBehaviour)]
pub struct MyBehavior {
    pub field: i32,
}

impl UdonBehaviour for MyBehavior {
    fn start(&mut self) { }
}
```

This generates equivalent UdonSharp C#:

```csharp
// Generated UdonSharp C#
public class MyBehavior : UdonSharpBehaviour
{
    public int field;
    
    void Start()
    {
        // Generated implementation
    }
}
```

#### Attributes

Rust UdonSharp uses attributes to control code generation:

```rust
#[derive(UdonBehaviour)]           // Makes this a UdonSharp behavior
#[udon_sync_mode(Manual)]          // Sets networking sync mode
pub struct NetworkedBehavior {
    #[udon_public]                 // Visible in Inspector
    pub public_field: String,
    
    #[udon_sync]                   // Synchronized over network
    pub synced_field: i32,
    
    private_field: bool,           // Private, not synchronized
}
```

#### API Bindings

Access VRChat and Unity APIs through type-safe bindings:

```rust
// VRChat APIs
let local_player = vrchat::Networking::local_player();
let all_players = vrchat::Networking::get_players();

// Unity APIs
let game_object = unity::GameObject::find("MyObject").unwrap();
let transform = game_object.transform();
transform.set_position(unity::Vector3::new(0.0, 1.0, 0.0));
```

## Chapter 4: Practical Exercise

### Exercise: Create a Player Greeter

Create a behavior that:
1. Displays a custom welcome message
2. Tracks the number of players who have joined
3. Shows different messages for VR vs Desktop users
4. Updates a UI text element with the current player count

#### Step 1: Define the Behavior

```rust
use udonsharp_core::prelude::*;
use udonsharp_bindings::{vrchat, unity};

#[derive(UdonBehaviour)]
pub struct PlayerGreeter {
    #[udon_public]
    pub welcome_message: String,
    
    #[udon_public]
    pub vr_bonus_message: String,
    
    #[udon_public]
    pub ui_text_object_name: String,
    
    total_joins: i32,
    vr_users: i32,
    desktop_users: i32,
}
```

#### Step 2: Implement the Logic

```rust
impl UdonBehaviour for PlayerGreeter {
    fn start(&mut self) {
        if self.welcome_message.is_empty() {
            self.welcome_message = "Welcome to VRChat!".to_string();
        }
        
        if self.vr_bonus_message.is_empty() {
            self.vr_bonus_message = "Enjoy the VR experience!".to_string();
        }
        
        self.update_ui();
        log::info!("PlayerGreeter initialized");
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.total_joins += 1;
        
        let player_name = player.get_display_name();
        let is_vr = player.is_user_in_vr();
        
        if is_vr {
            self.vr_users += 1;
            log::info!("VR player joined: {} - {}", player_name, self.vr_bonus_message);
        } else {
            self.desktop_users += 1;
            log::info!("Desktop player joined: {}", player_name);
        }
        
        log::info!("{}", self.welcome_message);
        self.update_ui();
    }
}

impl PlayerGreeter {
    fn update_ui(&self) {
        if let Some(ui_object) = unity::GameObject::find(&self.ui_text_object_name) {
            if let Some(text_component) = ui_object.get_component::<unity::UI::Text>() {
                let ui_text = format!(
                    "Total Joins: {}\nVR Users: {}\nDesktop Users: {}",
                    self.total_joins, self.vr_users, self.desktop_users
                );
                text_component.set_text(&ui_text);
            }
        }
    }
}
```

#### Step 3: Build and Test

1. Build the project: `cargo udonsharp build`
2. Copy generated files to Unity
3. Create a UI Text element named "PlayerStats"
4. Add the PlayerGreeter behavior to a GameObject
5. Set the UI text object name to "PlayerStats"
6. Test in Unity Play mode

## Chapter 5: Configuration and Customization

### Project Configuration

The `udonsharp.toml` file controls compilation settings:

```toml
[project]
name = "MyFirstWorld"
namespace = "MyWorld"
sync_mode = "None"

[compilation]
optimize_for_performance = false
generate_debug_info = true
target_udonsharp_version = "1.1"

[compilation.dev]
include_debug_comments = true
wasm_opt_level = 1

[compilation.release]
optimize_for_performance = true
generate_debug_info = false
wasm_opt_level = 3
```

### Build Profiles

Use different profiles for development vs production:

```bash
# Development build (faster, with debug info)
cargo udonsharp build

# Release build (optimized for performance)
cargo udonsharp build --release
```

## Chapter 6: Debugging and Troubleshooting

### Common Issues

#### Build Errors

**Error**: `wasm32-unknown-unknown target not found`
**Solution**: `rustup target add wasm32-unknown-unknown`

**Error**: `wasm-opt not found`
**Solution**: Install binaryen package for your platform

#### Unity Compilation Errors

**Error**: Generated C# has syntax errors
**Solution**: Check your Rust code for UdonSharp compatibility issues

**Error**: UdonSharp behavior not found
**Solution**: Ensure generated C# files are in the correct Assets directory

### Debugging Techniques

#### Logging

Use Rust's logging macros for debug output:

```rust
log::info!("Information message");
log::warn!("Warning message");
log::error!("Error message");
log::debug!("Debug message (only in debug builds)");
```

#### Unity Console

Check Unity's Console window for:
- Compilation errors
- Runtime log messages
- UdonSharp-specific warnings

#### VRChat Logs

In VRChat, check the output log for runtime messages and errors.

## Summary

In this tutorial, you learned:
- How to set up a Rust UdonSharp development environment
- The basic structure of a Rust UdonSharp project
- How to create and build your first UdonSharp behavior
- How to integrate generated C# code with Unity
- Basic debugging and troubleshooting techniques

## Next Steps

- **Tutorial 02**: Learn about networking and synchronization
- **Examples**: Try the [hello-world](../../hello-world/) and [player-counter](../../player-counter/) examples
- **Documentation**: Read the [API Reference](../../../docs/api-reference.md) for complete documentation

## Exercises for Practice

1. **Message Customizer**: Create a behavior that displays different messages based on the time of day
2. **Player Tracker**: Build a system that tracks player positions and shows them on a minimap
3. **Interactive Object**: Create an object that responds to player interactions with visual feedback

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust programming
- [VRChat Creator Documentation](https://creators.vrchat.com/) - Official VRChat world creation guide
- [UdonSharp Documentation](https://udonsharp.docs.vrchat.com/) - Original UdonSharp documentation