# Getting Started with Rust UdonSharp

This guide will help you get started with the Rust UdonSharp framework for VRChat world development.

## Prerequisites

Before you begin, ensure you have the following installed:

1. **Rust 1.70+** with the WebAssembly target:
   ```bash
   rustup install stable
   rustup target add wasm32-unknown-unknown
   ```

2. **Unity 2022.3+** with VRChat SDK3 Worlds
   - Download from [Unity Hub](https://unity.com/download)
   - Install VRChat SDK from [VRChat Creator Companion](https://vcc.docs.vrchat.com/)

3. **wasm-opt** (for optimization):
   ```bash
   # On macOS with Homebrew
   brew install binaryen
   
   # On Ubuntu/Debian
   sudo apt install binaryen
   
   # On Windows, download from https://github.com/WebAssembly/binaryen/releases
   ```

## Installation

Install the Rust UdonSharp framework:

```bash
cargo install cargo-udonsharp
```

Verify the installation:

```bash
cargo udonsharp --version
```

## Creating Your First Project

### 1. Create a New Project

```bash
cargo udonsharp new my-first-world
cd my-first-world
```

This creates a new Rust UdonSharp project with the following structure:

```
my-first-world/
├── Cargo.toml          # Project configuration
├── build.rs            # Build script for UdonSharp compilation
├── src/
│   └── lib.rs          # Main UdonSharp behavior
└── udonsharp.toml      # UdonSharp-specific configuration
```

### 2. Understanding the Project Structure

**Cargo.toml**: Standard Rust project configuration with UdonSharp dependencies:

```toml
[package]
name = "my-first-world"
version = "0.1.0"
edition = "2021"

[dependencies]
udonsharp = "0.1"

[lib]
crate-type = ["cdylib"]

[[bin]]
name = "udonsharp-build"
path = "build.rs"
```

**src/lib.rs**: Your main UdonSharp behavior:

```rust
use udonsharp::prelude::*;

#[derive(UdonBehaviour)]
pub struct MyFirstWorld {
    #[udon_public]
    pub message: String,
    
    player_count: i32,
}

impl UdonBehaviour for MyFirstWorld {
    fn start(&mut self) {
        self.message = "Hello from Rust!".to_string();
        log::info!("World initialized: {}", self.message);
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.player_count += 1;
        log::info!("Player joined: {} (Total: {})", 
            player.get_display_name(), 
            self.player_count
        );
    }
}
```

**udonsharp.toml**: Framework-specific configuration:

```toml
[project]
name = "MyFirstWorld"
namespace = "MyVRChatWorld"
sync_mode = "Manual"

[compilation]
optimize_for_performance = true
generate_debug_info = false
target_udonsharp_version = "1.1"

[bindings]
# Automatically scan Unity project for .asmdef files
auto_scan_unity_project = true
unity_project_path = "../MyUnityProject"

# Additional binding directories
binding_directories = [
    "Assets/VRChat/SDK",
    "Packages/com.vrchat.udonsharp"
]
```

### 3. Building Your Project

Build the project to generate UdonSharp C# files:

```bash
cargo udonsharp build
```

This process:
1. Compiles your Rust code to WebAssembly
2. Optimizes the WASM with wasm-opt
3. Converts WASM to UdonSharp C# using enhanced wasm2usharp
4. Applies OOP behavior analysis and file splitting
5. Generates organized C# files in the `output/` directory

### 4. Integration with Unity

After building, you'll find generated C# files in the `output/` directory:

```
output/
├── MyFirstWorld.cs           # Main UdonSharp behavior
├── MyFirstWorld_Generated.cs # Generated helper classes
└── bindings/
    ├── VRChatBindings.cs     # VRChat API bindings
    ├── UnityBindings.cs      # Unity API bindings
    └── SystemBindings.cs     # C# system bindings
```

Copy these files to your Unity project's Assets folder:

```bash
cp -r output/* /path/to/your/unity/project/Assets/Scripts/
```

### 5. Setting Up in Unity

1. **Create an Empty GameObject** in your scene
2. **Add the UdonSharp Behaviour** component
3. **Assign your generated script** (MyFirstWorld.cs)
4. **Configure public fields** in the inspector
5. **Build and test** your world

## Core Concepts

### UdonBehaviour Trait

The `UdonBehaviour` trait provides the standard UdonSharp lifecycle methods:

```rust
impl UdonBehaviour for MyBehaviour {
    fn start(&mut self) {
        // Called when the behavior starts
    }
    
    fn update(&mut self) {
        // Called every frame
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        // Called when a player joins
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        // Called when a player leaves
    }
}
```

### UdonSharp Attributes

Use attributes to control UdonSharp behavior:

```rust
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]  // Networking sync mode
pub struct MyBehaviour {
    #[udon_public]         // Visible in Unity inspector
    pub public_field: String,
    
    #[udon_sync]           // Synchronized across network
    pub synced_field: i32,
    
    private_field: bool,   // Private, not synchronized
}
```

### API Bindings

Access VRChat, Unity, and C# APIs through type-safe bindings:

```rust
use udonsharp::prelude::*;

// VRChat APIs
let local_player = vrchat::Networking::local_player();
let all_players = vrchat::Networking::get_players();

// Unity APIs
let game_object = unity::GameObject::find("MyObject").unwrap();
let transform = game_object.transform();
transform.set_position(unity::Vector3::new(0.0, 1.0, 0.0));

// C# System APIs
let mut list = cs_sys::collections::List::<String>::new();
list.add("Hello".to_string());
list.add("World".to_string());
```

## Development Workflow

### 1. Write Rust Code

Develop your UdonSharp behaviors using familiar Rust patterns:

```rust
use udonsharp::prelude::*;

#[derive(UdonBehaviour)]
pub struct InteractableButton {
    #[udon_public]
    pub button_text: String,
    
    #[udon_public] 
    pub target_object: Option<unity::GameObject>,
    
    click_count: i32,
}

impl UdonBehaviour for InteractableButton {
    fn start(&mut self) {
        if self.button_text.is_empty() {
            self.button_text = "Click Me!".to_string();
        }
    }
}

impl InteractableButton {
    #[udon_event("OnInteract")]
    pub fn on_interact(&mut self) {
        self.click_count += 1;
        
        log::info!("Button clicked {} times", self.click_count);
        
        if let Some(target) = &self.target_object {
            target.set_active(!target.active_self());
        }
    }
}
```

### 2. Test Locally

Run tests using the built-in testing framework:

```bash
cargo udonsharp test
```

### 3. Build and Deploy

Build for production:

```bash
cargo udonsharp build --release
```

### 4. Integration Testing

Test in Unity using ClientSim or upload to VRChat for testing.

## Next Steps

- Read the [API Reference](api-reference.md) for complete documentation
- Explore [Examples](../examples/) for common patterns
- Learn about the [Compilation Pipeline](compilation-pipeline.md)
- Follow [Best Practices](best-practices.md) for optimal development

## Troubleshooting

### Common Issues

**Build fails with WASM target not found:**
```bash
rustup target add wasm32-unknown-unknown
```

**wasm-opt not found:**
Install binaryen package for your platform.

**Unity compilation errors:**
Ensure generated C# files are in the correct Assets directory and UdonSharp is properly installed.

**Performance issues:**
Use `cargo udonsharp build --release` and review the [Performance Guide](performance.md).

## Getting Help

- Check the [FAQ](faq.md)
- Browse [Examples](../examples/)
- Join the [VRChat Discord](https://discord.gg/vrchat) #udonsharp channel
- Report issues on [GitHub](https://github.com/your-repo/rust-udonsharp)