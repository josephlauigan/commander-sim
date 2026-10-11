//! Python's `sum()` of floats. Since Python 3.12 it adds with Neumaier compensation, so its result can differ from a
//! plain left-to-right sum in the last bit (0 + 1.5 + 0.8 + 1.55 is 3.85 in Python, 3.8499999999999996 added
//! naively). The AI compares such sums (threat, position strength), and a last-bit difference breaks a tie the other
//! way, so the Rust uses `psum()` wherever the Python calls `sum()` on floats; a Python loop with `+=` stays a plain
//! sum.

pub trait PySum {
    fn psum(self) -> f64;
}

impl<I: Iterator<Item = f64>> PySum for I {
    fn psum(self) -> f64 {
        let (mut f, mut c) = (0.0f64, 0.0f64);
        for x in self {
            let t = f + x;
            if f.abs() >= x.abs() {
                c += (f - t) + x;
            } else {
                c += (x - t) + f;
            }
            f = t;
        }
        if c != 0.0 && c.is_finite() { f + c } else { f }
    }
}

#[cfg(test)]
mod tests {
    use super::PySum;

    #[test]
    fn matches_python() {
        assert_eq!([0.0, 1.5, 0.0, 0.8, 1.55].into_iter().psum(), 3.85);
        assert_eq!([0.1; 10].into_iter().psum(), 1.0); // Python 3.12: sum([0.1] * 10) == 1.0
        assert_eq!(std::iter::empty::<f64>().psum(), 0.0);
    }
}
