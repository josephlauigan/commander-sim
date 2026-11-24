//! A refused `join` leaves the host's lobby open: the second person can try again with the same code. The hub is driven
//! directly with a factory that refuses a chosen partner deck, since the real factory's `check_two` has already vetted
//! everything `new_session` checks by the time a second person arrives.

use practice::images::Images;
use practice::server::api::{Info, SessionApi, SessionFactory};
use practice::server::hub::Hub;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::time::Duration;

const REFUSED: &str = "the engine refuses this pairing";

struct Fake {
    deck: String,
    partner: Option<String>,
}

impl SessionApi for Fake {
    fn info(&self) -> Info {
        Info {
            deck: self.deck.clone(),
            tier: "t2".into(),
            seed: 1,
            seats: vec![self.deck.clone(), "a".into(), "b".into(), "c".into()],
            profile: "loose".into(),
            ai: "adaptive".into(),
            humans: std::iter::once(self.deck.clone()).chain(self.partner.clone()).collect(),
            names: vec![],
            partner: self.partner.clone(),
            waiting: true,
            finished: false,
        }
    }
    fn autopass_of(&self, _: &str) -> String {
        "respond".into()
    }
    fn events_after(&self, cursor: u64, timeout: Duration) -> (u64, Vec<Value>) {
        std::thread::sleep(timeout.min(Duration::from_millis(20)));
        (cursor, vec![])
    }
    fn start(&self) {}
    fn answer(&self, _: &str, _: Value) {}
    fn undo(&self, _: i64, _: &str) -> Option<String> {
        None
    }
    fn try_it(&self, _: &Value, _: &str) -> Option<String> {
        None
    }
    fn undo_point(&self, _: i64, _: &str) -> Option<usize> {
        None
    }
    fn answer_seats(&self) -> Vec<String> {
        vec![]
    }
    fn has_try_entry(&self, _: &Value, _: &str) -> bool {
        false
    }
    fn set_autopass(&self, _: &str, _: &Value) -> Option<String> {
        None
    }
    fn hint(&self, _: &str) -> Result<Value, String> {
        Err("no".into())
    }
    fn review(&self, _: &str) -> Result<Value, String> {
        Err("no".into())
    }
    fn saved(&self, _: Option<Duration>) -> Result<Value, String> {
        Err("no".into())
    }
    fn close(&self, _: Duration) {}
}

/// refuses a game whose partner deck is `refuse` (what a `new_session` failure after the lobby is taken looks like)
struct Factory {
    refuse: &'static str,
    made: AtomicUsize,
}

impl SessionFactory for Factory {
    fn new_session(&self, opts: &Value) -> Result<Arc<dyn SessionApi>, String> {
        if opts["partner"].as_str() == Some(self.refuse) {
            return Err(REFUSED.into());
        }
        self.made.fetch_add(1, Ordering::SeqCst);
        Ok(Arc::new(Fake {
            deck: opts["deck"].as_str().unwrap_or("").into(),
            partner: opts["partner"].as_str().map(String::from),
        }))
    }
    fn check_two(&self, _: &Value, _: &str) -> Result<(), String> {
        Ok(())
    }
    fn load_session(&self, _: &Value, _: bool) -> Result<Arc<dyn SessionApi>, String> {
        Err("not used".into())
    }
    fn card_names(&self, _: &Info) -> Vec<String> {
        vec![]
    }
}

fn hub(refuse: &'static str) -> (Arc<Hub>, Arc<Factory>) {
    let f = Arc::new(Factory { refuse, made: AtomicUsize::new(0) });
    let dir = std::env::temp_dir();
    (Hub::new(true, false, dir.clone(), Images::new(dir), f.clone()), f)
}

/// the host (browser token "host", yoshimaru) opens a lobby; returns its code
fn open(h: &Hub) -> String {
    h.open_lobby(&json!({"deck": "yoshimaru", "tier": "t2", "images": false}), "host", None).unwrap()
}

fn lobby_code(h: &Hub) -> Option<String> {
    h.state(Some("host"), true)["lobby"]["code"].as_str().map(String::from)
}

fn game_running(h: &Hub) -> bool {
    !h.state(Some("host"), true)["game"].is_null()
}

fn join(h: &Arc<Hub>, token: &str, code: &str, deck: &str) -> Option<String> {
    h.join(token, &json!(code), &json!(deck))
}

#[test]
fn a_refused_join_keeps_the_lobby_and_the_code() {
    let (h, f) = hub("sheoldred");
    let code = open(&h);
    // refused after the lobby was taken (the factory refuses this pairing): the lobby is still there, same code
    assert_eq!(join(&h, "guest1", &code, "sheoldred").as_deref(), Some(REFUSED));
    assert_eq!(lobby_code(&h).as_deref(), Some(code.as_str()), "the host's lobby is gone after a refused join");
    assert!(!game_running(&h));
    // a second guest with another deck joins with the same code and the game starts
    assert_eq!(join(&h, "guest2", &code, "urdragon"), None);
    assert_eq!(lobby_code(&h), None);
    assert!(game_running(&h));
    assert_eq!(f.made.load(Ordering::SeqCst), 1);
    assert_eq!(h.seat_of("guest2", false).as_deref(), Some("urdragon"));
    assert_eq!(h.seat_of("host", true).as_deref(), Some("yoshimaru"));
    h.shutdown();
}

#[test]
fn a_refused_join_can_be_retried_by_the_same_guest() {
    let (h, _) = hub("sheoldred");
    let code = open(&h);
    for _ in 0..3 {
        assert_eq!(join(&h, "guest1", &code, "sheoldred").as_deref(), Some(REFUSED));
        assert_eq!(lobby_code(&h).as_deref(), Some(code.as_str()));
    }
    assert_eq!(join(&h, "guest1", &code, "kyodai"), None);
    assert!(game_running(&h));
    h.shutdown();
}

#[test]
fn a_mirror_is_refused_and_keeps_the_lobby() {
    let (h, _) = hub("nobody");
    let code = open(&h);
    assert_eq!(join(&h, "guest1", &code, "yoshimaru").as_deref(), Some("The host plays that deck: pick another."));
    assert_eq!(lobby_code(&h).as_deref(), Some(code.as_str()));
    assert_eq!(join(&h, "guest2", &code, "sheoldred"), None);
    assert!(game_running(&h));
    h.shutdown();
}

#[test]
fn the_other_refusals_leave_the_lobby_alone() {
    let (h, _) = hub("nobody");
    let code = open(&h);
    let wrong = if code != "000000" { "000000" } else { "111111" };
    assert_eq!(
        join(&h, "g", wrong, "sheoldred").as_deref(),
        Some("That code is not right: the host's screen shows it.")
    );
    assert_eq!(
        join(&h, "host", &code, "sheoldred").as_deref(),
        Some("This browser is the host's: join from the other computer.")
    );
    assert!(join(&h, "g", &code, "nonsense").unwrap().starts_with("deck: one of "));
    assert_eq!(lobby_code(&h).as_deref(), Some(code.as_str()));
    h.shutdown();
    // nothing waits: the same message as before
    assert_eq!(join(&h, "g", &code, "sheoldred").as_deref(), Some("There is no game waiting for a second player."));
}

#[test]
fn a_successful_join_starts_the_game_and_closes_the_lobby() {
    let (h, f) = hub("nobody");
    let code = open(&h);
    assert_eq!(join(&h, "guest", &code, "sheoldred"), None);
    assert_eq!(lobby_code(&h), None);
    assert!(game_running(&h));
    assert_eq!(f.made.load(Ordering::SeqCst), 1);
    // the lobby is used up: a second try is "nothing waiting"
    assert_eq!(join(&h, "late", &code, "urdragon").as_deref(), Some("There is no game waiting for a second player."));
    h.shutdown();
}

/// Two guests at once, one of whom is refused after taking the lobby (or never gets it). Whatever the interleaving, the hub ends with
/// either the game running (the good guest got in) or the lobby open (the good guest arrived while the refused one
/// had it taken and was told nothing waits): never neither, and never both.
#[test]
fn two_guests_at_once_never_half_take_the_lobby() {
    let mut started = 0;
    for _ in 0..200 {
        let (h, _) = hub("sheoldred");
        let code = open(&h);
        let go = Arc::new(Barrier::new(2));
        let spawn = |token: &'static str, deck: &'static str| {
            let (h, code, go) = (h.clone(), code.clone(), go.clone());
            std::thread::spawn(move || {
                go.wait();
                join(&h, token, &code, deck)
            })
        };
        let (bad, good) = (spawn("g1", "sheoldred"), spawn("g2", "urdragon"));
        let (bad, good) = (bad.join().unwrap(), good.join().unwrap());
        // the refused guest either took the lobby first (the engine's refusal) or came second ("nothing waits")
        let nothing = "There is no game waiting for a second player.";
        assert!(matches!(bad.as_deref(), Some(m) if m == REFUSED || m == nothing), "{bad:?}");
        let (lobby, game) = (lobby_code(&h).is_some(), game_running(&h));
        match good {
            None => {
                assert!(game && !lobby, "the good guest got in: the game runs and the lobby is closed");
                started += 1;
            }
            Some(m) => {
                assert_eq!(m, nothing);
                assert_eq!(bad.as_deref(), Some(REFUSED), "the good guest can only miss if the bad one held the lobby");
                assert!(lobby && !game, "the good guest arrived mid-refusal: the lobby is back, no game");
                // and it still works afterwards
                assert_eq!(join(&h, "g2", &code, "urdragon"), None);
                assert!(game_running(&h));
            }
        }
        h.shutdown();
    }
    eprintln!("good guest got in first in {started} of 200 rounds");
}
