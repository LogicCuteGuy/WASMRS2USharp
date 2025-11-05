use std::process::Command;

fn main() {
    println!("Building multi-behavior tutorial...");
    
    // Build the WASM target
    let output = Command::new("cargo")
        .args(&[
            "build",
            "--target", "wasm32-unknown-unknown",
            "--release"
        ])
        .output()
        .expect("Failed to execute cargo build");
    
    if !output.status.success() {
        eprintln!("Cargo build failed:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    
    println!("WASM build completed successfully");
    println!("Tutorial project ready for UdonSharp compilation");
    
    // In a real implementation, this would trigger the UdonSharp compiler
    println!("Generated files would include:");
    println!("  - Counter.cs (Counter behavior)");
    println!("  - Display.cs (Display behavior)");
    println!("  - SharedRuntime.cs (Shared functions)");
    println!("  - Counter.prefab (Counter prefab)");
    println!("  - Display.prefab (Display prefab)");
    println!("  - TutorialSystem.prefab (Combined prefab)");
}