//! Fault-localization integration tests for simdutf8.

use std::fmt;

use crabcheck::quickcheck::{Arbitrary, Mutate};
use rand::Rng;
use simdutf8::etna::{property_basic_matches_std, property_compat_matches_std, PropertyResult};

#[derive(Clone)]
struct BiasedBytes(Vec<u8>);
impl fmt::Debug for BiasedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<R: Rng> Arbitrary<R> for BiasedBytes {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let roll: u8 = rng.random_range(0u8..4u8);
        if roll != 0 {
            let k: usize = rng.random_range(1usize..=3);
            let n = k * 64;
            let tail: u8 = rng.random_range(0xC0u8..=0xFFu8);
            let mut v = Vec::with_capacity(n);
            for _ in 0..(n - 1) {
                v.push(rng.random_range(0u8..=0x7Fu8));
            }
            v.push(tail);
            BiasedBytes(v)
        } else {
            let len: usize = rng.random_range(0usize..256);
            BiasedBytes((0..len).map(|_| rng.random()).collect())
        }
    }
}

impl<R: Rng> Mutate<R> for BiasedBytes {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.0.clone();
        match rng.random_range(0u8..3) {
            0 if !out.is_empty() => {
                let i = rng.random_range(0..out.len());
                let b = rng.random_range(0u32..8);
                out[i] ^= 1u8 << b;
            }
            1 if out.len() < 256 => out.push(rng.random()),
            _ if !out.is_empty() => { out.pop(); }
            _ => {}
        }
        BiasedBytes(out)
    }
}

fn to_opt(r: PropertyResult) -> Option<bool> {
    match r {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn property_basic_matches_std_test(BiasedBytes(v): BiasedBytes) -> Option<bool> {
    to_opt(property_basic_matches_std(v))
}

fn property_compat_matches_std_test(BiasedBytes(v): BiasedBytes) -> Option<bool> {
    to_opt(property_compat_matches_std(v))
}

fn emit_locate_json(r: &crabcheck::profiling::LocateResult) {
    use crabcheck::quickcheck::ResultStatus;
    let status = match &r.run.status {
        ResultStatus::Failed { .. } => "Failed",
        ResultStatus::Finished => "Finished",
        ResultStatus::GaveUp => "GaveUp",
        ResultStatus::TimedOut => "TimedOut",
        ResultStatus::Aborted { .. } => "Aborted",
    };
    let top = if let Some(s) = r.top() {
        serde_json::json!({
            "rank": s.rank, "file": s.region.file, "function": s.region.function,
            "start_line": s.region.start_line, "end_line": s.region.end_line,
            "ochiai": s.region.suspiciousness.ochiai, "delta": s.region.delta,
            "panic_overlap": s.panic_overlap,
            "confidence": format!("{}", s.confidence),
            "confidence_rule": s.confidence_rule,
        })
    } else { serde_json::Value::Null };
    let top_5: Vec<_> = r.suspects.iter().take(5).map(|s| serde_json::json!({
        "rank": s.rank, "file": s.region.file, "function": s.region.function,
        "start_line": s.region.start_line, "end_line": s.region.end_line,
        "confidence": format!("{}", s.confidence),
        "confidence_rule": s.confidence_rule,
        "panic_overlap": s.panic_overlap,
    })).collect();
    let diags: Vec<_> = r.diagnostics.iter().map(|d| d.tag()).collect();
    let out = serde_json::json!({
        "status": status, "passed": r.run.passed, "discarded": r.run.discarded,
        "n_panics": r.n_panics, "n_suspects": r.suspects.len(),
        "top": top, "top_5": top_5, "diagnostics": diags,
    });
    println!("@@LOCATE@@ {}", out);
}

#[test]
fn locate_basic_matches_std() {
    let report = crabcheck::quickcheck_with_locate!(property_basic_matches_std_test, "simdutf8");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_compat_matches_std() {
    let report = crabcheck::quickcheck_with_locate!(property_compat_matches_std_test, "simdutf8");
    eprintln!("{report}");
    emit_locate_json(&report);
}
