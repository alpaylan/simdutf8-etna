# simdutf8 — Injected Bugs

Total mutations: 2

Each `etna/<variant>` branch is a pre-applied snapshot containing exactly one buggy commit on top of `base_commit`. Both variants are also available as marauders blocks (comment-toggled injection) in the base source at `src/implementation/algorithm.rs`.

## Bug Index

| # | Name | Variant | File(s) | Injection | Fix Commit |
|---|------|---------|---------|-----------|------------|
| 1 | `validate_utf8_basic` drops incomplete-at-EOF flag | `incomplete_eof_basic_8c24752_1` | `src/implementation/algorithm.rs` | marauders | `8c247522cd4d4a07031170dfb7f9b0161c725d2a` |
| 2 | `validate_utf8_compat_simd0` drops incomplete-at-EOF flag | `incomplete_eof_compat_8c24752_2` | `src/implementation/algorithm.rs` | marauders | `8c247522cd4d4a07031170dfb7f9b0161c725d2a` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `incomplete_eof_basic_8c24752_1` | `property_basic_matches_std` | `witness_incomplete_eof_basic_case_f0_aligned`, `witness_incomplete_eof_basic_case_c0_aligned` |
| `incomplete_eof_compat_8c24752_2` | `property_compat_matches_std` | `witness_incomplete_eof_compat_case_f0_aligned`, `witness_incomplete_eof_compat_case_c0_aligned` |

## Framework Coverage

| Property | etna | proptest | quickcheck | crabcheck | hegel |
|----------|:----:|:--------:|:----------:|:---------:|:-----:|
| `property_basic_matches_std` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_compat_matches_std` | ✓ | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. `validate_utf8_basic` drops incomplete-at-EOF flag

- **Variant**: `incomplete_eof_basic_8c24752_1`
- **Location**: `src/implementation/algorithm.rs` (tail of `validate_utf8_basic`)
- **Property**: `property_basic_matches_std`
- **Witnesses**: `witness_incomplete_eof_basic_case_f0_aligned`, `witness_incomplete_eof_basic_case_c0_aligned`
- **Fix commit**: `8c247522cd4d4a07031170dfb7f9b0161c725d2a` — `fix: check for incomplete bytes at end + tests`
- **Invariant violated**: `simdutf8::basic::from_utf8(x).is_ok()` must agree with `core::str::from_utf8(x).is_ok()` for every byte sequence.
- **How the mutation triggers**: `Utf8CheckAlgorithm::check_block` tracks an `incomplete` SIMD mask that flags the positions of trailing multi-byte leading bytes in the last processed chunk (`is_incomplete` saturating-subtracts thresholds `0xEF`, `0xDF`, `0xBF` at the final 3 byte slots). `check_incomplete_pending` is the point where this mask is OR'd into `error`; without it, inputs whose only UTF-8 problem is a missing continuation byte past the last SIMD chunk finish with `error` still clear and are wrongly accepted. The mutation deletes that post-loop call, so e.g. `[b'a'; 63]` followed by `0xF0` is accepted by `basic::from_utf8` but rejected by `core::str::from_utf8`.

### 2. `validate_utf8_compat_simd0` drops incomplete-at-EOF flag

- **Variant**: `incomplete_eof_compat_8c24752_2`
- **Location**: `src/implementation/algorithm.rs` (tail of `validate_utf8_compat_simd0`)
- **Property**: `property_compat_matches_std`
- **Witnesses**: `witness_incomplete_eof_compat_case_f0_aligned`, `witness_incomplete_eof_compat_case_c0_aligned`
- **Fix commit**: `8c247522cd4d4a07031170dfb7f9b0161c725d2a` — `fix: check for incomplete bytes at end + tests`
- **Invariant violated**: `simdutf8::compat::from_utf8(x)` must match `core::str::from_utf8(x)` on validity *and* — on the error path — on both `valid_up_to()` and `error_len()`. The compat API advertises full `std::str::Utf8Error` parity, so a variant that wrongly accepts invalid input violates the stronger half of that parity.
- **How the mutation triggers**: Identical mechanism to variant 1, but in the compat code path (`validate_utf8_compat_simd0`). The compat path has an early-return escape inside the main loop for non-ASCII content, so the only way the mutation is observable is through inputs whose invalid byte survives as *just* the incomplete-at-EOF flag (no hard error set by `check_bytes`). The witness inputs end with a bare UTF-8 leading byte (`0xF0` or `0xC0`) at position 63 of a 64-byte chunk — exactly the configuration commit `8c24752` added the tail `check_incomplete_pending` to catch.

## Notes

- `marauders` injection: on the `main` branch, the base source has comment-toggled marauders blocks (marker `incomplete_eof_basic` / `incomplete_eof_compat`). `marauders list` prints both variants; `marauders convert --path src/implementation/algorithm.rs --to functional` rewrites them into a runtime `match` on `std::env::var("M_<variant>")`, which makes `M_<variant>=active` flip the injected code.
- `etna/<variant>` branches are pre-materialized versions of each mutation: identical to base except the associated `check_incomplete_pending()` call is absent, matching the pre-8c24752 code.
- The two variants share a fix commit (`8c24752`) but target distinct call sites in distinct functions. They are independent: either can be injected without activating the other, and each has its own dedicated witnesses that keep passing when only the *other* variant is active.
