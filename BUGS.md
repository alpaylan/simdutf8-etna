# simdutf8 — Injected Bugs

SIMD-accelerated UTF-8 validation (`basic::from_utf8` / `compat::from_utf8`) — ETNA workload.

Total mutations: 2

## Bug Index

| # | Variant | Name | Location | Injection | Fix Commit |
|---|---------|------|----------|-----------|------------|
| 1 | `incomplete_eof_basic_8c24752_1` | `incomplete_eof_basic` | `src/implementation/algorithm.rs` | `marauders` | `8c247522cd4d4a07031170dfb7f9b0161c725d2a` |
| 2 | `incomplete_eof_compat_8c24752_2` | `incomplete_eof_compat` | `src/implementation/algorithm.rs` | `marauders` | `8c247522cd4d4a07031170dfb7f9b0161c725d2a` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `incomplete_eof_basic_8c24752_1` | `BasicMatchesStd` | `witness_incomplete_eof_basic_case_f0_aligned`, `witness_incomplete_eof_basic_case_c0_aligned` |
| `incomplete_eof_compat_8c24752_2` | `CompatMatchesStd` | `witness_incomplete_eof_compat_case_f0_aligned`, `witness_incomplete_eof_compat_case_c0_aligned` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `BasicMatchesStd` | ✓ | ✓ | ✓ | ✓ |
| `CompatMatchesStd` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. incomplete_eof_basic

- **Variant**: `incomplete_eof_basic_8c24752_1`
- **Location**: `src/implementation/algorithm.rs`
- **Property**: `BasicMatchesStd`
- **Witness(es)**:
  - `witness_incomplete_eof_basic_case_f0_aligned`
  - `witness_incomplete_eof_basic_case_c0_aligned`
- **Source**: fix: check for incomplete bytes at end + tests
  > `Utf8CheckAlgorithm::check_block` tracked an `incomplete` SIMD mask for trailing multi-byte leaders but never folded it back into `error` after the last SIMD chunk. Inputs whose only UTF-8 flaw was a missing continuation byte past that chunk slipped through as valid. The fix adds the post-loop `check_incomplete_pending` call in both the basic and compat code paths.
- **Fix commit**: `8c247522cd4d4a07031170dfb7f9b0161c725d2a` — fix: check for incomplete bytes at end + tests
- **Invariant violated**: `simdutf8::basic::from_utf8(x).is_ok()` must agree with `core::str::from_utf8(x).is_ok()` for every byte sequence.
- **How the mutation triggers**: `Utf8CheckAlgorithm::check_block` tracks an `incomplete` SIMD mask that flags the positions of trailing multi-byte leading bytes in the last processed chunk (`is_incomplete` saturating-subtracts thresholds `0xEF`, `0xDF`, `0xBF` at the final 3 byte slots). `check_incomplete_pending` is the point where this mask is OR'd into `error`; without it, inputs whose only UTF-8 problem is a missing continuation byte past the last SIMD chunk finish with `error` still clear and are wrongly accepted. The mutation deletes that post-loop call, so e.g. `[b'a'; 63]` followed by `0xF0` is accepted by `basic::from_utf8` but rejected by `core::str::from_utf8`.

### 2. incomplete_eof_compat

- **Variant**: `incomplete_eof_compat_8c24752_2`
- **Location**: `src/implementation/algorithm.rs`
- **Property**: `CompatMatchesStd`
- **Witness(es)**:
  - `witness_incomplete_eof_compat_case_f0_aligned`
  - `witness_incomplete_eof_compat_case_c0_aligned`
- **Source**: fix: check for incomplete bytes at end + tests
  > `Utf8CheckAlgorithm::check_block` tracked an `incomplete` SIMD mask for trailing multi-byte leaders but never folded it back into `error` after the last SIMD chunk. Inputs whose only UTF-8 flaw was a missing continuation byte past that chunk slipped through as valid. The fix adds the post-loop `check_incomplete_pending` call in both the basic and compat code paths.
- **Fix commit**: `8c247522cd4d4a07031170dfb7f9b0161c725d2a` — fix: check for incomplete bytes at end + tests
- **Invariant violated**: `simdutf8::compat::from_utf8(x)` must match `core::str::from_utf8(x)` on validity *and* — on the error path — on both `valid_up_to()` and `error_len()`. The compat API advertises full `std::str::Utf8Error` parity, so a variant that wrongly accepts invalid input violates the stronger half of that parity.
- **How the mutation triggers**: Identical mechanism to variant 1, but in the compat code path (`validate_utf8_compat_simd0`). The compat path has an early-return escape inside the main loop for non-ASCII content, so the only way the mutation is observable is through inputs whose invalid byte survives as *just* the incomplete-at-EOF flag (no hard error set by `check_bytes`). The witness inputs end with a bare UTF-8 leading byte (`0xF0` or `0xC0`) at position 63 of a 64-byte chunk — exactly the configuration commit `8c24752` added the tail `check_incomplete_pending` to catch.
