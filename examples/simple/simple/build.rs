use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../simple_mod/src");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir);
    let status = Command::new("cargo")
        .args(&["build"])
        .args(&["--profile", "release-wasm"])
        .args(&["--package", "simple_mod"])
        .args(&["--target", "wasm32-unknown-unknown"])
        .args(&[
            "--target-dir",
            &dest_path.join("scripts_target").to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    std::fs::copy(
        dest_path.join("scripts_target/wasm32-unknown-unknown/release-wasm/simple_mod.wasm"),
        dest_path.join("simple_mod.wasm"),
    )
    .unwrap();
}
