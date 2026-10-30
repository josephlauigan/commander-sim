//! Tier 1 pool decks `tatyova-simic-landfall` and `teysa-orzhov-aristocrats` (EDHREC's bracket-2 average lists): the
//! cards the ability language doesn't cover (rust/agents/t12-tatyova-teysa.md). Summer Bloom, Urban Evolution,
//! Roiling Regrowth, Beanstalk Giant's size, Mossborn Hydra's landfall, Bonny Pall's attack trigger, Hallowed
//! Spiritkeeper and afterlife (Ministrant of Obligation, Orzhov Enforcer, Indebted Spirit) are compiled (dsl_parse.rs,
//! `// --- Tiers 1/2 (tatyova-teysa)`); the rest is here.
//!
//! Tatyova:
//! - Beanstalk Giant // Fertile Footsteps: the adventure ({2}{G}, sorcery): a basic land onto the battlefield; the card
//!   then stays castable as the Giant. Left out: it waits in the hand, not in exile (as Brazen Borrower here).
//! - Bonny Pall, Clearcutter: enters: Beau, a legendary blue Ox token whose power and toughness are your lands.
//! - Entish Restoration: sacrifice a land, then two basic lands tapped (three with a creature of power 4 or more).
//! - Loot, Exuberant Explorer: an additional land each of your turns; {4}{G}{G}, {T}: the top six, a creature card with
//!   mana value up to your lands onto the battlefield, the rest on the bottom in a random order.
//! - Mossborn Hydra: enters with a +1/+1 counter (its landfall doubling is compiled).
//! - Wrenn and Seven: +1 the top four, lands to hand and the rest to the graveyard; 0 the lands in hand onto the
//!   battlefield tapped; -3 a reach Treefolk whose power and toughness are your lands; -8 permanent cards from the
//!   graveyard to hand and the no-maximum-hand-size emblem.
//!
//! Teysa:
//! - Black Market: a charge counter whenever a creature dies; {B} per counter at the beginning of your first main
//!   phase (it empties as main 1 ends).
//! - Drivnod, Carnage Dominus: your death-caused triggers trigger once more (as Teysa Karlov); {B/P}{B/P}, exile three
//!   creature cards from your graveyard: an indestructible counter.
//! - Indebted Spirit: bestow {2}{W} on your best creature (+1/+1 and afterlife 1; it becomes a creature when the host
//!   leaves), through the bestow of partials.rs (Eidolon of Countless Battles).
//! - Luminous Broodmoth: your creature without flying dies: it returns with a flying counter.
//! - Teysa, Orzhov Scion: sacrifice three white creatures: exile target creature; another black creature of yours
//!   dies: a 1/1 white Spirit with flying.

use crate::engine::repl;
use crate::engine::pool::Kind;
use crate::cards::{CardDb, Colors};
use crate::engine::cast::on_cast;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, counter_window, trigger_window};
use crate::engine::tutors::shuffle_library;
use crate::engine::values::{colors_of, epow, pval, untargetable};
use crate::engine::zones::{Enter, Tokens, die, enter, land_ramp, landfall, make_tokens, max_by, sac_worth};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{WalkerAb, walker};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};

const GIANT: &str = "Beanstalk Giant // Fertile Footsteps";
const BONNY: &str = "Bonny Pall, Clearcutter";
const ENTISH: &str = "Entish Restoration";
const LOOT: &str = "Loot, Exuberant Explorer";
const HYDRA: &str = "Mossborn Hydra";
const WRENN: &str = "Wrenn and Seven";
const MARKET: &str = "Black Market";
const DRIVNOD: &str = "Drivnod, Carnage Dominus";
pub const INDEBTED: &str = "Indebted Spirit";
const BROODMOTH: &str = "Luminous Broodmoth";
const SCION: &str = "Teysa, Orzhov Scion";
/// the keyword mark on Beau and Wrenn's Treefolk: "power and toughness are each equal to the number of lands you
/// control" (`lands_pt`)
const LANDS_PT: &str = "t12_lands_pt";
const WRENN_EMBLEM: &str = "wrenn_seven";

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn remove_card(v: &mut Vec<CardId>, c: CardId) -> bool {
    match v.iter().position(|&x| x == c) {
        Some(i) => {
            v.remove(i);
            true
        }
        None => false,
    }
}

/// a creature has flying as it is now (or as it last existed): printed, granted, until end of turn, a counter
fn flies(g: &Game, m: PermId) -> bool {
    g.perm(m).fly || crate::dsl::has_kw(g, m, "flying")
}

/// p's lands in hand
fn lands_in_hand(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).land).collect()
}

/// a token whose power and toughness are its controller's lands (Beau, the Treefolk), with these keywords
fn lands_token(g: &mut Game, p: PlayerId, name: &str, color: &str, types: Vec<Sym>, kws: &[&str]) -> Res<Vec<PermId>> {
    let mut spec = Tokens { color: Some(Colors::from_letters(color)), types, ..Tokens::new(1, 0) };
    spec.tgh = Some(0);
    let mut marks = vec![Val::Str(intern(LANDS_PT))];
    marks.extend(kws.iter().map(|k| Val::Str(intern(k))));
    spec.data.set(DataKey::Kws, Val::List(marks));
    g.selfpt = true; // token_pt is read while selfpt is on
    let made = make_tokens(g, p, spec)?;
    for &t in &made {
        g.perm_mut(t).name = intern(name);
    }
    Ok(made)
}

fn has_mark(g: &Game, m: PermId, mark: &str) -> bool {
    let Some(Val::List(v)) = g.perm(m).data.get(DataKey::Kws) else { return false };
    v.iter().any(|k| matches!(k, Val::Str(s) if *s == mark))
}

/// token_pt: Beau's and the Treefolk's power and toughness, the lands their controller controls
fn lands_pt(g: &Game, m: PermId) -> (i32, i32) {
    if !has_mark(g, m, LANDS_PT) {
        return (0, 0);
    }
    let n = g.player(owner(g, m)).lands.len() as i32;
    (n, n)
}

// ================================================================== Beanstalk Giant // Fertile Footsteps
/// the adventure, before combat while the Giant itself ({6}{G}) is out of reach and a basic is left to find
fn footsteps_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let pl = g.player(p);
    if post != Some(false) || g.active != Some(p) || !pl.hand.contains(&c) || pl.adv_done.contains(&c) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 2, "G", false) || can_pay(g, p, 6, "G", false) {
        return Ok(vec![]);
    }
    let basic = pl.library.iter().any(|&x| g.db.get(x).basic && g.db.get(x).land);
    if !basic {
        return Ok(vec![]);
    }
    let lands = pl.lands.len() as f64;
    Ok(vec![Opt {
        utility: 2.2 + if lands <= 5.0 { 0.5 } else { 0.0 },
        label: "Fertile Footsteps (adventure)".into(),
        act: Some(Action::Plan { f: footsteps_go, arg: c.0 as i64 }),
    }])
}

/// Search your library for a basic land card, put it onto the battlefield, then shuffle. The card goes on an
/// adventure (here: back to the hand, castable as the Giant); countered, it goes to the graveyard.
fn footsteps_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "G", false) || !pay(g, p, 2, "G", false)? {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    crate::glog!(g, "  {} casts Fertile Footsteps (the adventure of Beanstalk Giant)", pname(g, p));
    on_cast(g, p, c)?;
    if !counter_window(g, p, c, 1.5, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    land_ramp(g, p, 1, false)?;
    shuffle_library(g, p);
    let pl = g.player_mut(p);
    pl.hand.push(c);
    pl.adv_done.push(c);
    Ok(true)
}

// ================================================================== Bonny Pall, Clearcutter
/// When Bonny Pall enters, create Beau, a legendary blue Ox creature token with "Beau's power and toughness are each
/// equal to the number of lands you control." (A second Beau goes to the legend rule: one is kept.)
fn bonny_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "create Beau", None)? {
        return Ok(());
    }
    let beau = g.player(o).perms.iter().any(|&x| live(g, x) && g.perm(x).token && &*g.perm(x).name == "Beau");
    if beau {
        crate::glog!(g, "    {} already has Beau (the legend rule keeps one)", pname(g, o));
        return Ok(());
    }
    lands_token(g, o, "Beau", "U", vec!["ox"], &[])?;
    crate::glog!(g, "    {} creates Beau ({} lands)", pname(g, o), g.player(o).lands.len());
    Ok(())
}

// ================================================================== Entish Restoration
/// Sacrifice a land. Search your library for up to two basic land cards, put them onto the battlefield tapped, then
/// shuffle; three instead if you control a creature with power 4 or greater (checked as it resolves).
fn entish_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    crate::impls::t3_korvold::sac_worst_land(g, p)?;
    let big = g.player(p).perms.iter().any(|&m| live(g, m) && g.is_creature(m) && epow(g, m) >= 4);
    land_ramp(g, p, if big { 3 } else { 2 }, true)?;
    shuffle_library(g, p);
    Ok("gy")
}

// ================================================================== Loot, Exuberant Explorer
/// You may play an additional land on each of your turns.
fn loot_extra(g: &Game, src: Src, p: PlayerId) -> i32 {
    (owner(g, src) == p) as i32
}

/// the creature cards Loot may put onto the battlefield: mana value up to the lands p controls
fn loot_fits(g: &Game, p: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    d.creature && d.cmc as usize <= g.player(p).lands.len()
}

/// {4}{G}{G}, {T}: used with the mana to spare (a main phase with nothing better, or the end-of-turn window)
fn loot_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if owner(g, src) != p || !live(g, src) || x.tapped || x.sick || !can_pay(g, p, 4, "GG", false) {
        return Ok(vec![]);
    }
    if g.player(p).library.len() < 10 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.8 + 0.1 * g.player(p).lands.len().min(10) as f64,
        label: "Loot: look at the top six".into(),
        act: Some(Action::Ability { src, f: loot_go, arg: 0 }),
    }])
}

/// Look at the top six cards of your library. You may reveal a creature card with mana value less than or equal to
/// the number of lands you control from among them and put it onto the battlefield. Put the rest on the bottom in a
/// random order.
fn loot_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || g.perm(src).tapped || !can_pay(g, p, 4, "GG", false) || !pay(g, p, 4, "GG", false)? {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), "the top six cards", None, None)? {
        return Ok(true);
    }
    let k = g.player(p).library.len().min(6);
    let pl = g.player_mut(p);
    let mut top: Vec<CardId> = (0..k).map(|_| pl.library.pop().unwrap()).collect();
    let fits: Vec<CardId> = top.iter().copied().filter(|&c| loot_fits(g, p, c)).collect();
    let pick = max_by(&fits, |c| g.db.get(c).cmc as f64 + 0.3 * g.db.get(c).pow as f64);
    if let Some(c) = pick {
        remove_card(&mut top, c);
    }
    g.rng.shuffle(&mut top);
    let lib = &mut g.player_mut(p).library;
    for c in top {
        lib.insert(0, c); // the bottom
    }
    match pick {
        Some(c) => {
            crate::glog!(g, "    {} puts {} onto the battlefield", pname(g, p), cname(g, c));
            enter(g, p, c, Enter::default())?;
        }
        None => crate::glog!(g, "    no creature to put onto the battlefield"),
    }
    Ok(true)
}

// ================================================================== Mossborn Hydra
/// This creature enters with a +1/+1 counter on it (Hardened Scales adds its own).
fn hydra_as(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    repl::put_counters(g, p, m, 1); // the counters step (Hardened Scales adds its own)
    Ok(())
}

// ================================================================== Wrenn and Seven
/// the expected lands among the top four (the AI doesn't look at its library)
fn wrenn_plus_v(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let lib = &g.player(p).library;
    if lib.len() < 8 {
        return None;
    }
    let share = lib.iter().filter(|&&c| g.db.get(c).land).count() as f64 / lib.len() as f64;
    Some(0.6 + 4.0 * share * 0.9)
}

/// +1: Reveal the top four cards of your library. Put all land cards revealed this way into your hand and the rest
/// into your graveyard.
fn wrenn_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let k = g.player(p).library.len().min(4);
    let pl = g.player_mut(p);
    let top: Vec<CardId> = (0..k).map(|_| pl.library.pop().unwrap()).collect();
    let (lands, rest): (Vec<CardId>, Vec<CardId>) = top.into_iter().partition(|&c| g.db.get(c).land);
    crate::glog!(g, "    {} reveals four: {} lands to hand", pname(g, p), lands.len());
    let pl = g.player_mut(p);
    for &c in &lands {
        pl.hand.push(c);
        pl.seen.insert(c);
    }
    pl.gy.extend(rest);
    Ok(())
}

/// 0: worth it with two or more lands in hand (each is a land drop and a landfall)
fn wrenn_zero_v(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let n = lands_in_hand(g, p).len();
    (n >= 2).then(|| 0.4 + 1.1 * n as f64)
}

/// 0: Put any number of land cards from your hand onto the battlefield tapped.
fn wrenn_zero(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let ls = lands_in_hand(g, p);
    crate::glog!(g, "    {} puts {} lands onto the battlefield tapped", pname(g, p), ls.len());
    for c in ls {
        if !remove_card(&mut g.player_mut(p).hand, c) {
            continue;
        }
        g.add_land_entering(p, c, true);
        landfall(g, p)?;
        if g.over {
            break;
        }
    }
    Ok(())
}

/// -3: a body as big as the lands
fn wrenn_tree_v(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let n = g.player(p).lands.len() as f64;
    (n >= 5.0).then(|| 0.5 + 0.45 * n)
}

/// −3: Create a green Treefolk creature token with reach and "This token's power and toughness are each equal to the
/// number of lands you control."
fn wrenn_tree(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    lands_token(g, p, "Treefolk", "G", vec!["treefolk"], &["reach"])?;
    crate::glog!(
        g,
        "    {} creates a {}/{} Treefolk with reach",
        pname(g, p),
        g.player(p).lands.len(),
        g.player(p).lands.len()
    );
    Ok(())
}

fn permanent_card(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.perm || d.land
}

fn wrenn_ult_v(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let n = g.player(p).gy.iter().filter(|&&c| permanent_card(g, c)).count() as f64;
    (n >= 4.0).then(|| 2.0 + 0.8 * n)
}

/// −8: Return all permanent cards from your graveyard to your hand. You get an emblem with "You have no maximum hand
/// size."
fn wrenn_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let back: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| permanent_card(g, c)).collect();
    let pl = g.player_mut(p);
    pl.gy.retain(|c| !back.contains(c));
    pl.hand.extend(back.iter().copied());
    pl.t0s.nomax = true;
    pl.emblems.push(WRENN_EMBLEM);
    crate::glog!(g, "    {} returns {} permanent cards to hand; no maximum hand size", pname(g, p), back.len());
    Ok(())
}

static WRENN_ABS: [WalkerAb; 4] = [
    WalkerAb { delta: 1, label: "the top four: lands to hand", val: wrenn_plus_v, eff: wrenn_plus },
    WalkerAb { delta: 0, label: "lands from hand onto the battlefield", val: wrenn_zero_v, eff: wrenn_zero },
    WalkerAb { delta: -3, label: "a Treefolk", val: wrenn_tree_v, eff: wrenn_tree },
    WalkerAb { delta: -8, label: "permanent cards back, the emblem", val: wrenn_ult_v, eff: wrenn_ult },
];

// ================================================================== Black Market
fn charges(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.get(DataKey::Charge).map_or(0, Val::int)
}

/// Whenever a creature dies, put a charge counter on this enchantment.
fn market_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m == src || !live(g, src) || !g.is_creature(m) {
        return Ok(());
    }
    let n = charges(g, src) + 1;
    g.perm_mut(src).data.set(DataKey::Charge, Val::Int(n));
    Ok(())
}

/// At the beginning of your first main phase, add {B} for each charge counter on this enchantment.
fn market_main(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let n = charges(g, src);
    if p != owner(g, src) || !live(g, src) || n <= 0 {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), &format!("add {{B}} x{n}"), None)? {
        g.player_mut(p).mana.add(Kind::B, n as u32);
        crate::glog!(g, "    {} adds {} black mana (Black Market)", pname(g, p), n);
    }
    Ok(())
}

// ================================================================== Drivnod, Carnage Dominus
/// If a creature dying causes a triggered ability of a permanent you control to trigger, that ability triggers an
/// additional time.
fn drivnod_copies(g: &Game, src: Src, p: PlayerId, kind: Sym, m: Option<PermId>) -> i32 {
    (kind == "dies" && owner(g, src) == p && live(g, src) && m.is_none_or(|m| g.is_creature(m))) as i32
}

fn gy_creatures(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect()
}

/// {B/P}{B/P}: {B}{B}, {B} and 2 life, or 4 life
fn phyrexian_bb(g: &Game, p: PlayerId) -> Option<(&'static str, i32)> {
    let life = g.player(p).life;
    if can_pay(g, p, 0, "BB", false) {
        Some(("BB", 0))
    } else if can_pay(g, p, 0, "B", false) && life > 12 {
        Some(("B", 2))
    } else if life > 16 {
        Some(("", 4))
    } else {
        None
    }
}

fn drivnod_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || post.is_none() || !live(g, src) || g.perm(src).data.truthy(DataKey::Indestr) {
        return Ok(vec![]);
    }
    if gy_creatures(g, p).len() < 3 || phyrexian_bb(g, p).is_none() {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.2 + 0.25 * pval(g, src),
        label: "Drivnod: an indestructible counter".into(),
        act: Some(Action::Ability { src, f: drivnod_go, arg: 0 }),
    }])
}

/// {B/P}{B/P}, Exile three creature cards from your graveyard: Put an indestructible counter on Drivnod. (The least
/// valuable three go.)
fn drivnod_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some((pips, life)) = phyrexian_bb(g, p) else { return Ok(false) };
    let mut cs = gy_creatures(g, p);
    if cs.len() < 3 || !live(g, src) {
        return Ok(false);
    }
    if !pips.is_empty() && !pay(g, p, 0, pips, false)? {
        return Ok(false);
    }
    if life > 0 {
        lose_life(g, p, life, Some(p), "other", None)?;
    }
    cs.sort_by(|&a, &b| {
        let w = |c: CardId| crate::engine::tutors::card_worth(g, p, c, true);
        w(a).total_cmp(&w(b))
    });
    for &c in cs.iter().take(3) {
        let pl = g.player_mut(p);
        remove_card(&mut pl.gy, c);
        pl.exile.push(c);
    }
    if ability_window(g, p, Some(src), "an indestructible counter", None, None)? && live(g, src) {
        g.perm_mut(src).data.set(DataKey::Indestr, Val::Bool(true));
        crate::glog!(g, "    {} puts an indestructible counter on Drivnod", pname(g, p));
    }
    Ok(true)
}

// ================================================================== Indebted Spirit
/// bestow {2}{W} before combat on p's best creature (the commander first), when it is worth more than a 1/1 body
fn indebted_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "W", false) {
        return Ok(vec![]);
    }
    let Some(host) = crate::impls::common::own_host(g, p, &crate::impls::common::AURA, None) else { return Ok(vec![]) };
    if pval(g, host) < 3.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.0 + 0.15 * pval(g, host).min(8.0),
        label: format!("bestow Indebted Spirit on {}", g.perm(host).name),
        act: Some(Action::Plan { f: indebted_go, arg: (c.0 as i64) | ((host.0 as i64) << 16) }),
    }])
}

fn indebted_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, host) = (CardId((arg & 0xffff) as u16), PermId((arg >> 16) as u32));
    if !g.player(p).hand.contains(&c) || !live(g, host) || owner(g, host) != p || !can_pay(g, p, 2, "W", false) {
        return Ok(false);
    }
    if !pay(g, p, 2, "W", false)? {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    crate::glog!(g, "  {} bestows Indebted Spirit on {}", pname(g, p), g.perm(host).name);
    on_cast(g, p, c)?;
    if !counter_window(g, p, c, 2.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    if !live(g, host) {
        enter(g, p, c, Enter::default())?; // its target is gone: it resolves as a creature
        return Ok(true);
    }
    let data = &mut g.perm_mut(host).data;
    match data.get_mut(DataKey::Bestow) {
        Some(Val::List(v)) => v.push(Val::Card(c)),
        _ => data.set(DataKey::Bestow, Val::List(vec![Val::Card(c)])),
    }
    g.selfpt = true;
    Ok(true)
}

/// partials.bestow_fall: the host of a bestowed Indebted Spirit died (afterlife 1 it had): a 1/1 white and black
/// Spirit with flying
pub fn indebted_afterlife(g: &mut Game, p: PlayerId) -> Res {
    let spec =
        Tokens { fly: true, color: Some(Colors::from_letters("WB")), types: vec!["spirit"], ..Tokens::new(1, 1) };
    make_tokens(g, p, spec)?;
    crate::glog!(g, "    afterlife 1 (Indebted Spirit): a 1/1 flying Spirit");
    Ok(())
}

// ================================================================== Luminous Broodmoth
/// Whenever a creature you control without flying dies, return it to the battlefield under its owner's control with
/// a flying counter on it.
fn broodmoth_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, src);
    let x = g.perm(m);
    if m == src || x.owner != o || x.token || !g.is_creature(m) || flies(g, m) {
        return Ok(());
    }
    let (Some(cd), back_to) = (x.phys.or(x.cd), x.orig) else { return Ok(()) };
    if !g.player(back_to).alive || !g.player(back_to).gy.contains(&cd) || g.player(back_to).is_commander(cd) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), &format!("return {}", cname(g, cd)), None)? {
        return Ok(());
    }
    if !g.player(back_to).gy.contains(&cd) || !crate::engine::zones::returns_left(g, cd) {
        return Ok(());
    }
    remove_card(&mut g.player_mut(back_to).gy, cd);
    let n = enter(g, back_to, cd, Enter::default())?;
    g.perm_mut(n).fly = true; // the flying counter
    crate::glog!(g, "    {} returns with a flying counter (Luminous Broodmoth)", cname(g, cd));
    Ok(())
}

// ================================================================== Teysa, Orzhov Scion
/// Whenever another black creature you control dies, create a 1/1 white Spirit creature token with flying.
fn scion_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, src);
    if m == src || g.perm(m).owner != o || !g.is_creature(m) || !colors_of(g, m).has('B') {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "create a 1/1 white Spirit with flying", None)? {
        let spec =
            Tokens { fly: true, color: Some(Colors::from_letters("W")), types: vec!["spirit"], ..Tokens::new(1, 1) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

/// the three white creatures Teysa would sacrifice: the least valuable (not Teysa herself)
fn scion_fodder(g: &Game, p: PlayerId, src: PermId) -> Vec<PermId> {
    let mut w: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != src && live(g, m) && g.is_creature(m) && colors_of(g, m).has('W') && !g.perm(m).is_cmd)
        .collect();
    w.sort_by(|&a, &b| sac_worth(g, a).total_cmp(&sac_worth(g, b)));
    w.truncate(3);
    w
}

fn scion_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let ts: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m) && !untargetable(g, m))
        .collect();
    max_by(&ts, |m| pval(g, m))
}

/// Sacrifice three white creatures: Exile target creature. When the best opposing creature is worth more than the
/// three bodies (less what their deaths bring: drains, cards).
fn scion_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || post.is_none() || !live(g, src) {
        return Ok(vec![]);
    }
    let fod = scion_fodder(g, p, src);
    if fod.len() < 3 {
        return Ok(vec![]);
    }
    let Some(t) = scion_target(g, p) else { return Ok(vec![]) };
    let cost: f64 = fod.iter().map(|&m| sac_worth(g, m) - crate::impls::common::death_value(g, p, Some(m))).sum();
    let u = pval(g, t) - cost.max(0.0) - 0.5;
    if u <= 0.5 || pval(g, t) < 3.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: u,
        label: format!("Teysa, Orzhov Scion: exile {}", g.perm(t).name),
        act: Some(Action::Ability { src, f: scion_go, arg: t.0 as i64 }),
    }])
}

fn scion_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    let fod = scion_fodder(g, p, src);
    if fod.len() < 3 || !live(g, t) || untargetable(g, t) {
        return Ok(false);
    }
    crate::glog!(g, "  {} sacrifices three white creatures to Teysa, Orzhov Scion", pname(g, p));
    for m in fod {
        if live(g, m) {
            die(g, m, "sac")?;
        }
    }
    if g.over {
        return Ok(true);
    }
    if ability_window(g, p, Some(src), "exile target creature", None, Some(t))? && live(g, t) && !untargetable(g, t) {
        apply_removal(g, Some(p), t, "exile", None)?;
    }
    Ok(true)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.token_pt.push(lands_pt);
    r.card(db, GIANT)?.hand_options = Some(footsteps_options);
    let c = r.card(db, BONNY)?;
    c.etb = Some(bonny_etb);
    r.card(db, ENTISH)?.resolve = Some(entish_resolve);
    let c = r.card(db, LOOT)?;
    c.extra_lands = Some(loot_extra);
    c.options = Some(loot_options);
    r.card(db, HYDRA)?.as_enters = Some(hydra_as);
    walker(r, db, WRENN, &WRENN_ABS)?;
    let c = r.card(db, MARKET)?;
    c.dies = Some(market_dies);
    c.main1 = Some(market_main);
    let c = r.card(db, DRIVNOD)?;
    c.trigger_copies = Some(drivnod_copies);
    c.options = Some(drivnod_options);
    r.card(db, INDEBTED)?.hand_options = Some(indebted_options);
    r.card(db, BROODMOTH)?.dies = Some(broodmoth_dies);
    let c = r.card(db, SCION)?;
    c.dies = Some(scion_dies);
    c.options = Some(scion_options);
    Ok(())
}
