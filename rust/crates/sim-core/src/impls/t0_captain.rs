//! Tier 0 pool deck card code: the cards of the Captain America precon (decklists/pool/t0-b2-precons/) that weren't
//! modeled. Filled in by the t0-captain agent (rust/agents/archive/t0-captain.md); the verdicts are in
//! data/card_overrides.toml.
//!
//! Cards modeled here:
//! - **Captain America, Team Leader**: another Hero entering gets vigilance, haste and a counter, so does he (`cap_team`).
//! - **Captain America, Living Legend**: the first time a creature of yours becomes tapped on your turn it untaps.
//!   `became_tapped` is called where creatures tap: the attack declaration, mana abilities, the ability language's tap
//!   costs, crewing and Hawkeye's ping.
//! - **Avengers Quinjet**: Crew 3, flying 4/4, the enter/attack choice (`quinjet_trigger`).
//! - **Folk Hero**, **Jarvis**, **Director Nick Fury**, **Herald's Horn**: cast and cost hooks on Hero spells.
//! - **Gift of Immortality**: an Aura whose host returns under its owner's control; the Aura follows at the next end step.
//! - **Love on the Battlefield**, **Ant-Man**, **Hercules**, **She-Hulk**, **Winter Soldier**, **Shang-Chi**,
//!   **Captain Marvel**, **Hawkeye**: see each function.
//! - **Quicksilver / Captain Mar-Vell**: "cast spells as though they had flash" is `as_flash`; the AI reads it in the
//!   end-of-turn window for creature spells.
//! - **Avenge / Heroic Return**: their cost reductions read who attacked whom (`T0c::attacked`).
//! - **Dismantling Wave** (hard cycling), **Raise the Palisade**, **Heroic Sacrifice**, **Avengers Tower**.

use crate::engine::repl;
use crate::engine::mana::restr_ok;
use crate::engine::pool::Restr;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, ability_window_card, trigger_window};
use crate::engine::values::{card_name, epow, etgh, has_type, once_per_turn, pval, subtypes, untargetable};
use crate::engine::zones::{Enter, bounce, die, draw, enter, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{AURA, AuraSpec};
use crate::state::{Ctx, DataKey, Game, Step, TurnStamp, Val};
use crate::sym::{Sym, intern};

const CAP_TL: &str = "Captain America, Team Leader";
const LEGEND: &str = "Captain America, Living Legend";
const MARVEL: &str = "Captain Marvel, Apex Avenger";
const MARVELL: &str = "Captain Mar-Vell, Space-Born";
const QUICKSILVER: &str = "Quicksilver, Speedster";
const HAWKEYE: &str = "Hawkeye, Avenging Archer";
const HERCULES: &str = "Hercules, Olympian Hero";
const SHANG: &str = "Shang-Chi and the Ten Rings";
const ANTMAN: &str = "Ant-Man, Elusive Avenger";
const QUINJET: &str = "Avengers Quinjet";

/// The deck's bookkeeping (a field of `Game`).
#[derive(Debug, Clone, Default)]
pub struct T0c {
    /// Captain America, Living Legend has entered this game (so tapping looks for it)
    pub legend: bool,
    /// Captain Marvel, Apex Avenger has entered this game
    pub marvel: bool,
    /// Marvel is copying a counter placement (so it doesn't copy her own)
    pub in_marvel: bool,
    /// (attacker, defender): the attacker's most recent turn included an attack on the defender
    pub attacked: Vec<(PlayerId, PlayerId)>,
    /// Gift of Immortality: (owner, the Aura's card, the host's card) to return at the next end step
    pub gift_due: Vec<(PlayerId, CardId, CardId)>,
    /// creatures Hawkeye dealt damage to this turn
    pub hawk_hit: Vec<(PermId, TurnStamp)>,
    /// Heroic Sacrifice: (the player, the chosen creature, the turn) damage to the player and their creatures goes there
    pub sac: Option<(PlayerId, PermId, TurnStamp)>,
    /// damage redirected to the Heroic Sacrifice creature so far this combat
    pub sac_dmg: i32,
}

// ------------------------------------------------------------------ helpers
fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn is_hero(g: &Game, m: PermId) -> bool {
    has_type(g, m, "hero")
}

fn hero_card(g: &Game, c: CardId) -> bool {
    g.db.get(c).has_subtype("hero")
}

fn named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && card_name(g, m) == Some(name))
}

fn add_kw(g: &mut Game, m: PermId, kw: Sym) {
    let k = &mut g.perm_mut(m).eot_kw;
    if !k.contains(&kw) {
        k.push(kw);
    }
}

fn opt(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

/// the rest of the cards go to the bottom of the library in a random order
pub(crate) fn bottom_random(g: &mut Game, p: PlayerId, mut rest: Vec<CardId>) {
    for i in (1..rest.len()).rev() {
        let j = g.rng.below(i as u64 + 1) as usize;
        rest.swap(i, j);
    }
    for c in rest {
        g.player_mut(p).library.insert(0, c);
    }
}

/// the top n cards of p's library, top first
pub(crate) fn pop_top(g: &mut Game, p: PlayerId, n: usize) -> Vec<CardId> {
    let lib = &mut g.player_mut(p).library;
    let k = n.min(lib.len());
    (0..k).map(|_| lib.pop().unwrap()).collect()
}

/// how much the AI wants a Hero card in hand
fn hero_worth(g: &Game, c: CardId) -> f64 {
    let d = g.db.get(c);
    d.cmc as f64 + d.pow as f64 * 0.3 + d.bomb as f64
}

// ------------------------------------------------------------------ Captain America, Team Leader
/// another Hero you control enters: vigilance and haste until end of turn, a +1/+1 counter on it and one on Captain
fn cap_team(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src || owner(g, src) != p || !live(g, m) || owner(g, m) != p || !g.is_creature(m) || !is_hero(g, m) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "Hero: vigilance, haste, +1/+1 counters", Some(5.0))? {
        return Ok(());
    }
    if live(g, m) {
        add_kw(g, m, "vigilance");
        add_kw(g, m, "haste");
        g.perm_mut(m).sick = false;
        repl::put_plus(g, m, 1);
    }
    if live(g, src) {
        repl::put_plus(g, src, 1);
    }
    Ok(())
}

// ------------------------------------------------------------------ Captain America, Living Legend
fn legend_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.t0c.legend = true;
    }
    Ok(())
}

/// Called wherever a creature becomes tapped (attacking, a mana ability, a tap cost): while Living Legend is out, the
/// first time each creature of the active player's becomes tapped in their turn it untaps.
pub fn became_tapped(g: &mut Game, m: PermId) {
    if !g.t0c.legend {
        return;
    }
    let p = owner(g, m);
    if g.active != Some(p) || !g.is_creature(m) || !g.perm(m).tapped || named(g, p, LEGEND).is_none() {
        return;
    }
    if once_per_turn(g, p, intern(&format!("legend_{}", m.0))) {
        g.perm_mut(m).tapped = false;
        crate::glog!(g, "    Captain America, Living Legend untaps {}", g.perm(m).name);
    }
}

// ------------------------------------------------------------------ Captain Marvel
fn marvel_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.t0c.marvel = true;
    }
    Ok(())
}

/// Called by `cardcode::put_plus` after n +1/+1 counters went on m: Captain Marvel gets the same, unless m is a Kree.
pub fn counters_put(g: &mut Game, m: PermId, n: i32) {
    if n <= 0 || !g.t0c.marvel || g.t0c.in_marvel || !g.is_creature(m) {
        return;
    }
    let p = owner(g, m);
    let Some(cm) = named(g, p, MARVEL) else { return };
    if cm == m || subtypes(g, m).contains(&"kree") {
        return;
    }
    g.t0c.in_marvel = true;
    repl::put_plus(g, cm, n);
    g.t0c.in_marvel = false;
    crate::glog!(g, "    Captain Marvel gets {} +1/+1 counter(s) too", n);
}

// ------------------------------------------------------------------ Ant-Man
/// Ant-Man can't be blocked by creatures with greater power (the Treasure trigger is compiled from the Oracle text)
pub fn evasion_blocked(g: &Game, b: PermId, a: PermId) -> bool {
    card_name(g, a) == Some(ANTMAN) && epow(g, b) > epow(g, a)
}

/// Ant-Man: whenever he deals combat damage to a player, create that many Treasure tokens
fn antman_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    if a == src && owner(g, src) == p && dmg > 0 && trigger_window(g, p, Some(src), "create Treasures", Some(4.0))? {
        crate::engine::zones::add_treasure(g, p, dmg)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Avengers Quinjet
/// crew 3: tap creatures with power 3 or more between them (summoning-sick ones first) before the first combat
fn quinjet_crew(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || g.perm(src).tapped || g.perm(src).data.truthy(DataKey::Anim) {
        return Ok(());
    }
    let mut cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).tapped && !g.perm(m).phased && m != src)
        .collect();
    cands.sort_by(|&a, &b| {
        (pval(g, a), -epow(g, a)).partial_cmp(&(pval(g, b), -epow(g, b))).unwrap_or(std::cmp::Ordering::Equal)
    });
    let (mut power, mut pick) = (0, vec![]);
    for m in cands {
        if power >= 3 {
            break;
        }
        if g.perm(m).sick || epow(g, m) < 4 || g.perm(m).token {
            pick.push(m);
            power += epow(g, m);
        }
    }
    if power < 3 || !super::lands::open_attack(g, p, 4, true) {
        return Ok(());
    }
    for m in pick {
        g.perm_mut(m).tapped = true;
        became_tapped(g, m);
    }
    let now = Val::Stamp(g.turn_stamp());
    let x = g.perm_mut(src);
    x.data.set(DataKey::Anim, Val::Bool(true));
    (x.pow, x.tgh) = (4, 4);
    x.fly = true;
    x.data.set(DataKey::Crewed, now);
    crate::glog!(g, "  {} crews Avengers Quinjet", pname(g, p));
    Ok(())
}

/// enters or attacks: put a Hero creature card from your hand onto the battlefield, else return a Hero creature card
/// from your graveyard to your hand
fn quinjet_trigger(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let heroes = |g: &Game, zone: &[CardId]| -> Option<CardId> {
        let cs: Vec<CardId> = zone.iter().copied().filter(|&c| g.db.get(c).creature && hero_card(g, c)).collect();
        max_by(&cs, |c| hero_worth(g, c))
    };
    if heroes(g, &g.player(p).hand).is_none() && heroes(g, &g.player(p).gy).is_none() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "Hero from hand to the battlefield, or from the graveyard to hand", Some(5.0))?
    {
        return Ok(());
    }
    if let Some(c) = heroes(g, &g.player(p).hand) {
        let i = g.player(p).hand.iter().position(|&x| x == c).unwrap();
        g.player_mut(p).hand.remove(i);
        enter(g, p, c, Enter::default())?;
        crate::glog!(g, "    Avengers Quinjet puts {} onto the battlefield", g.db.get(c).name);
    } else if let Some(c) = heroes(g, &g.player(p).gy) {
        let i = g.player(p).gy.iter().position(|&x| x == c).unwrap();
        g.player_mut(p).gy.remove(i);
        g.player_mut(p).hand.push(c);
        crate::glog!(g, "    Avengers Quinjet returns {} to hand", g.db.get(c).name);
    }
    Ok(())
}

fn quinjet_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && owner(g, src) == p { quinjet_trigger(g, src, p) } else { Ok(()) }
}

fn quinjet_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) == p && atk.contains(&src) {
        quinjet_trigger(g, src, p)?;
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ Folk Hero, Jarvis, Director Nick Fury, Horn
/// Folk Hero: your commander creatures have "whenever you cast a spell that shares a creature type with this creature,
/// draw a card (once each turn)"
fn folk_hero(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if owner(g, src) != caster || !live(g, src) {
        return Ok(());
    }
    let spell = g.db.get(c);
    let cmds: Vec<PermId> = g
        .player(caster)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            live(g, m) && x.is_cmd && x.orig == caster && g.is_creature(m) && {
                let subs = subtypes(g, m);
                spell.subtypes.iter().any(|s| subs.contains(&&**s))
            }
        })
        .collect();
    for m in cmds {
        let key = intern(&format!("folk_{}", m.0));
        if g.player(caster).flag_turn.get(key) == Some(&g.turn_stamp()) {
            continue;
        }
        if trigger_window(g, caster, Some(m), "Folk Hero: draw a card", Some(4.0))? {
            once_per_turn(g, caster, key);
            draw(g, caster, 1, false)?;
        }
    }
    Ok(())
}

/// Jarvis: whenever you cast a Hero spell, draw a card
fn jarvis(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if owner(g, src) == caster
        && live(g, src)
        && hero_card(g, c)
        && trigger_window(g, caster, Some(src), "draw a card", Some(4.0))?
    {
        draw(g, caster, 1, false)?;
    }
    Ok(())
}

/// Director Nick Fury: Hero spells you cast cost {1} less
fn fury_cost(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    if owner(g, src) == caster && live(g, src) && hero_card(g, c) { -1 } else { 0 }
}

/// Director Nick Fury: whenever you attack, look at the top four cards; a Hero card may go to your hand, the rest to
/// the bottom in a random order
fn fury_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || atk.is_empty() || g.player(p).library.is_empty() {
        return Ok(vec![]);
    }
    let key = intern(&format!("fury_{}", src.0));
    if g.player(p).flag_turn.get(key) == Some(&g.turn_stamp()) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "look at the top four cards for a Hero", Some(4.0))? {
        return Ok(vec![]);
    }
    once_per_turn(g, p, key);
    let mut top = pop_top(g, p, 4);
    let heroes: Vec<CardId> = top.iter().copied().filter(|&c| hero_card(g, c)).collect();
    if let Some(c) = max_by(&heroes, |c| hero_worth(g, c)) {
        top.retain(|&x| x != c);
        g.player_mut(p).hand.push(c);
        crate::glog!(g, "    Director Nick Fury finds {}", g.db.get(c).name);
    }
    bottom_random(g, p, top);
    Ok(vec![])
}

/// Herald's Horn: as it enters, choose a creature type (Hero for this deck)
fn horn_as_enters(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let t = if g.player(p).key == "captain-america-avengers-assemble" {
        "hero"
    } else {
        crate::impls::galadriel::choose_type(g, p, "Herald's Horn")?
    };
    g.perm_mut(m).data.set(DataKey::Ctype, Val::Str(t));
    crate::glog!(g, "    Herald's Horn names {}", t);
    Ok(())
}

/// Herald's Horn: creature spells of the chosen type cost {1} less
fn horn_cost(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    let Some(t) = crate::impls::galadriel::ctype(g, src) else { return 0 };
    if owner(g, src) == caster && live(g, src) && g.db.get(c).creature && g.db.get(c).has_subtype(t) { -1 } else { 0 }
}

/// Herald's Horn: at your upkeep, look at the top card; a creature card of the chosen type may go to your hand
fn horn_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let Some(t) = crate::impls::galadriel::ctype(g, src) else { return Ok(()) };
    if owner(g, src) != p || g.active != Some(p) {
        return Ok(());
    }
    let Some(&top) = g.player(p).library.last() else { return Ok(()) };
    if !(g.db.get(top).creature && g.db.get(top).has_subtype(t)) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "reveal the top card into your hand", Some(4.0))?
        && g.player(p).library.last() == Some(&top)
    {
        g.player_mut(p).library.pop();
        g.player_mut(p).hand.push(top);
        crate::glog!(g, "    Herald's Horn puts {} into hand", g.db.get(top).name);
    }
    Ok(())
}

// ------------------------------------------------------------------ Love on the Battlefield
fn love_key(m: PermId) -> Sym {
    intern(&format!("love_{}", m.0))
}

/// attacking with exactly two creatures: first strike for both, draw a card; either one connecting gets a counter
fn love_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || atk.len() != 2 || atk.iter().any(|&m| !live(g, m)) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "two attackers: first strike, draw a card", Some(5.0))? {
        return Ok(vec![]);
    }
    for &m in atk {
        add_kw(g, m, "first strike");
        let st = g.turn_stamp();
        g.player_mut(p).flag_turn.insert(love_key(m), st);
    }
    draw(g, p, 1, false)?;
    Ok(vec![])
}

fn love_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if owner(g, src) != p || !live(g, a) || g.player(p).flag_turn.get(love_key(a)) != Some(&g.turn_stamp()) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "+1/+1 counter", Some(4.0))? && live(g, a) {
        repl::put_plus(g, a, 1);
    }
    Ok(())
}

// ------------------------------------------------------------------ Hercules
fn hercules_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) == p
        && atk.contains(&src)
        && trigger_window(g, p, Some(src), "+1/+1 counter, indestructible", Some(5.0))?
        && live(g, src)
    {
        repl::put_plus(g, src, 1);
        add_kw(g, src, "indestructible");
    }
    Ok(vec![])
}

/// Hercules was dealt damage in combat and survived: the first time each turn, that many +1/+1 counters
fn hercules_damaged(g: &mut Game, m: PermId, dmg: i32) {
    if dmg > 0
        && live(g, m)
        && card_name(g, m) == Some(HERCULES)
        && once_per_turn(g, owner(g, m), intern(&format!("herc_{}", m.0)))
    {
        repl::put_plus(g, m, dmg);
        crate::glog!(g, "    Hercules gets {} +1/+1 counters", dmg);
    }
}

// ------------------------------------------------------------------ Hawkeye
fn hawk_mark(g: &mut Game, victim: PermId) {
    let st = g.turn_stamp();
    g.t0c.hawk_hit.retain(|x| x.1 == st);
    g.t0c.hawk_hit.push((victim, st));
}

/// Called from combat for a pair that fights (a deals damage to b, b to a): marks what Hawkeye hurts
pub fn pair_mark(g: &mut Game, a: PermId, b: PermId) {
    if owner(g, a) == owner(g, b) {
        return;
    }
    if card_name(g, a) == Some(HAWKEYE) && epow(g, a) > 0 {
        hawk_mark(g, b);
    }
    if card_name(g, b) == Some(HAWKEYE) && epow(g, b) > 0 {
        hawk_mark(g, a);
    }
}

/// Called from combat after a pair fought: a Hercules that survived counts the damage it took
pub fn pair_after(g: &mut Game, a: PermId, b: PermId) {
    let (pa, pb) = (epow(g, a), epow(g, b));
    hercules_damaged(g, b, pa);
    hercules_damaged(g, a, pb);
}

/// a creature an opponent controls died after Hawkeye dealt damage to it this turn: draw a card
fn hawkeye_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let st = g.turn_stamp();
    let p = owner(g, src);
    if owner(g, m) == p || !g.t0c.hawk_hit.contains(&(m, st)) {
        return Ok(());
    }
    g.t0c.hawk_hit.retain(|x| x.0 != m);
    if trigger_window(g, p, Some(src), "a creature Hawkeye damaged died: draw a card", Some(5.0))? {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// {T}: Hawkeye deals 1 damage to any target. Kill a 1-toughness creature when there is one (that draws a card);
/// otherwise ping an opponent's face in the end-of-turn window, when the tap costs nothing.
fn hawkeye_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if owner(g, src) != p || !live(g, src) || x.tapped || x.sick || post == Some(false) && g.step != Step::Main1 {
        return Ok(vec![]);
    }
    let tg: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && live(g, m) && !untargetable(g, m) && etgh(g, m) <= 1)
        .collect();
    if let Some(t) = max_by(&tg, |m| pval(g, m)).filter(|&t| pval(g, t) >= 1.5) {
        return Ok(vec![Opt {
            utility: 3.0 + pval(g, t) / 2.0,
            label: "Hawkeye kills a creature".into(),
            act: Some(Action::Ability { src, f: hawkeye_go, arg: t.0 as i64 }),
        }]);
    }
    if post.is_none() || post == Some(true) {
        return Ok(vec![Opt {
            utility: if post.is_none() { 1.0 } else { 0.3 },
            label: "Hawkeye pings".into(),
            act: Some(Action::Ability { src, f: hawkeye_go, arg: -1 }),
        }]);
    }
    Ok(vec![])
}

fn hawkeye_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let x = g.perm(src);
    if !live(g, src) || x.tapped || x.sick {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    became_tapped(g, src);
    let target = (arg >= 0).then(|| PermId(arg as u32)).filter(|&t| live(g, t) && etgh(g, t) <= 1);
    if !ability_window(g, p, Some(src), "1 damage to any target", Some(3.0), target)? {
        return Ok(true);
    }
    if let Some(t) = target {
        hawk_mark(g, t);
        apply_removal(g, Some(p), t, "dmg1", None)?;
    } else if let Some(&q) = g.opps(p).collect::<Vec<_>>().iter().min_by_key(|&&q| g.player(q).life) {
        lose_life(g, q, 1, Some(p), "ping", None)?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ She-Hulk
/// Other Heroes you control have trample
fn she_hulk_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "trample" && m != src && live(g, src) && owner(g, m) == owner(g, src) && is_hero(g, m)
}

/// Whenever a Hero you control becomes blocked, a +1/+1 counter on it for each creature blocking it
fn she_hulk_blocks(
    g: &mut Game,
    src: Src,
    p: PlayerId,
    _atk: &[PermId],
    _d: PlayerId,
    assign: &mut crate::hooks::Assign,
) -> Res {
    if owner(g, src) != p || !live(g, src) {
        return Ok(());
    }
    let mut blocked: Vec<(PermId, i32)> = vec![];
    for &(a, _) in assign.iter() {
        if live(g, a) && owner(g, a) == p && is_hero(g, a) && !blocked.iter().any(|x| x.0 == a) {
            blocked.push((a, assign.iter().filter(|x| x.0 == a).count() as i32));
        }
    }
    if blocked.is_empty() || !trigger_window(g, p, Some(src), "blocked Heroes get +1/+1 counters", Some(4.0))? {
        return Ok(());
    }
    for (a, n) in blocked {
        if live(g, a) {
            repl::put_plus(g, a, n);
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ Winter Soldier
fn winter_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !atk.contains(&src) {
        return Ok(vec![]);
    }
    let pw = epow(g, src);
    let cands: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).cmc as i32 <= pw).collect();
    let Some(best) = max_by(&cands, |c| hero_worth(g, c) + g.db.get(c).pow as f64 * 0.2) else { return Ok(vec![]) };
    if !trigger_window(g, p, Some(src), "return a creature card from the graveyard", Some(6.0))? {
        return Ok(vec![]);
    }
    let Some(i) = g.player(p).gy.iter().position(|&x| x == best) else { return Ok(vec![]) };
    g.player_mut(p).gy.remove(i);
    let plus = if hero_card(g, best) { 1 } else { 0 };
    enter(g, p, best, Enter { plus, ..Enter::default() })?;
    crate::glog!(g, "    Winter Soldier returns {}", g.db.get(best).name);
    Ok(vec![])
}

// ------------------------------------------------------------------ Shang-Chi
/// Whenever you draw a card, a +1/+1 counter on Shang-Chi; the tenth counter draws five cards and gains 5 life
fn shang_draw(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if owner(g, src) != p || !live(g, src) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "+1/+1 counter", Some(3.0))? && live(g, src) {
        repl::put_plus(g, src, 1);
        shang_ten(g, src, p)?;
    }
    Ok(())
}

/// the tenth +1/+1 counter went on (the count crossed ten): draw five cards and gain 5 life, once
fn shang_ten(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).plus >= 10 && !g.perm(src).data.truthy(DataKey::Used) {
        g.perm_mut(src).data.set(DataKey::Used, Val::Bool(true));
        if trigger_window(g, p, Some(src), "tenth counter: draw five cards, gain 5 life", Some(7.0))? {
            draw(g, p, 5, false)?;
            gain(g, p, 5)?;
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ Gift of Immortality
/// the enchanted creature died: it returns under its owner's control (the same mark Kaya's Ghostform uses), and the
/// Aura returns attached to it at the beginning of the next end step
fn gift_dies(g: &mut Game, _p: PlayerId, a: PermId, host: PermId) -> Res {
    let (Some(gift), Some(hc)) = (g.perm(a).cd, g.perm(host).cd) else { return Ok(()) };
    if g.perm(host).token {
        return Ok(());
    }
    let o = g.perm(host).orig;
    g.perm_mut(host).data.set(DataKey::Ghostform, Val::Player(o));
    g.t0c.gift_due.push((owner(g, a), gift, hc));
    Ok(())
}

static GIFT: AuraSpec = AuraSpec { host_dies: Some(gift_dies), ..AURA };

/// At the beginning of the end step (`cardcode::delayed_end_step`): Gift of Immortality returns attached to its creature
pub fn end_returns(g: &mut Game) -> Res {
    if g.t0c.gift_due.is_empty() {
        return Ok(());
    }
    for (o, gift, hc) in std::mem::take(&mut g.t0c.gift_due) {
        let Some(i) = g.player(o).gy.iter().position(|&x| x == gift) else { continue };
        let host = g
            .players
            .iter()
            .filter(|q| q.alive)
            .flat_map(|q| q.perms.iter().copied())
            .find(|&m| live(g, m) && g.perm(m).cd == Some(hc) && g.is_creature(m));
        let Some(host) = host else { continue };
        g.player_mut(o).gy.remove(i);
        g.attach_to = Some(host);
        enter(g, o, gift, Enter::default())?;
        g.attach_to = None;
        crate::glog!(g, "    Gift of Immortality returns attached to {}", g.perm(host).name);
    }
    Ok(())
}

// ------------------------------------------------------------------ flash
/// Quicksilver (while tapped) and Captain Mar-Vell (while an opponent has cast a spell this turn) let p cast spells as
/// though they had flash. The AI reads this in the end-of-turn window.
pub fn as_flash(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        if !live(g, m) {
            return false;
        }
        match card_name(g, m) {
            Some(QUICKSILVER) => g.perm(m).tapped,
            Some(MARVELL) => {
                let st = g.turn_stamp();
                g.opps(p).any(|q| matches!(&g.player(q).turn_casts, Some((s, cs)) if *s == st && !cs.is_empty()))
            }
            _ => false,
        }
    })
}

// ------------------------------------------------------------------ attacks, and the cost reductions that read them
/// Called from `combat::attack_triggers`: p attacked d
pub fn note_attack(g: &mut Game, p: PlayerId, d: PlayerId) {
    if !g.t0c.attacked.contains(&(p, d)) {
        g.t0c.attacked.push((p, d));
    }
}

/// Called from `cardcode::turn_start`: p's turn begins, so what p did on their last turn is forgotten
pub fn turn_start(g: &mut Game, p: PlayerId) {
    g.t0c.attacked.retain(|x| x.0 != p);
}

/// Avenge: {2} less if a player attacked you during their last turn
fn avenge_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.t0c.attacked.iter().any(|x| x.1 == p && x.0 != p) { -2 } else { 0 }
}

/// Avenge: destroy all creatures; you gain 1 life for each creature destroyed this way
fn avenge(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let all: Vec<PermId> =
        g.players.iter().flat_map(|q| q.perms.iter().copied()).filter(|&m| live(g, m) && g.is_creature(m)).collect();
    let mut n = 0;
    for m in all {
        if live(g, m) {
            die(g, m, "destroy")?;
            if !g.perm(m).on_bf {
                n += 1;
            }
        }
    }
    if n > 0 && g.player(p).alive {
        gain(g, p, n)?;
    }
    Ok("gy")
}

/// Heroic Return: {2} less if a creature is attacking you
fn heroic_return_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let now = g.active;
    if g.step == Step::Combat && g.active != Some(p) && g.t0c.attacked.iter().any(|x| Some(x.0) == now && x.1 == p) {
        -2
    } else {
        0
    }
}

/// Heroic Return: return a creature card from your graveyard to the battlefield; a Hero enters with two more counters
fn heroic_return(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let cands: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    if let Some(best) =
        max_by(&cands, |c| hero_worth(g, c) + g.db.get(c).pow as f64 * 0.2 + if hero_card(g, c) { 2.0 } else { 0.0 })
    {
        let i = g.player(p).gy.iter().position(|&x| x == best).unwrap();
        g.player_mut(p).gy.remove(i);
        let plus = if hero_card(g, best) { 2 } else { 0 };
        enter(g, p, best, Enter { plus, ..Enter::default() })?;
        crate::glog!(g, "    Heroic Return returns {}", g.db.get(best).name);
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Raise the Palisade
/// the creature type to keep, and what bouncing the rest is worth: opposing creatures count in full, our own that
/// come back count half (tokens are lost for good)
fn palisade_best(g: &Game, p: PlayerId) -> (Sym, f64) {
    let creatures = |q: PlayerId| -> Vec<PermId> {
        g.player(q).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
    };
    let mine = creatures(p);
    let mut types: Vec<Sym> = vec![""];
    for &m in &mine {
        for t in subtypes(g, m) {
            if !types.contains(&t) {
                types.push(t);
            }
        }
    }
    let mut best = ("", f64::MIN);
    for t in types {
        let kept = |m: PermId| !t.is_empty() && has_type(g, m, t);
        let theirs: f64 = g.opps(p).flat_map(|q| creatures(q)).filter(|&m| !kept(m)).map(|m| pval(g, m)).sum();
        let ours: f64 = mine
            .iter()
            .copied()
            .filter(|&m| !kept(m))
            .map(|m| if g.perm(m).token { pval(g, m) } else { 0.5 * pval(g, m) })
            .sum();
        if theirs - ours > best.1 {
            best = (t, theirs - ours);
        }
    }
    best
}

fn palisade_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if palisade_best(g, p).1 >= 5.0 { 45 } else { 0 }
}

fn palisade(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let (t, _) = palisade_best(g, p);
    crate::glog!(g, "    Raise the Palisade names {}", if t.is_empty() { "nothing" } else { t });
    let all: Vec<PermId> = g
        .players
        .iter()
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m) && !(!t.is_empty() && has_type(g, m, t)))
        .collect();
    for m in all {
        if live(g, m) {
            if g.perm(m).token {
                crate::engine::zones::leave(g, m)?;
            } else {
                bounce(g, m)?;
            }
        }
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Dismantling Wave: cycling
fn art_or_ench(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    crate::impls::t5::is_artifact(g, m)
        || x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT) || g.db.get(c).types.has(Types::ENCHANTMENT))
}

/// cycling {6}{W}{W}: when you cycle it, destroy all artifacts and enchantments. The AI cycles it when that takes more
/// from the opponents than from itself, or (at the end of the turn) when it has the mana and nothing better to do.
fn wave_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !can_pay(g, p, 6, "WW", false) {
        return Ok(vec![]);
    }
    let value = |g: &Game, q: PlayerId| -> f64 {
        g.player(q)
            .perms
            .iter()
            .copied()
            .filter(|&m| live(g, m) && art_or_ench(g, m))
            .map(|m| pval(g, m).max(0.5))
            .sum()
    };
    let theirs: f64 = g.opps(p).map(|q| value(g, q)).sum();
    let swing = theirs - value(g, p);
    let u = if swing >= 4.0 {
        4.0 + swing / 4.0
    } else if post.is_none() && swing > -1.0 {
        0.4
    } else {
        return Ok(vec![]);
    };
    Ok(vec![opt(u, "cycle Dismantling Wave".into(), wave_go, c.0 as i64)])
}

fn wave_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let Some(i) = g.player(p).hand.iter().position(|&x| x == c) else { return Ok(false) };
    if !can_pay(g, p, 6, "WW", false) || !pay(g, p, 6, "WW", false)? {
        return Ok(false);
    }
    g.player_mut(p).hand.remove(i);
    g.player_mut(p).gy.push(c);
    crate::glog!(g, "  {} cycles Dismantling Wave", pname(g, p));
    draw(g, p, 1, false)?;
    if ability_window_card(g, p, c, "destroy all artifacts and enchantments", Some(6.0), None)? {
        let all: Vec<PermId> = g
            .players
            .iter()
            .flat_map(|q| q.perms.iter().copied())
            .filter(|&m| live(g, m) && art_or_ench(g, m))
            .collect();
        for m in all {
            die(g, m, "destroy")?;
        }
    }
    Ok(true)
}

// ------------------------------------------------------------------ Avengers Tower
/// {T}: {C}, or one mana of any colour spent only on a Hero spell (or an ability of a Hero source)
fn tower_cols(g: &Game, p: PlayerId, _l: PermId) -> Colors {
    if restr_ok(g, p, Restr::Subtype("hero")) { g.player(p).ident } else { Colors::NONE }
}

/// {4},{T}: look at the top three cards; a Hero card may go to your hand, the rest to the bottom
fn tower_options(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(l).tapped || g.player(p).library.len() < 3 || !super::lands::can_pay_without(g, p, l, 4, "") {
        return Ok(vec![]);
    }
    Ok(vec![opt(if post.is_none() { 1.1 } else { 0.5 }, "Avengers Tower digs for a Hero".into(), tower_go, l.0 as i64)])
}

fn tower_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = PermId(arg as u32);
    if !g.player(p).lands.contains(&l) || g.perm(l).tapped || !super::lands::pay_without(g, p, l, 4, "")? {
        return Ok(false);
    }
    g.perm_mut(l).tapped = true;
    if ability_window_card(g, p, g.perm(l).card(), "look at the top three cards", Some(3.0), None)? {
        let mut top = pop_top(g, p, 3);
        let heroes: Vec<CardId> = top.iter().copied().filter(|&c| hero_card(g, c)).collect();
        if let Some(c) = max_by(&heroes, |c| hero_worth(g, c)) {
            top.retain(|&x| x != c);
            g.player_mut(p).hand.push(c);
            crate::glog!(g, "    Avengers Tower finds {}", g.db.get(c).name);
        }
        bottom_random(g, p, top);
    }
    Ok(true)
}

// ------------------------------------------------------------------ Heroic Sacrifice
/// the creature that soaks the damage: the sturdiest one we have (a token or a cheap one first among equals)
fn sac_pick(g: &Game, p: PlayerId) -> Option<PermId> {
    let cr: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect();
    max_by(&cr, |m| {
        etgh(g, m) as f64 * 2.0 - pval(g, m) * 0.5
            + if crate::engine::values::indestructible(g, m) { 20.0 } else { 0.0 }
    })
}

/// Heroic Sacrifice (instant): until end of turn all damage that would be dealt to you and your creatures goes to the
/// chosen creature instead. The AI casts it as the defender, after blocks, when the attack is dangerous
/// (`sac_defend`); damage is redirected in combat (`sac_creature`, `sac_player`, `sac_finish`).
fn sac_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = sac_pick(g, p) {
        g.t0c.sac = Some((p, t, g.turn_stamp()));
        g.t0c.sac_dmg = 0;
        crate::glog!(
            g,
            "    Heroic Sacrifice: damage to {} and their creatures goes to {}",
            pname(g, p),
            g.perm(t).name
        );
    }
    Ok("gy")
}

fn sac_active(g: &Game, d: PlayerId) -> Option<PermId> {
    match g.t0c.sac {
        Some((q, t, st)) if q == d && st == g.turn_stamp() && live(g, t) => Some(t),
        _ => None,
    }
}

/// Called from combat: b (a creature of d's) would be dealt `dmg` combat damage; true when Heroic Sacrifice sends it
/// to the chosen creature instead
pub fn sac_creature(g: &mut Game, d: PlayerId, b: PermId, dmg: i32) -> bool {
    match sac_active(g, d) {
        Some(t) if t != b && dmg > 0 => {
            g.t0c.sac_dmg += dmg;
            true
        }
        _ => false,
    }
}

/// Called from combat: d would take `dmg` combat damage; true when it goes to the chosen creature instead
pub fn sac_player(g: &mut Game, d: PlayerId, dmg: i32) -> bool {
    if sac_active(g, d).is_some() && dmg > 0 {
        g.t0c.sac_dmg += dmg;
        true
    } else {
        false
    }
}

/// Called at the end of combat damage: the chosen creature takes what was redirected; if it dies, its counters move to
/// another creature of the player's and they draw a card
pub fn sac_finish(g: &mut Game) -> Res {
    let n = std::mem::take(&mut g.t0c.sac_dmg);
    let Some((p, t, _)) = g.t0c.sac else { return Ok(()) };
    if n == 0 || !live(g, t) || n < etgh(g, t) || crate::engine::values::indestructible(g, t) {
        return Ok(());
    }
    let plus = g.perm(t).plus;
    die(g, t, "destroy")?;
    if live(g, t) {
        return Ok(());
    }
    let others: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| m != t && live(g, m) && g.is_creature(m)).collect();
    if let Some(m) = max_by(&others, |m| pval(g, m)) {
        if plus > 0 {
            repl::put_plus(g, m, plus);
        }
        draw(g, p, 1, false)?;
        crate::glog!(
            g,
            "    Heroic Sacrifice: {} dies; its counters move to {} and {} draws",
            g.perm(t).name,
            g.perm(m).name,
            pname(g, p)
        );
    }
    Ok(())
}

/// the defender's answer to an attack (hand_defend): cast Heroic Sacrifice when the damage that gets through, or the
/// blockers that would die, are worth more than the card
fn sac_defend(
    g: &mut Game,
    c: CardId,
    d: PlayerId,
    p: PlayerId,
    atk: &mut Vec<PermId>,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    use crate::engine::cast::{castable, on_cast};
    use crate::engine::mana::cost_of;
    use crate::engine::stack::counter_window;
    if !g.player(d).hand.contains(&c) || !castable(g, d, c, "hand") || g.t0c.sac.is_some_and(|x| x.2 == g.turn_stamp())
    {
        return Ok(());
    }
    let (gn, pips) = cost_of(g, d, c);
    if !can_pay(g, d, gn, &pips, false) {
        return Ok(());
    }
    let Some(t) = sac_pick(g, d) else { return Ok(()) };
    let open: i32 = atk
        .iter()
        .copied()
        .filter(|&a| live(g, a) && owner(g, a) == p && !assign.iter().any(|x| x.0 == a))
        .map(|a| epow(g, a))
        .sum();
    let at_risk: f64 = assign
        .iter()
        .filter(|&&(a, b)| live(g, b) && b != t && owner(g, b) == d && epow(g, a) >= etgh(g, b))
        .map(|&(_, b)| pval(g, b))
        .sum();
    let life = g.player(d).life;
    if !(open >= life.min(8).max(life / 3) || at_risk >= 5.0) {
        return Ok(());
    }
    pay(g, d, gn, &pips, false)?;
    if let Some(i) = g.player(d).hand.iter().position(|&x| x == c) {
        g.player_mut(d).hand.remove(i);
    }
    crate::glog!(g, "  {} casts Heroic Sacrifice", pname(g, d));
    on_cast(g, d, c)?;
    if counter_window(g, d, c, 4.0, vec![])? {
        sac_resolve(g, d, c, &Ctx::default())?;
    }
    g.player_mut(d).gy.push(c);
    Ok(())
}

// ------------------------------------------------------------------ registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, CAP_TL)?.etb = Some(cap_team);
    r.card(db, LEGEND)?.etb = Some(legend_etb);
    let c = r.card(db, MARVEL)?;
    c.etb = Some(marvel_etb);
    let c = r.card(db, QUINJET)?;
    c.etb = Some(quinjet_etb);
    c.attack = Some(quinjet_attack);
    c.crew = Some(quinjet_crew);
    *c = c.at_once(crate::hooks::Event::Crew).at_once(crate::hooks::Event::Etb);
    r.card(db, "Folk Hero")?.cast = Some(folk_hero);
    r.card(db, "Jarvis, Earth's Mightiest Butler")?.cast = Some(jarvis);
    let c = r.card(db, "Director Nick Fury")?;
    c.cost = Some(fury_cost);
    c.attack = Some(fury_attack);
    let c = r.card(db, "Herald's Horn")?;
    c.as_enters = Some(horn_as_enters);
    c.cost = Some(horn_cost);
    c.upkeep = Some(horn_upkeep);
    let c = r.card(db, "Love on the Battlefield")?;
    c.attack = Some(love_attack);
    c.combat_damage = Some(love_damage);
    r.card(db, ANTMAN)?.combat_damage = Some(antman_damage);
    r.card(db, HERCULES)?.attack = Some(hercules_attack);
    let c = r.card(db, HAWKEYE)?;
    c.dies = Some(hawkeye_dies);
    c.options = Some(hawkeye_options);
    let c = r.card(db, "She-Hulk, Wallbreaker")?;
    c.grant_kw = Some(she_hulk_kw);
    c.blocks = Some(she_hulk_blocks);
    r.card(db, "Winter Soldier, Reborn Avenger")?.attack = Some(winter_attack);
    r.card(db, SHANG)?.draw = Some(shang_draw);
    crate::impls::common::aura(r, db, "Gift of Immortality", &GIFT)?;
    let c = r.card(db, "Avenge")?;
    c.self_cost = Some(avenge_cost);
    c.resolve = Some(avenge);
    let c = r.card(db, "Heroic Return")?;
    c.self_cost = Some(heroic_return_cost);
    c.resolve = Some(heroic_return);
    let c = r.card(db, "Raise the Palisade")?;
    c.resolve = Some(palisade);
    c.prio = Some(palisade_prio);
    r.card(db, "Dismantling Wave")?.hand_options = Some(wave_options);
    let c = r.card(db, "Avengers Tower")?;
    c.land_cols = Some(tower_cols);
    c.land_options = Some(tower_options);
    let c = r.card(db, "Heroic Sacrifice")?;
    c.resolve = Some(sac_resolve);
    c.hand_defend = Some(sac_defend);
    r.card(db, QUICKSILVER)?;
    Ok(())
}
