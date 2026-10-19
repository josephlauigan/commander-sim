//! Practice mode's server, in Rust. The commander-practice binary (`main.rs`) runs it on a computer; the iPad app
//! (`ios-shell/`) links this library as a static library and calls the C functions below: `practice_start` starts the
//! server on the device and the app shows `http://127.0.0.1:<port>` in a web view.

pub mod advisor;
pub mod bundle;
pub mod catalog;
pub mod images;
pub mod server;
pub mod session;
pub mod shadow;
pub mod text;
pub mod view;

use server::{Config, ServerHandle};
use session::api_impl::RealSessions;
use sim_core::run::Engine;
use std::ffi::{CStr, c_char};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// the running server (the iPad app has one) and what it needs to restart
struct Running {
    handle: ServerHandle,
    cfg: Config,
    engine: Arc<Engine>,
}

static RUNNING: Mutex<Option<Running>> = Mutex::new(None);

fn running() -> std::sync::MutexGuard<'static, Option<Running>> {
    RUNNING.lock().unwrap_or_else(|e| e.into_inner())
}

/// Start the app's server: copy the bundled data into `app_dir` when this build's is new, load the engine from
/// there and listen on 127.0.0.1:`port` (0: any free port). Returns the port. A server already running is stopped
/// first. For Rust callers (the C function and the tests).
pub fn start_app(bundle_dir: &Path, app_dir: &Path, port: u16) -> Result<u16, String> {
    let data = app_dir.join("data");
    std::fs::create_dir_all(app_dir).map_err(|e| format!("{}: {e}", app_dir.display()))?;
    bundle::sync_data(bundle_dir, app_dir).map_err(|e| format!("copying the bundled data: {e}"))?;
    let engine = Arc::new(Engine::load(&data)?);
    let cfg = Config {
        saves: app_dir.join("saves"),
        data: data.clone(),
        static_dir: bundle_dir.join("static"),
        lan: false,
        app: true,
        port,
    };
    let mut slot = running();
    if let Some(old) = slot.take() {
        old.handle.stop();
    }
    let handle = server::start_with(cfg.clone(), Arc::new(RealSessions { engine: engine.clone() }))?;
    server::fetch_commanders(&handle, &data);
    let port = handle.port;
    *slot = Some(Running { handle, cfg, engine });
    Ok(port)
}

/// Stop the server (it ends the game in progress; the autosave stays for *Continue*). Returns whether one ran.
pub fn stop_app() -> bool {
    match running().take() {
        Some(r) => {
            r.handle.stop();
            true
        }
        None => false,
    }
}

/// Restart the server listening on the local network too (`lan`) or on this device only. Because the old listener
/// may still hold its port for a moment, the new server takes a free port: returns it.
pub fn set_lan_app(lan: bool) -> Result<u16, String> {
    let mut slot = running();
    let r = slot.as_mut().ok_or("the server isn't running")?;
    if r.cfg.lan == lan {
        return Ok(r.handle.port);
    }
    r.handle.stop();
    let cfg = Config { lan, port: 0, ..r.cfg.clone() };
    let handle = server::start_with(cfg.clone(), Arc::new(RealSessions { engine: r.engine.clone() }))?;
    r.handle = handle;
    r.cfg = cfg;
    Ok(r.handle.port)
}

fn path_arg(p: *const c_char) -> Option<PathBuf> {
    if p.is_null() {
        return None;
    }
    Some(PathBuf::from(unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()))
}

/// Start practice mode's server inside the app. `bundle_dir` is the app bundle's read-only `Resources` (`data/`,
/// `decklists/`, `static/`), `app_dir` the writable Application Support folder (the data is copied to `app_dir/data`,
/// saved games go to `app_dir/saves`). Listens on 127.0.0.1:`port` (0: any free port). Returns the port, or 0 if it
/// couldn't start (the reason goes to stderr).
///
/// # Safety
/// `bundle_dir` and `app_dir` must be valid NUL-terminated strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn practice_start(bundle_dir: *const c_char, app_dir: *const c_char, port: u16) -> u16 {
    let (Some(b), Some(a)) = (path_arg(bundle_dir), path_arg(app_dir)) else {
        eprintln!("practice_start: a folder is missing");
        return 0;
    };
    start_app(&b, &a, port).unwrap_or_else(|e| {
        eprintln!("practice_start: {e}");
        0
    })
}

/// Stop the server. Safe to call when none runs.
#[unsafe(no_mangle)]
pub extern "C" fn practice_stop() {
    stop_app();
}

/// The hidden two-player option: listen on the local network too (`true`) or on this device only. Restarts the
/// server on a new port and returns it (the app reloads its page from there), or 0 on failure.
#[unsafe(no_mangle)]
pub extern "C" fn practice_set_lan(lan: bool) -> u16 {
    set_lan_app(lan).unwrap_or_else(|e| {
        eprintln!("practice_set_lan: {e}");
        0
    })
}
