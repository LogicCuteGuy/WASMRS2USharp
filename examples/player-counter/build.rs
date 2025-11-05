use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=udonsharp.toml");
    
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("Build output directory: {}", out_dir);
    
    // This would integrate with udonsharp-build crate in the full implementation
    println!("Run 'cargo udonsharp build' to generate UdonSharp C# files with networking support");
}