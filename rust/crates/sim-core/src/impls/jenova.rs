//! The jenova candidate deck (decklists/JD/candidates/jenova.md): Golgari +1/+1 counters and sacrifice under Jenova,
//! Ancient Calamity. Its card code and AI plan.
//!
//! The cards (each is Full in `cardgen audit --mine`; the verdicts are in data/card_overrides.toml):
//! - **Jenova, Ancient Calamity**: at the beginning of combat on your turn, counters equal to her power on up to one
//!   other creature, which becomes a Mutant (a subtype in `Perm::ttypes`); a Mutant you control dying on your turn draws
//!   cards equal to its power. The engine fires the `crew` event at the start of the first combat phase of a turn,
//!   before attackers are chosen (nothing in this deck crews), which is where "beginning of combat" is modeled: a
//!   second combat phase (no extra-combat card runs in the deck) does not trigger it again.
//! - **Dormant Grove // Gnarled Grovestrider**: the same beginning of combat; at toughness 6 or more the Grove becomes
//!   the 3/6 creature (animated, `DataKey::Flipped`), and other creatures you control have vigilance.
//! - **Forced Adaptation**, **Lion Umbra** (enchant modified creature, +3/+3, reach, vigilance, umbra armor): Auras.
//!   Lion Umbra is itself an Aura its controller controls, so the creature it enchants stays modified: there is
//!   nothing to check after it is attached.
//! - **Increment** (Ambitious Augmenter, Hungry Graffalon, Topiary Lecturer): a cast hook comparing the mana spent
//!   (what the spell costs now; X is not counted) with the smaller of power and toughness.
//! - **Adventurous Eater** (enters prepared: a copy of Have a Bite for {B}), **Efflorescence** (infusion),
//!   **Old-Growth Educator** (infusion), **Essenceknit Scholar** (a Pest, and the end-step draw), **Edgar, Moonlit
//!   Sovereign**, **Teacher's Pest** (returns from the graveyard), **Vinelasher Adept** (basic landcycling {2}).
//!
//! Two things are not rules text but the AI's: Jenova's `pval` is high, so every "target creature you control" choice
//! (counter spells, Vinelasher Adept, Dormant Grove, Forced Adaptation) lands on her first, and sacrificing a Mutant
//! on your own turn is cheap (`mutant_sac_worth`, used by the engine's sacrifice choices and by `jenova_options`).

use super::common::{AURA, AuraSpec, aura, outlets, sac_through};
use crate::ai::pool::generic_prio;
use crate::cards::{CardDb, Colors};
use crate::engine::cast::{cast_card, parse_cost};
use crate::engine::combat::kw;
use crate::engine::life::gain;
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::stack::trigger_window;
use crate::engine::values::{card_name, epow, etgh, has_type, pval};
use crate::engine::zones::{Enter, Tokens, die, discard_cards, draw, enter, land_to_hand, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

pub const JENOVA: &str = "Jenova, Ancient Calamity";
const GROVE: &str = "Dormant Grove";
const EATER: &str = "Adventurous Eater";
const EDGAR: &str = "Edgar, Moonlit Sovereign";
const EFFLORESCENCE: &str = "Efflorescence";
const FORCED: &str = "Forced Adaptation";
const LION: &str = "Lion Umbra";

fn mutant() -> Sym {
    intern("mutant")
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn pn(g: &Game, m: PermId) -> String {
    g.perm(m).name.to_string()
}

/// p's Jenova, on the battlefield
fn jenova_of(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.on_bf && !x.phased && !x.neutered && card_name(g, m) == Some(JENOVA)
    })
}

fn is_mutant(g: &Game, m: PermId) -> bool {
    has_type(g, m, "mutant")
}

/// p's creatures (not phased out)
fn creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect()
}

/// +1/+1 counters on m, after doubling effects
fn put_counters(g: &mut Game, p: PlayerId, m: PermId, n: i32) {
    let k = n * crate::dsl::counter_mult(g, p);
    crate::cardcode::add_counters(g, m, k);
}

fn gained_life(g: &Game, p: PlayerId) -> bool {
    g.player(p).gained_turn.is_some_and(|x| x.0 == g.turn_stamp() && x.1 > 0)
}

// ================================================================== Jenova
/// who becomes the Mutant: the creature that gains the most from counters: big, evasive, already a Mutant (the counters
/// stack up for the draw), a body that is fine to sacrifice
fn mutant_score(g: &Game, m: PermId) -> f64 {
    let x = g.perm(m);
    let mut v = pval(g, m);
    if x.fly || kw(g, m, "flying") || kw(g, m, "menace") || kw(g, m, "unblockable") {
        v += 3.0;
    }
    if kw(g, m, "trample") || kw(g, m, "deathtouch") {
        v += 1.0;
    }
    if is_mutant(g, m) {
        v += 1.5;
    }
    if x.token || !x.is_cmd {
        v += 0.5;
    }
    if x.noatk && !is_mutant(g, m) {
        v -= 1.0;
    }
    v
}

/// Jenova's trigger (the beginning of combat on your turn): counters equal to her power on up to one other creature
fn jenova_combat(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let x = g.perm(src);
    if x.owner != p || x.phased || x.neutered || g.active != Some(p) {
        return Ok(());
    }
    let cands: Vec<PermId> = creatures(g, p).into_iter().filter(|&m| m != src).collect();
    let n = epow(g, src);
    // HUMAN(phase 9): a person picks the target; the AI takes the best
    let Some(t) = max_by(&cands, |m| mutant_score(g, m)) else { return Ok(()) };
    if n <= 0 {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), &format!("{n} +1/+1 counters on {}", pn(g, t)), Some(5.0))? {
        return Ok(());
    }
    if !g.perm(t).on_bf || !g.is_creature(t) {
        return Ok(());
    }
    put_counters(g, p, t, n);
    if !g.perm(t).ttypes.contains(&mutant()) {
        g.perm_mut(t).ttypes.push(mutant());
    }
    crate::glog!(g, "    Jenova puts {n} +1/+1 counters on {}: it becomes a Mutant", pn(g, t));
    Ok(())
}

/// whenever a Mutant you control dies during your turn, you draw cards equal to its power
fn jenova_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let p = g.perm(src).owner;
    let x = g.perm(m);
    if x.owner != p || g.active != Some(p) || !g.is_creature(m) || !is_mutant(g, m) || g.perm(src).phased {
        return Ok(());
    }
    let n = epow(g, m);
    if n <= 0 || !trigger_window(g, p, Some(src), &format!("draw {n} cards"), Some(4.0))? {
        return Ok(());
    }
    crate::glog!(g, "    Jenova: the Mutant {} died, {} draws {n}", pn(g, m), pname(g, p));
    draw(g, p, n as u32, false)
}

/// engine.sac_worth for a Mutant: on its controller's turn with Jenova out, it is worth the cards it draws when it
/// dies (so the engine's sacrifice choices feed it to outlets and sacrifice costs first)
pub fn mutant_sac_worth(g: &Game, m: PermId, v: f64) -> f64 {
    let x = g.perm(m);
    let p = x.owner;
    if g.active != Some(p) || !x.ttypes.contains(&mutant()) || x.is_cmd || jenova_of(g, p).is_none() {
        return v;
    }
    let draws = epow(g, m);
    if draws >= 3 { (0.25 * v - 0.5 * draws as f64).min(v) } else { v }
}

/// Jenova's plan: after combat (or at the end of the turn) sacrifice the biggest Mutant to a free sacrifice outlet, or
/// to Carnage Altar for {3}, when it draws three or more cards
fn jenova_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.active != Some(p) || post == Some(false) || g.perm(src).phased {
        return Ok(vec![]);
    }
    let muts: Vec<PermId> = creatures(g, p)
        .into_iter()
        .filter(|&m| m != src && is_mutant(g, m) && !g.perm(m).is_cmd && epow(g, m) >= 3)
        .collect();
    let Some(m) = max_by(&muts, |x| epow(g, x) as f64) else { return Ok(vec![]) };
    let draws = epow(g, m);
    let mut out = vec![];
    if let Some(&o) = outlets(g, p).iter().find(|&&o| o != m) {
        out.push(Opt {
            utility: (1.0 + 0.7 * draws as f64).min(7.0),
            label: format!("sacrifice the Mutant {} ({}): draw {draws}", pn(g, m), card_name(g, o).unwrap_or("")),
            act: Some(Action::Ability { src, f: sac_mutant, arg: m.0 as i64 | (o.0 as i64) << 32 }),
        });
    }
    if can_pay(g, p, 3, "", false)
        && let Some(altar) = g.player(p).perms.iter().copied().find(|&x| {
            let y = g.perm(x);
            y.on_bf && !y.phased && card_name(g, x) == Some("Carnage Altar")
        })
    {
        out.push(Opt {
            utility: (0.6 + 0.6 * draws as f64).min(6.0),
            label: format!("sacrifice the Mutant {} (Carnage Altar): draw {draws}, then 1 more", pn(g, m)),
            act: Some(Action::Ability { src, f: sac_mutant, arg: m.0 as i64 | (altar.0 as i64) << 32 | 1 << 62 }),
        });
    }
    Ok(out)
}

fn sac_mutant(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId((arg & 0xffff_ffff) as u32);
    let o = PermId(((arg >> 32) & 0x3fff_ffff) as u32);
    let altar = (arg >> 62) & 1 == 1;
    let (mo, oo) = (g.perm(m), g.perm(o));
    if !mo.on_bf || !oo.on_bf || mo.owner != p || oo.owner != p || g.perm(src).phased {
        return Ok(false);
    }
    if altar {
        if !can_pay(g, p, 3, "", false) {
            return Ok(false);
        }
        pay(g, p, 3, "", false)?;
        crate::glog!(g, "  {} sacrifices {} to Carnage Altar", pname(g, p), pn(g, m));
        die(g, m, "sac")?;
        draw(g, p, 1, false)?;
    } else {
        sac_through(g, p, o, m)?;
    }
    Ok(true)
}

// ================================================================== Dormant Grove // Gnarled Grovestrider
fn grove_flipped(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Flipped)
}

/// beginning of combat on your turn: a +1/+1 counter on target creature you control; then if it has toughness 6 or
/// more, the Grove transforms into Gnarled Grovestrider, a 3/6 Treefolk with vigilance
fn grove_combat(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let x = g.perm(src);
    if x.owner != p || x.phased || g.active != Some(p) || grove_flipped(g, src) {
        return Ok(());
    }
    let cands = creatures(g, p);
    let Some(t) = max_by(&cands, |m| pval(g, m)) else { return Ok(()) };
    if !trigger_window(g, p, Some(src), &format!("a +1/+1 counter on {}", pn(g, t)), None)? || !g.perm(t).on_bf {
        return Ok(());
    }
    put_counters(g, p, t, 1);
    if etgh(g, t) >= 6 && !grove_flipped(g, src) {
        let x = g.perm_mut(src);
        x.data.set(DataKey::Flipped, Val::Bool(true));
        x.data.set(DataKey::Anim, Val::Bool(true));
        x.pow = 3;
        x.tgh = 6;
        x.vig = true;
        x.ttypes = vec![intern("treefolk")];
        x.name = intern("Gnarled Grovestrider");
        crate::glog!(g, "    Dormant Grove transforms into Gnarled Grovestrider");
    }
    Ok(())
}

/// Gnarled Grovestrider: other creatures you control have vigilance
fn grove_kw(g: &Game, src: Src, m: PermId, k: Sym) -> bool {
    k == "vigilance" && m != src && grove_flipped(g, src) && g.perm(m).owner == g.perm(src).owner
}

// ================================================================== Auras
/// has a counter on it, or an Equipment or Aura its controller controls attached
fn modified(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.plus != 0
        || x.data.truthy(DataKey::Counters)
        || g.player(x.owner).perms.iter().any(|&a| a != m && g.perm(a).on_bf && g.perm(a).attached == Some(m))
}

fn modified_ok(g: &Game, _p: PlayerId, m: PermId) -> bool {
    modified(g, m)
}

static FORCED_ADAPTATION: AuraSpec = AURA;
static LION_UMBRA: AuraSpec =
    AuraSpec { pow: 3, tgh: 3, kws: &["reach", "vigilance"], umbra: true, host_ok: Some(modified_ok), ..AURA };

/// Forced Adaptation: at the beginning of your upkeep, a +1/+1 counter on enchanted creature
fn forced_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let x = g.perm(src);
    if x.owner != p || x.phased {
        return Ok(());
    }
    let Some(h) = x.attached.filter(|&h| g.perm(h).on_bf) else { return Ok(()) };
    if trigger_window(g, p, Some(src), &format!("a +1/+1 counter on {}", pn(g, h)), None)? && g.perm(h).on_bf {
        put_counters(g, p, h, 1);
    }
    Ok(())
}

// ================================================================== increment
/// mana spent on the spell being cast now (a flashback cost, or what the card costs now; a free cast spent none)
fn mana_spent(g: &Game, p: PlayerId, c: CardId) -> u32 {
    let Some((cc, ctx, zone)) = &g.cur_cast else { return 0 };
    if *cc != c || ctx.paid_otherwise {
        return 0;
    }
    if *zone == "gy" {
        let fb = g.db.get(c).tags.str(Tag::Fb).unwrap_or("");
        let (n, rest) = parse_cost(fb);
        return n + rest.chars().count() as u32;
    }
    let (gn, pips) = cost_of(g, p, c);
    gn + pips.chars().count() as u32
}

/// Increment: whenever you cast a spell, if the mana spent is greater than this creature's power or toughness (the
/// smaller of the two), a +1/+1 counter on it
fn increment(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let p = g.perm(src).owner;
    if caster != p || g.perm(src).phased || g.perm(src).neutered {
        return Ok(());
    }
    let low = epow(g, src).min(etgh(g, src));
    if (mana_spent(g, p, c) as i32) <= low {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "increment: a +1/+1 counter", None)? && g.perm(src).on_bf {
        put_counters(g, p, src, 1);
    }
    Ok(())
}

/// Ambitious Augmenter: when it dies with counters on it, a 0/0 green and blue Fractal gets those counters
fn augmenter_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let (p, n) = (g.perm(m).owner, g.perm(m).plus);
    if n <= 0 || !trigger_window(g, p, Some(m), &format!("a Fractal with {n} counters"), None)? {
        return Ok(());
    }
    let spec =
        Tokens { color: Some(Colors::from_letters("GU")), types: vec!["fractal"], tgh: Some(1), ..Tokens::new(1, 1) };
    for t in make_tokens(g, p, spec)? {
        // a 0/0 with the counters (made 1/1 so that it isn't put into the graveyard before they arrive)
        let x = g.perm_mut(t);
        x.pow = 0;
        x.tgh = 0;
        x.plus = n;
    }
    Ok(())
}

/// Topiary Lecturer: {T}: add an amount of {G} equal to its power
fn lecturer_mana(g: &Game, _p: PlayerId, m: PermId) -> u32 {
    epow(g, m).max(0) as u32
}

// ================================================================== infusion, prepared and the rest
/// Old-Growth Educator: infusion: two +1/+1 counters on it if you gained life this turn
fn educator_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || !gained_life(g, p) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "infusion: two +1/+1 counters", None)? && g.perm(src).on_bf {
        put_counters(g, p, src, 2);
    }
    Ok(())
}

/// Essenceknit Scholar: a 1/1 black and green Pest (it gains 1 life when it attacks: `cardcode::keyword_attack`)
fn scholar_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || !trigger_window(g, p, Some(src), "a Pest token", None)? {
        return Ok(());
    }
    let spec = Tokens { color: Some(Colors::from_letters("BG")), types: vec!["pest"], ..Tokens::new(1, 1) };
    make_tokens(g, p, spec)?;
    Ok(())
}

/// Essenceknit Scholar: at the beginning of your end step, if a creature died under your control this turn, draw a card
fn scholar_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner != p || g.perm(src).phased || g.player(p).creature_died != Some(g.turn_stamp()) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "draw a card", None)? {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// Edgar: at the beginning of your end step, if you didn't cast a spell this turn, two +1/+1 counters on him
fn edgar_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner != p || g.perm(src).phased || g.player(p).spells_this_turn > 0 {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "two +1/+1 counters", None)? && g.perm(src).on_bf {
        put_counters(g, p, src, 2);
    }
    Ok(())
}

fn edgar_targets(g: &Game, p: PlayerId) -> Vec<PermId> {
    creatures(g, p).into_iter().filter(|&m| g.perm(m).plus > 0).collect()
}

/// Edgar, {4}{G}: a +1/+1 counter on each creature you control with a +1/+1 counter on it
fn edgar_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || g.perm(src).phased || !can_pay(g, p, 4, "G", false) {
        return Ok(vec![]);
    }
    let n = edgar_targets(g, p).len();
    // spare mana only: in the main phases the hand's spells come first
    if n < 3 || (post.is_some() && g.active == Some(p) && g.player(p).hand.iter().any(|&c| !g.db.get(c).land)) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: (0.5 * n as f64).min(4.0),
        label: format!("Edgar: a counter on each of {n} creatures"),
        act: Some(Action::Ability { src, f: edgar_go, arg: 0 }),
    }])
}

fn edgar_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !g.perm(src).on_bf || !pay(g, p, 4, "G", false)? {
        return Ok(false);
    }
    crate::glog!(g, "  {} activates Edgar ({{4}}{{G}})", pname(g, p));
    for m in edgar_targets(g, p) {
        put_counters(g, p, m, 1);
    }
    Ok(true)
}

/// Adventurous Eater enters prepared
fn eater_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.perm_mut(m).data.set(DataKey::Prepared, Val::Bool(true));
    }
    Ok(())
}

/// Have a Bite (the prepared copy, {B}): a +1/+1 counter on target creature, and you gain 1 life. The copy is not a
/// card of the database, so it can't be countered (the cast is still counted: Edgar, magecraft, Rhystic Study ...).
fn eater_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p || x.phased || !x.data.truthy(DataKey::Prepared) || !can_pay(g, p, 0, "B", false) {
        return Ok(vec![]);
    }
    let Some(t) = max_by(&creatures(g, p), |m| pval(g, m)) else { return Ok(vec![]) };
    let u = 1.8 + if jenova_of(g, p) == Some(t) { 1.0 } else { 0.0 };
    Ok(vec![Opt {
        utility: if post.is_none() { u - 0.8 } else { u },
        label: "Have a Bite (prepared copy)".into(),
        act: Some(Action::Ability { src, f: bite_go, arg: t.0 as i64 }),
    }])
}

fn bite_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if !g.perm(src).data.truthy(DataKey::Prepared) || !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(false));
    crate::impls::mine::cast_copy_with(g, p, None, None, false, 2.0, &mut |g| {
        let t = if g.perm(t).on_bf { Some(t) } else { max_by(&creatures(g, p), |m| pval(g, m)) };
        if let Some(t) = t {
            put_counters(g, p, t, 1);
        }
        gain(g, p, 1)
    })?;
    Ok(true)
}

/// Efflorescence ({2}{G}, instant): two +1/+1 counters on target creature; infusion: if you gained life this turn, it
/// also gains trample and indestructible until end of turn
fn efflorescence(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = ctx.target.filter(|&t| g.perm(t).on_bf && g.is_creature(t)).or_else(|| {
        let cs = creatures(g, p);
        max_by(&cs, |m| pval(g, m))
    });
    let Some(t) = t else { return Ok("gy") };
    put_counters(g, p, t, 2);
    if gained_life(g, p) {
        for k in ["trample", "indestructible"] {
            let k = intern(k);
            let x = g.perm_mut(t);
            if !x.eot_kw.contains(&k) {
                x.eot_kw.push(k);
            }
        }
        crate::glog!(g, "    Efflorescence (infusion): {} gains trample and indestructible", pn(g, t));
    }
    Ok("gy")
}

fn efflorescence_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !can_pay(g, p, 2, "G", false) {
        return Ok(vec![]);
    }
    let Some(t) = max_by(&creatures(g, p), |m| pval(g, m)) else { return Ok(vec![]) };
    let mut u = if g.active == Some(p) && post == Some(false) { 2.6 } else { 1.4 };
    if jenova_of(g, p) == Some(t) {
        u += 0.8; // her power is every combat's counters
    }
    if gained_life(g, p) {
        u += 0.4;
    }
    Ok(vec![Opt {
        utility: u,
        label: "Efflorescence".into(),
        act: Some(Action::Plan { f: efflorescence_go, arg: c.0 as i64 | (t.0 as i64) << 16 }),
    }])
}

fn efflorescence_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg & 0xffff) as u16);
    let t = PermId((arg >> 16) as u32);
    if !g.player(p).hand.contains(&c) || !g.perm(t).on_bf || !pay(g, p, 2, "G", false)? {
        return Ok(false);
    }
    cast_card(g, p, c, "hand", Ctx { target: Some(t), ..Ctx::default() })?;
    Ok(true)
}

/// Teacher's Pest, {B}{G}: return it from your graveyard to the battlefield tapped
fn pest_gy(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !can_pay(g, p, 0, "BG", false) || !g.player(p).gy.contains(&c) {
        return Ok(vec![]);
    }
    let sac = jenova_of(g, p).is_some() || !outlets(g, p).is_empty();
    let u = if sac { 1.6 } else { 0.9 };
    Ok(vec![Opt {
        utility: if post.is_none() { u + 0.4 } else { u },
        label: "Teacher's Pest (return from the graveyard)".into(),
        act: Some(Action::Plan { f: pest_go, arg: c.0 as i64 }),
    }])
}

fn pest_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !can_pay(g, p, 0, "BG", false) {
        return Ok(false);
    }
    let Some(i) = g.player(p).gy.iter().position(|&x| x == c) else { return Ok(false) };
    pay(g, p, 0, "BG", false)?;
    g.player_mut(p).gy.remove(i);
    crate::glog!(g, "  {} returns Teacher's Pest from the graveyard", pname(g, p));
    let m = enter(g, p, c, Enter::default())?;
    g.perm_mut(m).tapped = true;
    Ok(true)
}

/// Vinelasher Adept, basic landcycling {2}: when the hand has no land to play, or the Adept can't be cast soon
fn adept_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !can_pay(g, p, 2, "", false) || g.player(p).library.is_empty() {
        return Ok(vec![]);
    }
    let lands_in_hand = g.player(p).hand.iter().filter(|&&x| g.db.get(x).land).count();
    let on_bf = g.player(p).lands.len();
    let u = if lands_in_hand == 0 && on_bf < 6 {
        2.4
    } else if on_bf < 5 && lands_in_hand < 2 {
        1.2
    } else if post.is_none() && on_bf < 8 {
        0.6
    } else {
        return Ok(vec![]);
    };
    Ok(vec![Opt {
        utility: u,
        label: "Vinelasher Adept (basic landcycling)".into(),
        act: Some(Action::Plan { f: adept_cycle, arg: c.0 as i64 }),
    }])
}

fn adept_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    crate::glog!(g, "  {} basic landcycles Vinelasher Adept", pname(g, p));
    discard_cards(g, p, &[c])?;
    land_to_hand(g, p)?;
    Ok(true)
}

// ================================================================== the AI
/// jenova's cast priorities: Jenova first (she is the engine), Auras only with a creature to carry them, counter
/// makers once there is a creature to grow; the rest by the generic priorities
pub fn jenova_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let name = &*g.db.get(c).name;
    let cs = creatures(g, p);
    let jen = jenova_of(g, p).is_some();
    match name {
        JENOVA => 90,
        FORCED => {
            if cs.is_empty() {
                0
            } else if jen {
                72
            } else {
                48
            }
        }
        LION => {
            if cs.iter().any(|&m| modified(g, m)) {
                if jen { 70 } else { 56 } // umbra armor on the commander first
            } else {
                0
            }
        }
        GROVE => {
            if cs.is_empty() {
                0
            } else if jen {
                68
            } else {
                52
            }
        }
        EDGAR | "Essenceknit Scholar" | "Hornet Queen" => generic_prio(g, p, c) + 4,
        _ => generic_prio(g, p, c),
    }
}

fn zero(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, JENOVA)?;
    c.crew = Some(jenova_combat); // beginning of combat (see the module doc)
    c.dies = Some(jenova_dies);
    c.options = Some(jenova_options);
    c.pval = Some(9.0);
    let c = r.card(db, GROVE)?;
    c.crew = Some(grove_combat);
    c.grant_kw = Some(grove_kw);
    aura(r, db, FORCED, &FORCED_ADAPTATION)?;
    r.card(db, FORCED)?.upkeep = Some(forced_upkeep);
    aura(r, db, LION, &LION_UMBRA)?;
    for name in ["Ambitious Augmenter", "Hungry Graffalon", "Topiary Lecturer"] {
        r.card(db, name)?.cast = Some(increment);
    }
    r.card(db, "Ambitious Augmenter")?.self_dies = Some(augmenter_dies);
    r.card(db, "Topiary Lecturer")?.dyn_mana_perm = Some(lecturer_mana);
    r.card(db, "Old-Growth Educator")?.etb = Some(educator_etb);
    let c = r.card(db, "Essenceknit Scholar")?;
    c.etb = Some(scholar_etb);
    c.end_step = Some(scholar_end);
    let c = r.card(db, EDGAR)?;
    c.end_step = Some(edgar_end);
    c.options = Some(edgar_options);
    let c = r.card(db, EATER)?;
    c.etb = Some(eater_etb);
    c.options = Some(eater_options);
    let c = r.card(db, EFFLORESCENCE)?;
    c.resolve = Some(efflorescence);
    c.prio = Some(zero);
    c.hand_options = Some(efflorescence_hand);
    r.card(db, "Teacher's Pest")?.gy_options = Some(pest_gy);
    r.card(db, "Vinelasher Adept")?.hand_options = Some(adept_hand);
    Ok(())
}
