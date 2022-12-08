use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=scripts/src");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir);
    let status = Command::new("cargo")
        // .arg("+nightly")
        .args(&["build", "--release"])
        .args(&["--package", "scripts"])
        .args(&["--target", "wasm32-unknown-unknown"])
        .args(&[
            "--target-dir",
            &dest_path.join("scripts_target").to_str().unwrap(),
        ])
        // .args(&["-Z", "unstable-options"])
        .status()
        .unwrap();
    assert!(status.success());
    std::fs::copy(
        dest_path.join("scripts_target/wasm32-unknown-unknown/release/scripts.wasm"),
        dest_path.join("scripts.wasm"),
    )
    .unwrap();
}
