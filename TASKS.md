# simdutf8 — ETNA Tasks

Total tasks: 8

ETNA tasks are **mutation/property/witness triplets**. Each row below is one runnable task: the command executes the framework-specific adapter against the buggy variant branch and should report a counterexample (or time out).

Run against a variant by first checking out its branch (`git checkout etna/<variant>`) or by activating the marauders block on `main` (`marauders convert --path src/implementation/algorithm.rs --to functional`, then `M_<variant>=active cargo run --release --bin etna -- <framework> <property>`).

## Task Index

| Task | Variant | Framework | Property | Witness(es) | Command |
|------|---------|-----------|----------|-------------|---------|
| 001 | `incomplete_eof_basic_8c24752_1` | proptest | `property_basic_matches_std` | `witness_incomplete_eof_basic_case_f0_aligned`, `witness_incomplete_eof_basic_case_c0_aligned` | `cargo run --release --bin etna -- proptest BasicMatchesStd` |
| 002 | `incomplete_eof_basic_8c24752_1` | quickcheck | `property_basic_matches_std` | `witness_incomplete_eof_basic_case_f0_aligned`, `witness_incomplete_eof_basic_case_c0_aligned` | `cargo run --release --bin etna -- quickcheck BasicMatchesStd` |
| 003 | `incomplete_eof_basic_8c24752_1` | crabcheck | `property_basic_matches_std` | `witness_incomplete_eof_basic_case_f0_aligned`, `witness_incomplete_eof_basic_case_c0_aligned` | `cargo run --release --bin etna -- crabcheck BasicMatchesStd` |
| 004 | `incomplete_eof_basic_8c24752_1` | hegel | `property_basic_matches_std` | `witness_incomplete_eof_basic_case_f0_aligned`, `witness_incomplete_eof_basic_case_c0_aligned` | `cargo run --release --bin etna -- hegel BasicMatchesStd` |
| 005 | `incomplete_eof_compat_8c24752_2` | proptest | `property_compat_matches_std` | `witness_incomplete_eof_compat_case_f0_aligned`, `witness_incomplete_eof_compat_case_c0_aligned` | `cargo run --release --bin etna -- proptest CompatMatchesStd` |
| 006 | `incomplete_eof_compat_8c24752_2` | quickcheck | `property_compat_matches_std` | `witness_incomplete_eof_compat_case_f0_aligned`, `witness_incomplete_eof_compat_case_c0_aligned` | `cargo run --release --bin etna -- quickcheck CompatMatchesStd` |
| 007 | `incomplete_eof_compat_8c24752_2` | crabcheck | `property_compat_matches_std` | `witness_incomplete_eof_compat_case_f0_aligned`, `witness_incomplete_eof_compat_case_c0_aligned` | `cargo run --release --bin etna -- crabcheck CompatMatchesStd` |
| 008 | `incomplete_eof_compat_8c24752_2` | hegel | `property_compat_matches_std` | `witness_incomplete_eof_compat_case_f0_aligned`, `witness_incomplete_eof_compat_case_c0_aligned` | `cargo run --release --bin etna -- hegel CompatMatchesStd` |

## Witness catalog

Each witness is a deterministic concrete test in `tests/etna_witnesses.rs`. On `base_commit` every witness passes. On each variant branch the witnesses listed for that variant fail; witnesses for the other variant keep passing, which also serves as a negative control.

### `property_basic_matches_std`

- `witness_incomplete_eof_basic_case_f0_aligned` — 63 `'a'` bytes followed by `0xF0`. Exactly one SIMD chunk; the 4-byte UTF-8 leading byte at position 63 triggers `is_incomplete` (`0xF0 - 0xEF = 0x01 ≠ 0`) and is otherwise flawless inside the chunk. `std` rejects the input; the fixed `basic::from_utf8` also rejects via `check_incomplete_pending`; the mutation accepts it.
- `witness_incomplete_eof_basic_case_c0_aligned` — same shape with `0xC0` (the smallest 2-byte leading byte) at position 63. Clears the `is_incomplete` threshold `0xBF` at the last slot by exactly one. Variant-detecting with a different leading-byte class.

### `property_compat_matches_std`

- `witness_incomplete_eof_compat_case_f0_aligned` — same 64-byte `'a' × 63 + 0xF0` input, run through `compat::from_utf8`. `std` reports `valid_up_to = 63, error_len = None`; the fixed compat path matches; the mutation returns `Ok(..)`, which the property flags as a full validity mismatch.
- `witness_incomplete_eof_compat_case_c0_aligned` — same 64-byte `'a' × 63 + 0xC0` input through the compat path. `std` reports `valid_up_to = 63, error_len = Some(1)` (not `None`, because `0xC0` is itself an invalid sequence start in a `None` continuation context); the fixed compat path matches; the mutation accepts.

## Negative controls

On each `etna/<variant>` branch, the witnesses belonging to the *other* variant continue to pass. This is verified as part of end-to-end validation (`stage:validate` in `progress.jsonl`) — every framework task for the non-active variant exits with `status:passed`, every framework task for the active variant exits with `status:failed` and reports a counterexample.
