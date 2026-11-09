//! The mulligan hint: the AI's own keep rule (`engine::turn::keeps_hand`) applied to your opening hand, with the rule
//! that decided it named in plain words, what the hand does in the deck's terms (mana, tutors, win lines, protection,
//! colours, speed), for Tymna/Thrasios the mulligan oracle's numbers for this exact hand, and, once you keep after
//! mulligans, the card the AI would put on the bottom and why.
//!
//! The verdict (`choice`) is `keeps_hand`'s and nothing here changes it: this module only explains it.

use serde_json::{Value, json};
use sim_core::ai::plan_data::{self, KeepBy, KeepTerm, KeepWhy, Loaded};
use sim_core::cards::Colors;
use sim_core::engine::turn::{Seat, bottom_pick, keeps_hand};
use sim_core::human::choices::card_label;
use sim_core::ids::{CardId, PlayerId};
use sim_core::impls::t6_keep;
use sim_core::state::Game;
use sim_core::tag::Tag;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

const COLOURS: [(char, &str); 5] = [('W', "white"), ('U', "blue"), ('B', "black"), ('R', "red"), ('G', "green")];

fn names(g: &Game, v: &[CardId]) -> String {
    let mut out: Vec<String> = v.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    out.sort();
    out.join(", ")
}

fn name_of(g: &Game, c: CardId) -> String {
    g.db.get(c).name.to_string()
}

fn num(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 { format!("{}", v.round() as i64) } else { format!("{v:.1}") }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// how many cards the AI would keep: the hand size in the prompt "Your opening hand (7 to keep)."
fn cards_to_keep(prompt: &str) -> usize {
    prompt.split('(').nth(1).and_then(|s| s.split(' ').next()).and_then(|s| s.parse().ok()).unwrap_or(7)
}

/// which draw this is, as the engine numbers it for Tier 6 keeps (`mull_try`: 0 the first seven, 1 the second, 2 and
/// 3 the hands that bottom cards): from the request's `mulligans`, else from the hand size in the prompt
pub fn tries_of(data: &Value, prompt: &str) -> i64 {
    data.get("mulligans").and_then(Value::as_i64).unwrap_or_else(|| {
        match 7usize.saturating_sub(cards_to_keep(prompt)) {
            0 => 0,
            b => b as i64 + 1,
        }
    })
}

/// run `f` with a Tier 6 deck told which draw this is, as `engine::turn::mulligan` tells it
fn with_tries<T>(g: &mut Game, p: PlayerId, tries: i64, f: impl FnOnce(&Game) -> T) -> T {
    if !g.player(p).t6 {
        return f(g);
    }
    let old = g.player(p).stats.get("mull_try").copied();
    g.player_mut(p).stats.insert("mull_try", tries);
    let r = f(g);
    match old {
        Some(v) => g.player_mut(p).stats.insert("mull_try", v),
        None => g.player_mut(p).stats.shift_remove("mull_try"),
    };
    r
}

// ------------------------------------------------------------------ the colours a card or land makes
fn land_colours(g: &Game, p: PlayerId, c: CardId) -> Colors {
    let d = g.db.get(c);
    let mut s = String::new();
    for (ch, _) in COLOURS {
        if d.identity.is_some_and(|i| i.has(ch)) || d.land_types.has(ch) {
            s.push(ch);
        }
    }
    Colors::from_letters(&s).union(t6_keep::tag_cols(g, p, c))
}

/// the colours a land of the hand makes: a fetch land makes those of the lands left in the library
fn land_makes(g: &Game, p: PlayerId, c: CardId) -> Colors {
    if g.db.get(c).tag(Tag::F) {
        return g
            .player(p)
            .library
            .iter()
            .filter(|&&x| g.db.get(x).land && !g.db.get(x).tag(Tag::F))
            .fold(Colors::NONE, |a, &x| a.union(land_colours(g, p, x)));
    }
    land_colours(g, p, c)
}

// ------------------------------------------------------------------ reading the keep rule
/// words for a set of the plan's `[sets]`
fn set_label(set: &str) -> String {
    match set {
        "tutors" => "Tutors".into(),
        "payoffs" => "Payoffs (cards that win or bury the table by themselves)".into(),
        "exilers" => "Exile-half cards of the Oracle line".into(),
        "enablers" => "Cards that enable the pile".into(),
        "engines" => "Draw or value engines".into(),
        "action" => "Action cards (a tutor, payoff, Oracle, Doomsday or a cantrip)".into(),
        "pieces" => "Win-line pieces".into(),
        "keep2" => "Cheap plays that make a 2-lander keepable (a mana rock or a creature costing 2 or less)".into(),
        "locks" => "Stax locks".into(),
        "led_fuel" => "Cards Lion's Eye Diamond fuels".into(),
        s => format!("Cards of the plan's \"{s}\" set"),
    }
}

fn set_cards<'a>(l: &'a Loaded, set: &str) -> &'a [String] {
    l.plan.sets.get(set).map_or(&[][..], |v| &v[..])
}

fn hand_named(g: &Game, p: PlayerId, list: &[String]) -> Vec<CardId> {
    g.player(p).hand.iter().copied().filter(|&c| list.iter().any(|n| **n == *g.db.get(c).name)).collect()
}

fn rituals_in_hand(g: &Game, p: PlayerId, l: &Loaded) -> Vec<CardId> {
    l.plan.mulligan.as_ref().map_or(vec![], |m| hand_named(g, p, &m.rituals))
}

/// "Mox Diamond 0.5, Sol Ring 2" for the pieces that make something by turn 2
fn counted_pieces(g: &Game, p: PlayerId) -> Vec<(CardId, f64)> {
    t6_keep::piece_values(g, p).into_iter().filter(|x| x.1 > 0.0).collect()
}

/// one rule condition in words: the label, what the rule wants, and what this hand has
struct Item {
    label: String,
    cmp: String,
    have: String,
}

fn cmp_words(op: &str, rhs: &str) -> String {
    match (op, rhs) {
        ("=", "0") => "none".into(),
        ("=", n) => format!("exactly {n}"),
        (">=", "1") => "at least 1".into(),
        (">=", n) => format!("{n} or more"),
        (">", n) => format!("more than {n}"),
        ("<=", n) => format!("at most {n}"),
        ("<", n) => format!("fewer than {n}"),
        (o, n) => format!("{o} {n}"),
    }
}

fn with_names(count: String, g: &Game, ids: &[CardId]) -> String {
    if ids.is_empty() { count } else { format!("{count}: {}", names(g, ids)) }
}

/// the keep rule's mana count in words: "3.5 = 2 lands + Mox Diamond 0.5 + Sol Ring 2" (every land in hand, not turn-2 mana)
fn mana_breakdown(g: &Game, p: PlayerId) -> String {
    let m = t6_keep::mana(g, p);
    let mut parts = vec![plural(m.lands, "land", "lands")];
    let pieces = counted_pieces(g, p);
    for &(c, v) in &pieces {
        parts.push(format!("{} {}", name_of(g, c), num(v)));
    }
    let extra = m.eff - m.lands as f64 - pieces.iter().map(|x| x.1).sum::<f64>();
    if extra > 0.01 {
        parts.push(format!("{} from double-mana lands", num(extra)));
    }
    format!("{} = {}", num(m.eff), parts.join(" + "))
}

fn item_of(g: &Game, p: PlayerId, l: &Loaded, t: &KeepTerm) -> Option<Item> {
    let value = t.value?;
    let rhs = if t.rhs == "one-lander" {
        format!("{} (a one-land hand's bar on this draw)", num(t6_keep::mana(g, p).need_one_lander()))
    } else {
        t.rhs.clone()
    };
    let cmp = cmp_words(&t.op, &rhs);
    let hand = &g.player(p).hand;
    let lands: Vec<CardId> = hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
    let item = |label: &str, have: String| Some(Item { label: label.into(), cmp: cmp.clone(), have });
    match t.lhs.as_str() {
        "hand-lands" => item("Lands", with_names(num(value), g, &lands)),
        "hand-b-lands" => item("Black-making lands", num(value)),
        "nonland" => item("Spells (nonland cards)", num(value)),
        "hand-size" => item("Cards in hand", num(value)),
        "tutors" => {
            let v: Vec<CardId> = hand.iter().copied().filter(|&c| g.db.get(c).tag(Tag::Tut)).collect();
            item("Tutors", with_names(num(value), g, &v))
        }
        "fast" => {
            let mut v: Vec<CardId> = counted_pieces(g, p).into_iter().map(|x| x.0).collect();
            v.extend(rituals_in_hand(g, p, l));
            item("Fast mana (Moxen, Sol Ring, cheap rocks, rituals)", with_names(num(value), g, &v))
        }
        "makers" => {
            let v: Vec<CardId> = counted_pieces(g, p).into_iter().map(|x| x.0).collect();
            item("Mana pieces that count by turn 2", with_names(num(value), g, &v))
        }
        "mana" | "eff" => {
            let r = rituals_in_hand(g, p, l);
            let extra = if t.lhs == "mana" && !r.is_empty() {
                format!(", plus {} ({})", plural(r.len(), "ritual", "rituals"), names(g, &r))
            } else {
                String::new()
            };
            item("Mana as the keep rule counts it", format!("{} ({}{extra})", num(value), mana_breakdown(g, p)))
        }
        s if s.starts_with("hand:") => {
            let set = &s["hand:".len()..];
            let v = hand_named(g, p, set_cards(l, set));
            item(&set_label(set), with_names(num(value), g, &v))
        }
        s if s.starts_with("hand-castable:") => {
            let set = &s["hand-castable:".len()..];
            let have = t6_keep::hand_sources(g, p);
            let v: Vec<CardId> = hand_named(g, p, set_cards(l, set))
                .into_iter()
                .filter(|&c| t6_keep::covers(have, g.db.get(c).colors))
                .collect();
            item(
                &format!("{} that your lands and mana can cast in colour", set_label(set)),
                with_names(num(value), g, &v),
            )
        }
        "hand-castable" => item("Spells your lands and mana can cast in colour", num(value)),
        "hand-gaps" => item("Colours your spells need that nothing in the hand makes", num(value)),
        s if s.starts_with("hand-gaps:") => {
            item(&format!("Colours the {} need that nothing in the hand makes", &s["hand-gaps:".len()..]), num(value))
        }
        s => item(s, num(value)),
    }
}

/// the conditions of one alternative as items (a `>=` and a `<=` on the same quantity become one range);
/// conditions that are not "quantity op number" come back as raw text in `label`
fn items_of(g: &Game, p: PlayerId, l: &Loaded, alt: &str) -> Vec<Item> {
    let terms = plan_data::keep_terms(l, g, p, alt);
    let mut out: Vec<Item> = vec![];
    let mut i = 0;
    while i < terms.len() {
        let t = &terms[i];
        if let Some(n) = terms.get(i + 1)
            && t.lhs == n.lhs
            && t.op == ">="
            && n.op == "<="
            && let Some(mut it) = item_of(g, p, l, t)
        {
            it.cmp = format!("{} to {}", t.rhs, n.rhs);
            out.push(it);
            i += 2;
            continue;
        }
        out.push(item_of(g, p, l, t).unwrap_or(Item {
            label: format!("Condition \"{}\"", t.text),
            cmp: String::new(),
            have: String::new(),
        }));
        i += 1;
    }
    out
}

fn item_short(i: &Item) -> String {
    if i.cmp.is_empty() { i.label.clone() } else { format!("{}: {}", i.label, i.cmp) }
}

fn item_full(i: &Item) -> String {
    if i.cmp.is_empty() { i.label.clone() } else { format!("{}: {} (you have {})", i.label, i.cmp, i.have) }
}

/// the rule that decided: a short phrase for the verdict line and the full sentence for the first detail line
struct RuleText {
    short: String,
    full: String,
}

fn rule_text(g: &Game, p: PlayerId, n_lands: usize, why: Option<&KeepWhy>) -> RuleText {
    let deck = g.player(p).name.to_string();
    let Some(why) = why else {
        return RuleText {
            short: "the default rule keeps 2 to 5 lands".to_string(),
            full: format!(
                "Rule: {deck} has no keep table of its own, so the AI's default applies: keep a hand with 2 to 5 lands (this one has {n_lands}). It does not look at tutors, ramp or win lines, so judge those yourself below."
            ),
        };
    };
    let l = plan_data::active(g, p);
    let act = if why.keep { "KEEPS" } else { "MULLIGANS" };
    match (&why.by, l) {
        (KeepBy::Settled, _) => RuleText {
            short: "you already mulliganed, and the table keeps a settled hand then".into(),
            full: format!(
                "Rule: {deck}'s keep table keeps a settled hand once you have mulliganed: any hand with 2 to 4 lands from the second draw on, or a 1-lander with 2 or more mana and 2 spells after two mulligans. The rows below it are skipped."
            ),
        },
        (KeepBy::Row { idx, alt }, Some(l)) => {
            let items = items_of(g, p, l, alt);
            let short = items.iter().map(item_short).collect::<Vec<_>>().join("; ");
            let full = items.iter().map(item_full).collect::<Vec<_>>().join("; ");
            RuleText {
                short: format!("row {idx} of {} of the keep table, {short}", why.rows),
                full: format!(
                    "Rule: row {idx} of {} in {deck}'s keep table {act} a hand with {full}. (The rows are tried in order; the first that fits decides.)",
                    why.rows
                ),
            }
        }
        (KeepBy::Otherwise, l) => {
            let keeps_rows = l.map_or(String::new(), |l| keep_rows_words(g, p, l));
            // the rows that keep and the rows that mulligan: a table of only mulligan rows is a list of reasons to
            // ship a hand, so a hand none of them fits is kept (and the other way round)
            let (n_keep, n_mull) = l.and_then(|l| l.plan.mulligan.as_ref()).map_or((0, 0), |m| {
                m.rule.iter().fold((0, 0), |(k, m), r| if r.keep { (k + 1, m) } else { (k, m + 1) })
            });
            let n = why.rows;
            let s = if n == 1 { "" } else { "s" };
            match (why.keep, n_keep, n_mull) {
                (true, 0, _) => RuleText {
                    short: format!("none of the {n} reason{s} to mulligan fits, so keep"),
                    full: format!(
                        "Rule: none of {deck}'s {n} reason{s} to mulligan fits this hand, so keep it. (Its keep table lists only hands to send back; a hand that passes them all is kept.)"
                    ),
                },
                (false, _, 0) => RuleText {
                    short: format!("none of the {n} kind{s} of hand it keeps fits, so mulligan"),
                    full: format!(
                        "Rule: none of the {n} kind{s} of hand {deck} keeps fits this one, so mulligan it. (Its keep table lists only hands to keep; any other hand goes back.){keeps_rows}"
                    ),
                },
                _ => {
                    let (done, act) = if why.keep { ("kept", "keep it") } else { ("sent back", "mulligan it") };
                    RuleText {
                        short: format!("no row of the keep table fits; a hand no row covers is {done}"),
                        full: format!(
                            "Rule: none of the {n} rows of {deck}'s keep table fits this hand ({n_keep} keep a hand, {n_mull} send one back). A hand no row covers is {done}, so {act}.{keeps_rows}"
                        ),
                    }
                }
            }
        }
        (KeepBy::Lands { lo, hi }, _) => RuleText {
            short: format!("outside the plan's {lo} to {hi} lands"),
            full: format!("Rule: {deck}'s plan keeps a hand of {lo} to {hi} lands only; this one has {n_lands}."),
        },
        (KeepBy::Pieces { need, have }, _) => RuleText {
            short: format!("{have} of the {need} key cards the plan wants in the hand"),
            full: format!(
                "Rule: {deck}'s plan keeps a hand whose lands are in range and that holds at least {need} win-line or engine card(s); this hand holds {have}."
            ),
        },
        (_, None) => RuleText { short: String::new(), full: String::new() },
    }
}

/// what the table keeps, for a hand that no row fits and the default sends back ("The table keeps: ...")
fn keep_rows_words(g: &Game, p: PlayerId, l: &Loaded) -> String {
    let Some(m) = &l.plan.mulligan else { return String::new() };
    let rows: Vec<String> = m
        .rule
        .iter()
        .filter(|r| r.keep)
        .map(|r| {
            let alts: Vec<String> = r
                .when
                .iter()
                .map(|w| items_of(g, p, l, w).iter().map(item_short).collect::<Vec<_>>().join(" and "))
                .collect();
            alts.join(" or ")
        })
        .collect();
    if rows.is_empty() || m.otherwise.unwrap_or(true) {
        String::new()
    } else {
        format!(" The table keeps: {}.", rows.join("; or "),)
    }
}

// ------------------------------------------------------------------ the hand in the deck's terms
/// the cards of the hand that tutor, the win lines they belong to, protection and engines
struct Key {
    tutors: Vec<CardId>,
    pieces: Vec<CardId>,
    protection: Vec<CardId>,
    free: Vec<CardId>,
    engines: Vec<CardId>,
}

fn key_cards(g: &Game, p: PlayerId, l: Option<&Loaded>) -> Key {
    let hand: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| !g.db.get(c).land).collect();
    let in_list = |c: CardId, list: &[String]| list.iter().any(|n| **n == *g.db.get(c).name);
    let mut tutors = vec![];
    let mut pieces = vec![];
    let mut protection = vec![];
    let mut free = vec![];
    let mut engines = vec![];
    for &c in &hand {
        let d = g.db.get(c);
        let tut = d.tag(Tag::Tut) || l.is_some_and(|l| in_list(c, set_cards(l, "tutors")));
        let piece = l.is_some_and(|l| {
            l.lines.iter().any(|(gs, _)| gs.iter().flatten().any(|n| **n == *d.name))
                || in_list(c, set_cards(l, "payoffs"))
                || in_list(c, set_cards(l, "pieces"))
        });
        let eng = d.tag(Tag::Eng)
            || d.tag(Tag::Rhystic)
            || d.tag(Tag::Remora)
            || l.is_some_and(|l| in_list(c, &l.plan.engines) || in_list(c, set_cards(l, "engines")));
        if tut {
            tutors.push(c);
        }
        if piece && !tut {
            pieces.push(c);
        }
        if sim_core::goldfish::protection_card(&g.db, c) {
            protection.push(c);
            if d.tag(Tag::Free) || (d.cmc == 0 && d.tag(Tag::Ctr)) {
                free.push(c);
            }
        }
        if eng {
            engines.push(c);
        }
    }
    Key { tutors, pieces, protection, free, engines }
}

fn hand_lines(g: &Game, p: PlayerId, l: Option<&Loaded>) -> Vec<String> {
    let mut d = vec![];
    let hand = &g.player(p).hand;
    let lands: Vec<CardId> = hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
    let spells = hand.len() - lands.len();
    d.push(if lands.is_empty() {
        format!("Lands: none, {}.", plural(spells, "spell", "spells"))
    } else {
        format!("Lands ({}): {}; {}.", lands.len(), names(g, &lands), plural(spells, "spell", "spells"))
    });
    // mana by turn 2, counted honestly (t6_keep)
    let m = t6_keep::mana(g, p);
    let pieces = t6_keep::piece_values(g, p);
    let curve = Curve::of(g, p, m.lands);
    let mut line = format!("Mana on turn 2: {}.", curve.turn2_breakdown(g));
    let dead: Vec<String> = pieces
        .iter()
        .filter(|x| x.1 <= 0.0)
        .map(|&(c, _)| format!("{} ({})", name_of(g, c), why_no_mana(g, c, m.lands, m.spells)))
        .collect();
    if !dead.is_empty() {
        line += &format!(" Makes nothing from this hand: {}.", dead.join("; "));
    }
    let rit = l.map_or(vec![], |l| rituals_in_hand(g, p, l));
    if !rit.is_empty() {
        line += &format!(
            " Rituals and spirit guides ({}) only fuel one spell each and are not in that number.",
            names(g, &rit)
        );
    }
    d.push(line);
    if (m.eff - curve.real(2)).abs() > 0.05 {
        d.push(format!(
            "The keep rule counts this hand as {} mana (all {} in hand plus the rocks that count), not what you have on turn 2.",
            num(m.eff),
            plural(m.lands, "land", "lands")
        ));
    }
    let k = key_cards(g, p, l);
    let mut keys = vec![];
    if !k.tutors.is_empty() {
        keys.push(format!("tutors - {}", names(g, &k.tutors)));
    }
    if !k.pieces.is_empty() {
        keys.push(format!("win-line pieces - {}", names(g, &k.pieces)));
    }
    if !k.protection.is_empty() {
        let free = if k.free.is_empty() { String::new() } else { format!(" (free to cast: {})", names(g, &k.free)) };
        keys.push(format!("protection - {}{free}", names(g, &k.protection)));
    }
    if !k.engines.is_empty() {
        keys.push(format!("draw engines - {}", names(g, &k.engines)));
    }
    d.push(if keys.is_empty() {
        "Key cards: no tutor, win-line piece, protection or draw engine in this hand.".to_string()
    } else {
        format!("Key cards: {}.", keys.join("; "))
    });
    d
}

fn why_no_mana(g: &Game, c: CardId, lands: usize, spells: usize) -> &'static str {
    let d = g.db.get(c);
    match &*d.name {
        "Mox Diamond" => "needs a second land to discard",
        "Chrome Mox" if spells < 2 => "needs two other spells, one to imprint",
        "Mox Opal" => "needs three artifacts for metalcraft",
        "Mox Jasper" => "needs a Dragon you can cast first",
        "Lion's Eye Diamond" => "only fuels one spell",
        _ if d.cmc == 1 && lands < 1 => "needs a land to cast it",
        _ if d.cmc == 2 && lands < 2 => "needs two lands to cast it",
        _ => "no mana by turn 2",
    }
}

// ------------------------------------------------------------------ colours
fn pips(ch: char, n: usize) -> String {
    (0..n).map(|_| format!("{{{ch}}}")).collect()
}

/// per colour the plan's key spells (and the commanders) need: how many sources of it the hand has
fn colour_lines(g: &Game, p: PlayerId, l: Option<&Loaded>) -> Vec<String> {
    let pl = g.player(p);
    let k = key_cards(g, p, l);
    let mut spells: Vec<CardId> = vec![];
    for c in k.tutors.iter().chain(&k.pieces).chain(&k.engines) {
        if !spells.contains(c) {
            spells.push(*c);
        }
    }
    if let Some(l) = l {
        for c in hand_named(g, p, set_cards(l, "action")) {
            if !spells.contains(&c) {
                spells.push(c);
            }
        }
    }
    spells.extend(pl.commanders());
    // the sources the hand has, by colour
    let lands: Vec<CardId> = pl.hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
    let mine = t6_keep::piece_values(g, p);
    let mut out = vec![];
    for (ch, cname) in COLOURS {
        let mut needs: Vec<(usize, String)> = spells
            .iter()
            .filter_map(|&c| {
                let n = g.db.get(c).pips.chars().filter(|&x| x == ch).count();
                (n > 0).then(|| (n, name_of(g, c)))
            })
            .collect();
        if needs.is_empty() {
            continue;
        }
        needs.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        needs.dedup_by(|a, b| a.1 == b.1);
        let most = needs[0].0;
        let mut src: Vec<CardId> = lands.iter().copied().filter(|&c| land_makes(g, p, c).has(ch)).collect();
        for &(c, v) in &mine {
            if v > 0.0 && t6_keep::piece_cols(g, p, c).has(ch) {
                src.push(c);
            }
        }
        let shown: Vec<String> = needs.iter().take(3).map(|(n, c)| format!("{c} {}", pips(ch, *n))).collect();
        let more = if needs.len() > 3 { format!(" and {} more", needs.len() - 3) } else { String::new() };
        let have = format!(
            "the hand has {} ({})",
            plural(src.len(), &format!("{cname} source"), &format!("{cname} sources")),
            if src.is_empty() { "none".to_string() } else { names(g, &src) }
        );
        let verdict = if src.is_empty() {
            format!("nothing makes {cname} yet, so those cannot be cast until you draw a source")
        } else if src.len() < most {
            format!("{} short for the heaviest (needs {most}), a draw away", most - src.len())
        } else {
            "enough".to_string()
        };
        out.push(format!("{}: needed by {}{more}; {have}: {verdict}.", capital(cname), shown.join(", ")));
    }
    if out.is_empty() {
        out.push("Colours: none of the key spells or the commanders need coloured mana beyond the generic.".into());
    } else {
        out.insert(0, "Colours the plan needs:".into());
    }
    out
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().collect::<String>() + c.as_str())
}

// ------------------------------------------------------------------ speed
/// a mana piece the hand casts: what it makes (`t6_keep`'s value), the turn it is cast and the first turn it gives mana
struct Played {
    card: CardId,
    v: f64,
    cast: usize,
    from: usize,
    /// Mox Jasper: the Dragon whose casting on turn `cast` switches it on
    dragon: Option<CardId>,
}

/// The mana the hand has on turn `t`, turn by turn: at most `cap` lands in all and one land drop a turn (never more
/// lands than the turn number), the double-mana lands, and the pieces the hand casts with the mana it has (cheapest
/// first; a piece cast on turn `n` gives mana from turn `n + 1`, a free Mox the turn it is cast). Mana only: colours
/// and opponents are ignored.
struct Curve {
    cap: usize,
    extra: f64,
    played: Vec<Played>,
    mana: Vec<f64>,
    /// per turn: is the Lotus Petal's one mana still there (counted in `mana` that turn)
    petal: Vec<bool>,
    petal_card: Option<CardId>,
}

impl Curve {
    fn of(g: &Game, p: PlayerId, cap: usize) -> Curve {
        let m = t6_keep::mana(g, p);
        let pv = t6_keep::piece_values(g, p);
        let sum: f64 = pv.iter().map(|x| x.1).sum();
        let extra = (m.eff - m.lands as f64 - sum).max(0.0);
        let jasper = |c: CardId| &*g.db.get(c).name == "Mox Jasper";
        let is_petal = |c: CardId| &*g.db.get(c).name == "Lotus Petal";
        // Lotus Petal is one mana, once: it is there each turn until it pays for something
        let petal_card = pv.iter().find(|x| x.1 > 0.0 && is_petal(x.0)).map(|x| x.0);
        let mut petal_left = petal_card.is_some();
        let mut petal = vec![];
        let mut left: Vec<(CardId, f64, usize)> = pv
            .iter()
            .filter(|x| x.1 > 0.0 && !jasper(x.0) && !is_petal(x.0))
            .map(|&(c, v)| (c, v, g.db.get(c).cmc as usize))
            .collect();
        // Mox Jasper is not mana until a Dragon is cast: the turn the hand can cast one (its cost leaves that turn's
        // mana, and Jasper makes mana from the next turn)
        let dragon = if pv.iter().any(|x| jasper(x.0)) { t6_keep::dragon_cast(g, p, cap) } else { None };
        left.sort_by(|a, b| a.2.cmp(&b.2).then(b.1.total_cmp(&a.1)));
        let mut played: Vec<Played> = vec![];
        let mut mana = vec![];
        for t in 1..=10usize {
            let lands = t.min(cap);
            let ex = if lands >= 1 { extra } else { 0.0 };
            // the mana without the Petal, and what this turn spends of it (the Petal is spent once that passes the rest)
            let start = lands as f64 + ex + played.iter().filter(|x| x.from <= t).map(|x| x.v).sum::<f64>();
            let petal_now = petal_left as u8 as f64;
            let (mut rem, mut added, mut spent) = (start + petal_now, 0.0, 0.0);
            // the Dragon's cost leaves this turn's mana after the zero-cost pieces have added theirs (they are first in
            // `left`, which is sorted by cost)
            let mut reserve = None;
            if let Some((dt, dc)) = dragon.filter(|x| x.0 == t) {
                reserve = Some(g.db.get(dc).cmc as f64);
                for &(c, _) in pv.iter().filter(|x| jasper(x.0)) {
                    played.push(Played { card: c, v: 1.0, cast: dt, from: dt + 1, dragon: Some(dc) });
                }
            }
            let mut i = 0;
            while i < left.len() {
                let (c, v, cmc) = left[i];
                if cmc > 0 {
                    if let Some(cost) = reserve.take() {
                        let pay = cost.min(rem);
                        rem -= pay;
                        spent += pay;
                    }
                }
                if cmc as f64 <= rem + 1e-9 {
                    rem -= cmc as f64;
                    spent += cmc as f64;
                    let from = if cmc == 0 { t } else { t + 1 };
                    if from == t {
                        rem += v;
                        added += v;
                    }
                    played.push(Played { card: c, v, cast: t, from, dragon: None });
                    left.remove(i);
                } else {
                    i += 1;
                }
            }
            if let Some(cost) = reserve.take() {
                spent += cost.min(rem);
            }
            mana.push(lands as f64 + ex + played.iter().filter(|x| x.from <= t).map(|x| x.v).sum::<f64>() + petal_now);
            petal.push(petal_left);
            if spent > start + added + 1e-9 {
                petal_left = false;
            }
        }
        Curve { cap, extra, played, mana, petal, petal_card }
    }

    fn real(&self, t: usize) -> f64 {
        self.mana[t - 1]
    }

    fn at(&self, t: usize) -> usize {
        self.real(t).round() as usize
    }

    /// the real mana on turn 2 in words: "4 = 2 lands + Sol Ring 2 (cast turn 1)"
    fn turn2_breakdown(&self, g: &Game) -> String {
        let mut parts = vec![plural(2.min(self.cap), "land", "lands")];
        for x in self.played.iter().filter(|x| x.from <= 2) {
            let when = match x.dragon {
                Some(d) => format!(" (once {} is cast turn {})", name_of(g, d), x.cast),
                None if g.db.get(x.card).cmc > 0 => format!(" (cast turn {})", x.cast),
                None => String::new(),
            };
            parts.push(format!("{} {}{when}", name_of(g, x.card), num(x.v)));
        }
        if let Some(c) = self.petal_card.filter(|_| self.petal.get(1) == Some(&true)) {
            parts.push(format!("{} 1 (once)", name_of(g, c)));
        }
        if self.extra > 0.01 && self.cap >= 1 {
            parts.push(format!("{} from double-mana lands", num(self.extra)));
        }
        let later: Vec<String> = self
            .played
            .iter()
            .filter(|x| x.from > 2)
            .map(|x| match x.dragon {
                Some(d) => format!(
                    "{} (once {} is cast turn {}, makes mana from turn {})",
                    name_of(g, x.card),
                    name_of(g, d),
                    x.cast,
                    x.from
                ),
                None => format!("{} (cast turn {}, makes mana from turn {})", name_of(g, x.card), x.cast, x.from),
            })
            .collect();
        let mut s = format!("{} = {}", num(self.real(2)), parts.join(" + "));
        if !later.is_empty() {
            s += &format!("; not yet on turn 2: {}", later.join(", "));
        }
        s
    }

    /// the first turn with at least `cost` mana
    fn first(&self, cost: usize) -> Option<usize> {
        (1..=10).find(|&t| self.at(t) >= cost)
    }

    /// the first turn by which every card of `costs` could be cast, a few a turn (largest that fits first)
    fn all_cast(&self, costs: &[usize]) -> Option<usize> {
        let mut left: Vec<usize> = costs.to_vec();
        left.sort_unstable_by(|a, b| b.cmp(a));
        for t in 1..=10 {
            let mut budget = self.at(t);
            let mut i = 0;
            while i < left.len() {
                if left[i] <= budget {
                    budget -= left[i];
                    left.remove(i);
                } else {
                    i += 1;
                }
            }
            if left.is_empty() {
                return Some(t);
            }
        }
        None
    }
}

fn turn_words(t: Option<usize>) -> String {
    t.map_or("not within ten turns".into(), |t| format!("turn {t}"))
}

fn speed_lines(g: &Game, p: PlayerId, l: Option<&Loaded>) -> Vec<String> {
    let mut d = vec![];
    let curve = Curve::of(g, p, 10);
    let pl = g.player(p);
    let mut cmd = vec![];
    for c in pl.commanders() {
        let cd = g.db.get(c);
        cmd.push(format!("{} (cost {}) {}", cd.name, cd.cmc, turn_words(curve.first(cd.cmc as usize))));
    }
    d.push(format!("Speed, if every land drop hits (mana only): commander {}.", cmd.join("; ")));
    let Some(l) = l else { return d };
    let k = key_cards(g, p, Some(l));
    let status = plan_data::line_status(l, g, p);
    if status.is_empty() {
        return d;
    }
    let short = |s: &plan_data::LineStatus| s.groups.iter().filter(|x| x.held < x.need).count();
    let Some(best) = status.iter().min_by_key(|s| short(s)) else { return d };
    let miss = short(best);
    let hand = &pl.hand;
    let held_cards = |grp: &plan_data::LineGroup| -> Vec<CardId> {
        hand.iter().copied().filter(|&c| grp.cards.iter().any(|n| **n == *g.db.get(c).name)).collect()
    };
    let in_hand: Vec<CardId> = best.groups.iter().flat_map(&held_cards).collect();
    let tut = if k.tutors.is_empty() { None } else { Some(names(g, &k.tutors)) };
    if miss == 0 {
        let mut costs = vec![];
        for grp in &best.groups {
            let mut h = held_cards(grp);
            h.sort_by_key(|&c| g.db.get(c).cmc);
            costs.extend(h.iter().take(grp.need.max(1)).map(|&c| g.db.get(c).cmc as usize));
        }
        d.push(format!(
            "Line: the hand holds all of \"{}\" ({}); its cards could all be cast by {} (cost {} in all; colours and opponents ignored).",
            best.name,
            names(g, &in_hand),
            turn_words(curve.all_cast(&costs)),
            costs.iter().sum::<usize>()
        ));
    } else if miss == 1 {
        let lacking: Vec<String> = best
            .groups
            .iter()
            .filter(|x| x.held < x.need)
            .map(|x| {
                let mut c = x.cards.clone();
                let n = c.len();
                c.truncate(3);
                c.join(" / ") + if n > 3 { " / ..." } else { "" }
            })
            .collect();
        d.push(format!(
            "Line: one piece short of \"{}\" (you hold {}; missing {}). {}",
            best.name,
            if in_hand.is_empty() { "none of it".to_string() } else { names(g, &in_hand) },
            lacking.join(", "),
            match tut {
                Some(t) => format!("You hold a tutor ({t}): one tutor away."),
                None => "No tutor in the hand, so it needs a draw.".to_string(),
            }
        ));
    } else {
        let total = best.groups.len();
        d.push(format!(
            "Line: nothing close. The nearest, \"{}\", has {} of {total} pieces. {}",
            best.name,
            total - miss,
            match tut {
                Some(t) => format!("A tutor ({t}) could start it."),
                None => "No line and no tutor in the hand.".to_string(),
            }
        ));
    }
    d
}

// ------------------------------------------------------------------ the oracle (Tymna/Thrasios only)
/// playouts per arm
const ORACLE_PLAYOUTS: u64 = 40;

fn oracle_cache() -> &'static Mutex<HashMap<String, Vec<String>>> {
    static C: OnceLock<Mutex<HashMap<String, Vec<String>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// the mulligan oracle's numbers for the hand on the table (`sim_core::mull_oracle`, the machinery of
/// `--mulligan-oracle`): the KEEP and the MULLIGAN arm played as goldfish games, 40 paired playouts each, on the adaptive
/// AI, with a seed taken from the deck and the hand, so the same hand always gives the same numbers. Not cached.
pub fn oracle_uncached(g: &Game, p: PlayerId, tries: i64) -> Option<Vec<String>> {
    let pl = g.player(p);
    let db = &g.db;
    let mut seats = vec![Seat {
        key: pl.key,
        name: pl.name,
        commander: pl.cmd,
        commander2: pl.cmd2,
        ident: pl.ident,
        cards: pl.deck_names.clone(),
    }];
    seats.extend(sim_core::goldfish::inert_seats(db).ok()?);
    let mut st = (*g.settings).clone();
    st.ai = sim_core::settings::AiMode::Adaptive;
    let st = std::sync::Arc::new(st);
    let mut hand_names: Vec<String> = pl.hand.iter().map(|&c| name_of(g, c)).collect();
    hand_names.sort();
    let seed = sim_core::rng::Rng::named(&format!("hint:{}:{tries}:{}", pl.key, hand_names.join("|"))).next_u64();
    let threads = std::thread::available_parallelism().map_or(2, |n| n.get()).min(8);
    let (keep, mull) = sim_core::mull_oracle::paired_scores(
        db,
        &g.registry,
        &st,
        &seats,
        &pl.hand,
        tries,
        seed,
        0..ORACLE_PLAYOUTS,
        threads,
    )?;
    let (km, mm, _diff, _se, v) = sim_core::mull_oracle::judge(&keep, &mull);
    let says = match v {
        sim_core::mull_oracle::Verdict::Keep => "The oracle says KEEP: kept, it is faster by more than the noise.",
        sim_core::mull_oracle::Verdict::Mull => {
            "The oracle says MULLIGAN: a fresh hand is faster by more than the noise."
        }
        sim_core::mull_oracle::Verdict::Unclear => "The oracle says too close to call.",
    };
    Some(vec![
        format!(
            "Oracle, {ORACLE_PLAYOUTS} goldfish games each way: kept, this hand kills on turn {km:.1} on average; mulliganed, on turn {mm:.1} (a game with no kill by turn {} counts as {}). {says}",
            sim_core::mull_oracle::ROUNDS,
            sim_core::mull_oracle::NO_KILL
        ),
        "Limit: goldfish games have no opponents, so counters, removal and stax count for nothing in those numbers."
            .into(),
    ])
}

/// `oracle_uncached`, remembered for the hand (the hint is asked again whenever you look)
fn oracle_lines(g: &Game, p: PlayerId, tries: i64) -> Vec<String> {
    let pl = g.player(p);
    if pl.key != "thrasios" || tries > 1 || pl.hand.len() != 7 {
        return vec![];
    }
    let mut hand_names: Vec<String> = pl.hand.iter().map(|&c| name_of(g, c)).collect();
    hand_names.sort();
    let key = format!("{tries}:{}", hand_names.join("|"));
    if let Some(v) = oracle_cache().lock().ok().and_then(|c| c.get(&key).cloned()) {
        return v;
    }
    let v = oracle_uncached(g, p, tries).unwrap_or_default();
    if !v.is_empty()
        && let Ok(mut c) = oracle_cache().lock()
    {
        c.insert(key, v.clone());
    }
    v
}

// ------------------------------------------------------------------ the hint
pub fn hint(g: &mut Game, p: PlayerId, prompt: &str, tries: i64) -> Value {
    let keep_n = cards_to_keep(prompt);
    let bottom = 7usize.saturating_sub(keep_n);
    let n = g.player(p).hand.iter().filter(|&&c| g.db.get(c).land).count();
    let spells = g.player(p).hand.len() - n;
    let keeps = with_tries(g, p, tries, |g| keeps_hand(g, p));
    let why = with_tries(g, p, tries, |g| plan_data::keeps_hand_why(g, p));
    let forced = bottom >= 2; // the AI keeps its fourth hand whatever it holds
    let rule = with_tries(g, p, tries, |g| rule_text(g, p, n, why.as_ref()));
    let l = plan_data::active(g, p);

    let text = if keeps {
        format!("Keep ({n} lands, {spells} spells): {}.", rule.short)
    } else if forced {
        "Keep: this is a small hand, and the AI keeps its fourth hand whatever it holds.".to_string()
    } else {
        format!("Mulligan ({n} land{}): {}.", if n == 1 { "" } else { "s" }, rule.short)
    };

    let mut d: Vec<String> = vec![rule.full.clone()];
    d.extend(hand_lines(g, p, l));
    d.extend(colour_lines(g, p, l));
    d.extend(speed_lines(g, p, l));
    d.extend(oracle_lines(g, p, tries));
    if bottom > 0 {
        let pick = bottom_pick(g, p);
        d.push(match pick {
            Some(c) => format!(
                "If you keep, you put {bottom} card(s) on the bottom; the AI would start with {}. {}",
                card_label(g, c),
                bottom_why(g, p, c)
            ),
            None => format!("If you keep, you put {bottom} card(s) on the bottom."),
        });
    }
    let keep_it = keeps || forced;
    json!({"text": text, "detail": d, "choice": if keep_it { "keep" } else { "mulligan" }})
}

/// why the AI's pick goes to the bottom: the card the hand's plan needs least
fn bottom_why(g: &Game, p: PlayerId, c: CardId) -> String {
    let d = g.db.get(c);
    let hand = &g.player(p).hand;
    let lands = hand.iter().filter(|&&x| g.db.get(x).land).count();
    if sim_core::impls::t6_plans::is_t6(g, p) {
        let wish = sim_core::ai::plans::wish_raw(g, p);
        let mut why: Vec<String> = vec![];
        if d.land {
            why.push(format!("a land, and you hold {lands} (it bottoms a land only past the third)"));
        } else {
            why.push(format!("cost {}", d.cmc));
            if !t6_keep::is_piece(g, c) {
                why.push("not a mana piece".into());
            }
            if !wish.contains(&c) {
                why.push("not one of the cards the deck's tutors are looking for".into());
            }
        }
        let mut h = g.clone();
        if let Some(i) = h.player(p).hand.iter().position(|&x| x == c) {
            h.player_mut(p).hand.remove(i);
        }
        let still = plan_data::keeps_hand_why(&h, p);
        let n = h.player(p).hand.iter().filter(|&&x| h.db.get(x).land).count();
        let rest = if keeps_hand(&h, p) {
            format!(
                " Without it the other cards still pass the keep rule ({}).",
                rule_text(&h, p, n, still.as_ref()).short
            )
        } else {
            String::new()
        };
        format!(
            "Why: it is the card this plan needs least ({}); the AI bottoms, among the cards whose loss leaves a hand its keep rule still takes, the lowest in value.{rest}",
            why.join(", ")
        )
    } else if d.land {
        format!(
            "Why: you hold {lands} lands, more than the 4 the AI wants to keep; a land is the cheapest card to lose."
        )
    } else if sim_core::ai::values::on(g, p) {
        format!(
            "Why: it is the spell worth least to this hand by the AI's card values (cost {}); with 4 or fewer lands it bottoms a spell, not a land.",
            d.cmc
        )
    } else {
        format!(
            "Why: it is the most expensive spell in the hand (cost {}), the hardest to cast on a short hand; with 4 or fewer lands it bottoms a spell, not a land.",
            d.cmc
        )
    }
}

/// the bottom-of-library choice after a mulligan (labels are `card_label`s of the hand)
pub fn bottom(g: &mut Game, p: PlayerId, choices: &[String]) -> Value {
    let Some(c) = bottom_pick(g, p) else {
        return json!({"text": "Bottom any card: your hand is empty.", "detail": [], "choice": null});
    };
    let lab = card_label(g, c);
    let idx = choices.iter().position(|x| *x == lab);
    json!({"text": format!("Put {lab} on the bottom."),
           "detail": [bottom_why(g, p, c)],
           "choice": idx})
}

/// whether the AI would keep the hand on the table at this request, and a short reason (for the end-of-game review)
pub fn verdict(g: &Game, p: PlayerId, prompt: &str) -> (bool, String) {
    let n = g.player(p).hand.iter().filter(|&&c| g.db.get(c).land).count();
    let bottom = 7usize.saturating_sub(cards_to_keep(prompt));
    let keeps = keeps_hand(g, p);
    let rule = rule_text(g, p, n, plan_data::keeps_hand_why(g, p).as_ref());
    if keeps {
        (true, format!("Keep ({}; {})", plural(n, "land", "lands"), rule.short))
    } else if bottom >= 2 {
        (true, format!("Keep (the AI keeps its fourth hand whatever it holds; this one has {n} lands)"))
    } else {
        (false, format!("Mulligan ({}; {})", plural(n, "land", "lands"), rule.short))
    }
}
