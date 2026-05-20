use std::fmt;

use crabcheck::profiling::quickcheck;
use crabcheck::quickcheck::{Arbitrary, Mutate};
use rand::Rng;
use simdutf8::etna::{property_basic_matches_std, property_compat_matches_std, PropertyResult};

// Mirror src/bin/etna.rs BiasedBytes: 3/4 chance biased toward 64/128/192-byte
// buffers with a trailing 0xC0..=0xFF continuation byte that lands right on
// a SIMD chunk boundary (the incomplete-EOF bug needs that shape). 1/4 random
// bytes 0..256 len. Critical: reinventing with plain random bytes misses
// the bug-triggering distribution.
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
            _ if !out.is_empty() => {
                out.pop();
            }
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

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        return;
    }
    let result = match (args[1].as_str(), args[2].as_str()) {
        ("crabcheck", "BasicMatchesStd") => quickcheck(|BiasedBytes(v)| {
            to_opt(property_basic_matches_std(v))
        }),
        ("crabcheck", "CompatMatchesStd") => quickcheck(|BiasedBytes(v)| {
            to_opt(property_compat_matches_std(v))
        }),
        (a, b) => panic!("Unknown: {a} {b}"),
    };
    println!("Result: {:?}", result);
}
