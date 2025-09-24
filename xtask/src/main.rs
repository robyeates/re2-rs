use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    println!("Regenerating bindings with bindgen…");
    let sys_crate = root.join("re2-rs-sys");

    let status = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("re2-rs-sys")
        .arg("--features")
        .arg("bindgen")
        .current_dir(&root)
        .status()
        .expect("failed to run cargo build with bindgen");

    assert!(status.success(), "bindgen build failed");

    let out_glob = root.join("target/debug/build");
    let mut latest: Option<PathBuf> = None;

    for entry in fs::read_dir(&out_glob).unwrap() {
        let e = entry.unwrap().path();
        if e.file_name().unwrap().to_string_lossy().starts_with("re2-rs-sys-") {
            let cand = e.join("out/bindings.rs");
            if cand.exists() {
                latest = Some(cand);
            }
        }
    }

    let latest = latest.expect("No bindings.rs found");
    let dest = sys_crate.join("src/bindings.rs");
    fs::copy(&latest, &dest).expect("Failed to copy bindings.rs");

    println!("Bindings updated at {}", dest.display());
}
