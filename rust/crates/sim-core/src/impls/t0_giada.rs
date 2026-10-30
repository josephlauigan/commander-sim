//! Tier 0 pool deck card code: the cards of the giada precon (decklists/pool/t0-b2-precons/) that weren't modeled
//! (rust/agents/archive/t0-giada.md). The audit verdicts are in data/card_overrides.toml; each function's doc comment
//! says how the card is modeled and what is left out.
//!
//! Mechanics: Clues are `Player::clues` (a counter, cracked for {2}: `ai/act.rs`), made by `investigate`; the monarch is
//! `cardcode::become_monarch`. Triggered abilities use `trigger_window` after their conditions (so they go on the
//! stack); "enters with" replacements and statics run at once.

use super::common::proliferate;
use crate::cards::{CardDb, Colors};
use crate::engine::life::gain;
use crate::engine::mana::{can_pay, pay};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::trigger_window;
use crate::engine::tutors::shuffle_library;
use crate::engine::values::{colors_of, commander_out, etgh, has_type, pval};
use crate::engine::zones::{Enter, Tokens, draw, enter, make_artifact_tokens, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn is_angel(g: &Game, m: PermId) -> bool {
    has_type(g, m, "angel")
}

/// Angels (creatures) p controls
fn angels(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m) && is_angel(g, m)).collect()
}

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && &*g.perm(m).name == name).collect()
}

/// "investigate" n times: Clue tokens (Academy Manufactor and token doubling apply)
fn investigate(g: &mut Game, p: PlayerId, n: i32) -> Res {
    if n > 0 && g.player(p).alive {
        crate::glog!(
            g,
            "    {} investigates{}",
            g.player(p).name,
            if n > 1 { format!(" x{n}") } else { String::new() }
        );
        make_artifact_tokens(g, p, "Clue", n)?;
    }
    Ok(())
}

/// the registered "some hooks run at once" bit
fn at_once(c: &mut crate::hooks::CardImpl, e: Event) {
    c.at_once |= 1u128 << (e as u32);
}

fn white() -> Colors {
    Colors::from_letters("W")
}

fn angel_token(n: u32, pow: i32) -> Tokens {
    Tokens {
        fly: true,
        color: Some(white()),
        types: vec![intern("angel")],
        sick: true,
        tgh: Some(pow),
        ..Tokens::new(n, pow)
    }
}

fn spirit_token() -> Tokens {
    Tokens { fly: true, color: Some(white()), types: vec![intern("spirit")], tgh: Some(1), ..Tokens::new(1, 1) }
}

// ================================================================== engine entry points
/// Angel of Vitality: "If you would gain life, you gain that much life plus 1 instead" (engine/life.rs `gain`): the
/// extra life, one per Angel of Vitality p controls
pub fn gain_bonus(g: &Game, p: PlayerId, n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| live(g, m) && !g.perm(m).neutered && &*g.perm(m).name == "Angel of Vitality")
        .count() as i32
}

/// Herald of Eternal Dawn: "You can't lose the game and your opponents can't win the game" (engine/life.rs
/// `check_state`: the player is not eliminated; with a live opponent left, nobody else can win either)
pub fn cant_lose(g: &Game, p: PlayerId) -> bool {
    !named(g, p, "Herald of Eternal Dawn").is_empty() || crate::impls::karn_cards::angel_holds(g, p)
}

/// Serra Avenger: "You can't cast Serra Avenger during your first, second, or third turns of the game"
/// (engine/cast.rs `castable`)
pub fn cast_locked(g: &Game, p: PlayerId, c: CardId) -> bool {
    &*g.db.get(c).name == "Serra Avenger" && g.player(p).turns <= 3
}

/// Seraph of the Sword: "Prevent all combat damage that would be dealt to this creature" (engine/combat.rs)
pub fn combat_proof(g: &Game, m: PermId) -> bool {
    !g.unpreventable() && !g.perm(m).neutered && &*g.perm(m).name == "Seraph of the Sword"
}

/// Metropolis Reformer: "Whenever this creature is dealt damage, you gain that much life" (engine/combat.rs for
/// combat damage, engine/removal.rs for damage spells)
pub fn damaged(g: &mut Game, m: PermId, dmg: i32) -> Res {
    if dmg > 0 && live(g, m) {
        g.mark_damaged(m); // Rooftop Assassin: "dealt damage this turn"
    }
    if dmg > 0 && live(g, m) && !g.perm(m).neutered && &*g.perm(m).name == "Metropolis Reformer" {
        let p = owner(g, m);
        crate::glog!(g, "    Metropolis Reformer: {} gains {}", g.player(p).name, dmg);
        gain(g, p, dmg)?;
    }
    // --- Tiers 4/5 (miirym): Wrathful Red Dragon: a Dragon of its controller's was dealt damage
    if dmg > 0 && live(g, m) {
        crate::impls::t45_miirym::wrathful_damaged(g, m, dmg)?;
    }
    Ok(())
}

// ================================================================== Giada, Font of Hope
/// the Angels p controlled before the ones in `entering` came (Giada: "for each Angel you already control")
fn angels_before(g: &Game, p: PlayerId, entering: &[PermId]) -> i32 {
    angels(g, p).into_iter().filter(|m| !entering.contains(m)).count() as i32
}

/// **Giada, Font of Hope**: each other Angel you control enters with an additional +1/+1 counter for each Angel you
/// already control (a replacement: runs at once). Flying, vigilance and the Angel-only {W} are in the tags and
/// `engine/mana.rs`.
fn giada_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src && owner(g, m) == owner(g, src) && p == owner(g, src) && g.is_creature(m) && is_angel(g, m) {
        let n = angels_before(g, p, &[m]);
        if n > 0 {
            crate::cardcode::add_counters(g, m, n);
        }
    }
    Ok(())
}

fn giada_tokens(g: &mut Game, src: Src, p: PlayerId, toks: &[PermId]) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    let n = angels_before(g, p, toks);
    for &m in toks {
        if m != src && live(g, m) && is_angel(g, m) && n > 0 {
            crate::cardcode::add_counters(g, m, n);
        }
    }
    Ok(())
}

// ================================================================== Search the Premises
/// **Search the Premises**: whenever a creature attacks you, investigate (one Clue per attacking creature). The hook
/// sees the attack as a whole and its main defender, so attackers sent at another player in a split attack are
/// counted if the main defender is you; planeswalkers of ours are not defended by it (the engine sends attackers at
/// them separately).
fn premises_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let me = owner(g, src);
    if d != me || p == me {
        return Ok(vec![]);
    }
    let n = atk.iter().filter(|&&a| g.perm(a).on_bf && owner(g, a) == p).count() as i32;
    if n > 0 && trigger_window(g, me, Some(src), "investigate for each attacker", None)? {
        investigate(g, me, n)?;
    }
    Ok(vec![])
}

// ================================================================== Secret Rendezvous
/// **Secret Rendezvous**: you and the target opponent each draw three cards. The opponent is the one with the
/// weakest board (the least able to use the cards).
fn rendezvous(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let opps: Vec<PlayerId> = g.opps(p).filter(|&q| g.player(q).alive).collect();
    let q = opps.iter().copied().min_by_key(|&q| (g.player(q).perms.len(), g.player(q).hand.len()));
    draw(g, p, 3, false)?;
    if let Some(q) = q {
        draw(g, q, 3, false)?;
    }
    Ok("gy")
}

fn rendezvous_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let hand = g.player(p).hand.iter().filter(|&&x| x != c).count();
    if hand >= 5 {
        0
    } else if hand >= 3 {
        35
    } else {
        55
    }
}

// ================================================================== Angel of Vitality
/// **Angel of Vitality**: +2/+2 as long as you have 25 or more life (the lifegain bonus is `gain_bonus`)
fn vitality_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    if g.player(p).life >= 25 { (2, 2) } else { (0, 0) }
}

/// puts the card's own power/toughness rule on (and makes the card's code live)
fn selfpt_on(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

// ================================================================== Angel of the Ruins
/// **Angel of the Ruins**: when it enters, exile up to two target artifacts and/or enchantments (the AI takes the
/// opponents' two most valuable; never its own)
fn ruins_targets(g: &Game, p: PlayerId) -> Vec<PermId> {
    let mut c = legal_targets(g, p, "exile", "ae", false, None);
    c.retain(|&m| g.perm(m).cd.is_some() && !g.is_creature(m));
    c.sort_by(|&a, &b| pval(g, b).total_cmp(&pval(g, a)));
    c.truncate(2);
    c
}

fn ruins_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != owner(g, src) || ruins_targets(g, p).is_empty() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "exile up to two artifacts and/or enchantments", Some(5.0))? {
        return Ok(());
    }
    for t in ruins_targets(g, p) {
        if live(g, t) {
            apply_removal(g, Some(p), t, "exile", None)?;
        }
    }
    Ok(())
}

/// **Plainscycling {2}** ({2}, discard this card: search for a Plains, put it into your hand), at the end of the
/// turn before yours when the hand is short of lands and the Angel is far from castable
fn ruins_cycle(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    let lands_in_hand = g.player(p).hand.iter().filter(|&&x| g.db.get(x).land).count();
    let has_plains = g.player(p).library.iter().any(|&x| &*g.db.get(x).name == "Plains");
    if !has_plains || lands_in_hand >= 2 || crate::engine::mana::total_mana(g, p, false) >= 7 {
        return Ok(vec![]);
    }
    let u = if lands_in_hand == 0 { 1.4 } else { 0.9 };
    Ok(vec![Opt {
        utility: u,
        label: "plainscycle Angel of the Ruins".into(),
        act: Some(Action::Plan { f: ruins_cycle_go, arg: c.0 as i64 }),
    }])
}

fn ruins_cycle_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    crate::impls::partials::remove_card(&mut g.player_mut(p).hand, c);
    g.player_mut(p).gy.push(c);
    crate::glog!(g, "    {} plainscycles Angel of the Ruins", g.player(p).name);
    let lib = &g.player(p).library;
    if let Some(i) = lib.iter().rposition(|&x| &*g.db.get(x).name == "Plains") {
        let pl = g.player_mut(p);
        let land = pl.library.remove(i);
        pl.hand.push(land);
        pl.seen.insert(land);
    }
    shuffle_library(g, p);
    Ok(true)
}

// ================================================================== Angelic Field Marshal
/// **Angelic Field Marshal**: lieutenant, as long as you control your commander it gets +2/+2 and creatures you
/// control have vigilance
fn marshal_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    if commander_out(g, p) { (2, 2) } else { (0, 0) }
}

fn marshal_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "vigilance" && owner(g, m) == owner(g, src) && g.is_creature(m) && commander_out(g, owner(g, src))
}

// ================================================================== Angelic Sleuth
/// **Angelic Sleuth**: whenever another permanent you control leaves the battlefield, if it had counters on it,
/// investigate (+1/+1, -1/-1, loyalty and lore counters)
fn sleuth_leaves(g: &mut Game, src: Src, m: PermId) -> Res {
    let x = g.perm(m);
    let counters = x.plus != 0
        || x.loyalty.is_some_and(|l| l > 0)
        || x.data.get(crate::state::DataKey::Lore).is_some_and(|v| !matches!(v, crate::state::Val::None));
    if m == src || x.owner != owner(g, src) || !counters {
        return Ok(());
    }
    let me = owner(g, src);
    if trigger_window(g, me, Some(src), "investigate", None)? {
        investigate(g, me, 1)?;
    }
    Ok(())
}

/// the Angelic Sleuths that see permanent m leave (engine/zones.rs `leave`): m is another permanent of theirs with counters
pub fn sleuths_for(g: &Game, m: PermId) -> Vec<PermId> {
    let x = g.perm(m);
    let counters = x.plus != 0
        || x.loyalty.is_some_and(|l| l > 0)
        || x.data.get(crate::state::DataKey::Lore).is_some_and(|v| !matches!(v, crate::state::Val::None));
    if !counters || g.hooks.is_empty() {
        return vec![];
    }
    g.player(x.owner)
        .perms
        .iter()
        .copied()
        .filter(|&s| s != m && live(g, s) && &*g.perm(s).name == "Angelic Sleuth" && !g.perm(s).neutered)
        .collect()
}

// ================================================================== Court of Grace
/// **Court of Grace**: when it enters you become the monarch; at the beginning of your upkeep, a 1/1 white Spirit
/// with flying, or a 4/4 white Angel with flying instead if you're the monarch
fn court_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "become the monarch", Some(5.0))? {
        crate::cardcode::become_monarch(g, p)?;
    }
    Ok(())
}

fn court_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "create a Spirit, or an Angel if you're the monarch", Some(5.0))? {
        if g.monarch == Some(p) {
            make_tokens(g, p, angel_token(1, 4))?;
        } else {
            make_tokens(g, p, spirit_token())?;
        }
    }
    Ok(())
}

// ================================================================== Defy Death
fn defy_target(g: &Game, p: PlayerId) -> Option<CardId> {
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    max_by(&cs, |c| {
        let d = g.db.get(c);
        (d.bomb as f64) * 100.0 + d.pow as f64 * 10.0 + d.cmc as f64 + if d.has_subtype("angel") { 15.0 } else { 0.0 }
    })
}

/// **Defy Death**: return target creature card from your graveyard to the battlefield; if it's an Angel, put two
/// +1/+1 counters on it
fn defy_death(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = defy_target(g, p) {
        crate::impls::partials::remove_card(&mut g.player_mut(p).gy, t);
        let m = enter(g, p, t, Enter::default())?;
        crate::glog!(g, "    Defy Death returns {}", g.db.get(t).name);
        if live(g, m) && g.db.get(t).has_subtype("angel") {
            put_counters(g, m, 2)?;
        }
    }
    Ok("gy")
}

fn defy_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match defy_target(g, p) {
        Some(t) if g.db.get(t).bomb >= 3 || g.db.get(t).pow >= 4 => 62,
        Some(_) => 35,
        None => 0,
    }
}

/// put n +1/+1 counters on m; Exemplar of Light draws (once each turn) when they land on it
fn put_counters(g: &mut Game, m: PermId, n: i32) -> Res {
    crate::cardcode::add_counters(g, m, n);
    exemplar_counters(g, m)
}

// ================================================================== Emeria Shepherd
/// **Emeria Shepherd**: landfall, you may return target nonland permanent card from your graveyard to your hand; if
/// the land is a Plains, to the battlefield instead (the best card; the land is the one that just came in)
fn shepherd_landfall(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).perm && !g.db.get(c).land).collect();
    let Some(t) = max_by(&cs, |c| {
        let d = g.db.get(c);
        (d.bomb as f64) * 100.0 + d.pow as f64 * 10.0 + d.cmc as f64
    }) else {
        return Ok(());
    };
    if !trigger_window(g, p, Some(src), "return a nonland permanent card", Some(5.0))? {
        return Ok(());
    }
    let plains = g.player(p).lands.last().is_some_and(|&l| g.db.get(g.land(l).card()).has_subtype("plains"));
    // the card may have left in response
    let Some(i) = g.player(p).gy.iter().position(|&c| c == t) else { return Ok(()) };
    g.player_mut(p).gy.remove(i);
    if plains {
        crate::glog!(g, "    Emeria Shepherd returns {} to the battlefield", g.db.get(t).name);
        enter(g, p, t, Enter::default())?;
    } else {
        crate::glog!(g, "    Emeria Shepherd returns {} to hand", g.db.get(t).name);
        g.player_mut(p).hand.push(t);
    }
    Ok(())
}

// ================================================================== Exemplar of Light
/// counters went onto Exemplar of Light m: its controller draws a card, only once each turn
fn exemplar_counters(g: &mut Game, m: PermId) -> Res {
    if !live(g, m) || &*g.perm(m).name != "Exemplar of Light" {
        return Ok(());
    }
    let p = owner(g, m);
    let key = intern(&format!("exemplar_{}", m.0));
    if crate::engine::values::once_per_turn(g, p, key) {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// **Exemplar of Light**: whenever you gain life, put a +1/+1 counter on it; whenever you put one or more +1/+1
/// counters on it, draw a card (once each turn: here also after Defy Death and proliferate)
fn exemplar_gain(g: &mut Game, src: Src, p: PlayerId, n: i32) -> Res {
    if p != owner(g, src) || n <= 0 {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "put a +1/+1 counter on it", Some(4.0))? && live(g, src) {
        put_counters(g, src, 1)?;
    }
    Ok(())
}

// ================================================================== Fateful Absence
/// **Fateful Absence**: destroy target creature or planeswalker; its controller investigates (whether or not it died)
fn absence(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| live(g, t)) {
        let q = owner(g, t);
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
        investigate(g, q, 1)?;
    }
    Ok("gy")
}

// ================================================================== Firemane Commando
/// **Firemane Commando**: whenever you attack with two or more creatures, draw a card; whenever another player
/// attacks with two or more creatures, they draw a card if none of those creatures attacked you (the hook sees the
/// attack's main defender: a split attack that also sends creatures at you is treated by who the main defender is)
fn commando_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let me = owner(g, src);
    let n = atk.iter().filter(|&&a| g.perm(a).on_bf && owner(g, a) == p).count();
    if n < 2 {
        return Ok(vec![]);
    }
    if p == me {
        if trigger_window(g, me, Some(src), "draw a card", None)? {
            draw(g, me, 1, false)?;
        }
    } else if d != me && g.player(p).alive && trigger_window(g, me, Some(src), "they draw a card", None)? {
        draw(g, p, 1, false)?;
    }
    Ok(vec![])
}

// ================================================================== Herald of War
/// **Herald of War**: whenever it attacks, put a +1/+1 counter on it
fn herald_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p == owner(g, src)
        && atk.contains(&src)
        && trigger_window(g, p, Some(src), "+1/+1 counter", None)?
        && live(g, src)
    {
        put_counters(g, src, 1)?;
    }
    Ok(vec![])
}

/// Angel spells and Human spells you cast cost {1} less for each +1/+1 counter on it
fn herald_cost(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    if caster == owner(g, src) && (d.has_subtype("angel") || d.has_subtype("human")) {
        -g.perm(src).plus.max(0)
    } else {
        0
    }
}

// ================================================================== Heraldic Banner
/// **Heraldic Banner**: the colour chosen is white (the deck is all white), so your white creatures get +1/+0 for
/// each Banner; {T}: add {W} (tags)
fn banner_pt(g: &Game, m: PermId) -> (i32, i32) {
    let p = owner(g, m);
    if !g.is_creature(m) || !colors_of(g, m).has('W') {
        return (0, 0);
    }
    (named(g, p, "Heraldic Banner").len() as i32, 0)
}

// ================================================================== Merchant of Truth
/// **Merchant of Truth**: whenever a nontoken creature you control dies, investigate (itself included)
fn merchant_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let me = owner(g, src);
    if owner(g, m) != me || g.perm(m).token || !g.is_creature(m) {
        return Ok(());
    }
    if trigger_window(g, me, Some(src), "investigate", None)? {
        investigate(g, me, 1)?;
    }
    Ok(())
}

fn merchant_self_dies(g: &mut Game, src: Src, m: PermId, cause: Sym) -> Res {
    if m == src { merchant_dies(g, src, m, cause) } else { Ok(()) }
}

/// Clues you control have exalted: attacking alone, +1/+1 for each Clue
fn merchant_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let n = g.player(p).clues as i32;
    if p == owner(g, src) && atk.len() == 1 && n > 0 && g.perm(atk[0]).on_bf {
        let e = &mut g.perm_mut(atk[0]).eot_pt;
        *e = (e.0 + n, e.1 + n);
    }
    Ok(vec![])
}

// ================================================================== Metropolis Reformer
/// **Metropolis Reformer**: you have hexproof
fn reformer_hexproof(g: &Game, src: Src, p: PlayerId) -> bool {
    p == owner(g, src)
}

// ================================================================== Norn's Choirmaster
fn is_my_commander(g: &Game, p: PlayerId, m: PermId) -> bool {
    owner(g, m) == p
        && (g.perm(m).is_cmd || g.perm(m).cd.is_some_and(|c| !g.perm(m).token && g.player(p).is_commander(c)))
}

fn choir_proliferate(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if trigger_window(g, p, Some(src), "proliferate", None)? {
        proliferate(g, p, 1)?;
        for m in named(g, p, "Exemplar of Light") {
            if g.perm(m).plus > 0 {
                exemplar_counters(g, m)?; // proliferate put a counter on it
            }
        }
    }
    Ok(())
}

/// **Norn's Choirmaster**: whenever a commander you control enters or attacks, proliferate
fn choir_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if p == owner(g, src) && is_my_commander(g, p, m) { choir_proliferate(g, src, p) } else { Ok(()) }
}

fn choir_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p == owner(g, src) && atk.iter().any(|&m| g.perm(m).on_bf && is_my_commander(g, p, m)) {
        choir_proliferate(g, src, p)?;
    }
    Ok(vec![])
}

// ================================================================== Righteous Valkyrie
/// **Righteous Valkyrie**: whenever another Angel or Cleric you control enters, gain life equal to its toughness
fn valkyrie_enters(g: &mut Game, src: Src, m: PermId) -> Res {
    let me = owner(g, src);
    if m == src
        || owner(g, m) != me
        || !live(g, m)
        || !g.is_creature(m)
        || !(is_angel(g, m) || has_type(g, m, "cleric"))
    {
        return Ok(());
    }
    let t = etgh(g, m);
    if t > 0 && trigger_window(g, me, Some(src), "gain life", None)? {
        gain(g, me, t)?;
    }
    Ok(())
}

fn valkyrie_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
        return Ok(());
    }
    valkyrie_enters(g, src, m)
}

fn valkyrie_tokens(g: &mut Game, src: Src, p: PlayerId, toks: &[PermId]) -> Res {
    if p == owner(g, src) {
        for &m in toks {
            valkyrie_enters(g, src, m)?;
        }
    }
    Ok(())
}

/// as long as you have at least 7 life more than your starting life total (40), creatures you control get +2/+2
fn valkyrie_pt(g: &Game, m: PermId) -> (i32, i32) {
    let p = owner(g, m);
    if !g.is_creature(m) || g.player(p).life < 47 {
        return (0, 0);
    }
    let k = named(g, p, "Righteous Valkyrie").len() as i32;
    (2 * k, 2 * k)
}

// ================================================================== Sephara, Sky's Blade
/// other creatures you control with flying have indestructible
fn sephara_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "indestructible"
        && m != src
        && owner(g, m) == owner(g, src)
        && g.is_creature(m)
        && (g.perm(m).fly || crate::dsl::has_kw(g, m, "flying"))
}

fn flyers_to_tap(g: &Game, p: PlayerId) -> Vec<PermId> {
    let mut v: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            live(g, m) && g.is_creature(m) && !g.perm(m).tapped && (g.perm(m).fly || crate::dsl::has_kw(g, m, "flying"))
        })
        .collect();
    v.sort_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)));
    v
}

/// **Sephara's alternative cost**: pay {W} and tap four untapped creatures you control with flying rather than pay
/// its mana cost (only when the ordinary cost is out of reach and the creatures can spare the turn)
fn sephara_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post == Some(true) || !g.player(p).hand.contains(&c) || g.active != Some(p) || !can_pay(g, p, 0, "W", false) {
        return Ok(vec![]);
    }
    if flyers_to_tap(g, p).len() < 4 || can_pay(g, p, 4, "WWW", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 6.0,
        label: "cast Sephara for {W} and four flyers".into(),
        act: Some(Action::Plan { f: sephara_go, arg: c.0 as i64 }),
    }])
}

fn sephara_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let fl = flyers_to_tap(g, p);
    if !g.player(p).hand.contains(&c) || fl.len() < 4 || !can_pay(g, p, 0, "W", false) {
        return Ok(false);
    }
    if !pay(g, p, 0, "W", false)? {
        return Ok(false);
    }
    for &m in fl.iter().take(4) {
        g.perm_mut(m).tapped = true;
    }
    crate::glog!(g, "    {} taps four flyers to cast Sephara", g.player(p).name);
    crate::engine::cast::cast_card(g, p, c, "hand", Ctx::default())
}

// ================================================================== Starnheim Aspirant
/// Angel spells you cast cost {2} less
fn aspirant_cost(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    if caster == owner(g, src) && g.db.get(c).has_subtype("angel") { -2 } else { 0 }
}

// ================================================================== Vanguard Seraph
fn flag_set(g: &Game, p: PlayerId, key: Sym) -> bool {
    g.player(p).flag_turn.get(key) == Some(&g.turn_stamp())
}

/// **Vanguard Seraph**: whenever you gain life for the first time each turn, surveil 1
fn seraph_gain(g: &mut Game, src: Src, p: PlayerId, n: i32) -> Res {
    let key = intern(&format!("vseraph_{}", src.0));
    if p != owner(g, src) || n <= 0 || flag_set(g, p, key) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "surveil 1", None)? && crate::engine::values::once_per_turn(g, p, key) {
        crate::cardcode::scry(g, p, 1, true)?;
    }
    Ok(())
}

// ================================================================== Wojek Investigator
/// **Wojek Investigator**: at the beginning of your upkeep, investigate once for each opponent who has more cards
/// in hand than you
fn wojek_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    let mine = g.player(p).hand.len();
    let n = g.opps(p).filter(|&q| g.player(q).alive && g.player(q).hand.len() > mine).count() as i32;
    if n > 0 && trigger_window(g, p, Some(src), "investigate per opponent with more cards", None)? {
        investigate(g, p, n)?;
    }
    Ok(())
}

// ================================================================== Youthful Valkyrie
/// **Youthful Valkyrie**: whenever another Angel you control enters, put a +1/+1 counter on it
fn youthful_enters(g: &mut Game, src: Src, m: PermId) -> Res {
    let me = owner(g, src);
    if m == src || owner(g, m) != me || !live(g, m) || !g.is_creature(m) || !is_angel(g, m) {
        return Ok(());
    }
    if trigger_window(g, me, Some(src), "+1/+1 counter", None)? && live(g, src) {
        put_counters(g, src, 1)?;
    }
    Ok(())
}

fn youthful_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    youthful_enters(g, src, m)
}

fn youthful_tokens(g: &mut Game, src: Src, p: PlayerId, toks: &[PermId]) -> Res {
    if p == owner(g, src) {
        for &m in toks {
            youthful_enters(g, src, m)?;
        }
    }
    Ok(())
}

// ================================================================== Bishop of Wings
/// **Bishop of Wings**: whenever an Angel you control enters, gain 4 life (itself is no Angel)
fn bishop_enters(g: &mut Game, src: Src, m: PermId) -> Res {
    let me = owner(g, src);
    if owner(g, m) != me || !live(g, m) || !g.is_creature(m) || !is_angel(g, m) {
        return Ok(());
    }
    if trigger_window(g, me, Some(src), "gain 4 life", None)? {
        gain(g, me, 4)?;
    }
    Ok(())
}

fn bishop_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    bishop_enters(g, src, m)
}

fn bishop_tokens(g: &mut Game, src: Src, p: PlayerId, toks: &[PermId]) -> Res {
    if p == owner(g, src) {
        for &m in toks {
            bishop_enters(g, src, m)?;
        }
    }
    Ok(())
}

/// whenever an Angel you control dies, a 1/1 white Spirit with flying
fn bishop_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let me = owner(g, src);
    if owner(g, m) != me || !g.is_creature(m) || !is_angel(g, m) {
        return Ok(());
    }
    if trigger_window(g, me, Some(src), "create a 1/1 Spirit", None)? {
        make_tokens(g, me, spirit_token())?;
    }
    Ok(())
}

// ================================================================== Linvala, the Preserver
/// **Linvala, the Preserver**: when it enters, if an opponent has more life than you, gain 5 life; and if an
/// opponent controls more creatures than you, create a 3/3 white Angel with flying
fn linvala_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != owner(g, src) {
        return Ok(());
    }
    let life_behind = |g: &Game| g.opps(p).any(|q| g.player(q).alive && g.player(q).life > g.player(p).life);
    let creatures =
        |g: &Game, q: PlayerId| g.player(q).perms.iter().filter(|&&x| live(g, x) && g.is_creature(x)).count();
    let board_behind = |g: &Game| g.opps(p).any(|q| g.player(q).alive && creatures(g, q) > creatures(g, p));
    if life_behind(g) && trigger_window(g, p, Some(src), "gain 5 life", Some(5.0))? && life_behind(g) {
        gain(g, p, 5)?;
    }
    if board_behind(g) && trigger_window(g, p, Some(src), "create a 3/3 Angel", Some(5.0))? && board_behind(g) {
        make_tokens(g, p, angel_token(1, 3))?;
    }
    Ok(())
}

// ================================================================== registry
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    // Giada
    let c = r.card(db, "Giada, Font of Hope")?;
    c.etb = Some(giada_etb);
    c.tokens_enter = Some(giada_tokens);
    at_once(c, Event::Etb);
    at_once(c, Event::TokensEnter);
    r.card(db, "Search the Premises")?.attack = Some(premises_attack);
    let c = r.card(db, "Secret Rendezvous")?;
    c.resolve = Some(rendezvous);
    c.prio = Some(rendezvous_prio);
    let c = r.card(db, "Angel of Vitality")?;
    c.etb = Some(selfpt_on);
    c.self_pt = Some(vitality_pt);
    at_once(c, Event::Etb);
    let c = r.card(db, "Angel of the Ruins")?;
    c.etb = Some(ruins_etb);
    c.hand_options = Some(ruins_cycle);
    let c = r.card(db, "Angelic Field Marshal")?;
    c.etb = Some(selfpt_on);
    c.self_pt = Some(marshal_pt);
    c.grant_kw = Some(marshal_kw);
    at_once(c, Event::Etb);
    r.card(db, "Angelic Sleuth")?.leaves = Some(sleuth_leaves);
    let c = r.card(db, "Court of Grace")?;
    c.etb = Some(court_etb);
    c.upkeep = Some(court_upkeep);
    let c = r.card(db, "Defy Death")?;
    c.resolve = Some(defy_death);
    c.prio = Some(defy_prio);
    r.card(db, "Emeria Shepherd")?.landfall = Some(shepherd_landfall);
    r.card(db, "Exemplar of Light")?.gain_life = Some(exemplar_gain);
    r.card(db, "Fateful Absence")?.resolve = Some(absence);
    r.card(db, "Firemane Commando")?.attack = Some(commando_attack);
    let c = r.card(db, "Herald of War")?;
    c.attack = Some(herald_attack);
    c.cost = Some(herald_cost);
    let c = r.card(db, "Heraldic Banner")?;
    c.etb = Some(selfpt_on);
    at_once(c, Event::Etb);
    r.creature_pt.push(banner_pt);
    let c = r.card(db, "Merchant of Truth")?;
    c.dies = Some(merchant_dies);
    c.self_dies = Some(merchant_self_dies);
    c.attack = Some(merchant_attack);
    at_once(c, Event::Attack);
    r.card(db, "Metropolis Reformer")?.player_hexproof = Some(reformer_hexproof);
    let c = r.card(db, "Norn's Choirmaster")?;
    c.etb = Some(choir_etb);
    c.attack = Some(choir_attack);
    let c = r.card(db, "Righteous Valkyrie")?;
    c.etb = Some(valkyrie_etb);
    c.tokens_enter = Some(valkyrie_tokens);
    at_once(c, Event::TokensEnter);
    r.creature_pt.push(valkyrie_pt);
    let c = r.card(db, "Sephara, Sky's Blade")?;
    c.grant_kw = Some(sephara_kw);
    c.hand_options = Some(sephara_hand);
    r.card(db, "Starnheim Aspirant")?.cost = Some(aspirant_cost);
    r.card(db, "Vanguard Seraph")?.gain_life = Some(seraph_gain);
    r.card(db, "Wojek Investigator")?.upkeep = Some(wojek_upkeep);
    let c = r.card(db, "Youthful Valkyrie")?;
    c.etb = Some(youthful_etb);
    c.tokens_enter = Some(youthful_tokens);
    at_once(c, Event::TokensEnter);
    let c = r.card(db, "Bishop of Wings")?;
    c.etb = Some(bishop_etb);
    c.tokens_enter = Some(bishop_tokens);
    c.dies = Some(bishop_dies);
    at_once(c, Event::TokensEnter);
    r.card(db, "Linvala, the Preserver")?.etb = Some(linvala_etb);
    register_temple(r, db)?;
    register_speaker(r, db)?;
    Ok(())
}

// ================================================================== Temple of the False God
/// **Temple of the False God**: {T}: Add {C}{C}. Activate only if you control five or more lands
fn temple_mana(g: &Game, p: PlayerId, _l: crate::ids::LandId) -> u32 {
    if g.player(p).lands.len() >= 5 { 2 } else { 0 }
}

pub fn register_temple(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Temple of the False God")?.dyn_mana_land = Some(temple_mana);
    Ok(())
}

// ================================================================== Speaker of the Heavens
/// **Speaker of the Heavens**: {T}: create a 4/4 white Angel with flying. Activate only if you have at least 7 life more
/// than your starting life total (47) and only as a sorcery. The AI taps it in its main phase whenever the life is there.
fn speaker_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if owner(g, src) != p
        || post.is_some()
        || g.active != Some(p)
        || !live(g, src)
        || x.tapped
        || x.sick
        || g.player(p).life < 47
    {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 6.0,
        label: "Speaker of the Heavens: a 4/4 Angel".into(),
        act: Some(Action::Ability { src, f: speaker_go, arg: 0 }),
    }])
}

fn speaker_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let x = g.perm(src);
    if !live(g, src) || x.tapped || x.sick || g.player(p).life < 47 || g.active != Some(p) {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if crate::engine::stack::ability_window(g, p, Some(src), "create a 4/4 Angel", None, None)? {
        make_tokens(g, p, angel_token(1, 4))?;
    }
    Ok(true)
}

pub fn register_speaker(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Speaker of the Heavens")?.options = Some(speaker_options);
    Ok(())
}
