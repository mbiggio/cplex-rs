use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

use bindgen::callbacks::{MacroParsingBehavior, ParseCallbacks};

const IGNORE_MACROS: [&str; 20] = [
    "FE_DIVBYZERO",
    "FE_DOWNWARD",
    "FE_INEXACT",
    "FE_INVALID",
    "FE_OVERFLOW",
    "FE_TONEAREST",
    "FE_TOWARDZERO",
    "FE_UNDERFLOW",
    "FE_UPWARD",
    "FP_INFINITE",
    "FP_INT_DOWNWARD",
    "FP_INT_TONEAREST",
    "FP_INT_TONEARESTFROMZERO",
    "FP_INT_TOWARDZERO",
    "FP_INT_UPWARD",
    "FP_NAN",
    "FP_NORMAL",
    "FP_SUBNORMAL",
    "FP_ZERO",
    "IPPORT_RESERVED",
];

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        if self.0.contains(name) {
            MacroParsingBehavior::Ignore
        } else {
            MacroParsingBehavior::Default
        }
    }
}

impl IgnoreMacros {
    fn new() -> Self {
        Self(IGNORE_MACROS.into_iter().map(|s| s.to_owned()).collect())
    }
}

fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=/usr/local/cplex/lib/x86-64_linux/static_pic");

    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=cplex");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // See https://github.com/rust-lang/rust-bindgen/issues/687
        .parse_callbacks(Box::new(IgnoreMacros::new()))
        // Rust doesn't support long double, and bindgen can't skip it
        // https://github.com/rust-lang/rust-bindgen/issues/1549
        .blocklist_function("__fpclassifyl")
        .blocklist_function("__infl")
        .blocklist_function(".*acoshl")
        .blocklist_function(".*acosl")
        .blocklist_function(".*asinhl")
        .blocklist_function(".*asinl")
        .blocklist_function(".*atan2l")
        .blocklist_function(".*atanhl")
        .blocklist_function(".*atanl")
        .blocklist_function(".*cbrtl")
        .blocklist_function(".*ceill")
        .blocklist_function(".*copysignl")
        .blocklist_function(".*coshl")
        .blocklist_function(".*cosl")
        .blocklist_function(".*dreml")
        .blocklist_function(".*erfcl")
        .blocklist_function(".*erfl")
        .blocklist_function(".*exp2l")
        .blocklist_function(".*expl")
        .blocklist_function(".*expm1l")
        .blocklist_function(".*fabsl")
        .blocklist_function(".*fdiml")
        .blocklist_function(".*floorl")
        .blocklist_function(".*finitel")
        .blocklist_function(".*fmal")
        .blocklist_function(".*fmaxl")
        .blocklist_function(".*fminl")
        .blocklist_function(".*fmodl")
        .blocklist_function(".*frexpl")
        .blocklist_function(".*gammal")
        .blocklist_function(".*hypotl")
        .blocklist_function(".*ilogbl")
        .blocklist_function(".*j0l")
        .blocklist_function(".*j1l")
        .blocklist_function(".*jnl")
        .blocklist_function(".*ldexpl")
        .blocklist_function(".*lgammal")
        .blocklist_function(".*lgammal_r")
        .blocklist_function(".*llrintl")
        .blocklist_function(".*llroundl")
        .blocklist_function(".*iseqsigl")
        .blocklist_function(".*issignalingl")
        .blocklist_function(".*isinfl")
        .blocklist_function(".*log10l")
        .blocklist_function(".*log1pl")
        .blocklist_function(".*log2l")
        .blocklist_function(".*logbl")
        .blocklist_function(".*logl")
        .blocklist_function(".*lrintl")
        .blocklist_function(".*lroundl")
        .blocklist_function(".*modfl")
        .blocklist_function(".*nanl")
        .blocklist_function(".*nearbyintl")
        .blocklist_function(".*nextafterl")
        .blocklist_function(".*nexttoward")
        .blocklist_function(".*nexttowardf")
        .blocklist_function(".*nexttowardl")
        .blocklist_function(".*powl")
        .blocklist_function(".*qecvt")
        .blocklist_function(".*qecvt_r")
        .blocklist_function(".*qfcvt")
        .blocklist_function(".*qfcvt_r")
        .blocklist_function(".*qgcvt")
        .blocklist_function(".*remainderl")
        .blocklist_function(".*remquol")
        .blocklist_function(".*rintl")
        .blocklist_function(".*roundl")
        .blocklist_function(".*scalbl")
        .blocklist_function(".*scalblnl")
        .blocklist_function(".*scalbnl")
        .blocklist_function(".*signbitl")
        .blocklist_function(".*significandl")
        .blocklist_function(".*sinhl")
        .blocklist_function(".*sinl")
        .blocklist_function(".*sqrtl")
        .blocklist_function(".*strtold")
        .blocklist_function(".*tanhl")
        .blocklist_function(".*tanl")
        .blocklist_function(".*tgammal")
        .blocklist_function(".*truncl")
        .blocklist_function(".*y0l")
        .blocklist_function(".*y1l")
        .blocklist_function(".*ynl")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
