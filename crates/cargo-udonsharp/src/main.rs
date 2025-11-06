//! Cargo subcommand for UdonSharp compilation
//! 
//! This provides `cargo udonsharp` command for seamless integration
//! with Rust development workflow.

use clap::{Parser, Subcommand};
use udonsharp_core::{UdonSharpResult, error::CompilationContext};
use udonsharp_compiler::{CompilationPipeline, UdonSharpConfig};
use udonsharp_bindings::UniversalBindingPipeline;
use std::path::Path;
use std::env;
use log::{info, warn, error};

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    #[command(name = "udonsharp")]
    UdonSharp(UdonSharpArgs),
}

#[derive(Parser)]
#[command(name = "udonsharp")]
#[command(version)]
#[command(about = "Compile Rust code to UdonSharp")]
#[command(long_about = "Cargo subcommand for compiling Rust projects to UdonSharp-compatible C# for VRChat world development")]
struct UdonSharpArgs {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Enable quiet mode (suppress non-error output)
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    quiet: bool,
    
    #[command(subcommand)]
    command: Option<UdonSharpCommand>,
}

#[derive(Subcommand)]
enum UdonSharpCommand {
    /// Build the current project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
        /// Generate debug information
        #[arg(long)]
        debug: bool,
        /// Target directory for output
        #[arg(long)]
        target_dir: Option<String>,
        /// Show compilation progress
        #[arg(long)]
        progress: bool,
        /// Build all workspace members
        #[arg(long)]
        workspace: bool,
        /// Package to build (for workspace)
        #[arg(short, long)]
        package: Option<String>,
    },
    /// Generate API bindings from .asmdef files
    Bindings {
        /// Directory to scan for .asmdef files
        #[arg(long)]
        scan_dir: Vec<String>,
        /// Output directory for generated bindings
        #[arg(long)]
        output: String,
        /// Force regeneration of existing bindings
        #[arg(long)]
        force: bool,
        /// Show binding generation progress
        #[arg(long)]
        progress: bool,
        /// Generate bindings for all workspace members
        #[arg(long)]
        workspace: bool,
    },
    /// Check project for errors without building
    Check {
        /// Check in release mode
        #[arg(long)]
        release: bool,
        /// Show detailed diagnostics
        #[arg(long)]
        detailed: bool,
        /// Check all workspace members
        #[arg(long)]
        workspace: bool,
        /// Package to check (for workspace)
        #[arg(short, long)]
        package: Option<String>,
    },
    /// Clean build artifacts
    Clean {
        /// Target directory to clean
        #[arg(long)]
        target_dir: Option<String>,
        /// Remove all generated files including bindings
        #[arg(long)]
        all: bool,
        /// Clean all workspace members
        #[arg(long)]
        workspace: bool,
    },
    /// Create a new UdonSharp project
    New {
        /// Project name
        name: String,
        /// Project template to use
        #[arg(long, default_value = "basic")]
        template: String,
        /// Initialize as a workspace
        #[arg(long)]
        workspace: bool,
        /// Include example files
        #[arg(long)]
        examples: bool,
        /// Include test files
        #[arg(long)]
        tests: bool,
        /// Include documentation
        #[arg(long, default_value = "true")]
        docs: bool,
        /// VRChat SDK path (optional)
        #[arg(long)]
        vrc_sdk_path: Option<String>,
        /// Unity project path (optional)
        #[arg(long)]
        unity_project_path: Option<String>,
        /// Custom features to include
        #[arg(long)]
        features: Vec<String>,
    },
    /// Initialize UdonSharp in an existing Rust project
    Init {
        /// Project template to use
        #[arg(long, default_value = "basic")]
        template: String,
        /// VRChat SDK path (optional)
        #[arg(long)]
        vrc_sdk_path: Option<String>,
        /// Unity project path (optional)
        #[arg(long)]
        unity_project_path: Option<String>,
    },
    /// List available project templates
    Templates,
}

#[tokio::main]
async fn main() -> UdonSharpResult<()> {
    let CargoCli::UdonSharp(args) = CargoCli::parse();
    
    // Initialize logging
    init_logging(&args)?;
    
    // If no subcommand is provided, default to build
    let command = args.command.unwrap_or(UdonSharpCommand::Build {
        release: false,
        debug: false,
        target_dir: None,
        progress: false,
        workspace: false,
        package: None,
    });
    
    match command {
        UdonSharpCommand::Build { release, debug, target_dir, progress, workspace, package } => {
            handle_build_command(release, debug, target_dir, progress, workspace, package).await
        }
        UdonSharpCommand::Bindings { scan_dir, output, force, progress, workspace } => {
            handle_bindings_command(scan_dir, output, force, progress, workspace).await
        }
        UdonSharpCommand::Check { release, detailed, workspace, package } => {
            handle_check_command(release, detailed, workspace, package).await
        }
        UdonSharpCommand::Clean { target_dir, all, workspace } => {
            handle_clean_command(target_dir, all, workspace).await
        }
        UdonSharpCommand::New { name, template, workspace, examples, tests, docs, vrc_sdk_path, unity_project_path, features } => {
            handle_new_command(name, template, workspace, examples, tests, docs, vrc_sdk_path, unity_project_path, features).await
        }
        UdonSharpCommand::Init { template, vrc_sdk_path, unity_project_path } => {
            handle_init_command(template, vrc_sdk_path, unity_project_path).await
        }
        UdonSharpCommand::Templates => {
            handle_templates_command().await
        }
    }
}

fn init_logging(args: &UdonSharpArgs) -> UdonSharpResult<()> {
    let log_level = if args.quiet {
        log::LevelFilter::Error
    } else if args.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .format_timestamp_secs()
        .init();
    
    Ok(())
}

async fn handle_build_command(
    release: bool,
    debug: bool,
    target_dir: Option<String>,
    progress: bool,
    workspace: bool,
    package: Option<String>,
) -> UdonSharpResult<()> {
    info!("Building UdonSharp project...");
    
    // Detect if we're in a Cargo workspace
    let manifest_path = find_cargo_manifest()?;
    let project_dir = manifest_path.parent().unwrap();
    
    if workspace {
        return handle_workspace_build(project_dir, release, debug, target_dir, progress, package).await;
    }
    
    if progress {
        info!("Building project with progress reporting...");
    } else {
        info!("Building project (release: {}, debug: {})", release, debug);
    }
    
    let context = if debug {
        CompilationContext::debug()
    } else {
        CompilationContext::new()
    };
    
    // Create configuration
    let mut config = UdonSharpConfig::default();
    config.optimize_for_performance = release;
    config.generate_debug_info = debug;
    
    if let Some(target) = target_dir {
        config.output_directory = Some(target);
    }
    
    // Create compilation pipeline
    let pipeline = CompilationPipeline::with_context(config, context);
    
    // Compile the project
    let result = pipeline.compile_project(project_dir).await?;
    
    pipeline.context().print_summary();
    
    if result.success {
        println!("✅ Build completed successfully!");
        if !result.output_files.is_empty() {
            println!("📄 Generated files:");
            for file in &result.output_files {
                println!("   {}", file);
            }
        }
    } else {
        error!("Build failed");
        return Err(udonsharp_core::UdonSharpError::compilation("Build failed"));
    }
    
    Ok(())
}

async fn handle_bindings_command(
    scan_dirs: Vec<String>,
    output: String,
    _force: bool,
    progress: bool,
    workspace: bool,
) -> UdonSharpResult<()> {
    info!("Generating API bindings...");
    
    if progress {
        println!("🔍 Scanning directories for .asmdef files...");
    }
    
    // Create output directory if it doesn't exist
    let output_path = Path::new(&output);
    if !output_path.exists() {
        std::fs::create_dir_all(output_path).map_err(|e| {
            udonsharp_core::UdonSharpError::configuration(format!("Failed to create output directory: {}", e))
        })?;
    }
    
    // Initialize binding pipeline
    let mut pipeline = UniversalBindingPipeline::new(output.clone());
    
    // Add scan directories or use default Unity paths
    if scan_dirs.is_empty() {
        // Try to detect Unity project paths automatically
        let unity_paths = detect_unity_paths()?;
        for path in unity_paths {
            pipeline.add_asmdef_directory(path.clone());
            if progress {
                println!("   Added detected Unity path: {}", path);
            }
        }
    } else {
        for dir in scan_dirs {
            if Path::new(&dir).exists() {
                pipeline.add_asmdef_directory(dir.clone());
                if progress {
                    println!("   Added scan directory: {}", dir);
                }
            } else {
                warn!("Scan directory does not exist: {}", dir);
            }
        }
    }
    
    // Generate bindings
    if progress {
        println!("🔧 Generating bindings...");
    }
    
    pipeline.scan_and_generate_all_bindings().map_err(|e| {
        udonsharp_core::UdonSharpError::compilation(format!("Failed to generate bindings: {}", e))
    })?;
    
    println!("✅ Successfully generated API bindings");
    println!("📁 Output directory: {}", output);
    
    Ok(())
}

async fn handle_check_command(release: bool, detailed: bool, workspace: bool, package: Option<String>) -> UdonSharpResult<()> {
    info!("Checking project for errors...");
    
    let manifest_path = find_cargo_manifest()?;
    let project_dir = manifest_path.parent().unwrap();
    
    let context = CompilationContext::new();
    let mut config = UdonSharpConfig::default();
    config.optimize_for_performance = release;
    config.check_only = true;
    
    let pipeline = CompilationPipeline::with_context(config, context);
    let result = pipeline.check_project(project_dir).await?;
    
    if detailed {
        pipeline.context().print_detailed_diagnostics();
    } else {
        pipeline.context().print_summary();
    }
    
    if result.success {
        println!("✅ No errors found");
    } else {
        error!("Check failed - errors found");
        return Err(udonsharp_core::UdonSharpError::compilation("Check failed"));
    }
    
    Ok(())
}

async fn handle_clean_command(target_dir: Option<String>, all: bool, workspace: bool) -> UdonSharpResult<()> {
    info!("Cleaning build artifacts...");
    
    let target_path = target_dir.unwrap_or_else(|| "target".to_string());
    let target_dir = Path::new(&target_path);
    
    if target_dir.exists() {
        std::fs::remove_dir_all(target_dir).map_err(|e| {
            udonsharp_core::UdonSharpError::configuration(format!("Failed to clean target directory: {}", e))
        })?;
        println!("🧹 Cleaned target directory: {}", target_path);
    }
    
    if all {
        // Clean additional generated files
        let bindings_dir = Path::new("generated");
        if bindings_dir.exists() {
            std::fs::remove_dir_all(bindings_dir).map_err(|e| {
                udonsharp_core::UdonSharpError::configuration(format!("Failed to clean bindings directory: {}", e))
            })?;
            println!("🧹 Cleaned generated bindings directory");
        }
    }
    
    println!("✅ Clean completed successfully");
    Ok(())
}

fn find_cargo_manifest() -> UdonSharpResult<std::path::PathBuf> {
    let current_dir = env::current_dir().map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to get current directory: {}", e))
    })?;
    
    let mut dir = current_dir.as_path();
    loop {
        let manifest_path = dir.join("Cargo.toml");
        if manifest_path.exists() {
            return Ok(manifest_path);
        }
        
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    
    Err(udonsharp_core::UdonSharpError::configuration(
        "Could not find Cargo.toml in current directory or any parent directory"
    ))
}

fn detect_unity_paths() -> UdonSharpResult<Vec<String>> {
    let mut paths = Vec::new();
    
    // Common Unity project paths
    let potential_paths = [
        "Library/PackageCache",
        "Assets",
        "Packages",
        "../Library/PackageCache",
        "../Assets", 
        "../Packages",
        "../../Library/PackageCache",
        "../../Assets",
        "../../Packages",
    ];
    
    for path in &potential_paths {
        if Path::new(path).exists() {
            paths.push(path.to_string());
        }
    }
    
    // If no Unity paths found, provide helpful message
    if paths.is_empty() {
        info!("No Unity project paths detected. You may need to specify --scan-dir manually.");
        info!("Common Unity paths include:");
        info!("  - Library/PackageCache (for installed packages)");
        info!("  - Assets (for project assets)");
        info!("  - Packages (for local packages)");
    }
    
    Ok(paths)
}

async fn handle_workspace_build(
    workspace_dir: &Path,
    release: bool,
    debug: bool,
    target_dir: Option<String>,
    progress: bool,
    package: Option<String>,
) -> UdonSharpResult<()> {
    info!("Building UdonSharp workspace...");
    
    let workspace_members = discover_workspace_members(workspace_dir)?;
    
    let members_to_build = if let Some(pkg) = package {
        // Build specific package
        if workspace_members.contains(&pkg) {
            vec![pkg]
        } else {
            return Err(udonsharp_core::UdonSharpError::configuration(
                format!("Package '{}' not found in workspace", pkg)
            ));
        }
    } else {
        // Build all members
        workspace_members
    };
    
    let mut all_success = true;
    let mut total_files = Vec::new();
    
    for member in members_to_build {
        println!("🔨 Building workspace member: {}", member);
        
        let member_dir = workspace_dir.join(&member);
        if !member_dir.exists() {
            warn!("Workspace member directory not found: {}", member);
            continue;
        }
        
        let context = if debug {
            CompilationContext::debug()
        } else {
            CompilationContext::new()
        };
        
        let mut config = UdonSharpConfig::default();
        config.optimize_for_performance = release;
        config.generate_debug_info = debug;
        
        if let Some(ref target) = target_dir {
            config.output_directory = Some(format!("{}/{}", target, member));
        }
        
        let pipeline = CompilationPipeline::with_context(config, context);
        
        match pipeline.compile_project(&member_dir).await {
            Ok(result) => {
                if result.success {
                    println!("✅ {} built successfully", member);
                    total_files.extend(result.output_files);
                } else {
                    println!("❌ {} build failed", member);
                    all_success = false;
                }
            }
            Err(e) => {
                error!("Failed to build {}: {}", member, e);
                all_success = false;
            }
        }
    }
    
    if all_success {
        println!("✅ All workspace members built successfully!");
        if !total_files.is_empty() {
            println!("📄 Total generated files: {}", total_files.len());
        }
    } else {
        return Err(udonsharp_core::UdonSharpError::compilation("Some workspace members failed to build"));
    }
    
    Ok(())
}

async fn handle_new_command(
    name: String,
    template: String,
    workspace: bool,
    examples: bool,
    tests: bool,
    docs: bool,
    vrc_sdk_path: Option<String>,
    unity_project_path: Option<String>,
    features: Vec<String>,
) -> UdonSharpResult<()> {
    info!("Creating new UdonSharp project: {}", name);
    
    let project_dir = Path::new(&name);
    if project_dir.exists() {
        return Err(udonsharp_core::UdonSharpError::configuration(
            format!("Directory '{}' already exists", name)
        ));
    }
    
    // Create project directory
    std::fs::create_dir_all(project_dir).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to create project directory: {}", e))
    })?;
    
    // Create scaffolding configuration
    let template_type = udonsharp_build::ProjectTemplate::from_str(&template).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Invalid template: {}", e))
    })?;
    
    let config = udonsharp_build::ScaffoldingConfig {
        template: template_type,
        project_name: name.clone(),
        workspace,
        include_examples: examples,
        include_tests: tests,
        include_documentation: docs,
        vrc_sdk_path,
        unity_project_path,
        custom_features: features,
    };
    
    // Create project with configuration
    udonsharp_build::create_project_with_config(project_dir, &config).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to create project: {}", e))
    })?;
    
    println!("✅ Created UdonSharp project: {}", name);
    println!("📁 Project directory: {}", project_dir.display());
    println!("🎯 Template: {} ({})", config.template.name(), config.template.description());
    
    if !config.template.features().is_empty() {
        println!("🔧 Features:");
        for feature in config.template.features() {
            println!("   - {}", feature);
        }
    }
    
    if config.include_examples {
        println!("📚 Examples included in examples/ directory");
    }
    
    if config.include_tests {
        println!("🧪 Tests included in tests/ directory");
    }
    
    if config.include_documentation {
        println!("📖 Documentation included in docs/ directory");
    }
    
    println!("🚀 To get started:");
    println!("   cd {}", name);
    println!("   cargo udonsharp build");
    
    Ok(())
}

async fn handle_init_command(
    template: String,
    vrc_sdk_path: Option<String>,
    unity_project_path: Option<String>,
) -> UdonSharpResult<()> {
    info!("Initializing UdonSharp in existing project...");
    
    let current_dir = env::current_dir().map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to get current directory: {}", e))
    })?;
    
    // Check if Cargo.toml exists
    let cargo_toml = current_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(udonsharp_core::UdonSharpError::configuration(
            "No Cargo.toml found. Run this command in a Rust project directory."
        ));
    }
    
    // Initialize UdonSharp configuration
    init_udonsharp_project(&current_dir, &template, vrc_sdk_path, unity_project_path)?;
    
    println!("✅ Initialized UdonSharp in current project");
    println!("🚀 To build your project:");
    println!("   cargo udonsharp build");
    
    Ok(())
}

fn discover_workspace_members(workspace_dir: &Path) -> UdonSharpResult<Vec<String>> {
    let cargo_toml = workspace_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Ok(Vec::new());
    }
    
    let content = std::fs::read_to_string(&cargo_toml).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to read Cargo.toml: {}", e))
    })?;
    
    // Parse Cargo.toml to find workspace members
    let toml_value: toml::Value = content.parse().map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to parse Cargo.toml: {}", e))
    })?;
    
    let mut members = Vec::new();
    
    if let Some(workspace) = toml_value.get("workspace") {
        if let Some(workspace_members) = workspace.get("members") {
            if let Some(members_array) = workspace_members.as_array() {
                for member in members_array {
                    if let Some(member_str) = member.as_str() {
                        // Extract just the directory name for display
                        let member_name = Path::new(member_str)
                            .file_name()
                            .unwrap_or_else(|| std::ffi::OsStr::new(member_str))
                            .to_string_lossy()
                            .to_string();
                        members.push(member_name);
                    }
                }
            }
        }
    }
    
    Ok(members)
}

fn create_workspace_project(
    name: &str,
    template: &str,
    vrc_sdk_path: Option<String>,
    unity_project_path: Option<String>,
) -> UdonSharpResult<()> {
    let project_dir = Path::new(name);
    
    // Create workspace Cargo.toml
    let workspace_cargo_toml = format!(
        r#"[workspace]
members = [
    "src"
]
resolver = "2"

[workspace.dependencies]
udonsharp-core = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}
udonsharp-macros = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}
udonsharp-bindings = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
log = "0.4"

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"

[profile.dev]
opt-level = 1
debug = false
panic = "abort"
"#
    );
    
    std::fs::write(project_dir.join("Cargo.toml"), workspace_cargo_toml).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to write workspace Cargo.toml: {}", e))
    })?;
    
    // Create src directory and project
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to create src directory: {}", e))
    })?;
    
    create_project_files(&src_dir, name, template, vrc_sdk_path, unity_project_path)?;
    
    // Create README.md
    let readme_content = format!(
        r#"# {}

A UdonSharp project written in Rust.

## Building

```bash
cargo udonsharp build
```

## Project Structure

- `src/` - Main UdonSharp project
- `generated/` - Auto-generated API bindings (created during build)

## Development

This project uses the Rust UdonSharp framework to compile Rust code to UdonSharp-compatible C# for VRChat world development.

For more information, see the [Rust UdonSharp documentation](https://github.com/vrchat-community/rust-udonsharp).
"#,
        name
    );
    
    std::fs::write(project_dir.join("README.md"), readme_content).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to write README.md: {}", e))
    })?;
    
    Ok(())
}

fn create_single_project(
    name: &str,
    template: &str,
    vrc_sdk_path: Option<String>,
    unity_project_path: Option<String>,
) -> UdonSharpResult<()> {
    let project_dir = Path::new(name);
    create_project_files(project_dir, name, template, vrc_sdk_path, unity_project_path)
}

fn create_project_files(
    project_dir: &Path,
    name: &str,
    template: &str,
    vrc_sdk_path: Option<String>,
    unity_project_path: Option<String>,
) -> UdonSharpResult<()> {
    // Create Cargo.toml
    let cargo_toml_content = create_cargo_toml_content(name, template)?;
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml_content).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to write Cargo.toml: {}", e))
    })?;
    
    // Create build.rs
    let build_rs_content = create_build_rs_content(vrc_sdk_path, unity_project_path)?;
    std::fs::write(project_dir.join("build.rs"), build_rs_content).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to write build.rs: {}", e))
    })?;
    
    // Create src directory
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to create src directory: {}", e))
    })?;
    
    // Create lib.rs based on template
    let lib_rs_content = create_lib_rs_content(template)?;
    std::fs::write(src_dir.join("lib.rs"), lib_rs_content).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to write lib.rs: {}", e))
    })?;
    
    // Create .gitignore
    let gitignore_content = r#"/target
/generated
Cargo.lock
*.wasm
*.cs
"#;
    std::fs::write(project_dir.join(".gitignore"), gitignore_content).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to write .gitignore: {}", e))
    })?;
    
    Ok(())
}

fn init_udonsharp_project(
    project_dir: &Path,
    template: &str,
    vrc_sdk_path: Option<String>,
    unity_project_path: Option<String>,
) -> UdonSharpResult<()> {
    // Add UdonSharp dependencies to existing Cargo.toml
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let mut cargo_content = std::fs::read_to_string(&cargo_toml_path).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to read Cargo.toml: {}", e))
    })?;
    
    // Add UdonSharp dependencies if not already present
    if !cargo_content.contains("udonsharp-core") {
        cargo_content.push_str("\n# UdonSharp dependencies\n");
        cargo_content.push_str("udonsharp-core = { git = \"https://github.com/vrchat-community/rust-udonsharp\", branch = \"main\" }\n");
        cargo_content.push_str("udonsharp-macros = { git = \"https://github.com/vrchat-community/rust-udonsharp\", branch = \"main\" }\n");
        cargo_content.push_str("udonsharp-bindings = { git = \"https://github.com/vrchat-community/rust-udonsharp\", branch = \"main\" }\n");
        cargo_content.push_str("wasm-bindgen = \"0.2\"\n");
        cargo_content.push_str("console_error_panic_hook = \"0.1\"\n");
        cargo_content.push_str("log = \"0.4\"\n");
        
        cargo_content.push_str("\n[build-dependencies]\n");
        cargo_content.push_str("udonsharp-build = { git = \"https://github.com/vrchat-community/rust-udonsharp\", branch = \"main\" }\n");
        
        std::fs::write(&cargo_toml_path, cargo_content).map_err(|e| {
            udonsharp_core::UdonSharpError::configuration(format!("Failed to update Cargo.toml: {}", e))
        })?;
    }
    
    // Create build.rs if it doesn't exist
    let build_rs_path = project_dir.join("build.rs");
    if !build_rs_path.exists() {
        let build_rs_content = create_build_rs_content(vrc_sdk_path, unity_project_path)?;
        std::fs::write(build_rs_path, build_rs_content).map_err(|e| {
            udonsharp_core::UdonSharpError::configuration(format!("Failed to write build.rs: {}", e))
        })?;
    }
    
    Ok(())
}

fn create_cargo_toml_content(name: &str, template: &str) -> UdonSharpResult<String> {
    let content = match template {
        "basic" => format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
description = "UdonSharp project written in Rust"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
udonsharp-core = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}
udonsharp-macros = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}
udonsharp-bindings = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
log = "0.4"

[build-dependencies]
udonsharp-build = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
]

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"

[profile.dev]
opt-level = 1
debug = false
panic = "abort"
"#,
            name
        ),
        "advanced" => format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
description = "Advanced UdonSharp project written in Rust"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
udonsharp-core = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}
udonsharp-macros = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}
udonsharp-bindings = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
log = "0.4"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

[build-dependencies]
udonsharp-build = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
]

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"

[profile.dev]
opt-level = 1
debug = false
panic = "abort"
"#,
            name
        ),
        _ => {
            return Err(udonsharp_core::UdonSharpError::configuration(
                format!("Unknown template: {}", template)
            ));
        }
    };
    
    Ok(content)
}

fn create_build_rs_content(
    vrc_sdk_path: Option<String>,
    unity_project_path: Option<String>,
) -> UdonSharpResult<String> {
    let mut content = String::from(
        r#"//! Build script for UdonSharp project
//! 
//! This build script automatically generates API bindings from Unity and VRChat
//! .asmdef files and sets up the build environment for UdonSharp compilation.

use udonsharp_build::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up UdonSharp build with automatic Unity project detection
    setup_udonsharp_build_with_config(|mut config| {
        config = config
            .with_unity_project_detection()
            .bindings_output_dir("generated")
            .auto_generate_bindings(true)
            .watch_asmdef_files(true);
"#,
    );
    
    if let Some(vrc_path) = vrc_sdk_path {
        content.push_str(&format!(
            r#"        
        // Add custom VRChat SDK path
        config = config.add_asmdef_scan_dir("{}");
"#,
            vrc_path
        ));
    }
    
    if let Some(unity_path) = unity_project_path {
        content.push_str(&format!(
            r#"        
        // Add custom Unity project path
        config = config.add_asmdef_scan_dir("{}/Library/PackageCache");
        config = config.add_asmdef_scan_dir("{}/Assets");
        config = config.add_asmdef_scan_dir("{}/Packages");
"#,
            unity_path, unity_path, unity_path
        ));
    }
    
    content.push_str(
        r#"        
        config
    })?;
    
    Ok(())
}
"#,
    );
    
    Ok(content)
}

fn create_lib_rs_content(template: &str) -> UdonSharpResult<String> {
    let content = match template {
        "basic" => r#"//! Basic UdonSharp project written in Rust
//! 
//! This demonstrates the basic structure of a UdonSharp project using Rust.

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

/// Main UdonSharp behaviour for this project
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct MyUdonBehaviour {
    #[udon_public]
    pub message: String,
    
    #[udon_sync]
    pub counter: i32,
    
    initialized: bool,
}

impl UdonBehaviour for MyUdonBehaviour {
    fn start(&mut self) {
        self.initialized = true;
        self.message = "Hello from Rust UdonSharp!".to_string();
        self.counter = 0;
        
        // Log a message to the VRChat console
        debug_log(&format!("UdonSharp behaviour started: {}", self.message));
    }
    
    fn update(&mut self) {
        // Update logic here
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.counter += 1;
        debug_log(&format!("Player joined: {}. Total players: {}", 
                          player.get_display_name(), self.counter));
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.counter -= 1;
        debug_log(&format!("Player left: {}. Total players: {}", 
                          player.get_display_name(), self.counter));
    }
}

impl MyUdonBehaviour {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            counter: 0,
            initialized: false,
        }
    }
    
    #[udon_event]
    pub fn on_interact(&mut self) {
        if self.initialized {
            debug_log("Interact event triggered!");
            self.counter += 1;
        }
    }
}

// Export the main behaviour for UdonSharp compilation
#[no_mangle]
pub extern "C" fn create_behaviour() -> MyUdonBehaviour {
    MyUdonBehaviour::new()
}
"#.to_string(),
        "advanced" => r#"//! Advanced UdonSharp project written in Rust
//! 
//! This demonstrates advanced features of the Rust UdonSharp framework.

use udonsharp_core::prelude::*;
use udonsharp_macros::*;
use udonsharp_bindings::{unity::*, vrchat::*, csharp::*};
use serde::{Serialize, Deserialize};

/// Configuration data structure
#[derive(Serialize, Deserialize, Clone)]
pub struct WorldConfig {
    pub world_name: String,
    pub max_players: i32,
    pub enable_networking: bool,
}

/// Advanced UdonSharp behaviour with networking and Unity integration
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct AdvancedWorldController {
    #[udon_public]
    pub config: String, // JSON serialized WorldConfig
    
    #[udon_sync]
    pub world_state: i32,
    
    #[udon_sync]
    pub active_players: i32,
    
    // Unity object references
    spawn_points: Vec<Transform>,
    ui_canvas: Option<GameObject>,
    
    // Internal state
    initialized: bool,
    config_data: Option<WorldConfig>,
}

impl UdonBehaviour for AdvancedWorldController {
    fn start(&mut self) {
        self.initialize_world();
        self.setup_ui();
        self.find_spawn_points();
        
        self.initialized = true;
        debug_log("Advanced world controller initialized");
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        // Update world state
        self.update_world_state();
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.active_players += 1;
        
        // Teleport player to spawn point
        if let Some(spawn_point) = self.get_next_spawn_point() {
            player.teleport_to(spawn_point.position(), spawn_point.rotation());
        }
        
        // Update UI
        self.update_player_count_ui();
        
        debug_log(&format!("Player {} joined. Active players: {}", 
                          player.get_display_name(), self.active_players));
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.active_players -= 1;
        self.update_player_count_ui();
        
        debug_log(&format!("Player {} left. Active players: {}", 
                          player.get_display_name(), self.active_players));
    }
}

impl AdvancedWorldController {
    pub fn new() -> Self {
        Self {
            config: String::new(),
            world_state: 0,
            active_players: 0,
            spawn_points: Vec::new(),
            ui_canvas: None,
            initialized: false,
            config_data: None,
        }
    }
    
    fn initialize_world(&mut self) {
        // Parse configuration
        if !self.config.is_empty() {
            match serde_json::from_str::<WorldConfig>(&self.config) {
                Ok(config) => {
                    self.config_data = Some(config);
                    debug_log("World configuration loaded successfully");
                }
                Err(e) => {
                    debug_log(&format!("Failed to parse world config: {}", e));
                    // Use default configuration
                    self.config_data = Some(WorldConfig {
                        world_name: "Rust UdonSharp World".to_string(),
                        max_players: 20,
                        enable_networking: true,
                    });
                }
            }
        }
    }
    
    fn setup_ui(&mut self) {
        // Find UI canvas
        self.ui_canvas = GameObject::find("UI Canvas");
        if self.ui_canvas.is_some() {
            debug_log("UI Canvas found and connected");
        }
    }
    
    fn find_spawn_points(&mut self) {
        // Find all spawn points in the scene
        for i in 0..10 {
            let spawn_name = format!("SpawnPoint_{}", i);
            if let Some(spawn_obj) = GameObject::find(&spawn_name) {
                self.spawn_points.push(spawn_obj.transform());
            }
        }
        
        debug_log(&format!("Found {} spawn points", self.spawn_points.len()));
    }
    
    fn get_next_spawn_point(&self) -> Option<&Transform> {
        if self.spawn_points.is_empty() {
            return None;
        }
        
        let index = (self.active_players as usize) % self.spawn_points.len();
        self.spawn_points.get(index)
    }
    
    fn update_world_state(&mut self) {
        // Update world state based on player count and time
        let time = Time::time();
        self.world_state = (time as i32) % 1000;
    }
    
    fn update_player_count_ui(&self) {
        if let Some(ref canvas) = self.ui_canvas {
            // Update UI text showing player count
            if let Some(text_obj) = canvas.find_child("PlayerCountText") {
                if let Some(text_component) = text_obj.get_component::<UnityEngine::UI::Text>() {
                    text_component.set_text(&format!("Players: {}", self.active_players));
                }
            }
        }
    }
    
    #[udon_event]
    pub fn on_world_reset(&mut self) {
        debug_log("World reset triggered");
        self.world_state = 0;
        
        // Reset all players to spawn points
        let players = Networking::get_players();
        for (i, player) in players.iter().enumerate() {
            if let Some(spawn_point) = self.spawn_points.get(i % self.spawn_points.len()) {
                player.teleport_to(spawn_point.position(), spawn_point.rotation());
            }
        }
    }
    
    #[udon_event]
    pub fn on_config_update(&mut self) {
        debug_log("Configuration update triggered");
        self.initialize_world();
    }
}

// Export the main behaviour for UdonSharp compilation
#[no_mangle]
pub extern "C" fn create_behaviour() -> AdvancedWorldController {
    AdvancedWorldController::new()
}
"#.to_string(),
        _ => {
            return Err(udonsharp_core::UdonSharpError::configuration(
                format!("Unknown template: {}", template)
            ));
        }
    };
    
    Ok(content)
}

async fn handle_templates_command() -> UdonSharpResult<()> {
    println!("📋 Available UdonSharp Project Templates\n");
    
    let templates = [
        udonsharp_build::ProjectTemplate::Basic,
        udonsharp_build::ProjectTemplate::Advanced,
        udonsharp_build::ProjectTemplate::Networking,
        udonsharp_build::ProjectTemplate::GameLogic,
        udonsharp_build::ProjectTemplate::UI,
        udonsharp_build::ProjectTemplate::Physics,
        udonsharp_build::ProjectTemplate::Audio,
    ];
    
    for template in &templates {
        println!("🎯 {}", template.name());
        println!("   Description: {}", template.description());
        println!("   Features:");
        for feature in template.features() {
            println!("     - {}", feature);
        }
        println!();
    }
    
    println!("💡 Usage:");
    println!("   cargo udonsharp new my-project --template basic");
    println!("   cargo udonsharp new my-project --template networking --examples --tests");
    println!("   cargo udonsharp new my-project --template ui --workspace");
    println!();
    println!("🔧 Additional Options:");
    println!("   --examples     Include example files");
    println!("   --tests        Include test files");
    println!("   --docs         Include documentation (default: true)");
    println!("   --workspace    Create as a workspace project");
    println!("   --features     Add custom features");
    
    Ok(())
}