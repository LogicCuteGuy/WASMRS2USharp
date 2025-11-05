# Rust UdonSharp Framework

A Rust-based framework that provides UdonSharp-like object-oriented programming patterns for VRChat world development. Write Rust code with UdonSharp semantics, then compile through WebAssembly to generate UdonSharp-compatible C# code.

## 🚀 Quick Start

```bash
# Install the framework
cargo install cargo-udonsharp

# Create a new UdonSharp project
cargo udonsharp new my-vrchat-world

# Build your project
cd my-vrchat-world
cargo udonsharp build
```

## 📖 Documentation

- [Getting Started Guide](docs/getting-started.md) - Learn the basics of Rust UdonSharp development
- [API Reference](docs/api-reference.md) - Complete API documentation for all bindings
- [Compilation Pipeline](docs/compilation-pipeline.md) - Understanding the Rust → WASM → UdonSharp process
- [Best Practices](docs/best-practices.md) - Guidelines for effective UdonSharp development in Rust
- [Examples](examples/) - Sample projects and tutorials

## 🏗️ Architecture

This framework creates a "Rust#" experience - Rust syntax with UdonSharp semantics:

```
Rust Code → WASM → Enhanced wasm2usharp → UdonSharp C# → VRChat
```

### Key Features

- **Type-Safe VRChat APIs**: Rust bindings for VRChat SDK, Unity Engine, and C# system libraries
- **UdonSharp Traits**: Familiar UdonSharp patterns using Rust's trait system
- **Automatic Binding Generation**: Universal binding generator from .asmdef files
- **Performance Optimization**: WASM optimization pipeline for VRChat constraints
- **File Organization**: Intelligent C# file splitting and organization
- **Testing Framework**: Comprehensive testing tools for UdonSharp development

## 📦 Crates

| Crate | Description |
|-------|-------------|
| `udonsharp-core` | Core traits, types, and runtime helpers |
| `udonsharp-macros` | Procedural macros for UdonSharp attributes |
| `udonsharp-bindings` | Auto-generated API bindings (VRChat, Unity, C#) |
| `udonsharp-compiler` | Rust → WASM compilation pipeline |
| `udonsharp-cli` | Command-line interface and project management |
| `cargo-udonsharp` | Cargo subcommand integration |
| `udonsharp-build` | Build system and project templates |
| `wasm2usharp-enhanced` | Enhanced WASM → UdonSharp converter with OOP analysis |
| `udonsharp-performance` | Performance monitoring and optimization tools |

## 🎯 Example

```rust
use udonsharp::prelude::*;

#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct WorldController {
    #[udon_public]
    pub world_name: String,
    
    #[udon_sync]
    pub player_count: i32,
    
    initialized: bool,
}

impl UdonBehaviour for WorldController {
    fn start(&mut self) {
        self.initialized = true;
        self.world_name = "My Rust World".to_string();
        
        // Use VRChat APIs
        let local_player = vrchat::Networking::local_player();
        log::info!("World started by: {}", local_player.get_display_name());
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.player_count += 1;
        
        // Use Unity APIs
        let welcome_text = unity::GameObject::find("WelcomeText")
            .unwrap()
            .get_component::<unity::UI::Text>()
            .unwrap();
            
        welcome_text.set_text(&format!("Welcome {}! Players: {}", 
            player.get_display_name(), 
            self.player_count
        ));
    }
}
```

## 🛠️ Requirements

- Rust 1.70+ with `wasm32-unknown-unknown` target
- Unity 2022.3+ with VRChat SDK
- wasm-opt (for optimization)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## 🔗 Links

- [VRChat Creator Documentation](https://creators.vrchat.com/)
- [UdonSharp Documentation](https://udonsharp.docs.vrchat.com/)
- [Rust WebAssembly Book](https://rustwasm.github.io/docs/book/)