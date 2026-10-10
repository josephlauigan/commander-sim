//! Python's `random.Random`, as far as the run harness uses it: seeded from a string (`random.Random('pool:500000')`),
//! `sample` and `shuffle`. Seat order comes from it (pools.draw_seats), so the same seed seats the same opponents in
//! the same order as the Python harness did. The games themselves use sim-core's own generator.
//!
//! Python seeds a string generator with the integer made of the string's bytes followed by their SHA-512 digest,
//! then runs the Mersenne Twister's `init_by_array` on that integer's 32-bit words (least significant first).

pub struct PyRandom {
    mt: [u32; 624],
    index: usize,
}

const N: usize = 624;
const M: usize = 397;

impl PyRandom {
    /// random.Random(s) for a str s (seed version 2)
    pub fn from_str(s: &str) -> PyRandom {
        let mut bytes = s.as_bytes().to_vec();
        bytes.extend_from_slice(&sha512(s.as_bytes()));
        // the big-endian integer's 32-bit words, least significant first
        let first = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
        let mut le: Vec<u8> = bytes[first..].iter().rev().copied().collect();
        while !le.len().is_multiple_of(4) {
            le.push(0);
        }
        let mut key: Vec<u32> = le.chunks(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
        if key.is_empty() {
            key.push(0);
        }
        PyRandom::from_key(&key)
    }

    fn from_key(key: &[u32]) -> PyRandom {
        let mut r = PyRandom { mt: [0; N], index: N };
        r.init_genrand(19650218);
        let (mut i, mut j) = (1usize, 0usize);
        let mut k = N.max(key.len());
        while k > 0 {
            let prev = r.mt[i - 1] ^ (r.mt[i - 1] >> 30);
            r.mt[i] = (r.mt[i] ^ prev.wrapping_mul(1664525)).wrapping_add(key[j]).wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                r.mt[0] = r.mt[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
            k -= 1;
        }
        k = N - 1;
        while k > 0 {
            let prev = r.mt[i - 1] ^ (r.mt[i - 1] >> 30);
            r.mt[i] = (r.mt[i] ^ prev.wrapping_mul(1566083941)).wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                r.mt[0] = r.mt[N - 1];
                i = 1;
            }
            k -= 1;
        }
        r.mt[0] = 0x8000_0000;
        r
    }

    fn init_genrand(&mut self, s: u32) {
        self.mt[0] = s;
        for i in 1..N {
            let prev = self.mt[i - 1] ^ (self.mt[i - 1] >> 30);
            self.mt[i] = 1812433253u32.wrapping_mul(prev).wrapping_add(i as u32);
        }
        self.index = N;
    }

    pub fn genrand_u32(&mut self) -> u32 {
        const UPPER: u32 = 0x8000_0000;
        const LOWER: u32 = 0x7fff_ffff;
        const MATRIX_A: u32 = 0x9908_b0df;
        if self.index >= N {
            for kk in 0..N {
                let y = (self.mt[kk] & UPPER) | (self.mt[(kk + 1) % N] & LOWER);
                self.mt[kk] = self.mt[(kk + M) % N] ^ (y >> 1) ^ if y & 1 != 0 { MATRIX_A } else { 0 };
            }
            self.index = 0;
        }
        let mut y = self.mt[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }

    /// getrandbits(k) for 0 <= k <= 32
    pub fn getrandbits(&mut self, k: u32) -> u32 {
        if k == 0 {
            return 0;
        }
        self.genrand_u32() >> (32 - k)
    }

    /// _randbelow_with_getrandbits(n), n >= 1
    pub fn randbelow(&mut self, n: usize) -> usize {
        let k = usize::BITS - n.leading_zeros();
        loop {
            let r = self.getrandbits(k) as usize;
            if r < n {
                return r;
            }
        }
    }

    /// random.shuffle (Python 3.11+)
    pub fn shuffle<T>(&mut self, x: &mut [T]) {
        for i in (1..x.len()).rev() {
            let j = self.randbelow(i + 1);
            x.swap(i, j);
        }
    }

    /// random.sample(population, k) (Python 3.11+)
    pub fn sample<T: Clone>(&mut self, population: &[T], k: usize) -> Vec<T> {
        let n = population.len();
        assert!(k <= n, "Sample larger than population");
        let mut setsize = 21usize;
        if k > 5 {
            setsize += 4usize.pow(((k * 3) as f64).log(4.0).ceil() as u32);
        }
        let mut result = Vec::with_capacity(k);
        if n <= setsize {
            let mut pool = population.to_vec();
            for i in 0..k {
                let j = self.randbelow(n - i);
                result.push(pool[j].clone());
                pool[j] = pool[n - i - 1].clone();
            }
        } else {
            let mut selected = std::collections::HashSet::new();
            for _ in 0..k {
                let mut j = self.randbelow(n);
                while selected.contains(&j) {
                    j = self.randbelow(n);
                }
                selected.insert(j);
                result.push(population[j].clone());
            }
        }
        result
    }
}

/// pools.draw_seats: k opponents drawn without replacement from pool_keys, plus `me` (if given), in a random order.
/// Depends only on the seed, the pool and me's key, never on anyone's card list, so two versions of a deck see
/// exactly the same opponents in the same seats.
pub fn draw_seats(seed: u64, pool_keys: &[String], me: Option<&str>, k: usize) -> Vec<String> {
    let mut r = PyRandom::from_str(&format!("pool:{seed}"));
    let mut sorted = pool_keys.to_vec();
    sorted.sort();
    let mut seats = r.sample(&sorted, k);
    if let Some(me) = me {
        seats.push(me.to_string());
    }
    r.shuffle(&mut seats);
    seats
}

// ------------------------------------------------------------------ SHA-512 (FIPS 180-4), for the seeding
const K: [u64; 80] = [
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc, 0x3956c25bf348b538,
    0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118, 0xd807aa98a3030242, 0x12835b0145706fbe,
    0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2, 0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235,
    0xc19bf174cf692694, 0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
    0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5, 0x983e5152ee66dfab,
    0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4, 0xc6e00bf33da88fc2, 0xd5a79147930aa725,
    0x06ca6351e003826f, 0x142929670a0e6e70, 0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed,
    0x53380d139d95b3df, 0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
    0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30, 0xd192e819d6ef5218,
    0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8, 0x19a4c116b8d2d0c8, 0x1e376c085141ab53,
    0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8, 0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373,
    0x682e6ff3d6b2b8a3, 0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b, 0xca273eceea26619c,
    0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178, 0x06f067aa72176fba, 0x0a637dc5a2c898a6,
    0x113f9804bef90dae, 0x1b710b35131c471b, 0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc,
    0x431d67c49c100d4c, 0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
];

pub fn sha512(msg: &[u8]) -> [u8; 64] {
    let mut h: [u64; 8] = [
        0x6a09e667f3bcc908,
        0xbb67ae8584caa73b,
        0x3c6ef372fe94f82b,
        0xa54ff53a5f1d36f1,
        0x510e527fade682d1,
        0x9b05688c2b3e6c1f,
        0x1f83d9abfb41bd6b,
        0x5be0cd19137e2179,
    ];
    let mut data = msg.to_vec();
    data.push(0x80);
    while data.len() % 128 != 112 {
        data.push(0);
    }
    data.extend_from_slice(&((msg.len() as u128) * 8).to_be_bytes());
    for block in data.chunks(128) {
        let mut w = [0u64; 80];
        for t in 0..16 {
            w[t] = u64::from_be_bytes(block[t * 8..t * 8 + 8].try_into().unwrap());
        }
        for t in 16..80 {
            let s0 = w[t - 15].rotate_right(1) ^ w[t - 15].rotate_right(8) ^ (w[t - 15] >> 7);
            let s1 = w[t - 2].rotate_right(19) ^ w[t - 2].rotate_right(61) ^ (w[t - 2] >> 6);
            w[t] = w[t - 16].wrapping_add(s0).wrapping_add(w[t - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for t in 0..80 {
            let s1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[t]).wrapping_add(w[t]);
            let s0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *x = x.wrapping_add(y);
        }
    }
    let mut out = [0u8; 64];
    for (i, x) in h.iter().enumerate() {
        out[i * 8..i * 8 + 8].copy_from_slice(&x.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    #[test]
    fn sha512_matches_hashlib() {
        assert!(hex(&sha512(b"abc")).starts_with("ddaf35a193617abacc417349ae204131"));
        assert!(hex(&sha512(&[b'x'; 200])).starts_with("ef978e23dc520404ae16fd17bde9ee59"));
    }

    #[test]
    fn string_seeded_stream_matches_python() {
        // random.Random('pool:500000').getrandbits(32) x 3, and random.Random('') x 2 (Python 3.12)
        let mut r = PyRandom::from_str("pool:500000");
        assert_eq!([r.genrand_u32(), r.genrand_u32(), r.genrand_u32()], [4137951970, 1747543397, 1875973494]);
        let mut r = PyRandom::from_str("");
        assert_eq!([r.genrand_u32(), r.genrand_u32()], [4124137760, 1951988028]);
    }

    #[test]
    fn seat_draws_match_python() {
        // pools.draw_seats(seed, ['b','a','c','e','d'], 'me', 3) in Python 3.12
        let keys: Vec<String> = ["b", "a", "c", "e", "d"].iter().map(|s| s.to_string()).collect();
        let cases = [
            (500000, ["me", "d", "e", "a"]),
            (500001, ["a", "me", "e", "c"]),
            (500002, ["e", "c", "a", "me"]),
            (123, ["b", "a", "me", "e"]),
            (999999, ["a", "e", "c", "me"]),
        ];
        for (seed, want) in cases {
            assert_eq!(draw_seats(seed, &keys, Some("me"), 3), want, "seed {seed}");
        }
    }
}
