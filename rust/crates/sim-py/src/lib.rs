//! The Python module `rust_sim`: the Rust engine called from the Python run harness (poolmode.py, compare.py).
//! Built by rust/build-py.sh, which copies the library to commander_sim/rust_sim.so.

use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use std::path::Path;

/// The number of card definitions in a cards.json export (a smoke test of the bridge).
#[pyfunction]
fn card_count(path: &str) -> PyResult<usize> {
    sim_core::export::load_cards(Path::new(path)).map(|c| c.len()).map_err(|e| PyIOError::new_err(e.to_string()))
}

#[pymodule]
fn rust_sim(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(card_count, m)?)?;
    Ok(())
}
