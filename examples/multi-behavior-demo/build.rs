use std::process::Command;

fn main() {
    println!("Building multi-behavior demo...");
    
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
    
    // TODO: Run UdonSharp compilation pipeline
    // This would be integrated with the actual compiler once implemented
    println!("Multi-behavior compilation would run here");
}