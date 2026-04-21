//! Deterministic witnesses for every ETNA variant.
//!
//! Each `witness_<name>_case_<tag>` test calls the framework-neutral
//! property defined in `simdutf8::etna` with a frozen input. The witness
//! passes on base and fails whenever the associated marauders variant is
//! active (`M_<variant>=active`), making the variant detectable without
//! PBT search.

use simdutf8::etna::{
    property_basic_matches_std, property_compat_matches_std, PropertyResult,
};

fn assert_pass(r: PropertyResult) {
    match r {
        PropertyResult::Pass => {}
        PropertyResult::Discard => panic!("unexpected Discard"),
        PropertyResult::Fail(msg) => panic!("{}", format!("property failed: {}", msg)),
    }
}

/// 64-byte input (exactly one SIMD chunk) ending in `0xF0` — a 4-byte
/// UTF-8 leading byte with no continuations. The incomplete-leading-byte
/// flag is set by the final `is_incomplete`/`check_block` call on the
/// chunk, and commit 8c24752 added `check_incomplete_pending()` at the
/// end of `validate_utf8_basic`/`validate_utf8_compat_simd0` to fold
/// that flag into `error`. Removing the call lets the mutation wrongly
/// accept this invalid input.
fn eof_incomplete_f0() -> Vec<u8> {
    let mut v = vec![b'a'; 63];
    v.push(0xF0);
    v
}

/// Same 64-byte alignment with a 2-byte leading byte (`0xC0`) at the
/// end. `is_incomplete` at position 63 compares against `0xBF`; `0xC0`
/// clears the threshold and sets the incomplete flag without producing
/// any other per-byte error, so the missing
/// `check_incomplete_pending()` call is the only thing that lets the
/// mutation accept this input.
fn eof_incomplete_c0() -> Vec<u8> {
    let mut v = vec![b'a'; 63];
    v.push(0xC0);
    v
}

#[test]
fn witness_incomplete_eof_basic_case_f0_aligned() {
    assert_pass(property_basic_matches_std(eof_incomplete_f0()));
}

#[test]
fn witness_incomplete_eof_basic_case_c0_aligned() {
    assert_pass(property_basic_matches_std(eof_incomplete_c0()));
}

#[test]
fn witness_incomplete_eof_compat_case_f0_aligned() {
    assert_pass(property_compat_matches_std(eof_incomplete_f0()));
}

#[test]
fn witness_incomplete_eof_compat_case_c0_aligned() {
    assert_pass(property_compat_matches_std(eof_incomplete_c0()));
}
