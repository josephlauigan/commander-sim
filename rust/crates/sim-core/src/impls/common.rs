//! Python's `cards/impl/common.py`: card code shared by many pool decks, and the shared machinery the engine calls by
//! name (through `cardcode.rs`): Auras (`aura`, `auras_on`, `attached_bonus`, `aura_fall`, umbra armor), the Oblivion
//! Ring family, sacrifice outlets and death payoffs (`aristocrat_options`, `sac_in_response`), the planeswalker
//! framework (`walker`, `ult_pressure`), Food, adventures and the fixed threat values (`CI.PVAL`).
//!
//! Ported for M5: everything the pilot decks (Sauron and Tier 1) play, and the table-driven entries of the same
//! families (every Aura, the O-Ring family, the death-trigger creatures, the sacrifice outlets). The rest of
//! common.py's cards (pillowfort taxes, stax pieces, the four planeswalkers, Urza's Saga, Walking Ballista ...) wait
//! for phase 6; they play with their tags meanwhile.

use crate::cards::{CardDb, CardDef, Colors, Types};
use crate::engine::cast::{castable, on_cast};
use crate::engine::hooks::{fire_trigger, total_trigger_copies};
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::values::{epow, has_type, once_per_turn, pval, protected_from, stopped, untargetable};
use crate::engine::zones::{
    Enter, Tokens, Zone, add_treasure, die, draw, edict, enter, leave, make_tokens, max_by, min_by, sac_worth,
    to_zone_card,
};
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::{DataKey, Game, TurnStamp, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

// ------------------------------------------------------------------ small helpers
/// m is on p's battlefield (Python's `m in p.perms`)
fn on(g: &Game, m: PermId, p: PlayerId) -> bool {
    let x = g.perm(m);
    x.on_bf && x.owner == p
}

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn name_of(g: &Game, m: PermId) -> &'static str {
    g.perm(m).name
}

/// the card name of a permanent ("" for a token)
fn card_name(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

fn def(g: &Game, m: PermId) -> Option<&CardDef> {
    g.perm(m).cd.map(|c| g.db.get(c))
}

fn opps(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    g.opps(p).collect()
}

fn player_name(g: &Game, p: PlayerId) -> &'static str {
    g.player(p).name
}

/// t1.fresh: once_per_turn(g, p, key) would pass; sets nothing (trigger guards run twice: probe, then real)
fn fresh(g: &Game, p: PlayerId, key: Sym) -> bool {
    g.player(p).flag_turn.get(key) != Some(&g.turn_stamp())
}

/// common.best_opp_creature: p's opponents' most valuable targetable creature
pub fn best_opp_creature(g: &Game, p: PlayerId, pred: impl Fn(PermId) -> bool) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased && !untargetable(g, m) && pred(m))
        .collect();
    max_by(&cs, |m| pval(g, m))
}

/// common.best_opp_nonland: p's opponents' most valuable targetable permanent
pub fn best_opp_nonland(g: &Game, p: PlayerId, pred: impl Fn(PermId) -> bool) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| !g.perm(m).phased && !untargetable(g, m) && pred(m))
        .collect();
    max_by(&cs, |m| pval(g, m))
}

/// common.n_ench: p's enchantments (not phased out)
pub fn n_ench(g: &Game, p: PlayerId) -> i32 {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| !g.perm(m).phased && def(g, m).is_some_and(|d| d.types.has(Types::ENCHANTMENT)))
        .count() as i32
}

/// common.n_auras: p's Auras attached to something
pub fn n_auras(g: &Game, p: PlayerId) -> i32 {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| def(g, m).is_some_and(|d| d.has_subtype("aura")) && g.perm(m).attached.is_some())
        .count() as i32
}

// ======================================================== Auras
/// (g, p: the Aura's controller, the Aura, the creature it enchants) -> (dp, dt)
pub type AuraBonusFn = fn(&Game, PlayerId, PermId, PermId) -> (i32, i32);
/// on_etb(g, p, the Aura, its host): what the Aura does as it enters (its enter trigger; attaching is not)
pub type AuraEtbFn = fn(&mut Game, PlayerId, PermId, PermId) -> Res;
/// host_ok(g, p, m): m may carry this Aura (Daybreak Coronet: an enchanted creature)
pub type HostOkFn = fn(&Game, PlayerId, PermId) -> bool;
/// host_pick(g, p, the Aura): the deck's own choice of host (Y'shtola's Curiosity)
pub type HostPickFn = fn(&mut Game, PlayerId, PermId) -> Res<Option<PermId>>;

/// An Aura's static effects (Python's `AURA[name]` dict).
#[derive(Debug, Clone, Copy)]
pub struct AuraSpec {
    pub pow: i32,
    pub tgh: i32,
    /// keywords the enchanted creature has (lower case)
    pub kws: &'static [&'static str],
    pub bonus: Option<AuraBonusFn>,
    /// umbra armor: destroyed instead of the creature
    pub umbra: bool,
    /// colours it gives protection from
    pub prot: &'static str,
    /// whose creature it goes on: 'own', 'opp', 'either'
    pub target: &'static str,
    pub on_etb: Option<AuraEtbFn>,
    pub host_ok: Option<HostOkFn>,
    /// where it goes when it leaves: Some("always") back to hand (Rancor), Some("host_dies") (Angelic Destiny)
    pub back: Option<&'static str>,
    pub host_pick: Option<HostPickFn>,
}

/// common.aura's defaults
pub const AURA: AuraSpec = AuraSpec {
    pow: 0,
    tgh: 0,
    kws: &[],
    bonus: None,
    umbra: false,
    prot: "",
    target: "own",
    on_etb: None,
    host_ok: None,
    back: None,
    host_pick: None,
};

/// common.aura: register an Aura (its static effects, and the enter hook that attaches it)
pub fn aura(r: &mut Registry, db: &CardDb, name: &str, spec: &'static AuraSpec) -> Result<(), String> {
    let c = r.card(db, name)?;
    c.aura = Some(spec);
    c.etb = Some(aura_etb);
    *c = c.at_once(Event::Etb); // no trigger_window of its own: it attaches as it enters
    Ok(())
}

/// the registered spec of a card (common.AURA[name])
pub fn aura_spec(g: &Game, c: CardId) -> Option<&'static AuraSpec> {
    g.registry.get(c)?.aura
}

fn spec_of(g: &Game, a: PermId) -> Option<&'static AuraSpec> {
    aura_spec(g, g.perm(a).cd?)
}

/// common.own_host: the creature an Aura should go on: the commander in a voltron deck, else the best creature
pub fn own_host(g: &Game, p: PlayerId, spec: &AuraSpec, aura_perm: Option<PermId>) -> Option<PermId> {
    let mut cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            let cr = g.is_creature(m);
            (cr && !x.phased && Some(m) != aura_perm && x.cd.is_some()) || (cr && x.token && !x.phased)
        })
        .collect();
    if let Some(ok) = spec.host_ok {
        cands.retain(|&m| ok(g, p, m));
    }
    if cands.is_empty() {
        return None;
    }
    let pref = crate::ai::plans::config(g.player(p).key).aura_host;
    for &m in &cands {
        if pref == Some("commander") && g.perm(m).is_cmd {
            return Some(m);
        }
    }
    max_by(&cands, |m| {
        let x = g.perm(m);
        (x.is_cmd as i32 * 2) as f64 + pval(g, m) + if x.fly { 3.0 } else { 0.0 }
    })
}

/// common._aura_etb: the Aura src entered: it enchants the creature it was put onto (g.attach_to), else the best
/// host; with none it goes to the graveyard
fn aura_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let Some(spec) = spec_of(g, src) else { return Ok(()) };
    let mut host = g.attach_to;
    if host.is_none_or(|h| !g.perm(h).on_bf) {
        host = None;
        // HUMAN(phase 9): human_aura_host (the person picks the creature)
        if let Some(f) = spec.host_pick {
            host = f(g, owner(g, src), src)?;
        }
        if host.is_none() {
            host = own_host(g, owner(g, src), spec, Some(src));
        }
    }
    let Some(host) = host else {
        leave(g, src)?;
        let (o, cd) = (owner(g, src), g.perm(src).cd.unwrap());
        g.player_mut(o).gy.push(cd);
        return Ok(());
    };
    g.perm_mut(src).attached = Some(host);
    g.auras.push(src);
    g.dsl_on = true;
    crate::glog!(g, "    {} enchants {}", card_name(g, src), name_of(g, host));
    if let Some(f) = spec.on_etb {
        f(g, owner(g, src), src, host)?;
    }
    Ok(())
}

fn auras_on_iter(g: &Game, m: PermId) -> impl Iterator<Item = PermId> + '_ {
    g.auras.iter().copied().filter(move |&a| {
        let x = g.perm(a);
        x.attached == Some(m) && x.on_bf && !x.phased
    })
}

/// common.auras_on: the Auras on m (on the battlefield, not phased out)
pub fn auras_on(g: &Game, m: PermId) -> Vec<PermId> {
    auras_on_iter(g, m).collect()
}

/// zur.LOCKS: the Auras that lock what they enchant (they don't count as m's own Auras)
const LOCK_AURAS: [&str; 5] = ["Arrest", "Prison Sentence", "Luminous Bonds", "Bound in Silence", "Encrust"];

/// engine.pval's count of m's controller's own Auras on m (common.auras_on, without zur.LOCKS)
pub fn own_auras_on(g: &Game, m: PermId) -> usize {
    let p = owner(g, m);
    auras_on_iter(g, m).filter(|&a| owner(g, a) == p && !LOCK_AURAS.contains(&card_name(g, a))).count()
}

/// common.attached_bonus: power and toughness from Auras, Elspeth's emblem, and the card code's own rules (TOKEN_PT,
/// SELF_PT, CREATURE_PT)
pub fn attached_bonus(g: &Game, m: PermId) -> (i32, i32) {
    let (mut dp, mut dt) = (0, 0);
    if !g.auras.is_empty() {
        for a in auras_on_iter(g, m) {
            let Some(spec) = spec_of(g, a) else { continue };
            dp += spec.pow;
            dt += spec.tgh;
            if let Some(f) = spec.bonus {
                let (x, y) = f(g, owner(g, a), a, m);
                dp += x;
                dt += y;
            }
        }
    }
    let x = g.perm(m);
    if g.is_creature(m) && g.player(x.owner).elspeth_emblem {
        dp += 2;
        dt += 2;
    }
    if x.cd.is_none() && !x.data.is_empty() {
        for f in &g.registry.token_pt {
            let (a, b) = f(g, m);
            dp += a;
            dt += b;
        }
    }
    if g.selfpt
        && g.player(x.owner).alive
        && let Some(f) = x.cd.and_then(|c| g.registry.get(c)).and_then(|i| i.self_pt)
    {
        let (a, b) = f(g, x.owner, m);
        dp += a;
        dt += b;
    }
    if g.selfpt && g.is_creature(m) {
        // the CREATURE_PT entries, then partials' coat_bonus + lineage_bonus + bestow_bonus (which partials
        // registers as one creature_pt entry giving (b, b))
        for f in &g.registry.creature_pt {
            let (a, b) = f(g, m);
            dp += a;
            dt += b;
        }
    }
    (dp, dt)
}

/// common.attached_kw: an Aura on m gives it keyword kw
pub fn attached_kw(g: &Game, m: PermId, kw: &str) -> bool {
    auras_on_iter(g, m).any(|a| spec_of(g, a).is_some_and(|s| s.kws.contains(&kw)))
}

/// common.attached_prot: the colours m's Auras give it protection from
pub fn attached_prot(g: &Game, m: PermId) -> Colors {
    auras_on_iter(g, m)
        .filter_map(|a| spec_of(g, a))
        .fold(Colors::NONE, |s, spec| s.union(Colors::from_letters(spec.prot)))
}

/// common.umbra_save: umbra armor: m would be destroyed: an umbra on it is destroyed instead
pub fn umbra_save(g: &mut Game, m: PermId) -> Res<bool> {
    for a in auras_on(g, m) {
        if spec_of(g, a).is_some_and(|s| s.umbra) {
            crate::glog!(g, "    {} is destroyed instead of {} (umbra armor)", card_name(g, a), name_of(g, m));
            die(g, a, "destroy")?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// common.aura_fall: m left the battlefield: Auras attached to it go to the graveyard (or back to hand). Python's
/// ON_DETACH and ON_HOST_DIES tables are empty (no list runs Kasmina's Transmutation or Gift of Immortality).
pub fn aura_fall(g: &mut Game, m: PermId) -> Res {
    for a in g.auras.clone() {
        if a == m {
            if let Some(i) = g.auras.iter().position(|&x| x == a) {
                g.auras.remove(i);
            }
            continue;
        }
        if g.perm(a).attached == Some(m) {
            let back = spec_of(g, a).and_then(|s| s.back);
            // always true: aura_fall runs once m has left (so 'host_dies' Auras return however their host left)
            let died = !g.perm(m).on_bf;
            let Some(i) = g.auras.iter().position(|&x| x == a) else { continue };
            g.auras.remove(i);
            leave(g, a)?;
            if back == Some("always") || (back == Some("host_dies") && died) {
                let (o, cd) = (owner(g, a), g.perm(a).cd.unwrap());
                g.player_mut(o).hand.push(cd);
            } else {
                to_zone_card(g, a, Zone::Gy);
            }
        }
    }
    Ok(())
}

// ---- the Auras' enter effects
/// Sheltered by Ghosts: O-Ring effect on entry (its enter trigger; attaching is not)
fn sheltered_etb(g: &mut Game, p: PlayerId, a: PermId, _host: PermId) -> Res {
    if !trigger_window(g, p, Some(a), "exile a nonland permanent", Some(5.0))? || !on(g, a, p) {
        return Ok(());
    }
    oring_exile(g, a, p, |g, m| g.perm(m).cd.is_some() || g.perm(m).token, false, false)
}

/// Cartouche of Solidarity: a 1/1 Warrior
fn cartouche_etb(g: &mut Game, p: PlayerId, a: PermId, _host: PermId) -> Res {
    if trigger_window(g, p, Some(a), "create a 1/1 Warrior", None)? {
        let spec = Tokens { color: Some(Colors::from_letters("W")), types: vec!["warrior"], ..Tokens::new(1, 1) };
        make_tokens(g, p, spec)?;
    }
    Ok(())
}

/// Sage's Reverie: draw a card per Aura you control attached to something
fn reverie_etb(g: &mut Game, p: PlayerId, a: PermId, _host: PermId) -> Res {
    if trigger_window(g, p, Some(a), "draw a card per Aura", None)? {
        let n = n_auras(g, p);
        draw(g, p, n.max(0) as u32, false)?;
    }
    Ok(())
}

fn draw1_etb(g: &mut Game, p: PlayerId, a: PermId, _host: PermId) -> Res {
    if trigger_window(g, p, Some(a), "draw a card", None)? {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

// ---- the Auras' bonuses
fn glitters(g: &Game, p: PlayerId, _a: PermId, _m: PermId) -> (i32, i32) {
    let k = g
        .player(p)
        .perms
        .iter()
        .filter(|&&x| def(g, x).is_some_and(|d| d.types.has(Types::ARTIFACT) || d.types.has(Types::ENCHANTMENT)))
        .count() as i32;
    (k, k)
}

fn coronet_ok(g: &Game, _p: PlayerId, m: PermId) -> bool {
    g.auras.iter().any(|&a| g.perm(a).attached == Some(m))
}

fn ethereal(g: &Game, p: PlayerId, _a: PermId, _m: PermId) -> (i32, i32) {
    (n_ench(g, p), n_ench(g, p))
}

fn reverie(g: &Game, p: PlayerId, _a: PermId, _m: PermId) -> (i32, i32) {
    (n_auras(g, p), n_auras(g, p))
}

fn ancestral_mask(g: &Game, _p: PlayerId, _a: PermId, _m: PermId) -> (i32, i32) {
    let all: i32 = g.players.iter().filter(|q| q.alive).map(|q| n_ench(g, q.id)).sum();
    let k = 2 * (all - 1);
    (k, k)
}

fn empyrial(g: &Game, p: PlayerId, _a: PermId, _m: PermId) -> (i32, i32) {
    let k = g.player(p).hand.len() as i32;
    (k, k)
}

/// common.aura(...) calls
static AURAS: &[(&str, AuraSpec)] = &[
    ("All That Glitters", AuraSpec { bonus: Some(glitters), ..AURA }),
    (
        "Angelic Destiny",
        AuraSpec { pow: 4, tgh: 4, kws: &["flying", "first strike"], back: Some("host_dies"), ..AURA },
    ),
    ("Battle Mastery", AuraSpec { kws: &["double strike"], ..AURA }),
    (
        "Cartouche of Solidarity",
        AuraSpec { pow: 1, tgh: 1, kws: &["first strike"], on_etb: Some(cartouche_etb), ..AURA },
    ),
    // Partial: +3/+3; the life-doubling combat trigger is modeled in a hook
    ("Celestial Mantle", AuraSpec { pow: 3, tgh: 3, ..AURA }),
    (
        "Daybreak Coronet",
        AuraSpec {
            pow: 3,
            tgh: 3,
            kws: &["first strike", "vigilance", "lifelink"],
            host_ok: Some(coronet_ok),
            ..AURA
        },
    ),
    ("Ethereal Armor", AuraSpec { kws: &["first strike"], bonus: Some(ethereal), ..AURA }),
    // Approximate: lifelink, umbra armor; moving it is not modeled
    ("Felidar Umbra", AuraSpec { kws: &["lifelink"], umbra: true, ..AURA }),
    // Approximate: protection from black (the most common removal colour) and the bounce-and-recast Light-Paws loop
    ("Flickering Ward", AuraSpec { prot: "B", ..AURA }),
    // Partial: +1/+0 and flying; graveyard recursion not modeled
    ("Gryff's Boon", AuraSpec { pow: 1, kws: &["flying"], ..AURA }),
    ("Hyena Umbra", AuraSpec { pow: 1, tgh: 1, kws: &["first strike"], umbra: true, ..AURA }),
    ("Mammoth Umbra", AuraSpec { pow: 3, tgh: 3, kws: &["vigilance"], umbra: true, ..AURA }),
    ("On Serra's Wings", AuraSpec { pow: 1, tgh: 1, kws: &["flying", "vigilance", "lifelink"], ..AURA }),
    ("Sage's Reverie", AuraSpec { bonus: Some(reverie), on_etb: Some(reverie_etb), ..AURA }),
    // Partial: +1/+1 vigilance; escape not modeled
    ("Sentinel's Eyes", AuraSpec { pow: 1, tgh: 1, kws: &["vigilance"], ..AURA }),
    // Approximate: O-Ring effect on entry; +1/+0 and lifelink; its ward is not modeled
    ("Sheltered by Ghosts", AuraSpec { pow: 1, kws: &["lifelink"], on_etb: Some(sheltered_etb), ..AURA }),
    // Approximate: indestructible; moving it is not modeled
    ("Shielded by Faith", AuraSpec { kws: &["indestructible"], ..AURA }),
    ("Spectra Ward", AuraSpec { pow: 2, tgh: 2, prot: "WUBRG", ..AURA }),
    // Approximate: gains life equal to the enchanted creature's combat damage (hook)
    ("Spirit Link", AURA),
    // Approximate: protection from creatures read as unblockable
    ("Spirit Mantle", AuraSpec { pow: 1, tgh: 1, kws: &["unblockable"], ..AURA }),
    // Approximate: indestructible; flash on a commander not modeled
    ("Timely Ward", AuraSpec { kws: &["indestructible"], ..AURA }),
    // Approximate: draw on entry; protection from creatures read as unblockable
    ("Unquestioned Authority", AuraSpec { kws: &["unblockable"], on_etb: Some(draw1_etb), ..AURA }),
    ("Ancestral Mask", AuraSpec { bonus: Some(ancestral_mask), ..AURA }),
    ("Rancor", AuraSpec { pow: 2, kws: &["trample"], back: Some("always"), ..AURA }),
    // Full: +1/+1, draw on damage to an opponent (hook), umbra armor
    ("Snake Umbra", AuraSpec { pow: 1, tgh: 1, umbra: true, ..AURA }),
    ("Spider Umbra", AuraSpec { pow: 1, tgh: 1, kws: &["reach"], umbra: true, ..AURA }),
    ("Empyrial Armor", AuraSpec { bonus: Some(empyrial), ..AURA }),
];

// ---- the enchanted creature's combat damage (common._host_damage)
/// Celestial Mantle: double your life total
fn mantle(g: &mut Game, src: Src, _p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if g.perm(src).attached == Some(a) && trigger_window(g, owner(g, src), Some(src), "double your life total", None)?
    {
        let o = owner(g, src);
        let life = g.player(o).life;
        gain(g, o, life)?;
    }
    Ok(())
}

/// Spirit Link: gain that much life
fn spirit_link(g: &mut Game, src: Src, _p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    if g.perm(src).attached == Some(a) && trigger_window(g, owner(g, src), Some(src), &format!("gain {dmg} life"), None)?
    {
        gain(g, owner(g, src), dmg)?;
    }
    Ok(())
}

/// Snake Umbra: draw a card
fn snake_umbra(g: &mut Game, src: Src, _p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if g.perm(src).attached == Some(a) && trigger_window(g, owner(g, src), Some(src), "draw a card", None)? {
        draw(g, owner(g, src), 1, false)?;
    }
    Ok(())
}

// ======================================================== exile-until-it-leaves (Oblivion Ring family)
/// which opposing permanents an O-Ring effect may take
pub type PermPred = fn(&Game, PermId) -> bool;

/// common.oring_exile: exile the best permanent(s) opponents control until src leaves the battlefield
pub fn oring_exile(
    g: &mut Game,
    src: PermId,
    p: PlayerId,
    pred: PermPred,
    per_opponent: bool,
    creature_only: bool,
) -> Res {
    let src_cd = g.perm(src).cd;
    let pips = Colors::from_letters(src_cd.map_or("", |c| &g.db.get(c).pips));
    let groups: Vec<Vec<PlayerId>> =
        if per_opponent { g.opps(p).map(|q| vec![q]).collect() } else { vec![g.opps(p).collect()] };
    let mut targets = vec![];
    for grp in groups {
        let cands: Vec<PermId> = grp
            .iter()
            .flat_map(|&o| g.player(o).perms.iter().copied())
            .filter(|&m| {
                !g.perm(m).phased
                    && !untargetable(g, m)
                    && pred(g, m)
                    && (g.is_creature(m) || !creature_only)
                    && !protected_from(g, m, pips)
            })
            .collect();
        if let Some(best) = max_by(&cands, |m| pval(g, m))
            && pval(g, best) >= 1.0
        {
            targets.push(best);
        }
    }
    let mut exiled = vec![];
    for m in targets {
        let o = owner(g, m);
        apply_removal(g, Some(p), m, "exile", src_cd)?;
        let x = g.perm(m);
        if !on(g, m, o) && !x.token {
            let cd = x.phys.or(x.cd).unwrap();
            if cd != g.player(o).cmd && g.player(x.orig).exile.contains(&cd) {
                exiled.push(Val::List(vec![Val::Card(cd), Val::Player(x.orig)]));
            }
        }
    }
    let mut all = match g.perm(src).data.get(DataKey::Oring) {
        Some(Val::List(v)) => v.clone(),
        _ => vec![],
    };
    all.extend(exiled);
    g.perm_mut(src).data.set(DataKey::Oring, Val::List(all));
    Ok(())
}

/// common.oring_return: src left the battlefield: what it exiled returns (an effect ending, not a trigger)
fn oring_return(g: &mut Game, src: Src, _m: PermId) -> Res {
    let Some(Val::List(v)) = g.perm(src).data.get(DataKey::Oring).cloned() else { return Ok(()) };
    for e in v {
        let Val::List(pair) = e else { continue };
        let (Val::Card(cd), Val::Player(o)) = (&pair[0], &pair[1]) else { continue };
        let (cd, o) = (*cd, *o);
        if g.player(o).exile.contains(&cd) && g.player(o).alive {
            let ex = &mut g.player_mut(o).exile;
            let i = ex.iter().position(|&x| x == cd).unwrap();
            ex.remove(i);
            enter(g, o, cd, Enter::default())?;
            crate::glog!(g, "    {} returns to the battlefield", g.db.get(cd).name);
        }
    }
    Ok(())
}

/// common._oring's enter trigger
fn oring_etb(g: &mut Game, src: Src, m: PermId, per_opponent: bool, creature_only: bool) -> Res {
    if m == src
        && trigger_window(g, owner(g, src), Some(src), "exile a permanent", Some(5.0))?
        && on(g, src, owner(g, src))
    {
        oring_exile(g, src, owner(g, src), |_, _| true, per_opponent, creature_only)?;
    }
    Ok(())
}

/// Oblivion Ring, Banishing Light, Cast Out (cycling not modeled), Detention Sphere (Approximate: exiles the one
/// target; other copies of the same name stay)
fn oring_any(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    oring_etb(g, src, m, false, false)
}

/// Journey to Nowhere: a creature
fn journey(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    oring_etb(g, src, m, false, true)
}

/// Grasp of Fate: one per opponent
fn grasp(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    oring_etb(g, src, m, true, false)
}

// ======================================================== Esper Sentinel
/// Approximate: opponents pay X only with 2 mana to spare after it; otherwise you draw
fn sentinel(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    let d = g.db.get(c);
    if caster == o || d.creature || d.land {
        return Ok(());
    }
    let st = g.turn_stamp();
    let noncreature = match &g.player(caster).turn_casts {
        Some((s, cs)) if *s == st => cs.iter().filter(|&&x| !g.db.get(x).creature).count(),
        _ => 0,
    };
    if noncreature != 1 {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), &format!("draw a card unless {} pays", player_name(g, caster)), None)? {
        return Ok(());
    }
    let x = epow(g, src);
    if x != 0 && can_pay(g, caster, x.max(0) as u32, "", false) && total_mana(g, caster, false) as i32 >= x + 2 {
        pay(g, caster, x.max(0) as u32, "", false)?;
        return Ok(());
    }
    draw(g, o, 1, false)
}

// ======================================================== cost reducers
fn danitha(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    if caster == owner(g, src) && (d.has_subtype("aura") || d.has_subtype("equipment")) { -1 } else { 0 }
}

/// Partial: Aura discount; heroic not modeled
fn hero_of_iroas(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    if caster == owner(g, src) && g.db.get(c).has_subtype("aura") { -1 } else { 0 }
}

/// Partial: enchantment discount; +1/+1 counters not modeled
fn starfield(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    if caster == owner(g, src) && g.db.get(c).types.has(Types::ENCHANTMENT) { -1 } else { 0 }
}

fn pearl(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    if caster == owner(g, src) && g.db.get(c).pips.contains('W') { -1 } else { 0 }
}

// ======================================================== sacrifice outlets and death payoffs (aristocrats)
/// a sacrifice outlet's effect when used: (g, p, the outlet, what was sacrificed)
type SacFx = fn(&mut Game, PlayerId, PermId, PermId) -> Res;

fn bombard(g: &mut Game, p: PlayerId, _src: PermId, _m: PermId) -> Res {
    let os = opps(g, p);
    if os.is_empty() {
        return Ok(());
    }
    let lethal: Vec<PlayerId> = os.iter().copied().filter(|&q| g.player(q).life <= 1).collect();
    let q = match lethal.first() {
        Some(&q) => q,
        None => min_by(&os, |q| g.player(q).life as f64).unwrap(),
    };
    lose_life(g, q, 1, Some(p), "triggers", None)
}

fn feeder(g: &mut Game, _p: PlayerId, src: PermId, _m: PermId) -> Res {
    g.perm_mut(src).plus += 1;
    Ok(())
}

fn ashnod(g: &mut Game, p: PlayerId, _src: PermId, _m: PermId) -> Res {
    g.player_mut(p).floating.c += 2;
    Ok(())
}

/// Phyrexian Altar (and t4's Skirk Prospector): one mana of any colour
fn altar(g: &mut Game, p: PlayerId, _src: PermId, _m: PermId) -> Res {
    g.player_mut(p).floating.any += 1;
    Ok(())
}

/// rules2._seer (it replaces common's None for Viscera Seer and Woe Strider): scry 1
fn seer(g: &mut Game, p: PlayerId, _src: PermId, _m: PermId) -> Res {
    crate::cardcode::scry(g, p, 1, false)
}

/// common.SAC_OUTLET: free sacrifice outlets and their effect when used (None: nothing but the death itself), as
/// the Python leaves it once every module has loaded: rules2 gives Viscera Seer and Woe Strider their scry, t4 adds
/// Skirk Prospector
const SAC_OUTLET: [(&str, Option<SacFx>); 10] = [
    ("Viscera Seer", Some(seer)),
    ("Carrion Feeder", Some(feeder)),
    ("Woe Strider", Some(seer)),
    ("Cartel Aristocrat", None),
    ("Ashnod's Altar", Some(ashnod)),
    ("Phyrexian Altar", Some(altar)),
    ("Goblin Bombardment", Some(bombard)),
    ("Yahenni, Undying Partisan", None),
    ("Spawning Pit", None),
    ("Skirk Prospector", Some(altar)),
];

/// common.DEATH_DRAIN: value per creature death of yours (drains count once per opponent)
const DEATH_DRAIN: [(&str, f64); 12] = [
    ("Blood Artist", 1.0),
    ("Zulaport Cutthroat", 1.0),
    ("Cruel Celebrant", 1.0),
    ("Bastion of Remembrance", 1.0),
    ("Falkenrath Noble", 1.0),
    ("Vindictive Vampire", 1.0),
    ("Syr Konrad, the Grim", 1.0),
    ("Poison-Tip Archer", 1.0),
    ("Elas il-Kor, Sadistic Pilgrim", 1.0),
    ("Mayhem Devil", 0.5),
    ("Goblin Bombardment", 0.3),
    ("Mirkwood Bats", 0.3),
];

/// common.EDICTS: each opponent sacrifices a creature
const EDICTS: [&str; 3] = ["Grave Pact", "Dictate of Erebos", "Butcher of Malakir"];

/// common.DEATH_DRAW
const DEATH_DRAW: [(&str, f64); 7] = [
    ("Grim Haruspex", 1.0),
    ("Midnight Reaper", 1.0),
    ("Morbid Opportunist", 0.5),
    ("Skemfar Avenger", 1.0),
    ("Dark Prophecy", 1.0),
    ("Pitiless Plunderer", 0.6),
    ("Korvold, Fae-Cursed King", 1.0),
];

fn lookup(t: &[(&str, f64)], name: &str) -> f64 {
    t.iter().find(|x| x.0 == name).map_or(0.0, |x| x.1)
}

fn sac_fx(name: &str) -> Option<Option<SacFx>> {
    SAC_OUTLET.iter().find(|x| x.0 == name).map(|x| x.1)
}

/// common.SAC_OUTLET membership (pool_ai casts sacrifice outlets at 50)
pub fn is_sac_outlet(name: &str) -> bool {
    sac_fx(name).is_some()
}

/// the bartist / drain hand tags among p's permanents (phased ones too, as the Python counts them)
fn drain_tags(g: &Game, p: PlayerId) -> f64 {
    g.player(p)
        .perms
        .iter()
        .filter(|&&x| def(g, x).is_some_and(|d| d.tag(Tag::Bartist) || d.tag(Tag::Drain)))
        .count() as f64
}

/// common.death_value: what one creature death is worth to p right now (drains, cards, edicts), with copies
pub fn death_value(g: &Game, p: PlayerId, m: Option<PermId>) -> f64 {
    let names: Vec<&str> = g
        .player(p)
        .perms
        .iter()
        .filter(|&&x| g.perm(x).cd.is_some() && !g.perm(x).phased)
        .map(|&x| card_name(g, x))
        .collect();
    let drain = names.iter().map(|n| lookup(&DEATH_DRAIN, n)).psum() + drain_tags(g, p);
    let draw_ = names.iter().map(|n| lookup(&DEATH_DRAW, n)).psum();
    let copies = 1 + if g.hooks.is_empty() { 0 } else { total_trigger_copies(g, p, "dies", m) };
    let mut edict_v = 0.0;
    let k = names.iter().filter(|n| EDICTS.contains(n)).count() as f64;
    if k > 0.0 {
        // each opponent loses the creature it values least
        for q in g.opps(p) {
            let cr: Vec<PermId> =
                g.player(q).perms.iter().copied().filter(|&x| g.is_creature(x) && !g.perm(x).phased).collect();
            if !cr.is_empty() {
                edict_v += 0.5 + 0.8 * cr.iter().map(|&x| pval(g, x)).fold(f64::INFINITY, f64::min);
            }
        }
    }
    let n = g.opps(p).count() as f64;
    copies as f64 * (drain * n * 0.8 + draw_ * 1.5 + k * edict_v)
}

/// common.outlets: p's free sacrifice outlets
pub fn outlets(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&x| {
            let n = card_name(g, x);
            g.perm(x).cd.is_some() && is_sac_outlet(n) && !g.perm(x).phased && !stopped(g, n)
        })
        .collect()
}

/// common.sac_through: sacrifice m through outlet src
pub fn sac_through(g: &mut Game, p: PlayerId, src: PermId, m: PermId) -> Res {
    let fx = sac_fx(card_name(g, src)).flatten();
    crate::glog!(g, "  {} sacrifices {} to {}", player_name(g, p), name_of(g, m), card_name(g, src));
    die(g, m, "sac")?;
    if let Some(f) = fx {
        f(g, p, src, m)?;
    }
    Ok(())
}

fn sac_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !on(g, src, p) || !on(g, m, p) {
        return Ok(false);
    }
    sac_through(g, p, src, m)?;
    Ok(true)
}

/// common.aristocrat_options: sacrifice fodder when the death triggers are worth more than the creature, or are
/// lethal
pub fn aristocrat_options(g: &mut Game, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let outs = outlets(g, p);
    let Some(&src) = outs.first() else { return Ok(vec![]) };
    let fod: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            let cr = g.is_creature(m);
            (cr && !x.phased && m != src && !x.is_cmd && x.cd.is_some() && !is_sac_outlet(card_name(g, m)))
                || (cr && x.token && !x.phased)
        })
        .collect();
    if fod.is_empty() {
        return Ok(vec![]);
    }
    let per = death_value(g, p, None);
    let pl = g.player(p);
    let mut drain_per = pl
        .perms
        .iter()
        .filter(|&&x| g.perm(x).cd.is_some())
        .map(|&x| lookup(&DEATH_DRAIN, card_name(g, x)))
        .psum()
        + drain_tags(g, p);
    drain_per *= (1 + if g.hooks.is_empty() { 0 } else { total_trigger_copies(g, p, "dies", None) }) as f64;
    let lethal = drain_per != 0.0 && g.opps(p).all(|q| g.player(q).life as f64 <= drain_per * fod.len() as f64);
    let m = min_by(&fod, |x| sac_worth(g, x)).unwrap();
    let x = g.perm(m);
    let busy = if post == Some(false) && !x.sick && !x.tapped { 0.8 } else { 0.0 };
    let gain_ = per - 1.5 * sac_worth(g, m) - busy;
    if !lethal && gain_ < 1.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: if lethal { 9.0 } else { 1.0 + gain_ },
        label: format!("sacrifice {} ({})", name_of(g, m), card_name(g, src)),
        act: Some(Action::Ability { src, f: sac_go, arg: m.0 as i64 }),
    }])
}

/// common.sac_in_response: a creature of p's is about to be exiled / bounced / stolen: sacrifice it for value
/// instead
pub fn sac_in_response(g: &mut Game, p: PlayerId, m: PermId, kind: Sym) -> Res<bool> {
    if !g.is_creature(m) || !matches!(kind, "exile" | "bounce" | "tuck") || g.perm(m).is_cmd {
        return Ok(false);
    }
    // HUMAN(phase 9): a person's creature is not sacrificed for them
    let Some(&o) = outlets(g, p).iter().find(|&&o| o != m) else { return Ok(false) };
    sac_through(g, p, o, m)?;
    Ok(true)
}

// ======================================================== simple death-trigger creatures
/// common._dies_other's guard: a creature m died (not src itself if `other`, yours if `yours`, a card if
/// `nontoken`); then the trigger's window
fn dies_guard(g: &mut Game, src: Src, m: PermId, nontoken: bool, other: bool, yours: bool, what: &str) -> Res<bool> {
    let x = g.perm(m);
    if !g.is_creature(m) || (other && m == src) || (yours && x.owner != owner(g, src)) || (nontoken && x.token) {
        return Ok(false);
    }
    trigger_window(g, owner(g, src), Some(src), what, None)
}

fn drain1(g: &mut Game, p: PlayerId) -> Res {
    for q in opps(g, p) {
        lose_life(g, q, 1, Some(p), "drain", None)?;
    }
    let n = g.opps(p).count() as i32;
    gain(g, p, n)
}

fn dmg_each(g: &mut Game, p: PlayerId, n: i32) -> Res {
    for q in opps(g, p) {
        lose_life(g, q, n, Some(p), "triggers", None)?;
    }
    Ok(())
}

fn scion(g: &mut Game, p: PlayerId) -> Res {
    make_tokens(g, p, Tokens { color: Some(Colors::NONE), types: vec!["eldrazi", "scion"], ..Tokens::new(1, 1) })?;
    Ok(())
}

fn celebrant(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if dies_guard(g, src, m, false, false, true, "each opponent loses 1 life")? {
        drain1(g, owner(g, src))?;
    }
    Ok(())
}

fn haruspex(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if dies_guard(g, src, m, true, true, true, "draw a card")? {
        draw(g, owner(g, src), 1, false)?;
    }
    Ok(())
}

fn reaper(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if dies_guard(g, src, m, true, false, true, "draw a card and lose 1 life")? {
        let p = owner(g, src);
        lose_life(g, p, 1, Some(p), "other", None)?;
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// Approximate: Scion token; its sacrifice-for-mana is not modeled
fn sifter(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if dies_guard(g, src, m, true, true, true, "create a Scion")? {
        scion(g, owner(g, src))?;
    }
    Ok(())
}

/// Approximate: Spawn token; its sacrifice-for-mana is not modeled
fn pawn(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if dies_guard(g, src, m, true, false, true, "create a Spawn")? {
        let spec =
            Tokens { tgh: Some(1), color: Some(Colors::NONE), types: vec!["eldrazi", "spawn"], ..Tokens::new(1, 0) };
        make_tokens(g, owner(g, src), spec)?;
    }
    Ok(())
}

fn requiem(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if dies_guard(g, src, m, false, true, true, "create a 1/1 Spirit")? && !has_type(g, m, "spirit") {
        let spec = Tokens { fly: true, color: Some(Colors::from_letters("W")), types: vec!["spirit"], ..Tokens::new(1, 1) };
        make_tokens(g, owner(g, src), spec)?;
    }
    Ok(())
}

fn prophecy(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if dies_guard(g, src, m, false, false, true, "draw a card and lose 1 life")? {
        let p = owner(g, src);
        draw(g, p, 1, false)?;
        lose_life(g, p, 1, Some(p), "other", None)?;
    }
    Ok(())
}

fn vindictive(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if dies_guard(g, src, m, false, true, true, "1 damage to each opponent")? {
        let p = owner(g, src);
        dmg_each(g, p, 1)?;
        gain(g, p, 1)?;
    }
    Ok(())
}

/// Approximate: any other creature dying deals 1 to each opponent (cards leaving graveyards ignored)
fn konrad(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if dies_guard(g, src, m, false, true, false, "1 damage to each opponent")? {
        dmg_each(g, owner(g, src), 1)?;
    }
    Ok(())
}

fn morbid(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let k = intern(&format!("morbid{}", src.0));
    if g.is_creature(m)
        && m != src
        && fresh(g, owner(g, src), k)
        && trigger_window(g, owner(g, src), Some(src), "draw a card", None)?
        && once_per_turn(g, owner(g, src), k)
    {
        draw(g, owner(g, src), 1, false)?;
    }
    Ok(())
}

/// Approximate: Scion token; its sacrifice-for-mana is not modeled
fn thrall(g: &mut Game, m: Src, _m: PermId, _cause: Sym) -> Res {
    if trigger_window(g, owner(g, m), Some(m), "create a Scion", None)? {
        scion(g, owner(g, m))?;
    }
    Ok(())
}

/// Approximate: drains on token sacrifices (Treasure / Food / Clue included); token creation not counted
fn bats_sac(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    let token = match what {
        Sacrificed::Token(k) => matches!(k, "Treasure" | "Food" | "Clue"),
        Sacrificed::Perm(x) => g.perm(x).token,
        Sacrificed::Card(_) => false,
    };
    if p == owner(g, src) && token && trigger_window(g, p, Some(src), "each opponent loses 1 life", None)? {
        for q in opps(g, p) {
            lose_life(g, q, 1, Some(p), "drain", None)?;
        }
    }
    Ok(())
}

fn revel(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if g.is_creature(m)
        && owner(g, m) != owner(g, src)
        && trigger_window(g, owner(g, src), Some(src), "create a Treasure", None)?
    {
        add_treasure(g, owner(g, src), 1)?;
    }
    Ok(())
}

fn revel_win(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src)
        && g.player(p).treasures >= 10
        && trigger_window(g, p, Some(src), "you win the game", Some(10.0))?
        && g.player(p).treasures >= 10
    {
        crate::ai::win(g, p, "Revel in Riches", None)?;
    }
    Ok(())
}

// ---- Priest of Forgotten Gods
fn priest_fodder(g: &Game, p: PlayerId, src: PermId) -> Vec<PermId> {
    let mut f: Vec<(PermId, f64)> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && m != src && !g.perm(m).is_cmd)
        .map(|m| (m, pval(g, m)))
        .filter(|&(m, v)| g.perm(m).token || v < 2.5)
        .collect();
    f.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap()); // stable, as Python's sorted
    f.into_iter().map(|x| x.0).collect()
}

/// Full: sacrifices two spare creatures: each opponent loses 2 and sacrifices, BB, draw
fn priest(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).tapped || g.perm(src).sick {
        return Ok(vec![]);
    }
    if priest_fodder(g, p, src).len() < 2 || g.opps(p).count() == 0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 3.5 + 0.5 * g.opps(p).count() as f64,
        label: "Priest of Forgotten Gods".into(),
        act: Some(Action::Ability { src, f: priest_go, arg: 0 }),
    }])
}

fn priest_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let f2: Vec<PermId> = priest_fodder(g, p, src).into_iter().take(2).collect();
    if f2.len() < 2 || g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    for m in f2 {
        die(g, m, "sac")?;
    }
    if !ability_window(g, p, Some(src), "each opponent loses 2 life and sacrifices a creature", Some(6.0), None)? {
        return Ok(true);
    }
    for q in opps(g, p) {
        lose_life(g, q, 2, Some(p), "drain", None)?;
        edict(g, q, false)?;
    }
    g.player_mut(p).floating.any += 2;
    draw(g, p, 1, false)?;
    check_state(g)?;
    Ok(true)
}

// ======================================================== staples
/// Full: taps for {C}; cracked for a card late (six or more lands)
fn mind_stone(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(true) || g.perm(src).tapped || g.player(p).lands.len() < 6 || !can_pay(g, p, 1, "", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.0,
        label: "crack Mind Stone".into(),
        act: Some(Action::Ability { src, f: mind_stone_go, arg: 0 }),
    }])
}

fn mind_stone_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).tapped || !on(g, src, p) {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if !can_pay(g, p, 1, "", false) {
        g.perm_mut(src).tapped = false;
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    die(g, src, "sac")?;
    if ability_window(g, p, Some(src), "draw a card", None, None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

/// common.exile_gy: q's graveyard is exiled
pub fn exile_gy(g: &mut Game, q: PlayerId, by: Option<&str>) {
    if g.player(q).gy.is_empty() {
        return;
    }
    let pl = g.player_mut(q);
    let gy = std::mem::take(&mut pl.gy);
    pl.exile.extend(gy);
    crate::glog!(g, "    {}'s graveyard is exiled{}", player_name(g, q), by.map_or(String::new(), |b| format!(" by {b}")));
}

/// Rest in Peace (Approximate): graveyards are exiled on entry and then kept empty (cards are exiled as soon as
/// state-based checks run; dies triggers still happen); undying/persist stop
fn rip(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, owner(g, src), Some(src), "exile all graveyards", None)? {
        for i in 0..g.players.len() {
            let q = PlayerId(i as u8);
            if g.player(q).alive {
                exile_gy(g, q, Some("Rest in Peace"));
            }
        }
    }
    Ok(())
}

fn rip_sweep(g: &mut Game, _src: Src) -> Res {
    for q in g.players.iter_mut() {
        if q.alive && !q.gy.is_empty() {
            let gy = std::mem::take(&mut q.gy);
            q.exile.extend(gy);
        }
    }
    Ok(())
}

fn rip_nogy(_g: &Game, _src: Src, _p: PlayerId) -> i32 {
    1
}

/// Full: battle cry; two Soldiers tapped and attacking
fn hero(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src) && trigger_window(g, p, Some(src), "create two attacking Soldiers", None)? {
        let spec = Tokens {
            color: Some(Colors::from_letters("W")),
            types: vec!["soldier"],
            attacking: true,
            sick: false,
            ..Tokens::new(2, 1)
        };
        return make_tokens(g, p, spec);
    }
    Ok(vec![])
}

/// Full: a deathtouch Snake each upkeep when you have none
fn ophiomancer(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let o = owner(g, src);
    let none = |g: &Game| !g.player(o).perms.iter().any(|&m| has_type(g, m, "snake"));
    if none(g) && trigger_window(g, o, Some(src), "create a deathtouch Snake", None)? && none(g) {
        let spec = Tokens { dt: true, color: Some(Colors::from_letters("B")), types: vec!["snake"], ..Tokens::new(1, 1) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

// ---- Pernicious Deed
/// what Pernicious Deed with X destroys (src: the Deed itself, spared while choosing X)
fn deed_hit(g: &Game, m: PermId, x: u32) -> bool {
    let p = g.perm(m);
    let kind = match p.cd {
        None => true,
        Some(c) => {
            let t = g.db.get(c).types;
            g.is_creature(m) || t.has(Types::ARTIFACT) || t.has(Types::ENCHANTMENT)
        }
    };
    kind && p.cd.map_or(0, |c| g.db.get(c).cmc) <= x
}

/// Full: {X}, sacrifice: destroys every artifact, creature and enchantment with mana value X or less; used when
/// opponents lose clearly more
fn deed(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post.is_none() {
        return Ok(vec![]);
    }
    let avail = total_mana(g, p, false);
    let mut best: Option<(f64, u32)> = None;
    for x in 0..=avail {
        let hit = |m: PermId| !g.perm(m).phased && m != src && deed_hit(g, m, x);
        let theirs = g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).filter(|&m| hit(m)).map(|m| pval(g, m)).psum();
        let mine = g.player(p).perms.iter().copied().filter(|&m| hit(m)).map(|m| pval(g, m)).psum();
        let net = theirs - 1.3 * mine;
        if best.is_none_or(|b| net > b.0) {
            best = Some((net, x));
        }
    }
    let Some((net, x)) = best.filter(|b| b.0 >= 8.0) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: net / 3.0,
        label: format!("Pernicious Deed (X={x})"),
        act: Some(Action::Ability { src, f: deed_go, arg: x as i64 }),
    }])
}

fn deed_go(g: &mut Game, src: PermId, p: PlayerId, x: i64) -> Res<bool> {
    let x = x as u32;
    if !on(g, src, p) || !can_pay(g, p, x, "", false) {
        return Ok(false);
    }
    pay(g, p, x, "", false)?;
    crate::glog!(g, "  {} sacrifices Pernicious Deed (X={x})", player_name(g, p));
    die(g, src, "sac")?;
    let label = format!("destroy everything with mana value {x} or less");
    if !ability_window(g, p, Some(src), &label, Some(8.0), None)? {
        return Ok(true);
    }
    let alive: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    for q in alive {
        for m in g.player(q).perms.clone() {
            if g.perm(m).phased {
                continue;
            }
            if deed_hit(g, m, x) && on(g, m, q) {
                die(g, m, "destroy")?;
            }
        }
    }
    check_state(g)?;
    Ok(true)
}

// ======================================================== planeswalkers (the walker framework)
/// value(g, p, src): the ability's utility now, None if it can't be used
pub type WalkerVal = fn(&Game, PlayerId, PermId) -> Option<f64>;
/// effect(g, p, src)
pub type WalkerEff = fn(&mut Game, PlayerId, PermId) -> Res;

/// One loyalty ability (Python's `(loyalty change, label, value, effect)`).
#[derive(Debug, Clone, Copy)]
pub struct WalkerAb {
    pub delta: i32,
    pub label: &'static str,
    pub val: WalkerVal,
    pub eff: WalkerEff,
}

/// common.walker: a planeswalker whose abilities the AI uses one per turn at sorcery speed, the most valuable it
/// can afford. (rules.ult appends an ultimate to a walker's list: register the whole list here.)
pub fn walker(r: &mut Registry, db: &CardDb, name: &str, abilities: &'static [WalkerAb]) -> Result<(), String> {
    let c = r.card(db, name)?;
    c.walker = Some(abilities);
    c.options = Some(walker_options);
    Ok(())
}

fn round_stamp(g: &Game, p: PlayerId) -> TurnStamp {
    TurnStamp { round: g.round, active: Some(p) }
}

/// common._uses: loyalty abilities src used this round
fn uses(g: &Game, p: PlayerId, src: PermId) -> i64 {
    let x = g.perm(src);
    if x.loyalty_used != Some(round_stamp(g, p)) {
        return 0;
    }
    x.data.get(DataKey::LoyaltyN).map_or(1, Val::int)
}

/// common._allowed: Oath of Teferi lets each planeswalker use two abilities a turn
fn allowed(g: &Game, p: PlayerId) -> i64 {
    let oath =
        g.player(p).perms.iter().any(|&m| card_name(g, m) == "Oath of Teferi" && !g.perm(m).phased);
    if oath { 2 } else { 1 }
}

/// Carth the Lion: each loyalty ability costs an extra [+1] (CI.total(g, 'loyalty_extra', p)).
/// PORT(phase 6): the loyalty_extra event has no CardImpl slot yet, so this is 0.
fn loyalty_extra(_g: &Game, _p: PlayerId) -> i32 {
    0
}

fn walker_abilities(g: &Game, src: PermId) -> &'static [WalkerAb] {
    g.perm(src).cd.and_then(|c| g.registry.get(c)).and_then(|i| i.walker).unwrap_or(&[])
}

/// common.walker's options hook
pub fn walker_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p {
        return Ok(vec![]);
    }
    if g.perm(src).loyalty.is_none() {
        let l = def(g, src).and_then(|d| d.start_loyalty).unwrap_or(3);
        g.perm_mut(src).loyalty = Some(l);
    }
    if uses(g, p, src) >= allowed(g, p) {
        return Ok(vec![]);
    }
    let extra = loyalty_extra(g, p);
    let name = card_name(g, src).to_string();
    let mut out = vec![];
    for (i, ab) in walker_abilities(g, src).iter().enumerate() {
        let delta = ab.delta + extra;
        if g.perm(src).loyalty.unwrap() + delta < 0 {
            continue;
        }
        let Some(u) = (ab.val)(g, p, src) else { continue };
        out.push(Opt {
            utility: u + 0.15 * delta as f64,
            label: format!("{name} {delta:+}"),
            act: Some(Action::Ability { src, f: walker_go, arg: i as i64 }),
        });
    }
    Ok(out)
}

fn walker_go(g: &mut Game, src: PermId, p: PlayerId, i: i64) -> Res<bool> {
    let ab = walker_abilities(g, src)[i as usize];
    let delta = ab.delta + loyalty_extra(g, p);
    if !on(g, src, p) || uses(g, p, src) >= allowed(g, p) || g.perm(src).loyalty.unwrap_or(0) + delta < 0 {
        return Ok(false);
    }
    let n = uses(g, p, src) + 1;
    let stamp = round_stamp(g, p);
    let x = g.perm_mut(src);
    x.loyalty_used = Some(stamp);
    x.data.set(DataKey::LoyaltyN, Val::Int(n));
    x.loyalty = Some(x.loyalty.unwrap_or(0) + delta); // loyalty costs aren't doubled (Doubling Season doubles effects)
    crate::glog!(g, "  {} uses {} ({:+}): {}", player_name(g, p), card_name(g, src), delta, ab.label);
    let dead = g.perm(src).loyalty.unwrap_or(0) <= 0; // state-based: at 0 loyalty it goes before its ability resolves
    if dead {
        leave(g, src)?;
        to_zone_card(g, src, Zone::Gy);
    }
    let imp = if delta <= -5 { Some(8.0) } else { None };
    if ability_window(g, p, Some(src), &format!("{delta:+}"), imp, None)? {
        (ab.eff)(g, p, src)?;
    }
    if on(g, src, p) && g.perm(src).loyalty.unwrap_or(0) <= 0 {
        leave(g, src)?;
        to_zone_card(g, src, Zone::Gy);
    }
    Ok(true)
}

/// common.ult_pressure: how close a planeswalker is to its ultimate (loyalty / the cost of its most expensive minus
/// ability, when that's 5 or more)
pub fn ult_pressure(g: &Game, m: PermId) -> f64 {
    let abil = walker_abilities(g, m);
    let loyalty = g.perm(m).loyalty.unwrap_or(0);
    if abil.is_empty() || loyalty == 0 {
        return 0.0;
    }
    let ult = -abil.iter().map(|a| a.delta).min().unwrap_or(0);
    if ult >= 5 { loyalty as f64 / ult as f64 } else { 0.0 }
}

// ======================================================== Food, adventures: the outside decks' card plays
/// common.sac_food: p sacrifices n Food
pub fn sac_food(g: &mut Game, p: PlayerId, n: u32) -> Res<bool> {
    if g.player(p).foods < n {
        return Ok(false);
    }
    g.player_mut(p).foods -= n;
    for _ in 0..n {
        if !g.hooks.is_empty() {
            fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p, what: Sacrificed::Token("Food") })?;
        }
    }
    Ok(true)
}

/// common.food_options: {2}, {T}, sacrifice a Food: gain 3 life (only when life matters)
pub fn food_options(g: &mut Game, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.player(p).foods < 1 || !can_pay(g, p, 2, "", false) || g.player(p).life > 15 || post == Some(false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt { utility: 2.0, label: "eat a Food".into(), act: Some(Action::Plan { f: eat_food, arg: 0 }) }])
}

fn eat_food(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.player(p).foods < 1 || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    sac_food(g, p, 1)?;
    gain(g, p, 3)?;
    Ok(true)
}

/// common.ADVENTURE: (the adventure half's cost (generic, pips), its removal kind, its targets)
const ADVENTURE: [(&str, (u32, &str), &str, &str); 1] = [
    // Approximate: Petty Theft bounces a threat, then the Faerie is castable (as from exile)
    ("Brazen Borrower", (1, "U"), "bounce", "nl"),
];

/// common.adventure_options: cast an adventure card's spell half at a real threat; the creature half is the card's
/// normal cast later
pub fn adventure_options(g: &mut Game, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    let mut o = vec![];
    for c in g.player(p).hand.clone() {
        let Some(&(_, (generic, pips), kind, tgt)) = ADVENTURE.iter().find(|x| *x.0 == *g.db.get(c).name) else {
            continue;
        };
        if g.player(p).adv_done.contains(&c) {
            continue;
        }
        if !can_pay(g, p, generic, pips, false) || !castable(g, p, c, "hand") {
            continue;
        }
        let tg = legal_targets(g, p, kind, tgt, false, None);
        let Some(t) = max_by(&tg, |m| pval(g, m)) else { continue };
        if pval(g, t) < 4.0 {
            continue;
        }
        o.push(Opt {
            utility: pval(g, t) - 3.5,
            label: format!("{} adventure -> {}", g.db.get(c).name, name_of(g, t)),
            act: Some(Action::Plan { f: adventure_go, arg: (c.0 as i64) | ((t.0 as i64) << 16) }),
        });
    }
    Ok(o)
}

fn adventure_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, t) = (CardId((arg & 0xffff) as u16), PermId((arg >> 16) as u32));
    let Some(&(_, (generic, pips), kind, _)) = ADVENTURE.iter().find(|x| *x.0 == *g.db.get(c).name) else {
        return Ok(false);
    };
    if !g.player(p).hand.contains(&c) || !g.perm(t).on_bf || !can_pay(g, p, generic, pips, false) {
        return Ok(false);
    }
    pay(g, p, generic, pips, false)?;
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    on_cast(g, p, c)?;
    crate::glog!(g, "  {} casts the adventure of {} -> {}", player_name(g, p), g.db.get(c).name, name_of(g, t));
    apply_removal(g, Some(p), t, kind, None)?;
    if !g.player(p).adv_done.contains(&c) {
        g.player_mut(p).adv_done.push(c);
    }
    Ok(true)
}

/// pool_ai.special_options' card plays, in the Python's order: common.adventure_options, t2.evoke_options,
/// common.aristocrat_options, common.food_options, partials.miracle_options and partials.incubator_options
pub fn pool_card_options(g: &mut Game, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut o = adventure_options(g, p, post)?;
    // PORT(phase 6): o.extend(t2::evoke_options(g, p, post)?) — t2.evoke_options goes here
    o.extend(aristocrat_options(g, p, post)?);
    o.extend(food_options(g, p, post)?);
    // PORT(M5, partials): o.extend(partials::miracle_options(g, p, post)?); then
    // o.extend(partials::incubator_options(g, p, post)?) — partials' agent ports them
    Ok(o)
}

// ======================================================== creature tutors (t1.tutor_named follows their chains)
/// common.TUTOR_PRED: what the creature tutors can find (Recruiter -> Spellseeker -> Reversal). Their enter
/// triggers (t1.tutor_named) wait for phase 6.
pub fn tutor_pred(name: &str) -> Option<fn(&CardDef) -> bool> {
    match name {
        "Spellseeker" => Some(|c| (c.instant || c.sorcery) && c.cmc <= 2),
        "Recruiter of the Guard" => Some(|c| c.creature && c.tgh <= 2),
        "Goblin Matron" => Some(|c| c.has_subtype("goblin")),
        _ => None,
    }
}

// ======================================================== threat values for key engines (CI.PVAL)
const PVAL: &[(&str, f64)] = &[
    ("Aurelia, the Warleader", 7.0),
    ("Isshin, Two Heavens as One", 6.0),
    ("Winota, Joiner of Forces", 8.0),
    ("Kaalia of the Vast", 7.0),
    ("Krenko, Mob Boss", 7.0),
    ("Yuriko, the Tiger's Shadow", 6.0),
    ("Marwyn, the Nurturer", 5.0),
    ("Lathril, Blade of the Elves", 5.0),
    ("Korvold, Fae-Cursed King", 7.0),
    ("Teysa Karlov", 6.0),
    ("Meren of Clan Nel Toth", 6.0),
    ("Kinnan, Bonder Prodigy", 8.0),
    ("Urza, Lord High Artificer", 8.0),
    ("Yawgmoth, Thran Physician", 8.0),
    ("Zur the Enchanter", 7.0),
    ("Chulane, Teller of Tales", 7.0),
    ("Grand Arbiter Augustin IV", 7.0),
    ("Light-Paws, Emperor's Voice", 5.0),
    ("Sythis, Harvest's Hand", 5.0),
    ("Brago, King Eternal", 6.0),
    ("Tatyova, Benthic Druid", 5.0),
    ("Atraxa, Praetors' Voice", 7.0),
    ("Tergrid, God of Fright // Tergrid's Lantern", 7.0),
    ("Prosper, Tome-Bound", 6.0),
    ("Craterhoof Behemoth", 8.0),
    ("Helm of the Host", 6.0),
    ("Aluren", 7.0),
    ("Necropotence", 6.0),
    ("Doubling Season", 6.0),
    ("Grave Pact", 6.0),
    ("Dictate of Erebos", 6.0),
    ("Impact Tremors", 5.0),
    ("Purphoros, God of the Forge", 5.0),
    ("Sanguine Bond", 6.0),
    ("Exquisite Blood", 6.0),
    ("Isochron Scepter", 5.0),
    ("Kiki-Jiki, Mirror Breaker", 7.0),
    ("Zealous Conscripts", 5.0),
    ("Felidar Guardian", 5.0),
    ("Phyrexian Altar", 5.0),
    ("Thornbite Staff", 4.0),
    ("Walking Ballista", 4.0),
    ("Consecrated Sphinx", 7.0),
    ("Hullbreaker Horror", 7.0),
    ("Seedborn Muse", 5.0),
    ("Combat Celebrant", 5.0),
    ("Hellrider", 5.0),
    ("Brutal Hordechief", 5.0),
    ("Hero of Bladehold", 6.0),
    ("Terror of the Peaks", 7.0),
    ("Scourge of Valkas", 5.0),
    ("Drakuseth, Maw of Flames", 7.0),
    ("Avenger of Zendikar", 6.0),
    ("Scute Swarm", 5.0),
    ("Omnath, Locus of Rage", 6.0),
    ("Titania, Protector of Argoth", 5.0),
    ("Rule of Law", 6.0),
    ("Deafening Silence", 5.0),
    ("Drannith Magistrate", 6.0),
    ("Grand Abolisher", 5.0),
    ("Trinisphere", 6.0),
    ("Solitary Confinement", 6.0),
    ("Notion Thief", 5.0),
    ("Mayhem Devil", 5.0),
    ("Rhystic Study", 5.0),
    ("Smothering Tithe", 5.0),
    ("Esper Sentinel", 3.0),
    ("Mystic Remora", 4.0),
    ("Sylvan Library", 4.0),
    ("Phyrexian Arena", 4.0),
    ("Bolas's Citadel", 6.0),
    ("The One Ring", 6.0),
    ("Gaea's Cradle", 0.0),
    ("Beastmaster Ascension", 5.0),
    ("Elesh Norn, Grand Cenobite", 8.0),
];

// ======================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    // Auras
    for (name, spec) in AURAS {
        aura(r, db, name, spec)?;
    }
    r.card(db, "Celestial Mantle")?.combat_damage = Some(mantle);
    r.card(db, "Spirit Link")?.combat_damage = Some(spirit_link);
    r.card(db, "Snake Umbra")?.combat_damage = Some(snake_umbra);
    // the Oblivion Ring family: the exile is a trigger, the return an effect ending
    for (name, f) in [
        ("Oblivion Ring", oring_any as crate::hooks::EtbFn),
        ("Banishing Light", oring_any),
        ("Cast Out", oring_any),
        ("Journey to Nowhere", journey),
        ("Detention Sphere", oring_any),
        ("Grasp of Fate", grasp),
    ] {
        let c = r.card(db, name)?;
        c.etb = Some(f);
        c.leaves = Some(oring_return);
        *c = c.at_once(Event::Leaves);
    }
    r.card(db, "Esper Sentinel")?.cast = Some(sentinel);
    // cost reducers
    r.card(db, "Danitha Capashen, Paragon")?.cost = Some(danitha);
    r.card(db, "Hero of Iroas")?.cost = Some(hero_of_iroas);
    r.card(db, "Starfield Mystic")?.cost = Some(starfield);
    r.card(db, "Pearl Medallion")?.cost = Some(pearl);
    // death triggers
    r.card(db, "Cruel Celebrant")?.dies = Some(celebrant);
    r.card(db, "Grim Haruspex")?.dies = Some(haruspex);
    r.card(db, "Midnight Reaper")?.dies = Some(reaper);
    r.card(db, "Sifter of Skulls")?.dies = Some(sifter);
    r.card(db, "Pawn of Ulamog")?.dies = Some(pawn);
    r.card(db, "Requiem Angel")?.dies = Some(requiem);
    r.card(db, "Dark Prophecy")?.dies = Some(prophecy);
    r.card(db, "Vindictive Vampire")?.dies = Some(vindictive);
    r.card(db, "Syr Konrad, the Grim")?.dies = Some(konrad);
    r.card(db, "Morbid Opportunist")?.dies = Some(morbid);
    r.card(db, "Carrier Thrall")?.self_dies = Some(thrall);
    r.card(db, "Mirkwood Bats")?.sacrifice = Some(bats_sac);
    let c = r.card(db, "Revel in Riches")?;
    c.dies = Some(revel);
    c.upkeep = Some(revel_win);
    r.card(db, "Priest of Forgotten Gods")?.options = Some(priest);
    // staples
    r.card(db, "Mind Stone")?.options = Some(mind_stone);
    let c = r.card(db, "Rest in Peace")?;
    c.etb = Some(rip);
    c.sba = Some(rip_sweep);
    c.no_graveyard = Some(rip_nogy);
    r.card(db, "Hero of Bladehold")?.attack = Some(hero);
    r.card(db, "Ophiomancer")?.upkeep = Some(ophiomancer);
    r.card(db, "Pernicious Deed")?.options = Some(deed);
    for &(name, v) in PVAL {
        r.card(db, name)?.pval = Some(v);
    }
    Ok(())
}

/// PORT(M5): common.saga_step: p's Urza's Sagas get a lore counter and their chapter abilities. Only Urza's Saga
/// uses it, which no pilot deck runs, and its Construct's size is t5's TOKEN_PT entry (phase 6).
pub fn saga_step(_g: &mut crate::state::Game, _p: crate::ids::PlayerId) -> crate::flow::Res {
    Ok(())
}
