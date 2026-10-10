//! Random numbers. The streams are kept as Python has them, because their separation is what pairs A/B runs: each
//! seat shuffles and mulligans from its own generator (`lib:{seed}:{deck}`), and play decisions use another
//! (`play:{seed}`), so changing one deck's list leaves every other seat's library and hand the same.
//!
//! The generator itself is not Python's Mersenne Twister: Rust games are checked against Python's statistically, not
//! seed for seed (documents/rust-rewrite-scope.md, section 7). It is xoshiro256**, seeded through SplitMix64 from
//! a 64-bit FNV-1a hash of the stream's name: fast, small to copy, and the same on every machine.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rng {
    s: [u64; 4],
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn splitmix(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

impl Rng {
    pub fn from_u64(seed: u64) -> Rng {
        let mut x = seed;
        Rng { s: [splitmix(&mut x), splitmix(&mut x), splitmix(&mut x), splitmix(&mut x)] }
    }

    /// A stream named like Python's: `Rng::named("play:500001")`, `Rng::named("lib:500001:sauron")`.
    pub fn named(name: &str) -> Rng {
        Rng::from_u64(fnv1a(name.as_bytes()))
    }

    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.s;
        let out = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        out
    }

    /// A float in [0, 1), like Python's `random()`.
    pub fn random(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// An integer in [0, n), without modulo bias. n must be positive.
    pub fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0);
        let zone = u64::MAX - (u64::MAX % n);
        loop {
            let x = self.next_u64();
            if x < zone {
                return x % n;
            }
        }
    }

    /// Python's `shuffle`: Fisher–Yates from the end.
    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            v.swap(i, j);
        }
    }

    pub fn choice<'a, T>(&mut self, v: &'a [T]) -> Option<&'a T> {
        if v.is_empty() { None } else { Some(&v[self.below(v.len() as u64) as usize]) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_streams_are_fixed_and_independent() {
        let mut a = Rng::named("play:500001");
        let mut b = Rng::named("play:500001");
        let mut c = Rng::named("lib:500001:sauron");
        let xs: Vec<u64> = (0..5).map(|_| a.next_u64()).collect();
        assert_eq!(xs, (0..5).map(|_| b.next_u64()).collect::<Vec<_>>());
        assert_ne!(xs, (0..5).map(|_| c.next_u64()).collect::<Vec<_>>());
    }

    #[test]
    fn floats_and_ranges_stay_in_bounds() {
        let mut r = Rng::named("t");
        let mut seen = [0u32; 7];
        for _ in 0..70_000 {
            let f = r.random();
            assert!((0.0..1.0).contains(&f));
            seen[r.below(7) as usize] += 1;
        }
        assert!(seen.iter().all(|&n| (9_000..11_000).contains(&n)), "{seen:?}");
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut r = Rng::named("s");
        let mut v: Vec<u32> = (0..100).collect();
        r.shuffle(&mut v);
        let mut w = v.clone();
        w.sort();
        assert_eq!(w, (0..100).collect::<Vec<_>>());
        assert_ne!(v, w);
    }
}
