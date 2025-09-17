re2-sys_minimal
=======
Forked to work around some hard dependencies of the existing crate. Look to push back upstream.

Vendored in RE2 and abseil and ICU to avoid `pkg-config`

* RE2 - 2025-08-12
* Abseil - 20250512.1 - From https://github.com/google/re2/blob/2025-08-12/MODULE.bazel


 re2-sys
=======

Rust bindings to the RE2 C++ API for use in the `re2` crate.

# License
[BSD-3-Clause](./LICENSE), in order to match re2's license.