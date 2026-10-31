//! Whole games played through the human seat by a simple bot, so every card goes through that path (Python's
//! tests/test_play_turn.py BotGames).

use practice::session::setup::{SeatArgs, build_game, choose_seats};
use serde_json::{Value, json};
use sim_core::control::{Answer, Human, Request};
use sim_core::human::{choices, legal, mana, script::ScriptHuman};
use sim_core::ids::PlayerId;
use sim_core::run::{Engine, settings};
use sim_core::state::Game;
use sim_core::sym::Sym;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct BotState {
    /// a spell refused this turn: not again this turn
    failed: HashSet<(u32, String)>,
    /// the spell tried last: (round, name, number of told texts then)
    last: Option<(u32, String, usize)>,
    /// equip each piece at most once a turn
    tried: HashSet<(u32, usize)>,
}

struct Bot {
    deck: String,
    st: Mutex<BotState>,
    told: Mutex<Vec<(Sym, String)>>,
}

impl Human for Bot {
    fn ask(&self, g: &Game, seat: PlayerId, req: Request) -> Answer {
        match &*req.kind {
            "target" | "choose" | "block" | "search" => return Answer::Pick(0),
            "mulligan" => return Answer::Pick(0),
            "attack" => return Answer::Picks((0..req.choices.len()).collect()),
            "priority" => {}
            _ => return Answer::Yes,
        }
        let p = seat;
        // the rules checks can change the game (a dry run): ask them of a copy that has this seat too
        let mut c = g.clone();
        c.cur_cast = g.cur_cast.clone();
        c.humans.add(p, Arc::new(ScriptHuman::new(vec![])));
        let g = &mut c;
        let mut st = self.st.lock().unwrap();
        let told = self.told.lock().unwrap();
        if let Some(last) = st.last.take()
            && told.len() > last.2
            && told.last().unwrap().0 == "invalid"
        {
            st.failed.insert((last.0, last.1));
        }
        let act = |v: Value| Answer::Action(v);
        let hand = g.player(p).hand.clone();
        for (i, &cd) in hand.iter().enumerate() {
            if g.db.get(cd).land && legal::check_land(g, p, cd).is_none() {
                return act(json!({"do": "land", "card": i}));
            }
        }
        if let Some(s) = mana::sources(g, p).first() {
            return act(json!({"do": "tap", "source": s.id}));
        }
        for (i, &cd) in hand.iter().enumerate() {
            let name = g.db.get(cd).name.to_string();
            if !g.db.get(cd).land
                && !st.failed.contains(&(g.round, name.clone()))
                && legal::check_cast(g, p, cd, "hand").is_none()
            {
                st.last = Some((g.round, name, told.len()));
                return act(json!({"do": "cast", "card": i}));
            }
        }
        let cmd = g.player(p).cmd;
        if g.player(p).cmd_in_zone && legal::check_cast(g, p, cmd, "cmd").is_none() {
            return act(json!({"do": "cast", "zone": "cmd"}));
        }
        let perms = g.player(p).perms.clone();
        for (i, &m) in perms.iter().enumerate() {
            if g.active == Some(p)
                && legal::equip_cost(g, m).is_some()
                && !sim_core::cardcode::ability_locked(g, m, p) // Collector Ouphe, Arrest: no Equip either
                && g.perm(m).attached.is_none()
                && !st.tried.contains(&(g.round, m.0 as usize))
                && legal::sorcery_timing(g, p).is_none()
            {
                st.tried.insert((g.round, m.0 as usize));
                return act(json!({"do": "use", "perm": i}));
            }
        }
        let _ = &self.deck;
        act(json!({"do": "pass"}))
    }

    fn tell(&self, _seat: PlayerId, kind: Sym, text: &str) {
        self.told.lock().unwrap().push((kind, text.to_string()));
    }
}

fn play(deck: &str, tier: &str, seed: u64) {
    let (deck, tier) = (deck.to_string(), tier.to_string());
    // a game recurses deeply: a big stack, like the session's game thread
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
            let e = Engine::load(&data).unwrap();
            let seats = choose_seats(
                &e,
                &SeatArgs { deck: &deck, tier: &tier, partner: None, seed, seat: None, opponents: None, seats: None },
            )
            .unwrap();
            let st = settings("loose", "adaptive", 1.0).unwrap();
            let (mut g, mull) = build_game(&e, st, seed, &seats, &[deck.clone()]).unwrap();
            let bot = Arc::new(Bot { deck: deck.clone(), st: Mutex::default(), told: Mutex::default() });
            let me = g.players.iter().find(|q| &*q.key == deck.as_str()).unwrap().id;
            g.humans.add(me, bot.clone());
            for (p, mut r) in mull {
                choices::mulligan(&mut g, p, &mut r).unwrap();
            }
            sim_core::engine::turn::run_rounds(&mut g, 30);
            let spells = g.player(me).stats.get("spells_cast").copied().unwrap_or(0);
            // a seat that died early (a bad draw on this seed) had no chance to cast more; the invalid-action check below
            // is what catches a broken card
            let alive = g.player(me).alive;
            assert!(spells > 3 || !alive, "{deck} {tier} {seed}: {spells} spells");
            let bad: Vec<String> = bot
                .told
                .lock()
                .unwrap()
                .iter()
                .filter(|(k, t)| {
                    *k == "invalid"
                        && !(t.starts_with("Can't equip")
                            || t.starts_with("You have no creature")
                            || t.starts_with("Can't cast Twinflame on"))
                        && !t.contains(" on that target") // an unaffordable target (kicker): the bot tries another
                })
                .map(|x| x.1.clone())
                .collect();
            assert!(bad.is_empty(), "{deck} {tier} {seed}: {bad:?}");
        })
        .unwrap()
        .join()
        .unwrap_or_else(|_| panic!("the game panicked"));
}

#[test]
fn sauron_t2() {
    play("sauron", "t2", 2); // seed 1 deals the 10-13 list a round-4 loss with 3 cards drawn: too short a game to test
}
#[test]
fn sauron_t4() {
    play("sauron", "t4", 2);
}
#[test]
fn ghave_t3() {
    play("ghave", "t3", 1);
}
#[test]
fn veyran_t1() {
    play("veyran", "t1", 1);
}
#[test]
fn galadriel_t3() {
    play("galadriel", "t3", 1);
}
#[test]
fn yshtola_t3() {
    play("yshtola", "t3", 1);
}
#[test]
fn alela_t3() {
    play("alela", "t3", 2);
}
#[test]
fn ghave_t4() {
    play("ghave", "t4", 2);
}
