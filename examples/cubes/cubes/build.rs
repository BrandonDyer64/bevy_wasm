use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../cubes_protocol/src");
    build_wasm_pkg("mod_with_bevy");
    build_wasm_pkg("mod_without_bevy");
}

fn build_wasm_pkg(name: &str) {
    println!("cargo:rerun-if-changed=../{}/src", name);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir);
    let status = Command::new("cargo")
        // .arg("+nightly")
        .args(&["build"])
        .args(&["--profile", "release-wasm"])
        .args(&["--package", name])
        .args(&["--target", "wasm32-unknown-unknown"])
        .args(&[
            "--target-dir",
            &dest_path
                .join("scripts_target")
                .join(name)
                .to_str()
                .unwrap(),
        ])
        // .args(&["-Z", "unstable-options"])
        .status()
        .unwrap();
    assert!(status.success());
    std::fs::copy(
        dest_path
            .join("scripts_target")
            .join(name)
            .join(&format!("wasm32-unknown-unknown/release-wasm/{name}.wasm")),
        dest_path.join(&format!("{name}.wasm")),
    )
    .unwrap();
}
