//! Practice mode's server, in Rust (documents/rust-rewrite-scope.md, phases 9–13). For now (M6) it is the iOS proof:
//! the iPad app (ios-shell/) links this library, calls `practice_start` to start a server on the device, and shows
//! `http://127.0.0.1:<port>` in a web view, as the practice app will. The page plays games with the Rust engine, so
//! it shows the engine and its card data working on the device. The table, the JSON API, saving and the rest come
//! with phases 9–13.

use sim_core::run::{Engine, settings, summary};
use std::ffi::{CStr, c_char};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

/// The server's state: where the card data is, the engine once loaded, and the next game's seed.
struct Server {
    data: PathBuf,
    engine: OnceLock<Result<Engine, String>>,
    seed: AtomicU64,
}

impl Server {
    fn engine(&self) -> Result<&Engine, String> {
        self.engine.get_or_init(|| Engine::load(&self.data)).as_ref().map_err(|e| e.clone())
    }
}

/// Start the server on 127.0.0.1:`port` (0: any free port) in a background thread, with the card data
/// (cards.json, decks.json) in `data_dir`. Returns the port it listens on, or 0 if it couldn't start.
///
/// # Safety
/// `data_dir` must be a valid NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn practice_start(data_dir: *const c_char, port: u16) -> u16 {
    if data_dir.is_null() {
        return 0;
    }
    let data = unsafe { CStr::from_ptr(data_dir) }.to_string_lossy().into_owned();
    start(Path::new(&data), port).unwrap_or(0)
}

/// `practice_start` for Rust callers (the commander-practice binary)
pub fn start(data: &Path, port: u16) -> Result<u16, String> {
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let server = Arc::new(Server { data: data.to_path_buf(), engine: OnceLock::new(), seed: AtomicU64::new(500_000) });
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let server = server.clone();
            std::thread::spawn(move || {
                let _ = serve(&server, stream);
            });
        }
    });
    Ok(port)
}

fn serve(server: &Server, mut stream: TcpStream) -> std::io::Result<()> {
    let mut line = String::new();
    let mut reader = BufReader::new(stream.try_clone()?);
    reader.read_line(&mut line)?;
    loop {
        // the rest of the request's headers
        let mut h = String::new();
        if reader.read_line(&mut h)? == 0 || h == "\r\n" || h == "\n" {
            break;
        }
    }
    let path = line.split_whitespace().nth(1).unwrap_or("/");
    let (status, kind, body) = match path {
        "/" => ("200 OK", "text/html; charset=utf-8", PAGE.to_string()),
        "/game" => match play(server) {
            Ok(json) => ("200 OK", "application/json", json),
            Err(e) => ("500 Internal Server Error", "application/json", serde_json::json!({ "error": e }).to_string()),
        },
        _ => ("404 Not Found", "text/plain", "not found".to_string()),
    };
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body.as_bytes())
}

/// one game of Sauron against three Tier 1 decks with the look-ahead AI, as JSON: who won, how, and how long it took
fn play(server: &Server) -> Result<String, String> {
    let e = server.engine()?;
    let seed = server.seed.fetch_add(1, Ordering::Relaxed);
    let mut keys: Vec<&str> = vec!["sauron"];
    let mut t1: Vec<&str> = e.decks.values().filter(|d| d.tier.as_deref() == Some("t1")).map(|d| &*d.key).collect();
    t1.sort();
    let k = (seed as usize) % t1.len();
    keys.extend((0..3).map(|i| t1[(k + i) % t1.len()]));
    let seats = keys
        .iter()
        .map(|&k| {
            let d = &e.decks[k];
            e.seat(k, &d.cards, &d.commander)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let t0 = Instant::now();
    let g = e.play(seed, &seats, settings("loose", "lookahead", 1.0)?, 20, false);
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    let s = summary(&g);
    let names: Vec<String> = keys.iter().map(|&k| e.decks[k].name.clone()).collect();
    let winner = s.winner.as_ref().map(|w| e.decks[w].name.clone());
    Ok(serde_json::json!({
        "seed": seed, "seats": names, "winner": winner, "wintype": s.wintype, "rounds": s.round,
        "decisions": s.search_n, "ms": ms.round(), "cards": e.db.len(),
    })
    .to_string())
}

/// The page: true black, as the iPad app is. It asks /game for a game and shows how it went.
const PAGE: &str = r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>Commander Practice</title>
<style>
  :root { color-scheme: dark; }
  body { margin: 0; background: #000; color: #e8e6e3; font: 17px/1.5 -apple-system, system-ui, sans-serif;
         padding: max(24px, env(safe-area-inset-top)) 24px 24px; }
  h1 { font-size: 26px; font-weight: 600; margin: 0 0 4px; }
  p.sub { color: #9a9790; margin: 0 0 24px; }
  button { font: inherit; color: #000; background: #e8e6e3; border: 0; border-radius: 10px; padding: 12px 20px; }
  button:disabled { opacity: .5; }
  ol { padding-left: 20px; } li { margin: 10px 0; }
  .muted { color: #9a9790; }
</style></head>
<body>
<h1>Commander Practice</h1>
<p class="sub">The Rust engine, running on this device. The practice table comes later; this page checks that the
engine and its card data work here.</p>
<button id="go">Play a game</button>
<ol id="games"></ol>
<script>
const go = document.getElementById('go'), list = document.getElementById('games');
go.onclick = async () => {
  go.disabled = true; go.textContent = 'Playing…';
  try {
    const r = await (await fetch('/game')).json();
    const li = document.createElement('li');
    li.innerHTML = r.error ? 'Error: ' + r.error :
      `<b>${r.winner ?? 'No one'}</b> won (${r.wintype ?? 'damage'}) in ${r.rounds} rounds` +
      `<br><span class="muted">${r.seats.join(', ')} · seed ${r.seed} · ${r.decisions} look-ahead decisions · ` +
      `${(r.ms / 1000).toFixed(1)} s · ${r.cards} cards loaded</span>`;
    list.prepend(li);
  } finally { go.disabled = false; go.textContent = 'Play a game'; }
};
</script>
</body></html>
"#;
