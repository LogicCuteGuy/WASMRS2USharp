# Implementation Plan

- [x] 1. Extend core attribute system for UdonBehaviour marking
  - Add `#[udon_behaviour]` attribute definition to udonsharp-macros crate
  - Implement attribute parameter parsing (name, events, dependencies)
  - Create validation logic for attribute configurations
  - _Requirements: 2.1, 2.2, 2.4_

- [x] 2. Implement WASM analysis and behavior identification
  - [x] 2.1 Extend WASM module parser to identify exported functions with attributes
    - Modify existing WASM parsing in wasm2usharp-enhanced to track function metadata
    - Create mapping between Rust function names and WASM exports
    - _Requirements: 1.1, 2.1_

  - [x] 2.2 Create BehaviorUnit data structure and identification logic
    - Define BehaviorUnit struct with name, functions, and dependencies
    - Implement logic to group functions into behavior units based on attributes
    - _Requirements: 1.1, 2.4_

  - [x] 2.3 Build function call graph analyzer
    - Implement call graph construction from WASM bytecode
    - Create dependency tracking between functions across behavior boundaries
    - _Requirements: 1.5, 4.1_

- [x] 3. Implement dependency analysis and validation
  - [x] 3.1 Create DependencyAnalyzer component
    - Build dependency graph data structure
    - Implement graph traversal algorithms for dependency detection
    - _Requirements: 4.1, 4.4_

  - [x] 3.2 Add circular dependency detection
    - Implement cycle detection algorithm in dependency graph
    - Generate clear error messages for circular dependencies
    - _Requirements: 4.4_

  - [x] 3.3 Create shared function identification logic
    - Analyze which functions should be moved to SharedRuntime
    - Implement optimization for minimizing code duplication
    - _Requirements: 5.1, 2.5_

- [x] 4. Extend code generation for multiple UdonBehaviour classes
  - [x] 4.1 Modify FileGenerator to support multiple output files
    - Extend existing file generation logic to create multiple .cs files
    - Implement naming conventions for generated UdonBehaviour classes
    - _Requirements: 1.1, 2.4_

  - [x] 4.2 Implement SharedRuntime class generation
    - Create SharedRuntime.cs template and generation logic
    - Generate utility functions and shared data structures
    - _Requirements: 1.2, 1.3, 5.1_

  - [x] 4.3 Add inter-behavior communication code generation
    - Generate method calls between different UdonBehaviour classes
    - Implement UdonSharp SendCustomEvent calls for cross-behavior communication
    - _Requirements: 3.1, 3.2_

  - [x] 4.4 Create Unity lifecycle method mapping
    - Map Rust functions to appropriate Unity event methods (Start, Update, etc.)
    - Generate proper UdonSharp method overrides
    - _Requirements: 2.3_

- [x] 5. Implement build system integration
  - [x] 5.1 Update compilation pipeline to handle multi-behavior projects
    - Modify existing pipeline in udonsharp-compiler to process behavior splitting
    - Integrate BehaviourSplitter into the compilation workflow
    - _Requirements: 1.1, 4.2_

  - [x] 5.2 Add Unity prefab generation for multiple behaviors
    - Generate Unity prefab files with appropriate component configurations
    - Set up component references and dependencies automatically
    - _Requirements: 1.4, 4.2, 4.3_

  - [x] 5.3 Implement initialization order management
    - Generate proper initialization sequence for dependent behaviors
    - Create startup coordination between multiple UdonBehaviour instances
    - _Requirements: 4.3_

- [x] 6. Add configuration and developer experience features
  - [x] 6.1 Extend udonsharp.toml configuration for multi-behavior settings
    - Add configuration options for behavior splitting preferences
    - Implement validation for multi-behavior project configurations
    - _Requirements: 1.1, 2.1_

  - [x] 6.2 Create example projects demonstrating multi-behavior usage
    - Build comprehensive example showing multiple UdonBehaviour classes
    - Create documentation and tutorials for the new feature
    - _Requirements: 1.1, 2.1, 3.1_

  - [x] 6.3 Add performance monitoring and optimization reporting
    - Generate reports on code sharing efficiency and behavior dependencies
    - Implement metrics collection for multi-behavior compilation
    - _Requirements: 5.5_

- [x] 7. Integration and validation
  - [x] 7.1 Update existing examples to demonstrate multi-behavior capability
    - Modify player-counter example to use multiple UdonBehaviour classes
    - Update game-manager example with behavior separation
    - _Requirements: 1.1, 2.1_

  - [x] 7.2 Implement error handling and user feedback
    - Add comprehensive error messages for multi-behavior compilation issues
    - Create helpful suggestions for resolving dependency problems
    - _Requirements: 4.4_

  - [x] 7.3 Create integration tests for multi-behavior compilation
    - Write tests covering end-to-end multi-behavior compilation
    - Test inter-behavior communication and dependency resolution
    - _Requirements: 1.1, 3.1, 4.1_