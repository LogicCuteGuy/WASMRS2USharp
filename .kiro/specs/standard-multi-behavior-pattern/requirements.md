# Requirements Document

## Introduction

This specification defines the standard pattern for using multiple `#[derive(UdonBehaviour)]` structs in a single WASM module for UdonSharp-Rust compilation. This pattern enables VRChat world developers to create complex, multi-system worlds using clean, type-safe Rust code that generates proper UdonSharp C# classes with full Unity integration.

## Glossary

- **UdonBehaviour_Struct**: A Rust struct annotated with `#[derive(UdonBehaviour)]` that represents a single UdonSharp behavior
- **Multi_Behavior_Module**: A single Rust WASM module containing multiple UdonBehaviour_Struct definitions
- **SharedRuntime**: A generated C# class containing shared functions and utilities used by multiple behaviors
- **Behavior_Communication**: The mechanism by which UdonBehaviour_Struct instances communicate via GameObject references and custom events
- **Field_Attribute**: Rust attributes like `#[udon_public]` and `#[udon_sync]` that control C# field generation
- **Custom_Event_Handler**: A method annotated with `#[udon_event]` that handles inter-behavior communication
- **Compilation_Pipeline**: The system that processes Multi_Behavior_Module and generates UdonSharp C# classes

## Requirements

### Requirement 1

**User Story:** As a VRChat world developer, I want to define multiple UdonBehaviour structs in a single Rust file, so that I can organize related behaviors together and maintain clean project structure.

#### Acceptance Criteria

1. WHEN a Multi_Behavior_Module contains multiple structs with `#[derive(UdonBehaviour)]`, THE Compilation_Pipeline SHALL generate separate C# classes for each struct
2. WHEN a UdonBehaviour_Struct is defined, THE Compilation_Pipeline SHALL validate that the struct name follows valid C# class naming conventions
3. WHEN a Multi_Behavior_Module contains at least 2 UdonBehaviour_Struct definitions, THE Compilation_Pipeline SHALL generate a SharedRuntime class
4. IF a UdonBehaviour_Struct name conflicts with existing Unity or UdonSharp classes, THEN THE Compilation_Pipeline SHALL generate a compilation error with suggested alternatives
5. WHEN processing a Multi_Behavior_Module, THE Compilation_Pipeline SHALL preserve the original struct names as C# class names

### Requirement 2

**User Story:** As a Rust developer, I want each UdonBehaviour struct to implement the UdonBehaviour trait, so that I have compile-time type safety and clear method interfaces.

#### Acceptance Criteria

1. WHEN a struct has `#[derive(UdonBehaviour)]`, THE Compilation_Pipeline SHALL require implementation of the UdonBehaviour trait
2. WHEN a UdonBehaviour_Struct implements the UdonBehaviour trait, THE Compilation_Pipeline SHALL generate corresponding UdonSharp event methods
3. WHEN the UdonBehaviour trait method `start()` is implemented, THE Compilation_Pipeline SHALL generate a C# `Start()` method
4. WHEN the UdonBehaviour trait method `update()` is implemented, THE Compilation_Pipeline SHALL generate a C# `Update()` method
5. WHEN the UdonBehaviour trait method `on_player_joined()` is implemented, THE Compilation_Pipeline SHALL generate a C# `OnPlayerJoined()` method

### Requirement 3

**User Story:** As a Unity developer, I want struct fields to be properly converted to C# fields with correct attributes, so that I can configure behaviors in the Unity Inspector and use VRChat networking.

#### Acceptance Criteria

1. WHEN a field has `#[udon_public]` attribute, THE Compilation_Pipeline SHALL generate a C# field with `[SerializeField]` and `public` visibility
2. WHEN a field has `#[udon_sync]` attribute, THE Compilation_Pipeline SHALL generate a C# field with `[UdonSynced]` attribute
3. WHEN a struct has `#[udon_sync_mode(Manual)]` attribute, THE Compilation_Pipeline SHALL generate a C# class with manual sync mode configuration
4. WHEN a field has no attribute, THE Compilation_Pipeline SHALL generate a private C# field
5. WHEN a field type is not supported by UdonSharp, THE Compilation_Pipeline SHALL generate a compilation error with supported alternatives

### Requirement 4

**User Story:** As a VRChat world developer, I want behaviors to communicate through GameObject references and custom events, so that I can create complex interactions between different systems.

#### Acceptance Criteria

1. WHEN a UdonBehaviour_Struct contains `Option<unity::GameObject>` fields, THE Compilation_Pipeline SHALL generate C# GameObject reference fields
2. WHEN a method calls `send_custom_event()` on a GameObject reference, THE Compilation_Pipeline SHALL generate appropriate UdonSharp SendCustomEvent calls
3. WHEN a method is annotated with `#[udon_event("EventName")]`, THE Compilation_Pipeline SHALL generate a public C# method that can receive custom events
4. WHEN behaviors reference each other via `unity::GameObject::find()`, THE Compilation_Pipeline SHALL generate appropriate GameObject.Find() calls
5. WHEN Behavior_Communication occurs between UdonBehaviour_Struct instances, THE Compilation_Pipeline SHALL ensure proper event parameter handling

### Requirement 5

**User Story:** As a Rust developer, I want shared functions and utilities to be automatically moved to a SharedRuntime class, so that I avoid code duplication and maintain clean separation of concerns.

#### Acceptance Criteria

1. WHEN multiple UdonBehaviour_Struct instances use the same public function, THE Compilation_Pipeline SHALL move that function to SharedRuntime
2. WHEN a Multi_Behavior_Module contains shared enums or structs, THE Compilation_Pipeline SHALL include them in SharedRuntime
3. WHEN SharedRuntime is generated, THE Compilation_Pipeline SHALL ensure all UdonBehaviour_Struct classes can access shared functionality
4. WHEN a function is moved to SharedRuntime, THE Compilation_Pipeline SHALL update all references in generated UdonBehaviour classes
5. WHEN SharedRuntime contains static variables, THE Compilation_Pipeline SHALL generate thread-safe access patterns

### Requirement 6

**User Story:** As a VRChat world developer, I want network synchronization to work seamlessly across multiple behaviors, so that I can create multiplayer experiences with proper state management.

#### Acceptance Criteria

1. WHEN a UdonBehaviour_Struct has `#[udon_sync]` fields, THE Compilation_Pipeline SHALL generate proper RequestSerialization() calls
2. WHEN a UdonBehaviour_Struct implements `on_post_deserialization()`, THE Compilation_Pipeline SHALL generate OnDeserialization() method
3. WHEN multiple UdonBehaviour_Struct instances have synchronized fields, THE Compilation_Pipeline SHALL ensure proper master client validation
4. WHEN network synchronization occurs, THE Compilation_Pipeline SHALL generate appropriate VRChat networking API calls
5. WHEN a UdonBehaviour_Struct modifies synchronized data, THE Compilation_Pipeline SHALL ensure only master client can make changes

### Requirement 7

**User Story:** As a Rust developer, I want comprehensive error handling and validation during compilation, so that I can identify and fix issues before deploying to VRChat.

#### Acceptance Criteria

1. WHEN a UdonBehaviour_Struct is missing required trait implementation, THE Compilation_Pipeline SHALL generate a clear error message with implementation guidance
2. WHEN field attributes are used incorrectly, THE Compilation_Pipeline SHALL generate specific error messages with correct usage examples
3. WHEN circular dependencies exist between UdonBehaviour_Struct instances, THE Compilation_Pipeline SHALL detect and report the dependency cycle
4. WHEN unsupported Rust features are used, THE Compilation_Pipeline SHALL generate errors with UdonSharp-compatible alternatives
5. WHEN compilation succeeds, THE Compilation_Pipeline SHALL generate a summary report of created behaviors and shared functionality

### Requirement 8

**User Story:** As a Unity developer, I want the generated C# code to follow UdonSharp conventions and best practices, so that I can easily integrate it into my Unity project and debug issues.

#### Acceptance Criteria

1. WHEN C# classes are generated, THE Compilation_Pipeline SHALL follow standard UdonSharp naming conventions and code structure
2. WHEN UdonBehaviour methods are generated, THE Compilation_Pipeline SHALL include proper null checks and error handling
3. WHEN GameObject references are used, THE Compilation_Pipeline SHALL generate safe reference validation code
4. WHEN custom events are handled, THE Compilation_Pipeline SHALL generate proper parameter validation and type checking
5. WHEN debugging information is needed, THE Compilation_Pipeline SHALL generate C# code with clear comments indicating original Rust source locations