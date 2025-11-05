# Rust UdonSharp Examples

This directory contains example projects demonstrating various VRChat world development patterns using the Rust UdonSharp framework.

## Examples Overview

### Basic Examples

- **[hello-world](hello-world/)** - Simple "Hello World" example showing basic UdonSharp behavior
- **[player-counter](player-counter/)** - Tracks and displays player count with networking
- **[interactive-button](interactive-button/)** - Basic interaction system with UI feedback

### Intermediate Examples

- **[game-manager](game-manager/)** - Complete game state management system
- **[networking-demo](networking-demo/)** - Advanced networking patterns and synchronization
- **[audio-system](audio-system/)** - Audio management and spatial sound effects

### Advanced Examples

- **[world-portals](world-portals/)** - Portal system for world navigation
- **[inventory-system](inventory-system/)** - Complex inventory management with persistence
- **[mini-game-collection](mini-game-collection/)** - Multiple mini-games in one world

### Tutorial Series

- **[tutorial-01-basics](tutorials/01-basics/)** - Getting started with Rust UdonSharp
- **[tutorial-02-networking](tutorials/02-networking/)** - Understanding networking in VRChat
- **[tutorial-03-ui-systems](tutorials/03-ui-systems/)** - Building user interfaces
- **[tutorial-04-performance](tutorials/04-performance/)** - Performance optimization techniques

## Running Examples

Each example includes:
- Complete Rust source code
- Unity project files (when applicable)
- Build instructions
- Documentation explaining the concepts

To run an example:

1. Navigate to the example directory
2. Follow the README instructions for that example
3. Build using `cargo udonsharp build`
4. Import generated C# files into your Unity project

## Learning Path

**Beginners**: Start with `hello-world`, then `player-counter`, then `interactive-button`

**Intermediate**: Try `game-manager` and `networking-demo`

**Advanced**: Explore `world-portals` and `inventory-system`

**Performance Focus**: Study `tutorial-04-performance` and the optimization techniques used in advanced examples

## Contributing Examples

We welcome contributions of new examples! Please:

1. Follow the existing structure and naming conventions
2. Include comprehensive documentation
3. Test thoroughly in VRChat
4. Add your example to this README

## Support

If you have questions about any example:
- Check the example's README file
- Review the main [documentation](../docs/)
- Ask in the VRChat Discord #udonsharp channel