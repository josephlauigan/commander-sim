//! Python's `cards/impl/common.py`: card code shared by many pool decks, and the shared machinery the engine calls by
//! name (through `cardcode.rs`): Auras (`aura`, `auras_on`, `attached_bonus`, `aura_fall`, umbra armor), the Oblivion
//! Ring family, sacrifice outlets and death payoffs (`aristocrat_options`, `sac_in_response`), the planeswalker
//! framework (`walker`, `ult_pressure`), Food, adventures and the fixed threat values (`CI.PVAL`).
//!
//! Ported for M5: everything the pilot decks (Sauron and Tier 1) play, and the table-driven entries of the same
//! families (every Aura, the O-Ring family, the death-trigger creatures, the sacrifice outlets, the uncounterable
//! grants). Phase 6: the rest (pillowfort taxes, stax pieces, the four planeswalkers, Urza's Saga, Walking Ballista,
//! graveyard hate, proliferate, the dynamic mana cards ...), registered by `register_phase6`.

use crate::cards::{CardDb, CardDef, Colors, Types};
use crate::engine::cast::{cast_card, castable, on_cast};
use crate::engine::hooks::{fire_trigger, hooked, total_trigger_copies};
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, ability_window_card, counter_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{epow, etgh, has_type, melira, once_per_turn, protected_from, pval, stopped, untargetable};
use crate::engine::zones::{
    Enter, Tokens, Zone, add_treasure, die, draw, edict, enter, exile_perm, leave, make_tokens, max_by, mill, min_by,
    sac_worth, searchable, to_zone_card,
};
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::{Ctx, DataKey, Game, TurnStamp, Val};
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
        for f in &g.registry.creature_pt {
            let (a, b) = f(g, m);
            dp += a;
            dt += b;
        }
        let b = partials_bonus(g, m);
        dp += b;
        dt += b;
    }
    (dp, dt)
}

/// `IP.coat_bonus(g, m) + IP.lineage_bonus(g, m) + IP.bestow_bonus(g, m)` (Coat of Arms, Lord of Lineage, bestowed
/// Eidolons)
fn partials_bonus(g: &Game, m: PermId) -> i32 {
    super::partials::coat_bonus(g, m) + super::partials::lineage_bonus(g, m) + super::partials::bestow_bonus(g, m)
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
            // Bug fix (phase 6): Python's `died = m not in m.owner.perms` is always true here, since aura_fall runs
            // once m has left, so a 'host_dies' Aura (Angelic Destiny) went back to hand however its creature left
            // (bounced, exiled ...). It returns only when the creature died: engine.die records it in `g.dying`.
            let died = g.dying == Some(m);
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
    ("Angelic Destiny", AuraSpec { pow: 4, tgh: 4, kws: &["flying", "first strike"], back: Some("host_dies"), ..AURA }),
    ("Battle Mastery", AuraSpec { kws: &["double strike"], ..AURA }),
    (
        "Cartouche of Solidarity",
        AuraSpec { pow: 1, tgh: 1, kws: &["first strike"], on_etb: Some(cartouche_etb), ..AURA },
    ),
    // Partial: +3/+3; the life-doubling combat trigger is modeled in a hook
    ("Celestial Mantle", AuraSpec { pow: 3, tgh: 3, ..AURA }),
    (
        "Daybreak Coronet",
        AuraSpec { pow: 3, tgh: 3, kws: &["first strike", "vigilance", "lifelink"], host_ok: Some(coronet_ok), ..AURA },
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
    if g.perm(src).attached == Some(a) && trigger_window(g, owner(g, src), Some(src), "double your life total", None)? {
        let o = owner(g, src);
        let life = g.player(o).life;
        gain(g, o, life)?;
    }
    Ok(())
}

/// Spirit Link: gain that much life
fn spirit_link(g: &mut Game, src: Src, _p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    if g.perm(src).attached == Some(a)
        && trigger_window(g, owner(g, src), Some(src), &format!("gain {dmg} life"), None)?
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
/// common.oring_exile: exile the best permanent(s) opponents control until src leaves the battlefield (pred: which
/// opposing permanents it may take)
pub fn oring_exile(
    g: &mut Game,
    src: PermId,
    p: PlayerId,
    pred: impl Fn(&Game, PermId) -> bool,
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
pub fn oring_return(g: &mut Game, src: PermId) -> Res {
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

/// the O-Ring family's leaves hook (common.oring_return)
fn oring_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    oring_return(g, src)
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
    g.player(p).perms.iter().filter(|&&x| def(g, x).is_some_and(|d| d.tag(Tag::Bartist) || d.tag(Tag::Drain))).count()
        as f64
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
    let mut drain_per =
        pl.perms.iter().filter(|&&x| g.perm(x).cd.is_some()).map(|&x| lookup(&DEATH_DRAIN, card_name(g, x))).psum()
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
        let spec =
            Tokens { fly: true, color: Some(Colors::from_letters("W")), types: vec!["spirit"], ..Tokens::new(1, 1) };
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
    crate::glog!(
        g,
        "    {}'s graveyard is exiled{}",
        player_name(g, q),
        by.map_or(String::new(), |b| format!(" by {b}"))
    );
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
        let spec =
            Tokens { dt: true, color: Some(Colors::from_letters("B")), types: vec!["snake"], ..Tokens::new(1, 1) };
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
        let theirs =
            g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).filter(|&m| hit(m)).map(|m| pval(g, m)).psum();
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

// ======================================================== uncounterable spells
/// Destiny Spinner (Approximate): your creature and enchantment spells can't be countered (the land animation is
/// rules2's)
fn spinner(g: &Game, src: Src, caster: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    caster == owner(g, src) && (d.creature || d.types.has(Types::ENCHANTMENT))
}

/// Allosaurus Shepherd: your green spells can't be countered (rules.py registers the same rule again)
fn shepherd(g: &Game, src: Src, caster: PlayerId, c: CardId) -> bool {
    caster == owner(g, src) && g.db.get(c).pips.contains('G')
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

pub(crate) fn round_stamp(g: &Game, p: PlayerId) -> TurnStamp {
    TurnStamp { round: g.round, active: Some(p) }
}

/// common._uses: loyalty abilities src used this round
pub(crate) fn uses(g: &Game, p: PlayerId, src: PermId) -> i64 {
    let x = g.perm(src);
    if x.loyalty_used != Some(round_stamp(g, p)) {
        return 0;
    }
    x.data.get(DataKey::LoyaltyN).map_or(1, Val::int)
}

/// common._allowed: Oath of Teferi lets each planeswalker use two abilities a turn
pub(crate) fn allowed(g: &Game, p: PlayerId) -> i64 {
    let oath = g.player(p).perms.iter().any(|&m| card_name(g, m) == "Oath of Teferi" && !g.perm(m).phased);
    if oath { 2 } else { 1 }
}

/// Carth the Lion: each loyalty ability costs an extra [+1] (CI.total(g, 'loyalty_extra', p) if g.hooks else 0)
fn loyalty_extra(g: &Game, p: PlayerId) -> i32 {
    if g.hooks.is_empty() { 0 } else { crate::engine::hooks::total_count(g, Event::LoyaltyExtra, p) }
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
    o.extend(super::t2::evoke_options(g, p, post)?);
    o.extend(aristocrat_options(g, p, post)?);
    o.extend(food_options(g, p, post)?);
    o.extend(super::partials::miracle_options(g, p, post)?);
    o.extend(super::partials::incubator_options(g, p, post)?);
    Ok(o)
}

// ======================================================== creature tutors (t1.tutor_named follows their chains)
/// common.TUTOR_PRED: what the creature tutors can find (Recruiter -> Spellseeker -> Reversal), for their enter
/// triggers (`tutor_etb`)
pub fn tutor_pred(name: &str) -> Option<fn(&CardDef) -> bool> {
    match name {
        "Spellseeker" => Some(|c| (c.instant || c.sorcery) && c.cmc <= 2),
        "Recruiter of the Guard" => Some(|c| c.creature && c.tgh <= 2),
        "Goblin Matron" => Some(|c| c.has_subtype("goblin")),
        // t5.py's (_tutor_etb('Trophy Mage', ...), _tutor_etb('Tribute Mage', ...)): t5 registers `tutor_etb` for them
        "Trophy Mage" => Some(|c| c.types.has(Types::ARTIFACT) && c.cmc == 3),
        "Tribute Mage" => Some(|c| c.types.has(Types::ARTIFACT) && c.cmc == 2),
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

// ======================================================== phase 6: the rest of common.py
// ---- small helpers
/// engine.casts_this_turn(g, p, pred): the spells p cast this turn that match pred
pub fn casts_matching(g: &Game, p: PlayerId, pred: impl Fn(&CardDef) -> bool) -> i32 {
    match &g.player(p).turn_casts {
        Some((st, cs)) if *st == g.turn_stamp() => cs.iter().filter(|&&c| pred(g.db.get(c))).count() as i32,
        _ => 0,
    }
}

/// common.noncre: a noncreature, nonland card
fn noncre(d: &CardDef) -> bool {
    !d.creature && !d.land
}

/// cardimpl._eot: +dp/+dt until end of turn
pub fn eot(g: &mut Game, m: PermId, dp: i32, dt: i32) {
    let x = &mut g.perm_mut(m).eot_pt;
    *x = (x.0 + dp, x.1 + dt);
}

/// m is a card with type t (`m.cd is not None and t in m.cd.types`)
fn card_is(g: &Game, m: PermId, t: Types) -> bool {
    def(g, m).is_some_and(|d| d.types.has(t))
}

/// `(c.bomb or c.pow)`
fn bomb_or_pow(d: &CardDef) -> i32 {
    if d.bomb != 0 { d.bomb } else { d.pow }
}

/// remove the first c from a list of cards (Python's `list.remove`)
fn remove_first(v: &mut Vec<CardId>, c: CardId) -> bool {
    match v.iter().position(|&x| x == c) {
        Some(i) => {
            v.remove(i);
            true
        }
        None => false,
    }
}

fn opt_ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

fn opt_plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

// ======================================================== staples: spells
/// Fact or Fiction (Approximate): the opponent's split is modeled as: you keep the best two of five. (rules.py's
/// split replaces it.)
fn fof(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut top = vec![];
    for _ in 0..g.player(p).library.len().min(5) {
        top.push(g.player_mut(p).library.pop().unwrap());
    }
    let w: Vec<f64> = top.iter().map(|&x| -card_worth(g, p, x, false)).collect();
    let mut idx: Vec<usize> = (0..top.len()).collect();
    idx.sort_by(|&a, &b| w[a].partial_cmp(&w[b]).unwrap_or(std::cmp::Ordering::Equal)); // stable, as Python's sort
    let top: Vec<CardId> = idx.into_iter().map(|i| top[i]).collect();
    let pl = g.player_mut(p);
    for &x in top.iter().take(2) {
        pl.hand.push(x);
        pl.seen.insert(x);
    }
    pl.gy.extend(top.iter().skip(2));
    Ok("gy")
}

fn prio_45(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    45
}

/// Council's Judgment (Approximate): exiles the best opposing nonland permanent (it doesn't target: hexproof
/// ignored); the vote is not modeled. (rules.py's vote replaces it.)
fn judgment(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let cands: Vec<PermId> =
        g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).filter(|&m| !g.perm(m).phased).collect();
    if let Some(m) = max_by(&cands, |m| pval(g, m)) {
        let o = owner(g, m);
        crate::glog!(g, "    {} ({}) is exiled by vote", name_of(g, m), player_name(g, o));
        let name = name_of(g, m);
        *g.player_mut(o).lost_names.entry(name).or_insert(0) += 1;
        exile_perm(g, m)?;
        check_state(g)?;
    }
    Ok("gy")
}

/// Sign in Blood (Full): you draw two and lose 2
fn sign_in_blood(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    draw(g, p, 2, false)?;
    lose_life(g, p, 2, Some(p), "other", None)?;
    Ok("gy")
}

// ---- Otawara, Soaring City: channel {3}{U} (less per legendary creature), discard: bounce
/// Full: land; channelled from hand to bounce a real threat
fn otawara(g: &mut Game, c: CardId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    let legends =
        g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && def(g, m).is_some_and(|d| d.tag(Tag::Leg))).count()
            as i64;
    let n = (3 - legends).max(0) as u32;
    if !can_pay(g, p, n, "U", false) {
        return Ok(vec![]);
    }
    let cands: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| {
            !untargetable(g, m)
                && !g.perm(m).phased
                && (g.is_creature(m)
                    || def(g, m).is_some_and(|d| {
                        d.types.has(Types::ARTIFACT)
                            || d.types.has(Types::ENCHANTMENT)
                            || d.types.has(Types::PLANESWALKER)
                    }))
        })
        .collect();
    let Some(t) = max_by(&cands, |m| pval(g, m)) else { return Ok(vec![]) };
    if pval(g, t) < 4.0 {
        return Ok(vec![]);
    }
    let arg = (c.0 as i64) | ((n as i64) << 16) | ((t.0 as i64) << 24);
    Ok(vec![opt_plan(pval(g, t) - 4.0, format!("Otawara -> {}", name_of(g, t)), otawara_go, arg)])
}

fn otawara_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, n, t) = (CardId((arg & 0xffff) as u16), ((arg >> 16) & 0xff) as u32, PermId((arg >> 24) as u32));
    if !g.player(p).hand.contains(&c) || !g.perm(t).on_bf || !can_pay(g, p, n, "U", false) {
        return Ok(false);
    }
    remove_first(&mut g.player_mut(p).hand, c);
    pay(g, p, n, "U", false)?;
    g.player_mut(p).gy.push(c);
    crate::glog!(g, "  {} channels Otawara", player_name(g, p));
    apply_removal(g, Some(p), t, "bounce", None)?;
    Ok(true)
}

// ---- Reflector Mage: bounce and the owner can't recast it until your next turn
fn reflector_cands(g: &Game, o: PlayerId) -> Vec<PermId> {
    g.opps(o)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&x| g.is_creature(x) && !untargetable(g, x))
        .collect()
}

/// Full
fn reflector(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if reflector_cands(g, o).is_empty() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "bounce a creature", Some(5.0))? {
        return Ok(());
    }
    let cands = reflector_cands(g, o);
    let Some(t) = max_by(&cands, |x| pval(g, x)) else { return Ok(()) };
    let (towner, name) = (owner(g, t), name_of(g, t));
    let cd = g.perm(src).cd;
    apply_removal(g, Some(o), t, "bounce", cd)?;
    if !on(g, t, towner) {
        let turns = g.player(o).turns + 1;
        g.player_mut(towner).locked_name = Some((name, turns, o));
    }
    Ok(())
}

fn reflector_lock(g: &Game, _src: Src, caster: PlayerId, c: CardId, _zone: Sym) -> bool {
    if let Some((name, turns, by)) = g.player(caster).locked_name
        && name == &*g.db.get(c).name
        && g.player(by).turns < turns
    {
        return false;
    }
    true
}

/// Spark Double (Approximate): enters as a copy of your best creature or planeswalker (+1 counter); goes to the
/// graveyard as Spark Double
fn spark(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || g.perm(src).data.truthy(DataKey::Copied) {
        return Ok(());
    }
    let o = owner(g, src);
    let cands: Vec<PermId> = g
        .player(o)
        .perms
        .iter()
        .copied()
        .filter(|&x| {
            x != src
                && g.perm(x).cd.is_some()
                && (g.is_creature(x) || card_is(g, x, Types::PLANESWALKER))
                && !g.perm(x).token
        })
        .collect();
    let Some(best) = max_by(&cands, |x| pval(g, x)) else { return Ok(()) };
    let best_cd = g.perm(best).cd.unwrap();
    let own_cd = g.perm(src).cd;
    leave(g, src)?;
    let n = enter(g, o, best_cd, Enter::default())?;
    let mut d = crate::state::PermData::default(); // `n.data = {'copied': True}`
    d.set(DataKey::Copied, Val::Bool(true));
    let creature = g.is_creature(n);
    let x = g.perm_mut(n);
    x.data = d;
    x.phys = own_cd;
    if creature {
        x.plus += 1;
    }
    if let Some(l) = x.loyalty {
        x.loyalty = Some(l + 1);
    }
    crate::glog!(g, "    Spark Double copies {}", g.db.get(best_cd).name);
    Ok(())
}

/// Frost Titan (Approximate): taps the best opposing creature on entry and attack (the no-untap is not enforced);
/// the targeting tax is ward {2}. (rules.py's Frost Titan replaces both hooks.)
fn frost(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, owner(g, src), Some(src), "tap a creature", None)? {
        frost_tap(g, src);
    }
    Ok(())
}

fn frost_atk(g: &mut Game, src: Src, _p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src) && trigger_window(g, owner(g, src), Some(src), "tap a creature", None)? {
        frost_tap(g, src);
    }
    Ok(vec![])
}

fn frost_tap(g: &mut Game, src: PermId) {
    let cands: Vec<PermId> = g
        .opps(owner(g, src))
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&x| g.is_creature(x) && !g.perm(x).phased && !untargetable(g, x))
        .collect();
    if let Some(t) = max_by(&cands, |x| pval(g, x)) {
        g.perm_mut(t).tapped = true;
        if g.perm(t).cd.is_some() {
            let turns = g.player(owner(g, t)).turns as i64 + 1;
            g.perm_mut(t).data.set(DataKey::Frozen, Val::Int(turns));
        }
    }
}

/// Dream Trawler (Approximate): draws on attack, +1/+0 per draw; the discard-for-hexproof is used by the protection
/// AI
fn trawler(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src) && trigger_window(g, p, Some(src), "draw a card", None)? {
        draw(g, p, 1, false)?;
    }
    Ok(vec![])
}

fn trawler_draw(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src) && trigger_window(g, p, Some(src), "gets +1/+0", Some(1.0))? && on(g, src, p) {
        eot(g, src, 1, 0);
    }
    Ok(())
}

// ---- rebound (Ephemerate)
/// t2.blink_value: how much re-entering is worth for p's permanent m. A copy of t2.py's (t2.rs is ported
/// separately): MERGE: call t2's.
fn blink_value(g: &Game, _p: PlayerId, m: PermId) -> f64 {
    let x = g.perm(m);
    let Some(cd) = x.cd else { return 0.0 };
    if x.token || x.is_cmd {
        return 0.0;
    }
    let mut v = crate::dsl::etb_value(g, cd) as f64;
    if g.registry.get(cd).is_some_and(|i| i.etb.is_some()) {
        v += 3.0;
    }
    if matches!(
        &*g.db.get(cd).name,
        "Oblivion Ring" | "Banishing Light" | "Detention Sphere" | "Cast Out" | "Journey to Nowhere"
    ) {
        v = 0.0;
    }
    v
}

/// mine.ETB_VALUE
const ETB_VALUE: [(Tag, f64); 9] = [
    (Tag::Atraxa, 8.0),
    (Tag::Archon, 6.0),
    (Tag::Rsd, 4.0),
    (Tag::Titan, 3.0),
    (Tag::Witness, 2.5),
    (Tag::Wall, 2.0),
    (Tag::Wurm, 4.0),
    (Tag::Bowmasters, 2.5),
    (Tag::Skate, 3.0),
];

/// mine.etb_value: what re-entering the battlefield is worth for permanent m. A copy of mine.py's (the Sephiroth
/// port has it): MERGE: call mine's, which also values Summon: Bahamut's restart (`bahamut_restart`, 0 here).
fn mine_etb_value(g: &Game, _p: PlayerId, m: PermId) -> f64 {
    let x = g.perm(m);
    let Some(cd) = x.cd else { return 0.0 };
    let d = g.db.get(cd);
    let mut v = ETB_VALUE.iter().filter(|e| d.tag(e.0)).map(|e| e.1).psum();
    if d.perm
        && let Some(n) = d.tags.int(Tag::Draw)
    {
        v += 1.5 * n as f64;
    }
    if g.registry.get(cd).is_some_and(|i| i.etb.is_some()) {
        v = v.max(2.0);
    }
    if d.tag(Tag::Bahamut) {
        v = 0.0; // MERGE: mine.bahamut_restart(g, p, m)
    }
    if let (Some(s), Some(l)) = (d.start_loyalty, x.loyalty) {
        v = v.max(0.6 * (s - l) as f64);
    }
    v -= 0.5 * x.plus.max(0) as f64;
    if !g.auras.is_empty() && g.is_creature(m) {
        v -= 2.0 * auras_on(g, m).len() as f64;
    }
    v
}

/// mine.blink_worth: what blinking p's creature m is worth
fn blink_worth(g: &Game, p: PlayerId, m: PermId) -> f64 {
    if def(g, m).is_some_and(|d| d.tag(Tag::Bahamut)) {
        return mine_etb_value(g, p, m);
    }
    blink_value(g, p, m).max(mine_etb_value(g, p, m))
}

/// t2.blink: exile m and return it under its owner's control (ETBs again, untapped, summoning sick). A copy of
/// t2.py's without its blink_depth guard (a rebound blinks once): MERGE: call t2's.
fn blink(g: &mut Game, _p: PlayerId, m: PermId) -> Res<Option<PermId>> {
    let x = g.perm(m);
    if x.token || x.cd.is_none() || !on(g, m, x.owner) {
        return Ok(None);
    }
    let (cd, orig, cmd) = (x.cd.unwrap(), x.orig, x.is_cmd);
    leave(g, m)?;
    let n = enter(g, orig, cd, Enter { orig: Some(orig), ..Enter::default() })?;
    g.perm_mut(n).is_cmd = cmd;
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::ExiledFromBf, Call::Leaves { m })?;
    }
    crate::glog!(g, "    {} is blinked", g.db.get(cd).name);
    Ok(Some(n))
}

/// common._ephemerate_rebound: cast it again from exile for free (a real cast) on the best enter-effect creature,
/// then to the graveyard; with no creature to target it isn't cast and stays in exile
fn ephemerate_rebound(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    // HUMAN(phase 9): play.cards.ephemerate_rebound (the person picks)
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).token && !g.perm(m).phased)
        .collect();
    let Some(t) = max_by(&cands, |m| blink_worth(g, p, m)) else { return Ok(()) };
    let pl = g.player_mut(p);
    remove_first(&mut pl.exile, c);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    on_cast(g, p, c)?;
    crate::glog!(g, "  {} casts Ephemerate from exile (rebound) on {}", player_name(g, p), name_of(g, t));
    if !g.over && counter_window(g, p, c, 3.0, vec![])? && on(g, t, p) && blink_worth(g, p, t) > 0.0 {
        blink(g, p, t)?;
    }
    g.player_mut(p).gy.push(c);
    Ok(())
}

// ======================================================== graveyard hate
/// common.gy_worth: how much it's worth to owner to exile q's graveyard (reanimation targets, flashback, escape,
/// recursion)
pub fn gy_worth(g: &Game, owner_: PlayerId, q: PlayerId) -> f64 {
    if q == owner_ {
        return -1.0;
    }
    let mut v = 0.0;
    for &c in &g.player(q).gy {
        let d = g.db.get(c);
        if d.creature {
            v += (bomb_or_pow(d) - 3).max(0) as f64 * 1.2;
        }
        if d.tag(Tag::Fb)
            || matches!(&*d.name, "Uro, Titan of Nature's Wrath" | "Life from the Loam" | "Bloodghast" | "Gravecrawler")
        {
            v += 2.0;
        }
    }
    if g.player(q).perms.iter().any(|&m| {
        matches!(card_name(g, m), "Meren of Clan Nel Toth" | "Sheoldred, Whispering One" | "Syr Konrad, the Grim")
    }) {
        v += 3.0;
    }
    if g.player(q).key == "seph" {
        v += 3.0;
    }
    if g.player(q).hand.iter().any(|&c| g.db.get(c).tag(Tag::Rean))
        || g.player(q).gy.iter().any(|&c| g.db.get(c).tag(Tag::Rean))
    {
        v += 3.0;
    }
    v
}

/// the opponent whose graveyard is worth the most to exile (`max(g.opps(p), key=gy_worth, default=None)`)
fn worst_gy(g: &Game, p: PlayerId) -> Option<PlayerId> {
    let os = opps(g, p);
    max_by(&os, |q| gy_worth(g, p, q))
}

/// Bojuka Bog (Full): enters tapped; exiles the most dangerous graveyard
fn bog(g: &mut Game, p: PlayerId, _l: LandId) -> Res {
    if let Some(q) = worst_gy(g, p)
        && gy_worth(g, p, q) > 0.0
    {
        exile_gy(g, q, Some("Bojuka Bog"));
    }
    Ok(())
}

/// Dauthi Voidwalker (Approximate): opponents' cards going to the graveyard are exiled with void counters
fn dauthi_sweep(g: &mut Game, src: Src) -> Res {
    let o = owner(g, src);
    for q in opps(g, o) {
        if !g.player(q).gy.is_empty() {
            let gy = std::mem::take(&mut g.player_mut(q).gy);
            let mut void = match g.perm(src).data.get(DataKey::Void) {
                Some(Val::List(v)) => v.clone(),
                _ => vec![],
            };
            void.extend(gy.iter().map(|&c| Val::List(vec![Val::Card(c), Val::Player(q)])));
            g.perm_mut(src).data.set(DataKey::Void, Val::List(void));
            g.player_mut(q).exile.extend(gy);
        }
    }
    Ok(())
}

fn dauthi_void(g: &Game, src: PermId) -> Vec<(CardId, PlayerId)> {
    let Some(Val::List(v)) = g.perm(src).data.get(DataKey::Void) else { return vec![] };
    v.iter()
        .filter_map(|e| match e {
            Val::List(x) => match (&x[0], &x[1]) {
                (Val::Card(c), Val::Player(q)) => Some((*c, *q)),
                _ => None,
            },
            _ => None,
        })
        .filter(|&(c, q)| g.player(q).exile.contains(&c) && !g.db.get(c).land)
        .collect()
}

/// sacrifice it: play the best exiled card free
fn dauthi_play(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.perm(src).tapped || g.perm(src).sick {
        return Ok(vec![]);
    }
    let void = dauthi_void(g, src);
    let Some((c, q)) =
        super::partials::first_max(&void, |(c, _)| card_worth(g, p, c, false) + g.db.get(c).cmc as f64 * 5.0)
    else {
        return Ok(vec![]);
    };
    let cmc = g.db.get(c).cmc;
    if cmc < 4 {
        return Ok(vec![]);
    }
    let label = format!("Dauthi Voidwalker plays {}", g.db.get(c).name);
    Ok(vec![opt_ability(2.0 + cmc as f64 * 0.6, label, src, dauthi_go, (c.0 as i64) | ((q.0 as i64) << 16))])
}

fn dauthi_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, q) = (CardId((arg & 0xffff) as u16), PlayerId((arg >> 16) as u8));
    if !on(g, src, p) || !g.player(q).exile.contains(&c) {
        return Ok(false);
    }
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    let label = format!("play {}", g.db.get(c).name);
    if !ability_window_card(g, p, cd, &label, Some(5.0), None)? || !g.player(q).exile.contains(&c) {
        return Ok(true);
    }
    remove_first(&mut g.player_mut(q).exile, c);
    crate::glog!(g, "  Dauthi Voidwalker: {} plays {} free", player_name(g, p), g.db.get(c).name);
    if g.db.get(c).perm {
        enter(g, p, c, Enter { orig: Some(q), ..Enter::default() })?;
    } else {
        g.player_mut(p).hand.push(c);
        cast_card(g, p, c, "hand", Ctx::default())?;
    }
    Ok(true)
}

/// Tormod's Crypt (Full), Soul-Guide Lantern (Approximate): exiles a graveyard in response to reanimation, or
/// proactively when it holds real threats. Both cost {0} and are sacrificed (common._gy_hate_card).
fn gy_hate(g: &mut Game, src: Src, _reanimator: PlayerId, gy_owner: PlayerId) -> Res<bool> {
    let o = owner(g, src);
    if g.perm(src).tapped || !can_pay(g, o, 0, "", false) {
        return Ok(false);
    }
    pay(g, o, 0, "", false)?;
    die(g, src, "sac")?;
    let name = card_name(g, src).to_string();
    exile_gy(g, gy_owner, Some(&name));
    Ok(true)
}

fn gy_hate_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).tapped || !can_pay(g, p, 0, "", false) {
        return Ok(vec![]);
    }
    let Some(q) = worst_gy(g, p) else { return Ok(vec![]) };
    let w = gy_worth(g, p, q);
    if w < 8.0 {
        return Ok(vec![]);
    }
    let label = format!("{} on {}", card_name(g, src), player_name(g, q));
    Ok(vec![opt_ability(1.0 + w / 4.0, label, src, gy_hate_go, q.0 as i64)])
}

fn gy_hate_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let q = PlayerId(arg as u8);
    if !on(g, src, p) || !can_pay(g, p, 0, "", false) {
        return Ok(false);
    }
    pay(g, p, 0, "", false)?;
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    let name = g.db.get(cd).name.to_string();
    if ability_window_card(g, p, cd, &format!("exile {}'s graveyard", player_name(g, q)), None, None)? {
        exile_gy(g, q, Some(&name));
    }
    Ok(true)
}

/// cardimpl.gy_response: an opponent answers a reanimation spell by exiling the graveyard (Tormod's Crypt,
/// Soul-Guide Lantern)
pub fn gy_response(g: &mut Game, reanimator: PlayerId, value: f64, src_player: PlayerId) -> Res<bool> {
    if value < 5.0 {
        return Ok(false);
    }
    for (src, imp) in hooked(g, Event::GyHate) {
        // HUMAN(phase 9): a person activates their own
        if owner(g, src) != reanimator && (imp.gy_hate.unwrap())(g, src, reanimator, src_player)? {
            return Ok(true);
        }
    }
    Ok(false)
}

// ======================================================== pillowfort: attack taxes and caps
/// common._tax: mana per creature attacking src's controller (attack_tax)
fn pillowfort_tax(g: &Game, src: Src, attacker: PlayerId, d: PlayerId) -> Option<i32> {
    let o = owner(g, src);
    if d != o || attacker == o {
        return Some(0);
    }
    Some(match card_name(g, src) {
        "Ghostly Prison" | "Propaganda" | "Windborn Muse" | "Elephant Grass" => 2,
        "Baird, Steward of Argive" => 1,
        // Approximate: {W/P} per attacker read as {1} (rules2.py's Norn's Annex replaces it)
        "Norn's Annex" => 1,
        "Sphere of Safety" => g.player(o).perms.iter().filter(|&&m| card_is(g, m, Types::ENCHANTMENT)).count() as i32,
        // Approximate: attack tax while untapped; the blocking tax while it attacks is ignored
        "Archangel of Tithes" => !g.perm(src).tapped as i32,
        _ => 0,
    })
}

fn crawlspace(g: &Game, src: Src, _attacker: PlayerId, d: PlayerId) -> Option<i32> {
    if d == owner(g, src) { Some(2) } else { None }
}

/// Silent Arbiter (Approximate): one attacker per combat; the one-blocker limit is not modeled
fn arbiter(_g: &Game, _src: Src, _attacker: PlayerId, _d: PlayerId) -> Option<i32> {
    Some(1)
}

/// Elephant Grass (Approximate): its cumulative upkeep is paid while it matters (three turns)
fn grass(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "cumulative upkeep {1}", None)? {
        return Ok(());
    }
    let age = g.perm(src).data.get(DataKey::Age).map_or(0, Val::int) + 1;
    g.perm_mut(src).data.set(DataKey::Age, Val::Int(age));
    if age <= 3 && can_pay(g, p, age as u32, "", false) {
        pay(g, p, age as u32, "", false)?;
    } else {
        die(g, src, "sac")?;
    }
    Ok(())
}

// ======================================================== shroud / hexproof grants
/// common.IC_auras
fn ic_auras(g: &Game, m: PermId) -> bool {
    !g.auras.is_empty() && !auras_on(g, m).is_empty()
}

/// Greater Auramancy: your other enchantments and enchanted creatures have shroud
fn auramancy(g: &Game, src: Src, m: PermId, k: Sym) -> bool {
    k == "shroud" && owner(g, m) == owner(g, src) && m != src && (card_is(g, m, Types::ENCHANTMENT) || ic_auras(g, m))
}

/// Sterling Grove: your other enchantments have shroud
fn grove_kw(g: &Game, src: Src, m: PermId, k: Sym) -> bool {
    k == "shroud" && owner(g, m) == owner(g, src) && m != src && card_is(g, m, Types::ENCHANTMENT)
}

/// Privileged Position: your other permanents have hexproof
fn privileged(g: &Game, src: Src, m: PermId, k: Sym) -> bool {
    k == "hexproof" && owner(g, m) == owner(g, src) && m != src
}

/// Sterling Grove (Full): sacrificed late to put an enchantment on top
fn grove(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(true) || !can_pay(g, p, 1, "", false) {
        return Ok(vec![]);
    }
    if !g.player(p).library.iter().any(|&c| g.db.get(c).types.has(Types::ENCHANTMENT)) {
        return Ok(vec![]);
    }
    let u = if g.player(p).hand.len() > 2 { 0.5 } else { 2.0 };
    Ok(vec![opt_ability(u, "Sterling Grove tutor".into(), src, grove_go, 0)])
}

fn grove_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on(g, src, p) || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    if !ability_window_card(g, p, cd, "an enchantment on top", None, None)? {
        return Ok(true);
    }
    let cs: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| g.db.get(c).types.has(Types::ENCHANTMENT)).collect();
    let Some(c) = max_by(&cs, |c| card_worth(g, p, c, false)) else {
        shuffle_library(g, p); // (Aven Mindcensor: maybe none in the top four)
        return Ok(true);
    };
    remove_first(&mut g.player_mut(p).library, c);
    shuffle_library(g, p);
    g.player_mut(p).library.push(c);
    Ok(true)
}

// ======================================================== fog (Spore Frog)
/// Spore Frog (Full): sacrificed to fog a big attack
fn spore_frog(
    g: &mut Game,
    src: Src,
    _p: PlayerId,
    atk: &[PermId],
    d: PlayerId,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if d != owner(g, src) || g.fog == Some(g.turn_stamp()) {
        return Ok(());
    }
    let incoming: i32 = atk.iter().filter(|&&a| !assign.iter().any(|x| x.0 == a)).map(|&a| epow(g, a)).sum();
    if incoming as f64 >= (6.0f64).max(g.player(d).life as f64 * 0.35) {
        die(g, src, "sac")?;
        g.fog = Some(g.turn_stamp());
        crate::glog!(g, "    Spore Frog prevents all combat damage this turn");
    }
    Ok(())
}

// ======================================================== planeswalkers
/// common.always(x) for the walkers' abilities
fn always_2_5(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(2.5)
}

fn always_3(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(3.0)
}

fn always_9(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(9.0)
}

fn always_10(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(10.0)
}

// ---- Elspeth, Sun's Champion (Approximate: tokens, the power-4 sweep when it pays, the emblem as a lasting anthem)
fn elspeth_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let spec = Tokens { color: Some(Colors::from_letters("W")), types: vec!["soldier"], ..Tokens::new(3, 1) };
    make_tokens(g, p, spec)?;
    Ok(())
}

fn elspeth_sweep_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let big = |m: PermId| g.is_creature(m) && epow(g, m) >= 4;
    let o = g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).filter(|&m| big(m)).map(|m| pval(g, m)).psum();
    let m = g.player(p).perms.iter().copied().filter(|&m| big(m)).map(|m| pval(g, m)).psum();
    if o - m >= 6.0 { Some(o - m - 3.0) } else { None }
}

fn elspeth_sweep(g: &mut Game, _p: PlayerId, _src: PermId) -> Res {
    for q in 0..g.players.len() {
        for m in g.players[q].perms.clone() {
            if g.is_creature(m) && epow(g, m) >= 4 {
                die(g, m, "destroy")?;
            }
        }
    }
    Ok(())
}

/// galadriel.elspeth_emblem (CI.elspeth_emblem): creatures p controls get +2/+2 and have flying. A copy of
/// galadriel.py's three lines: MERGE: call galadriel's if it is ported.
fn elspeth_emblem(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    g.player_mut(p).elspeth_emblem = true;
    g.dsl_on = true; // flying is read through the keyword checks
    crate::glog!(g, "    {} gets an emblem: creatures they control get +2/+2 and have flying", player_name(g, p));
    Ok(())
}

static ELSPETH: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "three Soldiers", val: always_3, eff: elspeth_plus },
    WalkerAb { delta: -3, label: "destroy power 4+", val: elspeth_sweep_val, eff: elspeth_sweep },
    WalkerAb { delta: -7, label: "emblem", val: always_9, eff: elspeth_emblem },
];

// ---- Teferi, Hero of Dominaria (Approximate: +1 draws and untaps two lands; -3 tucks a threat)
pub fn teferi_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    draw(g, p, 1, false)?;
    for l in g.player(p).lands.iter().copied().take(2).collect::<Vec<_>>() {
        g.land_mut(l).tapped = false;
    }
    Ok(())
}

pub fn teferi_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    best_opp_nonland(g, p, |_| true).map(|t| pval(g, t)).filter(|&v| v >= 5.0).map(|v| v - 2.5)
}

pub fn teferi_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = best_opp_nonland(g, p, |_| true) {
        apply_removal(g, Some(p), t, "tuck", None)?;
    }
    Ok(())
}

/// rules.ult('Teferi, Hero of Dominaria', -8, 'emblem'): rules.give_emblem(p, 'teferi') (whenever you draw, exile an
/// opposing permanent: rules.emblem_draw). rules.py appends it to common's list.
pub fn teferi_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let e = &mut g.player_mut(p).emblems;
    if !e.contains(&"teferi") {
        e.push("teferi");
    }
    Ok(())
}

static TEFERI: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "draw, untap two lands", val: always_3, eff: teferi_plus },
    WalkerAb { delta: -3, label: "tuck a threat", val: teferi_minus_val, eff: teferi_minus },
    WalkerAb { delta: -8, label: "emblem", val: always_10, eff: teferi_ult },
];

// ---- Liliana, Death's Majesty (Full)
fn lili_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let spec = Tokens { color: Some(Colors::from_letters("B")), types: vec!["zombie"], ..Tokens::new(1, 2) };
    make_tokens(g, p, spec)?;
    mill(g, p, 2)
}

fn lili_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let gy = &g.player(p).gy;
    if !gy.iter().any(|&c| g.db.get(c).creature && bomb_or_pow(g.db.get(c)) >= 4) {
        return None;
    }
    gy.iter().filter(|&&c| g.db.get(c).creature).map(|&c| bomb_or_pow(g.db.get(c))).max().map(|v| v as f64 - 1.0)
}

fn lili_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    if let Some(c) = super::partials::first_max(&cs, |c| {
        let d = g.db.get(c);
        (d.bomb, d.pow, d.cmc)
    }) {
        remove_first(&mut g.player_mut(p).gy, c);
        let n = enter(g, p, c, Enter::default())?;
        g.perm_mut(n).ttypes = vec!["zombie"];
    }
    Ok(())
}

fn lili_ult_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let n = g.opps(p).flat_map(|q| g.player(q).perms.iter()).filter(|&&m| g.is_creature(m)).count();
    if n >= 4 { Some(8.0) } else { None }
}

fn lili_ult(g: &mut Game, _p: PlayerId, _src: PermId) -> Res {
    for q in 0..g.players.len() {
        for m in g.players[q].perms.clone() {
            if g.is_creature(m) && !has_type(g, m, "zombie") {
                die(g, m, "destroy")?;
            }
        }
    }
    Ok(())
}

static LILIANA: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "Zombie, mill two", val: always_2_5, eff: lili_plus },
    WalkerAb { delta: -3, label: "reanimate", val: lili_minus_val, eff: lili_minus },
    WalkerAb { delta: -7, label: "destroy non-Zombies", val: lili_ult_val, eff: lili_ult },
];

// ---- Narset, Parter of Veils (Full: static: each opponent can't draw more than one card each turn (engine.draw,
// tag narset); -2: the best noncreature, nonland card of the top four)
fn narset_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    if g.player(p).library.len() >= 4 { Some(2.5) } else { None }
}

/// common.narset_dig: -2: look at the top four; a noncreature, nonland card into your hand (the AI's best); the rest
/// on the bottom in a random order
pub fn narset_dig(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let mut top = vec![];
    for _ in 0..g.player(p).library.len().min(4) {
        top.push(g.player_mut(p).library.pop().unwrap());
    }
    // HUMAN(phase 9): the person picks one (or nothing)
    let ok: Vec<usize> = (0..top.len()).filter(|&i| noncre(g.db.get(top[i]))).collect();
    let pick = max_by(&ok, |i| card_worth(g, p, top[i], false));
    let mut rest: Vec<CardId> = (0..top.len()).filter(|&i| Some(i) != pick).map(|i| top[i]).collect();
    g.rng.shuffle(&mut rest);
    for c in rest {
        g.player_mut(p).library.insert(0, c);
    }
    if let Some(i) = pick {
        let pl = g.player_mut(p);
        pl.hand.push(top[i]);
        pl.seen.insert(top[i]);
    }
    crate::glog!(g, "    Narset: {} takes {}", player_name(g, p), if pick.is_some() { "a card" } else { "nothing" });
    Ok(())
}

static NARSET: [WalkerAb; 1] =
    [WalkerAb { delta: -2, label: "dig for a noncreature, nonland card", val: narset_val, eff: narset_dig }];

// ======================================================== proliferate
/// the Vivid lands (lands.py: they enter with two charge counters, `DataKey::Charge`, 2 when unset)
const VIVID: [&str; 5] = ["Vivid Creek", "Vivid Grove", "Vivid Marsh", "Vivid Meadow", "Vivid Crag"];

/// common.proliferate: each permanent / player with counters gets one more of each kind it has (you choose: your
/// +1/+1, loyalty and other counters, opponents' -1/-1 and poison). Orc Armies are left to their hand tag in the four
/// main decks.
///
/// Bug fix (phase 6): Python also adds to `m.data['counters']`, which nothing writes, so the counters the engine
/// keeps elsewhere were never proliferated. Here proliferate also adds: a poison counter to each opponent who has
/// any (not under Melira), a charge counter to your Vivid lands with one left, a +1/+1 counter to your creature lands'
/// kept counters (Raging Ravine), your Incubator token's counters and The Ozolith's counters.
pub fn proliferate(g: &mut Game, p: PlayerId, times: u32) -> Res {
    let extra = if g.hooks.is_empty() {
        0
    } else {
        hooked(g, Event::ProliferateExtra).iter().map(|(src, imp)| (imp.proliferate_extra.unwrap())(g, *src, p)).sum()
    };
    let times = times as i32 * (1 + extra); // Tekuthal
    let dbl = if g.player(p).perms.iter().any(|&m| card_name(g, m) == "Doubling Season") { 2 } else { 1 };
    for _ in 0..times.max(0) {
        let mut sagas = vec![];
        for m in g.player(p).perms.clone() {
            let x = g.perm(m);
            if x.army || x.phased {
                continue;
            }
            if x.plus > 0 {
                g.perm_mut(m).plus += dbl;
            }
            if card_is(g, m, Types::PLANESWALKER)
                && let Some(l) = g.perm(m).loyalty
            {
                g.perm_mut(m).loyalty = Some(l + dbl);
            }
            if g.perm(m).data.get(DataKey::Lore).is_some()
                && let Some(saga) = g.perm(m).cd.and_then(|c| g.registry.get(c)).and_then(|i| i.saga)
                && (saga.0)(g, p, m)
            {
                sagas.push((m, saga.1));
            }
        }
        // the fix: the other counters the engine keeps for permanents (lands, the Incubator, The Ozolith)
        for l in g.player(p).lands.clone() {
            let d = &g.land(l).data;
            let charge = d.get(DataKey::Charge).map(Val::int);
            let kept = d.int(DataKey::Counters);
            if VIVID.contains(&&*g.db.get(g.land(l).cd).name) && charge.unwrap_or(2) > 0 {
                g.land_mut(l).data.set(DataKey::Charge, Val::Int(charge.unwrap_or(2) + dbl as i64));
            }
            if kept > 0 {
                g.land_mut(l).data.set(DataKey::Counters, Val::Int(kept + dbl as i64));
            }
        }
        let pl = g.player_mut(p);
        if pl.incubator > 0 {
            pl.incubator += dbl;
        }
        if pl.ozolith_counters > 0 {
            pl.ozolith_counters += dbl;
        }
        for (m, add_lore) in sagas {
            // a lore counter: that chapter triggers (Summon: Bahamut)
            if on(g, m, p) {
                add_lore(g, p, m)?;
            }
            if g.over || !g.player(p).alive {
                return Ok(());
            }
        }
        let mut poisoned = false;
        for q in opps(g, p) {
            for m in g.player(q).perms.clone() {
                if g.is_creature(m) && g.perm(m).plus < 0 {
                    g.perm_mut(m).plus -= 1;
                    if etgh(g, m) <= 0 {
                        die(g, m, "sba")?;
                    }
                }
            }
            // the fix: opponents' poison counters
            if g.player(q).poison > 0 && !melira(g, q) {
                g.player_mut(q).poison += 1;
                poisoned = true;
            }
        }
        if poisoned {
            check_state(g)?;
        }
        // CI.fire(g, 'proliferated', p): no card hooks it
    }
    Ok(())
}

// ======================================================== mana: Cradle, Nykthos, Coffers, Circle of Dreams, Marwyn
/// common.devotion: p's devotion to colour col
fn devotion(g: &Game, p: PlayerId, col: char) -> u32 {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| !g.perm(m).phased && def(g, m).is_some_and(|d| d.perm))
        .map(|&m| def(g, m).unwrap().pips.chars().filter(|&ch| ch == col).count() as u32)
        .sum()
}

fn creature_count(g: &Game, p: PlayerId) -> u32 {
    g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count() as u32
}

/// Gaea's Cradle (Full): G for each creature you control
fn cradle(g: &Game, p: PlayerId, _l: LandId) -> u32 {
    creature_count(g, p)
}

/// Nykthos, Shrine to Nyx (Approximate): taps for devotion minus the {2} activation
fn nykthos(g: &Game, p: PlayerId, _l: LandId) -> u32 {
    let best = "WUBRG".chars().map(|c| devotion(g, p, c) as i32).max().unwrap();
    (best - 2).max(1) as u32
}

/// Circle of Dreams Druid (Full): G for each creature you control
fn circle(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    creature_count(g, p)
}

/// common._urborg: Urborg, Tomb of Yawgmoth (Approximate): every land counts as a Swamp for Cabal Coffers and Crypt
/// Ghast
fn urborg(g: &Game) -> bool {
    g.players
        .iter()
        .filter(|q| q.alive)
        .any(|q| q.lands.iter().any(|&l| &*g.db.get(g.land(l).cd).name == "Urborg, Tomb of Yawgmoth"))
}

fn is_swamp(g: &Game, l: LandId) -> bool {
    let d = g.db.get(g.land(l).cd);
    d.has_subtype("swamp") || &*d.name == "Swamp" || urborg(g)
}

/// Cabal Coffers (Approximate): B per Swamp minus the {2} activation (Urborg makes every land a Swamp)
fn coffers(g: &Game, p: PlayerId, _l: LandId) -> u32 {
    let n = g.player(p).lands.iter().filter(|&&l| is_swamp(g, l)).count() as i32;
    (n - 2).max(1) as u32
}

/// Marwyn, the Nurturer: G equal to its power
fn marwyn(g: &Game, _p: PlayerId, m: PermId) -> u32 {
    epow(g, m).max(1) as u32
}

/// Crypt Ghast (Partial: extort is partials'): Swamps tap for an extra B
fn ghast(g: &Game, src: Src, p: PlayerId, l: LandId) -> i32 {
    (p == owner(g, src) && is_swamp(g, l)) as i32
}

/// Collector Ouphe (Approximate): artifact mana (rocks, Treasures) is off for everyone; other artifact abilities
/// still work
fn ouphe(_g: &Game, _src: Src, _p: PlayerId) -> i32 {
    1
}

// ======================================================== stax: cost increases, spell limits, locks
/// common._tax_spell's table: (name, amount, which spells, opponents only, only during its controller's turn)
const TAX_SPELL: [(&str, i32, fn(&CardDef) -> bool, bool, bool); 8] = [
    // Full: first strike; noncreature spells cost {1} more (yours too)
    ("Thalia, Guardian of Thraben", 1, noncre, false, false),
    ("Sphere of Resistance", 1, |_| true, false, false),
    ("Thorn of Amethyst", 1, noncre, false, false),
    ("Glowrider", 1, noncre, false, false),
    ("Vryn Wingmare", 1, noncre, false, false),
    // Approximate: spells cost {1} more on your turn (abilities and afterlife not modeled)
    ("Tithe Taker", 1, |_| true, true, true),
    // Approximate: opponents' artifacts and enchantments cost {2} more; the sacrifice ability is not used
    ("Aura of Silence", 2, |c| c.types.has(Types::ARTIFACT) || c.types.has(Types::ENCHANTMENT), true, false),
    // Approximate: opponents' artifact, instant and sorcery spells cost {1} more; loyalty abilities unused
    ("Dovin, Hand of Control", 1, |c| c.types.has(Types::ARTIFACT) || c.instant || c.sorcery, true, false),
];

/// common._tax_spell's cost hook (for every card in TAX_SPELL; rules.py registers Dovin's again)
pub fn tax_spell(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    let Some(&(_, amount, pred, opponents, own_turn)) = TAX_SPELL.iter().find(|x| x.0 == card_name(g, src)) else {
        return 0;
    };
    let o = owner(g, src);
    if opponents && caster == o {
        return 0;
    }
    if own_turn && g.active != Some(o) {
        return 0;
    }
    if pred(g.db.get(c)) { amount } else { 0 }
}

/// Trinisphere (Full): every spell costs at least three
fn trinisphere(g: &Game, _src: Src, _caster: PlayerId, c: CardId) -> i32 {
    if !g.db.get(c).land { 3 } else { 0 }
}

/// Grand Arbiter Augustin IV (Full): your white and blue spells cost {1} less each; opponents' spells cost {1} more
fn gaa(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    if caster == owner(g, src) {
        let pips = &g.db.get(c).pips;
        return -(pips.contains('W') as i32 + pips.contains('U') as i32);
    }
    1
}

/// common._limit's table: each player can't cast more than n spells matching pred each turn
const LIMITS: [(&str, i32, fn(&CardDef) -> bool); 4] = [
    ("Rule of Law", 1, |_| true),
    ("Deafening Silence", 1, noncre),
    ("Ethersworn Canonist", 1, |c| !c.types.has(Types::ARTIFACT)),
    // Approximate: one spell per turn; opponents' nonbasic lands entering tapped is ignored
    ("Archon of Emeria", 1, |_| true),
];

/// common._limit's can_cast hook
fn limit(g: &Game, src: Src, caster: PlayerId, c: CardId, _zone: Sym) -> bool {
    let Some(&(_, n, pred)) = LIMITS.iter().find(|x| x.0 == card_name(g, src)) else { return true };
    if !pred(g.db.get(c)) {
        return true;
    }
    casts_matching(g, caster, pred) < n
}

/// Drannith Magistrate (Full): opponents can't cast from anywhere but their hand
fn drannith(g: &Game, src: Src, caster: PlayerId, _c: CardId, zone: Sym) -> bool {
    caster == owner(g, src) || zone == "hand"
}

/// Grand Abolisher (Approximate): opponents can't cast spells on your turn; activated abilities are not restricted
fn abolisher(g: &Game, src: Src, caster: PlayerId, _c: CardId, _zone: Sym) -> bool {
    caster == owner(g, src) || g.active != Some(owner(g, src))
}

/// Lavinia, Azorius Renegade (Approximate): opponents can't cast noncreature spells above their land count; free
/// spells (Force of Will) are off
fn lavinia(g: &Game, src: Src, caster: PlayerId, c: CardId, _zone: Sym) -> bool {
    let d = g.db.get(c);
    if caster == owner(g, src) || d.creature {
        return true;
    }
    d.cmc as usize <= g.player(caster).lands.len()
        && !(d.tag(Tag::Free) && !can_pay(g, caster, d.generic, &d.pips, false))
}

/// Thalia, Heretic Cathar (Approximate): opponents' creatures enter tapped (nonbasic lands are not tapped)
fn thalia_hc(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src && owner(g, m) != owner(g, src) && g.is_creature(m) {
        g.perm_mut(m).tapped = true;
    }
    Ok(())
}

// ======================================================== staples: Walking Ballista, tutors, Solitude, Magus
/// Walking Ballista (Approximate): cast with X = spare mana ({X}{X}: two mana per counter)
fn ballista(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        let x = g.last_x;
        g.perm_mut(src).plus += x.div_euclid(2);
        g.last_x = 0;
    }
    Ok(())
}

/// pings X/1 creatures or lethal players (infinite-mana kills are in the combo framework)
fn ballista_ping(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let plus = g.perm(src).plus;
    if plus <= 0 || post.is_none() {
        return Ok(vec![]);
    }
    let os = opps(g, p);
    let lethal = os.iter().copied().find(|&q| g.player(q).life <= plus);
    let tg = os
        .iter()
        .flat_map(|&q| g.player(q).perms.iter().copied())
        .find(|&m| g.is_creature(m) && etgh(g, m) <= 1 && pval(g, m) >= 2.5 && !untargetable(g, m));
    if lethal.is_none() && tg.is_none() {
        return Ok(vec![]);
    }
    // the targets as the option was made: (lethal player + 1) in the low byte, the creature + 1 above it
    let arg = lethal.map_or(0, |q| q.0 as i64 + 1) | (tg.map_or(0, |m| m.0 as i64 + 1) << 8);
    let u = if lethal.is_some() { 8.0 } else { 2.0 };
    Ok(vec![opt_ability(u, "Walking Ballista ping".into(), src, ballista_go, arg)])
}

fn ballista_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let lethal = match arg & 0xff {
        0 => None,
        q => Some(PlayerId((q - 1) as u8)),
    };
    let tg = match arg >> 8 {
        0 => None,
        m => Some(PermId((m - 1) as u32)),
    };
    if g.perm(src).plus <= 0 {
        return Ok(false);
    }
    g.perm_mut(src).plus -= 1;
    let imp = if lethal.is_some() { 8.0 } else { 3.0 };
    if !ability_window(g, p, Some(src), "1 damage", Some(imp), None)? {
        return Ok(true);
    }
    if let Some(q) = lethal {
        lose_life(g, q, 1, Some(p), "triggers", None)?;
    } else if let Some(t) = tg
        && g.perm(t).on_bf
    {
        apply_removal(g, Some(p), t, "dmg1", None)?;
    }
    if etgh(g, src) <= 0 {
        die(g, src, "sba")?;
    }
    Ok(true)
}

/// common._ballista_prio: cast for value with four or more mana; a deck with the Scepter / Power Artifact combo keeps
/// it as the kill
fn ballista_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let mana = total_mana(g, p, false);
    if mana < 4 {
        return 0;
    }
    let pl = g.player(p);
    let combo = ["Isochron Scepter", "Power Artifact", "Rings of Brighthearth"];
    let has = pl.library.iter().chain(&pl.hand).any(|&c| combo.contains(&&*g.db.get(c).name))
        || pl.perms.iter().any(|&m| combo.contains(&card_name(g, m)));
    if has && mana < 8 {
        return 0;
    }
    45
}

/// common._tutor_etb: a creature tutor's enter trigger (Spellseeker, Recruiter of the Guard, Goblin Matron; t5.py
/// registers Trophy Mage and Tribute Mage with it): t1.tutor_named with the card's TUTOR_PRED
pub fn tutor_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let Some(pred) = tutor_pred(card_name(g, src)) else { return Ok(()) };
    if m == src && trigger_window(g, owner(g, src), Some(src), "search for a card", None)? {
        super::t1::tutor_named(g, owner(g, src), &|g, c| pred(g.db.get(c)), 1, "hand")?;
    }
    Ok(())
}

/// Solitude (Approximate): exiles the best opposing creature on entry (rules2.py's evoke is its hand option)
fn solitude(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let t = best_opp_creature(g, o, |_| true);
    if t.is_none_or(|t| pval(g, t) < 2.0) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "exile a creature", Some(6.0))? {
        return Ok(());
    }
    if let Some(t) = best_opp_creature(g, o, |_| true).filter(|&t| pval(g, t) >= 2.0) {
        let cd = g.perm(src).cd;
        apply_removal(g, Some(o), t, "exile", cd)?;
    }
    Ok(())
}

/// Magus of the Moon (Full): nonbasic lands tap for R (engine mana check)
fn magus(_g: &Game, _src: Src) -> bool {
    true
}

/// cardimpl.blood_moon: a Blood Moon effect is on (`bool(list(hooked(g, 'blood_moon')))`)
pub fn blood_moon(g: &Game) -> bool {
    !hooked(g, Event::BloodMoon).is_empty()
}

/// common.blood_moon_active
pub fn blood_moon_active(g: &Game) -> bool {
    g.players
        .iter()
        .filter(|q| q.alive)
        .any(|q| q.perms.iter().any(|&m| card_name(g, m) == "Magus of the Moon" && !g.perm(m).phased))
}

/// Ajani's Chosen (Approximate): a 2/2 Cat per enchantment entering; the Aura move is not used
fn ajani_chosen(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if owner(g, m) == o
        && card_is(g, m, Types::ENCHANTMENT)
        && trigger_window(g, o, Some(src), "create a 2/2 Cat", None)?
    {
        let spec = Tokens { color: Some(Colors::from_letters("W")), types: vec!["cat"], ..Tokens::new(1, 2) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

// ======================================================== Urza's Saga (land)
/// LAND_ETB: lore counter I
fn saga_etb(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    g.player_mut(p).sagas.push((l, 1));
    Ok(())
}

/// common._saga_construct: {2}, {T}: a 0/0 Construct that gets +1/+1 for each artifact you control. The {2} comes
/// from other sources (the Saga taps as part of the cost). Made when it would be at least a 3/3, or when the mana
/// would go unused anyway (nothing castable in hand needs it)
fn saga_construct(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    if g.land(l).tapped || !g.player(p).lands.contains(&l) {
        return Ok(());
    }
    g.land_mut(l).tapped = true; // the Saga taps as part of the cost: it can't pay its own {2}
    if !can_pay(g, p, 2, "", false) {
        g.land_mut(l).tapped = false;
        return Ok(());
    }
    let arts = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| !g.perm(m).phased && (card_is(g, m, Types::ARTIFACT) || g.perm(m).data.truthy(DataKey::Artifact)))
        .count() as i64;
    let avail = total_mana(g, p, false) as i64;
    let need = g
        .player(p)
        .hand
        .iter()
        .map(|&c| g.db.get(c))
        .filter(|d| !d.land)
        .map(|d| d.generic as i64 + d.pips.chars().count() as i64)
        .filter(|&n| n <= avail)
        .max()
        .unwrap_or(0);
    if arts + 1 < 3 && avail - 2 < need {
        g.land_mut(l).tapped = false;
        return Ok(());
    }
    pay(g, p, 2, "", false)?;
    g.selfpt = true; // before the token exists: a 0/0 is checked at once
    let mut data = crate::state::PermData::default();
    data.set(DataKey::Construct, Val::Bool(true));
    data.set(DataKey::Artifact, Val::Bool(true));
    let spec = Tokens { tgh: Some(0), color: Some(Colors::NONE), types: vec!["construct"], data, ..Tokens::new(1, 0) };
    make_tokens(g, p, spec)?;
    crate::glog!(g, "    Urza's Saga: {} makes a Construct ({}/{})", player_name(g, p), arts + 1, arts + 1);
    Ok(())
}

/// common.saga_step: each precombat main phase a lore counter: chapter II grants the Construct ability (used that
/// turn), chapter III's search trigger can be answered with one more Construct before the Saga is sacrificed
pub fn saga_step(g: &mut Game, p: PlayerId) -> Res {
    for (l, _) in g.player(p).sagas.clone() {
        let Some(i) = g.player(p).sagas.iter().position(|x| x.0 == l) else { continue };
        if !g.player(p).lands.contains(&l) {
            g.player_mut(p).sagas.remove(i);
            continue;
        }
        let lore = g.player(p).sagas[i].1 + 1;
        g.player_mut(p).sagas[i].1 = lore;
        if lore == 2 {
            saga_construct(g, p, l)?;
        }
        if lore >= 3 {
            saga_construct(g, p, l)?; // in response to chapter III
            let cs: Vec<CardId> = searchable(g, p)
                .into_iter()
                .filter(|&c| {
                    let d = g.db.get(c);
                    d.types.has(Types::ARTIFACT) && d.cmc <= 1 && !d.land && !d.tag(Tag::X)
                })
                .collect();
            if !cs.is_empty() {
                // the deck's wish list first (a Breach piece, a combo part)
                let want = crate::ai::tutor_pick(g, p, "art");
                let c = cs
                    .iter()
                    .copied()
                    .find(|&x| Some(x) == want)
                    .unwrap_or_else(|| max_by(&cs, |c| crate::ai::tutor_value(g, p, c)).unwrap());
                remove_first(&mut g.player_mut(p).library, c);
                shuffle_library(g, p);
                enter(g, p, c, Enter::default())?;
                crate::glog!(
                    g,
                    "    Urza's Saga: {} puts {} onto the battlefield",
                    player_name(g, p),
                    g.db.get(c).name
                );
            }
            crate::engine::turn::remove_land(g, p, l);
            let cd = g.land(l).cd;
            g.player_mut(p).gy.push(cd);
            if let Some(i) = g.player(p).sagas.iter().position(|x| x.0 == l) {
                g.player_mut(p).sagas.remove(i);
            }
        }
    }
    Ok(())
}

// ======================================================== Powerstone tokens, Static Net
/// common.make_powerstone: n Powerstone tokens: artifacts with "{T}: Add {C}. This mana can't be spent to cast a
/// nonartifact spell" (engine.mana_units offers it only when an artifact is being paid for)
pub fn make_powerstone(g: &mut Game, p: PlayerId, n: u32, tapped: bool) -> Res {
    let ps = g.db.id("Powerstone").expect("Powerstone: an engine-made card (cards.rs)");
    for _ in 0..n * crate::dsl::token_mult(g, p) {
        let m = enter(g, p, ps, Enter::default())?;
        let x = g.perm_mut(m);
        x.token = true;
        x.tapped = tapped;
    }
    crate::glog!(g, "    {} creates {} Powerstone token(s)", player_name(g, p), n);
    Ok(())
}

/// Static Net (Full): when it enters, you gain 2 life and create a tapped Powerstone (its exile is the ability
/// language's)
fn static_net(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m != src || !trigger_window(g, o, Some(src), "gain 2 life; a Powerstone", None)? {
        return Ok(());
    }
    gain(g, o, 2)?;
    crate::glog!(g, "    {} gains 2 life (Static Net)", player_name(g, o));
    make_powerstone(g, o, 1, true)
}

// ======================================================== the rest of the registration
fn register_phase6(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    // staples: spells
    let c = r.card(db, "Fact or Fiction")?;
    c.resolve = Some(fof);
    c.prio = Some(prio_45);
    r.card(db, "Council's Judgment")?.resolve = Some(judgment);
    let c = r.card(db, "Sign in Blood")?;
    c.resolve = Some(sign_in_blood);
    c.prio = Some(prio_45);
    r.card(db, "Otawara, Soaring City")?.hand_options = Some(otawara);
    let c = r.card(db, "Reflector Mage")?;
    c.etb = Some(reflector);
    c.can_cast = Some(reflector_lock);
    let c = r.card(db, "Spark Double")?;
    c.etb = Some(spark);
    *c = c.at_once(Event::Etb);
    let c = r.card(db, "Frost Titan")?;
    c.etb = Some(frost);
    c.attack = Some(frost_atk);
    let c = r.card(db, "Dream Trawler")?;
    c.attack = Some(trawler);
    c.draw = Some(trawler_draw);
    r.card(db, "Ephemerate")?.rebound = Some(ephemerate_rebound);
    // graveyard hate
    r.card(db, "Bojuka Bog")?.land_etb = Some(bog);
    let c = r.card(db, "Dauthi Voidwalker")?;
    c.sba = Some(dauthi_sweep);
    c.options = Some(dauthi_play);
    for name in ["Tormod's Crypt", "Soul-Guide Lantern"] {
        let c = r.card(db, name)?;
        c.gy_hate = Some(gy_hate);
        c.options = Some(gy_hate_options);
    }
    // pillowfort
    for name in [
        "Ghostly Prison",
        "Propaganda",
        "Windborn Muse",
        "Baird, Steward of Argive",
        "Sphere of Safety",
        // "Norn's Annex": rules2.py replaces common's tax with its own (0: the {W/P} is rules2.annex_life's), so
        // common's isn't registered (MERGE: rules2's `_annex_tax` registers Norn's Annex's attack_tax)
        "Archangel of Tithes",
        "Elephant Grass",
    ] {
        r.card(db, name)?.attack_tax = Some(pillowfort_tax);
    }
    r.card(db, "Crawlspace")?.attack_cap = Some(crawlspace);
    r.card(db, "Silent Arbiter")?.attack_cap = Some(arbiter);
    r.card(db, "Elephant Grass")?.upkeep = Some(grass);
    // shroud / hexproof grants
    r.card(db, "Greater Auramancy")?.grant_kw = Some(auramancy);
    let c = r.card(db, "Sterling Grove")?;
    c.grant_kw = Some(grove_kw);
    c.options = Some(grove);
    r.card(db, "Privileged Position")?.grant_kw = Some(privileged);
    r.card(db, "Spore Frog")?.blocks = Some(spore_frog);
    // planeswalkers
    walker(r, db, "Elspeth, Sun's Champion", &ELSPETH)?;
    walker(r, db, "Teferi, Hero of Dominaria", &TEFERI)?;
    walker(r, db, "Liliana, Death's Majesty", &LILIANA)?;
    walker(r, db, "Narset, Parter of Veils", &NARSET)?;
    // mana
    r.card(db, "Gaea's Cradle")?.dyn_mana_land = Some(cradle);
    r.card(db, "Nykthos, Shrine to Nyx")?.dyn_mana_land = Some(nykthos);
    r.card(db, "Circle of Dreams Druid")?.dyn_mana_perm = Some(circle);
    r.card(db, "Cabal Coffers")?.dyn_mana_land = Some(coffers);
    r.card(db, "Marwyn, the Nurturer")?.dyn_mana_perm = Some(marwyn);
    r.card(db, "Crypt Ghast")?.land_mana = Some(ghast);
    r.card(db, "Collector Ouphe")?.no_artifact_mana = Some(ouphe);
    // stax
    for (name, ..) in TAX_SPELL {
        r.card(db, name)?.cost = Some(tax_spell);
    }
    r.card(db, "Trinisphere")?.min_cost = Some(trinisphere);
    r.card(db, "Grand Arbiter Augustin IV")?.cost = Some(gaa);
    for (name, ..) in LIMITS {
        r.card(db, name)?.can_cast = Some(limit);
    }
    r.card(db, "Drannith Magistrate")?.can_cast = Some(drannith);
    r.card(db, "Grand Abolisher")?.can_cast = Some(abolisher);
    r.card(db, "Lavinia, Azorius Renegade")?.can_cast = Some(lavinia);
    let c = r.card(db, "Thalia, Heretic Cathar")?;
    c.etb = Some(thalia_hc);
    *c = c.at_once(Event::Etb);
    // staples
    let c = r.card(db, "Walking Ballista")?;
    c.etb = Some(ballista);
    *c = c.at_once(Event::Etb);
    c.options = Some(ballista_ping);
    c.prio = Some(ballista_prio);
    for name in ["Spellseeker", "Recruiter of the Guard", "Goblin Matron"] {
        r.card(db, name)?.etb = Some(tutor_etb);
    }
    r.card(db, "Solitude")?.etb = Some(solitude);
    r.card(db, "Magus of the Moon")?.blood_moon = Some(magus);
    r.card(db, "Ajani's Chosen")?.etb = Some(ajani_chosen);
    r.card(db, "Urza's Saga")?.land_etb = Some(saga_etb);
    r.card(db, "Static Net")?.etb = Some(static_net);
    Ok(())
}

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
        c.leaves = Some(oring_leaves);
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
    r.card(db, "Destiny Spinner")?.uncounterable = Some(spinner);
    r.card(db, "Allosaurus Shepherd")?.uncounterable = Some(shepherd);
    for &(name, v) in PVAL {
        r.card(db, name)?.pval = Some(v);
    }
    register_phase6(r, db)
}
