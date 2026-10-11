//! compare.py's reports: the strength axes and the per-axis table of a paired run, the verdict, and the deck
//! analysis (--analyze). Same layout and wording as the Python harness.

use crate::play::{GameOut, PlayerOut};
use crate::stats::{Counter, Moments};
use std::collections::{BTreeMap, HashSet};

pub struct Metric {
    pub key: &'static str,
    pub label: &'static str,
    pub axis: &'static str,
    /// is a higher value better?
    pub up: bool,
    /// None: every deck; Some("combo"): Veyran and Sauron; Some(deck): that deck only
    pub only: Option<&'static str>,
}

const fn m(key: &'static str, label: &'static str, axis: &'static str, up: bool, only: Option<&'static str>) -> Metric {
    Metric { key, label, axis, up, only }
}

/// compare.METRICS: (key, label, axis, which direction is better, applies to)
pub const METRICS: [Metric; 27] = [
    m("win", "Win rate", "Outcome", true, None),
    m("survived", "Rounds survived (or game length)", "Outcome", true, None),
    m("mulls", "Extra mulligans per game", "Mana & consistency", false, None),
    m("land_miss", "Missed land drops, turns 1-5", "Mana & consistency", false, None),
    m("mana_T4", "Mana available on your turn 4", "Mana & consistency", true, None),
    m("mana_T6", "Mana available on your turn 6", "Mana & consistency", true, None),
    m("plan_reached", "Games where the game plan came online", "Speed", true, None),
    m("plan_turn", "Turn the game plan came online", "Speed", false, None),
    m("cards_drawn", "Cards drawn per game (incl. draw step)", "Card flow", true, None),
    m("tutored", "Tutors resolved per game", "Card flow", true, None),
    m("spells_cast", "Spells cast per game", "Card flow", true, None),
    m("counters", "Counterspells cast", "Interaction", true, None),
    m("removal", "Spot removal cast", "Interaction", true, None),
    m("wipes", "Board wipes cast", "Interaction", true, None),
    m("protection", "Protection used (HI/sac/Tidebinder/counter-war)", "Resilience", true, None),
    m("threats_lost", "Your key permanents removed by opponents", "Resilience", false, None),
    m("countered", "Your spells countered", "Resilience", false, None),
    m("recovered", "Removed bombs brought back", "Resilience", true, Some("seph")),
    m("bombs_landed", "Bombs put on the battlefield", "Threat & pressure", true, Some("seph")),
    m("dmg_dealt", "Damage dealt to opponents per game", "Threat & pressure", true, None),
    m("dmg_combat", "  ...of which combat damage", "Threat & pressure", true, None),
    m("dmg_burn", "  ...of which magecraft / burn", "Threat & pressure", true, None),
    m("dmg_drain", "  ...of which drain triggers", "Threat & pressure", true, None),
    m("dmg_aether", "  ...of which Aetherflux shots", "Threat & pressure", true, None),
    m("dmg_triggers", "  ...of which other triggers", "Threat & pressure", true, None),
    m("combo_rate", "Games with a combo attempt", "Speed", true, Some("combo")),
    m("aether_shots", "Aetherflux shots per game", "Threat & pressure", true, Some("veyran")),
];

/// compare.PLAN: what "the game plan came online" means for each deck
pub fn plan_text(deck: &str) -> &'static str {
    match deck {
        "seph" => "first 6+ power bomb on the battlefield",
        "veyran" => "Veyran + a magecraft payoff on the battlefield",
        "sauron" => "Sword + Assault combo attempted",
        "galadriel" => "first Alliance trigger from Galadriel",
        "yshtola" => "first drain from Y'shtola's cast trigger",
        "alela" => "first Faerie made by Alela",
        "jodah" => "first legend cast free by Jodah",
        "najeela" => "first WUBRG extra combat",
        _ => "",
    }
}

fn axis_note(axis: &str) -> &'static str {
    match axis {
        "Outcome" => "the bottom line; everything else explains why it moved.",
        "Mana & consistency" => {
            "can the deck cast its spells on time? Missed land drops and low mana stall everything else."
        }
        "Speed" => "how fast the deck does the thing it is built to do.",
        "Card flow" => "how many resources the deck sees; more cards seen means more options and fewer dead draws.",
        "Interaction" => "how often the deck answers what opponents do.",
        "Resilience" => "how well the plan survives opposing interaction.",
        "Threat & pressure" => "how much the deck pushes the game toward a win.",
        _ => "",
    }
}

/// compare.plan_turn: the turn the deck's plan came online (None, or 0, if it never did)
pub fn plan_turn(me: &PlayerOut, deck: &str) -> Option<u32> {
    let ms = |k: &str| me.milestone.get(k).copied();
    let t = match deck {
        "seph" => me.first_bomb,
        "veyran" => ms("engine"),
        "najeela" => ms("act"),
        "galadriel" => ms("alliance"),
        "yshtola" => ms("yshtola"),
        "alela" => ms("faerie"),
        "jodah" => ms("jodah"),
        _ => ms("combo"),
    };
    t.filter(|&t| t != 0)
}

/// compare.axis_values: one game's value on each strength axis, for player `me` playing `deck`
pub fn axis_values(g: &GameOut, me: usize, deck: &str) -> Vec<(&'static str, f64)> {
    let p = &g.players[me];
    let s = |k: &str| p.stat(k) as f64;
    let tide: i64 = p.stats.iter().filter(|(k, _)| k.starts_with("tide_")).map(|(_, v)| *v).sum();
    let mut v = vec![
        ("win", if g.winner == Some(me) { 1.0 } else { 0.0 }),
        ("survived", p.stats.get("elim_round").map_or(g.round as f64, |&x| x as f64)),
        ("mulls", s("mulls")),
        ("land_miss", s("land_miss")),
        ("cards_drawn", s("cards_drawn")),
        ("tutored", s("tutored")),
        ("spells_cast", s("spells_cast")),
        ("counters", s("counters_cast")),
        ("removal", s("removal_cast")),
        ("wipes", s("wipes_cast")),
        ("protection", (p.stat("hi_used") + p.stat("sac_saves") + p.stat("counterwar_won") + tide) as f64),
        ("threats_lost", s("threats_lost")),
        ("countered", s("spells_countered")),
        ("recovered", s("bomb_recovered")),
        ("bombs_landed", s("bombs_landed")),
        ("dmg_dealt", s("dmg_dealt")),
    ];
    if p.stat("had_T4") != 0 {
        v.push(("mana_T4", s("mana_T4")));
    }
    if p.stat("had_T6") != 0 {
        v.push(("mana_T6", s("mana_T6")));
    }
    for (k, src) in [
        ("dmg_combat", "dmgk_combat"),
        ("dmg_burn", "dmgk_burn"),
        ("dmg_drain", "dmgk_drain"),
        ("dmg_aether", "dmgk_aether"),
        ("dmg_triggers", "dmgk_triggers"),
    ] {
        v.push((k, s(src)));
    }
    v.push(("combo_rate", if p.stat("combo_attempt") != 0 { 1.0 } else { 0.0 }));
    v.push(("aether_shots", s("aether_shots")));
    let plan = plan_turn(p, deck);
    v.push(("plan_reached", if plan.is_some() { 1.0 } else { 0.0 }));
    if let Some(t) = plan {
        v.push(("plan_turn", t as f64));
    }
    v
}

/// one run's per-axis sums, keyed by metric
pub type Axes = BTreeMap<&'static str, Moments>;

/// compare.verdict: the one-line conclusion of a paired run
pub fn verdict(deltas: &[f64], ses: &[f64], profiles: &[String]) -> String {
    let clear: Vec<bool> = deltas.iter().zip(ses).map(|(d, s)| d.abs() > 2.0 * s).collect();
    if deltas.len() == 1 {
        if !clear[0] {
            return "NO CLEAR EFFECT (within noise)".into();
        }
        let w = if deltas[0] > 0.0 { "BETTER" } else { "WORSE" };
        return format!("{w} under the {} profile (the only one run)", profiles[0]);
    }
    let same_sign = deltas.iter().all(|d| *d > 0.0) || deltas.iter().all(|d| *d <= 0.0);
    if clear.iter().all(|&c| c) && same_sign {
        return if deltas[0] > 0.0 { "BETTER under both profiles" } else { "WORSE under both profiles" }.into();
    }
    if clear.iter().all(|&c| c) {
        return "TABLE-DEPENDENT (clear under both profiles, but in opposite directions)".into();
    }
    if let Some(i) = clear.iter().position(|&c| c) {
        let w = if deltas[i] > 0.0 { "BETTER" } else { "WORSE" };
        return format!("LEANS {w} (clear only under {}; rerun with a larger --n to confirm)", profiles[i]);
    }
    "NO CLEAR EFFECT (within noise)".into()
}

/// compare.axis_report: per-axis table, baseline -> variant for each profile; * marks a change beyond noise (2 SE).
/// pairs: (profile, baseline axes, variant axes)
pub fn axis_report(deck: &str, pairs: &[(String, Axes, Axes)], lookahead: bool) {
    let profs: Vec<&String> = pairs.iter().map(|p| &p.0).collect();
    println!("\n--- What changed, by strength axis ({deck}) ---");
    println!("  Each cell: current -> new, change, and the noise band (±2 standard errors) for that change.");
    println!("  * = the change is larger than its noise band, so it is unlikely to be luck.");
    println!("  game plan for {deck}: {}", plan_text(deck));
    let heads: Vec<String> = profs.iter().map(|p| format!("{:^39}", format!("[{p}]"))).collect();
    println!("    {:48} {}", "", heads.join("   "));
    // (axis, label, good) -> profiles, in the order found
    let mut moved: Vec<((&str, &str, bool), Vec<String>)> = vec![];
    let mut cur_axis: Option<&str> = None;
    for mt in &METRICS {
        if mt.only == Some("combo") && !matches!(deck, "veyran" | "sauron") {
            continue;
        }
        if let Some(o) = mt.only
            && o != "combo"
            && o != deck
        {
            continue;
        }
        let mean0 = |a: &Axes| a.get(mt.key).map_or(0.0, |x| x.mean());
        if mt.key.starts_with("dmg_")
            && mt.key != "dmg_dealt"
            && pairs.iter().all(|(_, b, v)| mean0(b) == 0.0 && mean0(v) == 0.0)
        {
            continue;
        }
        let mut cells = vec![];
        for (prof, b, v) in pairs {
            let (Some(ab), Some(av)) = (b.get(mt.key), v.get(mt.key)) else {
                cells.push(format!("{:>36}", "n/a"));
                continue;
            };
            let d = av.mean() - ab.mean();
            let se = (ab.var() / ab.n as f64 + av.var() / av.n as f64).sqrt();
            let sig = se > 0.0 && d.abs() > 2.0 * se;
            let pctfmt = matches!(mt.key, "win" | "plan_reached" | "combo_rate");
            let f = |x: f64| if pctfmt { format!("{:5.1}%", 100.0 * x) } else { format!("{x:6.2}") };
            let dd = if pctfmt { format!("{:+.1}pt", 100.0 * d) } else { format!("{d:+.2}") };
            let nz = if pctfmt { format!("±{:.1}", 200.0 * se) } else { format!("±{:.2}", 2.0 * se) };
            cells.push(format!(
                "{} -> {} {dd:>7} {nz:>6}{}",
                f(ab.mean()),
                f(av.mean()),
                if sig { "*" } else { " " }
            ));
            if sig {
                let good = (d > 0.0) == mt.up;
                let key = (mt.axis, mt.label, good);
                match moved.iter_mut().find(|x| x.0 == key) {
                    Some(x) => x.1.push(prof.clone()),
                    None => moved.push((key, vec![prof.clone()])),
                }
            }
        }
        if cur_axis != Some(mt.axis) {
            println!("\n  {}", mt.axis);
            cur_axis = Some(mt.axis);
        }
        println!("    {:48} {}", mt.label, cells.join("   "));
    }
    println!("\n--- Reading it ---");
    if moved.is_empty() {
        let wh = if pairs.len() > 1 { "under either profile".to_string() } else { format!("under the {} profile", pairs[0].0) };
        let more = format!(
            "about 4x the games halves the noise{}",
            if lookahead { " (--ai adaptive makes that cheap)" } else { "" }
        );
        println!("  No axis moved beyond noise {wh}: the swap is roughly a like-for-like");
        println!("  trade in this pod, or the new card rarely matters. To resolve smaller effects, {more}.");
        return;
    }
    let mut by_axis: Vec<(&str, Vec<(&str, bool, &Vec<String>)>)> = vec![];
    for ((axis, label, good), ps) in &moved {
        match by_axis.iter_mut().find(|x| x.0 == *axis) {
            Some(x) => x.1.push((label, *good, ps)),
            None => by_axis.push((axis, vec![(label, *good, ps)])),
        }
    }
    for (axis, items) in &by_axis {
        println!("  {axis}: {}", axis_note(axis));
        for (label, good, ps) in items {
            let wh = if ps.len() == profs.len() { "both profiles".to_string() } else { format!("{} profile only", ps[0]) };
            println!("    {:6} - {label} ({wh})", if *good { "better" } else { "worse" });
        }
    }
    println!("  With ~40 checks per run, one or two stars can appear by chance; trust the ones that show");
    println!("  up under both profiles, or that line up with the card's job.");
    println!("  Note: per-game counts rise with game length, so check 'Rounds survived' before reading");
    println!("  more cards drawn or more spells cast as a pure gain.");
}

// ------------------------------------------------------------------ deck analysis (--analyze)
pub const KINDS: [&str; 9] = ["combat", "burn", "drain", "aether", "triggers", "combo", "commander damage", "decked", "other"];
const DMG_KINDS: [&str; 5] = ["combat", "burn", "drain", "aether", "triggers"];

/// compare.analysis_record
#[derive(Default)]
pub struct Analysis {
    pub n: i64,
    pub win: i64,
    pub wintype: Counter<String>,
    pub kills: Counter<&'static str>,
    /// (who eliminated the deck, how)
    pub death: Counter<(String, String)>,
    pub death_round: i64,
    pub survived: i64,
    pub plan: Counter<u32>,
    pub combo: i64,
    pub aether: i64,
    pub seen: Counter<String>,
    pub cast: Counter<String>,
    pub win_seen: Counter<String>,
    pub win_cast: Counter<String>,
    pub lost: Counter<String>,
    pub dmg: Counter<&'static str>,
}

impl Analysis {
    /// compare.analysis_add: one finished game, from `deck`'s point of view
    pub fn add(&mut self, g: &GameOut, deck: &str) {
        let Some(mi) = g.player(deck) else { return };
        let me = &g.players[mi];
        let won = g.winner == Some(mi);
        self.n += 1;
        self.win += won as i64;
        if won {
            self.wintype.add(g.wintype_or_damage().to_string(), 1);
        }
        for k in KINDS {
            self.kills.add(k, me.stat(&format!("kills_{k}")));
        }
        for k in DMG_KINDS {
            self.dmg.add(k, me.stat(&format!("dmgk_{k}")));
        }
        let elim = me.stats.get("elim_round").copied().unwrap_or(g.round as i64);
        if !me.alive {
            let who = me.killer.map_or("self".to_string(), |k| g.players[k].key.to_string());
            let how = if me.death_kind.is_empty() { "other" } else { me.death_kind };
            self.death.add((who, how.to_string()), 1);
            self.death_round += elim;
        }
        self.survived += elim;
        if let Some(t) = plan_turn(me, deck) {
            self.plan.add(t, 1);
        }
        self.combo += (me.stat("combo_attempt") != 0) as i64;
        self.aether += (me.stat("aether_shots") != 0) as i64;
        let mut seen: HashSet<&str> = me.seen.iter().map(|s| s.as_str()).collect();
        seen.extend(me.cast.iter().map(|s| s.as_str()));
        for n in seen {
            self.seen.add(n.to_string(), 1);
            self.win_seen.add(n.to_string(), won as i64);
        }
        let cast: HashSet<&str> = me.cast.iter().map(|s| s.as_str()).collect();
        for n in cast {
            self.cast.add(n.to_string(), 1);
            self.win_cast.add(n.to_string(), won as i64);
        }
        for (n, k) in &me.lost {
            self.lost.add(n.clone(), *k as i64);
        }
    }
}

/// what the card report needs to know about a card
pub struct CardInfo {
    pub name: String,
    pub land: bool,
    /// a reactive card: counterspell, protection held for a response, Tidebinder-style
    pub held: bool,
}

/// compare.print_analysis. name_of maps a pool deck's key to its display name.
pub fn print_analysis(
    deck: &str,
    cards: &[CardInfo],
    r: &Analysis,
    profile: &str,
    ai: &str,
    name_of: &dyn Fn(&str) -> String,
) {
    let nn = r.n;
    let n = nn as f64;
    let base = r.win as f64 / n;
    println!("\n=========== {deck} analysis [{profile} profile, {ai} AI, {nn} games] ===========");
    println!(
        "Win rate {:.1}%   (4-player baseline 25%)   average rounds survived {:.1}",
        100.0 * base,
        r.survived as f64 / n
    );

    println!("\n-- How it wins --");
    if r.win != 0 {
        for (k, v) in r.wintype.most_common() {
            println!("  {k:10} {:5.1}% of wins", (100 * v) as f64 / r.win as f64);
        }
    }
    let tk = r.kills.total();
    if tk != 0 {
        println!("  Opponents it eliminated: {:.2} per game, by", tk as f64 / n);
        for (k, v) in r.kills.most_common() {
            if v != 0 {
                println!("    {k:17} {:5.1}%", (100 * v) as f64 / tk as f64);
            }
        }
    }
    let td = r.dmg.total();
    if td != 0 {
        println!("  Damage dealt per game: {:.1}, from", td as f64 / n);
        for (k, v) in r.dmg.most_common() {
            if v != 0 {
                println!("    {k:17} {:6.1}  ({:.0}%)", v as f64 / n, (100 * v) as f64 / td as f64);
            }
        }
    }

    println!("\n-- How it loses --");
    let nd = r.death.total();
    if nd != 0 {
        println!(
            "  Eliminated in {:.1}% of games, on average in round {:.1}",
            (100 * nd) as f64 / n,
            r.death_round as f64 / nd as f64
        );
        for ((who, how), v) in r.death.most_common().into_iter().take(8) {
            let who = if !crate::decks::MINE.contains(&who.as_str()) && who != "self" { name_of(&who) } else { who };
            println!("    by {who:8} via {how:17} {:5.1}% of games", (100 * v) as f64 / n);
        }
    }

    println!("\n-- Game plan timing --");
    println!("  Plan: {}", plan_text(deck));
    let mut cum = 0;
    let pn = r.plan.total();
    let mut line = vec![];
    for t in 2..13u32 {
        cum += r.plan.get(&t);
        line.push(format!("T{t} {:.0}%", (100 * cum) as f64 / n));
    }
    println!("  Online by turn:  {}", line.join("  "));
    println!("  Online at all: {:.1}% of games", (100 * pn) as f64 / n);
    if matches!(deck, "veyran" | "sauron") {
        println!("  Combo attempted in {:.1}% of games", (100 * r.combo) as f64 / n);
    }
    if deck == "veyran" {
        println!("  Aetherflux fired in {:.1}% of games", (100 * r.aether) as f64 / n);
    }

    println!("\n-- Card report (correlation, not causation: see notes) --");
    // (name, seen, cast, won when cast, lost, held)
    struct Row<'a> {
        name: &'a str,
        seen: i64,
        cast: i64,
        win_cast: i64,
        lost: i64,
        held: bool,
    }
    let rows: Vec<Row> = cards
        .iter()
        .filter(|c| !c.land)
        .map(|c| Row {
            name: &c.name,
            seen: r.seen.get(&c.name),
            cast: r.cast.get(&c.name),
            win_cast: r.win_cast.get(&c.name),
            lost: r.lost.get(&c.name),
            held: c.held,
        })
        .collect();
    let minn = 100.max(nn / 50);
    let mut rated: Vec<&Row> = rows.iter().filter(|x| x.cast >= minn).collect();
    let wr = |x: &Row| x.win_cast as f64 / x.cast as f64;
    rated.sort_by(|a, b| (-wr(a)).partial_cmp(&-wr(b)).unwrap());
    let trunc = |s: &str| s.chars().take(42).collect::<String>();
    let fmt = |x: &Row| {
        format!(
            "    {:42} cast in {:5.1}% of games, win rate when cast {:5.1}% ({:+.1})",
            trunc(x.name),
            (100 * x.cast) as f64 / n,
            (100 * x.win_cast) as f64 / x.cast as f64,
            100.0 * (x.win_cast as f64 / x.cast as f64 - base)
        )
    };
    println!("  Highest win rate when cast (cast in at least {minn} games):");
    for x in rated.iter().take(10) {
        println!("{}", fmt(x));
    }
    println!("  Lowest win rate when cast:");
    for x in rated.iter().skip(rated.len().saturating_sub(10)).rev() {
        println!("{}", fmt(x));
    }
    let ratio = |x: &Row| x.cast as f64 / x.seen as f64;
    let mut dead: Vec<&Row> = rows.iter().filter(|x| x.seen >= minn && ratio(x) < 0.5).collect();
    dead.sort_by(|a, b| ratio(a).partial_cmp(&ratio(b)).unwrap());
    println!("  Often stuck in hand (drawn, but cast in under half of those games):");
    for x in dead.iter().take(12) {
        let why = if x.held {
            " [reactive: only cast in response]"
        } else if x.cast == 0 {
            " [never cast by the sim: untagged]"
        } else {
            ""
        };
        println!(
            "    {:42} cast {:5.1}% of the games it was drawn{why}",
            trunc(x.name),
            (100 * x.cast) as f64 / x.seen.max(1) as f64
        );
    }
    let mut lost: Vec<&Row> = rows.iter().filter(|x| x.lost != 0).collect();
    lost.sort_by(|a, b| b.lost.cmp(&a.lost));
    if !lost.is_empty() {
        println!("  Most often removed by opponents (per game):");
        for x in lost.iter().take(8) {
            println!("    {:42} {:.2}", trunc(x.name), x.lost as f64 / n);
        }
    }
    println!("\n  Notes: a high win rate when cast can mean the card helps, or that it only gets cast in");
    println!("  games that were already going well (expensive cards, finishers). Low numbers on cheap cards");
    println!("  that are cast early are the most telling. Use the list to pick swaps, then confirm each one");
    println!("  with a normal paired comparison.");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn verdicts() {
        let one = s(&["loose"]);
        let two = s(&["conservative", "loose"]);
        assert_eq!(verdict(&[0.01], &[0.01], &one), "NO CLEAR EFFECT (within noise)");
        assert_eq!(verdict(&[0.05], &[0.01], &one), "BETTER under the loose profile (the only one run)");
        assert_eq!(verdict(&[0.05, 0.04], &[0.01, 0.01], &two), "BETTER under both profiles");
        assert_eq!(verdict(&[-0.05, -0.04], &[0.01, 0.01], &two), "WORSE under both profiles");
        assert_eq!(
            verdict(&[0.05, -0.04], &[0.01, 0.01], &two),
            "TABLE-DEPENDENT (clear under both profiles, but in opposite directions)"
        );
        assert_eq!(
            verdict(&[0.01, -0.04], &[0.01, 0.01], &two),
            "LEANS WORSE (clear only under loose; rerun with a larger --n to confirm)"
        );
        assert_eq!(verdict(&[0.0, 0.0], &[0.0, 0.0], &two), "NO CLEAR EFFECT (within noise)");
    }

    #[test]
    fn python_float_formatting() {
        // f'{x:.1f}' and f'{x:+.1f}' in Python: exact decimal value, ties to even
        let xs = [0.25, 0.35, 2.5, -0.04, 1.05, 0.125, 12.345, 100.0 * 0.0125];
        let a: Vec<String> = xs.iter().map(|x| format!("{x:.1}")).collect();
        let b: Vec<String> = xs.iter().map(|x| format!("{x:+.1}")).collect();
        assert_eq!(a, ["0.2", "0.3", "2.5", "-0.0", "1.1", "0.1", "12.3", "1.2"]);
        assert_eq!(b, ["+0.2", "+0.3", "+2.5", "-0.0", "+1.1", "+0.1", "+12.3", "+1.2"]);
        assert_eq!(format!("{:+.1}", -0.0f64), "-0.0");
        assert_eq!((format!("{:.0}", 0.5), format!("{:.0}", 1.5)), ("0".into(), "2".into()));
    }
}
