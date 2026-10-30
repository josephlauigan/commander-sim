//! Card code for the green counters and landfall cards (Hardened Scales, Bristly Bill, the Canopies, plot, foretell,
//! disguise ...), keyed by card name so every deck that plays one of these cards gets it (the Titania deck above all).
//! The verdicts are in data/card_overrides.toml; there is no deck-specific AI here.
//!
//! What is modeled here, card by card (the cards the compiled abilities already play are named where this file
//! adds to them):
//! - **Hardened Scales**: an entry of the replacement step (`engine::repl`), applied wherever the engine puts +1/+1
//!   counters on a creature (`repl::add_counters`, the ability language's counters / proliferate / doubling,
//!   `common::proliferate`, enters-with-counters). Two Scales stack.
//! - **Bristly Bill**: the landfall counter is the compiled ability (the engine picks the target); the {3}{G}{G}
//!   doubling is `bill_options` (an instant-speed ability: the end of the turn before yours with the mana free, or
//!   before combat when it makes the attack lethal through the blockers).
//! - **Boughside Wanderers**, **Cactarantula** (draws when an opponent's spell or ability targets it; it costs {1} less
//!   with a Desert, and the decks that play it have no Desert), **Carnivorous Canopy** (artifact, enchantment or flier; proliferate at
//!   mana value 3 or less), **Ghalta**, **Silhana Ledgewalker**, **Venomous Brutalizer** (DSL toxic 3, pay {1}{G} to
//!   proliferate), **Wildgrowth Archaic** (converge: in a mono-green deck, the colors spent are one when any mana is
//!   spent), **Royal Treatment** (a real Royal Role: +1/+1 and ward {1}, not a counter).
//! - Plot (**Freestrider Commando**, **Tumbleweed Rising**), foretell (**Lifestream's Blessing**), disguise (**Greenbelt
//!   Radical**, face down only), collect evidence (**Sample Collector**), web-slinging (**Spider-Man, Brooklyn Visionary**) and
//!   prepared (**Studious First-Year**): see their sections.

use crate::cards::CardDb;
use crate::engine::cast::parse_cost;
use crate::engine::mana::can_pay;
use crate::engine::mana::cost_of;
use crate::engine::mana::pay;
use crate::engine::removal::apply_removal;
use crate::engine::repl::{self, Ctr, On, Path};
use crate::engine::stack::{counter_window, may_window, trigger_window};
use crate::engine::values::{card_name, epow, etgh, pval};
use crate::engine::zones::{discard_cards, land_to_hand};
use crate::engine::zones::{draw, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

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
/// how much a +1/+1 counter is worth on this creature (the choice for every "target creature you control"
/// that grows, here Additive Evolution and Sample Collector): Silhana Ledgewalker first (hexproof, only
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

/// the creature a +1/+1 counter goes on, among `mine`
fn counter_target(g: &Game, mine: &[PermId]) -> Option<PermId> {
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
            repl::add_counters(g, m, k);
        }
    }
    Ok(true)
}

/// +1/+1 counters on p's creatures, and what doubling them would add (each creature with counters gets as many again,
/// plus one per Hardened Scales)
fn doubling_gain(g: &Game, p: PlayerId) -> (i32, i32) {
    let sc = repl::scales(g, p); // an estimate: Hardened Scales on each placement
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

/// the damage p's untapped attackers get through to q, all-out at q: each blocker of q that can block chumps one
/// of the biggest attackers it can legally block (menace takes two blockers); trample passes the excess over the
/// blocker's toughness. `doubled`: after the doubling (each creature with counters gets as many again, plus Scales).
fn damage_through(g: &Game, p: PlayerId, q: PlayerId, doubled: bool) -> i32 {
    use crate::engine::combat::{can_block, kw};
    let sc = repl::scales(g, p); // an estimate: Hardened Scales on each placement
    let mut atk: Vec<(PermId, i32)> = vec![];
    for m in creatures(g, p) {
        let x = g.perm(m);
        if x.sick && !kw(g, m, "haste") || x.tapped || x.noatk {
            continue;
        }
        let mut pw = epow(g, m);
        if doubled && x.plus > 0 {
            pw += x.plus + sc;
        }
        if pw > 0 {
            atk.push((m, pw));
        }
    }
    atk.sort_by_key(|a| -a.1);
    let mut blockers: Vec<PermId> =
        g.player(q).perms.iter().copied().filter(|&b| live(g, b) && g.is_creature(b) && !g.perm(b).tapped).collect();
    let mut total = 0;
    for (a, pw) in atk {
        let mut cand: Vec<PermId> = blockers.iter().copied().filter(|&b| can_block(g, b, a)).collect();
        let need = if kw(g, a, "menace") { 2 } else { 1 };
        if cand.len() < need {
            total += pw;
            continue;
        }
        // use the blockers that can block the fewest of the remaining attackers' kind: the smallest first
        cand.sort_by_key(|&b| etgh(g, b));
        let used: Vec<PermId> = cand.into_iter().take(need).collect();
        blockers.retain(|b| !used.contains(b));
        if kw(g, a, "trample") {
            let tough: i32 = used.iter().map(|&b| etgh(g, b).max(0)).sum();
            total += (pw - tough).max(0);
        }
    }
    total
}

/// the opponent the doubling makes dead through their blockers (the one with the least life)
fn lethal_victim(g: &Game, p: PlayerId) -> Option<PlayerId> {
    g.opps(p)
        .filter(|&q| {
            let life = g.player(q).life;
            damage_through(g, p, q, false) < life && damage_through(g, p, q, true) >= life
        })
        .min_by_key(|&q| g.player(q).life)
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
        // before combat: only when it makes the attack lethal through the blockers
        Some(false) => {
            if lethal_victim(g, p).is_some() {
                let act = Some(Action::Ability { src, f: double_go, arg: 0 });
                Ok(vec![Opt { utility: 9.0, label: format!("{label} (lethal attack)"), act }])
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
    if (g.player(o).library.len() > 2 || (crate::human::is_human(g, o) && !g.player(o).library.is_empty()))
        && may_window(g, o, Some(m), "you may draw a card", Some(3.0))?
    {
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

// ================================================================== Carnivorous Canopy and Crushing Canopy
/// an opposing permanent a Canopy may destroy: Carnivorous Canopy takes an artifact, an enchantment or a creature with
/// flying; Crushing Canopy an enchantment or a creature with flying
fn canopy_ok(g: &Game, m: PermId, artifacts: bool) -> bool {
    let x = g.perm(m);
    if !x.on_bf || x.phased || crate::engine::values::untargetable(g, m) {
        return false;
    }
    let flier = g.is_creature(m) && (x.fly || crate::dsl::has_kw(g, m, "flying"));
    let Some(c) = x.cd else { return flier };
    let d = g.db.get(c);
    flier || d.types.has(crate::cards::Types::ENCHANTMENT) || (artifacts && d.types.has(crate::cards::Types::ARTIFACT))
}

/// the permanent a Canopy destroys: the chosen target if it is still legal, else the best of theirs
fn canopy_target(g: &Game, p: PlayerId, ctx: &Ctx, artifacts: bool) -> Option<PermId> {
    ctx.target.filter(|&t| canopy_ok(g, t, artifacts)).or_else(|| {
        let cands: Vec<PermId> =
            g.opps(p).flat_map(|q| g.player(q).perms.clone()).filter(|&m| canopy_ok(g, m, artifacts)).collect();
        max_by(&cands, |m| pval(g, m) + if g.is_creature(m) { 1.0 } else { 0.0 })
    })
}

/// Carnivorous Canopy: destroy target artifact, enchantment or creature with flying; if its mana value was 3 or less,
/// proliferate
fn canopy_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = canopy_target(g, p, ctx, true) {
        let mv = g.perm(t).cd.map_or(0, |c| g.db.get(c).cmc);
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
        if mv <= 3 {
            super::common::proliferate(g, p, 1)?;
        }
    }
    Ok(intern("gy"))
}

/// Crushing Canopy (instant): destroy target creature with flying, or target enchantment. The compiled spell would
/// be cast at any creature and find no flier; this one is offered only with a legal target (`tgt=fe`).
fn crushing_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = canopy_target(g, p, ctx, false) {
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
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

/// +1/+1 counters a creature card enters with because of Wildgrowth Archaic (`engine::zones::enter`, for a card that was
/// cast): the Archaic itself enters with one per color spent (converge); any creature cast while an Archaic was out
/// (the flag its cast hook set) enters with that many more. Hardened Scales applies to the total.
pub fn enters_with(g: &Game, p: PlayerId, cd: CardId, was_cast: bool) -> i32 {
    if !was_cast || !g.db.get(cd).creature {
        return 0;
    }
    let own = if &*g.db.get(cd).name == "Wildgrowth Archaic" { colors_spent(g, p, cd) } else { 0 };
    let more = i32::from(g.player(p).flag_turn.get(&archaic_key(cd)) == Some(&g.turn_stamp()));
    own + more + crate::impls::karn_engines::enters_with(g, cd, was_cast) // Hangarback Walker's X
}

// ================================================================== Royal Treatment
/// the Royal Role on m (`DataKey::Role` holding "royal"; a Monster Role is `Bool(true)`): ward {1}
fn role_ward(g: &Game, m: PermId) -> u32 {
    if g.perm(m).data.get(DataKey::Role) == Some(&Val::Str(intern("royal"))) { 1 } else { 0 }
}

/// Royal Treatment ({G}, instant): the creature gains hexproof until end of turn and gets a Royal Role (+1/+1 and
/// ward {1}; a Role you control on it is replaced). The AI casts it in response to removal aimed at a creature worth
/// saving (`protect`), and otherwise to protect a creature in response to nothing it can see: never.
pub fn royal_apply(g: &mut Game, m: PermId) {
    super::partials::eot_kw(g, m, "hexproof");
    g.perm_mut(m).data.set(DataKey::Role, Val::Str(intern("royal")));
}

// ================================================================== plot and foretell
/// A card plotted or foretold on turn t stays in the hand, marked in `impulse_long` with `u32::MAX - 1 - t` (the way
/// Irascible Wolverine's plot and Ultimate Magic: Meteor's foretell are kept); it is cast on a later turn by its own
/// option, and its normal cast priority is 0 meanwhile.
fn exiled_turn(g: &Game, p: PlayerId, c: CardId) -> Option<u32> {
    g.player(p)
        .impulse_long
        .iter()
        .find(|x| x.0 == c && x.1 < u32::MAX && x.1 > u32::MAX / 2)
        .map(|x| u32::MAX - 1 - x.1)
}

fn mark_exiled(g: &mut Game, p: PlayerId, c: CardId) {
    let pl = g.player_mut(p);
    let mark = u32::MAX - 1 - pl.turns;
    pl.impulse_long.push((c, mark));
}

fn clear_mark(g: &mut Game, p: PlayerId, c: CardId) {
    g.player_mut(p).impulse_long.retain(|x| x.0 != c);
}

/// the greatest power among p's creatures
fn greatest_power(g: &Game, p: PlayerId) -> i32 {
    creatures(g, p).into_iter().map(|m| epow(g, m)).max().unwrap_or(0).max(0)
}

fn flag_key(name: &str) -> Sym {
    intern(name)
}

fn set_flag(g: &mut Game, p: PlayerId, name: &str) {
    let st = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(flag_key(name), st);
}

fn flag_set(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).flag_turn.get(&flag_key(name)) == Some(&g.turn_stamp())
}

fn clear_flag(g: &mut Game, p: PlayerId, name: &str) {
    g.player_mut(p).flag_turn.shift_remove(&flag_key(name));
}

const FREESTRIDER: &str = "Freestrider Commando";
const TUMBLEWEED: &str = "Tumbleweed Rising";
const LIFESTREAM: &str = "Lifestream's Blessing";

/// Plot {3}{G} (Freestrider Commando) and {2}{G} (Tumbleweed Rising)
fn plot_cost(name: &str) -> Option<(u32, &'static str)> {
    match name {
        FREESTRIDER => Some((3, "G")),
        TUMBLEWEED => Some((2, "G")),
        _ => None,
    }
}

/// what a free cast of a plotted card is worth
fn plot_value(g: &Game, p: PlayerId, name: &str) -> f64 {
    match name {
        FREESTRIDER => 6.0,
        _ => {
            let x = greatest_power(g, p);
            if x >= 2 { 2.0 + 0.6 * x as f64 } else { 0.0 }
        }
    }
}

/// plot (main phase after combat, with the mana spare) and the free cast from the plot on a later turn
fn plot_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(post) = post else { return Ok(vec![]) };
    if g.active != Some(p) {
        return Ok(vec![]);
    }
    let name = g.db.get(c).name.to_string();
    let Some((gn, pips)) = plot_cost(&name) else { return Ok(vec![]) };
    let turns = g.player(p).turns;
    Ok(match exiled_turn(g, p, c) {
        Some(t) if turns > t => {
            let u = plot_value(g, p, &name);
            if u <= 0.0 {
                vec![]
            } else {
                vec![Opt {
                    utility: u,
                    label: format!("{name} (plotted, free)"),
                    act: Some(Action::Plan { f: plot_free, arg: c.0 as i64 }),
                }]
            }
        }
        Some(_) => vec![],
        None if post && can_pay(g, p, gn, pips, false) && (name == FREESTRIDER || greatest_power(g, p) >= 3) => {
            vec![Opt {
                utility: 1.3,
                label: format!("plot {name}"),
                act: Some(Action::Plan { f: plot_go, arg: c.0 as i64 }),
            }]
        }
        None => vec![],
    })
}

fn plot_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let name = g.db.get(c).name.to_string();
    let Some((gn, pips)) = plot_cost(&name) else { return Ok(false) };
    if !g.player(p).hand.contains(&c) || exiled_turn(g, p, c).is_some() || !can_pay(g, p, gn, pips, false) {
        return Ok(false);
    }
    if !crate::engine::mana::pay(g, p, gn, pips, false)? {
        return Ok(false);
    }
    mark_exiled(g, p, c);
    crate::glog!(g, "  {} plots {name}", pname(g, p));
    Ok(true)
}

fn plot_free(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || exiled_turn(g, p, c).is_none_or(|t| g.player(p).turns <= t) {
        return Ok(false);
    }
    clear_mark(g, p, c);
    crate::glog!(g, "  {} casts the plotted {} for free", pname(g, p), g.db.get(c).name);
    set_flag(g, p, "green_plot_free");
    let r = crate::engine::cast::cast_card(g, p, c, "hand", Ctx::default());
    clear_flag(g, p, "green_plot_free");
    r?;
    Ok(true)
}

/// Freestrider Commando enters with two +1/+1 counters if it wasn't cast or no mana was spent to cast it (a free
/// plotted cast)
fn freestrider_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || g.perm(m).owner != p {
        return Ok(());
    }
    if flag_set(g, p, "green_plot_free") || !g.last_cast_etb {
        crate::glog!(g, "    Freestrider Commando enters with two +1/+1 counters");
        repl::add_counters(g, m, 2);
    }
    Ok(())
}

/// Tumbleweed Rising: an X/X green Elemental, X the greatest power among your creatures
fn tumbleweed_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let x = greatest_power(g, p);
    if x > 0 {
        let spec = crate::engine::zones::Tokens {
            color: Some(crate::cards::Colors::from_letters("G")),
            types: vec!["elemental"],
            tgh: Some(x),
            ..crate::engine::zones::Tokens::new(1, x)
        };
        crate::engine::zones::make_tokens(g, p, spec)?;
    }
    Ok(intern("gy"))
}

// ------------------------------------------------------------------ Lifestream's Blessing (foretell)
/// Lifestream's Blessing: draw X cards, X the greatest power among your creatures; cast from exile, gain twice X life
fn lifestream_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let x = greatest_power(g, p);
    let from_exile = flag_set(g, p, "green_foretold");
    crate::glog!(g, "    Lifestream's Blessing: X = {x}");
    draw(g, p, x as u32, false)?;
    if from_exile {
        crate::engine::life::gain(g, p, 2 * x)?;
    }
    Ok(intern("gy"))
}

/// foretell for {2} (your main phase after combat, with creatures big enough), cast later for {4}{G}: at instant
/// speed, at the end of the turn before yours when the mana is free
fn lifestream_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = greatest_power(g, p);
    let turns = g.player(p).turns;
    let mine = g.active == Some(p);
    Ok(match exiled_turn(g, p, c) {
        Some(t) if (turns > t || !mine) && x >= 3 && can_pay(g, p, 4, "G", false) => {
            let u = 1.5 + 0.35 * x as f64 + if post.is_none() { 1.5 } else { 0.0 };
            vec![Opt {
                utility: u,
                label: "Lifestream's Blessing (foretold)".into(),
                act: Some(Action::Plan { f: lifestream_cast_foretold, arg: c.0 as i64 }),
            }]
        }
        Some(_) => vec![],
        None if mine && post == Some(true) && x >= 3 && can_pay(g, p, 2, "", false) => vec![Opt {
            utility: 1.4,
            label: "foretell Lifestream's Blessing".into(),
            act: Some(Action::Plan { f: lifestream_foretell, arg: c.0 as i64 }),
        }],
        None => vec![],
    })
}

fn lifestream_foretell(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || exiled_turn(g, p, c).is_some() || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    if !crate::engine::mana::pay(g, p, 2, "", false)? {
        return Ok(false);
    }
    mark_exiled(g, p, c);
    crate::glog!(g, "  {} foretells Lifestream's Blessing", pname(g, p));
    Ok(true)
}

fn lifestream_cast_foretold(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || exiled_turn(g, p, c).is_none() || !can_pay(g, p, 4, "G", false) {
        return Ok(false);
    }
    if !crate::engine::mana::pay(g, p, 4, "G", false)? {
        return Ok(false);
    }
    clear_mark(g, p, c);
    set_flag(g, p, "green_foretold");
    let r = crate::engine::cast::cast_card(g, p, c, "hand", Ctx::default());
    clear_flag(g, p, "green_foretold");
    r?;
    Ok(true)
}

// ================================================================== collect evidence (Sample Collector)
/// The cards a "collect evidence n" would exile from p's graveyard: one card with exactly enough mana value if there
/// is one, else the biggest first. Nothing in this deck wants its graveyard, so any cards will do; None: not enough.
pub fn evidence_set(g: &Game, p: PlayerId, n: u32) -> Option<Vec<CardId>> {
    let mut cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).cmc > 0).collect();
    if let Some(c) = cs.iter().copied().filter(|&c| g.db.get(c).cmc >= n).min_by_key(|&c| g.db.get(c).cmc) {
        return Some(vec![c]);
    }
    cs.sort_by_key(|&c| std::cmp::Reverse(g.db.get(c).cmc));
    let (mut total, mut out) = (0, vec![]);
    for c in cs {
        if total >= n {
            break;
        }
        total += g.db.get(c).cmc;
        out.push(c);
    }
    (total >= n).then_some(out)
}

/// collect evidence n: exile cards with total mana value n or more from your graveyard (the cost of Sample Collector's
/// reflexive trigger); false, and nothing exiled, when the graveyard is too small
pub fn collect_evidence(g: &mut Game, p: PlayerId, n: u32) -> bool {
    let Some(set) = evidence_set(g, p, n) else { return false };
    for c in set {
        let pl = g.player_mut(p);
        if let Some(i) = pl.gy.iter().position(|&x| x == c) {
            pl.gy.remove(i);
            pl.exile.push(c);
        }
    }
    true
}

/// Sample Collector: whenever it attacks, you may collect evidence 3; when you do, a +1/+1 counter on target
/// creature you control
fn collector_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if !atk.contains(&src) || g.perm(src).owner != p || evidence_set(g, p, 3).is_none() {
        return Ok(vec![]);
    }
    if trigger_window(g, p, Some(src), "collect evidence 3: a +1/+1 counter", None)? && collect_evidence(g, p, 3) {
        crate::glog!(g, "    {} collects evidence 3", pname(g, p));
        let mine = creatures(g, p);
        if let Some(t) = counter_target(g, &mine) {
            repl::add_counters(g, t, 1);
        }
    }
    Ok(vec![])
}

// ================================================================== web-slinging (Spider-Man, Brooklyn Visionary)
const SPIDER: &str = "Spider-Man, Brooklyn Visionary";

/// Web-slinging {2}{G}: cast Spider-Man for it if you also return a tapped creature you control to its owner's hand.
/// Offered when the full cost is out of reach; the creature returned is the cheapest to lose (a tapped mana creature
/// that can be cast again, never a token).
fn spider_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !can_pay(g, p, 2, "G", false) {
        return Ok(vec![]);
    }
    let (gn, pips) = crate::engine::mana::cost_of(g, p, c);
    if can_pay(g, p, gn, &pips, false) {
        return Ok(vec![]); // cast it for its mana cost
    }
    let Some((m, lose)) = web_victim(g, p) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: (3.5 - lose).max(0.6),
        label: format!("{SPIDER} (web-slinging, returning {})", g.perm(m).name),
        act: Some(Action::Plan { f: spider_go, arg: c.0 as i64 | (m.0 as i64) << 16 }),
    }])
}

/// the tapped creature to return for web-slinging, and what losing the tempo costs
fn web_victim(g: &Game, p: PlayerId) -> Option<(PermId, f64)> {
    let cands: Vec<PermId> = creatures(g, p)
        .into_iter()
        .filter(|&m| g.perm(m).tapped && !g.perm(m).token && g.perm(m).cd.is_some())
        .collect();
    let m = crate::engine::zones::min_by(&cands, |m| pval(g, m) + g.perm(m).plus.max(0) as f64 * 1.5)?;
    Some((m, 0.4 * pval(g, m) + 0.6 * g.perm(m).plus.max(0) as f64))
}

fn spider_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg & 0xffff) as u16);
    let m = PermId((arg >> 16) as u32);
    if !g.player(p).hand.contains(&c) || !live(g, m) || !g.perm(m).tapped || !can_pay(g, p, 2, "G", false) {
        return Ok(false);
    }
    if !crate::engine::mana::pay(g, p, 2, "G", false)? {
        return Ok(false);
    }
    crate::glog!(g, "  {} returns {} to hand (web-slinging)", pname(g, p), g.perm(m).name);
    crate::engine::zones::bounce(g, m)?;
    crate::engine::cast::cast_card(g, p, c, "hand", Ctx::default())?;
    Ok(true)
}

// ================================================================== disguise (Greenbelt Radical)
const RADICAL: &str = "Greenbelt Radical";

/// a face-down Greenbelt Radical (a 2/2 with ward {2}; `galadriel::enter_face_down`)
fn is_face_down_radical(g: &Game, m: PermId) -> bool {
    live(g, m) && g.perm(m).data.truthy(DataKey::Facedown) && card_name(g, m) == Some(RADICAL)
}

/// turn a face-down Radical face up for {5}{G}{G}: a +1/+1 counter on each creature you control, which gain trample
/// until end of turn
fn radical_up(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !is_face_down_radical(g, m) || !can_pay(g, p, 5, "GG", false) {
        return Ok(false);
    }
    if !crate::engine::mana::pay(g, p, 5, "GG", false)? {
        return Ok(false);
    }
    super::galadriel::turn_face_up(g, p, m);
    g.perm_mut(m).neutered = false;
    for x in creatures(g, p) {
        repl::add_counters(g, x, 1);
        super::partials::eot_kw(g, x, "trample");
    }
    Ok(true)
}

/// Greenbelt Radical's turn-face-up option for p's own turn (a deck's `special_options` offers it)
pub fn radical_options(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    let mut out = vec![];
    if g.active != Some(p) || !can_pay(g, p, 5, "GG", false) {
        return out;
    }
    let n = creatures(g, p).len() as f64;
    for m in g.player(p).perms.clone() {
        if is_face_down_radical(g, m) {
            let u = 1.2 + 0.7 * n + if post { 0.0 } else { 1.0 };
            out.push(Opt {
                utility: u,
                label: "Greenbelt Radical (turn face up)".into(),
                act: Some(Action::Plan { f: radical_up, arg: m.0 as i64 }),
            });
        }
    }
    out
}

/// extra ward on a permanent: a Royal Role's {1}, a face-down Radical's {2} (`cardcode::aura_ward`)
pub fn extra_ward(g: &Game, m: PermId) -> u32 {
    let x = g.perm(m);
    let more = super::bulk_white::extra_ward(g, m); // face-down disguise creatures, Bronze Guardian's artifacts
    if x.data.is_empty() {
        return more;
    }
    more + role_ward(g, m) // a face-down disguise creature (Greenbelt Radical too) is in `more`
}

/// Disguise {5}{G}{G}: cast face down for {3} as a 2/2 with ward {2}. The AI does that with the mana to spare; the
/// normal cast (a vanilla 4/4) is never worth it.
fn radical_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !can_pay(g, p, 3, "", false) || !castable(g, p, c) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.2,
        label: "Greenbelt Radical (face down)".into(),
        act: Some(Action::Plan { f: radical_down, arg: c.0 as i64 }),
    }])
}

fn castable(g: &Game, p: PlayerId, c: CardId) -> bool {
    crate::engine::cast::castable(g, p, c, "hand")
}

fn radical_down(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 3, "", false) || !castable(g, p, c) {
        return Ok(false);
    }
    if !crate::engine::mana::pay(g, p, 3, "", false)? {
        return Ok(false);
    }
    let pl = g.player_mut(p);
    if let Some(i) = pl.hand.iter().position(|&x| x == c) {
        pl.hand.remove(i);
    }
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts a face-down creature (disguise)", pname(g, p));
    crate::engine::cast::on_cast(g, p, c)?;
    if !counter_window(g, p, c, 3.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    super::galadriel::enter_face_down(g, p, c)?;
    Ok(true)
}

// ================================================================== Studious First-Year (prepared)
/// Studious First-Year enters prepared: while it is, you may cast a copy of its spell (Rampant Growth, {1}{G}),
/// which unprepares it
fn firstyear_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.perm_mut(m).data.set(DataKey::Prepared, Val::Bool(true));
    }
    Ok(())
}

fn firstyear_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if post.is_none()
        || x.owner != p
        || x.phased
        || !x.data.truthy(DataKey::Prepared)
        || g.active != Some(p)
        || !can_pay(g, p, 1, "G", false)
        || g.player(p).library.is_empty()
    {
        return Ok(vec![]);
    }
    let u = if bill_of(g, p).is_some() { 2.9 } else { 1.9 };
    Ok(vec![Opt {
        utility: u,
        label: "Rampant Growth (Studious First-Year, prepared copy)".into(),
        act: Some(Action::Ability { src, f: firstyear_go, arg: 0 }),
    }])
}

fn firstyear_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !g.perm(src).data.truthy(DataKey::Prepared) || !can_pay(g, p, 1, "G", false) {
        return Ok(false);
    }
    if !crate::engine::mana::pay(g, p, 1, "G", false)? {
        return Ok(false);
    }
    g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(false));
    super::mine::cast_copy_with(g, p, None, None, false, 2.0, &mut |g| crate::engine::zones::land_ramp(g, p, 1, true))?;
    Ok(true)
}

// ================================================================== Additive Evolution and Wild Hypothesis (Fractals)
/// a 0/0 green and blue Fractal token with n +1/+1 counters (made 1/1 first so that it is not put into the graveyard
/// before the counters arrive); Hardened Scales adds one
fn make_fractal(g: &mut Game, p: PlayerId, n: i32) -> Res {
    let spec = crate::engine::zones::Tokens {
        color: Some(crate::cards::Colors::from_letters("GU")),
        types: vec!["fractal"],
        tgh: Some(1),
        ..crate::engine::zones::Tokens::new(1, 1)
    };
    use crate::engine::repl::counters;
    for t in crate::engine::zones::make_tokens(g, p, spec)? {
        let k = counters(g, p, On::Perm(t), Ctr::Plus, n, Path::SCALES); // Hardened Scales
        let x = g.perm_mut(t);
        x.pow = 0;
        x.tgh = 0;
        x.plus = k;
    }
    Ok(())
}

const EVOLUTION: &str = "Additive Evolution";

/// Additive Evolution: when it enters, a 0/0 Fractal with three +1/+1 counters
fn evolution_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && g.perm(m).owner == p {
        make_fractal(g, p, 3)?;
    }
    Ok(())
}

/// Additive Evolution: at the beginning of combat on your turn, a +1/+1 counter on target creature you control, which
/// gains vigilance until end of turn (the `crew` event is the start of the first combat)
fn evolution_combat(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner != p || g.active != Some(p) || !live(g, src) {
        return Ok(());
    }
    let mine = creatures(g, p);
    let Some(t) = counter_target(g, &mine) else { return Ok(()) };
    if trigger_window(g, p, Some(src), &format!("a +1/+1 counter on {}", g.perm(t).name), None)? && live(g, t) {
        repl::add_counters(g, t, 1);
        super::partials::eot_kw(g, t, "vigilance");
    }
    Ok(())
}

const HYPOTHESIS: &str = "Wild Hypothesis";

/// Wild Hypothesis ({X}{G}): a 0/0 Fractal with X +1/+1 counters, then surveil 2
fn hypothesis_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    make_fractal(g, p, ctx.x.max(0))?;
    crate::cardcode::scry(g, p, 2, true)?;
    Ok(intern("gy"))
}

/// cast with all the spare mana but {G}: X is what is left (two or more), in either main phase
fn hypothesis_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !castable(g, p, c) {
        return Ok(vec![]);
    }
    let x = hypothesis_x(g, p);
    if x < 2 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: (0.6 + 0.35 * x as f64).min(4.0) + if post == Some(true) { 0.3 } else { 0.0 },
        label: format!("Wild Hypothesis (X = {x})"),
        act: Some(Action::Plan { f: hypothesis_go, arg: c.0 as i64 }),
    }])
}

fn hypothesis_x(g: &Game, p: PlayerId) -> i32 {
    let mana = crate::engine::mana::total_mana(g, p, false) as i32;
    (mana - 1).max(0)
}

fn hypothesis_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let x = hypothesis_x(g, p);
    if !g.player(p).hand.contains(&c) || x < 1 || !can_pay(g, p, x as u32, "G", false) {
        return Ok(false);
    }
    if !crate::engine::mana::pay(g, p, x as u32, "G", false)? {
        return Ok(false);
    }
    crate::engine::cast::cast_card(g, p, c, "hand", Ctx { x, ..Ctx::default() })?;
    Ok(true)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, BILL)?.options = Some(bill_options);
    r.card(db, "Boughside Wanderers")?.etb = Some(wanderers_etb);
    r.card(db, "Ghalta the Unstoppable")?.self_cost = Some(ghalta_cost);
    let c = r.card(db, "Venomous Brutalizer")?;
    c.etb = Some(brutalizer_etb);
    r.card(db, "Carnivorous Canopy")?.resolve = Some(canopy_resolve);
    r.card(db, "Crushing Canopy")?.resolve = Some(crushing_resolve);
    let c = r.card(db, "Wildgrowth Archaic")?;
    c.cast = Some(archaic_cast);
    let c = r.card(db, FREESTRIDER)?;
    c.hand_options = Some(plot_options);
    c.etb = Some(freestrider_etb);
    let c = r.card(db, TUMBLEWEED)?;
    c.hand_options = Some(plot_options);
    c.resolve = Some(tumbleweed_resolve);
    let c = r.card(db, LIFESTREAM)?;
    c.resolve = Some(lifestream_resolve);
    c.hand_options = Some(lifestream_options);
    r.card(db, "Sample Collector")?.attack = Some(collector_attack);
    r.card(db, SPIDER)?.hand_options = Some(spider_options);
    r.card(db, RADICAL)?.hand_options = Some(radical_hand);
    let c = r.card(db, EVOLUTION)?;
    c.etb = Some(evolution_etb);
    c.crew = Some(evolution_combat);
    let c = r.card(db, HYPOTHESIS)?;
    c.resolve = Some(hypothesis_resolve);
    c.hand_options = Some(hypothesis_options);
    let c = r.card(db, "Studious First-Year")?;
    c.etb = Some(firstyear_etb);
    c.options = Some(firstyear_options);
    let c = r.card(db, "Topiary Lecturer")?;
    c.cast = Some(increment);
    c.dyn_mana_perm = Some(lecturer_mana);
    r.card(db, "Vinelasher Adept")?.hand_options = Some(adept_hand);
    r.card(db, KURA)?.self_dies = Some(kura_dies);
    Ok(())
}

// ================================================================== Topiary Lecturer and Vinelasher Adept
/// +1/+1 counters on m, after doubling effects
fn put_counters(g: &mut Game, p: PlayerId, m: PermId, n: i32) {
    let k = repl::counters(g, p, On::Perm(m), Ctr::Plus, n, Path::MULT);
    repl::add_counters(g, m, k);
}

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

/// Topiary Lecturer: {T}: add an amount of {G} equal to its power
fn lecturer_mana(g: &Game, _p: PlayerId, m: PermId) -> u32 {
    epow(g, m).max(0) as u32
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

/// did p gain life this turn (infusion; Follow the Lumarets)
pub fn gained_life(g: &Game, p: PlayerId) -> bool {
    let st = g.turn_stamp();
    g.player(p).gained_turn.is_some_and(|x| x.0 == st && x.1 > 0)
}

// ================================================================== Kura, the Boundless Sky
const KURA: &str = "Kura, the Boundless Sky";

/// Kura, the Boundless Sky: flying, deathtouch. When it dies, choose one: up to three land cards from the library to
/// the hand (the Titania deck turns each one into a landfall), or an X/X green Spirit, X = lands you control. The AI
/// takes the lands while it still has land drops to make (fewer than two lands in hand and under seven on the
/// battlefield) and the library has lands; the Spirit once the lands are down.
fn kura_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let p = g.perm(m).owner;
    if !g.player(p).alive {
        return Ok(());
    }
    let lib_lands = g.player(p).library.iter().filter(|&&c| g.db.get(c).land).count();
    let in_hand = g.player(p).hand.iter().filter(|&&c| g.db.get(c).land).count();
    let on_bf = g.player(p).lands.len();
    let lands_mode = lib_lands > 0 && in_hand < 2 && on_bf < 7;
    let label = if lands_mode { "search for up to three lands" } else { "an X/X Spirit (X = lands)" };
    if !trigger_window(g, p, Some(m), label, None)? {
        return Ok(());
    }
    if lands_mode {
        // nonbasic lands first (they are the deck's best), then basics
        let mut found = 0;
        for basic in [false, true] {
            while found < 3 {
                let pick = g.player(p).library.iter().position(|&c| {
                    let d = g.db.get(c);
                    d.land && is_basic_name(&d.name) == basic
                });
                let Some(i) = pick else { break };
                let c = g.player_mut(p).library.remove(i);
                g.player_mut(p).hand.push(c);
                found += 1;
            }
        }
        crate::engine::tutors::shuffle_library(g, p);
    } else {
        let x = g.player(p).lands.len() as i32;
        let mut spec = crate::engine::zones::Tokens::new(1, x);
        spec.tgh = Some(x);
        spec.color = Some(crate::cards::Colors::from_letters("G"));
        spec.types = vec![intern("spirit")];
        crate::engine::zones::make_tokens(g, p, spec)?;
    }
    Ok(())
}

fn is_basic_name(n: &str) -> bool {
    matches!(n, "Forest" | "Island" | "Mountain" | "Plains" | "Swamp")
}
