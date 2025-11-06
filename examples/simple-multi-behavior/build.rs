fn main() {
    println!("cargo:rerun-if-changed=udonsharp.toml");
    println!("cargo:rerun-if-changed=src/");
}