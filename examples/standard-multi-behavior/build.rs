fn main() {
    // This build script is called by cargo during compilation
    // It can be used to set up any build-time configuration
    
    println!("cargo:rerun-if-changed=udonsharp.toml");
    println!("cargo:rerun-if-changed=src/");
}