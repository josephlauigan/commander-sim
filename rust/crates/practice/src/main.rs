//! commander-practice: practice mode's server on a computer (documents/rust-rewrite-scope.md, phase 11). For now it
//! serves the iOS proof's page (see lib.rs), so the same server can be tried without an iPad.
//!     cargo run --release -p practice -- [--data DIR] [--port N]

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let data = arg("--data")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data"));
    let port: u16 = arg("--port").and_then(|p| p.parse().ok()).unwrap_or(8765);
    match practice::start(&data, port) {
        Ok(port) => {
            println!("Commander Practice (Rust): http://127.0.0.1:{port}");
            loop {
                std::thread::park();
            }
        }
        Err(e) => {
            eprintln!("practice: can't start the server: {e}");
            std::process::exit(1);
        }
    }
}
