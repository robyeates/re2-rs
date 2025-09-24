use std::{env, fs, path::PathBuf, process::Command};
use std::path::Path;

//
// --- Build re2-rs / re2-rs-icu ---
//
// 1. ICU
//    - Linked dynamically if the `icu` feature is enabled.
//    - We do not vendor ICU source here: shipping the full tree would bloat the crate (>100 MB).
//    - On Linux/macOS: expect ICU to be available via system packages (e.g. libicu-dev, icu-devel, or Homebrew icu4c).
//    - On Windows: expect a prebuilt ICU release to be downloaded/unzipped and exposed via the
//      ICU_ROOT environment variable. Example:
//      https://github.com/unicode-org/icu/releases/download/release-77-1/icu4c-77_1-Win64-MSVC2022.zip
//
// 2. Abseil
//    - Required by RE2.
//    - Lightweight, can be built directly with `cc` from vendored sources.
//
// 3. RE2
//    - Core regular expression engine.
//    - Also small enough to vendor and build directly.
//
// 4. re2-rs bindings
//    - Unsafe C bindings (c-bindings.cc/h) wrapping RE2 for use in Rust.
//    - Bindings are either generated with `bindgen` or copied from a pregenerated file.
//
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

    if with_icu {
        println!("--- Linking ICU (system/prebuilt) ---");
        link_icu();
        println!("--- Finished ICU setup ---");
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

struct IcuConfig {
    include_paths: Vec<PathBuf>,
    link_paths: Vec<PathBuf>,
    libs: Vec<String>,
}

/// Probe ICU: ICU_ROOT (Windows) → pkg-config → Homebrew fallback (macOS)
fn probe_icu() -> Option<IcuConfig> {
    if cfg!(target_os = "windows") {
        if let Ok(icu_root) = env::var("ICU_ROOT") {
            return Some(IcuConfig {
                include_paths: vec![PathBuf::from(&icu_root).join("include")],
                link_paths: vec![PathBuf::from(&icu_root).join("lib64")],
                libs: vec![
                    "icuuc".into(),
                    "icuin".into(),
                    "icudt".into(),
                    "icutu".into(),
                ],
            });
        } else {
            println!("cargo:warning=ICU_ROOT not set. Please download and unzip a prebuilt ICU, e.g.:");
            println!("cargo:warning=  https://github.com/unicode-org/icu/releases/download/release-77-1/icu4c-77_1-Win64-MSVC2022.zip");
            println!("cargo:warning=Set ICU_ROOT to the extracted folder (containing include/, lib64/, bin64/).");
            panic!("ICU_ROOT not set; cannot build with ICU on Windows");
        }
    }

    if let Ok(lib) = pkg_config::Config::new().probe("icu-i18n") {
        return Some(IcuConfig {
            include_paths: lib.include_paths,
            link_paths: lib.link_paths,
            libs: lib.libs,
        });
    }

    if cfg!(target_os = "macos") {
        let brew_prefix = Command::new("brew")
            .arg("--prefix")
            .arg("icu4c")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
            .trim()
            .to_string();

        if !brew_prefix.is_empty() {
            return Some(IcuConfig {
                include_paths: vec![PathBuf::from(format!("{}/include", brew_prefix))],
                link_paths: vec![PathBuf::from(format!("{}/lib", brew_prefix))],
                libs: vec!["icui18n".into(), "icuuc".into(), "icudata".into()],
            });
        }
    }

    None
}

fn link_icu() {

    let cfg = probe_icu().unwrap_or_else(|| {
        println!("cargo:warning=ICU not found. Set ICU_ROOT (Windows) or install via pkg-config/Homebrew.");
        panic!("ICU not found; cannot build with feature `icu`");
    });

    for path in &cfg.include_paths {
        println!("cargo:include={}", path.display());
    }
    for lib_path in &cfg.link_paths {
        println!("cargo:rustc-link-search=native={}", lib_path.display());
    }
    for lib in &cfg.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    // Windows-only: copy DLLs into target dir for test runs
    if cfg!(target_os = "windows") {
        let icu_root = env::var("ICU_ROOT").unwrap();
        let bin = PathBuf::from(&icu_root).join("bin64");

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let target_dir = out_dir.ancestors().nth(3).unwrap();
        let deps_dir = target_dir.join("deps");

        for dll in ["icuuc77.dll", "icuin77.dll", "icudt77.dll", "icutu77.dll"] {
            let src = bin.join(dll);
            let dst = deps_dir.join(dll);

            if let Err(e) = std::fs::copy(&src, &dst) {
                println!("cargo:warning=Could not copy {}: {}", dll, e);
            }
        }
    }
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

fn build_re2(vendor: PathBuf, with_icu: bool) {
    let mut re2 = cc::Build::new();

    re2.include(vendor.join("re2"));
    re2.include(vendor.join("abseil-cpp"));

    if with_icu {
        if let Some(cfg) = probe_icu() {
            for inc in &cfg.include_paths {
                re2.include(inc);
                println!("cargo:warning=Using ICU include path {}", inc.display());
            }
        }
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
