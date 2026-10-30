use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo"));
    let shaders_dir = manifest_dir.join("../shaders");

    for changed in [shaders_dir.join("rust-toolchain.toml"), shaders_dir.join("Cargo.toml"), shaders_dir.join("src"), manifest_dir.join("../sgpu/crates/sgpu-shaders/src")] {
        println!("cargo:rerun-if-changed={}", changed.display());
    }

    let channel = toolchain_channel(&shaders_dir.join("rust-toolchain.toml"));

    let status = Command::new("rustup")
        .arg("run")
        .arg(&channel)
        .arg("cargo")
        .arg("run")
        .arg("--release")
        .arg("--bin")
        .arg("compile")
        .arg("--features")
        .arg("compile-driver")
        .arg("--manifest-path")
        .arg(shaders_dir.join("Cargo.toml"))
        .current_dir(&shaders_dir)
        .env_remove("RUSTC")
        .env_remove("RUSTDOC")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .status()
        .expect("failed to spawn rustup; the minceraft build compiles rust-gpu shaders via shaders/rust-toolchain.toml");
    assert!(status.success(), "shader build (compile bin) failed");

    let spv = shaders_dir.join("out/shaders.spv");
    assert!(spv.exists(), "compile bin did not produce {}", spv.display());
    println!("cargo:rustc-env=SHADERS_SPV={}", spv.display());
}

fn toolchain_channel(path: &Path) -> String {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    text.lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("channel"))
        .and_then(|rest| rest.trim_start().strip_prefix('='))
        .and_then(|rest| rest.trim().strip_prefix('"'))
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or_else(|| panic!("no `channel = \"...\"` in {}", path.display()))
        .to_owned()
}
