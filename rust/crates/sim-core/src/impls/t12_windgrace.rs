//! Tier 2 pool deck `lord-windgrace-jund-lands` (EDHREC's bracket-3 average list): the cards the ability language
//! doesn't cover (rust/agents/t12-windgrace.md). The typed discard costs (Soul of Windgrace's "discard a land card"),
//! "Tap it." and "the number of lands you control" (Lumra's power and toughness) are compiled (dsl_parse.rs,
//! `// --- Tiers 1/2 (windgrace)`); Gruul Turf and Rakdos Carnarium are bounce lands (card_overrides.toml); the rest
//! is here.
//!
//! - Constant Mists: cast in the declare-blockers window when an attack hurts (`hand_defend`): all combat damage to
//!   players is prevented this turn (`g.fog`); buyback (sacrifice a land) with seven or more lands, or four with a
//!   way to replay lands from the graveyard. Left out: combat damage between creatures is still dealt.
//! - Nissa of Shadowed Boughs: landfall, a loyalty counter; +1 untap a land of yours (a tapped one making the most),
//!   which becomes a 3/3 haste menace creature until end of turn when it has an open attack; -5 the best creature card
//!   with mana value up to your land count from hand or graveyard, with two +1/+1 counters.
//! - Realms Uncharted: up to four lands with different names; the opponent puts the two best into the graveyard, the
//!   other two go to hand.
//! - Scapeshift: sacrifices lands (as many as the library has lands for), finds that many: Dark Depths and
//!   Thespian's Stage, Field of the Dead, then new names and colours; each enters tapped with its landfall.
//! - Dark Depths: ten ice counters; {3}: remove one (with spare mana at the end of an opponent's turn, or in the
//!   second main phase); the last one gone: it is sacrificed for Marit Lage, a legendary 20/20 black Avatar with
//!   flying and indestructible.
//! - Thespian's Stage: {2},{T}: a copy of Dark Depths (the combo `Dark Depths + Thespian's Stage`, chained in
//!   combos::all): the legend rule puts Dark Depths into the graveyard, the copy with no ice counters is sacrificed
//!   for Marit Lage. Left out: copying any other land.
//! - Elvish Reclaimer: +2/+2 with three or more land cards in your graveyard; {2},{T}, sacrifice a land (a tapped one,
//!   never a key land): a land onto the battlefield tapped (the missing half of Depths and Stage, Field of the Dead,
//!   a colour; at the end of an opponent's turn unless it finds a combo half).
//! - Icetill Explorer: an additional land each turn; lands from your graveyard (its landfall mill is compiled).
//! - Lotus Field: three mana of any one colour (tags); enters tapped; when it enters, sacrifice two lands (the least
//!   useful others first, itself if there aren't two).
//! - Lumra, Bellow of the Woods: enters: mill four, then every land card in your graveyard onto the battlefield
//!   tapped (its power and toughness are compiled).
//! - Mina and Denn, Wildborn: an additional land each turn; {R}{G}, return a land you control to hand: target creature
//!   gains trample (your best creature): used to replay the land at once for another landfall when a land drop is
//!   left and landfall payoffs are out.
//! - Six: attacks: mill three, a land card milled into hand; during your turn, nonland permanent cards in your
//!   graveyard have retrace (cast from the graveyard by discarding a land card).
//! - Soul of Windgrace: enters or attacks: the best land card from any graveyard onto the battlefield tapped under
//!   your control (its three activated abilities are compiled).
//! - Spelunking: enters: draw a card, then the best land from hand onto the battlefield (a Cave: gain 4 life); lands
//!   you control enter untapped (`lands_enter_untapped`, read by `Game::add_land` and the land-drop scoring).
//! - Tannuk, Memorial Ensign: landfall, 1 damage to each opponent; the second time it resolves in a turn, draw a card.
//! - Worldsoul's Rage: X (all spare mana, tag `tokx`) damage to the best target; then up to X land cards from hand
//!   and graveyard onto the battlefield tapped.

use crate::cards::{CardDb, Colors};
use crate::engine::cast::{cast_card, castable, on_cast};
use crate::engine::hooks::hooked;
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, land_colors_now, pay, total_mana};
use crate::engine::stack::{ability_window, ability_window_card, counter_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::epow;
use crate::engine::zones::{
    Enter, Tokens, discard_cards, draw, enter, landfall, make_tokens, max_by, mill, min_by, searchable,
};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::impls::combos::{Combo, Ready, Step, StepKind, done, no_src};
use crate::impls::common::{WalkerAb, walker};
use crate::impls::lands::{animate, can_pay_without, open_attack, pay_without, sac_land};
use crate::impls::partials::{at_once, eot_kw, remove_card};
use crate::state::{Ctx, DataKey, Game, PermData, Step as TurnStep, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

const DEPTHS: &str = "Dark Depths";
const STAGE: &str = "Thespian's Stage";
const FIELD: &str = "Field of the Dead";
const LOTUS: &str = "Lotus Field";
const LAGE: &str = "Marit Lage";
const SPELUNKING: &str = "Spelunking";

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

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn lname(g: &Game, l: LandId) -> &str {
    cname(g, g.land(l).cd)
}

/// p's land of this name on the battlefield
fn land_named(g: &Game, p: PlayerId, name: &str) -> Option<LandId> {
    g.player(p).lands.iter().copied().find(|&l| lname(g, l) == name)
}

fn gy_lands(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).land).collect()
}

fn hand_lands(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).land).collect()
}

/// p's land drops this turn (Exploration, Azusa, Icetill Explorer ...)
fn land_drops(g: &Game, p: PlayerId) -> u32 {
    1 + crate::cardcode::extra_lands(g, p) + g.player(p).extra_land_now
}

/// p's landfall payoffs on the battlefield (card code that handles landfall), plus a live Field of the Dead
fn landfall_payoffs(g: &Game, p: PlayerId) -> i32 {
    let hooks = if g.hooks.is_empty() {
        0
    } else {
        hooked(g, Event::Landfall).iter().filter(|(s, _)| owner(g, *s) == p).count() as i32
    };
    let field = land_named(g, p, FIELD).is_some() && g.player(p).lands.len() >= 6;
    hooks + field as i32
}

/// p's payoffs for a land going to the graveyard from the battlefield (Titania, The Gitrog Monster)
fn land_gy_payoffs(g: &Game, p: PlayerId) -> i32 {
    if g.hooks.is_empty() {
        return 0;
    }
    hooked(g, Event::LandGy).iter().filter(|(s, _)| owner(g, *s) == p).count() as i32
}

/// Spelunking: "Lands you control enter untapped." (Game::add_land and turn::land_enters_tapped ask)
pub fn lands_enter_untapped(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        x.on_bf && !x.phased && !x.neutered && x.cd.is_some_and(|c| cname(g, c) == SPELUNKING)
    })
}

/// --- casual speed: the land-drop choice (turn::land_score): Lotus Field with fewer than two other lands sacrifices
/// itself (and the one other land), a lost land drop, so any other land goes first
pub fn land_adjust(g: &Game, p: PlayerId, c: CardId) -> f64 {
    if cname(g, c) == LOTUS && g.player(p).lands.len() < 2 { -20.0 } else { 0.0 }
}

/// a land p keeps: the Dark Depths line, Field of the Dead, lands that make two or more mana
fn key_land(g: &Game, l: LandId) -> bool {
    let d = g.db.get(g.land(l).cd);
    matches!(&*d.name, DEPTHS | STAGE | FIELD | "Gaea's Cradle" | "Urborg, Tomb of Yawgmoth")
        || d.tags.int(Tag::Amt).unwrap_or(1) >= 2
}

/// p's least useful land (not `except`): not a key land, tapped first, the fewest colours, a basic first
fn worst_land(g: &Game, p: PlayerId, except: Option<LandId>) -> Option<LandId> {
    let ls: Vec<LandId> = g.player(p).lands.iter().copied().filter(|&l| Some(l) != except).collect();
    min_by(&ls, |l| {
        let d = g.db.get(g.land(l).cd);
        100.0 * key_land(g, l) as i32 as f64
            + 10.0 * !g.land(l).tapped as i32 as f64
            + land_colors_now(g, p, l).count() as f64
            + 0.5 * !d.basic as i32 as f64
    })
}

/// card c (already out of its zone) enters as p's land: its enter effect (Lotus Field, Bojuka Bog), landfall, a
/// fetch land cracked at once
pub(crate) fn land_onto_bf(g: &mut Game, p: PlayerId, c: CardId, tapped: bool) -> Res {
    let l = g.add_land(p, c, tapped);
    crate::glog!(g, "    {} puts {} onto the battlefield", pname(g, p), cname(g, c));
    crate::cardcode::land_etb(g, p, l)?;
    landfall(g, p)?;
    if g.db.get(c).tag(Tag::F) && g.player(p).lands.contains(&l) {
        crate::engine::turn::crack_fetch(g, p, l)?;
    }
    Ok(())
}

/// the library half of Dark Depths + Thespian's Stage when p has (or `with`) the other one
pub(crate) fn depths_half(g: &Game, p: PlayerId, cands: &[CardId], with: &[CardId]) -> Option<CardId> {
    let have = |n: &str| land_named(g, p, n).is_some() || with.iter().any(|&c| cname(g, c) == n);
    let want = if have(DEPTHS) && !have(STAGE) {
        STAGE
    } else if have(STAGE) && !have(DEPTHS) {
        DEPTHS
    } else {
        return None;
    };
    cands.iter().copied().find(|&c| cname(g, c) == want)
}

/// how much p wants land card c next to its lands and the ones already `chosen` (searches, Scapeshift, recursion)
fn land_want(g: &Game, p: PlayerId, c: CardId, chosen: &[CardId]) -> f64 {
    let d = g.db.get(c);
    let name = &*d.name;
    let on_bf = |n: &str| land_named(g, p, n).is_some() || chosen.iter().any(|&x| cname(g, x) == n);
    let mut v = 0.0;
    match name {
        DEPTHS if on_bf(STAGE) => v += 50.0,
        STAGE if on_bf(DEPTHS) => v += 50.0,
        DEPTHS | STAGE => v += 4.0,
        FIELD if !on_bf(FIELD) => v += 6.0 + (g.player(p).lands.len() >= 5) as i32 as f64 * 8.0,
        LOTUS => v -= if g.player(p).lands.len() >= 8 { 0.0 } else { 20.0 },
        _ => {}
    }
    if !on_bf(name) {
        v += 1.5; // a new name (Field of the Dead)
    }
    let have: Colors = g.player(p).lands.iter().fold(Colors::NONE, |a, &l| a.union(land_colors_now(g, p, l)));
    let cols =
        d.tags.str(Tag::C).map_or(Colors::NONE, |s| if s == "A" { g.player(p).ident } else { Colors::from_letters(s) });
    v += 2.0 * "WUBRG".chars().filter(|&ch| cols.has(ch) && !have.has(ch) && g.player(p).ident.has(ch)).count() as f64;
    v += d.tags.int(Tag::Amt).unwrap_or(1).max(1) as f64 - 1.0;
    if d.tag(Tag::F) {
        v += 0.8 * landfall_payoffs(g, p) as f64; // a second landfall
    }
    v + 0.01 * card_worth(g, p, c, false)
}

/// the best k land cards of `cands` for p, picked one at a time (different names when `distinct`)
fn pick_lands(g: &Game, p: PlayerId, cands: &[CardId], k: usize, distinct: bool) -> Vec<CardId> {
    let mut out: Vec<CardId> = vec![];
    let mut pool: Vec<CardId> = cands.to_vec();
    while out.len() < k {
        pool.retain(|&c| !out.contains(&c) && (!distinct || !out.iter().any(|&x| cname(g, x) == cname(g, c))));
        let Some(c) = max_by(&pool, |c| land_want(g, p, c, &out)) else { break };
        out.push(c);
    }
    out
}

/// Marit Lage: a legendary 20/20 black Avatar creature token with flying and indestructible
fn marit_lage(g: &mut Game, p: PlayerId) -> Res<Option<PermId>> {
    let mut data = PermData::default();
    data.set(DataKey::Indestr, Val::Bool(true));
    let spec =
        Tokens { fly: true, color: Some(Colors::from_letters("B")), types: vec!["avatar"], data, ..Tokens::new(1, 20) };
    let ms = make_tokens(g, p, spec)?;
    for &m in &ms {
        g.perm_mut(m).name = intern(LAGE);
    }
    crate::glog!(g, "    {} creates Marit Lage", pname(g, p));
    Ok(ms.first().copied())
}

fn lages(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && &*g.perm(m).name == LAGE).collect()
}

// ================================================================== Dark Depths
/// ice counters left on Dark Depths: it enters with ten (the land keeps the number removed)
pub fn ice(g: &Game, l: LandId) -> i64 {
    (10 - g.land(l).data.int(DataKey::IceRemoved)).max(0)
}

/// {3}: remove an ice counter, with mana to spare: at the end of an opponent's turn, or in the second main phase
fn depths_options(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !g.player(p).lands.contains(&l) || ice(g, l) == 0 || !can_pay_without(g, p, l, 3, "") {
        return Ok(vec![]);
    }
    let eot = post.is_none() && g.active != Some(p);
    if !eot && post != Some(true) {
        return Ok(vec![]);
    }
    let u = if eot { 1.0 } else { 0.15 } + 0.1 * (10 - ice(g, l)) as f64;
    Ok(vec![Opt {
        utility: u,
        label: format!("{DEPTHS}: remove an ice counter ({} left)", ice(g, l)),
        act: Some(Action::Plan { f: depths_go, arg: l.0 as i64 }),
    }])
}

fn depths_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !g.player(p).lands.contains(&l) || ice(g, l) == 0 || !pay_without(g, p, l, 3, "")? {
        return Ok(false);
    }
    let cd = g.land(l).cd;
    if !ability_window_card(g, p, cd, "remove an ice counter", None, None)? || !g.player(p).lands.contains(&l) {
        return Ok(true);
    }
    let n = g.land(l).data.int(DataKey::IceRemoved) + 1;
    g.land_mut(l).data.set(DataKey::IceRemoved, Val::Int(n));
    crate::glog!(g, "    {} removes an ice counter from Dark Depths ({} left)", pname(g, p), ice(g, l));
    if ice(g, l) == 0 {
        // "When Dark Depths has no ice counters on it, sacrifice it. If you do, create Marit Lage"
        sac_land(g, p, l)?;
        marit_lage(g, p)?;
    }
    Ok(true)
}

// ================================================================== Thespian's Stage (+ Dark Depths)
/// Dark Depths and an untapped Thespian's Stage, and {2} besides the Stage
fn stage_ready(g: &Game, p: PlayerId) -> Ready {
    let (Some(_), Some(s)) = (land_named(g, p, DEPTHS), land_named(g, p, STAGE)) else {
        return (false, vec![], vec![]);
    };
    if g.land(s).tapped || total_mana(g, p, false) < 3 {
        return (false, vec![], vec![]);
    }
    (true, vec![], vec![])
}

/// {2},{T}: the Stage becomes a copy of Dark Depths with no ice counters: the legend rule puts Dark Depths into the
/// graveyard, the copy's trigger sacrifices it for Marit Lage
fn stage_copy(g: &mut Game, p: PlayerId) -> Res<bool> {
    let (Some(d), Some(s)) = (land_named(g, p, DEPTHS), land_named(g, p, STAGE)) else { return Ok(false) };
    if g.land(s).tapped || !pay_without(g, p, s, 2, "")? {
        return Ok(false);
    }
    g.land_mut(s).tapped = true;
    let cd = g.land(s).cd;
    crate::glog!(g, "  {} activates Thespian's Stage: a copy of Dark Depths", pname(g, p));
    if !ability_window_card(g, p, cd, "becomes a copy of Dark Depths", Some(8.0), None)? {
        return Ok(false);
    }
    if !g.player(p).lands.contains(&s) || !g.player(p).lands.contains(&d) {
        return Ok(false);
    }
    sac_land(g, p, d)?; // the legend rule
    sac_land(g, p, s)?;
    Ok(marit_lage(g, p)?.is_some())
}

fn no_finish(_g: &mut Game, _p: PlayerId) -> Res {
    Ok(())
}

/// Dark Depths + Thespian's Stage: Marit Lage, which opponents may answer (exile, bounce, -X/-X) before it attacks
pub static COMBOS: [Combo; 1] = [Combo {
    name: "Dark Depths + Thespian's Stage",
    groups: &[&[DEPTHS], &[STAGE]],
    ready: stage_ready,
    mana: (0, ""),
    finish: Some(no_finish),
    xcost: &[],
    steps: &[
        Step {
            label: "Thespian's Stage copies Dark Depths",
            kind: StepKind::Engine,
            src: no_src,
            pay: (0, ""),
            run: stage_copy,
            keys: crate::impls::combos::no_keys,
        },
        Step { label: "Marit Lage", kind: StepKind::Permanent, src: no_src, pay: (0, ""), run: done, keys: lages },
    ],
}];

// ================================================================== Lotus Field
/// When it enters, sacrifice two lands: the least useful others, itself when there aren't two
fn lotus_etb(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    for _ in 0..2 {
        let x = worst_land(g, p, Some(l)).unwrap_or(l);
        if !g.player(p).lands.contains(&x) {
            break;
        }
        crate::glog!(g, "    Lotus Field: {} sacrifices {}", pname(g, p), lname(g, x));
        sac_land(g, p, x)?;
        if x == l {
            break;
        }
    }
    Ok(())
}

// ================================================================== extra land drops, lands from the graveyard
fn extra_land(g: &Game, src: Src, p: PlayerId) -> i32 {
    (owner(g, src) == p) as i32
}

// ================================================================== Elvish Reclaimer
fn reclaimer_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    if gy_lands(g, p).len() >= 3 { (2, 2) } else { (0, 0) }
}

fn selfpt_on(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

/// the land Reclaimer should find and how much it's worth: the missing half of Depths and Stage, Field of the Dead,
/// a new colour
fn reclaimer_target(g: &Game, p: PlayerId) -> Option<(CardId, f64)> {
    let lib: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| g.db.get(c).land).collect();
    if let Some(c) = depths_half(g, p, &lib, &[]) {
        return Some((c, 9.0));
    }
    let c = pick_lands(g, p, &lib, 1, false).into_iter().next()?;
    let v = land_want(g, p, c, &[]);
    Some((c, 0.4 + 0.15 * v + 0.4 * landfall_payoffs(g, p) as f64))
}

/// {2},{T}, sacrifice a land: at the end of an opponent's turn, or at once for a combo half or Field of the Dead
fn reclaimer_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if p != owner(g, src) || !live(g, src) || x.tapped || (x.sick && !crate::engine::combat::has_haste(g, src)) {
        return Ok(vec![]);
    }
    if worst_land(g, p, None).is_none_or(|l| key_land(g, l)) || !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    let Some((c, v)) = reclaimer_target(g, p) else { return Ok(vec![]) };
    let now = v >= 5.0 || cname(g, c) == FIELD;
    let eot = post.is_none() && g.active != Some(p);
    if !(eot || (now && post.is_some())) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: v,
        label: format!("Elvish Reclaimer: {}", cname(g, c)),
        act: Some(Action::Ability { src, f: reclaimer_go, arg: 0 }),
    }])
}

fn reclaimer_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || g.perm(src).tapped || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    g.perm_mut(src).tapped = true;
    let Some(l) = worst_land(g, p, None) else { return Ok(false) };
    crate::glog!(g, "  {} activates Elvish Reclaimer, sacrificing {}", pname(g, p), lname(g, l));
    sac_land(g, p, l)?;
    if !ability_window(g, p, Some(src), "search for a land card", None, None)? {
        return Ok(true);
    }
    if let Some((c, _)) = reclaimer_target(g, p) {
        remove_card(&mut g.player_mut(p).library, c);
        shuffle_library(g, p);
        land_onto_bf(g, p, c, true)?;
    } else {
        shuffle_library(g, p);
    }
    Ok(true)
}

// ================================================================== Mina and Denn, Wildborn
/// {R}{G}, return a land you control to its owner's hand: target creature gains trample until end of turn. Used when
/// a land drop is left and nothing in hand to play: the land is played again at once (another landfall)
fn mina_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || !live(g, src) || post.is_none() || g.active != Some(p) {
        return Ok(vec![]);
    }
    let pl = g.player(p);
    if pl.lands_played >= land_drops(g, p) || !hand_lands(g, p).is_empty() || pl.lands.len() < 3 {
        return Ok(vec![]);
    }
    let pay_off = landfall_payoffs(g, p);
    if pay_off == 0 || !can_pay(g, p, 0, "RG", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 0.6 + 0.9 * pay_off as f64,
        label: "Mina and Denn: return a land to replay it".into(),
        act: Some(Action::Ability { src, f: mina_go, arg: 0 }),
    }])
}

fn mina_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || !can_pay(g, p, 0, "RG", false) {
        return Ok(false);
    }
    pay(g, p, 0, "RG", false)?;
    // a tapped land first (it comes back untapped), a fetch land or bounce land before a basic
    let ls = g.player(p).lands.clone();
    let Some(l) = max_by(&ls, |l| {
        let d = g.db.get(g.land(l).cd);
        10.0 * g.land(l).tapped as i32 as f64 + 3.0 * d.tag(Tag::F) as i32 as f64 - 20.0 * key_land(g, l) as i32 as f64
    }) else {
        return Ok(false);
    };
    let c = g.land(l).cd;
    crate::engine::turn::remove_land(g, p, l);
    g.player_mut(p).hand.push(c);
    crate::glog!(g, "  {} activates Mina and Denn, returning {}", pname(g, p), cname(g, c));
    let mine: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect();
    if ability_window(g, p, Some(src), "trample", None, None)?
        && let Some(m) = max_by(&mine, |m| epow(g, m) as f64)
    {
        eot_kw(g, m, "trample");
    }
    if g.player(p).lands_played < land_drops(g, p) && remove_card(&mut g.player_mut(p).hand, c) {
        crate::engine::turn::play_land_card(g, p, c, "plays")?;
    }
    Ok(true)
}

// ================================================================== Lumra, Bellow of the Woods
/// When Lumra enters, mill four cards. Then return all land cards from your graveyard to the battlefield tapped.
fn lumra_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != owner(g, src) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "mill four, every land card back", Some(5.0))? {
        return Ok(());
    }
    let n = g.player(p).library.len().min(4);
    mill(g, p, n)?;
    for c in gy_lands(g, p) {
        if remove_card(&mut g.player_mut(p).gy, c) {
            land_onto_bf(g, p, c, true)?;
        }
    }
    Ok(())
}

// ================================================================== Six
/// Whenever Six attacks, mill three cards. You may put a land card from among them into your hand.
fn six_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p != owner(g, src) || !atk.contains(&src) || !trigger_window(g, p, Some(src), "mill three", None)? {
        return Ok(vec![]);
    }
    let lib = &g.player(p).library;
    let top: Vec<CardId> = lib[lib.len().saturating_sub(3)..].to_vec();
    mill(g, p, top.len())?;
    let milled: Vec<CardId> = top.into_iter().filter(|&c| g.db.get(c).land && g.player(p).gy.contains(&c)).collect();
    if let Some(c) = pick_lands(g, p, &milled, 1, false).into_iter().next() {
        remove_card(&mut g.player_mut(p).gy, c);
        g.player_mut(p).hand.push(c);
        crate::glog!(g, "    {} puts {} into hand (Six)", pname(g, p), cname(g, c));
    }
    Ok(vec![])
}

/// During your turn, nonland permanent cards in your graveyard have retrace: the best one cast from the graveyard,
/// discarding the least useful land card
fn six_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || !live(g, src) || post.is_none() || g.active != Some(p) || hand_lands(g, p).is_empty() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for c in g.player(p).gy.clone() {
        let d = g.db.get(c);
        if d.land || !d.perm || !castable(g, p, c, "gy") {
            continue;
        }
        let (gn, pips) = crate::engine::mana::cost_of(g, p, c);
        if !can_pay(g, p, gn, &pips, false) {
            continue;
        }
        let v = crate::ai::decks::deck_prio(g, p, c) as f64 / 10.0 + 0.3 * card_worth(g, p, c, true) / 10.0 - 0.3;
        if v <= 0.5 {
            continue;
        }
        out.push(Opt {
            utility: v,
            label: format!("{} from the graveyard (retrace, Six)", cname(g, c)),
            act: Some(Action::Plan { f: retrace_go, arg: c.0 as i64 }),
        });
    }
    Ok(out)
}

fn retrace_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (gn, pips) = crate::engine::mana::cost_of(g, p, c);
    let ls = hand_lands(g, p);
    let Some(land) = min_by(&ls, |x| land_want(g, p, x, &[])) else { return Ok(false) };
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    discard_cards(g, p, &[land])?;
    crate::glog!(
        g,
        "  {} casts {} from the graveyard (retrace), discarding {}",
        pname(g, p),
        cname(g, c),
        cname(g, land)
    );
    cast_card(g, p, c, "mgy", Ctx::default())?;
    Ok(true)
}

// ================================================================== Soul of Windgrace
/// the best land card in any graveyard for p: (its owner, the card)
fn soul_pick(g: &Game, p: PlayerId) -> Option<(PlayerId, CardId)> {
    let all: Vec<(PlayerId, CardId)> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.gy.iter().copied().filter(|&c| g.db.get(c).land).map(move |c| (q.id, c)))
        .collect();
    max_by(&all, |x| land_want(g, p, x.1, &[]))
}

/// Whenever Soul of Windgrace enters or attacks, you may put a land card from a graveyard onto the battlefield tapped
/// under your control.
fn soul_land(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let Some((q, c)) = soul_pick(g, p) else { return Ok(()) };
    if land_want(g, p, c, &[]) < 0.0 || !trigger_window(g, p, Some(src), "a land from a graveyard", None)? {
        return Ok(());
    }
    if remove_card(&mut g.player_mut(q).gy, c) {
        land_onto_bf(g, p, c, true)?;
    }
    Ok(())
}

fn soul_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && p == owner(g, src) {
        soul_land(g, src, p)?;
    }
    Ok(())
}

fn soul_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p == owner(g, src) && atk.contains(&src) {
        soul_land(g, src, p)?;
    }
    Ok(vec![])
}

// ================================================================== Spelunking
/// When this enchantment enters, draw a card, then you may put a land card from your hand onto the battlefield. If
/// you put a Cave onto the battlefield this way, you gain 4 life.
fn spelunking_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != owner(g, src) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "draw, then a land from hand", None)? {
        return Ok(());
    }
    draw(g, p, 1, false)?;
    let ls = hand_lands(g, p);
    let Some(c) = pick_lands(g, p, &ls, 1, false).into_iter().next() else { return Ok(()) };
    if land_want(g, p, c, &[]) < 0.0 || !remove_card(&mut g.player_mut(p).hand, c) {
        return Ok(());
    }
    land_onto_bf(g, p, c, false)?;
    if g.db.get(c).subtypes.iter().any(|s| &**s == "cave") {
        gain(g, p, 4)?;
    }
    Ok(())
}

// ================================================================== Tannuk, Memorial Ensign
/// Landfall: 1 damage to each opponent; the second time this ability resolves this turn, draw a card
fn tannuk_landfall(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !trigger_window(g, p, Some(src), "1 damage to each opponent", None)? {
        return Ok(());
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        lose_life(g, q, 1, Some(p), "triggers", None)?;
    }
    let st = g.turn_stamp();
    let n = match g.perm(src).data.get(DataKey::Resolved) {
        Some(Val::List(v)) if v.first() == Some(&Val::Stamp(st)) => v.get(1).map_or(0, Val::int),
        _ => 0,
    } + 1;
    g.perm_mut(src).data.set(DataKey::Resolved, Val::List(vec![Val::Stamp(st), Val::Int(n)]));
    if n == 2 {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

// ================================================================== Nissa of Shadowed Boughs
/// Landfall: a loyalty counter on Nissa
fn nissa_landfall(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src) && live(g, src) {
        let l = g.perm(src).loyalty.unwrap_or(4);
        g.perm_mut(src).loyalty = Some(l + 1);
    }
    Ok(())
}

/// +1: untap target land you control (a tapped one making the most); with an open attack before combat on your turn,
/// it becomes a 3/3 Elemental creature with haste and menace until end of turn
fn nissa_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let ls = g.player(p).lands.clone();
    let Some(l) = max_by(&ls, |l| {
        let d = g.db.get(g.land(l).cd);
        10.0 * g.land(l).tapped as i32 as f64 + d.tags.int(Tag::Amt).unwrap_or(1) as f64
    }) else {
        return Ok(());
    };
    g.land_mut(l).tapped = false;
    let precombat = g.active == Some(p) && matches!(g.step, TurnStep::Start | TurnStep::Main1);
    if precombat && open_attack(g, p, 3, true) && !matches!(lname(g, l), DEPTHS | STAGE) {
        animate(g, p, l, 3, 3, &["haste", "menace"], false);
    }
    Ok(())
}

/// the best creature card in p's hand or graveyard with mana value up to p's land count
fn nissa_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let n = g.player(p).lands.len() as u32;
    let pl = g.player(p);
    let cs: Vec<CardId> =
        pl.hand.iter().chain(&pl.gy).copied().filter(|&c| g.db.get(c).creature && g.db.get(c).cmc <= n).collect();
    max_by(&cs, |c| card_worth(g, p, c, false) + g.db.get(c).cmc as f64)
}

/// -5: the creature card onto the battlefield with two +1/+1 counters
fn nissa_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let Some(c) = nissa_pick(g, p) else { return Ok(()) };
    if !remove_card(&mut g.player_mut(p).hand, c) && !remove_card(&mut g.player_mut(p).gy, c) {
        return Ok(());
    }
    crate::glog!(g, "    {} puts {} onto the battlefield (Nissa)", pname(g, p), cname(g, c));
    enter(g, p, c, Enter { plus: 2, ..Enter::default() })?;
    Ok(())
}

static NISSA: [WalkerAb; 2] = [
    WalkerAb {
        delta: 1,
        label: "untap a land (a 3/3 haste menace)",
        val: |g, p, _| Some(2.0 + if open_attack(g, p, 3, true) { 1.0 } else { 0.0 }),
        eff: nissa_plus,
    },
    WalkerAb {
        delta: -5,
        label: "a creature with two +1/+1 counters",
        val: |g, p, _| nissa_pick(g, p).map(|c| 2.0 + 0.6 * g.db.get(c).cmc as f64),
        eff: nissa_minus,
    },
];

// ================================================================== Constant Mists
/// cast in response to an attack that hurts: all combat damage this turn is prevented; buyback with lands to spare
fn mists_defend(
    g: &mut Game,
    c: CardId,
    d: PlayerId,
    p: PlayerId,
    atk: &mut Vec<PermId>,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    let st = g.turn_stamp();
    if g.fog == Some(st) || !g.player(d).hand.contains(&c) || !castable(g, d, c, "hand") {
        return Ok(());
    }
    let (gn, pips) = crate::engine::mana::cost_of(g, d, c);
    if !can_pay(g, d, gn, &pips, false) {
        return Ok(());
    }
    let power: i32 = atk
        .iter()
        .filter(|&&a| g.perm(a).on_bf && g.perm(a).owner == p && !assign.iter().any(|x| x.0 == a))
        .map(|&a| epow(g, a))
        .sum();
    let life = g.player(d).life;
    if power < life && (power as f64) < (7.0f64).max(life as f64 * 0.4) {
        return Ok(());
    }
    pay(g, d, gn, &pips, false)?;
    remove_card(&mut g.player_mut(d).hand, c);
    crate::glog!(g, "  {} casts Constant Mists", pname(g, d));
    // buyback: sacrifice a land as it is cast (seven or more, or four with lands to replay from the graveyard)
    let spare = g.player(d).lands.len() >= 7 || (g.player(d).lands.len() >= 4 && crate::cardcode::lands_from_gy(g, d));
    let buyback = spare && worst_land(g, d, None).is_some_and(|l| !key_land(g, l));
    if buyback && let Some(l) = worst_land(g, d, None) {
        crate::glog!(g, "    buyback: {} sacrifices {}", pname(g, d), lname(g, l));
        sac_land(g, d, l)?;
    }
    on_cast(g, d, c)?;
    if !counter_window(g, d, c, 4.0, vec![])? {
        g.player_mut(d).gy.push(c);
        return Ok(());
    }
    g.fog = Some(st);
    crate::glog!(g, "    Constant Mists prevents all combat damage this turn");
    if buyback {
        g.player_mut(d).hand.push(c)
    } else {
        g.player_mut(d).gy.push(c)
    }
    Ok(())
}

// ================================================================== Realms Uncharted
/// up to four lands with different names; an opponent puts the two best into the graveyard, the rest go to hand
fn realms_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let lib: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| g.db.get(c).land).collect();
    let mut got = pick_lands(g, p, &lib, 4, true);
    for &c in &got {
        remove_card(&mut g.player_mut(p).library, c);
    }
    shuffle_library(g, p);
    got.sort_by(|&a, &b| land_want(g, p, b, &[]).total_cmp(&land_want(g, p, a, &[])));
    for (i, &c) in got.iter().enumerate() {
        // the opponent chooses two (all of them, with two or fewer found): the best ones
        if i < 2 {
            g.player_mut(p).gy.push(c);
        } else {
            g.player_mut(p).hand.push(c);
        }
    }
    crate::glog!(g, "    Realms Uncharted: {} lands found", got.len());
    Ok("gy")
}

fn realms_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let lands = hand_lands(g, p).len();
    if lands == 0 {
        50
    } else if lands == 1 && crate::cardcode::lands_from_gy(g, p) {
        35
    } else {
        0
    }
}

// ================================================================== Scapeshift
/// how many lands Scapeshift should sacrifice (and find): all of them, up to the lands in the library
fn shift_n(g: &Game, p: PlayerId) -> usize {
    let lib = searchable(g, p).into_iter().filter(|&c| g.db.get(c).land).count();
    g.player(p).lands.len().min(lib)
}

fn shift_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let n = shift_n(g, p) as i32;
    let lib: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| g.db.get(c).land).collect();
    let combo = (lib.iter().any(|&c| cname(g, c) == DEPTHS) && lib.iter().any(|&c| cname(g, c) == STAGE))
        || depths_half(g, p, &lib, &[]).is_some();
    let field = lib.iter().any(|&c| cname(g, c) == FIELD) || land_named(g, p, FIELD).is_some();
    let value = n * (landfall_payoffs(g, p) + land_gy_payoffs(g, p));
    if (combo && n >= 2) || (field && n >= 7) || value >= 6 { 60 } else { 0 }
}

/// Sacrifice any number of lands. Search your library for up to that many land cards, put them onto the battlefield
/// tapped, then shuffle.
fn shift_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = shift_n(g, p);
    for _ in 0..n {
        let Some(l) = worst_land(g, p, None).or_else(|| g.player(p).lands.first().copied()) else { break };
        sac_land(g, p, l)?;
    }
    let lib: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| g.db.get(c).land).collect();
    let got = pick_lands(g, p, &lib, n, false);
    for &c in &got {
        remove_card(&mut g.player_mut(p).library, c);
    }
    shuffle_library(g, p);
    crate::glog!(g, "    Scapeshift: {} sacrifices {n} lands and finds {}", pname(g, p), got.len());
    for c in got {
        land_onto_bf(g, p, c, true)?;
    }
    Ok("gy")
}

// ================================================================== Worldsoul's Rage
fn rage_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let x = total_mana(g, p, false) as i32 - 2;
    let lands = (hand_lands(g, p).len() + gy_lands(g, p).len()) as i32;
    let lethal = g.opps(p).any(|q| g.player(q).life <= x);
    if x >= 1 && (lethal || (x >= 3 && lands >= 2) || x >= 6) { 55 } else { 0 }
}

/// X damage to the best target; then up to X land cards from hand and/or graveyard onto the battlefield tapped
fn rage_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x.max(0);
    if x > 0 {
        crate::impls::t3::best_target_any(g, p, x)?;
    }
    let pl = g.player(p);
    let cands: Vec<CardId> = pl.hand.iter().chain(&pl.gy).copied().filter(|&c| g.db.get(c).land).collect();
    let got: Vec<CardId> =
        pick_lands(g, p, &cands, x as usize, false).into_iter().filter(|&c| land_want(g, p, c, &[]) >= 0.0).collect();
    for c in got {
        if remove_card(&mut g.player_mut(p).hand, c) || remove_card(&mut g.player_mut(p).gy, c) {
            land_onto_bf(g, p, c, true)?;
        }
    }
    Ok("gy")
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Constant Mists")?;
    c.hand_defend = Some(mists_defend);
    c.prio = Some(|_, _, _| 0); // a response only (`mists_defend`)
    walker(r, db, "Nissa of Shadowed Boughs", &NISSA)?;
    let c = r.card(db, "Nissa of Shadowed Boughs")?;
    c.landfall = Some(nissa_landfall);
    at_once(c, Event::Landfall, true);
    let c = r.card(db, "Realms Uncharted")?;
    c.resolve = Some(realms_resolve);
    c.prio = Some(realms_prio);
    let c = r.card(db, "Scapeshift")?;
    c.resolve = Some(shift_resolve);
    c.prio = Some(shift_prio);
    r.card(db, DEPTHS)?.land_options = Some(depths_options);
    let c = r.card(db, "Elvish Reclaimer")?;
    c.self_pt = Some(reclaimer_pt);
    c.etb = Some(selfpt_on);
    *c = c.at_once(Event::Etb);
    c.options = Some(reclaimer_options);
    let c = r.card(db, "Icetill Explorer")?;
    c.extra_lands = Some(extra_land);
    c.lands_from_gy = Some(extra_land);
    r.card(db, LOTUS)?.land_etb = Some(lotus_etb);
    r.card(db, "Lumra, Bellow of the Woods")?.etb = Some(lumra_etb);
    let c = r.card(db, "Mina and Denn, Wildborn")?;
    c.extra_lands = Some(extra_land);
    c.options = Some(mina_options);
    let c = r.card(db, "Six")?;
    c.attack = Some(six_attack);
    c.options = Some(six_options);
    let c = r.card(db, "Soul of Windgrace")?;
    c.etb = Some(soul_etb);
    c.attack = Some(soul_attack);
    r.card(db, SPELUNKING)?.etb = Some(spelunking_etb);
    r.card(db, "Tannuk, Memorial Ensign")?.landfall = Some(tannuk_landfall);
    let c = r.card(db, "Worldsoul's Rage")?;
    c.resolve = Some(rage_resolve);
    c.prio = Some(rage_prio);
    Ok(())
}
