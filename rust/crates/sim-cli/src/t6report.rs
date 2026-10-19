//! The Tier 6 report that follows the balance table of `--calibrate within --pool t6`: how each deck wins, how often
//! it goes for a combo and what stops it, mulligans, the interaction it casts, and the pod's seat shares. Every
//! number comes from counters kept while the games were played (`sim_core::t6stats`, plus the winner, round and
//! elimination data of `GameOut`), so the report costs no games and no log parsing.

use crate::play::GameOut;
use crate::stats::Counter;
use std::collections::HashMap;

/// Targets from decklists/pool/TARGETS-t6.md: seat share of decisive games, low / real value / high
pub const SEAT_TARGETS: [(f64, f64, f64); 4] =
    [(30.0, 34.1, 38.0), (24.0, 27.3, 31.0), (19.0, 22.5, 26.0), (14.0, 17.4, 21.0)];

/// one deck's tallies over the games it was seated in
#[derive(Default, Clone)]
pub struct DeckT6 {
    pub seated: u64,
    /// wins with a real winner (not the round-cap tiebreak)
    pub wins: u64,
    pub timeout_wins: u64,
    /// how the wins came (a combo line's name, "damage", "commander damage", "poison", "other")
    pub route: Counter<String>,
    pub win_rounds: Vec<u32>,
    /// times it went for each combo line
    pub go: Counter<String>,
    /// "<line>|<card>": times a line was stopped, and by what
    pub stops: Counter<String>,
    /// hands with 0, 1, 2, 3 mulligans
    pub mulls: [u64; 4],
    /// "<kind>|<card>|<victim key>" for kind counter / removal, "<kind>|<card>" for stax / wipe
    pub ix: Counter<String>,
    /// speed (rounds, one entry per game that reached it): a line first ready / one tutor away / gone for
    pub ready_r: Vec<u32>,
    pub reach_r: Vec<u32>,
    pub go_r: Vec<u32>,
    /// games where a line was ready and the deck went for it later: rounds waited (go - ready)
    pub waited: Vec<u32>,
    /// games where a line was ready and the deck never went for a line
    pub ready_no_go: u64,
    /// the option chosen at decisions where a line was ready and the deck had not gone yet
    pub instead: Counter<String>,
}

/// one seat's tallies (seat 0 goes first), over all games
#[derive(Default, Clone)]
pub struct SeatT6 {
    pub games: u64,
    pub wins: u64,
    pub win_rounds: Vec<u32>,
    pub mull_hands: u64,
    pub go: i64,
    pub stops: i64,
    /// removal and counters other seats resolved on this seat (decks counted by key; only games with distinct keys)
    pub removal_in: i64,
    pub counter_in: i64,
    pub dead: u64,
    pub death_rounds: Vec<u32>,
    /// this seat was the first one out
    pub first_out: u64,
    pub games_distinct: u64,
    /// (sum, games) of mana and hand size on the seat's turn 4, and of the turn its commander first resolved
    pub mana4: (i64, i64),
    pub hand4: i64,
    pub cmd_turn: (i64, i64),
}

/// a whole calibration run of one tier
#[derive(Default, Clone)]
pub struct T6Tally {
    pub games: u64,
    pub timeouts: u64,
    /// wins by seat (turn order, 0-3) in decisive games
    pub seat_wins: [u64; 4],
    pub decks: HashMap<String, DeckT6>,
    pub seats: [SeatT6; 4],
}

/// how the winner won: a combo line's name, "commander damage", "poison", "damage", "other"; None for the round cap
pub fn win_route(g: &GameOut, w: usize) -> Option<String> {
    match g.wintype {
        Some("timeout") => None,
        Some("combo") => {
            let mut lines: Vec<&str> = g.players[w].stats.keys().filter_map(|k| k.strip_prefix("win|")).collect();
            lines.sort();
            Some(lines.first().map_or("combo (unnamed)".to_string(), |l| l.to_string()))
        }
        Some(x) if !x.is_empty() && x != "damage" => Some("other".to_string()),
        _ => {
            // the last opponent out decides: how that player died
            let last = g
                .players
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != w)
                .max_by_key(|(i, p)| (p.stat("elim_round"), *i))
                .map(|(_, p)| p.death_kind);
            Some(
                match last {
                    Some("commander damage") => "commander damage",
                    Some(k) if k.contains("poison") || k.contains("infect") => "poison",
                    Some("combat" | "burn" | "drain" | "triggers" | "life" | "ping" | "aether") => "damage",
                    Some(k) if k.starts_with("dmg") => "damage",
                    _ => "other",
                }
                .to_string(),
            )
        }
    }
}

impl T6Tally {
    pub fn add(&mut self, g: &GameOut) {
        self.games += 1;
        let timeout = g.wintype == Some("timeout");
        self.timeouts += timeout as u64;
        if let Some(w) = g.winner
            && !timeout
            && w < 4
        {
            self.seat_wins[w] += 1;
        }
        self.add_seats(g, timeout);
        for (i, p) in g.players.iter().enumerate() {
            let d = self.decks.entry(p.key.to_string()).or_default();
            d.seated += 1;
            if g.winner == Some(i) {
                match win_route(g, i) {
                    Some(r) => {
                        d.wins += 1;
                        d.route.add(r, 1);
                        d.win_rounds.push(g.round);
                    }
                    None => d.timeout_wins += 1,
                }
            }
            let m = p.stat("mull_n").clamp(0, 3) as usize;
            d.mulls[m] += 1;
            let sp = |k: &str| p.stats.get(k).map(|&r| r as u32);
            let (rd, rc, go) = (sp("sp|ready"), sp("sp|reach"), sp("sp|go"));
            d.ready_r.extend(rd);
            d.reach_r.extend(rc);
            d.go_r.extend(go);
            match (rd, go) {
                (Some(r), Some(g)) => d.waited.push(g.saturating_sub(r)),
                (Some(_), None) => d.ready_no_go += 1,
                _ => {}
            }
            for (k, &v) in &p.stats {
                if let Some(l) = k.strip_prefix("spx|") {
                    d.instead.add(l.to_string(), v);
                }
            }
            for (k, &v) in &p.stats {
                if let Some(c) = k.strip_prefix("go|") {
                    d.go.add(c.to_string(), v);
                } else if let Some(c) = k.strip_prefix("stop|") {
                    d.stops.add(c.to_string(), v);
                } else if let Some(c) = k.strip_prefix("ix|") {
                    d.ix.add(c.to_string(), v);
                }
            }
        }
    }

    /// per-seat tallies (stats only)
    fn add_seats(&mut self, g: &GameOut, timeout: bool) {
        let n = g.players.len().min(4);
        let mut keys: Vec<&str> = g.players.iter().map(|p| p.key).collect();
        keys.sort();
        keys.dedup();
        let distinct = keys.len() == g.players.len();
        let first_out = g.players[..n].iter().filter_map(|p| p.stats.get("elim_round").copied()).min();
        for i in 0..n {
            let p = &g.players[i];
            let s = &mut self.seats[i];
            s.games += 1;
            if g.winner == Some(i) && !timeout {
                s.wins += 1;
                s.win_rounds.push(g.round);
            }
            if p.stat("had_T4") > 0 {
                s.mana4.0 += p.stat("mana_T4");
                s.mana4.1 += 1;
                s.hand4 += p.stat("hand_T4");
            }
            if let Some(&t) = p.milestone.get("cmd_cast") {
                s.cmd_turn.0 += t as i64;
                s.cmd_turn.1 += 1;
            }
            s.mull_hands += (p.stat("mull_n") > 0) as u64;
            for (k, &v) in &p.stats {
                if k.starts_with("go|") {
                    s.go += v;
                } else if k.starts_with("stop|") {
                    s.stops += v;
                }
            }
            if let Some(&er) = p.stats.get("elim_round") {
                s.dead += 1;
                s.death_rounds.push(er as u32);
                if Some(er) == first_out {
                    s.first_out += 1;
                }
            }
            if distinct {
                s.games_distinct += 1;
                for (j, q) in g.players[..n].iter().enumerate() {
                    if j == i {
                        continue;
                    }
                    for (k, &v) in &q.stats {
                        if let Some(r) = k.strip_prefix("ix|removal|")
                            && r.rsplit('|').next() == Some(p.key)
                        {
                            s.removal_in += v;
                        } else if let Some(r) = k.strip_prefix("ix|counter|")
                            && r.rsplit('|').next() == Some(p.key)
                        {
                            s.counter_in += v;
                        }
                    }
                }
            }
        }
    }

    /// decisive games: those with a winner that was not the round-cap tiebreak
    pub fn decisive(&self) -> u64 {
        self.seat_wins.iter().sum()
    }

    pub fn seat_share(&self, seat: usize) -> f64 {
        100.0 * self.seat_wins[seat] as f64 / self.decisive().max(1) as f64
    }
}

fn pct(a: u64, b: u64) -> f64 {
    100.0 * a as f64 / b.max(1) as f64
}

fn median(v: &[u32]) -> f64 {
    let mut v = v.to_vec();
    v.sort();
    match v.len() {
        0 => 0.0,
        n if n % 2 == 1 => v[n / 2] as f64,
        n => (v[n / 2 - 1] + v[n / 2]) as f64 / 2.0,
    }
}

/// "a 12, b 5, c 3" for the top entries of a counter
fn top(c: &Counter<String>, n: usize, label: &dyn Fn(&str) -> String) -> String {
    c.most_common().into_iter().take(n).map(|(k, v)| format!("{} {v}", label(&k))).collect::<Vec<_>>().join(", ")
}

/// the speed lines of one deck: median round a line was ready / one tutor away / gone for / won, and what it did
/// instead while ready (see `sim_core::t6stats` speed keys)
fn render_speed(d: &DeckT6) -> String {
    let n = d.seated.max(1);
    let one = |v: &[u32]| {
        if v.is_empty() { "-".to_string() } else { format!("{:.1} ({:.0}% of games)", median(v), pct(v.len() as u64, n)) }
    };
    let mut o = format!(
        "      speed (median round): ready {}, one tutor away {}, went for a line {}, won {}\n",
        one(&d.ready_r),
        one(&d.reach_r),
        one(&d.go_r),
        if d.win_rounds.is_empty() { "-".to_string() } else { format!("{:.1}", median(&d.win_rounds)) }
    );
    if !d.ready_r.is_empty() {
        let w = &d.waited;
        let late = w.iter().filter(|&&x| x > 0).count() as u64;
        o += &format!(
            "      ready before go: went for it later in {} of {} ready games (median wait {:.1} rounds, mean {:.2}); never went in {}\n",
            late,
            d.ready_r.len(),
            median(w),
            w.iter().sum::<u32>() as f64 / w.len().max(1) as f64,
            d.ready_no_go
        );
        let lbl = |s: &str| s.to_string();
        o += &format!("      while ready and not gone it chose: {}\n", top(&d.instead, 6, &lbl));
    }
    o
}

/// the per-seat block: win round, mulligans, tries, stops, removal taken, deaths
fn render_seats(t: &T6Tally) -> String {
    let mut o = String::from("  by seat (1 goes first): wins, win round avg/median, mulligan hands, combo tries per game, stopped, removal+counters taken per game, died, death round avg, first out\n");
    for (i, s) in t.seats.iter().enumerate() {
        if s.games == 0 {
            continue;
        }
        let avg = |v: &[u32]| if v.is_empty() { 0.0 } else { v.iter().sum::<u32>() as f64 / v.len() as f64 };
        o += &format!(
            "      seat {}: wins {} ({:.1}% of games), win round {:.1}/{:.1}, mulligans {:.1}%, tries {:.2}/game, stopped {:.0}%, taken {:.2}+{:.2}/game, died {:.0}%, death round {:.1}, first out {:.0}%, wins by round <=4/5/6/8: {}/{}/{}/{} of {}, T4 mana {:.1} hand {:.1}, commander out turn {:.1}\n",
            i + 1,
            s.wins,
            pct(s.wins, s.games),
            avg(&s.win_rounds),
            median(&s.win_rounds),
            pct(s.mull_hands, s.games),
            s.go as f64 / s.games as f64,
            pct(s.stops as u64, s.go.max(0) as u64),
            s.removal_in as f64 / s.games_distinct.max(1) as f64,
            s.counter_in as f64 / s.games_distinct.max(1) as f64,
            pct(s.dead, s.games),
            avg(&s.death_rounds),
            pct(s.first_out, s.games),
            s.win_rounds.iter().filter(|&&r| r <= 4).count(),
            s.win_rounds.iter().filter(|&&r| r <= 5).count(),
            s.win_rounds.iter().filter(|&&r| r <= 6).count(),
            s.win_rounds.iter().filter(|&&r| r <= 8).count(),
            s.wins,
            s.mana4.0 as f64 / s.mana4.1.max(1) as f64,
            s.hand4 as f64 / s.mana4.1.max(1) as f64,
            s.cmd_turn.0 as f64 / s.cmd_turn.1.max(1) as f64,
        );
    }
    o
}

/// the report text for one tier's run: `keys` are the tier's decks in file order, `name` turns a key into the printed name
pub fn render(t: &T6Tally, keys: &[String], name: &dyn Fn(&str) -> String) -> String {
    let mut o = String::new();
    let dec = t.decisive();
    o += &format!(
        "  -- Tier 6 report ({} games, {} decisive; round-cap draws {:.1}%) --\n",
        t.games,
        dec,
        pct(t.timeouts, t.games)
    );
    let seats: Vec<String> = (0..4)
        .map(|i| {
            let (lo, real, hi) = SEAT_TARGETS[i];
            let s = t.seat_share(i);
            let flag = if s < lo || s > hi { " OUT" } else { "" };
            format!("seat {} {s:.1}% (real {real}%, sim {lo:.0}-{hi:.0}){flag}", i + 1)
        })
        .collect();
    o += &format!("  win share by seat of decisive games: {}\n", seats.join("; "));
    o += &render_seats(t);
    for k in keys {
        let Some(d) = t.decks.get(k) else { continue };
        let tot = d.wins + d.timeout_wins;
        o += &format!(
            "  {}: seated {}, won {} ({:.1}%; {} by the round cap)\n",
            name(k),
            d.seated,
            tot,
            pct(tot, d.seated),
            d.timeout_wins
        );
        let routes: Vec<String> = d
            .route
            .most_common()
            .into_iter()
            .map(|(r, n)| format!("{r} {n} ({:.0}%)", pct(n as u64, d.wins)))
            .collect();
        o += &format!("      wins by: {}\n", if routes.is_empty() { "none".into() } else { routes.join(", ") });
        if !d.win_rounds.is_empty() {
            let avg = d.win_rounds.iter().sum::<u32>() as f64 / d.win_rounds.len() as f64;
            o += &format!("      win round: average {avg:.1}, median {:.1}\n", median(&d.win_rounds));
        }
        let (goes, stops) = (d.go.total() as u64, d.stops.total() as u64);
        o += &format!(
            "      went for a combo {} times ({:.2} per game){}; stopped {} ({:.0}%)\n",
            goes,
            goes as f64 / d.seated.max(1) as f64,
            if goes > 0 { format!(" [{}]", top(&d.go, 4, &|s| s.to_string())) } else { String::new() },
            stops,
            pct(stops, goes)
        );
        if stops > 0 {
            let mut by_card: Counter<String> = Counter::new();
            for (k, v) in d.stops.most_common() {
                by_card.add(k.rsplit('|').next().unwrap_or("?").to_string(), v);
            }
            o += &format!(
                "      stopped by: {}\n",
                top(&d.stops, 6, &|s| s.replace('|', " by ").replace(" by ?", " by an unnamed answer"))
            );
            o += &format!(
                "      stopped by card: {}\n",
                top(&by_card, 6, &|s| if s == "?" {
                    "(no card named: a cost, a ward or a lock)".to_string()
                } else {
                    s.to_string()
                })
            );
        }
        o += &render_speed(d);
        let m = d.mulls;
        o += &format!(
            "      mulligans: {} of {} hands ({:.1}%): one {}, two {}, three {}\n",
            m[1] + m[2] + m[3],
            d.seated,
            pct(m[1] + m[2] + m[3], d.seated),
            m[1],
            m[2],
            m[3]
        );
        for kind in ["counter", "removal", "stax", "wipe"] {
            let (mut n, mut cards, mut victims): (i64, Counter<String>, Counter<String>) =
                (0, Counter::new(), Counter::new());
            for (k, v) in d.ix.most_common() {
                let mut f = k.split('|');
                if f.next() != Some(kind) {
                    continue;
                }
                n += v;
                cards.add(f.next().unwrap_or("?").to_string(), v);
                if let Some(vk) = f.next() {
                    victims.add(vk.to_string(), v);
                }
            }
            if n == 0 {
                continue;
            }
            let hit =
                if victims.is_empty() { String::new() } else { format!("; hit {}", top(&victims, 5, &|s| name(s))) };
            o += &format!(
                "      {}: {} ({}){}\n",
                match kind {
                    "counter" => "counters resolved",
                    "removal" => "removal resolved",
                    "stax" => "stax cast",
                    _ => "wipes",
                },
                n,
                top(&cards, 5, &|s| s.to_string()),
                hit
            );
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::PlayerOut;

    fn player(key: &'static str) -> PlayerOut {
        PlayerOut {
            key,
            turns: 8,
            alive: true,
            killer: None,
            stats: HashMap::new(),
            milestone: HashMap::new(),
            first_bomb: None,
            death_kind: "",
            seen: vec![],
            cast: vec![],
            lost: vec![],
            cast_turns: vec![],
        }
    }

    fn game(winner: Option<usize>, wintype: Option<&'static str>, round: u32) -> GameOut {
        GameOut { winner, wintype, round, players: vec![player("a"), player("b"), player("c"), player("d")], log: None }
    }

    #[test]
    fn seat_shares_count_decisive_games_only() {
        let mut t = T6Tally::default();
        for w in [0, 0, 0, 1, 1, 2, 3] {
            t.add(&game(Some(w), Some("combo"), 8));
        }
        t.add(&game(Some(3), Some("timeout"), 20)); // the round-cap tiebreak: not decisive
        t.add(&game(None, None, 20));
        assert_eq!((t.games, t.timeouts, t.decisive()), (9, 1, 7));
        assert_eq!(t.seat_wins, [3, 2, 1, 1]);
        assert!((t.seat_share(0) - 300.0 / 7.0).abs() < 1e-9);
        assert!((t.seat_share(3) - 100.0 / 7.0).abs() < 1e-9);
        // a deck's round-cap win is kept apart from its real wins
        assert_eq!((t.decks["d"].wins, t.decks["d"].timeout_wins), (1, 1));
    }

    #[test]
    fn the_seat_block_counts_wins_rounds_and_removal_taken_per_seat() {
        let mut g = game(Some(1), Some("combo"), 6);
        g.players[0].stats.insert("mull_n", 1);
        g.players[0].stats.insert("elim_round", 4);
        g.players[2].stats.insert("ix|removal|Swords to Plowshares|a", 3);
        g.players[1].stats.insert("go|X", 2);
        g.players[1].stats.insert("stop|X|Force of Will", 1);
        let mut t = T6Tally::default();
        t.add(&g);
        assert_eq!((t.seats[1].wins, t.seats[1].win_rounds.clone(), t.seats[0].wins), (1, vec![6], 0));
        assert_eq!((t.seats[0].mull_hands, t.seats[0].dead, t.seats[0].first_out), (1, 1, 1));
        assert_eq!((t.seats[0].removal_in, t.seats[1].go, t.seats[1].stops), (3, 2, 1));
        let text = render(&t, &[], &|k| k.to_string());
        assert!(text.contains("seat 2: wins 1 (100.0% of games), win round 6.0/6.0"), "{text}");
        assert!(text.contains("seat 1: wins 0 (0.0% of games)"), "{text}");
    }

    #[test]
    fn a_combo_win_is_named_by_its_line_and_counters_are_summed() {
        let mut g = game(Some(1), Some("combo"), 6);
        g.players[1].stats.insert("win|Breach + Oracle", 1);
        g.players[1].stats.insert("go|Breach + Oracle", 2);
        g.players[1].stats.insert("stop|Breach + Oracle|Force of Will", 1);
        g.players[1].stats.insert("ix|counter|Force of Will|a", 2);
        g.players[1].stats.insert("mull_n", 2);
        g.players[0].stats.insert("ix|removal|Swords to Plowshares|b", 1);
        let mut t = T6Tally::default();
        t.add(&g);
        t.add(&g);
        let b = &t.decks["b"];
        assert_eq!(b.route.get(&"Breach + Oracle".to_string()), 2);
        assert_eq!((b.go.total(), b.stops.total(), b.mulls), (4, 2, [0, 0, 2, 0]));
        assert_eq!(b.win_rounds, vec![6, 6]);
        assert_eq!(b.ix.get(&"counter|Force of Will|a".to_string()), 4);
        let keys: Vec<String> = ["a", "b"].iter().map(|s| s.to_string()).collect();
        let text = render(&t, &keys, &|k| k.to_uppercase());
        assert!(text.contains("B: seated 2, won 2 (100.0%; 0 by the round cap)"), "{text}");
        assert!(text.contains("wins by: Breach + Oracle 2 (100%)"), "{text}");
        assert!(text.contains("win round: average 6.0, median 6.0"), "{text}");
        assert!(text.contains("went for a combo 4 times (2.00 per game)"), "{text}");
        assert!(text.contains("stopped 2 (50%)"), "{text}");
        assert!(text.contains("Breach + Oracle by Force of Will 2"), "{text}");
        assert!(text.contains("counters resolved: 4 (Force of Will 4); hit A 4"), "{text}");
        assert!(text.contains("A: seated 2, won 0"), "{text}");
        assert!(text.contains("removal resolved: 2 (Swords to Plowshares 2); hit B 2"), "{text}");
    }

    #[test]
    fn the_speed_lines_give_median_ready_reach_go_and_win_rounds() {
        let mut t = T6Tally::default();
        for (rd, go) in [(3, Some(5)), (4, Some(4)), (6, None)] {
            let mut g = game(Some(1), Some("combo"), 8);
            g.players[1].stats.insert("sp|ready", rd);
            g.players[1].stats.insert("sp|reach", rd - 1);
            if let Some(go) = go {
                g.players[1].stats.insert("sp|go", go);
            }
            g.players[1].stats.insert("spx|Cast Sol Ring", 2);
            t.add(&g);
        }
        let d = &t.decks["b"];
        assert_eq!((d.ready_r.clone(), d.waited.clone(), d.ready_no_go), (vec![3, 4, 6], vec![2, 0], 1));
        let text = render(&t, &["b".to_string()], &|k| k.to_string());
        assert!(text.contains("ready 4.0 (100% of games), one tutor away 3.0"), "{text}");
        assert!(text.contains("went for a line 4.5"), "{text}");
        assert!(text.contains("went for it later in 1 of 3 ready games"), "{text}");
        assert!(text.contains("Cast Sol Ring 6"), "{text}");
    }

    #[test]
    fn damage_wins_are_told_apart_by_how_the_last_opponent_died() {
        for (kind, want) in
            [("commander damage", "commander damage"), ("burn", "damage"), ("poison", "poison"), ("decked", "other")]
        {
            let mut g = game(Some(0), None, 9);
            g.players[1].death_kind = kind;
            g.players[1].stats.insert("elim_round", 9);
            g.players[2].death_kind = "combat";
            g.players[2].stats.insert("elim_round", 7);
            assert_eq!(win_route(&g, 0).as_deref(), Some(want), "{kind}");
        }
        assert_eq!(win_route(&game(Some(0), Some("timeout"), 20), 0), None);
        assert_eq!(win_route(&game(Some(0), Some("alt"), 5), 0).as_deref(), Some("other"));
    }
}
