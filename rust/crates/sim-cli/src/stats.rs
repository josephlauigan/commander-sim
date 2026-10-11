//! The statistics the reports use, computed the way Python computed them so the printed numbers are the same:
//! an insertion-ordered counter (collections.Counter, whose most_common keeps first-seen order among ties),
//! Wilson intervals, Python 3.12's compensated float sum, and the paired win-rate difference.

use std::collections::HashMap;
use std::hash::Hash;

/// collections.Counter: counts in first-insertion order
#[derive(Debug, Clone)]
pub struct Counter<K: Eq + Hash + Clone> {
    items: Vec<(K, i64)>,
    index: HashMap<K, usize>,
}

impl<K: Eq + Hash + Clone> Default for Counter<K> {
    fn default() -> Self {
        Counter { items: vec![], index: HashMap::new() }
    }
}

impl<K: Eq + Hash + Clone> Counter<K> {
    pub fn new() -> Self {
        Self::default()
    }

    /// c[k] += v (inserts k even when v is 0, as Python's `c[k] += 0` does)
    pub fn add(&mut self, k: K, v: i64) {
        match self.index.get(&k) {
            Some(&i) => self.items[i].1 += v,
            None => {
                self.index.insert(k.clone(), self.items.len());
                self.items.push((k, v));
            }
        }
    }

    /// c[k], 0 when missing
    pub fn get(&self, k: &K) -> i64 {
        self.index.get(k).map_or(0, |&i| self.items[i].1)
    }

    pub fn total(&self) -> i64 {
        self.items.iter().map(|x| x.1).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// most_common(): by count, highest first; ties keep insertion order
    pub fn most_common(&self) -> Vec<(K, i64)> {
        let mut v = self.items.clone();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    }

    /// keys sorted by -count (Python's `sorted(c, key=lambda k: -c[k])`)
    pub fn keys_by_count(&self) -> Vec<K> {
        self.most_common().into_iter().map(|x| x.0).collect()
    }
}

/// poolmode.wilson: the Wilson score interval for k successes in n trials
pub fn wilson(k: u64, n: u64) -> (f64, f64) {
    let z = 1.96f64;
    if n == 0 {
        return (0.0, 0.0);
    }
    let nf = n as f64;
    let p = k as f64 / nf;
    let d = 1.0 + z * z / nf;
    let c = (p + z * z / (2 * n) as f64) / d;
    let h = z * (p * (1.0 - p) / nf + z * z / (4 * n * n) as f64).sqrt() / d;
    (c - h, c + h)
}

/// Python 3.12's sum() of floats (start 0): Neumaier's compensated summation
pub fn py_fsum(xs: impl IntoIterator<Item = f64>) -> f64 {
    let mut it = xs.into_iter();
    let Some(first) = it.next() else { return 0.0 };
    let mut f = 0.0 + first;
    let mut c = 0.0f64;
    for x in it {
        let t = f + x;
        if f.abs() >= x.abs() {
            c += (f - t) + x;
        } else {
            c += (x - t) + f;
        }
        f = t;
    }
    if c != 0.0 && c.is_finite() {
        f += c;
    }
    f
}

/// poolmode.paired_delta: the mean win-rate change over the seeds both runs played, its standard error, and how many
/// pairs there were. Each item is (seed, baseline won, variant won), any order.
pub fn paired_delta(base: &HashMap<u64, bool>, var: &HashMap<u64, bool>) -> (f64, f64, usize) {
    let mut seeds: Vec<u64> = base.keys().filter(|s| var.contains_key(s)).copied().collect();
    seeds.sort();
    let d: Vec<i64> = seeds.iter().map(|s| var[s] as i64 - base[s] as i64).collect();
    let n = d.len();
    if n == 0 {
        return (0.0, 0.0, 0);
    }
    let m = d.iter().sum::<i64>() as f64 / n as f64;
    let var = py_fsum(d.iter().map(|&x| (x as f64 - m) * (x as f64 - m))) / n.saturating_sub(1).max(1) as f64;
    (m, (var / n as f64).sqrt(), n)
}

/// running sums for one strength axis (poolmode's S1, S2, N)
#[derive(Debug, Clone, Copy, Default)]
pub struct Moments {
    pub s1: f64,
    pub s2: f64,
    pub n: u64,
}

impl Moments {
    pub fn add(&mut self, x: f64) {
        self.s1 += x;
        self.s2 += x * x;
        self.n += 1;
    }

    /// {'mean', 'var', 'n'} as poolmode.run computes them
    pub fn mean(&self) -> f64 {
        self.s1 / self.n as f64
    }

    pub fn var(&self) -> f64 {
        let m = self.mean();
        (self.s2 / self.n as f64 - m * m).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_ties_keep_insertion_order() {
        let mut c = Counter::new();
        c.add("b", 2);
        c.add("a", 3);
        c.add("c", 2);
        c.add("d", 0);
        let mc: Vec<_> = c.most_common().into_iter().map(|x| x.0).collect();
        assert_eq!(mc, ["a", "b", "c", "d"]);
        assert_eq!(c.get(&"zz"), 0);
        assert_eq!(c.total(), 7);
    }

    #[test]
    fn wilson_matches_python() {
        // poolmode.wilson(192, 960) and (0, 10) in Python
        let (lo, hi) = wilson(192, 960);
        assert_eq!((format!("{:.12}", lo), format!("{:.12}", hi)), ("0.175914408227".into(), "0.226477022066".into()));
        let (lo, hi) = wilson(0, 10);
        assert_eq!(format!("{:.1}-{:.1}", 100.0 * lo, 100.0 * hi), "-0.0-27.8"); // Python prints -0.0 here too
        assert_eq!(wilson(0, 0), (0.0, 0.0));
    }

    #[test]
    fn compensated_sum_matches_python_312() {
        // sum([0.1] * 10) == 1.0 in Python 3.12 (0.9999999999999999 before)
        assert_eq!(py_fsum(std::iter::repeat_n(0.1, 10)), 1.0);
        assert_eq!(py_fsum([1e100, 1.0, -1e100]), 1.0);
        assert_eq!(py_fsum([]), 0.0);
    }

    #[test]
    fn paired_delta_by_seed() {
        let b: HashMap<u64, bool> = [(1, true), (2, false), (3, false), (4, true), (9, true)].into();
        let v: HashMap<u64, bool> = [(1, true), (2, true), (3, true), (4, false)].into();
        let (d, se, n) = paired_delta(&b, &v);
        assert_eq!(n, 4);
        assert!((d - 0.25).abs() < 1e-15);
        // d = [0, 1, 1, -1]: var = (0.0625 + 0.5625 * 2 + 1.5625) / 3
        assert!((se - ((0.0625 + 0.5625 * 2.0 + 1.5625) / 3.0 / 4.0f64).sqrt()).abs() < 1e-15);
    }

    #[test]
    fn moments() {
        let mut m = Moments::default();
        for x in [1.0, 0.0, 1.0, 1.0] {
            m.add(x);
        }
        assert_eq!((m.mean(), m.var(), m.n), (0.75, 0.1875, 4));
    }
}
