use std::{env, fs, path::PathBuf, process::Command};
use std::path::Path;

fn main() {
    let vendor = PathBuf::from("../vendor");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let with_icu = cfg!(feature = "icu");

    println!("=== build.rs start ===");
    println!("OUT_DIR = {:?}", out_dir);
    println!("TARGET  = {:?}", env::var("TARGET").unwrap());

    // --- Rebuild triggers ---
    println!("cargo:rerun-if-changed=src/c-bindings.cc");
    println!("cargo:rerun-if-changed=src/c-bindings.h");
    println!("cargo:rerun-if-changed=../vendor/re2");
    println!("cargo:rerun-if-changed=../vendor/abseil-cpp");
    println!("cargo:rerun-if-changed=../vendor/icu");

    if with_icu {
        println!("--- Compiling ICU sources ---");
        build_icu(&vendor, &out_dir, with_icu);
        println!("--- Finished ICU ---");
    }

    println!("--- Building Abseil (subset) ---");
    build_absl(&vendor, with_icu);
    println!("--- Finished Abseil ---");

    println!("--- Building RE2 ---");
    build_re2(vendor, with_icu);
    println!("--- Finished RE2 ---");

    println!("cargo:rustc-link-lib=static=re2_core");
    println!("cargo:rerun-if-changed=src/c-bindings.h");
    println!("cargo:rerun-if-changed=src/c-bindings.cc");

    //
    // --- Bindings mode (dual) ---
    //
    #[cfg(feature = "bindgen")]
    {
        println!("cargo:warning=Generating fresh bindings with bindgen");
        let bindings = bindgen::Builder::default()
            .header("src/c-bindings.h")
            .allowlist_function("re2_.*")
            .allowlist_type("RE2Wrapper")
            .generate()
            .expect("Unable to generate bindings");

        let out_path = out_dir.join("bindings.rs");
        bindings
            .write_to_file(&out_path)
            .expect("Couldn't write bindings");
        println!("cargo:rerun-if-changed=src/c-bindings.h");
    }

    #[cfg(not(feature = "bindgen"))]
    {
        eprintln!("info: Using pregenerated bindings.rs");
        let src = PathBuf::from("src/bindings.rs");
        let dst = out_dir.join("bindings.rs");
        fs::copy(&src, &dst).expect("Couldn't copy pregenerated bindings");
    }

    println!("=== build.rs end ===");
}

fn build_re2(vendor: PathBuf, with_icu: bool) {
    let mut re2 = cc::Build::new();

    re2.include(vendor.join("re2"));
    re2.include(vendor.join("abseil-cpp"));

    if with_icu {
        re2.include(vendor.join("icu/source/common"));
        re2.include(vendor.join("icu/source/i18n"));
    }

    let compiler = re2.get_compiler();
    let is_msvc = compiler.is_like_msvc();

    add_common_defines(&mut re2, is_msvc, with_icu);

    for entry in glob::glob("../vendor/re2/*.cc").unwrap() {
        let file = entry.unwrap();
        println!("RE2 {}", file.display());
        re2.file(&file);
    }
    for entry in glob::glob("../vendor/re2/util/*.cc").unwrap() {
        let file = entry.unwrap();
        println!("RE2 UTIL {}", file.display());
        re2.file(&file);
    }

    re2.file("src/c-bindings.cc");

    re2.compile("re2_core");
}

fn build_absl(vendor: &Path, with_icu: bool) {
    println!("--- Building Abseil ---");

    let absl_targets: &[(&str, &[&str])] = &[
        ("absl_base", &[
            "absl/base/*.cc",
            "absl/base/internal/*.cc",
        ]),
        ("absl_strings", &[
            "absl/strings/*.cc",
            "absl/strings/internal/*.cc",
        ]),
        ("absl_synchronization", &[
            "absl/synchronization/*.cc",
            "absl/synchronization/internal/*.cc"
        ]),
        ("absl_time", &[
            "absl/time/*.cc",
            "absl/time/internal/*.cc",
        ]),
        ("absl_time_cctz", &[
            "absl/time/internal/cctz/src/*.cc",
        ]),
        ("absl_numeric", &["absl/numeric/*.cc"]),
        ("absl_hash", &[
            "absl/hash/*.cc",
            "absl/hash/internal/*.cc",
        ]),
        ("absl_log", &[
            "absl/log/*.cc",
            "absl/log/internal/*.cc",
            "absl/log/internal/check_op.cc",
        ]),
        ("absl_symbolize", &[
            "absl/debugging/*.cc",
            "absl/debugging/internal/*.cc",
        ]),
        ("absl_stacktrace", &[
            "absl/debugging/stacktrace.cc",
        ]),
        ("absl_str_format", &[
            "absl/strings/str_format.cc",
            "absl/strings/internal/str_format/*.cc",
        ]),
        ("absl_container", &[
            "absl/container/*.cc",
            "absl/container/internal/*.cc",
        ]),
    ];



    for (libname, patterns) in absl_targets {
        let mut absl = cc::Build::new();
        absl.cpp(true).include(vendor.join("abseil-cpp"));

        let compiler = absl.get_compiler();
        let is_msvc = compiler.is_like_msvc();

        add_common_defines(&mut absl, is_msvc, with_icu);

        let mut count = 0;
        for pat in *patterns {
            let globpat = vendor.join("abseil-cpp").join(pat).display().to_string();
            for entry in glob::glob(&globpat).unwrap() {
                let file = entry.unwrap();
                let path = file.display().to_string();
                if path.contains("test") || path.contains("benchmark") || path.contains("mock") {
                    println!("SKIP ABSL {}", path);
                    continue;
                }
                println!("ABSL [{}] {}", libname, path);
                absl.file(&file);
                count += 1;
            }
        }

        if count > 0 {
            absl.compile(libname);
            println!("Built Abseil library: {} ({} files)", libname, count);
            println!("cargo:rustc-link-lib=static={}", libname);
        } else {
            println!("No source files for {}, skipping", libname);
        }
    }
}


fn build_icu(vendor: &Path, out_dir: &Path, with_icu: bool) {

    let mut cc_probe = cc::Build::new();
    cc_probe
        .include(vendor.join("icu/common"))
        .include(vendor.join("icu/common/unicode"))
        .include(vendor.join("icu/i18n"))
        .include(vendor.join("icu/i18n/unicode"));

    let compiler = cc_probe.get_compiler();
    let is_msvc = compiler.is_like_msvc();

    add_common_defines(&mut cc_probe, is_msvc, with_icu);
    if is_msvc {
        add_msvc_includes(&mut cc_probe);
    }

    let obj_suffix = if is_msvc { "obj" } else { "o" };

    for entry in glob::glob("../vendor/icu/common/*.cpp")
        .unwrap()
        .chain(glob::glob("../vendor/icu/i18n/*.cpp").unwrap())
    {
        let file = entry.unwrap();
        let fname = file.file_name().unwrap();
        let obj = out_dir.join(fname).with_extension(obj_suffix);

        let path_str = file.display().to_string();
        if path_str.contains("test") || path_str.contains("sample") || path_str.contains("bench") {
            println!("SKIP ICU {}", path_str);
            continue;
        }

        println!("ICU compile -> {}", path_str);

        let mut cmd = Command::new(compiler.path());
        cmd.args(compiler.args());
        if is_msvc {
            cmd.arg("/c").arg(&file).arg("/Fo").arg(&obj);
        } else {
            cmd.arg("-c").arg(&file).arg("-o").arg(&obj);
        }

        let status = cmd.status().expect("failed to spawn compiler");
        assert!(status.success(), "ICU compilation failed for {:?}", file);

        println!("cargo:rustc-link-arg={}", obj.display());
    }
}

fn add_common_defines(build: &mut cc::Build, is_msvc: bool, with_icu: bool) {
    build.cpp(true);
    if is_msvc {
        build.flag("/std:c++17").flag("/EHsc");
        build.define("NOMINMAX", None);
    } else {
        build.flag_if_supported("-std=c++17");
    }
    if with_icu {
        build.define("RE2_WITH_ICU", None);
        build.define("RE2_USE_ICU", Some("1"));
    }
}

//bodge for windows to ensure we get stddef.h, stdint.h etc. for ICU
fn add_msvc_includes(build: &mut cc::Build) {
    let compiler = build.get_compiler();
    if !compiler.is_like_msvc() {
        return;
    }

    // Check if INCLUDE env var is already set
    if let Ok(include) = env::var("INCLUDE") {
        for path in include.split(';') {
            if !path.is_empty() {
                build.include(path);
            }
        }
        return;
    }

    println!("No INCLUDE found, running /Bv now");
    // Fallback: query MSVC's default include paths via `cl /Bv`
    match Command::new("cl").arg("/Bv").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // Typical line: "Compiler Passes: ... -I C:\Program Files (x86)\..."
                if let Some(idx) = line.find("-I") {
                    let path = line[idx + 2..].trim();
                    println!("cargo:warning=Adding MSVC include path from cl /Bv: {}", path);
                    build.include(path);
                }
            }
        }
        Err(e) => {
            println!(
                "cargo:warning=Could not run `cl /Bv` to detect MSVC include paths: {}",
                e
            );
        }
    }
}