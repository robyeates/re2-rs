#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

mod bindings;

#[allow(dead_code)]
#[cfg(feature = "bindgen")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[allow(dead_code)]
#[cfg(not(feature = "bindgen"))]
mod prebuilt {
    // ship a static version for people without clang/bindgen installed
    include!("bindings.rs");
}
#[cfg(not(feature = "bindgen"))]
pub use prebuilt::*;