//! The event stream (`GET /api/events`): server-sent events, each with an id; a reconnect with `Last-Event-ID` (or
//! `?since=N`) resumes after it. Idle streams get `: keep-alive` comments every 15 s.

use super::http::set_cookie;
use super::hub::Hub;
use serde_json::Value;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

pub const KEEP_ALIVE: Duration = Duration::from_secs(15);

/// a history event as `seat` sees it, or None: another seat's request is only who is deciding, another seat's
/// messages and secrets are left out, and each event carries this seat's view of the table
pub fn for_seat(ev: &Value, seat: &str) -> Option<Value> {
    if let Some(s) = ev.get("seat")
        && s.as_str() != Some(seat)
    {
        return (ev["kind"] == "request")
            .then(|| serde_json::json!({"id": ev["id"], "kind": "waiting", "who": ev["who"]}));
    }
    if let Some(h) = ev.get("hide")
        && !h.is_null()
        && h.as_str() == Some(seat)
    {
        return None;
    }
    let mut out = ev.as_object().cloned().unwrap_or_default();
    for k in ["seat", "hide", "views", "who"] {
        out.remove(k);
    }
    if let Some(v) = ev["views"].get(seat)
        && !v.is_null()
    {
        out.insert("view".into(), v.clone());
    }
    Some(Value::Object(out))
}

/// a page catching up on many events at once (a reload, a reconnect): every action carries the table it left, but
/// only the last table matters, so the others are left out
pub fn catch_up(evs: Vec<Value>, pending: Option<&Value>) -> Vec<Value> {
    let last = evs.iter().rposition(|e| e.get("view").is_some());
    evs.into_iter()
        .enumerate()
        .map(|(i, mut e)| {
            let keep = Some(i) == last || pending.is_some_and(|p| p["id"] == e["id"]);
            if !keep && let Some(o) = e.as_object_mut() {
                o.remove("view");
            }
            e
        })
        .collect()
}

/// the SSE response head
pub fn head(new_token: Option<&str>) -> String {
    let mut h = String::from(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: close\r\n",
    );
    if let Some(t) = new_token {
        h += &format!("Set-Cookie: {}\r\n", set_cookie(t));
    }
    h + "\r\n"
}

/// stream the events to a browser until it goes away (or the server stops)
pub fn stream(w: &mut impl Write, hub: &Arc<Hub>, token: &str, local: bool, mut since: u64, new_token: Option<&str>) {
    stream_with(w, hub, token, local, &mut since, new_token, KEEP_ALIVE)
}

pub fn stream_with(
    w: &mut impl Write,
    hub: &Arc<Hub>,
    token: &str,
    local: bool,
    since: &mut u64,
    new_token: Option<&str>,
    wait: Duration,
) {
    if w.write_all(head(new_token).as_bytes()).and_then(|_| w.flush()).is_err() {
        return;
    }
    while !hub.stopped() {
        let evs = hub.events_after(*since, wait);
        let io = if evs.is_empty() {
            w.write_all(b": keep-alive\n\n").and_then(|_| w.flush())
        } else {
            *since = evs.last().and_then(|e| e["id"].as_u64()).unwrap_or(*since);
            let Some(seat) = hub.seat_of(token, local) else { continue }; // each time: the page may join a game meanwhile
            let mut evs: Vec<Value> = evs.iter().filter_map(|e| for_seat(e, &seat)).collect();
            if evs.len() > 20 {
                evs = catch_up(evs, hub.pending_of(&seat).as_ref());
            }
            let mut buf = String::new();
            for e in &evs {
                buf += &format!("id: {}\ndata: {}\n\n", e["id"], e);
            }
            w.write_all(buf.as_bytes()).and_then(|_| w.flush())
        };
        if io.is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn other_seats_requests_are_only_who() {
        let ev = json!({"id": 3, "kind": "request", "seat": "b", "who": "Bee", "request": {"kind": "main"}});
        assert_eq!(for_seat(&ev, "a"), Some(json!({"id": 3, "kind": "waiting", "who": "Bee"})));
        let own = for_seat(&ev, "b").unwrap();
        assert_eq!(own, json!({"id": 3, "kind": "request", "request": {"kind": "main"}}));
        let secret = json!({"id": 4, "kind": "log", "seat": "b", "text": "x"});
        assert_eq!(for_seat(&secret, "a"), None);
    }

    #[test]
    fn hidden_from_one_seat_and_views_by_seat() {
        let ev = json!({"id": 5, "kind": "log", "hide": "a", "text": "t", "views": {"a": {"x": 1}, "b": {"x": 2}}});
        assert_eq!(for_seat(&ev, "a"), None);
        assert_eq!(
            for_seat(&ev, "b").unwrap(),
            json!({"id": 5, "kind": "log", "hide": "a", "text": "t", "view": {"x": 2}})
                .as_object()
                .map(|o| {
                    let mut o = o.clone();
                    o.remove("hide");
                    Value::Object(o)
                })
                .unwrap()
        );
    }

    #[test]
    fn catching_up_keeps_the_last_table_and_the_pending_one() {
        let evs: Vec<Value> = (1..=5).map(|i| json!({"id": i, "view": {"n": i}})).collect();
        let out = catch_up(evs, Some(&json!({"id": 2})));
        let with: Vec<u64> =
            out.iter().filter(|e| e.get("view").is_some()).map(|e| e["id"].as_u64().unwrap()).collect();
        assert_eq!(with, vec![2, 5]);
    }

    struct Flaky(Vec<u8>, usize);
    impl Write for Flaky {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if self.1 == 0 {
                return Err(std::io::ErrorKind::BrokenPipe.into());
            }
            self.1 -= 1;
            self.0.extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn an_idle_stream_sends_keep_alives_and_ends_when_the_browser_goes() {
        use crate::images::Images;
        use crate::server::api::NoSessions;
        let hub = Hub::new(false, false, std::env::temp_dir(), Images::new(std::env::temp_dir()), Arc::new(NoSessions));
        let mut w = Flaky(vec![], 4); // the head, its flush is not counted separately: three keep-alives, then a broken pipe
        let mut since = 0;
        stream_with(&mut w, &hub, "tok", true, &mut since, Some("newtok"), Duration::from_millis(10));
        let s = String::from_utf8(w.0).unwrap();
        assert!(s.starts_with("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream"));
        assert!(s.contains("Set-Cookie: practice_seat=newtok;"));
        assert_eq!(s.matches(": keep-alive\n\n").count(), 3);
    }
}
