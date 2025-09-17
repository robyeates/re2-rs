/* SPDX-License-Identifier: BSD-3-Clause */

use std::path::PathBuf;

fn main() {
    // 1. Try system RE2 (Linux/macOS with libre2-dev + libabsl-dev)
    if pkg_config::Config::new().probe("re2").is_ok() {
        generate_bindings();
        return;
    }

    // 2. Fallback: build vendored RE2 + Abseil
    let re2_dir = PathBuf::from("vendor/re2");
    let absl_dir = PathBuf::from("vendor/abseil-cpp");

    let mut build = cc::Build::new();
    build.cpp(true)
        .include(&re2_dir)
        .include(&absl_dir)
        .include(absl_dir.join("absl"));

    if std::env::var("CARGO_CFG_TARGET_ENV").unwrap() == "msvc" {
        build.define("_WIN32", None);
        build.define("NOMINMAX", None);
        build.define("ABSL_INTERNAL_HAVE_WIN32_STACKTRACE", None);

        // Prevent accidental inclusion of Unix/macOS headers
        for macro_name in [
            "__APPLE__", "__linux__", "__FreeBSD__", "__hexagon__", "__Fuchsia__",
            "__native_client__", "__OpenBSD__", "__EMSCRIPTEN__", "__ASYLO__",
            "__asmjs__", "__wasm__",
        ] {
            build.flag(&format!("/U{}", macro_name));
        }

        // Disable POSIX-only codepaths in Abseil
        build.define("ABSL_LOW_LEVEL_WRITE_SUPPORTED", Some("0"));
        build.define("ABSL_HAVE_POSIX_WRITE", Some("0"));

        build.flag("/std:c++17");
        build.flag("/EHsc");
    } else {
        build.flag("-std=c++17");
    }

    // Our C shim
    build.file("src/c-bindings.cc");

    // Add all RE2 sources
    for entry in glob::glob("vendor/re2/*.cc").unwrap() {
        build.file(entry.unwrap());
    }
    // Add RE2 util/ sources (rune.cc, etc.)
    for entry in glob::glob("vendor/re2/util/*.cc").unwrap() {
        build.file(entry.unwrap());
    }

    // Add all Abseil sources except *_test.cc and *_benchmark.cc
    for entry in glob::glob("vendor/abseil-cpp/absl/**/*.cc").unwrap() {
        let path = entry.unwrap();
        let fname = path.file_name().unwrap().to_string_lossy();
        let path_str = path.to_string_lossy();
        if fname.ends_with("_test.cc")
            || fname.ends_with("_benchmark.cc")
            || path_str.contains("helpers")
            || path_str.contains("test_matchers")
            || path_str.contains("mock_log")
            || fname.ends_with("_test_common.cc")
            || path_str.contains("testing") {
            continue; // skip unit tests/benchmarks
        }
        build.file(path);
    }

    build.compile("re2_with_absl");

    println!("cargo:rerun-if-changed=vendor/re2");
    println!("cargo:rerun-if-changed=vendor/abseil-cpp");
    println!("cargo:rerun-if-changed=src/c-bindings.cc");
    println!("cargo:rerun-if-changed=src/c-bindings.h");

    println!("cargo:rustc-link-lib=static=re2_with_absl");

    // 3. Always generate bindings for Rust
    generate_bindings();
}

fn generate_bindings() {
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindgen::Builder::default()
        .header("src/c-bindings.h")
        .clang_arg("-xc") // treat as C, not C++
        .allowlist_function("re2_.*")
        .allowlist_type("RE2Wrapper")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
