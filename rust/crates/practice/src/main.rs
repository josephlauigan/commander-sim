//! commander-practice: practice mode's server on a computer (Python's `python3 -m commander_sim.play`).
//!     cargo run --release -p practice -- [--port 8765] [--no-browser] [--lan] [--data DIR] [--static DIR]
//! `--text` (the terminal table) and its options (--deck --tier --seed --seat --opponents --profile --ai --step) are
//! accepted and checked but the terminal table is not ported yet.

use practice::server::{self, Config, lan::lan_addresses};
use practice::session::api_impl::RealSessions;
use sim_core::run::Engine;
use std::path::PathBuf;
use std::sync::Arc;

const USAGE: &str = "usage: commander-practice [--port N] [--no-browser] [--lan] [--data DIR] [--static DIR]\n\
    \x20      [--text --deck KEY --tier T --seed N --seat 1-4 --opponents A,B,C --profile conservative|loose --ai lookahead|adaptive --step]";

fn die(msg: &str) -> ! {
    eprintln!("commander-practice: {msg}\n{USAGE}");
    std::process::exit(2);
}

fn main() {
    let mut args = std::env::args();
    args.next();
    let (mut port, mut no_browser, mut lan, mut text) = (8765u16, false, false, false);
    let (mut data, mut static_dir): (Option<PathBuf>, Option<PathBuf>) = (None, None);
    while let Some(a) = args.next() {
        let (flag, inline) = match a.split_once('=') {
            Some((f, v)) if f.starts_with("--") => (f.to_string(), Some(v.to_string())),
            _ => (a.clone(), None),
        };
        let value = |args: &mut std::env::Args| {
            inline.clone().or_else(|| args.next()).unwrap_or_else(|| die(&format!("{flag} needs a value")))
        };
        match flag.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return;
            }
            "--port" => port = value(&mut args).parse().unwrap_or_else(|_| die("--port needs a number")),
            "--no-browser" => no_browser = true,
            "--lan" => lan = true,
            "--text" => text = true,
            "--step" => {}
            "--data" => data = Some(PathBuf::from(value(&mut args))),
            "--static" => static_dir = Some(PathBuf::from(value(&mut args))),
            "--deck" | "--tier" | "--seed" | "--seat" | "--opponents" | "--profile" | "--ai" => {
                let v = value(&mut args);
                let ok = match flag.as_str() {
                    "--seed" => v.parse::<u64>().is_ok(),
                    "--seat" => matches!(v.as_str(), "1" | "2" | "3" | "4"),
                    "--profile" => matches!(v.as_str(), "conservative" | "loose"),
                    "--ai" => matches!(v.as_str(), "lookahead" | "adaptive"),
                    _ => true,
                };
                if !ok {
                    die(&format!("bad value {v:?} for {flag}"));
                }
            }
            _ => die(&format!("unrecognized argument {a:?}")),
        }
    }
    if text {
        eprintln!("commander-practice: the terminal table (--text) is not ported yet; use the browser table");
        std::process::exit(2);
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let data =
        data.unwrap_or_else(|| if PathBuf::from("data/decks.json").exists() { "data".into() } else { root.join("data") });
    let static_dir = static_dir.unwrap_or_else(|| {
        let near = data.join("../commander_sim/play/static");
        if near.join("index.html").exists() { near } else { root.join("commander_sim/play/static") }
    });
    if !static_dir.join("index.html").exists() {
        die(&format!("no index.html in {} (use --static DIR)", static_dir.display()));
    }
    let engine = match Engine::load(&data) {
        Ok(e) => Arc::new(e),
        Err(e) => {
            eprintln!("commander-practice: can't load the card data from {}: {e}", data.display());
            std::process::exit(1);
        }
    };
    let cfg = Config { saves: data.join("saves"), data: data.clone(), static_dir, lan, app: false, port };
    let handle = match server::start_with(cfg, Arc::new(RealSessions { engine })) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("commander-practice: can't start the server on port {port}: {e}");
            std::process::exit(1);
        }
    };
    server::fetch_commanders(&handle, &data);
    let url = format!("http://127.0.0.1:{}/", handle.port);
    println!("Practice mode: {url}  (Ctrl+C to stop)");
    if lan {
        let addrs = lan_addresses();
        let p = handle.port;
        let open = if addrs.is_empty() {
            format!("this computer's address, port {p}")
        } else {
            addrs.iter().map(|a| format!("http://{a}:{p}/")).collect::<Vec<_>>().join(" or ")
        };
        println!(
            "Listening on the local network too. Your friend opens {open} (the join code shows when you open a two-player table)."
        );
        println!(
            "If their page does not load, allow the port through this computer's firewall (e.g. sudo ufw allow {p}/tcp). \
             Anyone on the network can open the page; only someone with the code can take the second seat."
        );
    }
    if !no_browser {
        let u = url.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(500));
            for opener in ["xdg-open", "open"] {
                let quiet = std::process::Stdio::null;
                if std::process::Command::new(opener).arg(&u).stdout(quiet()).stderr(quiet()).spawn().is_ok() {
                    break;
                }
            }
        });
    }
    // Ctrl+C ends the process (the autosave is written as the game goes)
    loop {
        std::thread::park();
    }
}
