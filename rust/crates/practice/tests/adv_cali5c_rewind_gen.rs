//! A rewind starts the game's next thread: the new thread must see its own generation (abort flag, answer queues) from its
//! first decision on. If `start()` installs the generation after spawning the thread, a replay that reaches its first
//! live decision at once reads the closed generation of the game before the rewind, answers Cancel and ends the game: the
//! session then shows `over` (or `over closed`) right after `reset` instead of the decision it rewound to.

use practice::session::{MY_DECKS, Options, Session};
use serde_json::{Value, json};
use sim_core::run::Engine;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

const LIMIT: Duration = Duration::from_secs(120);

/// the next request / over / error event after `cursor`
fn next(s: &Arc<Session>, cursor: &mut u64) -> Value {
    let end = Instant::now() + LIMIT;
    while Instant::now() < end {
        let (c, evs) = s.events_after(*cursor, Duration::from_millis(100));
        *cursor = c;
        for ev in evs {
            if matches!(ev["kind"].as_str(), Some("request" | "over" | "error")) {
                return ev;
            }
        }
    }
    panic!("no request in {LIMIT:?}");
}

#[test]
fn a_rewind_always_ends_at_a_decision_never_at_a_stale_over() {
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let e = Arc::new(Engine::load(&data).expect("load the card data"));
    let deck = MY_DECKS.iter().find(|k| e.format_of(k).seats == 2).expect("a 2-seat deck").to_string();
    let fmt = e.format_of(&deck).key.clone();
    let guest = MY_DECKS.iter().find(|k| **k != deck && e.admits(k, &fmt).is_ok()).expect("a guest deck").to_string();
    let next_job = AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..12 {
            sc.spawn(|| {
                loop {
                    let i = next_job.fetch_add(1, Ordering::SeqCst);
                    if i >= 120 {
                        return;
                    }
                    let mut o = Options::new(e.clone(), &deck, "");
                    o.partner = Some(guest.clone());
                    o.seed = Some(1 + (i as u64 % 3));
                    o.ai = "adaptive".into();
                    o.compare = false;
                    o.views = true;
                    o.max_rounds = 3;
                    let s = Session::new(o).unwrap();
                    s.start();
                    let mut cur = 0;
                    // keep the first two hands, then undo the second answer: a rewind to a live decision
                    let mut at = next(&s, &mut cur);
                    let mut seat = String::new();
                    for _ in 0..2 {
                        assert_eq!(at["kind"], "request", "{at}");
                        seat = at["seat"].as_str().unwrap().to_string();
                        s.answer(&seat, &json!("keep"));
                        at = next(&s, &mut cur);
                    }
                    assert_eq!(at["kind"], "request", "{at}");
                    assert!(s.undo(1, &seat).is_none());
                    let after = next(&s, &mut cur);
                    assert_eq!(after["kind"], "request", "job {i}: the rewind ended at {after}");
                    s.close(Duration::from_secs(10));
                }
            });
        }
    });
}
