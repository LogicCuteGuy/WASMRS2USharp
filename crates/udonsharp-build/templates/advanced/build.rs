//! Build script for advanced UdonSharp project
//! 
//! This build script automatically generates API bindings from Unity and VRChat
//! .asmdef files and sets up the build environment for UdonSharp compilation.

use udonsharp_build::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up UdonSharp build with automatic Unity project detection
    setup_udonsharp_build_with_config(|config| {
        config
            .with_unity_project_detection()
            .bindings_output_dir("generated")
            .auto_generate_bindings(true)
            .watch_asmdef_files(true)
    })?;
    
    Ok(())
}