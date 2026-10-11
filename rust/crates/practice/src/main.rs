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
    if args.iter().any(|a| a == "--text") {
        std::process::exit(text_game(&data, &arg));
    }
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

/// `--text`: play a practice game in the terminal (--deck --tier --seed --seat --opponents a,b,c --profile --ai --step)
fn text_game(data: &std::path::Path, arg: &dyn Fn(&str) -> Option<String>) -> i32 {
    use practice::session::{Options, Session};
    let engine = match sim_core::run::Engine::load(data) {
        Ok(e) => std::sync::Arc::new(e),
        Err(e) => {
            eprintln!("practice: can't load the card data: {e}");
            return 1;
        }
    };
    let mut o = Options::new(engine, &arg("--deck").unwrap_or("sauron".into()), &arg("--tier").unwrap_or("t1".into()));
    o.seed = arg("--seed").and_then(|s| s.parse().ok());
    o.seat = arg("--seat").and_then(|s| s.parse().ok());
    o.opponents = arg("--opponents").map(|s| s.split(',').map(str::to_string).collect());
    if let Some(p) = arg("--profile") {
        o.profile = p;
    }
    if let Some(a) = arg("--ai") {
        o.ai = a;
    }
    o.step = std::env::args().any(|a| a == "--step");
    let s = match Session::new(o) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("practice: {e}");
            return 1;
        }
    };
    let stdin = std::io::stdin();
    let ev = practice::text::run(&s, &mut stdin.lock(), &mut std::io::stdout());
    if ev["kind"] == "error" { 1 } else { 0 }
}
