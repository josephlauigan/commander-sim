//! Tier 6 "mana" batch: the mana rocks, rituals, dorks and mana-adjacent artifacts of the Tier 6 decks that the
//! compiled Oracle text and the engine's tags don't cover (the audit verdicts are in data/card_overrides.toml; each
//! card's doc comment says how it is modeled and what is left out; what needs the shared engine is in
//! rust/agents/engine-needs-mana.md).

use super::partials::at_once;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::life::lose_life;
use crate::engine::mana::{Source, Unit};
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::values::{epow, pval};
use crate::impls::t5::is_artifact;
use crate::tag::Tag;
use crate::engine::zones::{die, draw, max_by, min_by};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::Sym;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg }) }
}

// ================================================================== Chromatic Star
/// **Chromatic Star** ({1} artifact): {1}, {T}, sacrifice: one mana of any colour (a mana ability: floating mana of
/// any colour of the player's identity); when it is put into a graveyard from the battlefield, draw a card (any way
/// it dies: sacrificed or destroyed). The AI cracks it for the card in the second main phase or the end-of-turn
/// window when it has a spare {1}. The data override drops the old `rock` tag: Star is not a free mana rock.
fn star_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post == Some(false) || !live(g, src) || owner(g, src) != p || g.perm(src).tapped || !can_pay(g, p, 1, "", false)
    {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.0, "crack Chromatic Star", src, star_go, 0)])
}

fn star_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if !pay(g, p, 1, "", false)? {
        g.perm_mut(src).tapped = false;
        return Ok(false);
    }
    crate::glog!(g, "  {} cracks Chromatic Star", g.player(p).name);
    die(g, src, "sac")?;
    g.player_mut(p).floating.any += 1;
    Ok(true)
}

/// when it is put into a graveyard from the battlefield: draw a card
fn star_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, m);
    if trigger_window(g, o, Some(m), "draw a card", None)? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ================================================================== Manifold Key
/// how much mana tapping artifact m makes (its `rock` tag, or the card code's own count), 0 if it makes none
fn rock_amt(g: &Game, p: PlayerId, m: PermId) -> u32 {
    let Some(c) = g.perm(m).cd else { return 0 };
    let Some(rock) = g.db.get(c).tags.str(Tag::Rock) else { return 0 };
    let a = rock.split_once(':').map_or(rock, |x| x.0);
    crate::cardcode::dyn_mana_perm(g, p, m).unwrap_or_else(|| a.parse().unwrap_or(1))
}

/// the tapped mana artifact worth untapping: it must make two or more mana (untapping pays back the {1} and more)
fn key_untap_target(g: &Game, p: PlayerId, key: PermId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != key && live(g, m) && g.perm(m).tapped && is_artifact(g, m) && rock_amt(g, p, m) >= 2)
        .collect();
    max_by(&cs, |m| rock_amt(g, p, m) as f64)
}

/// the creature that gets "can't be blocked" before combat: the biggest untapped attacker (three or more power) when
/// the opponents have an untapped blocker (the same pick as Amphin Pathmage's)
fn key_unblockable_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let blockers = g.opps(p).any(|q| g.player(q).perms.iter().any(|&m| live(g, m) && g.is_creature(m) && !g.perm(m).tapped));
    if !blockers {
        return None;
    }
    let st = g.turn_stamp();
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            live(g, m)
                && g.is_creature(m)
                && !x.tapped
                && !x.sick
                && !x.noatk
                && x.data.get(DataKey::Unbl) != Some(&Val::Stamp(st))
                && epow(g, m) >= 3
        })
        .collect();
    max_by(&cs, |m| epow(g, m) as f64)
}

/// **Manifold Key** ({1} artifact): {1}, {T}: untap another target artifact (the AI untaps a tapped mana artifact
/// that makes two or more mana: Sol Ring, Basalt Monolith, Grim Monolith ... when it holds a spell); {3}, {T}: target
/// creature can't be blocked this turn (before combat, on the biggest attacker, when blockers are untapped).
fn key_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(vec![]);
    }
    let mut o = vec![];
    if can_pay(g, p, 1, "", false)
        && key_untap_target(g, p, src).is_some()
        && g.player(p).hand.iter().any(|&c| !g.db.get(c).land)
    {
        o.push(ability(2.0, "untap an artifact (Manifold Key)", src, key_untap, 0));
    }
    if post == Some(false) && g.active == Some(p) && can_pay(g, p, 3, "", false) && key_unblockable_target(g, p).is_some()
    {
        o.push(ability(2.0, "unblockable (Manifold Key)", src, key_unblock, 0));
    }
    Ok(o)
}

fn key_untap(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(false);
    }
    let Some(t) = key_untap_target(g, p, src) else { return Ok(false) };
    g.perm_mut(src).tapped = true;
    if !pay(g, p, 1, "", false)? {
        g.perm_mut(src).tapped = false;
        return Ok(false);
    }
    if !ability_window(g, p, Some(src), "untap an artifact", None, None)? {
        return Ok(true);
    }
    if live(g, t) {
        g.perm_mut(t).tapped = false;
        crate::glog!(g, "    Manifold Key untaps {}", g.perm(t).name);
    }
    Ok(true)
}

fn key_unblock(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(false);
    }
    let Some(t) = key_unblockable_target(g, p) else { return Ok(false) };
    g.perm_mut(src).tapped = true;
    if !pay(g, p, 3, "", false)? {
        g.perm_mut(src).tapped = false;
        return Ok(false);
    }
    if !ability_window(g, p, Some(src), "can't be blocked", None, None)? {
        return Ok(true);
    }
    let st = g.turn_stamp();
    g.perm_mut(t).data.set(DataKey::Unbl, Val::Stamp(st));
    crate::glog!(g, "    {} can't be blocked this turn (Manifold Key)", g.perm(t).name);
    Ok(true)
}

// ================================================================== Honor-Worn Shaku
/// the legendary permanents p could tap to untap Shaku: untapped, not mana sources themselves, and not a creature
/// that could still attack or block usefully (a summoning-sick one is fine: it is tapped only until p's next untap)
fn shaku_legends(g: &Game, p: PlayerId, shaku: PermId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            let Some(c) = x.cd else { return false };
            let d = g.db.get(c);
            m != shaku
                && live(g, m)
                && !x.tapped
                && d.tag(Tag::Leg)
                && d.tags.str(Tag::Rock).is_none()
                && d.tags.str(Tag::Dork).is_none()
                && (!g.is_creature(m) || x.sick)
        })
        .collect()
}

/// **Honor-Worn Shaku** ({3} artifact): {T}: add {C}; tap an untapped legendary permanent you control: untap it. The
/// mana count is 1 plus one per legendary permanent that can be tapped for the untap (each untap makes another {C}).
fn shaku_mana(g: &Game, p: PlayerId, m: PermId) -> u32 {
    1 + shaku_legends(g, p, m).len() as u32
}

/// tapped for n mana: the extra n - 1 are paid by tapping the least useful legendary permanents
fn shaku_tap(g: &mut Game, p: PlayerId, m: PermId, used: u32) -> Res {
    g.perm_mut(m).tapped = false; // untapped by the ability each time, then tapped for mana again: it ends tapped
    for _ in 1..used {
        let ls = shaku_legends(g, p, m);
        if let Some(l) = min_by(&ls, |l| pval(g, l)) {
            g.perm_mut(l).tapped = true;
        }
    }
    g.perm_mut(m).tapped = true;
    Ok(())
}

// ================================================================== Metalworker
/// **Metalworker** ({3} artifact creature, 1/2): {T}: reveal any number of artifact cards in your hand; add {C}{C}
/// for each. The mana count is twice the artifact cards in hand (a dork tag in the data override makes it a mana
/// creature: summoning sick the turn it arrives). Limitation: if the caster takes the artifact being cast out of the
/// hand before paying, that card is not counted.
fn metalworker_mana(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    2 * g.player(p).hand.iter().filter(|&&c| g.db.get(c).types.has(Types::ARTIFACT)).count() as u32
}

// ================================================================== Treasonous Ogre
/// the AI never pays life for Treasonous Ogre's mana below this life total
const OGRE_LIFE_FLOOR: i32 = 10;

/// **Treasonous Ogre** ({3}{R}, 2/3): dethrone (the engine's keyword); pay 3 life: add {R}. The mana ability is an
/// extra mana unit of red whose count is the number of 3-life payments above `OGRE_LIFE_FLOOR` (at most ten); the
/// payment planner takes lands and rocks first (the unit makes several mana, so it ranks last for small costs) and
/// paying with it costs 3 life per mana (`ogre_tap`). Offered only while the Ogre is untapped (an attacking Ogre
/// does not offer it). The compiled "pay 3 life: floating mana" ability is dropped by the data override, so the AI
/// no longer burns life for mana nobody is waiting to spend.
fn ogre_mana(g: &Game, src: Src, p: PlayerId, _u: &[Unit]) -> Vec<Unit> {
    if !live(g, src) || g.perm(src).tapped {
        return vec![];
    }
    let k = ((g.player(p).life - OGRE_LIFE_FLOOR) / 3).clamp(0, 10) as u32;
    if k == 0 {
        return vec![];
    }
    vec![Unit { src: Source::Perm(src), cols: Colors::from_letters("R"), amt: k }]
}

/// the Ogre's mana was used: 3 life per mana (the Ogre does not tap)
fn ogre_tap(g: &mut Game, p: PlayerId, m: PermId, used: u32) -> Res {
    g.perm_mut(m).tapped = false;
    lose_life(g, p, 3 * used as i32, Some(p), "other", Some(true))?;
    crate::glog!(g, "    {} pays {} life for Treasonous Ogre's mana", g.player(p).name, 3 * used);
    Ok(())
}

// ================================================================== _____ Goblin
/// the red mana the name sticker makes: one per unique vowel (A, E, I, O, U, Y) on it. The sticker sheets are chosen
/// before the game and the sim has no sheets, so it assumes the Tier 6 primer's guaranteed floor of four unique
/// vowels (the best nine sheets give at least {R}{R}{R}{R}).
const GOBLIN_VOWELS: u32 = 4;

/// **_____ Goblin** ({2}{R}, 2/2): when it enters, you may put a name sticker on it; add {R} for each unique vowel on
/// the sticker (Approximate: always four, see `GOBLIN_VOWELS`; the mana floats as red).
fn goblin_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "put a name sticker on it, add R for each unique vowel", None)? {
        g.player_mut(o).floating.r += GOBLIN_VOWELS;
        crate::glog!(g, "    {} adds {} red (_____ Goblin's name sticker)", g.player(o).name, GOBLIN_VOWELS);
    }
    Ok(())
}

// ================================================================== Desperate Ritual
/// **Desperate Ritual** ({1}{R} Arcane instant): add {R}{R}{R} (floating red mana). Splice onto Arcane is not
/// modeled: no other Arcane spell is played alongside it in any deck (Godo is the only deck that plays it). Like
/// Cabal Ritual, the AI casts it only through a deck plan (priority 0).
fn desperate_ritual(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    g.player_mut(p).floating.r += 3;
    crate::glog!(g, "    {} adds RRR (Desperate Ritual)", g.player(p).name);
    Ok("gy")
}

// ================================================================== Rite of Flame
/// **Rite of Flame** ({R} sorcery): add {R}{R}, then {R} for each card named Rite of Flame in each graveyard (the
/// copies already there: this one is still on the stack). Floating red mana; the AI casts it only through a deck plan.
fn rite_of_flame(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let copies = g.players.iter().flat_map(|q| q.gy.iter()).filter(|&&x| &*g.db.get(x).name == "Rite of Flame").count();
    g.player_mut(p).floating.r += 2 + copies as u32;
    crate::glog!(g, "    {} adds {} red (Rite of Flame)", g.player(p).name, 2 + copies);
    Ok("gy")
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Chromatic Star")?;
    c.options = Some(star_options);
    c.self_dies = Some(star_dies);
    at_once(c, Event::SelfDies, false);
    let c = r.card(db, "Desperate Ritual")?;
    c.resolve = Some(desperate_ritual);
    c.prio = Some(zero_prio);
    r.card(db, "Manifold Key")?.options = Some(key_options);
    let c = r.card(db, "Honor-Worn Shaku")?;
    c.dyn_mana_perm = Some(shaku_mana);
    c.on_tap_perm = Some(shaku_tap);
    r.card(db, "Metalworker")?.dyn_mana_perm = Some(metalworker_mana);
    let c = r.card(db, "Treasonous Ogre")?;
    c.extra_mana = Some(ogre_mana);
    c.on_tap_perm = Some(ogre_tap);
    r.card(db, "_____ Goblin")?.etb = Some(goblin_etb);
    let c = r.card(db, "Rite of Flame")?;
    c.resolve = Some(rite_of_flame);
    c.prio = Some(zero_prio);
    Ok(())
}
