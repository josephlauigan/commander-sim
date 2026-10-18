//! Tier 6 batch `dack`: Dack Fayden's white stax pile (rust/agents/t6-batches.md, audit/t6/dack-fayden-white-stax).
//! Cards only: no engine file is touched. What needs the engine is listed in rust/agents/engine-needs-dack.md.
//!
//! - Fiend Hunter, All-Fates Stalker: banishers built on `common::oring_exile` / `oring_return`.
//! - Touch the Spirit Realm: the same, plus Channel (`touch_channel`).
//! - Radiant Solar: venture (Lost Mine of Phandelver only) and its hand ability; Dack marks donated creatures.
//! - Preston, the Vanisher: Illusion token copies of creatures that entered without being cast; the five-Illusion exile.
//! - Dack Fayden, Helping Hand: the flip, shuffle, donation and goad.
//!
//!
//! The banishers' enter hooks read "you" from the enter call's `p` (the player the creature entered under), not from
//! the permanent's current controller: Dack puts creatures onto the battlefield under its controller and then
//! donates them, so by the time the queued enter trigger runs the permanent belongs to an opponent but the trigger is
//! still its original controller's (CR 603.3a).

use crate::cards::{CardDb, Colors, Types};
use crate::engine::life::{gain, lose_life};
use crate::engine::cast::cast_card;
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::stack::trigger_window;
use crate::engine::removal::apply_removal;
use crate::engine::tutors::shuffle_library;
use crate::engine::values::{pval, threat, untargetable};
use crate::engine::zones::{Enter, Tokens, die, enter, discard_cards, draw, enter_token_copy, exile_perm, make_tokens, max_by};
use crate::sym::intern;
use crate::impls::marchesa::steal;
use crate::impls::t2::flicker_worth;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{oring_exile, oring_return};
use crate::state::{Ctx, DataKey, Game, Val};


// ================================================================== Fiend Hunter
/// Full: "you may exile another target creature"; the exiled card returns under its owner's control when Fiend
/// Hunter leaves (a leaves trigger). The AI takes the best opposing creature worth taking (pval >= 1).
fn hunter_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "exile another target creature", Some(5.0))? && g.perm(src).on_bf {
        oring_exile(g, src, p, |g, x| x != src && g.is_creature(x), false, true)?;
    }
    Ok(())
}

fn hunter_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    oring_return(g, src)
}

// ================================================================== All-Fates Stalker
fn pname(g: &Game, p: PlayerId) -> crate::sym::Sym {
    g.player(p).name
}

fn has_sub(g: &Game, m: PermId, s: &str) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).has_subtype(s))
}

/// the ETB: exile up to one target non-Assassin creature until Stalker leaves the battlefield
fn stalker_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src
        && trigger_window(g, p, Some(src), "exile a non-Assassin creature", Some(5.0))?
        && g.perm(src).on_bf
    {
        oring_exile(g, src, p, |g, x| !has_sub(g, x, "assassin"), false, true)?;
    }
    Ok(())
}

/// "until this creature leaves": an effect ending, so the card returns at once
fn stalker_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    oring_return(g, src)
}

/// Warp {1}{W}: cast from hand for {1}{W}; exile it at the beginning of the next end step (its exiled creature comes
/// back); the card may be cast from exile on a later turn. Not offered to the AI: casting from exile needs an
/// exile-zone options hook the engine lacks (engine-needs-dack.md), so a warped Stalker would be stranded.
const WARP_RECAST_SUPPORTED: bool = false;

fn stalker_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !WARP_RECAST_SUPPORTED || post != Some(false) || !can_pay(g, p, 1, "W", false) {
        return Ok(vec![]);
    }
    // warp pays when the full cost can't be paid this turn
    let (cg, cp) = cost_of(g, p, c);
    if can_pay(g, p, cg, &cp, false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.2,
        label: "All-Fates Stalker (warp)".into(),
        act: Some(Action::Plan { f: stalker_warp, arg: c.0 as i64 }),
    }])
}

fn stalker_warp(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "W", false) {
        return Ok(false);
    }
    pay(g, p, 1, "W", false)?;
    cast_card(g, p, c, "hand", Ctx::default())?;
    let found = g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(c));
    if let Some(m) = found {
        g.perm_mut(m).data.set(DataKey::Warp, Val::Bool(true));
    }
    Ok(true)
}

/// a warped Stalker is exiled at the beginning of the next end step
fn stalker_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    if !g.perm(src).on_bf || !g.perm(src).data.truthy(DataKey::Warp) {
        return Ok(());
    }
    g.perm_mut(src).data.remove(DataKey::Warp);
    crate::glog!(g, "    All-Fates Stalker (warp) is exiled");
    exile_perm(g, src)
}

// ================================================================== Touch the Spirit Realm
fn is_art_or_creature(g: &Game, m: PermId) -> bool {
    g.is_creature(m) || g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT))
}

/// Full (both halves): the enchantment's ETB exiles up to one target artifact or creature until it leaves (the AI
/// takes the best opposing one); Channel is `touch_channel`.
fn touch_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "exile an artifact or creature", Some(5.0))? && g.perm(src).on_bf {
        oring_exile(g, src, p, is_art_or_creature, false, false)?;
    }
    Ok(())
}

fn touch_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    oring_return(g, src)
}

fn touch_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let best = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| !g.perm(m).phased && !untargetable(g, m) && is_art_or_creature(g, m))
        .map(|m| pval(g, m))
        .fold(0.0, f64::max);
    if best >= 2.0 { 45 + 30.min((6.0 * best) as i32) } else { 0 }
}

/// Channel -- {1}{W}, discard Touch the Spirit Realm: exile target artifact or creature; it returns to the
/// battlefield under its owner's control at the beginning of the next end step (the engine's `eot_returns` list,
/// which Eerie Interlude uses). A token is gone for good. Returns false when it can't be done.
pub fn touch_channel(g: &mut Game, p: PlayerId, c: CardId, target: PermId) -> Res<bool> {
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "W", false) {
        return Ok(false);
    }
    let x = g.perm(target);
    if !x.on_bf || x.phased || !is_art_or_creature(g, target) || untargetable(g, target) {
        return Ok(false);
    }
    pay(g, p, 1, "W", false)?;
    discard_cards(g, p, &[c])?;
    crate::glog!(g, "    {} channels Touch the Spirit Realm on {}", g.player(p).name, g.perm(target).name);
    let (cd, orig, tok) = (g.perm(target).phys.or(g.perm(target).cd), g.perm(target).orig, g.perm(target).token);
    apply_removal(g, Some(p), target, "exile", Some(c))?;
    if !tok
        && !g.perm(target).on_bf
        && let Some(cd) = cd
        && !g.player(orig).is_commander(cd)
        && g.player(orig).exile.contains(&cd)
    {
        g.eot_returns.push((orig, cd));
    }
    Ok(true)
}

fn touch_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, m) = (CardId((arg >> 32) as u16), PermId((arg & 0xffff_ffff) as u32));
    touch_channel(g, p, c, m)
}

/// the AI channels it on its own turn to flicker its best enters-the-battlefield creature (it comes back at the end
/// step, summoning sick but untapped), or to clear a good opposing blocker for the turn
fn touch_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !can_pay(g, p, 1, "W", false) {
        return Ok(vec![]);
    }
    let arg = |m: PermId| ((c.0 as i64) << 32) | m.0 as i64;
    let mut out = vec![];
    let mine: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && x.owner == p && x.orig == p && !x.token && g.is_creature(m)
        })
        .collect();
    if let Some(m) = max_by(&mine, |m| flicker_worth(g, p, m))
        && flicker_worth(g, p, m) >= 3.0
    {
        out.push(Opt {
            utility: flicker_worth(g, p, m).min(6.0),
            label: format!("Touch the Spirit Realm (channel) on {}", g.perm(m).name),
            act: Some(Action::Plan { f: touch_go, arg: arg(m) }),
        });
    }
    Ok(out)
}

// ================================================================== donation bookkeeping
/// Dack's ETB donates creatures after they entered under its controller, but their queued enter triggers (and the
/// triggers of the other pile creatures that see them) belong to the player who controlled them when they triggered
/// (CR 603.3a). `donated` marks the permanent with (the giver, `enter_no` at the donation); a creature `m` that
/// entered no later than the donation entered while the giver still controlled `src`.
pub fn mark_donated(g: &mut Game, m: PermId, giver: PlayerId) {
    let n = g.enter_no as i64;
    g.perm_mut(m).data.set(DataKey::Flip, Val::List(vec![Val::Player(giver), Val::Int(n)]));
}

/// was `src` under p's control when `m` entered under p? (its controller now, or its controller before Dack gave it
/// away in the same resolution)
pub fn controlled_when(g: &Game, src: PermId, p: PlayerId, m: PermId) -> bool {
    let x = g.perm(src);
    if x.owner == p {
        return true;
    }
    matches!(x.data.get(DataKey::Flip), Some(Val::List(v))
        if matches!((v.first(), v.get(1)), (Some(Val::Player(q)), Some(Val::Int(n)))
            if *q == p && i64::from(g.perm(m).born) <= *n))
}

// ================================================================== Radiant Solar: the dungeon
/// Venture into the dungeon. Only Lost Mine of Phandelver is ever chosen (the best of the three for this deck: its
/// third room drains each opponent for 1): Cave Entrance (scry 1), Goblin Lair (a 1/1 Goblin), Dark Pool (each
/// opponent loses 1 life and you gain 1), Temple of Dumathoin (draw a card); after the last room the dungeon is
/// completed and the next venture enters a new Lost Mine. The room is kept per player in `Player::grudge` under the
/// key "dungeon_room" (that map is only read by the defender choice, by player key); completions under
/// "dungeons_completed". (A real dungeon system is an engine need: engine-needs-dack.md.)
pub fn venture(g: &mut Game, p: PlayerId) -> Res {
    let room_key = intern("dungeon_room");
    let room = g.player(p).grudge.get(&room_key).copied().unwrap_or(0.0) as u32 + 1;
    g.player_mut(p).grudge.insert(room_key, room as f64);
    match room {
        1 => {
            crate::glog!(g, "    {} ventures into Lost Mine of Phandelver: Cave Entrance (scry 1)", pname(g, p));
            crate::cardcode::scry(g, p, 1, false)?;
        }
        2 => {
            crate::glog!(g, "    {} ventures: Goblin Lair (a 1/1 Goblin)", pname(g, p));
            let spec = Tokens { color: Some(Colors::from_letters("R")), types: vec![intern("goblin")], ..Tokens::new(1, 1) };
            make_tokens(g, p, spec)?;
        }
        3 => {
            crate::glog!(g, "    {} ventures: Dark Pool (each opponent loses 1, {} gains 1)", pname(g, p), pname(g, p));
            for q in g.opps(p).filter(|&q| g.player(q).alive).collect::<Vec<_>>() {
                lose_life(g, q, 1, Some(p), intern("drain"), None)?;
            }
            gain(g, p, 1)?;
        }
        _ => {
            crate::glog!(g, "    {} ventures: Temple of Dumathoin (draw a card); the dungeon is completed", pname(g, p));
            g.player_mut(p).grudge.insert(room_key, 0.0);
            let k = intern("dungeons_completed");
            *g.player_mut(p).grudge.entry(k).or_insert(0.0) += 1.0;
            draw(g, p, 1, false)?;
        }
    }
    Ok(())
}

// ================================================================== Radiant Solar
/// Full: whenever it or another nontoken creature you control enters, venture into the dungeon (including creatures
/// that entered with it under Dack's control before being donated); {W}, discard it: venture and gain 3 life
/// (`solar_hand`, the AI uses it with spare {W} when it can't cast the Angel this turn).
fn solar_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let counts = m == src || (!g.perm(m).token && g.is_creature(m));
    if counts
        && controlled_when(g, src, p, m)
        && trigger_window(g, p, Some(src), "venture into the dungeon", Some(3.0))?
    {
        venture(g, p)?;
    }
    Ok(())
}

/// {W}, discard Radiant Solar: venture into the dungeon and you gain 3 life
pub fn solar_discard(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "W", false) {
        return Ok(false);
    }
    pay(g, p, 0, "W", false)?;
    discard_cards(g, p, &[c])?;
    venture(g, p)?;
    gain(g, p, 3)?;
    Ok(true)
}

fn solar_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    solar_discard(g, p, CardId(arg as u16))
}

fn solar_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(true) || g.active != Some(p) || !can_pay(g, p, 0, "W", false) {
        return Ok(vec![]);
    }
    // main phase 2 with {W} left over and the Angel not castable: cycle it into a room and 3 life
    let (cg, cp) = cost_of(g, p, c);
    if can_pay(g, p, cg + 1, &format!("{cp}W"), false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.5,
        label: "Radiant Solar (discard: venture, gain 3)".into(),
        act: Some(Action::Plan { f: solar_go, arg: c.0 as i64 }),
    }])
}

// ================================================================== Preston, the Vanisher
/// the creature type Illusion on a creature of p's (a token copy made by Preston, or any Illusion)
fn illusions(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && !g.perm(m).phased && crate::engine::values::subtypes(g, m).contains(&"illusion"))
        .collect()
}

/// Whenever another nontoken creature you control enters, if it wasn't cast, create a token that's a copy of that
/// creature, except it's a 0/1 white Illusion. The copy's own enters trigger fires (it is a copy); Preston doesn't
/// trigger for tokens. Preston counts creatures that entered under its controller in the same resolution it was
/// donated in (`controlled_when`). The token keeps the copied card's creature types as well (engine need: copy
/// exceptions that replace types).
fn preston_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let copy_born = g.player(p).grudge.get(&intern("preston_copy_born")).copied().unwrap_or(0.0);
    if m == src
        || g.perm(m).token
        || f64::from(g.perm(m).born) == copy_born
        || !g.is_creature(m)
        || g.last_cast_etb
        || !controlled_when(g, src, p, m)
    {
        return Ok(());
    }
    let Some(cd) = g.perm(m).cd else { return Ok(()) };
    let label = format!("create a token copy of {}", g.db.get(cd).name);
    if !trigger_window(g, p, Some(src), &label, Some(4.0))? {
        return Ok(());
    }
    // the copy is not marked as a token until it has entered: remember its entry number so it never triggers Preston
    let born_key = intern("preston_copy_born");
    let next = g.enter_no as f64 + 1.0;
    g.player_mut(p).grudge.insert(born_key, next);
    let made = enter_token_copy(g, p, cd);
    g.player_mut(p).grudge.insert(born_key, 0.0);
    if let Some(t) = made? {
        let x = g.perm_mut(t);
        x.pow = 0;
        x.tgh = 1;
        x.plus = 0;
        x.colors = Colors::from_letters("W");
        if !x.ttypes.contains(&"illusion") {
            x.ttypes.push(intern("illusion"));
        }
        g.bf_ver += 1;
        crate::glog!(g, "    Preston makes a 0/1 Illusion copy of {}", g.db.get(cd).name);
    }
    Ok(())
}

/// {1}{W}, sacrifice five Illusions: exile target nonland permanent
pub fn preston_exile(g: &mut Game, src: PermId, p: PlayerId, target: PermId) -> Res<bool> {
    let tgt_ok = {
        let x = g.perm(target);
        x.on_bf && !x.phased && !untargetable(g, target) && !g.db.get(x.cd.unwrap_or(CardId(0))).land
    };
    let ills = illusions(g, p);
    if !g.perm(src).on_bf || ills.len() < 5 || !tgt_ok || !can_pay(g, p, 1, "W", false) {
        return Ok(false);
    }
    pay(g, p, 1, "W", false)?;
    // sacrifice the least valuable five
    let mut ills = ills;
    ills.sort_by(|&a, &b| pval(g, a).partial_cmp(&pval(g, b)).unwrap_or(std::cmp::Ordering::Equal));
    for &m in ills.iter().take(5) {
        die(g, m, "sac")?;
    }
    crate::glog!(g, "    Preston exiles {}", g.perm(target).name);
    if g.perm(target).on_bf {
        apply_removal(g, Some(p), target, "exile", g.perm(src).cd)?;
    }
    Ok(true)
}

fn preston_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    preston_exile(g, src, p, PermId(arg as u32))
}

fn preston_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || illusions(g, p).len() < 5 || !can_pay(g, p, 1, "W", false) {
        return Ok(vec![]);
    }
    let cands: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| {
            let x = g.perm(m);
            !x.phased && !untargetable(g, m) && x.cd.is_some_and(|c| !g.db.get(c).land)
        })
        .collect();
    let Some(t) = max_by(&cands, |m| pval(g, m)) else { return Ok(vec![]) };
    let v = pval(g, t);
    if v < 3.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: v.min(8.0),
        label: format!("Preston: sacrifice five Illusions, exile {}", g.perm(t).name),
        act: Some(Action::Ability { src, f: preston_go, arg: t.0 as i64 }),
    }])
}

// ================================================================== Dack Fayden, Helping Hand
/// When Dack Fayden enters, reveal cards from the top of your library until you reveal X creature cards, where X is
/// the number of opponents you have. Put those creature cards onto the battlefield, then shuffle. They're goaded for
/// the rest of the game. For each of those permanents, choose a different opponent. Each opponent gains control of
/// the permanent for which they were chosen.
///
/// The order follows the rules: every creature enters under Dack's controller (so their enters triggers, Preston's
/// and Radiant Solar's are queued as Dack's controller's), then each is given away (`mark_donated` records the giver
/// for the queued triggers), then the triggers resolve with the opponents' creatures as legal targets. The AI hands
/// the most valuable creature to the least threatening opponent. Goad lasts only until the goader's next turn here
/// (`DataKey::Goaded` has no "rest of the game": engine need).
fn dack_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || !trigger_window(g, p, Some(src), "reveal creatures and give them away", Some(6.0))? {
        return Ok(());
    }
    flip_and_donate(g, p)?;
    Ok(())
}

/// Dack's ability for p: returns the permanents it put onto the battlefield (now owned by the opponents they went to).
/// Triggers of the entering creatures wait (`g.resolving`, as for a resolving spell) until every creature has been
/// given away, then go on the stack.
pub fn flip_and_donate(g: &mut Game, p: PlayerId) -> Res<Vec<PermId>> {
    g.resolving += 1;
    let r = flip_and_donate_inner(g, p);
    g.resolving -= 1;
    let pile = r?;
    if g.resolving == 0 && !g.trig_queue.is_empty() {
        crate::engine::stack::flush_triggers(g)?;
    }
    Ok(pile)
}

fn flip_and_donate_inner(g: &mut Game, p: PlayerId) -> Res<Vec<PermId>> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let x = opps.len();
    // reveal from the top (the end of the vector) until X creature cards are revealed; the rest stay in the library
    let mut found: Vec<CardId> = vec![];
    let mut i = g.player(p).library.len();
    while i > 0 && found.len() < x {
        i -= 1;
        let c = g.player(p).library[i];
        if g.db.get(c).creature {
            found.push(c);
        }
    }
    for c in &found {
        let lib = &mut g.player_mut(p).library;
        if let Some(i) = lib.iter().position(|x| x == c) {
            lib.remove(i);
        }
    }
    crate::glog!(
        g,
        "    Dack reveals {} creature cards: {}",
        found.len(),
        found.iter().map(|&c| g.db.get(c).name.to_string()).collect::<Vec<_>>().join(", ")
    );
    let mut pile = vec![];
    for c in found {
        pile.push(enter(g, p, c, Enter::default())?);
    }
    shuffle_library(g, p);
    // the most valuable creature goes to the least threatening opponent; each opponent gets at most one
    let mut order: Vec<PermId> = pile.iter().copied().filter(|&m| g.perm(m).on_bf).collect();
    order.sort_by(|&a, &b| pval(g, b).partial_cmp(&pval(g, a)).unwrap_or(std::cmp::Ordering::Equal));
    let mut who = opps;
    who.sort_by(|&a, &b| threat(g, p, a).partial_cmp(&threat(g, p, b)).unwrap_or(std::cmp::Ordering::Equal));
    let turns = g.player(p).turns as i64;
    let mut given = vec![];
    for (m, q) in order.into_iter().zip(who) {
        mark_donated(g, m, p);
        steal(g, q, m, false);
        g.perm_mut(m).data.set(DataKey::Goaded, Val::List(vec![Val::Player(p), Val::Int(turns)]));
        given.push(m);
    }
    for &m in &pile {
        // a pile creature that left again (died to a state-based action) has nothing to goad
        if g.perm(m).on_bf && !given.contains(&m) {
            g.perm_mut(m).data.set(DataKey::Goaded, Val::List(vec![Val::Player(p), Val::Int(turns)]));
        }
    }
    Ok(pile)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Fiend Hunter")?;
    c.etb = Some(hunter_etb);
    c.leaves = Some(hunter_leaves);
    let c = r.card(db, "All-Fates Stalker")?;
    c.etb = Some(stalker_etb);
    c.leaves = Some(stalker_leaves);
    crate::impls::partials::at_once(c, crate::hooks::Event::Leaves, true); // an "until" duration, not a trigger
    c.end_step = Some(stalker_end);
    c.hand_options = Some(stalker_hand);
    let c = r.card(db, "Touch the Spirit Realm")?;
    c.etb = Some(touch_etb);
    c.leaves = Some(touch_leaves);
    crate::impls::partials::at_once(c, crate::hooks::Event::Leaves, true);
    c.prio = Some(touch_prio);
    c.hand_options = Some(touch_hand);
    r.card(db, "Dack Fayden, Helping Hand")?.etb = Some(dack_etb);
    let c = r.card(db, "Preston, the Vanisher")?;
    c.etb = Some(preston_etb);
    c.options = Some(preston_options);
    let c = r.card(db, "Radiant Solar")?;
    c.etb = Some(solar_etb);
    c.hand_options = Some(solar_hand);
    Ok(())
}
