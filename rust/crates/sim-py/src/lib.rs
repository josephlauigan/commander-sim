//! The Python module `rust_sim`: the Rust engine called from the Python run harness (poolmode.py, compare.py).
//! Built by rust/build-py.sh, which copies the library to commander_sim/rust_sim.so.

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use sim_core::run::{Engine, settings, summary};
use std::path::Path;
use std::sync::OnceLock;

/// The number of card definitions in a cards.json export (a smoke test of the bridge).
#[pyfunction]
fn card_count(path: &str) -> PyResult<usize> {
    sim_core::export::load_cards(Path::new(path)).map(|c| c.len()).map_err(|e| PyIOError::new_err(e.to_string()))
}

/// the card data and card code, loaded on the first game (from the data directory it names)
fn engine(data: &str) -> PyResult<&'static Engine> {
    static E: OnceLock<Result<Engine, String>> = OnceLock::new();
    E.get_or_init(|| Engine::load(Path::new(data))).as_ref().map_err(|e| PyIOError::new_err(e.clone()))
}

/// play_game(data, seed, seats, profile, ai, temp, max_rounds, trace) -> a JSON summary of the game.
/// seats: a JSON list of [deck key, [card names], commander name] in turn order (ais.play_pool_game's seats).
#[pyfunction]
#[pyo3(signature = (data, seed, seats, profile, ai, temp=1.0, max_rounds=20, trace=false))]
#[allow(clippy::too_many_arguments)]
fn play_game(
    py: Python<'_>,
    data: &str,
    seed: u64,
    seats: &str,
    profile: &str,
    ai: &str,
    temp: f64,
    max_rounds: u32,
    trace: bool,
) -> PyResult<String> {
    let e = engine(data)?;
    let specs: Vec<(String, Vec<String>, String)> =
        serde_json::from_str(seats).map_err(|err| PyValueError::new_err(format!("seats: {err}")))?;
    let seats = specs.iter().map(|(k, cs, cmd)| e.seat(k, cs, cmd)).collect::<Result<Vec<_>, _>>();
    let seats = seats.map_err(PyValueError::new_err)?;
    let st = settings(profile, ai, temp).map_err(PyValueError::new_err)?;
    let out = py.detach(|| summary(&e.play(seed, &seats, st, max_rounds, trace)));
    serde_json::to_string(&out).map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pymodule]
fn rust_sim(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(card_count, m)?)?;
    m.add_function(wrap_pyfunction!(play_game, m)?)?;
    Ok(())
}
