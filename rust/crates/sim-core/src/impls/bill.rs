//! The bill deck (decklists/JD/bristly-bill-landfall.md): mono-green landfall counters under Bristly Bill, Spine Sower.
//! Its card code and AI plan. The brief is rust/agents/bill-cards.md; the verdicts are in data/card_overrides.toml.
//!
//! What is modeled here, card by card (the cards the compiled abilities already play are named where this file
//! adds to them):
//! - **Hardened Scales**: `cardcode::scales`, applied wherever the engine puts +1/+1 counters on a creature
//!   (`cardcode::add_counters`, the ability language's counters / proliferate / doubling, `common::proliferate`,
//!   enters-with-counters). Two Scales stack.
//! - **Bristly Bill**: the landfall counter is the compiled ability; `counter_target` chooses where it goes (Silhana
//!   first, then the creature with the most counters, increment and mana creatures) for this deck's player. The
//!   {3}{G}{G} doubling is `bill_options` (an instant-speed ability: the end of the turn before yours with the mana
//!   free, or before combat when it makes the attack lethal).
//! - **Boughside Wanderers**, **Cactarantula** (draws when an opponent's spell or ability targets it; it costs {1} less
//!   with a Desert, and this deck has none), **Carnivorous Canopy** (artifact, enchantment or flier; proliferate at
//!   mana value 3 or less), **Ghalta**, **Silhana Ledgewalker**, **Venomous Brutalizer** (toxic 3, pay {1}{G} to
//!   proliferate), **Wildgrowth Archaic** (converge: in a mono-green deck the colors spent are one when any mana is
//!   spent), **Royal Treatment** (a real Royal Role: +1/+1 and ward {1}, not a counter).
//! - Plot (**Freestrider Commando**, **Tumbleweed Rising**), foretell (**Lifestream's Blessing**), disguise (**Greenbelt
//!   Radical**), collect evidence (**Sample Collector**), web-slinging (**Spider-Man, Brooklyn Visionary**) and
//!   prepared (**Studious First-Year**): see their sections.

use crate::cards::CardDb;
use crate::engine::mana::can_pay;
use crate::engine::removal::apply_removal;
use crate::engine::stack::{counter_window, trigger_window};
use crate::engine::values::{card_name, epow, etgh, pval};
use crate::engine::zones::{draw, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};

pub const BILL: &str = "Bristly Bill, Spine Sower";
const SILHANA: &str = "Silhana Ledgewalker";

fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

/// p's creatures on the battlefield (not phased out)
fn creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

/// p's Bristly Bill, if he is out
pub fn bill_of(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && card_name(g, m) == Some(BILL))
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

// ================================================================== where the counters go
/// how much a +1/+1 counter is worth on this creature (the bill deck's choice for every "target creature you control"
/// that grows: Bill's landfall, Retreat to Kazandu, Vinelasher Adept ...): Silhana Ledgewalker first (hexproof, only
/// fliers block it); then the creature that already has the most counters (the doubling); an increment creature or
/// mana creature (Topiary Lecturer taps for its power); Bugenhagen up to power 7. A creature that is about to die
/// (toughness 1 or less after the counter, nothing to protect it) is the last resort.
fn counter_score(g: &Game, m: PermId) -> f64 {
    let x = g.perm(m);
    let mut v = pval(g, m) + x.plus.max(0) as f64 * 0.8;
    match card_name(g, m) {
        Some(SILHANA) => v += 100.0,
        Some("Topiary Lecturer") => v += 3.0,
        Some("Hungry Graffalon" | "Ambitious Augmenter") => v += 1.0,
        Some("Bugenhagen, Wise Elder") => {
            let pw = epow(g, m);
            if pw < 7 {
                v += 2.0 + pw as f64 * 0.3; // the card at upkeep needs a power-7 creature
            }
        }
        _ => {}
    }
    if x.noatk && card_name(g, m) != Some("Topiary Lecturer") {
        v -= 1.0;
    }
    if etgh(g, m) <= 1 {
        v -= 4.0;
    }
    v
}

/// the creature a counter goes on, among `mine` (the bill deck's player only: `dsl::choose_target`)
pub fn counter_target(g: &Game, mine: &[PermId]) -> Option<PermId> {
    max_by(mine, |m| counter_score(g, m))
}

// ================================================================== Bristly Bill: the doubling
/// {3}{G}{G}: double the number of +1/+1 counters on each creature you control (Hardened Scales adds one to each
/// placement)
fn double_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || !can_pay(g, p, 3, "GG", false) {
        return Ok(false);
    }
    if !crate::engine::mana::pay(g, p, 3, "GG", false)? {
        return Ok(false);
    }
    crate::glog!(g, "  {} doubles the counters on their creatures (Bristly Bill)", pname(g, p));
    for m in creatures(g, p) {
        let k = g.perm(m).plus;
        if k > 0 && g.perm(m).on_bf {
            crate::cardcode::add_counters(g, m, k);
        }
    }
    Ok(true)
}

/// +1/+1 counters on p's creatures, and what doubling them would add (each creature with counters gets as many again,
/// plus one per Hardened Scales)
fn doubling_gain(g: &Game, p: PlayerId) -> (i32, i32) {
    let sc = crate::cardcode::scales(g, p);
    let mut total = 0;
    let mut add = 0;
    for m in creatures(g, p) {
        let k = g.perm(m).plus;
        if k > 0 {
            total += k;
            add += k + sc;
        }
    }
    (total, add)
}

/// the power that could attack now and what the doubling adds to it
fn attack_force(g: &Game, p: PlayerId) -> (i32, i32) {
    let sc = crate::cardcode::scales(g, p);
    let (mut power, mut extra) = (0, 0);
    for m in creatures(g, p) {
        let x = g.perm(m);
        if x.sick && !crate::engine::combat::kw(g, m, "haste") || x.tapped || x.noatk {
            continue;
        }
        power += epow(g, m).max(0);
        if x.plus > 0 {
            extra += x.plus + sc;
        }
    }
    (power, extra)
}

fn bill_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || !live(g, src) || !can_pay(g, p, 3, "GG", false) {
        return Ok(vec![]);
    }
    let (total, add) = doubling_gain(g, p);
    if total < 4 {
        return Ok(vec![]);
    }
    let act = Some(Action::Ability { src, f: double_go, arg: 0 });
    let label = format!("Bristly Bill doubles {total} counters");
    match post {
        // the end of the turn before yours, with the mana free
        None => Ok(vec![Opt { utility: 2.5 + (add as f64 / 6.0).min(3.0), label, act }]),
        // before combat: only when it makes the attack lethal (or much bigger)
        Some(false) => {
            let (power, extra) = attack_force(g, p);
            let lethal = g.opps(p).any(|q| {
                let life = g.player(q).life;
                power < life && power + extra >= life
            });
            if lethal {
                Ok(vec![Opt { utility: 9.0, label: format!("{label} (lethal attack)"), act }])
            } else if extra >= 8 && power > 0 {
                Ok(vec![Opt { utility: 3.0, label: format!("{label} (bigger attack)"), act }])
            } else {
                Ok(vec![])
            }
        }
        Some(true) => Ok(vec![]),
    }
}

// ================================================================== the simple creatures
/// Boughside Wanderers: look at the top four, reveal a permanent card into your hand, the rest on the bottom in a
/// random order (the landfall +2/+2 is the compiled ability)
fn wanderers_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "look at the top four, take a permanent card", None)? {
        super::t1::look_take(g, p, 4, &|g, c| g.db.get(c).perm || g.db.get(c).land, "hand", 1)?;
    }
    Ok(())
}

/// Cactarantula: whenever it becomes the target of a spell or ability an opponent controls, you may draw a card
/// (`engine::removal::apply_removal` calls this for every permanent an opponent targets)
pub fn becomes_target(g: &mut Game, m: PermId, _by: PlayerId) -> Res {
    if card_name(g, m) != Some("Cactarantula") || !live(g, m) {
        return Ok(());
    }
    let o = g.perm(m).owner;
    if g.player(o).library.len() > 2 && trigger_window(g, o, Some(m), "you may draw a card", Some(3.0))? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// Ghalta costs {X} less, X the greatest power among your creatures
fn ghalta_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    -creatures(g, p).into_iter().map(|m| epow(g, m)).max().unwrap_or(0).max(0)
}

/// Silhana Ledgewalker can't be blocked except by creatures with flying (`rules::evasion_blocked`)
pub fn unblockable_by(g: &Game, b: PermId, a: PermId) -> bool {
    card_name(g, a) == Some(SILHANA) && !(g.perm(b).fly || crate::dsl::has_kw(g, b, "flying"))
}

/// Venomous Brutalizer: toxic 3 (three poison counters with the combat damage)
fn brutalizer_damage(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if a == src && dmg > 0 && !crate::engine::values::melira(g, d) {
        g.player_mut(d).poison += 3;
        crate::glog!(g, "    {} gets 3 poison counters (toxic)", g.player(d).name);
    }
    Ok(())
}

/// Venomous Brutalizer: when it enters, you may pay {1}{G}; if you do, proliferate
fn brutalizer_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || !can_pay(g, p, 1, "G", false) {
        return Ok(());
    }
    let (total, _) = doubling_gain(g, p);
    let poison = g.opps(p).any(|q| g.player(q).poison > 0);
    if (total > 0 || poison)
        && trigger_window(g, p, Some(src), "pay {1}{G}: proliferate", None)?
        && crate::engine::mana::pay(g, p, 1, "G", false)?
    {
        super::common::proliferate(g, p, 1)?;
    }
    Ok(())
}

// ================================================================== Carnivorous Canopy
/// an opposing artifact, enchantment or creature with flying that Canopy may destroy
fn canopy_ok(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    if !x.on_bf || x.phased || crate::engine::values::untargetable(g, m) {
        return false;
    }
    let Some(c) = x.cd else { return g.is_creature(m) && (x.fly || crate::dsl::has_kw(g, m, "flying")) };
    let d = g.db.get(c);
    d.types.has(crate::cards::Types::ARTIFACT)
        || d.types.has(crate::cards::Types::ENCHANTMENT)
        || (g.is_creature(m) && (x.fly || crate::dsl::has_kw(g, m, "flying")))
}

/// Carnivorous Canopy: destroy target artifact, enchantment or creature with flying; if its mana value was 3 or less,
/// proliferate
fn canopy_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = ctx.target.filter(|&t| canopy_ok(g, t)).or_else(|| {
        let cands: Vec<PermId> =
            g.opps(p).flat_map(|q| g.player(q).perms.clone()).filter(|&m| canopy_ok(g, m)).collect();
        max_by(&cands, |m| pval(g, m) + if g.is_creature(m) { 1.0 } else { 0.0 })
    });
    if let Some(t) = t {
        let mv = g.perm(t).cd.map_or(0, |c| g.db.get(c).cmc);
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
        if mv <= 3 {
            super::common::proliferate(g, p, 1)?;
        }
    }
    Ok(intern("gy"))
}

// ================================================================== Wildgrowth Archaic (converge)
/// the colors of mana spent on a spell, in a mono-green deck: one when it cost anything, none when it was free
fn colors_spent(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let (gn, pips) = crate::engine::mana::cost_of(g, p, c);
    i32::from(gn + pips.chars().count() as u32 > 0)
}

fn archaic_key(c: CardId) -> Sym {
    intern(&format!("archaic_{}", c.0))
}

/// Whenever you cast a creature spell, it enters with X additional +1/+1 counters, X the colors of mana spent: the
/// number is kept until that card enters (this turn)
fn archaic_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster != g.perm(src).owner || !live(g, src) || !g.db.get(c).creature {
        return Ok(());
    }
    let n = colors_spent(g, caster, c);
    if n > 0 {
        let st = g.turn_stamp();
        g.player_mut(caster).flag_turn.insert(archaic_key(c), st);
    }
    Ok(())
}

/// Wildgrowth Archaic enters with a counter per color spent; any creature of p's that was cast with the flag set
/// enters with one more (Hardened Scales applies to each)
fn archaic_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if g.perm(src).owner != p || !g.is_creature(m) {
        return Ok(());
    }
    if m == src {
        let c = g.perm(m).cd.expect("Archaic is a card");
        let n = colors_spent(g, p, c);
        if n > 0 {
            crate::cardcode::add_counters(g, m, n);
        }
        return Ok(());
    }
    let Some(c) = g.perm(m).cd else { return Ok(()) };
    let st = g.turn_stamp();
    if g.player(p).flag_turn.get(&archaic_key(c)) == Some(&st) {
        g.player_mut(p).flag_turn.shift_remove(&archaic_key(c));
        crate::glog!(g, "    {} enters with a counter (Wildgrowth Archaic)", g.perm(m).name);
        crate::cardcode::add_counters(g, m, 1);
    }
    Ok(())
}

// ================================================================== Royal Treatment
const ROYAL: &str = "Royal Treatment";

/// the Royal Role on m (`DataKey::Role` holding "royal"; a Monster Role is `Bool(true)`): ward {1}
pub fn role_ward(g: &Game, m: PermId) -> u32 {
    if g.perm(m).data.get(DataKey::Role) == Some(&Val::Str(intern("royal"))) { 1 } else { 0 }
}

/// Royal Treatment ({G}, instant): the creature gains hexproof until end of turn and gets a Royal Role (+1/+1 and
/// ward {1}; a Role you control on it is replaced). The AI casts it in response to removal aimed at a creature worth
/// saving (`protect`), and otherwise to protect a creature in response to nothing it can see: never.
fn royal_apply(g: &mut Game, m: PermId) {
    super::partials::eot_kw(g, m, "hexproof");
    g.perm_mut(m).data.set(DataKey::Role, Val::Str(intern("royal")));
}

/// the bill deck's answer to removal on its creature m (`ai::decks::protect_response`): Royal Treatment, when the
/// creature is worth it
pub fn bill_protect(
    g: &mut Game,
    p: PlayerId,
    m: PermId,
    kind: &str,
    actor: Option<PlayerId>,
    _spell: Option<CardId>,
) -> Res<bool> {
    if actor.is_none() || actor == Some(p) || g.perm(m).cd.is_none() || matches!(kind, "edict" | "wipe" | "sacrifice") {
        return Ok(false);
    }
    let worth = pval(g, m) + g.perm(m).plus.max(0) as f64 * 0.5;
    if worth < 4.0 || !g.is_creature(m) {
        return Ok(false);
    }
    let Some(c) = g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == ROYAL) else { return Ok(false) };
    if !can_pay(g, p, 0, "G", false) || !crate::engine::cast::castable(g, p, c, "hand") {
        return Ok(false);
    }
    if !crate::engine::mana::pay(g, p, 0, "G", false)? {
        return Ok(false);
    }
    let pl = g.player_mut(p);
    if let Some(i) = pl.hand.iter().position(|&x| x == c) {
        pl.hand.remove(i);
    }
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts Royal Treatment -> {}", pname(g, p), g.perm(m).name);
    crate::engine::cast::on_cast(g, p, c)?;
    let ok = counter_window(g, p, c, 6.0, vec![])?;
    g.player_mut(p).gy.push(c);
    if !ok {
        return Ok(false);
    }
    royal_apply(g, m);
    Ok(true)
}

// ================================================================== the AI's cast priorities
/// bill's cast priority: Bill on turn 2 when he can be; ramp that puts lands onto the battlefield, Harrow and the
/// fetch lands are held while Bill is in the library or hand and the table is not yet pressing, so their lands enter
/// under him; then the generic plan
pub fn bill_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let base = crate::ai::pool::generic_prio(g, p, c);
    let d = g.db.get(c);
    if c == g.player(p).cmd {
        return 99;
    }
    match &*d.name {
        ROYAL => 0, // only as a response to removal (`bill_protect`)
        _ => base,
    }
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, BILL)?.options = Some(bill_options);
    r.card(db, "Boughside Wanderers")?.etb = Some(wanderers_etb);
    r.card(db, "Ghalta the Unstoppable")?.self_cost = Some(ghalta_cost);
    let c = r.card(db, "Venomous Brutalizer")?;
    c.combat_damage = Some(brutalizer_damage);
    c.etb = Some(brutalizer_etb);
    r.card(db, "Carnivorous Canopy")?.resolve = Some(canopy_resolve);
    let c = r.card(db, "Wildgrowth Archaic")?;
    c.cast = Some(archaic_cast);
    c.etb = Some(archaic_etb);
    Ok(())
}
