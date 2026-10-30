//! Tier 5 pool deck `winota-boros-humans-cheat` (EDHREC's bracket-4 average list): the cards the ability language
//! doesn't cover (rust/agents/t45-winota.md). "The Ring tempts you" is compiled (dsl_parse.rs, `// --- Tiers 4/5
//! (winota)`); the rest is here, with small hooks in the engine marked the same way.
//!
//! - Breath of Fury: an Aura on your cheapest attacker; when the enchanted creature deals combat damage to a player,
//!   it is sacrificed, the Aura moves to another creature of yours, your creatures untap and there is an additional
//!   combat (once per combat; nothing if it has no creature to move to).
//! - Flare of Fortitude: a protector (ai/pool.rs PROTECTORS: hexproof and indestructible for your permanents against
//!   removal and wipes) and the last chance against lethal damage (zones::last_chance): your life total can't change
//!   until end of turn. Cast for {2}{W}{W} or by sacrificing a cheap nontoken white creature. Left out: lands gain
//!   nothing (they are not permanents in the engine).
//! - High Noon: each player can't cast more than one spell each turn; {4}{R}, sacrifice: 5 damage to any target
//!   (lethal to a player, else the best creature it kills).
//! - Windcrag Siege: Mardu when the deck's commander (or two of its permanents) has an attack trigger: attack
//!   triggers of your permanents trigger an additional time (the Isshin rule); else Jeskai: a 1/1 red Goblin at your
//!   upkeep with lifelink and haste until end of turn.
//! - Alexios, Deimos of Kosmos: at each player's upkeep that player gains control of it, untaps it, puts a +1/+1
//!   counter on it and it gains haste; it attacks each combat if able (`DataKey::MustAttack`), can't attack its owner
//!   (its owner goads it while someone else controls it: it attacks another player) and can't be sacrificed
//!   (values::no_sac). Left out: in a two-player game the goad lets it attack its owner.
//! - Angrath's Marauders: damage from your sources is doubled (combat::fiery, with Fiery Emancipation). Left out:
//!   damage you deal to yourself.
//! - Blinkmoth Nexus: a creature land ({1}: 1/1 flier until end of turn, lands.rs MANLANDS). Left out: the {1},{T}
//!   pump of a Blinkmoth.
//! - Boromir, Warden of the Tower: an opponent's spell cast with no mana spent is countered (`cast_counter`);
//!   sacrificed to make your creatures indestructible in response to removal or a wipe (PROTECTORS), the Ring tempts
//!   you (compiled for the ability language's own use of the sacrifice).
//! - Gingerbrute: {1} before combat: it can't be blocked this turn except by creatures with haste (`haste_blockers`).
//! - Greymond, Avacyn's Stalwart: first strike and lifelink for your Humans (the AI's pick); four or more Humans:
//!   Humans you control get +2/+2.
//! - Lena, Selfless Champion: enters: a 1/1 white Soldier per nontoken creature you control; sacrificed in response to
//!   removal or a wipe: your creatures with less power than Lena gain indestructible until end of turn.
//! - Myrel, Shield of Argive: during your turn opponents can't cast spells (`can_cast`) or activate abilities of
//!   artifacts, creatures and enchantments (rules::ability_locked); its attack tokens are compiled.
//! - Needleverge Pathway // Pillarverge Pathway: played as the face whose colour the deck lacks ({R} or {W}).
//! - Phoenix Chick: you attack with three or more creatures: pay {R}{R}, it returns from your graveyard tapped and
//!   attacking with a +1/+1 counter (combat::attack_triggers).
//! - Phyrexian Revoker: names what Pithing Needle would (t6_stax::needle_stops). Left out: mana abilities are not
//!   stopped (the engine's mana code doesn't ask).
//! - Soulless Jailer: noncreature spells can't be cast from graveyards or exile (`can_cast`); reanimation (the pool
//!   AI's, the ability language's, Phoenix Chick) finds nothing. Left out: other hand-written returns from the
//!   graveyard.
//! - Sundering Eruption // Volcanic Fissure: before combat, when the attack needs it: destroy the best opposing land
//!   (its controller fetches a basic, tapped), creatures without flying can't block this turn; or played as a land
//!   ({R}; 3 life or tapped).
//!
//! Combo: Rionya, Fire Dancer + Combat Celebrant (a hasty Celebrant copy each combat exerts for another combat:
//! infinite combats), `COMBOS`, chained in combos::all.

use crate::engine::repl;
use crate::cards::{CardDb, Colors};
use crate::engine::cast::{castable, casts_this_turn, on_cast};
use crate::engine::control::{Duration, GoadFor, gain_control, goad};
use crate::engine::life::prevents_damage;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, counter_window, trigger_window, trigger_window_card};
use crate::engine::values::{card_name, epow, etgh, has_type, no_sac, pval, untargetable};
use crate::engine::zones::{Enter, Tokens, die, enter, land_ramp, make_tokens};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::combos::{Combo, Ready};
use crate::impls::common::{AURA, AuraSpec, aura};
use crate::impls::partials::eot_kw;
use crate::state::{Ctx, DataKey, Game, Step, Val};
use crate::sym::{Sym, intern};

const BREATH: &str = "Breath of Fury";
const FLARE: &str = "Flare of Fortitude";
const NOON: &str = "High Noon";
const SIEGE: &str = "Windcrag Siege";
const ALEXIOS: &str = "Alexios, Deimos of Kosmos";
const BOROMIR: &str = "Boromir, Warden of the Tower";
const GINGERBRUTE: &str = "Gingerbrute";
const GREYMOND: &str = "Greymond, Avacyn's Stalwart";
const LENA: &str = "Lena, Selfless Champion";
const MYREL: &str = "Myrel, Shield of Argive";
const PATHWAY: &str = "Needleverge Pathway // Pillarverge Pathway";
const CHICK: &str = "Phoenix Chick";
const REVOKER: &str = "Phyrexian Revoker";
const JAILER: &str = "Soulless Jailer";
const ERUPTION: &str = "Sundering Eruption // Volcanic Fissure";
const RIONYA: &str = "Rionya, Fire Dancer";
const CELEBRANT: &str = "Combat Celebrant";

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, src: PermId) -> PlayerId {
    g.perm(src).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

/// p's live creatures
fn creatures_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

/// p's first live permanent named `name`
fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && card_name(g, m) == Some(name))
}

fn flying(g: &Game, m: PermId) -> bool {
    g.perm(m).fly || crate::dsl::has_kw(g, m, "flying")
}

/// could m attack this turn (untapped, not summoning sick or hasty, some power)
fn can_attack(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    live(g, m)
        && g.is_creature(m)
        && !x.tapped
        && !x.noatk
        && epow(g, m) > 0
        && (!x.sick || crate::engine::combat::has_haste(g, m))
}

/// the mode a permanent chose as it entered (`DataKey::Mode`)
fn mode_is(g: &Game, m: PermId, want: &str) -> bool {
    matches!(g.perm(m).data.get(DataKey::Mode), Some(Val::Str(s)) if *s == want)
}

// ================================================================== Breath of Fury
/// the creature Breath of Fury should go on (and move to): one that is sacrificed when it connects, so the cheapest
/// creature likely to connect: evasive, a non-Human (Winota's trigger), a token, able to attack now
fn breath_pick(g: &Game, p: PlayerId, except: Option<PermId>) -> Option<PermId> {
    let score = |m: PermId| {
        let x = g.perm(m);
        let evasive = flying(g, m) || crate::dsl::has_kw(g, m, "unblockable");
        3.0 * evasive as i32 as f64
            + 1.5 * (!has_type(g, m, "human")) as i32 as f64
            + 1.0 * x.token as i32 as f64
            + 1.0 * can_attack(g, m) as i32 as f64
            - 2.0 * x.is_cmd as i32 as f64
            - 0.6 * pval(g, m)
    };
    creatures_of(g, p).into_iter().filter(|&m| Some(m) != except).max_by(|&a, &b| score(a).total_cmp(&score(b)))
}

fn breath_host(g: &mut Game, p: PlayerId, _aura: PermId) -> Res<Option<PermId>> {
    Ok(breath_pick(g, p, None))
}

static BREATH_SPEC: AuraSpec = AuraSpec { target: "own", host_pick: Some(breath_host), ..AURA };

/// When enchanted creature deals combat damage to a player, sacrifice it and attach this Aura to a creature you
/// control. If you do, untap all creatures you control and after this phase, there is an additional combat phase.
fn breath_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if owner(g, src) != p || g.perm(src).attached != Some(a) || !live(g, src) {
        return Ok(());
    }
    // once per combat: the Aura moving to another attacker in the same damage step doesn't trigger again
    let now = Val::List(vec![Val::Stamp(g.turn_stamp()), Val::Int(g.player(p).combat_no as i64)]);
    if g.perm(src).data.get(DataKey::Used) == Some(&now) {
        return Ok(());
    }
    g.perm_mut(src).data.set(DataKey::Used, now);
    if !trigger_window(g, p, Some(src), "sacrifice it, move the Aura: untap, an additional combat", Some(5.0))? {
        return Ok(());
    }
    if !g.perm(src).on_bf {
        return Ok(());
    }
    let host = breath_pick(g, p, Some(a));
    if let Some(h) = host {
        g.perm_mut(src).attached = Some(h); // attached before the creature goes (else the Aura would fall with it)
    }
    if g.perm(a).on_bf && g.perm(a).owner == p {
        crate::glog!(g, "    {} sacrifices {} ({BREATH})", pname(g, p), g.perm(a).name);
        die(g, a, "sac")?;
    }
    let Some(h) = host.filter(|&h| g.perm(h).on_bf && g.perm(src).on_bf) else {
        return crate::cardcode::aura_fall(g, a); // nothing to move to: the Aura goes with the creature
    };
    crate::glog!(g, "    {BREATH} moves to {}: untap, an additional combat", g.perm(h).name);
    for m in creatures_of(g, p) {
        g.perm_mut(m).tapped = false;
    }
    g.player_mut(p).extra_combats += 1;
    Ok(())
}

// ================================================================== Flare of Fortitude
/// the nontoken white creature p would sacrifice instead of paying for Flare of Fortitude: its least valuable, worth
/// under 3.5, not its commander. None for any other protector's name.
pub fn flare_fodder(g: &Game, p: PlayerId, name: &str, keep: Option<PermId>) -> Option<PermId> {
    if name != FLARE {
        return None;
    }
    creatures_of(g, p)
        .into_iter()
        .filter(|&m| {
            let x = g.perm(m);
            Some(m) != keep
                && !x.token
                && !x.is_cmd
                && !no_sac(g, m)
                && x.cd.is_some_and(|c| g.db.get(c).pips.contains('W'))
                && pval(g, m) < 3.5
        })
        .min_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)))
}

/// Until end of turn, your life total can't change, and permanents you control gain hexproof and indestructible.
fn flare_effect(g: &mut Game, p: PlayerId) {
    let pl = g.player_mut(p);
    pl.life_locked = true;
    pl.life_lock_eot = true;
    for m in g.player(p).perms.clone() {
        eot_kw(g, m, "hexproof");
        eot_kw(g, m, "indestructible");
    }
    crate::glog!(g, "    {}: life can't change, permanents hexproof and indestructible this turn", FLARE);
}

/// ai/pool.rs: a protector was used (`use_protector`): Flare of Fortitude's full effect, Boromir's Ring
pub fn after_protector(g: &mut Game, p: PlayerId, name: &str) -> Res {
    match name {
        FLARE => flare_effect(g, p),
        BOROMIR => {
            for m in creatures_of(g, p) {
                eot_kw(g, m, "indestructible"); // creatures you control gain indestructible until end of turn
            }
            crate::cardcode::ring_tempt(g, p)?
        }
        _ => {}
    }
    Ok(())
}

/// zones::last_chance: q is about to lose to damage or life loss: Flare of Fortitude from hand (its mana or a
/// nontoken white creature), if it can
pub fn flare_last_chance(g: &mut Game, q: PlayerId) -> Res<bool> {
    let Some(c) = g.player(q).hand.iter().copied().find(|&c| &*g.db.get(c).name == FLARE) else { return Ok(false) };
    if crate::engine::values::silenced(g, q) || !castable(g, q, c, "hand") {
        return Ok(false);
    }
    let fodder = if can_pay(g, q, 2, "WW", false) { None } else { flare_fodder(g, q, FLARE, None) };
    if fodder.is_none() && !can_pay(g, q, 2, "WW", false) {
        return Ok(false);
    }
    g.player_mut(q).hand.retain(|&x| x != c);
    match fodder {
        Some(f) => {
            crate::glog!(g, "  {} sacrifices {} to cast {FLARE}", pname(g, q), g.perm(f).name);
            die(g, f, "sac")?;
        }
        None => {
            pay(g, q, 2, "WW", false)?;
        }
    }
    let pl = g.player_mut(q);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    on_cast(g, q, c)?;
    g.player_mut(q).gy.push(c);
    if !counter_window(g, q, c, 9.0, vec![])? {
        return Ok(false);
    }
    flare_effect(g, q);
    g.player_mut(q).stat("protection_used", 1);
    Ok(true)
}

// ================================================================== High Noon
/// Each player can't cast more than one spell each turn.
fn one_spell(g: &Game, _src: Src, caster: PlayerId, c: CardId, _zone: Sym) -> bool {
    g.db.get(c).land || casts_this_turn(g, caster) < 1
}

/// the 5 damage's worth: a player it kills (9), else the best creature it kills (its value less the enchantment's)
fn noon_value(g: &Game, p: PlayerId) -> f64 {
    let n = 5 * crate::engine::combat::fiery(g, p);
    let lethal = g.opps(p).any(|q| {
        let x = g.player(q);
        x.life <= n && !x.life_locked && !prevents_damage(g, q, Some(p))
    });
    if lethal {
        return 9.0;
    }
    g.opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m) && !untargetable(g, m) && etgh(g, m) <= n)
        .map(|m| pval(g, m) - 2.0)
        .fold(0.0, f64::max)
}

fn noon_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p
        || !live(g, src)
        || crate::cardcode::ability_locked(g, src, p)
        || !can_pay(g, p, 4, "R", false)
    {
        return Ok(vec![]);
    }
    let v = noon_value(g, p);
    if v < 2.5 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: v,
        label: format!("{NOON}: 5 damage to any target"),
        act: Some(Action::Ability { src, f: noon_go, arg: 0 }),
    }])
}

/// {4}{R}, Sacrifice this enchantment: It deals 5 damage to any target.
fn noon_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || !pay(g, p, 4, "R", false)? {
        return Ok(false);
    }
    crate::glog!(g, "  {} sacrifices {NOON}: 5 damage", pname(g, p));
    die(g, src, "sac")?;
    if ability_window(g, p, None, "5 damage to any target", None, None)? {
        crate::impls::t3::best_target_any(g, p, 5)?;
    }
    Ok(true)
}

// ================================================================== Windcrag Siege
/// does p's game plan run on attack triggers: its commander has one (Winota), or two of its permanents do
fn attack_payoffs(g: &Game, p: PlayerId) -> bool {
    let attack_trigger = |c: CardId| {
        g.registry.get(c).is_some_and(|i| i.attack.is_some())
            || g.db
                .get(c)
                .abilities
                .iter()
                .any(|a| matches!(a, crate::dsl::model::Ability::Triggered { event, .. } if event == "attack"))
    };
    let pl = g.player(p);
    if [Some(pl.cmd), pl.cmd2].into_iter().flatten().any(attack_trigger) {
        return true;
    }
    pl.perms.iter().filter(|&&m| g.perm(m).cd.is_some_and(attack_trigger)).count() >= 2
}

/// As this enchantment enters, choose Mardu or Jeskai.
fn siege_as(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let mode = if attack_payoffs(g, p) { "mardu" } else { "jeskai" };
    g.perm_mut(m).data.set(DataKey::Mode, Val::Str(intern(mode)));
    crate::glog!(g, "    {SIEGE}: {mode}");
    Ok(())
}

/// Mardu: if a creature attacking causes a triggered ability of a permanent you control to trigger, that ability
/// triggers an additional time.
fn siege_copies(g: &Game, src: Src, p: PlayerId, kind: Sym, _x: Option<PermId>) -> i32 {
    (kind == "attack" && owner(g, src) == p && live(g, src) && mode_is(g, src, "mardu")) as i32
}

/// Jeskai: at the beginning of your upkeep, create a 1/1 red Goblin creature token. It gains lifelink and haste until
/// end of turn.
fn siege_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if owner(g, src) != p || !mode_is(g, src, "jeskai") {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "a 1/1 Goblin with lifelink and haste", None)? {
        let spec = Tokens { color: Some(Colors::from_letters("R")), types: vec!["goblin"], ..Tokens::new(1, 1) };
        for m in make_tokens(g, p, spec)? {
            eot_kw(g, m, "lifelink");
            eot_kw(g, m, "haste");
        }
    }
    Ok(())
}

// ================================================================== Alexios, Deimos of Kosmos
/// At the beginning of each player's upkeep, that player gains control of Alexios, untaps it, and puts a +1/+1
/// counter on it. It gains haste until end of turn. (It attacks each combat if able and can't attack its owner: its
/// owner goads it while another player controls it.)
fn alexios_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if !live(g, src) || !g.player(p).alive {
        return Ok(());
    }
    let ctl = owner(g, src);
    if !trigger_window(g, ctl, Some(src), "that player gains control, untaps it, a +1/+1 counter", None)? {
        return Ok(());
    }
    if !g.perm(src).on_bf || !gain_control(g, p, src, Duration::Permanent) {
        return Ok(());
    }
    g.perm_mut(src).tapped = false;
    repl::put_plus(g, src, 1);
    eot_kw(g, src, "haste");
    let now = Val::Stamp(g.turn_stamp());
    g.perm_mut(src).data.set(DataKey::MustAttack, now);
    let orig = g.perm(src).orig;
    if orig != p {
        goad(g, orig, src, GoadFor::NextTurn); // it can't attack its owner
    } else {
        g.perm_mut(src).data.remove(DataKey::Goaded);
    }
    Ok(())
}

// ================================================================== Boromir, Warden of the Tower
/// Whenever an opponent casts a spell, if no mana was spent to cast it, counter that spell.
fn boromir_counter(g: &Game, src: Src, caster: PlayerId, c: CardId, ctx: &Ctx) -> bool {
    live(g, src) && caster != owner(g, src) && !g.db.get(c).land && ctx.mana_spent == 0
}

// ================================================================== Gingerbrute
/// {1}: it can't be blocked this turn except by creatures with haste. Before combat, when a defender has an untapped
/// creature without haste that could block it.
fn ginger_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || owner(g, src) != p || !can_attack(g, src) {
        return Ok(vec![]);
    }
    if g.perm(src).eot_kw.contains(&"haste_blockers")
        || crate::cardcode::ability_locked(g, src, p)
        || !can_pay(g, p, 1, "", false)
    {
        return Ok(vec![]);
    }
    let walls = g.opps(p).any(|q| {
        g.player(q).perms.iter().any(|&b| {
            live(g, b)
                && g.is_creature(b)
                && !g.perm(b).tapped
                && !crate::engine::combat::has_haste(g, b)
                && crate::engine::combat::can_block(g, b, src)
        })
    });
    if !walls {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 0.6 + 0.3 * epow(g, src) as f64,
        label: format!("{GINGERBRUTE}: evasion"),
        act: Some(Action::Ability { src, f: ginger_go, arg: 0 }),
    }])
}

fn ginger_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || !pay(g, p, 1, "", false)? {
        return Ok(false);
    }
    if ability_window(g, p, Some(src), "can't be blocked except by creatures with haste", None, None)? && live(g, src) {
        eot_kw(g, src, "haste_blockers");
    }
    Ok(true)
}

// ================================================================== Greymond, Avacyn's Stalwart
/// As Greymond enters, choose two abilities from among first strike, vigilance, and lifelink: the AI takes first
/// strike (its Humans win combat) and lifelink (the deck races).
fn greymond_as(g: &mut Game, _p: PlayerId, m: PermId) -> Res {
    let pick = vec![Val::Str("first strike"), Val::Str("lifelink")];
    g.perm_mut(m).data.set(DataKey::Mode, Val::List(pick));
    g.selfpt = true; // the +2/+2 is read through `creature_pt`
    Ok(())
}

fn greymonds(g: &Game, p: PlayerId) -> impl Iterator<Item = PermId> + '_ {
    g.player(p).perms.iter().copied().filter(move |&m| live(g, m) && card_name(g, m) == Some(GREYMOND))
}

/// Humans you control have each of the chosen abilities.
fn greymond_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    if !live(g, src) || g.perm(m).owner != owner(g, src) || !g.is_creature(m) || !has_type(g, m, "human") {
        return false;
    }
    matches!(g.perm(src).data.get(DataKey::Mode), Some(Val::List(v)) if v.iter().any(|k| *k == Val::Str(kw)))
}

/// As long as you control four or more Humans, Humans you control get +2/+2 (once per Greymond).
fn greymond_pt(g: &Game, m: PermId) -> (i32, i32) {
    let p = g.perm(m).owner;
    let n = greymonds(g, p).count() as i32;
    if n == 0 || !has_type(g, m, "human") {
        return (0, 0);
    }
    let humans =
        g.player(p).perms.iter().filter(|&&x| live(g, x) && g.is_creature(x) && has_type(g, x, "human")).count();
    if humans >= 4 { (2 * n, 2 * n) } else { (0, 0) }
}

// ================================================================== Lena, Selfless Champion
/// When Lena enters, create a 1/1 white Soldier creature token for each nontoken creature you control.
fn lena_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "a 1/1 Soldier for each nontoken creature you control", None)? {
        return Ok(());
    }
    let n = creatures_of(g, o).into_iter().filter(|&x| !g.perm(x).token).count() as u32;
    if n > 0 {
        let spec = Tokens { color: Some(Colors::from_letters("W")), types: vec!["soldier"], ..Tokens::new(n, 1) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

/// ai/pool.rs `protect`: Lena (`src`) saves only a creature with less power than hers; any other protector: true
pub fn lena_saves(g: &Game, name: &str, src: Option<PermId>, m: PermId) -> bool {
    name != LENA || src.is_some_and(|s| m != s && g.is_creature(m) && epow(g, m) < epow(g, s))
}

/// ai/pool.rs `wipe_response`: Lena (`src`) against a wipe her indestructibility answers: sacrificed when the
/// creatures with less power than hers are worth saving (they gain indestructible until end of turn; the wipe takes
/// the rest). None for any other protector; Some(false) when Lena isn't worth it.
pub fn lena_wipe(g: &mut Game, q: PlayerId, name: &str, src: Option<PermId>, _kind: Sym) -> Res<Option<bool>> {
    if name != LENA {
        return Ok(None);
    }
    let Some(s) = src.filter(|&s| live(g, s) && owner(g, s) == q && !no_sac(g, s)) else { return Ok(Some(false)) };
    let cap = epow(g, s);
    let saved: Vec<PermId> = creatures_of(g, q).into_iter().filter(|&m| m != s && epow(g, m) < cap).collect();
    let worth: f64 = saved.iter().map(|&m| pval(g, m)).sum();
    if worth < 4.0 {
        return Ok(Some(false));
    }
    crate::glog!(g, "    {} sacrifices {LENA}: {} creature(s) indestructible", pname(g, q), saved.len());
    die(g, s, "sac")?;
    for m in saved {
        if g.perm(m).on_bf {
            eot_kw(g, m, "indestructible");
        }
    }
    g.player_mut(q).stat("protection_used", 1);
    Ok(Some(true))
}

// ================================================================== Myrel, Shield of Argive
/// During your turn, your opponents can't cast spells (their abilities: rules::ability_locked).
fn myrel_lock(g: &Game, src: Src, caster: PlayerId, _c: CardId, _zone: Sym) -> bool {
    !live(g, src) || caster == owner(g, src) || g.active != Some(owner(g, src))
}

// ================================================================== Needleverge Pathway // Pillarverge Pathway
/// a pathway's two faces: (card, front colour, back colour)
const PATHWAYS: [(&str, &str, &str); 1] = [(PATHWAY, "R", "W")];

/// As it is played, the face whose colour p needs more: the coloured pips of its hand against its other lands'
/// sources of that colour
fn pathway_etb(g: &mut Game, p: PlayerId, l: PermId) -> Res {
    let name = g.db.get(g.perm(l).card()).name.clone();
    let Some(&(_, front, back)) = PATHWAYS.iter().find(|x| *x.0 == *name) else { return Ok(()) };
    let need = |g: &Game, col: &str| {
        let ch = col.chars().next().unwrap();
        let pips =
            g.player(p).hand.iter().map(|&c| g.db.get(c).pips.chars().filter(|&x| x == ch).count()).sum::<usize>();
        let have = g
            .player(p)
            .lands
            .iter()
            .filter(|&&x| x != l && crate::engine::mana::land_colors_now(g, p, x).has(ch))
            .count();
        pips as f64 - 0.8 * have as f64
    };
    let pick = if need(g, back) > need(g, front) { back } else { front };
    g.perm_mut(l).data.set(DataKey::Mode, Val::Str(intern(pick)));
    crate::glog!(g, "    {} is played as its {{{pick}}} face", name);
    Ok(())
}

/// the face it was played as; one put onto the battlefield another way is its front face
fn pathway_cols(g: &Game, _p: PlayerId, l: PermId) -> Colors {
    let name = &g.db.get(g.perm(l).card()).name;
    let front = PATHWAYS.iter().find(|x| *x.0 == **name).map_or("C", |x| x.1);
    match g.perm(l).data.get(DataKey::Mode) {
        Some(Val::Str(s)) => Colors::from_letters(s),
        _ => Colors::from_letters(front),
    }
}

// ================================================================== Phoenix Chick
/// combat::attack_triggers: p attacked with `atk`: Phoenix Chick in p's graveyard ("whenever you attack with three or
/// more creatures, you may pay {R}{R}: return it tapped and attacking with a +1/+1 counter"). Returns the creatures
/// that joined the attack.
pub fn gy_attack(g: &mut Game, p: PlayerId, atk: &[PermId]) -> Res<Vec<PermId>> {
    if atk.len() < 3 {
        return Ok(vec![]);
    }
    let chicks: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| &*g.db.get(c).name == CHICK).collect();
    let mut out = vec![];
    for c in chicks {
        if jailed(g) || !can_pay(g, p, 0, "RR", false) {
            break;
        }
        if !trigger_window_card(g, p, c, "pay {R}{R}: return it attacking")? {
            continue;
        }
        let Some(i) = g.player(p).gy.iter().position(|&x| x == c) else { continue };
        if !can_pay(g, p, 0, "RR", false) || !pay(g, p, 0, "RR", false)? {
            continue;
        }
        g.player_mut(p).gy.remove(i);
        let m = enter(g, p, c, Enter { sick: false, plus: 1, ..Enter::default() })?;
        if g.perm(m).on_bf {
            g.perm_mut(m).tapped = true;
            crate::glog!(g, "    {CHICK} returns tapped and attacking");
            out.push(m);
        }
    }
    Ok(out)
}

// ================================================================== Phyrexian Revoker
/// As this creature enters, choose a nonland card name: what Pithing Needle would name (t6_stax::needle_stops reads it)
fn revoker_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if let Some((name, _)) = crate::ai::decks::flute_pick(g, o) {
        g.perm_mut(src).data.set(DataKey::Imprint, Val::Card(name));
        crate::glog!(g, "    {REVOKER} names {}", g.db.get(name).name);
    }
    Ok(())
}

// ================================================================== Soulless Jailer
/// is a Soulless Jailer on the battlefield: permanent cards in graveyards can't enter the battlefield
pub fn jailed(g: &Game) -> bool {
    !g.hooks.is_empty()
        && g.players
            .iter()
            .filter(|q| q.alive)
            .any(|q| q.perms.iter().any(|&m| live(g, m) && !g.perm(m).neutered && card_name(g, m) == Some(JAILER)))
}

/// Players can't cast noncreature spells from graveyards or exile.
fn jailer_lock(g: &Game, src: Src, _caster: PlayerId, c: CardId, zone: Sym) -> bool {
    !live(g, src) || g.db.get(c).creature || !matches!(zone, "gy" | "exile")
}

// ================================================================== Sundering Eruption // Volcanic Fissure
/// the land Sundering Eruption destroys: the best opposing nonbasic, else a land of the most threatening opponent
fn eruption_target(g: &Game, p: PlayerId) -> Option<(PlayerId, PermId)> {
    if let Some(x) = crate::impls::fixes::best_opp_land(g, p) {
        return Some(x);
    }
    let q = crate::impls::t3::top_threat(g, p)?;
    g.player(q)
        .lands
        .iter()
        .copied()
        .find(|&l| !g.perm(l).tapped)
        .or(g.player(q).lands.first().copied())
        .map(|l| (q, l))
}

/// Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield
/// tapped, then shuffle. Creatures without flying can't block this turn.
fn eruption_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if let Some((q, l)) = eruption_target(g, p) {
        crate::impls::fixes::destroy_land(g, q, l)?;
        land_ramp(g, q, 1, true)?;
    }
    g.ground_noblock = Some(g.turn_stamp());
    crate::glog!(g, "    creatures without flying can't block this turn");
    Ok("gy")
}

/// the ground blockers the opponents have up against p's attack, and p's creatures that could attack now
fn ground_wall(g: &Game, p: PlayerId) -> (usize, usize) {
    let walls = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&b| live(g, b) && g.is_creature(b) && !g.perm(b).tapped && !flying(g, b))
        .count();
    let attackers = creatures_of(g, p).into_iter().filter(|&m| can_attack(g, m) && !flying(g, m)).count();
    (walls, attackers)
}

/// cast before combat when ground blockers stand in front of two or more ground attackers
fn eruption_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.active != Some(p) || g.step != Step::Main1 || g.player(p).combat_no > 0 {
        return 0;
    }
    let (walls, attackers) = ground_wall(g, p);
    if walls >= 1 && attackers >= 2 { 65 } else { 0 }
}

fn eruption_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    Ok(crate::impls::t6_lands::mdfc_land_opt(g, c, p, post).into_iter().collect())
}

/// cardcode::evasion_blocked: b can't block a (Sundering Eruption's turn: only creatures with flying block)
pub fn evasion_blocked(g: &Game, b: PermId, _a: PermId) -> bool {
    g.ground_noblock == Some(g.turn_stamp()) && !flying(g, b)
}

// ================================================================== combos
/// Rionya, Fire Dancer + Combat Celebrant: each combat Rionya makes a hasty copy of the Celebrant, which exerts for
/// another combat: infinite combats. Ready before combat on p's turn with both on the battlefield.
fn rionya_ready(g: &Game, p: PlayerId) -> Ready {
    let no = || (false, vec![], vec![]);
    if g.active != Some(p) || g.step != Step::Main1 || g.player(p).combat_no > 0 {
        return no();
    }
    let Some(r) = on_bf(g, p, RIONYA) else { return no() };
    let cel = g
        .player(p)
        .perms
        .iter()
        .copied()
        .find(|&m| live(g, m) && !g.perm(m).token && card_name(g, m) == Some(CELEBRANT));
    match cel {
        Some(c) => (true, vec![r, c], vec![]),
        None => no(),
    }
}

pub static COMBOS: [Combo; 1] = [Combo {
    name: "Rionya, Fire Dancer + Combat Celebrant",
    groups: &[&[RIONYA], &[CELEBRANT]],
    ready: rionya_ready,
    mana: (0, ""),
    finish: None,
    xcost: &[],
    steps: &[],
}];

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let has = |n: &str| db.id(n).is_some();
    if has(BREATH) {
        aura(r, db, BREATH, &BREATH_SPEC)?;
        r.card(db, BREATH)?.combat_damage = Some(breath_damage);
    }
    if has(FLARE) {
        r.card(db, FLARE)?.prio = Some(|_, _, _| 0); // a response only (PROTECTORS, `flare_last_chance`)
    }
    if has(NOON) {
        let c = r.card(db, NOON)?;
        c.can_cast = Some(one_spell);
        c.options = Some(noon_options);
    }
    if has(SIEGE) {
        let c = r.card(db, SIEGE)?;
        c.as_enters = Some(siege_as);
        c.trigger_copies = Some(siege_copies);
        c.upkeep = Some(siege_upkeep);
    }
    if has(ALEXIOS) {
        r.card(db, ALEXIOS)?.upkeep = Some(alexios_upkeep);
    }
    if has(BOROMIR) {
        r.card(db, BOROMIR)?.cast_counter = Some(boromir_counter);
    }
    if has(GINGERBRUTE) {
        r.card(db, GINGERBRUTE)?.options = Some(ginger_options);
    }
    if has(GREYMOND) {
        let c = r.card(db, GREYMOND)?;
        c.as_enters = Some(greymond_as);
        c.grant_kw = Some(greymond_kw);
        r.creature_pt.push(greymond_pt);
    }
    if has(LENA) {
        r.card(db, LENA)?.etb = Some(lena_etb);
    }
    if has(MYREL) {
        r.card(db, MYREL)?.can_cast = Some(myrel_lock);
    }
    for (name, _, _) in PATHWAYS {
        if has(name) {
            let c = r.card(db, name)?;
            c.land_etb = Some(pathway_etb);
            c.land_cols = Some(pathway_cols);
        }
    }
    if has(REVOKER) {
        r.card(db, REVOKER)?.etb = Some(revoker_etb);
    }
    if has(JAILER) {
        r.card(db, JAILER)?.can_cast = Some(jailer_lock);
    }
    if has(ERUPTION) {
        let c = r.card(db, ERUPTION)?;
        c.resolve = Some(eruption_resolve);
        c.prio = Some(eruption_prio);
        c.hand_options = Some(eruption_options);
        c.land_cols = Some(crate::impls::t6_lands::mdfc_cols);
    }
    Ok(())
}
