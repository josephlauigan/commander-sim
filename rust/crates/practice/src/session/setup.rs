//! Seating a practice game and building it (Python's `Session._choose_seats` / `_exact_seats` and
//! `ais.setup_pool_game(human=...)`).

use sim_core::engine::turn::mulligan;
use sim_core::ids::PlayerId;
use sim_core::rng::Rng;
use sim_core::run::Engine;
use sim_core::settings::Settings;
use sim_core::state::Game;
use std::sync::Arc;

/// the decks people can play: every deck in data/my_decks.toml, in its order (a test checks it against decks.json)
pub const MY_DECKS: [&str; 13] = ["seph", "veyran", "sauron", "yshtola", "galadriel", "alela", "kefka", "sheoldred", "cloud", "thrasios", "karn", "thanos", "urdragon"];
pub const TIERS: [&str; 7] = ["t0", "t1", "t2", "t3", "t4", "t5", "t6"];

/// the pool decks of a tier, sorted by key
pub fn pool_keys(e: &Engine, tier: &str) -> Vec<String> {
    let mut v: Vec<String> = e.decks.values().filter(|d| d.tier.as_deref() == Some(tier)).map(|d| d.key.clone()).collect();
    v.sort();
    v
}

/// k keys drawn without replacement (Python's `random.sample`)
fn sample(r: &mut Rng, keys: &[String], k: usize) -> Vec<String> {
    let mut v = keys.to_vec();
    r.shuffle(&mut v);
    v.truncate(k);
    v
}

pub struct SeatArgs<'a> {
    pub deck: &'a str,
    pub tier: &'a str,
    pub partner: Option<&'a str>,
    pub seed: u64,
    pub seat: Option<usize>,
    pub opponents: Option<&'a [String]>,
    pub seats: Option<&'a [String]>,
}

/// a seating given in full (playing the same seed again)
fn exact_seats(e: &Engine, a: &SeatArgs, seats: &[String]) -> Result<Vec<String>, String> {
    let keys = pool_keys(e, a.tier);
    let humans: Vec<&str> = std::iter::once(a.deck).chain(a.partner).collect();
    let n = 4 - humans.len();
    let bad = seats.len() != 4
        || humans.iter().any(|h| seats.iter().filter(|s| s == h).count() != 1)
        || (0..4).any(|i| (0..i).any(|j| seats[i] == seats[j]))
        || seats.iter().any(|k| !humans.contains(&k.as_str()) && !keys.contains(k));
    if bad {
        return Err(format!(
            "seats: {} and {n} of {}, in turn order",
            if a.partner.is_some() { "your decks" } else { "your deck" },
            keys.join(", ")
        ));
    }
    Ok(seats.to_vec())
}

/// the seat order for a new game
pub fn choose_seats(e: &Engine, a: &SeatArgs) -> Result<Vec<String>, String> {
    if let Some(s) = a.seats {
        return exact_seats(e, a, s);
    }
    let keys = pool_keys(e, a.tier);
    if let Some(partner) = a.partner {
        // two people: two opponents, everyone seated at random
        if let Some(o) = a.opponents
            && (o.len() != 2 || o.iter().any(|k| !keys.contains(k)))
        {
            return Err(format!("pick two of: {}", keys.join(", ")));
        }
        if a.seat.is_some() {
            return Err("with two players the seats are drawn at random".into());
        }
        let mut seats = match a.opponents {
            Some(o) => o.to_vec(),
            None => sample(&mut Rng::named(&format!("pool:{}", a.seed)), &keys, 2),
        };
        seats.push(a.deck.to_string());
        seats.push(partner.to_string());
        Rng::named(&format!("seats:{}", a.seed)).shuffle(&mut seats);
        return Ok(seats);
    }
    let mut seats = if let Some(o) = a.opponents {
        if o.len() != 3 || o.iter().any(|k| !keys.contains(k)) {
            return Err(format!("pick three of: {}", keys.join(", ")));
        }
        let mut s = o.to_vec();
        s.push(a.deck.to_string());
        Rng::named(&format!("seats:{}", a.seed)).shuffle(&mut s);
        s
    } else {
        // pools.draw_seats: three opponents and the deck, in a random order
        let mut r = Rng::named(&format!("pool:{}", a.seed));
        let mut s = sample(&mut r, &keys, 3);
        s.push(a.deck.to_string());
        r.shuffle(&mut s);
        s
    };
    if let Some(n) = a.seat {
        if !(1..=4).contains(&n) {
            return Err("seat is 1 to 4".into());
        }
        let i = seats.iter().position(|s| s == a.deck).unwrap();
        let d = seats.remove(i);
        seats.insert(n - 1, d);
    }
    Ok(seats)
}

/// A game seated in this order, shuffled and mulliganed (but for the people's seats, whose mulligans they make:
/// the shuffle generator of each is returned for that), before turn one.
pub fn build_game(
    e: &Engine,
    settings: Settings,
    seed: u64,
    seats: &[String],
    humans: &[String],
) -> Result<(Game, Vec<(PlayerId, Rng)>), String> {
    let specs = seats
        .iter()
        .map(|k| {
            let d = e.decks.get(k).ok_or_else(|| format!("unknown deck {k:?}"))?;
            e.seat(k, &d.cards, &d.commander)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let players = specs
        .iter()
        .enumerate()
        .map(|(i, s)| s.player(PlayerId(i as u8)))
        .collect();
    let mut g = Game::new(e.db.clone(), e.registry.clone(), Arc::new(settings), players, Rng::named(&format!("play:{seed}")));
    let mut mull = vec![];
    for i in 0..g.players.len() {
        let p = PlayerId(i as u8);
        let mut r = Rng::named(&format!("lib:{seed}:{}", g.player(p).key));
        let mut lib = std::mem::take(&mut g.player_mut(p).library);
        r.shuffle(&mut lib);
        g.player_mut(p).library = lib;
        if humans.iter().any(|h| h == &*g.player(p).key) {
            mull.push((p, r));
            continue;
        }
        #[allow(clippy::let_unit_value)]
        let _ = mulligan(&mut g, p, &mut r);
        let hand = g.player(p).hand.clone();
        g.player_mut(p).seen.extend(hand);
    }
    Ok((g, mull))
}
