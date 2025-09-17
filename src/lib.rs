#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

pub mod ffi;

pub mod wrapper;
pub use wrapper::Regex;

//TODO STATIC BINDINGS!!!
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));