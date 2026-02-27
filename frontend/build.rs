use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src");

    // Build the frontend for WASM
    let output = Command::new("wasm-pack")
        .args(&["build", "--target", "web", "--out-dir", "pkg", "--dev"])
        .output()
        .expect("Failed to execute wasm-pack");

    if !output.status.success() {
        println!("cargo:warning=wasm-pack build failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    println!("cargo:rustc-env=TARGET={}", std::env::var("TARGET").unwrap());
}
