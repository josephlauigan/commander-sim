//! Tier 1 pool deck `isshin-mardu-attack-triggers` (EDHREC's bracket-2 average list): the cards the ability language
//! doesn't cover (rust/agents/t12-isshin.md). Fervent Charge, Ingris Stingerquill, Mishra, Claimed by Gix and
//! Commissar Severina Raine are compiled (dsl_parse.rs, `// --- Tiers 1/2 (isshin)`); the rest is here. Every attack
//! trigger below is a hook on `Event::Attack`, so Isshin, Two Heavens as One doubles it (`combat::attack_triggers`).
//!
//! - Curse of Opulence: enchants the opponent with the most life as it enters; whenever any player attacks that
//!   player, you create a Gold token, and an opponent of yours doing the attacking creates one too. Gold is a Treasure
//!   here (the sim's sacrifice-for-one-mana-of-any-colour token): it also counts as a Treasure sacrificed (Captain
//!   Lannery Storm). Left out: moving the curse when its player leaves (it simply stops).
//! - Reconnaissance: your creatures have vigilance on your turn (removing an attacker from combat at end of combat,
//!   after its damage, and untapping it). Left out: removing a blocked attacker from combat to save it.
//! - Adriana, Captain of the Guard: melee on Adriana and your other creatures: each of your attackers gets +1/+1 for
//!   each opponent you attacked this combat, once per Adriana.
//! - Alesha, Who Laughs at Fate: raid: at your end step, if you attacked this turn, the best creature card in your
//!   graveyard with mana value up to Alesha's power to the battlefield (the attack counter is compiled).
//! - Breena, the Demagogue: whenever any player attacks one of your opponents who has more life than another of your
//!   opponents, the attacking player draws a card and you put two +1/+1 counters on your best creature (once per
//!   opponent attacked).
//! - Captain Lannery Storm: whenever you sacrifice a Treasure, +1/+0 until end of turn (its attack Treasure is
//!   compiled).
//! - Firebender Ascension: a quest counter whenever an attacking creature of yours has a compiled "whenever ~ attacks"
//!   trigger or a hand-written attack trigger; from four counters on, that trigger is copied. Left out: tag-driven
//!   attack triggers (Hero of Bladehold, Adeline ...) add no counter; hand-written triggers with a condition (Aurelia's
//!   first attack) count even when they don't fire; the Soldier token's firebending ({R} until end of combat).
//! - Hammers of Moradin: whenever it attacks, for each opponent, tap that player's best untapped creature (myriad is
//!   the engine's, `t0_zurgo`).
//! - Otharri, Suns' Glory: whenever it attacks, an experience counter, then a 2/2 red Rebel tapped and attacking per
//!   experience counter; {2}{R}{W}, tap an untapped Rebel: return it from your graveyard tapped (after combat or at the
//!   end of a turn).
//! - Raiyuu, Storm's Edge: a Samurai or Warrior of yours attacks alone: untap it, and in the first combat an
//!   additional combat phase.
//! - Tectonic Giant: whenever it attacks, 3 damage to each opponent (the burn mode, the better one at a table of
//!   three). Left out: the impulse-draw mode, and the trigger on being targeted by an opponent's spell.

use crate::cards::{CardDb, Colors};
use crate::dsl::model::Ability;
use crate::engine::hooks::run_hook;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::trigger_window;
use crate::engine::tutors::card_worth;
use crate::engine::values::{epow, has_type, pval, untargetable};
use crate::engine::zones::{Enter, Tokens, add_treasure, draw, enter, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::eot;
use crate::impls::partials::{at_once, remove_card};
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;

pub const CURSE: &str = "Curse of Opulence";
pub const OTHARRI: &str = "Otharri, Suns' Glory";

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, src: PermId) -> PlayerId {
    g.perm(src).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// p's live creatures
fn creatures_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

/// the players p's attackers went at this combat (`Game::atk_targets`; the main defender when none is recorded)
pub fn attacked_players(g: &Game, atk: &[PermId], d: PlayerId) -> Vec<PlayerId> {
    let mut out = vec![];
    for &m in atk {
        let q = g.atk_targets.iter().find(|x| x.0 == m).map_or(d, |x| x.1);
        if !out.contains(&q) {
            out.push(q);
        }
    }
    if out.is_empty() {
        out.push(d);
    }
    out
}

// ================================================================== Curse of Opulence
/// enters: it enchants the opponent with the most life (the one the table attacks)
fn curse_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let opps: Vec<PlayerId> = g.opps(o).collect();
    if let Some(q) = max_by(&opps, |q| g.player(q).life as f64) {
        g.perm_mut(src).data.set(DataKey::T12Isshin, Val::Player(q));
    }
    Ok(())
}

/// the player Curse of Opulence enchants
pub fn cursed(g: &Game, src: PermId) -> Option<PlayerId> {
    match g.perm(src).data.get(DataKey::T12Isshin) {
        Some(Val::Player(q)) if g.player(*q).alive => Some(*q),
        _ => None,
    }
}

/// enchanted player is attacked: a Gold token for you, and one for an opponent of yours doing the attacking
fn curse_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let o = owner(g, src);
    let Some(q) = cursed(g, src) else { return Ok(vec![]) };
    if !attacked_players(g, atk, d).contains(&q) || !trigger_window(g, o, Some(src), "create a Gold token", Some(1.5))?
    {
        return Ok(vec![]);
    }
    add_treasure(g, o, 1)?;
    if p != o {
        add_treasure(g, p, 1)?;
    }
    Ok(vec![])
}

fn curse_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).turns <= 6 { 45 } else { 25 }
}

// ================================================================== Reconnaissance
/// your creatures, on your turn: vigilance (each attacker leaves combat untapped after its damage)
fn recon_grant(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    let o = owner(g, src);
    kw == "vigilance" && g.active == Some(o) && g.perm(m).owner == o && g.is_creature(m)
}

fn recon_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if creatures_of(g, p).len() >= 2 { 40 } else { 20 }
}

// ================================================================== Adriana, Captain of the Guard
/// melee (Adriana's own and the one she grants): each of your attackers +1/+1 per opponent attacked this combat
fn adriana_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let o = owner(g, src);
    if p != o || !live(g, src) {
        return Ok(vec![]);
    }
    let n = attacked_players(g, atk, d).iter().filter(|&&q| q != o).count() as i32;
    let mine: Vec<PermId> = atk.iter().copied().filter(|&m| live(g, m) && g.perm(m).owner == o).collect();
    if n == 0 || mine.is_empty() || !trigger_window(g, o, Some(src), "melee", Some(2.0))? {
        return Ok(vec![]);
    }
    for m in mine {
        if live(g, m) {
            eot(g, m, n, n);
        }
    }
    Ok(vec![])
}

// ================================================================== Alesha, Who Laughs at Fate
/// raid: your end step, if you attacked this turn: a creature card with mana value up to Alesha's power back
fn alesha_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if p != o || !live(g, src) || g.player(o).attacking != Some(g.turn_stamp()) {
        return Ok(());
    }
    let pow = epow(g, src);
    let fits = |g: &Game| -> Vec<CardId> {
        g.player(o).gy.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).cmc as i32 <= pow).collect()
    };
    if fits(g).is_empty() || !trigger_window(g, o, Some(src), "return a creature card", Some(3.0))? {
        return Ok(());
    }
    let cs = fits(g);
    let Some(c) = max_by(&cs, |c| card_worth(g, o, c, true) + g.db.get(c).cmc as f64) else { return Ok(()) };
    remove_card(&mut g.player_mut(o).gy, c);
    crate::glog!(g, "    Alesha returns {}", g.db.get(c).name);
    enter(g, o, c, Enter::default())?;
    Ok(())
}

// ================================================================== Breena, the Demagogue
/// a player attacks one of your opponents with more life than another of your opponents: they draw, and two +1/+1
/// counters on your best creature
fn breena_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let o = owner(g, src);
    if !live(g, src) {
        return Ok(vec![]);
    }
    for q in attacked_players(g, atk, d) {
        if q == o || !g.player(q).alive {
            continue;
        }
        let life = g.player(q).life;
        if !g.opps(o).any(|x| x != q && g.player(x).life < life) {
            continue;
        }
        if !trigger_window(g, o, Some(src), "draw, two +1/+1 counters", Some(2.5))? {
            continue;
        }
        draw(g, p, 1, false)?;
        let mine = creatures_of(g, o);
        if let Some(m) = max_by(&mine, |m| pval(g, m)) {
            g.perm_mut(m).plus += 2;
        }
    }
    Ok(vec![])
}

// ================================================================== Captain Lannery Storm
/// you sacrifice a Treasure: +1/+0 until end of turn
fn lannery_sac(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p == owner(g, src) && live(g, src) && what == Sacrificed::Token("Treasure") {
        eot(g, src, 1, 0);
    }
    Ok(())
}

// ================================================================== Firebender Ascension
/// an attacking creature's own attack trigger: a compiled one (its index) or its hand-written hook
#[derive(Clone, Copy)]
enum AtkTrig {
    Compiled(CardId, u16),
    Hook,
}

/// the attack triggers of p's attacking creatures (the triggered abilities their attack caused)
fn atk_trigs(g: &Game, p: PlayerId, atk: &[PermId]) -> Vec<(PermId, AtkTrig)> {
    let mut out = vec![];
    for &m in atk {
        if !live(g, m) || g.perm(m).owner != p {
            continue;
        }
        let Some(c) = g.perm(m).cd else { continue };
        if g.dsl_on {
            for (i, a) in g.db.get(c).abilities.iter().enumerate() {
                if let Ability::Triggered { event, source, .. } = a
                    && event == "attack"
                    && source.as_deref() == Some("self")
                {
                    out.push((m, AtkTrig::Compiled(c, i as u16)));
                }
            }
        }
        if g.registry.get(c).is_some_and(|i| i.attack.is_some()) {
            out.push((m, AtkTrig::Hook));
        }
    }
    out
}

fn quests(g: &Game, src: PermId) -> i64 {
    match g.perm(src).data.get(DataKey::T12Isshin) {
        Some(Val::Int(n)) => *n,
        _ => 0,
    }
}

/// a quest counter per attack trigger of your attacking creatures; from four, that trigger is copied
fn ascension_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let o = owner(g, src);
    if p != o || !live(g, src) {
        return Ok(vec![]);
    }
    let trigs = atk_trigs(g, o, atk);
    if trigs.is_empty() || !trigger_window(g, o, Some(src), "quest counters", Some(2.0))? {
        return Ok(vec![]);
    }
    for (m, t) in trigs {
        if !live(g, src) || g.over {
            break;
        }
        let n = quests(g, src) + 1;
        g.perm_mut(src).data.set(DataKey::T12Isshin, Val::Int(n));
        if n < 4 || !live(g, m) {
            continue;
        }
        crate::glog!(g, "    Firebender Ascension copies {}'s attack trigger", g.perm(m).name);
        match t {
            AtkTrig::Compiled(c, idx) => {
                let ctx = crate::dsl::DslCtx {
                    event_player: Some(d),
                    attack: true,
                    atk_count: atk.len() as i32,
                    ..crate::dsl::DslCtx::base()
                };
                crate::dsl::run_trigger(g, o, m, c, idx, &ctx)?;
            }
            AtkTrig::Hook => run_hook(g, m, Event::Attack, &Call::Attack { p, atk: atk.to_vec(), d })?,
        }
    }
    Ok(vec![])
}

// ================================================================== Hammers of Moradin
/// whenever it attacks: for each opponent, tap that player's best untapped creature
fn hammers_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let o = owner(g, src);
    if p != o || !atk.contains(&src) {
        return Ok(vec![]);
    }
    let pick = |g: &Game, q: PlayerId| -> Option<PermId> {
        let cs: Vec<PermId> =
            creatures_of(g, q).into_iter().filter(|&m| !g.perm(m).tapped && !untargetable(g, m)).collect();
        max_by(&cs, |m| epow(g, m) as f64 + pval(g, m) / 10.0)
    };
    let opps: Vec<PlayerId> = g.opps(o).collect();
    if opps.iter().all(|&q| pick(g, q).is_none()) || !trigger_window(g, o, Some(src), "tap blockers", Some(2.0))? {
        return Ok(vec![]);
    }
    for q in opps {
        if let Some(m) = pick(g, q) {
            g.perm_mut(m).tapped = true;
        }
    }
    Ok(vec![])
}

// ================================================================== Otharri, Suns' Glory
/// whenever it attacks: an experience counter, then a 2/2 red Rebel tapped and attacking per experience counter
fn otharri_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let o = owner(g, src);
    if p != o || !atk.contains(&src) || !trigger_window(g, o, Some(src), "experience, Rebels", Some(4.0))? {
        return Ok(vec![]);
    }
    g.player_mut(o).experience += 1;
    let n = g.player(o).experience.max(0) as u32;
    let spec = Tokens {
        attacking: true,
        sick: false,
        tgh: Some(2),
        color: Some(Colors::from_letters("R")),
        types: vec!["rebel"],
        ..Tokens::new(n, 2)
    };
    make_tokens(g, o, spec)
}

fn rebel(g: &Game, p: PlayerId) -> Option<PermId> {
    let rs: Vec<PermId> =
        creatures_of(g, p).into_iter().filter(|&m| !g.perm(m).tapped && has_type(g, m, "rebel")).collect();
    rs.into_iter().min_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)))
}

/// {2}{R}{W}, tap an untapped Rebel: back from the graveyard tapped (after combat or at the end of a turn: it can't
/// attack this turn anyway)
fn otharri_gy(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post == Some(false) || !g.player(p).gy.contains(&c) || rebel(g, p).is_none() || !can_pay(g, p, 2, "RW", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 4.0,
        label: format!("{OTHARRI} (return from the graveyard)"),
        act: Some(Action::Plan { f: otharri_return, arg: c.0 as i64 }),
    }])
}

fn otharri_return(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let Some(r) = rebel(g, p) else { return Ok(false) };
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 2, "RW", false) || !pay(g, p, 2, "RW", false)? {
        return Ok(false);
    }
    g.perm_mut(r).tapped = true;
    remove_card(&mut g.player_mut(p).gy, c);
    crate::glog!(g, "  {} returns {} from the graveyard", g.player(p).name, OTHARRI);
    let m = enter(g, p, c, Enter::default())?;
    g.perm_mut(m).tapped = true;
    Ok(true)
}

// ================================================================== Raiyuu, Storm's Edge
/// a Samurai or Warrior of yours attacks alone: untap it; the first combat gets an additional one
fn raiyuu_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let o = owner(g, src);
    let [m] = atk else { return Ok(vec![]) };
    let m = *m;
    let ok = |g: &Game| has_type(g, m, "samurai") || has_type(g, m, "warrior") || g.perm(m).warrior;
    if p != o || !live(g, src) || g.perm(m).owner != o || !ok(g) {
        return Ok(vec![]);
    }
    if !trigger_window(g, o, Some(src), "untap, additional combat", Some(5.0))? {
        return Ok(vec![]);
    }
    if live(g, m) {
        g.perm_mut(m).tapped = false;
    }
    if g.player(o).combat_no == 1 {
        g.player_mut(o).extra_combats += 1;
        crate::glog!(g, "    Raiyuu: an additional combat");
    }
    Ok(vec![])
}

// ================================================================== Tectonic Giant
/// whenever it attacks: 3 damage to each opponent
fn giant_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let o = owner(g, src);
    if p != o || !atk.contains(&src) || !trigger_window(g, o, Some(src), "3 damage to each opponent", Some(3.0))? {
        return Ok(vec![]);
    }
    let opps: Vec<PlayerId> = g.opps(o).collect();
    for q in opps {
        lose_life(g, q, 3, Some(o), "triggers", None)?;
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, CURSE)?;
    c.etb = Some(curse_etb);
    at_once(c, Event::Etb, true);
    c.attack = Some(curse_attack);
    c.prio = Some(curse_prio);
    let c = r.card(db, "Reconnaissance")?;
    c.grant_kw = Some(recon_grant);
    c.prio = Some(recon_prio);
    r.card(db, "Adriana, Captain of the Guard")?.attack = Some(adriana_attack);
    r.card(db, "Alesha, Who Laughs at Fate")?.end_step = Some(alesha_end);
    r.card(db, "Breena, the Demagogue")?.attack = Some(breena_attack);
    let c = r.card(db, "Captain Lannery Storm")?;
    c.sacrifice = Some(lannery_sac);
    at_once(c, Event::Sacrifice, true);
    r.card(db, "Firebender Ascension")?.attack = Some(ascension_attack);
    r.card(db, "Hammers of Moradin")?.attack = Some(hammers_attack);
    let c = r.card(db, OTHARRI)?;
    c.attack = Some(otharri_attack);
    c.gy_options = Some(otharri_gy);
    r.card(db, "Raiyuu, Storm's Edge")?.attack = Some(raiyuu_attack);
    r.card(db, "Tectonic Giant")?.attack = Some(giant_attack);
    Ok(())
}
