//! Tier 1 pool deck `lathril-golgari-elves` (EDHREC's bracket-2 "core" average list): the cards the ability language
//! doesn't run (rust/agents/t12-lathril.md). The general patterns are compiled in cardgen's dsl_parse.rs and run in
//! dsl/mod.rs, under `// --- Tiers 1/2 (lathril)`: "an Elf you control dies" / "another Elf you control enters"
//! triggers (Elderfang Venom's drain, Wolverine Riders' life, High Perfect Morcant's blight), "attacking Elves you
//! control have deathtouch" (Elderfang Venom), "each opponent blights N" (`blight` below), and Abomination of
//! Llanowar's count (Elves you control plus Elf cards in your graveyard).
//!
//! - Banner of Kinship: names a creature type as it enters (the deck's main one) and enters with a fellowship counter
//!   (kept as `plus`, so proliferate adds to them) per creature of yours of that type; those creatures get +1/+1 per
//!   counter.
//! - Bounty of Skemfar: the top six; a land onto the battlefield tapped and the best Elf card to hand, the rest to the
//!   bottom in a random order.
//! - Crown of Skemfar: +1/+1 per Elf you control and reach on the creature it enchants; {2}{G} from the graveyard back
//!   to the hand (at the end-of-turn window, or with spare mana after combat).
//! - Pact of the Serpent: the AI names its main creature type and targets itself (X cards and X life), or an opponent
//!   the X life loss kills. Left out: a person picks the type, not the target (always themselves).
//! - Champions of the Perfect: behold an Elf (engine/cast.rs pays it: an Elf card from the hand or an Elf you control,
//!   the cheapest to lose) and exile it; it returns to its owner's hand when Champions leaves. Its cast draw is
//!   compiled.
//! - Elven Chorus: creature spells from the top of the library (impls/t3_marwyn.rs `top_options`, as Realmwalker);
//!   its creatures' mana is the `rite` tag. Looking at the top card is information the AI already has.
//! - High Perfect Morcant: tap three untapped Elves (summoning-sick ones too): proliferate, at sorcery speed after
//!   combat when it adds counters worth having. Its blight trigger is compiled.
//! - Miara, Thorn of the Glade: it or another Elf of yours dies: pay {1} and 1 life to draw.
//! - Nullmage Shepherd: tap four untapped creatures: destroy the best opposing artifact or enchantment (at the end of
//!   an opponent's turn, or at once for a valuable one).
//! - Return Upon the Tide: the best creature card from your graveyard returns; an Elf brings two 1/1 Elf Warriors.
//!   Left out: foretell (it is always hard-cast for {4}{B}).
//! - Rhys the Exiled: {B}, sacrifice another Elf (the least valuable): regenerate. Its attack life gain is compiled.
//! - Ruthless Winnower: each player's upkeep, that player sacrifices a non-Elf creature (their least valuable).
//! - Tyvar Kell: Elves you control tap for {B} (`extra_mana`; the summoning-sick ones too while Tyvar, Jubilant
//!   Brawler gives haste to abilities); +1 a +1/+1 counter on an Elf (a tapped mana Elf first), untap it, deathtouch
//!   until end of turn; 0 a 1/1 Elf Warrior; -6 the emblem (an Elf spell you cast: draw two, and it has haste that
//!   turn: engine/combat.rs `has_haste`).

use crate::cards::{CardDb, Colors, Types};
use crate::engine::life::{check_state, lose_life};
use crate::engine::mana::{Source, Unit, can_pay, pay, total_mana};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::card_worth;
use crate::engine::values::{epow, etgh, has_type, no_sac, pval, untargetable};
use crate::engine::zones::{Enter, die, draw, enter, exile_perm, landfall, max_by, min_by, minus_counter, sac_worth};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{AURA, AuraSpec, WalkerAb, aura, walker};
use crate::impls::t1::elf_tokens;
use crate::state::{Ctx, DataKey, Game, Step, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

const BANNER: &str = "Banner of Kinship";
const BOUNTY: &str = "Bounty of Skemfar";
const CROWN: &str = "Crown of Skemfar";
const PACT: &str = "Pact of the Serpent";
const CHAMPIONS: &str = "Champions of the Perfect";
const CHORUS: &str = "Elven Chorus";
const MORCANT: &str = "High Perfect Morcant";
const MIARA: &str = "Miara, Thorn of the Glade";
const SHEPHERD: &str = "Nullmage Shepherd";
const TIDE: &str = "Return Upon the Tide";
const RHYS: &str = "Rhys the Exiled";
const WINNOWER: &str = "Ruthless Winnower";
const TYVAR: &str = "Tyvar Kell";
const TYVAR_EMBLEM: Sym = "tyvar_kell";

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn named(g: &Game, m: PermId, name: &str) -> bool {
    g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == name)
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

/// p's live creatures
fn creatures_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

/// p's live Elf creatures
fn elves_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    creatures_of(g, p).into_iter().filter(|&m| has_type(g, m, "elf")).collect()
}

fn elf_card(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.has_subtype("elf") || d.has_kw("changeling")
}

/// the mana a creature's own tap ability makes now (0 if it has none)
fn dork_mana(g: &Game, p: PlayerId, m: PermId) -> u32 {
    let Some(c) = g.perm(m).cd else { return 0 };
    if !g.db.get(c).tag(Tag::Dork) {
        return 0;
    }
    crate::cardcode::dyn_mana_perm(g, p, m).unwrap_or(1)
}

/// the untapped creatures (filtered) p would tap for a "tap N untapped creatures" cost: the ones that make no mana and
/// are worth least first; summoning-sick ones can pay it (no {T} symbol)
fn tappers(g: &Game, p: PlayerId, n: usize, ok: impl Fn(PermId) -> bool) -> Option<Vec<PermId>> {
    let mut cs: Vec<PermId> = creatures_of(g, p).into_iter().filter(|&m| !g.perm(m).tapped && ok(m)).collect();
    if cs.len() < n {
        return None;
    }
    let key = |m: PermId| (dork_mana(g, p, m) as f64) * 2.0 + pval(g, m) - if g.perm(m).sick { 1.0 } else { 0.0 };
    cs.sort_by(|&a, &b| key(a).total_cmp(&key(b)));
    cs.truncate(n);
    Some(cs)
}

fn tap_all(g: &mut Game, ms: &[PermId]) -> Res {
    for &m in ms {
        g.perm_mut(m).tapped = true;
        crate::impls::t3_marwyn::on_tapped(g, m)?; // Dionus, Legolas
    }
    Ok(())
}

/// p's turn, in a main phase
fn my_main(g: &Game, p: PlayerId) -> bool {
    g.active == Some(p) && matches!(g.step, Step::Start | Step::Main1 | Step::Main2)
}

/// p's most common creature type among its creatures and the count
fn main_type(g: &Game, p: PlayerId) -> Option<(Sym, i32)> {
    let mut counts: Vec<(Sym, i32)> = vec![];
    for m in creatures_of(g, p) {
        let mut seen: Vec<Sym> = vec![];
        for t in crate::engine::values::subtypes(g, m) {
            if seen.contains(&t) {
                continue;
            }
            seen.push(t);
            match counts.iter_mut().find(|x| x.0 == t) {
                Some(x) => x.1 += 1,
                None => counts.push((t, 1)),
            }
        }
    }
    counts.into_iter().max_by_key(|x| x.1)
}

// ================================================================== blight (the ability language's "blight N")
/// q blights n: q puts n -1/-1 counters on a creature it controls, the one it loses least by (a creature that
/// survives them, else its least valuable). Called by the ability language ("each opponent blights 1": High Perfect
/// Morcant).
pub fn blight(g: &mut Game, q: PlayerId, n: i32) -> Res {
    if n <= 0 || !g.player(q).alive {
        return Ok(());
    }
    let cs = creatures_of(g, q);
    let loss = |m: PermId| if etgh(g, m) > n { 0.4 * n as f64 + 0.1 * pval(g, m) } else { pval(g, m) };
    let Some(m) = min_by(&cs, loss) else { return Ok(()) };
    if minus_counter(g, m, n) {
        crate::glog!(g, "    {} blights {n}: -1/-1 counters on {}", pname(g, q), g.perm(m).name);
    }
    check_state(g)
}

// ================================================================== Banner of Kinship
fn banner_type(g: &Game, m: PermId) -> Option<Sym> {
    match g.perm(m).data.get(DataKey::Ctype) {
        Some(Val::Str(s)) => Some(*s),
        _ => None,
    }
}

/// as it enters: name a creature type; a fellowship counter per creature of yours of that type
fn banner_as(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    if !named(g, m, BANNER) {
        return Ok(());
    }
    let t = crate::impls::galadriel::choose_type(g, p, BANNER)?;
    let n = creatures_of(g, p).into_iter().filter(|&x| x != m && has_type(g, x, t)).count() as i32;
    let x = g.perm_mut(m);
    x.data.set(DataKey::Ctype, Val::Str(t));
    x.plus = n;
    g.selfpt = true; // the bonus is read through `creature_pt`
    crate::glog!(g, "    Banner of Kinship names {t}: {n} fellowship counters");
    Ok(())
}

/// creatures you control of the chosen type get +1/+1 per fellowship counter
fn banner_pt(g: &Game, m: PermId) -> (i32, i32) {
    let p = owner(g, m);
    let mut k = 0;
    for &b in &g.player(p).perms {
        if live(g, b)
            && named(g, b, BANNER)
            && let Some(t) = banner_type(g, b)
            && has_type(g, m, t)
        {
            k += g.perm(b).plus.max(0);
        }
    }
    (k, k)
}

/// cast it with three or more creatures of the deck's main type
fn banner_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match main_type(g, p) {
        Some((_, n)) if n >= 3 => 50 + (2 * n).min(20),
        _ => 0,
    }
}

// ================================================================== Bounty of Skemfar
fn bounty_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).library.len() >= 6 { 52 } else { 0 }
}

fn bounty(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = 6.min(g.player(p).library.len());
    let mut top: Vec<CardId> = (0..n).filter_map(|_| g.player_mut(p).library.pop()).collect();
    crate::glog!(g, "    {} reveals the top {n} cards (Bounty of Skemfar)", pname(g, p));
    let lands: Vec<CardId> = top.iter().copied().filter(|&c| g.db.get(c).land).collect();
    let land = max_by(&lands, |c| g.db.get(c).tags.str(Tag::C).map_or(0, |s| s.chars().count()) as f64);
    if let Some(l) = land {
        top.retain(|&x| x != l);
    }
    let elves: Vec<CardId> = top.iter().copied().filter(|&c| elf_card(g, c) && !g.db.get(c).land).collect();
    let elf = max_by(&elves, |c| card_worth(g, p, c, false));
    if let Some(e) = elf {
        top.retain(|&x| x != e);
        g.player_mut(p).hand.push(e);
        crate::glog!(g, "    {} puts {} into their hand", pname(g, p), g.db.get(e).name);
    }
    g.rng.shuffle(&mut top);
    for c in top {
        g.player_mut(p).library.insert(0, c);
    }
    if let Some(l) = land {
        crate::glog!(g, "    {} enters tapped", g.db.get(l).name);
        g.add_land_entering(p, l, true);
        landfall(g, p)?;
    }
    Ok(intern("gy"))
}

// ================================================================== Crown of Skemfar
/// +1/+1 for each Elf you control
fn crown_bonus(g: &Game, p: PlayerId, _a: PermId, _m: PermId) -> (i32, i32) {
    let k = elves_of(g, p).len() as i32;
    (k, k)
}

static CROWN_SPEC: AuraSpec = AuraSpec { kws: &["reach"], bonus: Some(crown_bonus), ..AURA };

fn crown_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let n = elves_of(g, p).len() as i32;
    if creatures_of(g, p).is_empty() || n < 2 { 0 } else { 48 + (2 * n).min(16) }
}

/// {2}{G}: return it from the graveyard to the hand (at the end-of-turn window, or after combat with mana to recast it)
fn crown_gy(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let spare = total_mana(g, p, false) >= 7;
    if !(post.is_none() || (post == Some(true) && spare)) || !can_pay(g, p, 2, "G", false) {
        return Ok(vec![]);
    }
    if creatures_of(g, p).is_empty() {
        return Ok(vec![]);
    }
    let opt = Opt {
        utility: 1.2,
        label: "return Crown of Skemfar to hand".into(),
        act: Some(Action::Plan { f: crown_go, arg: c.0 as i64 }),
    };
    Ok(vec![opt])
}

fn crown_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 2, "G", false) {
        return Ok(false);
    }
    pay(g, p, 2, "G", false)?;
    let gy = &mut g.player_mut(p).gy;
    if let Some(i) = gy.iter().position(|&x| x == c) {
        gy.remove(i);
    }
    g.player_mut(p).hand.push(c);
    crate::glog!(g, "  {} returns Crown of Skemfar to their hand", pname(g, p));
    Ok(true)
}

// ================================================================== Pact of the Serpent
/// the target and creature type: an opponent the life loss kills, else yourself with your main type (X >= 3, life to
/// spare)
fn pact_plan(g: &Game, p: PlayerId) -> Option<(PlayerId, Sym, i32)> {
    for q in g.opps(p) {
        if crate::engine::values::player_hexproof(g, q) {
            continue;
        }
        if let Some((t, n)) = main_type(g, q)
            && n >= g.player(q).life
        {
            return Some((q, t, n));
        }
    }
    let (t, n) = main_type(g, p)?;
    let pl = g.player(p);
    (n >= 3 && pl.life - n >= 12 && pl.library.len() as i32 > n + 3).then_some((p, t, n.min(pl.life - 12)))
}

fn pact_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match pact_plan(g, p) {
        Some((q, _, _)) if q != p => 80,
        Some((_, _, n)) => 44 + (3 * n).min(30),
        None => 0,
    }
}

fn pact(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let (q, t) = if crate::human::is_human(g, p) {
        (p, crate::impls::galadriel::choose_type(g, p, PACT)?)
    } else {
        match pact_plan(g, p) {
            Some((q, t, _)) => (q, t),
            None => (p, crate::impls::galadriel::choose_type(g, p, PACT)?),
        }
    };
    let x = creatures_of(g, q).into_iter().filter(|&m| has_type(g, m, t)).count() as i32;
    crate::glog!(g, "    Pact of the Serpent names {t}: {} draws {x} and loses {x}", pname(g, q));
    if x > 0 {
        draw(g, q, x as u32, false)?;
        lose_life(g, q, x, Some(p), "drain", None)?;
        check_state(g)?;
    }
    Ok(intern("gy"))
}

// ================================================================== Champions of the Perfect (behold)
/// What a behold cost exiles: a card from the hand, or a permanent you control.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Behold {
    Hand(CardId),
    Perm(PermId),
}

/// "behold a(n) <type> and exile it" (Champions of the Perfect): the type it names, if this cost text is one
pub fn behold_type(text: &str) -> Option<&str> {
    let i = text.find("behold ")? + "behold ".len();
    let rest = &text[i..];
    let rest = rest.strip_prefix("an ").or_else(|| rest.strip_prefix("a "))?;
    let ty = rest.strip_suffix(" and exile it")?;
    (!ty.contains(' ')).then_some(ty)
}

/// the card or permanent p would behold and exile for spell c: an Elf card from the hand (it comes back, so it costs
/// half its worth) or an Elf p controls (a token is gone for good), the cheapest; never the commander
pub fn behold_pick(g: &Game, p: PlayerId, c: CardId, ty: &str) -> Option<Behold> {
    let mut best: Option<(f64, Behold)> = None;
    let mut consider = |v: f64, b: Behold| {
        if best.is_none_or(|x| v < x.0) {
            best = Some((v, b));
        }
    };
    for &h in &g.player(p).hand {
        let d = g.db.get(h);
        if h != c && (d.has_subtype(ty) || d.has_kw("changeling")) {
            consider(0.5 * card_worth(g, p, h, false), Behold::Hand(h));
        }
    }
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        if live(g, m) && !x.is_cmd && has_type(g, m, ty) {
            consider(pval(g, m) * if x.token { 1.0 } else { 0.8 }, Behold::Perm(m));
        }
    }
    best.map(|x| x.1)
}

/// pay it: exile the card (remembered with the spell, so it returns when the permanent leaves)
pub fn behold_pay(g: &mut Game, p: PlayerId, c: CardId, b: Behold) -> Res {
    match b {
        Behold::Hand(h) => {
            let hand = &mut g.player_mut(p).hand;
            if let Some(i) = hand.iter().position(|&x| x == h) {
                hand.remove(i);
            }
            g.player_mut(p).exile.push(h);
            g.beheld.push((c, h));
            crate::glog!(g, "    {} beholds and exiles {} from their hand", pname(g, p), g.db.get(h).name);
        }
        Behold::Perm(m) => {
            let (cd, token) = (g.perm(m).cd, g.perm(m).token);
            crate::glog!(g, "    {} beholds and exiles {}", pname(g, p), g.perm(m).name);
            exile_perm(g, m)?;
            if let (Some(cd), false) = (cd, token) {
                g.beheld.push((c, cd));
            }
        }
    }
    Ok(())
}

/// when it leaves the battlefield, the exiled card returns to its owner's hand
fn champions_leaves(g: &mut Game, src: Src, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let Some(cd) = g.perm(src).cd else { return Ok(()) };
    let Some(i) = g.beheld.iter().position(|x| x.0 == cd) else { return Ok(()) };
    let (_, card) = g.beheld.remove(i);
    for q in g.players.iter().map(|x| x.id).collect::<Vec<_>>() {
        let ex = &mut g.player_mut(q).exile;
        if let Some(j) = ex.iter().position(|&x| x == card) {
            ex.remove(j);
            g.player_mut(q).hand.push(card);
            crate::glog!(g, "    {} returns to {}'s hand (Champions of the Perfect)", g.db.get(card).name, pname(g, q));
            break;
        }
    }
    Ok(())
}

// ================================================================== High Perfect Morcant
/// what one proliferate is worth to p: its permanents' +1/+1, loyalty and fellowship counters, the opponents' -1/-1
fn prolif_value(g: &Game, p: PlayerId) -> f64 {
    let mut v = 0.0;
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        if !live(g, m) {
            continue;
        }
        if x.plus > 0 {
            v += if named(g, m, BANNER) {
                0.6 * creatures_of(g, p)
                    .iter()
                    .filter(|&&c| banner_type(g, m).is_some_and(|t| has_type(g, c, t)))
                    .count() as f64
            } else {
                0.4
            };
        }
        if x.loyalty.is_some() {
            v += 0.8;
        }
    }
    for q in g.opps(p) {
        for &m in &g.player(q).perms {
            if live(g, m) && g.is_creature(m) && g.perm(m).plus < 0 {
                v += if etgh(g, m) <= 1 { pval(g, m) } else { 0.4 };
            }
        }
    }
    v
}

/// tap three untapped Elves: proliferate (sorcery speed; after combat, when the counters are worth it)
fn morcant_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || !live(g, src) || post != Some(true) || !my_main(g, p) || !g.stack.is_empty() {
        return Ok(vec![]);
    }
    let st = g.turn_stamp();
    if g.perm(src).data.get(DataKey::Act) == Some(&Val::Stamp(st)) {
        return Ok(vec![]); // once a turn is enough for the AI
    }
    if tappers(g, p, 3, |m| has_type(g, m, "elf")).is_none() {
        return Ok(vec![]);
    }
    let v = prolif_value(g, p);
    if v < 1.0 {
        return Ok(vec![]);
    }
    Ok(vec![ability(v.min(6.0), "High Perfect Morcant: tap three Elves, proliferate".into(), src, morcant_go, 0)])
}

fn morcant_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some(ts) = tappers(g, p, 3, |m| has_type(g, m, "elf")) else { return Ok(false) };
    tap_all(g, &ts)?;
    let st = g.turn_stamp();
    g.perm_mut(src).data.set(DataKey::Act, Val::Stamp(st));
    crate::glog!(g, "  {} taps three Elves: proliferate (High Perfect Morcant)", pname(g, p));
    if ability_window(g, p, Some(src), "proliferate", None, None)? {
        crate::impls::common::proliferate(g, p, 1)?;
        check_state(g)?;
    }
    Ok(true)
}

// ================================================================== Miara, Thorn of the Glade
fn miara_draw(g: &mut Game, src: Src, me: PlayerId) -> Res {
    if g.player(me).life <= 6 || !can_pay(g, me, 1, "", false) || g.player(me).library.is_empty() {
        return Ok(());
    }
    if trigger_window(g, me, Some(src), "pay {1} and 1 life: draw a card", None)? && can_pay(g, me, 1, "", false) {
        pay(g, me, 1, "", false)?;
        lose_life(g, me, 1, Some(me), "other", None)?;
        draw(g, me, 1, false)?;
    }
    Ok(())
}

/// another Elf of yours dies
fn miara_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let me = owner(g, src);
    if m == src || owner(g, m) != me || !g.is_creature(m) || !has_type(g, m, "elf") {
        return Ok(());
    }
    miara_draw(g, src, me)
}

/// Miara itself dies
fn miara_self(g: &mut Game, src: Src, _m: PermId, _cause: Sym) -> Res {
    let me = owner(g, src);
    miara_draw(g, src, me)
}

// ================================================================== Nullmage Shepherd
fn shepherd_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let ae = |m: PermId| {
        let t = crate::engine::values::ptypes(g, m);
        g.perm(m).cd.is_some_and(|c| !g.db.get(c).land) && (t.has(Types::ARTIFACT) || t.has(Types::ENCHANTMENT))
    };
    crate::impls::common::best_opp_nonland(g, p, ae)
}

/// tap four untapped creatures: destroy the best opposing artifact or enchantment (at the end of an opponent's turn,
/// when they untap next; at once for a valuable one)
fn shepherd_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || !live(g, src) {
        return Ok(vec![]);
    }
    let Some(t) = shepherd_target(g, p) else { return Ok(vec![]) };
    let v = pval(g, t);
    if v < 2.5 || (post.is_some() && v < 5.0) || tappers(g, p, 4, |_| true).is_none() {
        return Ok(vec![]);
    }
    let label = format!("Nullmage Shepherd: tap four creatures, destroy {}", g.perm(t).name);
    Ok(vec![ability(v.min(7.0), label, src, shepherd_go, t.0 as i64)])
}

fn shepherd_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if !live(g, t) || untargetable(g, t) {
        return Ok(false);
    }
    let Some(ts) = tappers(g, p, 4, |_| true) else { return Ok(false) };
    tap_all(g, &ts)?;
    crate::glog!(g, "  {} taps four creatures: destroy {} (Nullmage Shepherd)", pname(g, p), g.perm(t).name);
    if ability_window(g, p, Some(src), "destroy target artifact or enchantment", None, Some(t))? && live(g, t) {
        crate::engine::removal::apply_removal(g, Some(p), t, "destroy", None)?;
    }
    Ok(true)
}

// ================================================================== Return Upon the Tide
fn tide(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if crate::impls::t45_winota::jailed(g) {
        return Ok(intern("gy")); // Soulless Jailer
    }
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    let pick = cs.iter().copied().max_by(|&a, &b| {
        let (x, y) = (g.db.get(a), g.db.get(b));
        (x.bomb, x.pow, x.cmc).cmp(&(y.bomb, y.pow, y.cmc))
    });
    let Some(c) = pick else { return Ok(intern("gy")) };
    let gy = &mut g.player_mut(p).gy;
    if let Some(i) = gy.iter().position(|&x| x == c) {
        gy.remove(i);
    }
    let elf = elf_card(g, c);
    crate::glog!(g, "    {} returns to the battlefield (Return Upon the Tide)", g.db.get(c).name);
    enter(g, p, c, Enter::default())?;
    if elf && !g.over {
        elf_tokens(g, p, 2)?;
    }
    Ok(intern("gy"))
}

// ================================================================== Rhys the Exiled
/// {B}, sacrifice another Elf (the least valuable, worth less than Rhys): regenerate
fn rhys_regen(g: &mut Game, m: PermId) -> Res<bool> {
    let p = owner(g, m);
    if !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    let fod: Vec<PermId> =
        elves_of(g, p).into_iter().filter(|&x| x != m && !no_sac(g, x) && !g.perm(x).is_cmd).collect();
    let Some(f) = min_by(&fod, |x| sac_worth(g, x)) else { return Ok(false) };
    if pval(g, f) >= pval(g, m) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    crate::glog!(g, "    {} pays {{B}} and sacrifices {} to regenerate Rhys", pname(g, p), g.perm(f).name);
    die(g, f, "sac")?;
    if !ability_window(g, p, Some(m), "regenerate Rhys the Exiled", Some(3.0), Some(m))? || !g.perm(m).on_bf {
        return Ok(false);
    }
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "    Rhys the Exiled regenerates");
    Ok(true)
}

// ================================================================== Ruthless Winnower
/// each player's upkeep: that player sacrifices a non-Elf creature (their least valuable)
fn winnower_upkeep(g: &mut Game, src: Src, q: PlayerId) -> Res {
    let me = owner(g, src);
    let cands: Vec<PermId> =
        creatures_of(g, q).into_iter().filter(|&m| !has_type(g, m, "elf") && !no_sac(g, m)).collect();
    if cands.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, me, Some(src), "that player sacrifices a non-Elf creature", None)? {
        return Ok(());
    }
    let cands: Vec<PermId> = cands.into_iter().filter(|&m| live(g, m)).collect();
    let pick = if crate::human::is_human(g, q) && !cands.is_empty() {
        let labels: Vec<String> = cands.iter().map(|&m| g.perm(m).name.to_string()).collect();
        let k =
            crate::human::choices::choose_one(g, q, "sacrifice", "Ruthless Winnower: sacrifice a non-Elf", &labels)?;
        cands.get(k).copied()
    } else {
        min_by(&cands, |m| sac_worth(g, m))
    };
    if let Some(m) = pick {
        crate::glog!(g, "    {} sacrifices {} (Ruthless Winnower)", pname(g, q), g.perm(m).name);
        die(g, m, "sac")?;
    }
    Ok(())
}

// ================================================================== Tyvar Kell
/// Elves you control have "{T}: Add {B}": each untapped Elf without a mana ability of its own (a mana Elf taps for its
/// own), not summoning sick (unless creature abilities have haste), and, before combat on your turn, only one that
/// doesn't attack
fn tyvar_mana(g: &Game, src: Src, p: PlayerId, u: &[Unit]) -> Vec<Unit> {
    if owner(g, src) != p || !live(g, src) {
        return vec![];
    }
    let precombat = g.active == Some(p) && matches!(g.step, Step::Start | Step::Main1);
    let ahaste = crate::impls::t6_legends1b::ability_haste(g, p);
    let mut out = vec![];
    for m in elves_of(g, p) {
        let x = g.perm(m);
        if x.tapped || (x.sick && !ahaste) || (precombat && !x.noatk) {
            continue;
        }
        if u.iter().any(|w| w.src == Source::Perm(m)) || (!g.auras.is_empty() && crate::cardcode::locked(g, m, "noact"))
        {
            continue;
        }
        out.push(Unit { src: Source::Perm(m), cols: Colors::from_letters("B"), amt: 1 });
    }
    out
}

/// +1: the Elf that gains most: a tapped mana Elf (it untaps for mana), else the strongest
fn tyvar_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let es = elves_of(g, p);
    let tapped: Vec<PermId> = es.iter().copied().filter(|&m| g.perm(m).tapped && dork_mana(g, p, m) > 0).collect();
    if let Some(m) = max_by(&tapped, |m| dork_mana(g, p, m) as f64) {
        return Some(m);
    }
    max_by(&es, |m| epow(g, m) as f64 + 0.1 * pval(g, m))
}

fn tyvar_plus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = tyvar_target(g, p)?;
    let untap = if g.perm(t).tapped { dork_mana(g, p, t) as f64 * 0.6 } else { 0.0 };
    Some(1.8 + untap)
}

fn tyvar_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let Some(t) = tyvar_target(g, p) else { return Ok(()) };
    crate::cardcode::add_counters(g, t, 1);
    let x = g.perm_mut(t);
    x.tapped = false;
    if !x.eot_kw.contains(&"deathtouch") {
        x.eot_kw.push("deathtouch");
    }
    crate::glog!(g, "    a +1/+1 counter on {}, untapped, deathtouch until end of turn", g.perm(t).name);
    Ok(())
}

fn tyvar_zero(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    elf_tokens(g, p, 1)?;
    Ok(())
}

fn tyvar_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    g.player_mut(p).emblems.push(TYVAR_EMBLEM);
    crate::glog!(g, "    {} gets Tyvar Kell's emblem", pname(g, p));
    Ok(())
}

static TYVAR_ABS: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "+1/+1 counter on an Elf, untap it", val: tyvar_plus_val, eff: tyvar_plus },
    WalkerAb { delta: 0, label: "1/1 Elf Warrior", val: |_, _, _| Some(1.6), eff: tyvar_zero },
    WalkerAb { delta: -6, label: "the emblem", val: |_, _, _| Some(9.0), eff: tyvar_ult },
];

/// the emblem (rules.emblem_cast): whenever you cast an Elf spell, it gains haste until end of turn and you draw two
pub fn tyvar_emblem_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let k = g.player(p).emblems.iter().filter(|&&e| e == TYVAR_EMBLEM).count();
    if k == 0 || !elf_card(g, c) {
        return Ok(());
    }
    let st = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(intern(&format!("tyvar_haste{}", c.0)), st);
    for _ in 0..k {
        crate::glog!(g, "    Tyvar Kell's emblem: {} draws two", pname(g, p));
        draw(g, p, 2, false)?;
    }
    Ok(())
}

/// an Elf spell cast this turn under Tyvar Kell's emblem has haste (engine/combat.rs `has_haste`)
pub fn emblem_haste(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    let pl = g.player(x.owner);
    if x.token || !pl.emblems.contains(&TYVAR_EMBLEM) {
        return false;
    }
    let Some(c) = x.cd else { return false };
    pl.flag_turn.get(intern(&format!("tyvar_haste{}", c.0))) == Some(&g.turn_stamp())
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let has = |n: &str| db.id(n).is_some();
    if has(BANNER) {
        let c = r.card(db, BANNER)?;
        c.as_enters = Some(banner_as);
        c.prio = Some(banner_prio);
        r.creature_pt.push(banner_pt);
    }
    if has(BOUNTY) {
        let c = r.card(db, BOUNTY)?;
        c.resolve = Some(bounty);
        c.prio = Some(bounty_prio);
    }
    if has(CROWN) {
        aura(r, db, CROWN, &CROWN_SPEC)?;
        let c = r.card(db, CROWN)?;
        c.gy_options = Some(crown_gy);
        c.prio = Some(crown_prio);
    }
    if has(PACT) {
        let c = r.card(db, PACT)?;
        c.resolve = Some(pact);
        c.prio = Some(pact_prio);
    }
    if has(CHAMPIONS) {
        r.card(db, CHAMPIONS)?.leaves = Some(champions_leaves);
    }
    if has(CHORUS) {
        r.card(db, CHORUS)?.options = Some(crate::impls::t3_marwyn::top_options);
    }
    if has(MORCANT) {
        r.card(db, MORCANT)?.options = Some(morcant_options);
    }
    if has(MIARA) {
        let c = r.card(db, MIARA)?;
        c.dies = Some(miara_dies);
        c.self_dies = Some(miara_self);
    }
    if has(SHEPHERD) {
        r.card(db, SHEPHERD)?.options = Some(shepherd_options);
    }
    if has(TIDE) {
        r.card(db, TIDE)?.resolve = Some(tide);
    }
    if has(RHYS) {
        r.card(db, RHYS)?.self_regen = Some(rhys_regen);
    }
    if has(WINNOWER) {
        r.card(db, WINNOWER)?.upkeep = Some(winnower_upkeep);
    }
    if has(TYVAR) {
        walker(r, db, TYVAR, &TYVAR_ABS)?;
        r.card(db, TYVAR)?.extra_mana = Some(tyvar_mana);
    }
    Ok(())
}
