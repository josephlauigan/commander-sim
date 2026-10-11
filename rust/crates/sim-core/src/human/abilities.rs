//! Python's play/abilities.py. Ported by the practice-cards task.
//! Activated abilities of your decks' cards that the engine keeps in deck AI code rather than in card hooks (the AI
//! uses them through its own plans). For the human seat they are abilities like any other: listed on the permanent or
//! land, checked for timing, tapping and mana, then carried out. Each effect returns None or the reason it can't.
//!
//! Python lists `(label, fn)`; here an `Ability<T>` (T is the `PermId` or `LandId` the ability belongs to). `act`
//! returns `Ok(None)` when it was carried out (or the person cancelled) and `Ok(Some(why))` when it can't be done.

use super::{Target, choices, legal, mana};
use crate::engine::{self, stack, zones};
use crate::flow::Res;
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{DataKey, Game, Step, Val};
use crate::tag::Tag;

/// the reason an activation can't be done, or None (done, or cancelled)
pub type Why = Option<String>;
pub type Act<T> = Box<dyn Fn(&mut Game, PlayerId, T) -> Res<Why>>;

/// one activated ability the person can use: its label in the list, and what activating it does
pub struct Ability<T> {
    pub label: String,
    pub act: Act<T>,
}

pub type PermAbility = Ability<PermId>;
pub type LandAbility = Ability<LandId>;

pub fn ability<T>(label: impl Into<String>, f: impl Fn(&mut Game, PlayerId, T) -> Res<Why> + 'static) -> Ability<T> {
    Ability { label: label.into(), act: Box::new(f) }
}

fn name(g: &Game, c: CardId) -> String {
    g.db.get(c).name.to_string()
}

fn tap_ok(g: &Game, m: PermId, creature: bool) -> Why {
    let x = g.perm(m);
    if x.tapped {
        return Some(format!("{} is tapped.", x.name));
    }
    if creature && x.sick {
        return Some(format!("{} has summoning sickness (it came under your control this turn).", x.name));
    }
    None
}

fn land_tap_ok(g: &Game, l: LandId) -> Why {
    if g.land(l).tapped { Some(format!("{} is tapped.", name(g, g.land(l).cd))) } else { None }
}

fn pay(g: &mut Game, p: PlayerId, gn: u32, pips: &str, what: &str) -> Why {
    mana::pay_from_pool(g, p, gn, pips).map(|why| format!("Can't activate {what}. {why}"))
}

// ------------------------------------------------------------------ permanents
/// abilities.archivist: Jace's Archivist: {U}, {T}: each player discards their hand, then draws as many as the most
/// discarded
fn archivist(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, true).or_else(|| pay(g, p, 0, "U", "Jace's Archivist")) {
        return Ok(Some(why));
    }
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "  {} activates Jace's Archivist", g.player(p).name);
    if !stack::ability_window(g, p, Some(m), "each player discards their hand and draws", None, None)? {
        return Ok(None);
    }
    let n = g.players.iter().filter(|q| q.alive).map(|q| q.hand.len()).max().unwrap_or(0);
    crate::glog!(g, "    everyone discards and draws {}", n);
    let ids: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    for &q in &ids {
        if g.player(q).alive {
            let hand = g.player(q).hand.clone();
            zones::discard_cards(g, q, &hand)?;
        }
    }
    for &q in &ids {
        if g.player(q).alive {
            zones::draw(g, q, n as u32, false)?;
        }
    }
    engine::life::check_state(g)?;
    Ok(None)
}

/// abilities.assault: Aggravated Assault: {3}{R}{R}: untap your creatures; after this main phase, an additional
/// combat (followed by a main phase). Sorcery speed
fn assault(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = legal::sorcery_timing(g, p) {
        return Ok(Some(why.replace("do that", "activate Aggravated Assault")));
    }
    if let Some(why) = pay(g, p, 3, "RR", "Aggravated Assault") {
        return Ok(Some(why));
    }
    if !stack::ability_window(g, p, Some(m), "untap, an additional combat", Some(7.0), None)? {
        return Ok(None);
    }
    for x in g.player(p).perms.clone() {
        if g.is_creature(x) {
            g.perm_mut(x).tapped = false;
        }
    }
    crate::glog!(g, "  {} activates Aggravated Assault: creatures untap, an extra combat follows", g.player(p).name);
    if g.step == Step::Main1 {
        g.player_mut(p).extra_combats += 1; // it comes right after this turn's combat
    } else {
        engine::combat::combat(g, p)?; // main 2: the extra combat now, then this main phase continues
    }
    Ok(None)
}

/// abilities.tortured: Tortured Existence: {B}, discard a creature card: return target creature card from your
/// graveyard to your hand
fn tortured(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let tg: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    let fodder: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    if tg.is_empty() {
        return Ok(Some("There is no creature card in your graveyard to return.".into()));
    }
    if fodder.is_empty() {
        return Ok(Some("You have no creature card in hand to discard.".into()));
    }
    if let Some(why) = mana::cost_problem(g, p, 0, "B") {
        return Ok(Some(format!("Can't activate Tortured Existence. {why}")));
    }
    let labels: Vec<String> = tg.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) = choices::choose(
        g,
        p,
        "target",
        "Tortured Existence: return which creature card?",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(None); // targets first, then the costs
    };
    mana::pay_from_pool(g, p, 0, "B");
    let d = choices::pick_cards(g, p, &fodder, 1, "Tortured Existence: discard a creature card", "choose")?[0];
    zones::discard_cards(g, p, &[d])?;
    let c = tg[k];
    if !stack::ability_window(g, p, Some(m), &format!("return {} to hand", name(g, c)), None, None)? {
        return Ok(None);
    }
    if let Some(i) = g.player(p).gy.iter().position(|&x| x == c) {
        g.player_mut(p).gy.remove(i);
        g.player_mut(p).hand.push(c);
    }
    crate::glog!(g, "  {} uses Tortured Existence: {} for {}", g.player(p).name, name(g, d), name(g, c));
    Ok(None)
}

/// abilities.loop_abilities: Sephiroth's infinite loops as a table shortcut: on each piece of a loop that is
/// assembled, 'start the loop' (the engine plays it out as its end result, after a window for opponents to answer a
/// key piece)
fn loop_abilities(g: &Game, p: PlayerId, m: PermId) -> Vec<PermAbility> {
    use crate::impls::mine;
    let mut out = vec![];
    let Some(cd) = g.perm(m).cd else { return out };
    let nm = name(g, cd);
    if &*g.player(p).key != "seph" || !mine::loop_card(&nm) {
        return out;
    }
    for (i, keys, kills) in mine::seph_loops(g, p) {
        let l = &mine::LOOPS[i];
        if !l.groups.iter().any(|grp| grp.contains(&nm.as_str())) && !mine::OUTLETS.contains(&nm.as_str()) {
            continue;
        }
        let label =
            format!("start the loop: {}{}", l.name, if kills { "" } else { " (no payoff on the battlefield yet)" });
        out.push(ability(label, move |g: &mut Game, p: PlayerId, _m: PermId| {
            mine::run_loop(g, p, i, &keys, kills)?;
            Ok(None)
        }));
    }
    out
}

/// abilities.permanent_abilities: the abilities of permanent m
pub fn permanent_abilities(g: &Game, p: PlayerId, m: PermId) -> Vec<PermAbility> {
    let Some(cd) = g.perm(m).cd else { return vec![] };
    let d = g.db.get(cd);
    let mut out = vec![];
    if d.tag(Tag::Archivist) {
        out.push(ability("{U}, {T}: each player discards their hand and draws that many", archivist));
    }
    if d.tag(Tag::Assault) {
        out.push(ability("{3}{R}{R}: untap your creatures; an additional combat (sorcery speed)", assault));
    }
    if d.tags.str(Tag::Fill) == Some("tortured") {
        out.push(ability(
            "{B}, discard a creature card: return a creature card from your graveyard to your hand",
            tortured,
        ));
    }
    out.extend(loop_abilities(g, p, m));
    out
}

// ------------------------------------------------------------------ lands
/// abilities.passage: Rogue's Passage: {4}, {T}: target creature can't be blocked this turn
fn passage(g: &mut Game, p: PlayerId, l: LandId) -> Res<Why> {
    if let Some(why) = land_tap_ok(g, l) {
        return Ok(Some(why));
    }
    if let Some(why) = mana::cost_problem(g, p, 4, "") {
        return Ok(Some(format!("Can't activate Rogue's Passage. {why}")));
    }
    let cre: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| {
            g.is_creature(m) && !g.perm(m).phased && !(g.perm(m).owner != p && engine::values::untargetable(g, m))
        })
        .collect();
    if cre.is_empty() {
        return Ok(Some("There is no creature to target.".into()));
    }
    let labels: Vec<String> = cre.iter().map(|&x| legal::describe_target(g, p, Target::Perm(x))).collect();
    let Some(k) = choices::choose(
        g,
        p,
        "target",
        "Rogue's Passage: which creature can't be blocked this turn?",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(None);
    };
    mana::pay_from_pool(g, p, 4, "");
    g.land_mut(l).tapped = true;
    let m = cre[k];
    crate::glog!(g, "  {} activates Rogue's Passage: {} can't be blocked this turn", g.player(p).name, g.perm(m).name);
    let cd = g.land(l).cd;
    if stack::ability_window_card(g, p, cd, &format!("{} can't be blocked", g.perm(m).name), None, Some(m))?
        && g.perm(m).on_bf
    {
        let st = g.turn_stamp();
        g.perm_mut(m).data.set(DataKey::Unbl, Val::Stamp(st));
    }
    Ok(None)
}

/// abilities.grounds: Scavenger Grounds: {2}, {T}, sacrifice a Desert: exile all graveyards
fn grounds(g: &mut Game, p: PlayerId, l: LandId) -> Res<Why> {
    if let Some(why) = land_tap_ok(g, l) {
        return Ok(Some(why));
    }
    let deserts: Vec<LandId> =
        g.player(p).lands.iter().copied().filter(|&x| g.db.get(g.land(x).cd).tag(Tag::Desert)).collect();
    if let Some(why) = pay(g, p, 2, "", "Scavenger Grounds") {
        return Ok(Some(why));
    }
    g.land_mut(l).tapped = true;
    let d = if deserts.contains(&l) { l } else { deserts[0] }; // it is a Desert itself: sacrifice it (the usual choice)
    engine::turn::remove_land(g, p, d);
    let dcd = g.land(d).cd;
    g.player_mut(p).gy.push(dcd);
    crate::glog!(g, "  {} activates Scavenger Grounds", g.player(p).name);
    let cd = g.land(l).cd;
    if !stack::ability_window_card(g, p, cd, "exile all graveyards", Some(9.0), None)? {
        return Ok(None);
    }
    for q in g.players.iter().filter(|q| q.alive).map(|q| q.id).collect::<Vec<_>>() {
        let gy = std::mem::take(&mut g.player_mut(q).gy);
        g.player_mut(q).exile.extend(gy);
    }
    crate::glog!(g, "    all graveyards are exiled");
    Ok(None)
}

/// abilities.land_abilities: the abilities of land l (cards.py's first, then the Passage / Scavenger Grounds ones)
pub fn land_abilities(g: &Game, p: PlayerId, l: LandId) -> Vec<LandAbility> {
    let mut out = super::cards::land_abilities(g, p, l);
    let d = g.db.get(g.land(l).cd);
    if d.tag(Tag::Passage) {
        out.push(ability("{4}, {T}: target creature can't be blocked this turn", passage));
    }
    if d.tag(Tag::Desert) && &*d.name == "Scavenger Grounds" {
        out.push(ability("{2}, {T}, sacrifice a Desert: exile all graveyards", grounds));
    }
    out
}
