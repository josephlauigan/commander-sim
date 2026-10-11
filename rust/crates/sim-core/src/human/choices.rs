//! The smaller choices, made by the person in the human seat: mulligans, discards, sacrifices, tutors, basic lands
//! searched for, and scry / surveil (Python's `play/choices.py`). The engine calls these where it would otherwise let
//! the AI decide, after `human_of(g, p)` says there is a person; they are never reached in a simulation.
//!
//! Every helper returns `Res`: a person's answer of `Answer::Cancel` (the session was closed or rewound) is
//! `Err(Stop::Cancelled)`, which unwinds the whole game through the engine's `?`.

use super::{Target, human_of, legal};
use crate::cards::Types;
use crate::control::{Answer, Request};
use crate::dsl::model::{Effect, Num, To};
use crate::engine::values::{etgh, no_damage, protected_from, untargetable};
use crate::engine::zones::{self, Zone};
use crate::engine::{self};
use crate::flow::{Res, Stop};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::intern;
use crate::tag::Tag;
use serde_json::{Value, json};

pub const BASICS: [&str; 6] = ["Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes"];

/// 'Sheoldred, the Apocalypse {2}{B}{B}' (Python's `card_label`).
pub fn card_label(g: &Game, c: CardId) -> String {
    let d = g.db.get(c);
    let mut cost = String::new();
    if d.generic > 0 {
        cost.push_str(&format!("{{{}}}", d.generic));
    }
    for x in d.pips.chars() {
        cost.push_str(&format!("{{{x}}}"));
    }
    format!("{} {}", d.name, cost).trim().to_string()
}

/// Where each choice sits on the table, for the browser to click it there (Python's `human.table_refs`): a label
/// naming a permanent -> `{seat, perm}`; a player -> `{player}`; a card in p's hand -> `{hand}`; else null.
pub fn table_refs(g: &Game, p: PlayerId, labels: &[String]) -> Vec<Value> {
    let mut place: Vec<(String, Value)> = vec![];
    let mut put = |k: String, v: Value| {
        if !place.iter().any(|x| x.0 == k) {
            place.push((k, v));
        }
    };
    for q in g.players.iter().filter(|q| q.alive) {
        put(legal::describe_target(g, p, Target::Player(q.id)), json!({ "player": q.key }));
        for (i, &m) in q.perms.iter().enumerate() {
            put(legal::describe_target(g, p, Target::Perm(m)), json!({ "seat": q.key, "perm": i }));
        }
    }
    let mut hand: Vec<(String, usize)> = vec![];
    for (i, &c) in g.player(p).hand.iter().enumerate() {
        let k = card_label(g, c);
        if !hand.iter().any(|x| x.0 == k) {
            hand.push((k, i));
        }
    }
    labels
        .iter()
        .map(|lb| {
            if let Some(r) = place.iter().find(|x| &x.0 == lb) {
                return r.1.clone();
            }
            if let Some(h) = hand.iter().find(|x| &x.0 == lb) {
                return json!({ "hand": h.1 });
            }
            if lb.contains(" (") {
                // 'Archmage Emeritus (Veyran, 2/2) (X = 4)'
                if let Some(r) = place.iter().find(|x| lb.starts_with(&format!("{} ", x.0))) {
                    return r.1.clone();
                }
            }
            Value::Null
        })
        .collect()
}

/// Ask the person to pick one of `labels`; returns its index, or None if they cancel (the last choice, named by
/// `cancel`, e.g. 'no block'). `cancel: None` is a forced choice with no way out (after five bad answers: the
/// first). Python's `human.choose`.
pub fn choose(
    g: &Game,
    p: PlayerId,
    kind: &str,
    prompt: &str,
    labels: &[String],
    cancel: Option<&str>,
    data: Option<Value>,
) -> Res<Option<usize>> {
    let Some(h) = human_of(g, p) else { return Ok(if cancel.is_some() { None } else { Some(0) }) };
    let mut shown: Vec<String> = labels.to_vec();
    if let Some(c) = cancel {
        shown.push(c.to_string());
    }
    let mut data = match data {
        Some(Value::Object(m)) => m,
        _ => serde_json::Map::new(),
    };
    if h.live() {
        let refs = table_refs(g, p, labels);
        if refs.iter().any(|r| !r.is_null()) {
            data.insert("refs".into(), Value::Array(refs));
        }
    }
    let req = Request { kind: intern(kind), prompt: prompt.to_string(), choices: shown, data: Value::Object(data) };
    for _ in 0..5 {
        match h.ask(g, p, req.clone()) {
            Answer::Cancel => return Err(Stop::Cancelled),
            Answer::Decline if cancel.is_some() => return Ok(None),
            Answer::Pick(i) if cancel.is_some() && i == labels.len() => return Ok(None),
            Answer::Pick(i) if i < labels.len() => return Ok(Some(i)),
            _ => {}
        }
        h.tell(
            p,
            "invalid",
            if cancel.is_some() { "Pick one of the numbers, or cancel." } else { "Pick one of the numbers." },
        );
    }
    Ok(if cancel.is_some() { None } else { Some(0) })
}

/// `choose` for a forced choice (no way out): the index.
pub fn choose_one(g: &Game, p: PlayerId, kind: &str, prompt: &str, labels: &[String]) -> Res<usize> {
    Ok(choose(g, p, kind, prompt, labels, None, None)?.unwrap_or(0))
}

/// The person picks n of `cards` (fewer if there are fewer); no cancelling. Returns the picked cards.
pub fn pick_cards(g: &Game, p: PlayerId, cards: &[CardId], n: usize, prompt: &str, kind: &str) -> Res<Vec<CardId>> {
    let mut left: Vec<CardId> = cards.to_vec();
    let mut picked = vec![];
    let total = n.min(cards.len());
    for i in 0..total {
        let labels: Vec<String> = left.iter().map(|&c| card_label(g, c)).collect();
        let k = choose_one(g, p, kind, &format!("{prompt} ({} of {total})", i + 1), &labels)?;
        picked.push(left.remove(k));
    }
    Ok(picked)
}

// ------------------------------------------------------------------ mulligan
/// London mulligan with a free first mulligan (multiplayer Commander): draw seven, keep or shuffle back and draw
/// seven again; on keeping after k mulligans, put k - 1 cards on the bottom.
pub fn mulligan(g: &mut Game, p: PlayerId, rng: &mut crate::rng::Rng) -> Res {
    let Some(h) = human_of(g, p) else { return Ok(()) };
    let mut k = 0usize;
    loop {
        {
            let pl = g.player_mut(p);
            let hand = std::mem::take(&mut pl.hand);
            pl.library.extend(hand);
            rng.shuffle(&mut pl.library);
            for _ in 0..7 {
                let c = pl.library.pop().unwrap();
                pl.hand.push(c);
            }
        }
        let bottom = k.saturating_sub(1);
        let note =
            if bottom == 0 { String::new() } else { format!(" If you keep, you put {bottom} card(s) on the bottom.") };
        if k >= 6 {
            break;
        }
        let hand = g.player(p).hand.clone();
        let req = Request {
            kind: "mulligan",
            prompt: format!("Your opening hand ({} to keep).{note}", 7 - bottom),
            choices: vec!["keep".into(), "mulligan".into()],
            data: json!({
                "hand": hand.iter().map(|&c| card_label(g, c)).collect::<Vec<_>>(),
                "names": hand.iter().map(|&c| g.db.get(c).name.to_string()).collect::<Vec<_>>(),
            }),
        };
        match h.ask(g, p, req) {
            Answer::Cancel => return Err(Stop::Cancelled),
            Answer::Pick(0) | Answer::Yes => break,
            _ => k += 1,
        }
    }
    let bottom = k.saturating_sub(1);
    if bottom > 0 {
        let hand = g.player(p).hand.clone();
        for c in pick_cards(g, p, &hand, bottom, "Put a card on the bottom of your library", "choose")? {
            let pl = g.player_mut(p);
            let i = pl.hand.iter().position(|&x| x == c).unwrap();
            pl.hand.remove(i);
            pl.library.insert(0, c);
        }
    }
    g.player_mut(p).stats.insert("mulls", k as i64);
    let (n, name) = (g.player(p).hand.len(), g.player(p).name);
    if k > 0 {
        crate::glog!(g, "{} keeps {} cards after {} mulligan(s)", name, n, k);
    } else {
        crate::glog!(g, "{} keeps {} cards", name, n);
    }
    Ok(())
}

// ------------------------------------------------------------------ discards and sacrifices
/// p discards n cards of their choice.
pub fn discard(g: &mut Game, p: PlayerId, n: usize, why: &str) -> Res {
    let hand = g.player(p).hand.clone();
    let cards = pick_cards(g, p, &hand, n, why, "choose")?;
    if !cards.is_empty() {
        zones::discard_cards(g, p, &cards)?;
    }
    Ok(())
}

pub fn discard_to_hand_size(g: &mut Game, p: PlayerId, size: usize) -> Res {
    let over = g.player(p).hand.len().saturating_sub(size);
    if over > 0 {
        discard(g, p, over, &format!("End of turn: discard down to {size}"))?;
    }
    Ok(())
}

/// q sacrifices one of `cands` (their creatures).
pub fn sacrifice_creature(g: &mut Game, q: PlayerId, cands: &[PermId], why: &str) -> Res {
    if cands.is_empty() {
        return Ok(());
    }
    let labels: Vec<String> = cands.iter().map(|&m| legal::describe_target(g, q, Target::Perm(m))).collect();
    let k = choose_one(g, q, "choose", why, &labels)?;
    zones::die(g, cands[k], "sac")
}

// ------------------------------------------------------------------ searching the library
/// p searches their library for up to k cards matching `pred`; returns them (taken out of the library, not yet put
/// anywhere). Then the library is shuffled.
pub fn search(
    g: &mut Game,
    p: PlayerId,
    pred: &dyn Fn(&Game, CardId) -> bool,
    k: usize,
    prompt: &str,
    allow_none: bool,
) -> Res<Vec<CardId>> {
    let mut pool: Vec<CardId> = vec![];
    for c in zones::searchable(g, p) {
        if pred(g, c) && !pool.iter().any(|&x| g.db.get(x).name == g.db.get(c).name) {
            pool.push(c);
        }
    }
    pool.sort_by(|&a, &b| g.db.get(a).name.cmp(&g.db.get(b).name));
    let mut got = vec![];
    for i in 0..k {
        if pool.is_empty() {
            break;
        }
        let labels: Vec<String> = pool.iter().map(|&c| card_label(g, c)).collect();
        let pr = if k > 1 { format!("{prompt} ({} of {k})", i + 1) } else { prompt.to_string() };
        let j = choose(g, p, "search", &pr, &labels, if allow_none { Some("find nothing") } else { None }, None)?;
        let Some(j) = j else { break };
        let c = pool.remove(j);
        let lib = &mut g.player_mut(p).library;
        let at = lib.iter().position(|&x| x == c).unwrap();
        lib.remove(at);
        got.push(c);
    }
    shuffle_library(g, p);
    Ok(got)
}

/// `g.rng.shuffle(p.library)`
pub fn shuffle_library(g: &mut Game, p: PlayerId) {
    let mut lib = std::mem::take(&mut g.player_mut(p).library);
    g.rng.shuffle(&mut lib);
    g.player_mut(p).library = lib;
}

/// One basic land card of p's choice (by name), taken out of the library, or None.
pub fn pick_basic(g: &mut Game, p: PlayerId, prompt: &str) -> Res<Option<CardId>> {
    let cands = zones::searchable(g, p);
    let mut names: Vec<&str> = vec![];
    for &c in &cands {
        let d = g.db.get(c);
        if d.land && BASICS.contains(&&*d.name) && !names.contains(&&*d.name) {
            names.push(&d.name);
        }
    }
    names.sort_unstable();
    if names.is_empty() {
        return Ok(None);
    }
    let labels: Vec<String> = names.iter().map(|s| s.to_string()).collect();
    let Some(j) = choose(g, p, "search", prompt, &labels, Some("find nothing"), None)? else { return Ok(None) };
    let c = cands.iter().copied().find(|&c| *g.db.get(c).name == *labels[j]).unwrap();
    let lib = &mut g.player_mut(p).library;
    let at = lib.iter().position(|&x| x == c).unwrap();
    lib.remove(at);
    Ok(Some(c))
}

// ------------------------------------------------------------------ the top of the library
/// Scry n (`to_gy` false: the rest on the bottom) or surveil n (`to_gy` true): each card on top or away, in the
/// order kept.
pub fn scry(g: &mut Game, p: PlayerId, n: usize, to_gy: bool) -> Res {
    let n = n.min(g.player(p).library.len());
    if n == 0 {
        return Ok(());
    }
    let top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect(); // top card first
    let away_word = if to_gy { "your graveyard" } else { "the bottom" };
    let (mut keep, mut away) = (vec![], vec![]);
    for &c in &top {
        let labels = vec!["keep on top".to_string(), format!("put on {away_word}")];
        let prompt = format!("{}: {}", if to_gy { "Surveil" } else { "Scry" }, card_label(g, c));
        let k = choose_one(g, p, "choose", &prompt, &labels)?;
        if k == 0 { keep.push(c) } else { away.push(c) }
    }
    let pl = g.player_mut(p);
    if to_gy {
        pl.gy.extend(away);
    } else {
        pl.library.splice(0..0, away);
    }
    for &c in keep.iter().rev() {
        pl.library.push(c); // kept cards stay in the order they were
    }
    let name = pl.name;
    crate::glog!(g, "  {} {} {}: {} on top", name, if to_gy { "surveils" } else { "scries" }, n, keep.len());
    Ok(())
}

// ------------------------------------------------------------------ modes and "you may"
fn effect_word(k: &str) -> Option<&'static str> {
    Some(match k {
        "regrow" => "return a card from your graveyard to your hand",
        "discard" => "a player discards",
        "destroy" => "destroy",
        "exile" => "exile",
        "damage" => "deal damage",
        "draw" => "draw cards",
        "bounce" => "return a permanent to its owner's hand",
        "tap" => "tap",
        "untap" => "untap",
        "counter_spell" => "counter a spell",
        "token" => "create tokens",
        "grant" => "give an ability",
        "sacrifice" => "a player sacrifices",
        "gain_life" => "gain life",
        "lose_life" => "lose life",
        "counters" => "put counters",
        "pump" => "pump",
        "mill" => "mill",
        "search" => "search your library",
        "reanimate" => "return a creature to the battlefield",
        _ => return None,
    })
}

/// 'deal damage 2' / 'destroy (artifact)': a short label for one mode (a list of effects).
pub fn describe_mode(ms: &[Effect]) -> String {
    let mut out = vec![];
    for e in ms {
        let mut w = match effect_word(&e.kind) {
            Some(s) => s.to_string(),
            None => e.kind.replace('_', " "),
        };
        if let Some(Num::Int(n)) = e.n
            && (n > 1 || e.kind == "damage")
        {
            w += &format!(" {n}");
        }
        let filter = match (&e.what, &e.to) {
            (Some(s), _) => s.filter.as_ref(),
            (None, Some(To::Sel(s))) => s.filter.as_ref(),
            _ => None,
        };
        if let Some(t) = filter.and_then(|f| f.kind.as_ref()).filter(|t| !t.is_empty()) {
            w += &format!(" ({})", t.replace('_', " "));
        }
        out.push(w);
    }
    out.join(", then ")
}

/// The person chooses k different modes; returns their indexes in `modes`, in the order picked.
pub fn pick_modes(g: &Game, p: PlayerId, modes: &[Vec<Effect>], k: usize, name: &str) -> Res<Vec<usize>> {
    let mut left: Vec<usize> = (0..modes.len()).collect();
    let mut got = vec![];
    for i in 0..k {
        if left.is_empty() {
            break;
        }
        let labels: Vec<String> = left.iter().map(|&x| describe_mode(&modes[x])).collect();
        let prompt =
            if k > 1 { format!("{name}: choose a mode ({} of {k})", i + 1) } else { format!("{name}: choose a mode") };
        let j = choose_one(g, p, "choose", &prompt, &labels)?;
        got.push(left.remove(j));
    }
    Ok(got)
}

pub fn yes_no(g: &Game, p: PlayerId, prompt: &str) -> Res<bool> {
    Ok(choose_one(g, p, "choose", prompt, &["yes".to_string(), "no".to_string()])? == 0)
}

// ------------------------------------------------------------------ "deals N damage to any target"
/// Everything a damage ability may hit: creatures and planeswalkers (hexproof, shroud, protection respected) and
/// players (not under protection from everything).
pub fn damage_targets(g: &Game, p: PlayerId, colours: crate::cards::Colors) -> Vec<Target> {
    let mut out = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &m in &q.perms {
            let x = g.perm(m);
            let pw = x.cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER));
            if x.phased || !(g.is_creature(m) || pw) {
                continue;
            }
            if (x.owner != p && untargetable(g, m)) || (!colours.is_empty() && protected_from(g, m, colours)) {
                continue;
            }
            out.push(Target::Perm(m));
        }
    }
    out.extend(g.players.iter().filter(|q| legal::player_targetable(g, q.id)).map(|q| Target::Player(q.id)));
    out
}

/// p's `source` deals n damage to a target p picks: a creature dies if n reaches its toughness, a planeswalker
/// loses that much loyalty, a player loses that much life.
pub fn deal_damage(g: &mut Game, p: PlayerId, n: i32, source: &str, kind: &str) -> Res {
    if n <= 0 {
        return Ok(());
    }
    let tg = damage_targets(g, p, Default::default());
    if tg.is_empty() {
        return Ok(());
    }
    let labels: Vec<String> = tg.iter().map(|&x| legal::describe_target(g, p, x)).collect();
    let k = choose_one(g, p, "target", &format!("{source}: deal {n} damage to which target?"), &labels)?;
    let m = match tg[k] {
        Target::Player(q) => return engine::life::lose_life(g, q, n, Some(p), intern(kind), None),
        Target::Perm(m) => m,
    };
    let name = g.perm(m).name;
    if g.is_creature(m) && no_damage(g, m) {
        crate::glog!(g, "    the damage to {} is prevented", name);
    } else if g.is_creature(m) && etgh(g, m) <= n {
        engine::removal::apply_removal(g, Some(p), m, &format!("dmg{n}"), None)?;
    } else if g.is_creature(m) {
        crate::glog!(g, "    {} deals {} damage to {} (it survives)", source, n, name);
    }
    let x = g.perm(m);
    let pw = x.cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER));
    if x.on_bf && pw && x.loyalty.is_some() {
        let l = x.loyalty.unwrap() - n;
        g.perm_mut(m).loyalty = Some(l);
        crate::glog!(g, "    {} deals {} damage to {} (loyalty {})", source, n, name, l.max(0));
        if l <= 0 {
            zones::leave(g, m)?;
            zones::to_zone_card(g, m, Zone::Gy);
        }
    }
    engine::life::check_state(g)
}

// ------------------------------------------------------------------ the Ring
pub fn ring_bearer(g: &Game, p: PlayerId, cands: &[PermId]) -> Res<PermId> {
    let labels: Vec<String> = cands.iter().map(|&m| legal::describe_target(g, p, Target::Perm(m))).collect();
    let k = choose_one(g, p, "choose", "The Ring tempts you: choose your Ring-bearer", &labels)?;
    Ok(cands[k])
}

// ------------------------------------------------------------------ creatures that remove something when they enter
/// Noxious Gearhulk and similar: "when this enters, you may destroy another target creature" (pick, or none).
pub fn etb_removal(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let Some(cd) = g.perm(m).cd else { return Ok(()) };
    let tg: Vec<PermId> = legal::spell_targets(g, p, cd)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|x| match x {
            Target::Perm(x) if x != m => Some(x),
            _ => None,
        })
        .collect();
    if tg.is_empty() {
        return Ok(());
    }
    let labels: Vec<String> = tg.iter().map(|&x| legal::describe_target(g, p, Target::Perm(x))).collect();
    let name = g.perm(m).name;
    let Some(k) = choose(g, p, "target", &format!("{name} enters: choose a target"), &labels, Some("no target"), None)?
    else {
        return Ok(());
    };
    let rem = g.db.get(cd).tags.str(Tag::Rem).unwrap_or("").to_string();
    engine::removal::apply_removal(g, Some(p), tg[k], &rem, Some(cd))
}

// ------------------------------------------------------------------ taxes: "unless that player pays {N}"
/// An opponent's tax (Rhystic Study, Smothering Tithe, Mystic Remora): ask p whether to pay {n}. Paid from p's
/// floating mana first, then by tapping p's sources (as you would at the table). Returns true if paid.
pub fn pay_tax(g: &mut Game, p: PlayerId, n: u32, what: &str) -> Res<bool> {
    super::mana::pay_tax(g, p, n, what)
}

// ------------------------------------------------------------------ reanimation and filling the graveyard
/// Animate Dead, Necromancy, Reanimate: any graveyard.
pub const ANY_GRAVEYARD: [&str; 3] = ["animate", "necro", "reanimate"];

/// [(creature card, graveyard owner)] a reanimation spell of this kind may return.
pub fn rean_candidates(g: &Game, p: PlayerId, kind: &str) -> Vec<(CardId, PlayerId)> {
    let owners: Vec<PlayerId> = if ANY_GRAVEYARD.contains(&kind) {
        g.players.iter().filter(|q| q.alive).map(|q| q.id).collect()
    } else {
        vec![p]
    };
    owners
        .iter()
        .flat_map(|&q| g.player(q).gy.iter().filter(|&&c| g.db.get(c).creature).map(move |&c| (c, q)))
        .collect()
}

/// The target of reanimation spell c: (card, owner), or None if cancelled / nothing to return.
pub fn choose_rean(g: &Game, p: PlayerId, c: CardId) -> Res<Option<(CardId, PlayerId)>> {
    let d = g.db.get(c);
    let cands = rean_candidates(g, p, d.tags.str(Tag::Rean).unwrap_or("animate"));
    if cands.is_empty() {
        return Ok(None);
    }
    let labels: Vec<String> = cands
        .iter()
        .map(|&(x, q)| {
            let whose = if q == p { "your graveyard".to_string() } else { format!("{}'s graveyard", g.player(q).name) };
            format!("{} ({whose})", card_label(g, x))
        })
        .collect();
    let k = choose(g, p, "target", &format!("{}: return which creature card?", d.name), &labels, Some("cancel"), None)?;
    Ok(k.map(|k| cands[k]))
}

/// Entomb, Buried Alive, Unmarked Grave, Grisly Salvage, Deadly Dispute: the person picks what goes where.
pub fn fill(g: &mut Game, p: PlayerId, kind: &str) -> Res {
    match kind {
        "entomb" => {
            for c in search(g, p, &|_, _| true, 1, "Entomb: put a card from your library into your graveyard", true)? {
                g.player_mut(p).gy.push(c);
            }
        }
        "buried" => {
            let pr = "Buried Alive: up to three creature cards into your graveyard";
            for c in search(g, p, &|g, c| g.db.get(c).creature, 3, pr, true)? {
                g.player_mut(p).gy.push(c);
            }
        }
        "unmarked" => {
            let pr = "Unmarked Grave: a nonlegendary card into your graveyard";
            for c in search(g, p, &|g, c| !g.db.get(c).tags.has(Tag::Leg), 1, pr, true)? {
                g.player_mut(p).gy.push(c);
            }
        }
        "grisly" => {
            let n = g.player(p).library.len().min(5);
            let mut top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
            let ok: Vec<CardId> = top.iter().copied().filter(|&c| g.db.get(c).creature || g.db.get(c).land).collect();
            if !ok.is_empty() {
                let labels: Vec<String> = ok.iter().map(|&c| card_label(g, c)).collect();
                let pr = "Grisly Salvage: put a creature or land card into your hand?";
                if let Some(k) = choose(g, p, "choose", pr, &labels, Some("none"), None)? {
                    let at = top.iter().position(|&x| x == ok[k]).unwrap();
                    top.remove(at);
                    g.player_mut(p).hand.push(ok[k]);
                }
            }
            g.player_mut(p).gy.extend(top);
            let name = g.player(p).name;
            crate::glog!(g, "  {} reveals five with Grisly Salvage", name);
        }
        "dispute" => zones::add_treasure(g, p, 1)?,
        _ => {}
    }
    Ok(())
}

/// An additional cost: sacrifice an artifact or creature (a Treasure counts). False if p has none.
pub fn sac_artifact_or_creature(g: &mut Game, p: PlayerId, what: &str) -> Res<bool> {
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            (g.is_creature(m) || x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT))) && !x.phased
        })
        .collect();
    let mut labels: Vec<String> = cands.iter().map(|&m| legal::describe_target(g, p, Target::Perm(m))).collect();
    if g.player(p).treasures > 0 {
        labels.push("a Treasure token".into());
    }
    if labels.is_empty() {
        return Ok(false);
    }
    let k = choose_one(g, p, "choose", &format!("{what}: sacrifice an artifact or creature"), &labels)?;
    if k == cands.len() {
        g.player_mut(p).treasures -= 1;
        let name = g.player(p).name;
        crate::glog!(g, "  {} sacrifices a Treasure", name);
    } else {
        zones::die(g, cands[k], "sac")?;
    }
    Ok(true)
}

/// p picks a target opponent (Archon of Cruelty).
pub fn target_opponent(g: &Game, p: PlayerId, source: &str) -> Res<PlayerId> {
    let all: Vec<PlayerId> = g.opps(p).collect();
    let t: Vec<PlayerId> = all.iter().copied().filter(|&q| legal::player_targetable(g, q)).collect();
    let opps = if t.is_empty() { all } else { t };
    let labels: Vec<String> = opps.iter().map(|&q| legal::describe_target(g, p, Target::Player(q))).collect();
    let k = choose_one(g, p, "target", &format!("{source}: target opponent?"), &labels)?;
    Ok(opps[k])
}

/// Alania, Divergent Storm: copy spell c by having a target opponent draw a card? That opponent, or None.
pub fn alania(g: &Game, p: PlayerId, c: CardId, opps: &[PlayerId]) -> Res<Option<PlayerId>> {
    let labels: Vec<String> = opps.iter().map(|&q| legal::describe_target(g, p, Target::Player(q))).collect();
    let pr = format!("Alania: have an opponent draw a card to copy {}?", g.db.get(c).name);
    Ok(choose(g, p, "target", &pr, &labels, Some("no copy"), None)?.map(|k| opps[k]))
}

// ------------------------------------------------------------------ copies of spells
/// A copy of spell c (Thousand-Year Storm, Jin-Gitaxias, Ral, Return the Favor, Mizzix's Mastery): you may choose
/// new targets for it. Returns the copy's ctx.
pub fn copy_targets(g: &Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Ctx> {
    let mut ctx = ctx.clone();
    let old = ctx.target.map(Target::Perm).or(ctx.face.map(Target::Player));
    let Some(tg) = legal::spell_targets(g, p, c) else { return Ok(ctx) };
    if tg.is_empty() {
        return Ok(ctx);
    }
    let keep = old.is_some_and(|o| tg.contains(&o));
    let labels: Vec<String> = tg.iter().map(|&x| legal::describe_target(g, p, x)).collect();
    let cancel = if keep {
        Some(format!("keep the same target ({})", legal::describe_target(g, p, old.unwrap())))
    } else {
        None
    };
    let pr = format!("The copy of {}: choose its target", g.db.get(c).name);
    let Some(k) = choose(g, p, "target", &pr, &labels, cancel.as_deref(), None)? else { return Ok(ctx) };
    ctx.target = None;
    ctx.face = None;
    match tg[k] {
        Target::Player(q) => ctx.face = Some(q),
        Target::Perm(m) => ctx.target = Some(m),
    }
    Ok(ctx)
}

// ------------------------------------------------------------------ Atraxa, Grand Unifier (Sephiroth, the Savior)
pub const TYPE_NAMES: [(char, &str); 8] = [
    ('A', "artifact"),
    ('B', "battle"),
    ('C', "creature"),
    ('E', "enchantment"),
    ('I', "instant"),
    ('L', "land"),
    ('P', "planeswalker"),
    ('S', "sorcery"),
];

fn type_of(ch: char) -> Types {
    match ch {
        'A' => Types::ARTIFACT,
        'C' => Types::CREATURE,
        'E' => Types::ENCHANTMENT,
        'I' => Types::INSTANT,
        'L' => Types::LAND,
        'P' => Types::PLANESWALKER,
        'S' => Types::SORCERY,
        _ => Types::default(), // battles: no card has the type here
    }
}

/// Atraxa's enter trigger: the top ten revealed; for each card type you may put a card of that type into your hand
/// (a card with two types fills one of them). Returns the cards taken.
pub fn atraxa_pick(g: &mut Game, p: PlayerId, top: &[CardId]) -> Res<Vec<CardId>> {
    let names: Vec<String> = top.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    let pname = g.player(p).name;
    crate::glog!(g, "    {} reveals the top {}: {}", pname, top.len(), names.join(", "));
    let mut taken: Vec<CardId> = vec![];
    for (t, word) in TYPE_NAMES {
        let ty = type_of(t);
        let cands: Vec<CardId> = top
            .iter()
            .copied()
            .filter(|&c| g.db.get(c).types.has(ty) && ty != Types::default() && !taken.contains(&c))
            .collect();
        if cands.is_empty() {
            continue;
        }
        let labels: Vec<String> = cands.iter().map(|&c| card_label(g, c)).collect();
        let art = if "aei".contains(word.chars().next().unwrap()) { "an" } else { "a" };
        let pr = format!("Atraxa: put {art} {word} card into your hand? (revealed: {})", names.join(", "));
        if let Some(k) = choose(g, p, "choose", &pr, &labels, Some(&format!("no {word}")), None)? {
            taken.push(cands[k]);
        }
    }
    Ok(taken)
}
