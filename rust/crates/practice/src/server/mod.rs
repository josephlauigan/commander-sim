//! The practice server (Python's `play/server.py`): a thin HTTP layer on `std::net` + `httparse`, a thread per
//! connection, SSE for the event stream, static files from a folder next to the data. `start(Config)` is what the
//! shell calls; the session behind it comes from a `SessionFactory`.

pub mod api;
pub mod http;
pub mod hub;
pub mod lan;
pub mod routes;
pub mod sse;

use crate::images::Images;
use api::{NoSessions, SessionFactory};
use http::{Conn, ReadError, Response};
use hub::Hub;
use routes::{Reply, Site, dispatch};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Where things are and how the server listens.
#[derive(Clone, Debug)]
pub struct Config {
    /// the card data folder (decks.json, ...); card images live in `<data>/images`
    pub data: PathBuf,
    /// saved games (the autosave too)
    pub saves: PathBuf,
    /// the page's files (index.html, app.js, ...)
    pub static_dir: PathBuf,
    /// listen on the local network too (for a second player)
    pub lan: bool,
    /// the iPad app's server
    pub app: bool,
    /// 0: any free port
    pub port: u16,
}

pub struct ServerHandle {
    pub port: u16,
    pub hub: Arc<Hub>,
    listener_addr: std::net::SocketAddr,
}

impl ServerHandle {
    /// end the game and stop accepting connections
    pub fn stop(&self) {
        self.hub.shutdown();
        self.hub.wake();
        // unblock the accept loop
        let _ = TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], self.listener_addr.port())),
            Duration::from_secs(1),
        );
    }
}

/// The server with the real sessions. (practice-session's factory replaces `NoSessions` here when it lands.)
pub fn start(cfg: Config) -> Result<ServerHandle, String> {
    start_with(cfg, Arc::new(NoSessions))
}

pub fn start_with(cfg: Config, factory: Arc<dyn SessionFactory>) -> Result<ServerHandle, String> {
    let host = if cfg.lan { "0.0.0.0" } else { "127.0.0.1" };
    let listener = TcpListener::bind((host, cfg.port)).map_err(|e| e.to_string())?;
    let addr = listener.local_addr().map_err(|e| e.to_string())?;
    let images = Images::new(cfg.data.join("images"));
    let hub = Hub::new(cfg.lan, cfg.app, cfg.saves.clone(), images, factory);
    let site = Arc::new(Site::new(hub.clone(), cfg.data.clone(), cfg.static_dir.clone()));
    let h = hub.clone();
    std::thread::Builder::new()
        .name("practice-server".into())
        .spawn(move || {
            for stream in listener.incoming() {
                if h.stopped() {
                    break;
                }
                let Ok(stream) = stream else { continue };
                let site = site.clone();
                let _ = std::thread::Builder::new().name("practice-conn".into()).spawn(move || serve(&site, stream));
            }
        })
        .map_err(|e| e.to_string())?;
    Ok(ServerHandle { port: addr.port(), hub, listener_addr: addr })
}

/// at start-up, in the background: the commanders' images for the setup screen (once; cached after)
pub fn fetch_commanders(handle: &ServerHandle, data: &std::path::Path) {
    if std::env::var_os("PRACTICE_NO_FETCH").is_some() {
        return;
    }
    let (images, data) = (handle.hub.images.clone(), data.to_path_buf());
    let _ = std::thread::Builder::new().name("practice-commanders".into()).spawn(move || {
        if let Ok(cat) = crate::catalog::catalog(&data) {
            let _ = images.prepare_online(&crate::catalog::commanders(&cat), &mut |_, _| {});
        }
    });
}

fn serve(site: &Site, stream: TcpStream) {
    let peer = stream.peer_addr().ok();
    let _ = stream.set_read_timeout(Some(Duration::from_secs(120)));
    let _ = stream.set_nodelay(true);
    let Ok(mut out) = stream.try_clone() else { return };
    let mut conn = Conn::new(stream);
    loop {
        let req = match conn.read_request(peer) {
            Ok(r) => r,
            Err(ReadError::Closed) => return,
            Err(ReadError::Bad(code, msg)) => {
                let _ = Response::json(code, &serde_json::json!({"error": msg})).write_to(&mut out, false);
                return;
            }
        };
        let keep = req.keep_alive();
        match dispatch(site, &req) {
            Reply::Respond(r) => {
                if r.write_to(&mut out, keep).is_err() || !keep {
                    return;
                }
            }
            Reply::Events { token, local, since, new_token } => {
                sse::stream(&mut out, &site.hub, &token, local, since, new_token.as_deref());
                return;
            }
        }
    }
}
