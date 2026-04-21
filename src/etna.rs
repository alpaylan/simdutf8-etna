//! ETNA benchmark harness for simdutf8.
//!
//! Defines `PropertyResult` plus framework-neutral `property_*` functions
//! that express the observable invariants of `simdutf8::basic::from_utf8`
//! and `simdutf8::compat::from_utf8` relative to `core::str::from_utf8`.
//!
//! Each property takes a `Vec<u8>` and compares the result of the
//! simdutf8 public API against the standard library's reference
//! implementation. Mutations that break either validation path are
//! detected when the two disagree on validity, on `valid_up_to`, or on
//! `error_len`.

#![allow(missing_docs)]

use crate::{basic, compat};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

/// `simdutf8::basic::from_utf8(input).is_ok()` must match
/// `core::str::from_utf8(input).is_ok()` on every input. A mutation
/// that suppresses an error (incorrect accept) or spuriously reports
/// one (incorrect reject) breaks this equivalence.
pub fn property_basic_matches_std(input: Vec<u8>) -> PropertyResult {
    let std_ok = core::str::from_utf8(&input).is_ok();
    let simd_ok = basic::from_utf8(&input).is_ok();
    if std_ok == simd_ok {
        PropertyResult::Pass
    } else {
        PropertyResult::Fail(format!(
            "basic::from_utf8 disagreed with std::str::from_utf8 (std_ok={std_ok}, simd_ok={simd_ok})"
        ))
    }
}

/// `simdutf8::compat::from_utf8(input)` must agree with
/// `core::str::from_utf8(input)` on validity AND, on the error path,
/// on both `valid_up_to()` and `error_len()`. The compat API claims
/// full `std::str::Utf8Error` parity, so any deviation in the error
/// fields — including an off-by-one `valid_up_to` due to a broken
/// backward scan — is a defect.
pub fn property_compat_matches_std(input: Vec<u8>) -> PropertyResult {
    match (core::str::from_utf8(&input), compat::from_utf8(&input)) {
        (Ok(_), Ok(_)) => PropertyResult::Pass,
        (Err(se), Err(ce)) => {
            if se.valid_up_to() != ce.valid_up_to() {
                return PropertyResult::Fail(format!(
                    "valid_up_to mismatch: std={} simd={}",
                    se.valid_up_to(),
                    ce.valid_up_to()
                ));
            }
            if se.error_len() != ce.error_len() {
                return PropertyResult::Fail(format!(
                    "error_len mismatch: std={:?} simd={:?}",
                    se.error_len(),
                    ce.error_len()
                ));
            }
            PropertyResult::Pass
        }
        (Ok(_), Err(ce)) => PropertyResult::Fail(format!(
            "compat::from_utf8 errored on valid input (valid_up_to={}, error_len={:?})",
            ce.valid_up_to(),
            ce.error_len()
        )),
        (Err(se), Ok(_)) => PropertyResult::Fail(format!(
            "compat::from_utf8 accepted invalid input (std valid_up_to={}, error_len={:?})",
            se.valid_up_to(),
            se.error_len()
        )),
    }
}
