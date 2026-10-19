//! Cards of the two four-colour partner decks (Tymna the Weaver with Thrasios, Triton Hero or Kraum, Ludevic's Opus):
//! the ones the compiled Oracle text and the engine's tags don't cover. The audit verdicts are in
//! data/card_overrides.toml; each function's doc comment says how the card is modeled and what is left out.
//!
//! - Thrasios, Kraum, Black Widow, Abigale, Cuboid Colony, Elemental Mascot, Falcon and Redwing, Photon, Faeburrow
//!   Elder, Paradox Surveyor, Spectral Lynx, Twisted Fates: a hook slot each (see `register`).
//! - Sphinx of New Prahv's tax is ward {2} in the card data (the engine charges ward when a spell targets, and the
//!   AI won't aim a spell it can't pay the ward for), which is the same as "costs {2} more" for spells.
//! - Razia and King Darien answer removal and wipes through `protect` and `darien_wipe`, called from the top of
//!   `ai::decks::protect_response` / `wipe_response`.
//! - Tymna the Weaver herself is in `legends.rs`.

use super::partials::eot_kw;
use crate::cards::{CardDb, Colors};
use crate::engine::cast::{castable, casts_this_turn};
use crate::engine::mana::{Source, Unit, can_pay, pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::card_worth;
use crate::engine::values::{colors_of, epow, etgh, once_per_turn, pval, silenced, untargetable};
use crate::engine::zones::{Enter, Tokens, die, draw, enter, landfall, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

pub const THRASIOS: &str = "Thrasios, Triton Hero";
pub const KRAUM: &str = "Kraum, Ludevic's Opus";
pub const BLACK_WIDOW: &str = "Black Widow, Agile Avenger";
pub const ABIGALE: [&str; 2] = ["Abigale, Poet Laureate", "Abigale, Poet Laureate // Heroic Stanza"];
pub const COLONY: &str = "Cuboid Colony";
pub const MASCOT: &str = "Elemental Mascot";
pub const FALCON: &str = "Falcon and Redwing";
pub const PHOTON: &str = "Photon, Mighty Marvel";
pub const ELDER: &str = "Faeburrow Elder";
pub const SURVEYOR: &str = "Paradox Surveyor";
pub const LYNX: &str = "Spectral Lynx";
pub const FATES: &str = "Twisted Fates";
pub const RAZIA: &str = "Razia, Boros Archangel";
pub const DARIEN: &str = "King Darien XLVIII";

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn name_of(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

fn on_bf_named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && !g.perm(m).phased && name_of(g, m) == name)
        .collect()
}

fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg }) }
}

// ================================================================== Thrasios, Triton Hero
/// **Thrasios**: {4}: scry 1, then reveal the top card of your library; a land goes onto the battlefield tapped,
/// anything else is drawn. The AI activates it as often as it can afford at the end of the turn before its own (the
/// end-of-turn window, `post` None: the mana would untap anyway), and in its second main phase after everything else
/// it wanted to do (a low utility, above "stop"). Never when the library is nearly empty.
fn thrasios_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || g.perm(src).phased {
        return Ok(vec![]);
    }
    let cost = crate::impls::t6_green1::creature_ability_cost(g, p, 4); // Training Grounds, Biomancer's Familiar
    let (u, when) = match post {
        None => (3.0, "eot"),
        Some(true) if g.active == Some(p) => (-2.0, "main2"),
        _ => return Ok(vec![]),
    };
    if crate::engine::values::once_per_turn(g, p, crate::sym::intern(&format!("ty_th_f_{when}"))) {
        g.player_mut(p).stat(crate::sym::intern(&format!("ty_th_turns_{when}")), 1);
    }
    if !can_pay(g, p, cost, "", false) || g.player(p).library.len() < 3 {
        return Ok(vec![]);
    }
    if when == "main2" {
        // keep the mana for the cheapest counterspell held (a Mana Leak up beats a scry), spend the rest
        let keep = g
            .player(p)
            .hand
            .iter()
            .filter(|&&c| g.db.get(c).tag(Tag::Ctr) && !g.db.get(c).creature)
            .map(|&c| g.db.get(c).cmc)
            .min()
            .unwrap_or(0);
        if crate::engine::mana::total_mana(g, p, false) < cost + keep {
            return Ok(vec![]);
        }
    }
    if crate::engine::values::once_per_turn(g, p, crate::sym::intern(&format!("ty_th_g_{when}"))) {
        g.player_mut(p).stat(crate::sym::intern(&format!("ty_th_spare4_{when}")), 1);
    }
    Ok(vec![ability(u, "Thrasios: scry 1, a land or a card", src, thrasios_go, 0)])
}

fn thrasios_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let cost = crate::impls::t6_green1::creature_ability_cost(g, p, 4);
    if !g.perm(src).on_bf || !can_pay(g, p, cost, "", false) || g.player(p).library.len() < 3 {
        return Ok(false);
    }
    g.player_mut(p).stat(crate::sym::intern("ty_thrasios_acts"), 1);
    if crate::t6stats::is_t6(g, p) {
        let when = if g.active == Some(p) { "own" } else { "eot" };
        g.player_mut(p).stat(crate::sym::intern(&format!("mn|dig|{when}")), 1);
    }
    pay(g, p, cost, "", false)?;
    if !ability_window(g, p, Some(src), "scry 1, then a land or a card", None, None)? {
        return Ok(true);
    }
    // Tier 6: while mana is short, a land on top stays (a land onto the battlefield is +1 mana for good)
    let keep_land = crate::t6stats::is_t6(g, p)
        && crate::engine::mana::total_mana(g, p, false) < 8
        && g.player(p).library.last().is_some_and(|&t| g.db.get(t).land);
    if !keep_land {
        crate::cardcode::scry(g, p, 1, false)?;
    }
    let Some(&top) = g.player(p).library.last() else { return Ok(true) };
    if g.db.get(top).land {
        crate::glog!(g, "    Thrasios reveals {}: onto the battlefield tapped", g.db.get(top).name);
        if crate::t6stats::is_t6(g, p) {
            g.player_mut(p).stat(crate::sym::intern("mn|dig|land"), 1);
        }
        g.player_mut(p).library.pop();
        g.player_mut(p).seen.insert(top);
        let l = g.add_land(p, top, true);
        landfall(g, p)?;
        if g.db.get(top).tag(Tag::F) && g.player(p).lands.last() == Some(&l) {
            crate::engine::turn::crack_fetch(g, p, l)?;
        }
    } else {
        crate::glog!(g, "    Thrasios reveals {}: drawn", g.db.get(top).name);
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

// ================================================================== Kraum, Ludevic's Opus
/// **Kraum**: whenever an opponent casts their second spell each turn, you draw a card (flying, haste and partner
/// are keywords; partner is the engine's)
fn kraum_cast(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    let o = owner(g, src);
    if caster == o || !g.player(o).alive || casts_this_turn(g, caster) != 2 {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "draw a card", None)? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ================================================================== Black Widow, Agile Avenger
/// **Black Widow**: whenever an opponent draws their second card each turn, a +1/+1 counter on her and you draw a
/// card (menace is a keyword)
fn widow_draw(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if p == o || !g.player(p).alive || g.player(p).draw_n != 2 {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "a counter and a card", None)? {
        return Ok(());
    }
    let key = intern(&format!("widow{}{}", src.0, g.player(p).key));
    if once_per_turn(g, o, key) {
        crate::cardcode::add_counters(g, src, 1);
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ================================================================== Abigale, Poet Laureate // Heroic Stanza
/// **Abigale**: whenever you cast a creature spell, she becomes prepared. While prepared, {1}{W/B} casts a copy of
/// Heroic Stanza (a +1/+1 counter on target creature) and unprepares her. Sorcery speed; the AI puts the counter on
/// the creature it most wants bigger (an evasive one first) in its first main phase, or its second if it has to.
fn abigale_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster == owner(g, src) && g.db.get(c).creature {
        g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(true));
    }
    Ok(())
}

/// the colour of the hybrid symbol the AI can pay with
fn stanza_pip(g: &Game, p: PlayerId) -> Option<&'static str> {
    ["W", "B"].into_iter().find(|pip| can_pay(g, p, 1, pip, false))
}

/// the creature a counter helps most: evasion first, then power
fn counter_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && !g.perm(m).phased && g.is_creature(m))
        .collect();
    max_by(&cs, |m| {
        let ev = ["flying", "menace", "trample"].iter().filter(|k| crate::dsl::has_kw(g, m, k)).count() as f64;
        ev * 3.0 + epow(g, m) as f64 + if g.perm(m).sick { 0.0 } else { 1.0 }
    })
}

fn abigale_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none()
        || p != owner(g, src)
        || g.active != Some(p)
        || g.perm(src).phased
        || !g.perm(src).data.truthy(DataKey::Prepared)
        || stanza_pip(g, p).is_none()
        || counter_target(g, p).is_none()
    {
        return Ok(vec![]);
    }
    let u = if post == Some(false) { 1.6 } else { 1.0 };
    Ok(vec![ability(u, "Heroic Stanza (Abigale, prepared copy)", src, abigale_go, 0)])
}

fn abigale_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some(pip) = stanza_pip(g, p) else { return Ok(false) };
    let Some(t) = counter_target(g, p) else { return Ok(false) };
    if !g.perm(src).data.truthy(DataKey::Prepared) || !pay(g, p, 1, pip, false)? {
        return Ok(false);
    }
    g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(false));
    super::mine::cast_copy_with(g, p, None, None, false, 2.0, &mut |g| {
        if g.perm(t).on_bf {
            crate::glog!(g, "    Heroic Stanza: a +1/+1 counter on {}", g.perm(t).name);
            crate::cardcode::add_counters(g, t, 1);
        }
        Ok(())
    })?;
    Ok(true)
}

// ================================================================== Cuboid Colony
/// **Cuboid Colony** (flash, flying, trample): increment, a +1/+1 counter whenever you cast a spell with more mana
/// spent than its power or toughness. The mana spent is the spell's mana value (an X spell's X is not counted).
fn colony_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster != owner(g, src) {
        return Ok(());
    }
    let spent = g.db.get(c).cmc as i32;
    if spent > epow(g, src).min(etgh(g, src)) && trigger_window(g, caster, Some(src), "increment", None)? {
        crate::cardcode::add_counters(g, src, 1);
    }
    Ok(())
}

// ================================================================== Elemental Mascot
/// **Elemental Mascot** (flying, vigilance): opus. The +1/+0 for an instant or sorcery is the compiled ability; this
/// is the rest: with five or more mana spent (the spell's mana value), exile the top card of your library, playable
/// until the end of your next turn.
fn mascot_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let d = g.db.get(c);
    if caster != owner(g, src) || !(d.instant || d.sorcery) || d.cmc < 5 || g.player(caster).library.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, caster, Some(src), "exile the top card", None)? {
        return Ok(());
    }
    let Some(top) = g.player_mut(caster).library.pop() else { return Ok(()) };
    crate::glog!(g, "    Elemental Mascot exiles {} (playable until the end of your next turn)", g.db.get(top).name);
    let pl = g.player_mut(caster);
    pl.seen.insert(top);
    pl.hand.push(top);
    let until = pl.turns + 1;
    pl.impulse_long.push((top, until));
    Ok(())
}

// ================================================================== Falcon and Redwing
/// **Falcon and Redwing** (flying): whenever it deals combat damage to a player, that many 1/1 white Bird tokens with
/// flying, then a +1/+1 counter on it
fn falcon_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if a != src || p != owner(g, src) || d == p || dmg <= 0 || !g.perm(src).on_bf {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), &format!("{dmg} Birds and a counter"), None)? {
        return Ok(());
    }
    let spec = Tokens {
        fly: true,
        tgh: Some(1),
        color: Some(Colors::from_letters("W")),
        types: vec!["bird"],
        ..Tokens::new(dmg as u32, 1)
    };
    make_tokens(g, p, spec)?;
    crate::cardcode::add_counters(g, src, 1);
    Ok(())
}

// ================================================================== Photon, Mighty Marvel
/// **Photon** (flying): whenever it deals combat damage to a player, add that much mana of any one colour, which
/// doesn't empty until end of turn. Floating mana lasts to the end of the turn here too. The AI picks the colour
/// most of its hand wants.
fn photon_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if a != src || p != owner(g, src) || d == p || dmg <= 0 {
        return Ok(());
    }
    let mut want = [0u32; 5];
    for &c in &g.player(p).hand {
        let cd = g.db.get(c);
        if cd.land {
            continue;
        }
        for (i, col) in ['W', 'U', 'B', 'R', 'G'].into_iter().enumerate() {
            want[i] += cd.pips.chars().filter(|&x| x == col).count() as u32;
        }
    }
    let best = (0..5).max_by_key(|&i| (want[i], std::cmp::Reverse(i))).unwrap_or(0);
    let n = dmg as u32;
    let f = &mut g.player_mut(p).floating;
    if want[best] == 0 {
        f.c += n; // nothing coloured to cast: colourless is as good
    } else {
        match best {
            0 => f.w += n,
            1 => f.u += n,
            2 => f.b += n,
            3 => f.r += n,
            _ => f.g += n,
        }
    }
    crate::glog!(g, "    Photon adds {n} mana");
    Ok(())
}

// ================================================================== Faeburrow Elder
/// the colours among the permanents p controls
pub fn colors_among(g: &Game, p: PlayerId) -> Colors {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| g.perm(m).on_bf && !g.perm(m).phased)
        .fold(Colors::NONE, |acc, &m| acc.union(colors_of(g, m)))
}

/// **Faeburrow Elder** (vigilance): +1/+1 for each colour among permanents you control (base 0/0)
fn elder_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    let n = colors_among(g, p).count() as i32;
    (n, n)
}

fn elder_enters(g: &mut Game, _p: PlayerId, _m: PermId) -> Res {
    g.selfpt = true;
    Ok(())
}

fn elder_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

/// **Faeburrow Elder**: {T}: for each colour among permanents you control, add one mana of that colour. One unit of
/// mana per colour, all from the same untapped source (using any of them taps it, so the others are lost).
fn elder_mana(g: &Game, src: Src, p: PlayerId, _u: &[Unit]) -> Vec<Unit> {
    let x = g.perm(src);
    if x.owner != p || x.tapped || x.sick || x.phased || !x.on_bf {
        return vec![];
    }
    let cols = colors_among(g, p);
    ['W', 'U', 'B', 'R', 'G']
        .into_iter()
        .filter(|&c| cols.has(c))
        .map(|c| Unit { src: Source::Perm(src), cols: Colors::from_letters(&c.to_string()), amt: 1 })
        .collect()
}

// ================================================================== Paradox Surveyor
/// does c have {X} in its mana cost (the `x` tag)?
fn has_x(g: &Game, c: CardId) -> bool {
    g.db.get(c).tag(Tag::X)
}

/// **Paradox Surveyor** (reach): when it enters, look at the top five cards; you may reveal a land card or a card with
/// {X} in its cost from among them and put it in your hand; the rest go to the bottom in a random order. The AI takes
/// the best such card by `card_worth` (a land when it has few).
fn surveyor_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || owner(g, m) != p {
        return Ok(());
    }
    let n = g.player(p).library.len().min(5);
    let mut top: Vec<CardId> = (0..n).filter_map(|_| g.player_mut(p).library.pop()).collect();
    for &c in &top {
        g.player_mut(p).seen.insert(c);
    }
    let lands_out = g.player(p).lands.len() + g.player(p).hand.iter().filter(|&&c| g.db.get(c).land).count();
    let eligible: Vec<CardId> = top.iter().copied().filter(|&c| g.db.get(c).land || has_x(g, c)).collect();
    let pick = max_by(&eligible, |c| {
        let base = card_worth(g, p, c, false);
        if g.db.get(c).land && lands_out < 6 { base + 100.0 } else { base }
    });
    if let Some(c) = pick {
        top.retain(|&x| x != c);
        g.player_mut(p).hand.push(c);
        crate::glog!(g, "    Paradox Surveyor puts {} into the hand", g.db.get(c).name);
    }
    let mut rest = top;
    g.rng.shuffle(&mut rest);
    g.player_mut(p).library.splice(0..0, rest);
    Ok(())
}

// ================================================================== Spectral Lynx
/// **Spectral Lynx**: protection from green is the card data's; {B}: regenerate (the AI pays when it can)
fn lynx_regen(g: &mut Game, m: PermId) -> Res<bool> {
    let p = owner(g, m);
    if !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "    {} regenerates", g.perm(m).name);
    Ok(true)
}

// ================================================================== Twisted Fates
/// **Twisted Fates**: destroy target nonland permanent; put a +1/+1 counter on each creature target player controls.
/// The player is always the caster (the counters never help an opponent).
fn fates_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &crate::state::Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) {
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
    }
    for m in g.player(p).perms.clone() {
        if g.perm(m).on_bf && !g.perm(m).phased && g.is_creature(m) {
            crate::cardcode::add_counters(g, m, 1);
        }
    }
    Ok("gy")
}

// ================================================================== Razia and King Darien: answers to removal
/// Razia, Boros Archangel: {T}: the next 3 damage that would be dealt to target creature you control this turn is
/// dealt to another target creature instead. The AI uses it against burn of 3 or less aimed at its creature that
/// would die to it (all of the damage moves, to the opposing creature it hurts most); damage from combat and burn
/// of more than 3 is not redirected.
fn razia_redirect(g: &mut Game, owner_p: PlayerId, m: PermId, kind: Sym) -> Res<bool> {
    let Some(n) = kind.strip_prefix("dmg").and_then(|s| s.parse::<i32>().ok()) else { return Ok(false) };
    if !(1..=3).contains(&n) || etgh(g, m) > n || !g.is_creature(m) {
        return Ok(false);
    }
    let Some(r) = on_bf_named(g, owner_p, RAZIA).into_iter().find(|&r| !g.perm(r).tapped && r != m) else {
        return Ok(false);
    };
    let victims: Vec<PermId> = g
        .opps(owner_p)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&x| g.perm(x).on_bf && g.is_creature(x) && !untargetable(g, x))
        .collect();
    let Some(v) = max_by(&victims, |x| pval(g, x) + if etgh(g, x) <= n { 100.0 } else { 0.0 }) else {
        return Ok(false);
    };
    g.perm_mut(r).tapped = true;
    crate::glog!(g, "    Razia redirects {n} damage from {} to {}", g.perm(m).name, g.perm(v).name);
    apply_removal(g, Some(owner_p), v, &format!("dmg{n}"), None)?;
    Ok(true)
}

/// King Darien XLVIII: sacrifice it and creature tokens you control gain hexproof and indestructible until end of
/// turn. The AI does it when removal is aimed at a token worth more than the King, or when a destroy sweeper would
/// kill tokens worth more than the King (the tokens then survive it).
fn darien_save(g: &mut Game, owner_p: PlayerId, worth: f64) -> Res<bool> {
    let Some(&k) = on_bf_named(g, owner_p, DARIEN).first() else { return Ok(false) };
    if worth <= pval(g, k) + 0.5 {
        return Ok(false);
    }
    if !ability_window(g, owner_p, Some(k), "creature tokens gain hexproof and indestructible", None, None)? {
        return Ok(false);
    }
    crate::glog!(g, "    King Darien is sacrificed: tokens gain hexproof and indestructible");
    die(g, k, "sac")?;
    for m in g.player(owner_p).perms.clone() {
        if g.perm(m).on_bf && g.perm(m).token && g.is_creature(m) {
            eot_kw(g, m, intern("hexproof"));
            eot_kw(g, m, intern("indestructible"));
        }
    }
    Ok(true)
}

/// `ai::decks::protect_response`, first thing: removal of `kind` is aimed at `owner_p`'s permanent m
pub fn protect(
    g: &mut Game,
    owner_p: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if !g.perm(m).on_bf || g.humans.get(owner_p).is_some() {
        return Ok(false);
    }
    if let (Some(s), Some(a)) = (spell, actor)
        && a != owner_p
        && hexproof_from(g, m, g.db.get(s).colors)
    {
        return Ok(true); // Skrelv gave it hexproof from the spell's colour this turn: the spell has no legal target
    }
    if skrelv_protect(g, owner_p, m, actor, spell)? {
        return Ok(true);
    }
    if endurance_protect(g, owner_p, m, kind, actor)? {
        return Ok(true);
    }
    if g.hooks.is_empty() {
        return blessing_protect(g, owner_p, m, actor, spell);
    }
    if razia_redirect(g, owner_p, m, kind)? {
        return Ok(true);
    }
    if g.perm(m).token && g.is_creature(m) && kind != "bounce" && darien_save(g, owner_p, pval(g, m))? {
        return Ok(true);
    }
    blessing_protect(g, owner_p, m, actor, spell)
}

/// `ai::decks::wipe_response`, first thing: a sweeper of `kind` is about to hit q's board
pub fn darien_wipe(g: &mut Game, q: PlayerId, kind: Sym, caster: PlayerId) -> Res {
    if q == caster || g.hooks.is_empty() || g.humans.get(q).is_some() {
        return Ok(());
    }
    if !matches!(kind, "destroy" | "dmg13" | "austere" | "nib" | "austere2" | "vandal") {
        return Ok(());
    }
    let w: f64 = g
        .player(q)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && g.perm(m).token && g.is_creature(m) && !g.perm(m).phased)
        .map(|m| pval(g, m))
        .sum();
    if w > 0.0 {
        darien_save(g, q, w)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ registration
/// the card's code slots, if the card data has the card (Abigale has two names, and the data may lack a card)
fn card_if<'a>(r: &'a mut Registry, db: &CardDb, name: &str) -> Result<Option<&'a mut crate::hooks::CardImpl>, String> {
    if db.id(name).is_none() {
        return Ok(None);
    }
    Ok(Some(r.card(db, name)?))
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    crate::impls::tymna_partners::register(r, db)?;
    if let Some(c) = card_if(r, db, THRASIOS)? {
        c.options = Some(thrasios_options);
    }
    if let Some(c) = card_if(r, db, KRAUM)? {
        c.cast = Some(kraum_cast);
    }
    if let Some(c) = card_if(r, db, BLACK_WIDOW)? {
        c.draw = Some(widow_draw);
    }
    for name in ABIGALE {
        if let Some(c) = card_if(r, db, name)? {
            c.cast = Some(abigale_cast);
            c.options = Some(abigale_options);
        }
    }
    if let Some(c) = card_if(r, db, COLONY)? {
        c.cast = Some(colony_cast);
    }
    if let Some(c) = card_if(r, db, MASCOT)? {
        c.cast = Some(mascot_cast);
    }
    if let Some(c) = card_if(r, db, FALCON)? {
        c.combat_damage = Some(falcon_damage);
    }
    if let Some(c) = card_if(r, db, PHOTON)? {
        c.combat_damage = Some(photon_damage);
    }
    if let Some(c) = card_if(r, db, ELDER)? {
        c.self_pt = Some(elder_pt);
        c.as_enters = Some(elder_enters);
        c.etb = Some(elder_etb);
        c.extra_mana = Some(elder_mana);
    }
    if let Some(c) = card_if(r, db, SURVEYOR)? {
        c.etb = Some(surveyor_etb);
    }
    if let Some(c) = card_if(r, db, LYNX)? {
        c.self_regen = Some(lynx_regen);
    }
    if let Some(c) = card_if(r, db, FATES)? {
        c.resolve = Some(fates_resolve);
    }
    register_more(r, db)?;
    register_rocks(r, db)?;
    register_charm(r, db)?;
    register_witherbloom(r, db)?;
    register_gear(r, db)?;
    register_phyrexian(r, db)?;
    register_phyrexian2(r, db)?;
    register_audit_ysh(r, db)?;
    Ok(())
}

// ================================================================== Dakkon Blackblade, Star Compass, Embrace the Paradox
pub const DAKKON: &str = "Dakkon Blackblade";
pub const COMPASS: &str = "Star Compass";
pub const EMBRACE: &str = "Embrace the Paradox";

/// **Dakkon Blackblade**: power and toughness each equal the number of lands you control (a 0/0 base in the card data)
fn dakkon_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    let n = g.player(p).lands.len() as i32;
    (n, n)
}

/// **Star Compass**: `{T}`: one mana of any colour a basic land you control could produce (engine/mana.rs asks for the
/// colours; it enters tapped, which is the compiled note)
pub fn star_compass_cols(g: &Game, p: PlayerId) -> Colors {
    let mut s = Colors::NONE;
    for &l in &g.player(p).lands {
        let letter = match &*g.db.get(g.land(l).cd).name {
            "Plains" | "Snow-Covered Plains" => "W",
            "Island" | "Snow-Covered Island" => "U",
            "Swamp" | "Snow-Covered Swamp" => "B",
            "Mountain" | "Snow-Covered Mountain" => "R",
            "Forest" | "Snow-Covered Forest" => "G",
            _ => continue,
        };
        s = s.union(Colors::from_letters(letter));
    }
    s
}

/// **Embrace the Paradox**: draw three cards, then you may put a land card from your hand onto the battlefield tapped
/// (the AI always does: the land making the most colours, a fetch land breaking ties)
fn embrace_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &crate::state::Ctx) -> Res<Sym> {
    draw(g, p, 3, false)?;
    let ls: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
    let pick = max_by(&ls, |c| {
        let t = &g.db.get(c).tags;
        t.str(Tag::C).map_or(0, |s| s.chars().count()) as f64 * 10.0 + t.has(Tag::F) as i32 as f64
    });
    if let Some(c) = pick {
        super::partials::remove_card(&mut g.player_mut(p).hand, c);
        let l = g.add_land(p, c, true);
        landfall(g, p)?;
        if g.db.get(c).tag(Tag::F) && g.player(p).lands.last() == Some(&l) {
            crate::engine::turn::crack_fetch(g, p, l)?;
        }
    }
    Ok("gy")
}

pub fn register_more(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if let Some(c) = card_if(r, db, DAKKON)? {
        c.self_pt = Some(dakkon_pt);
        c.as_enters = Some(elder_enters);
        c.etb = Some(elder_etb);
    }
    if let Some(c) = card_if(r, db, EMBRACE)? {
        c.resolve = Some(embrace_resolve);
    }
    Ok(())
}

// ================================================================== Elenda, the Dusk Rose; rocks that enter tapped
pub const ELENDA: &str = "Elenda, the Dusk Rose";

/// **Elenda, the Dusk Rose**: when she dies, X 1/1 white Vampire tokens with lifelink, X her power. (The counter she
/// gets whenever another creature dies is the compiled ability; the compiled death trigger made a fixed three.)
fn elenda_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(m).owner;
    let x = epow(g, m).max(0);
    if x == 0 || !trigger_window(g, o, Some(m), &format!("{x} Vampire tokens"), None)? {
        return Ok(());
    }
    let spec = Tokens {
        tgh: Some(1),
        lifelink: true,
        color: Some(Colors::from_letters("W")),
        types: vec!["vampire"],
        ..Tokens::new(x as u32, 1)
    };
    make_tokens(g, o, spec)?;
    Ok(())
}

/// the artifact enters tapped (Moss Diamond, Star Compass; the engine keeps "enters tapped" on artifacts as a note)
fn enters_tapped(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.perm_mut(m).tapped = true;
    }
    Ok(())
}

pub fn register_rocks(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    for name in ["Moss Diamond", COMPASS] {
        if let Some(c) = card_if(r, db, name)? {
            c.etb = Some(enters_tapped);
        }
    }
    if let Some(c) = card_if(r, db, ELENDA)? {
        c.self_dies = Some(elenda_dies);
    }
    Ok(())
}

// ================================================================== Silverquill Charm
pub const CHARM: &str = "Silverquill Charm";

/// the modes the AI would pick now: (utility, mode 1-3, target). Mode 1: two +1/+1 counters on target creature (our
/// best); 2: exile target creature with power 2 or less (their best); 3: each opponent loses 3 life and we gain 3.
fn charm_modes(g: &Game, p: PlayerId) -> Vec<(f64, i64, Option<PermId>)> {
    let mut out = vec![];
    if let Some(t) = counter_target(g, p) {
        out.push((1.0 + 0.2 * epow(g, t) as f64, 1, Some(t)));
    }
    let victims: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&x| g.perm(x).on_bf && g.is_creature(x) && epow(g, x) <= 2 && !untargetable(g, x))
        .collect();
    if let Some(v) = max_by(&victims, |x| pval(g, x)) {
        out.push((0.9 + 0.5 * pval(g, v), 2, Some(v)));
    }
    let opps = g.opps(p).filter(|&q| g.player(q).alive).count() as f64;
    out.push((0.8 + 0.3 * opps + if g.player(p).life < 15 { 0.6 } else { 0.0 }, 3, None));
    out
}

fn charm_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let (cg, cp) = crate::engine::mana::cost_of(g, p, c);
    if !can_pay(g, p, cg, &cp, false) || !crate::engine::cast::castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    let label = |m: i64| match m {
        1 => "Silverquill Charm: two +1/+1 counters",
        2 => "Silverquill Charm: exile a creature",
        _ => "Silverquill Charm: drain 3",
    };
    let eot = if post.is_none() { -0.4 } else { 0.0 };
    Ok(charm_modes(g, p)
        .into_iter()
        .map(|(u, mode, t)| {
            let arg = c.0 as i64 | (mode << 32) | (t.map_or(0, |m| m.0 as i64 + 1) << 40);
            Opt { utility: u + eot, label: label(mode).into(), act: Some(Action::Plan { f: charm_go, arg }) }
        })
        .collect())
}

fn charm_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg & 0xffff_ffff) as u16);
    let mode = (arg >> 32) & 0xff;
    let t = (arg >> 40) as u32;
    let target = (t > 0).then(|| PermId(t - 1));
    let ctx = crate::state::Ctx { x: mode as i32, target, ..Default::default() };
    crate::ai::brain::do_cast_with(g, p, c, None, ctx)
}

fn charm_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &crate::state::Ctx) -> Res<Sym> {
    let (mode, target) = if (1..=3).contains(&ctx.x) {
        (ctx.x as i64, ctx.target)
    } else {
        let best = charm_modes(g, p).into_iter().max_by(|a, b| a.0.total_cmp(&b.0));
        best.map_or((3, None), |(_, m, t)| (m, t))
    };
    match (mode, target) {
        (1, Some(t)) if g.perm(t).on_bf => crate::cardcode::add_counters(g, t, 2),
        (2, Some(t)) if g.perm(t).on_bf && g.is_creature(t) && epow(g, t) <= 2 => {
            apply_removal(g, Some(p), t, "exile", Some(c))?;
        }
        (3, _) => {
            for q in g.opps(p).collect::<Vec<_>>() {
                if g.player(q).alive {
                    crate::engine::life::lose_life(g, q, 3, Some(p), "drain", None)?;
                }
            }
            crate::engine::life::gain(g, p, 3)?;
        }
        _ => {}
    }
    Ok(intern("gy"))
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

pub fn register_charm(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if let Some(c) = card_if(r, db, CHARM)? {
        c.resolve = Some(charm_resolve);
        c.hand_options = Some(charm_options);
        c.prio = Some(zero_prio);
    }
    Ok(())
}

// ================================================================== Witherbloom Charm
pub const WITHERBLOOM: &str = "Witherbloom Charm";

/// the permanent mode 1 would sacrifice: a token worth next to nothing (Treasure, Food, Clue, a spare 1/1)
fn witherbloom_fodder(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && !g.perm(m).phased && g.perm(m).token && pval(g, m) <= 1.5)
        .collect();
    cs.iter().copied().min_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)))
}

/// the opposing nonland permanent with mana value 2 or less (a token counts 0) that is worth the most
fn witherbloom_victim(g: &Game, p: PlayerId) -> Option<PermId> {
    let vs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&x| {
            let y = g.perm(x);
            y.on_bf
                && !y.phased
                && !y.is_cmd
                && !untargetable(g, x)
                && !y.cd.is_some_and(|c| g.db.get(c).land)
                && y.cd.is_none_or(|c| g.db.get(c).cmc <= 2)
        })
        .collect();
    max_by(&vs, |x| pval(g, x))
}

/// the modes the AI would pick now: (utility, mode 1-3, target). Mode 1: sacrifice a worthless token, draw two; 2: gain
/// 5 life; 3: destroy their best nonland permanent with mana value 2 or less.
fn witherbloom_modes(g: &Game, p: PlayerId) -> Vec<(f64, i64, Option<PermId>)> {
    let mut out = vec![];
    if witherbloom_fodder(g, p).is_some() {
        out.push((1.6, 1, None));
    }
    if let Some(v) = witherbloom_victim(g, p).filter(|&v| pval(g, v) >= 2.0) {
        out.push((0.9 + 0.5 * pval(g, v), 3, Some(v)));
    }
    if g.player(p).life <= 20 {
        out.push((0.3 + if g.player(p).life < 15 { 1.0 } else { 0.0 }, 2, None));
    }
    out
}

fn witherbloom_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let (cg, cp) = crate::engine::mana::cost_of(g, p, c);
    if !can_pay(g, p, cg, &cp, false) || !crate::engine::cast::castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    let label = |m: i64| match m {
        1 => "Witherbloom Charm: sacrifice a token, draw two",
        2 => "Witherbloom Charm: gain 5 life",
        _ => "Witherbloom Charm: destroy a permanent",
    };
    let eot = if post.is_none() { -0.4 } else { 0.0 };
    Ok(witherbloom_modes(g, p)
        .into_iter()
        .map(|(u, mode, t)| {
            let arg = c.0 as i64 | (mode << 32) | (t.map_or(0, |m| m.0 as i64 + 1) << 40);
            Opt { utility: u + eot, label: label(mode).into(), act: Some(Action::Plan { f: charm_go, arg }) }
        })
        .collect())
}

/// **Witherbloom Charm**: all three modes. Mode 1 sacrifices the cheapest worthless token (the AI never sacrifices a
/// real permanent for it); the mode is chosen as the spell is cast (`Ctx::x`), as for Silverquill Charm.
fn witherbloom_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &crate::state::Ctx) -> Res<Sym> {
    let (mode, target) = if (1..=3).contains(&ctx.x) {
        (ctx.x as i64, ctx.target)
    } else {
        let best = witherbloom_modes(g, p).into_iter().max_by(|a, b| a.0.total_cmp(&b.0));
        best.map_or((2, None), |(_, m, t)| (m, t))
    };
    match (mode, target) {
        (1, _) => {
            if let Some(f) = witherbloom_fodder(g, p) {
                die(g, f, "sac")?;
                draw(g, p, 2, false)?;
            }
        }
        (2, _) => crate::engine::life::gain(g, p, 5)?,
        (3, Some(t)) if g.perm(t).on_bf && g.perm(t).cd.is_none_or(|c| g.db.get(c).cmc <= 2) => {
            apply_removal(g, Some(p), t, "destroy", Some(c))?;
        }
        _ => {}
    }
    Ok(intern("gy"))
}

pub fn register_witherbloom(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if let Some(c) = card_if(r, db, WITHERBLOOM)? {
        c.resolve = Some(witherbloom_resolve);
        c.hand_options = Some(witherbloom_options);
        c.prio = Some(zero_prio);
    }
    Ok(())
}

// ================================================================== the Equipment package (Gilgamesh and friends)
pub const GILGAMESH: &str = "Gilgamesh, Master-at-Arms";
pub const LANCE: &str = "Dragoon's Lance";
pub const KATANA: &str = "Samurai's Katana";
pub const HEIRLOOM: &str = "Hero's Heirloom";
pub const FLAIL: &str = "Conqueror's Flail";
pub const PRISON: &str = "Boiling Rock Prison";

fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

/// m is a Samurai: printed (Gilgamesh) or made one by Samurai's Katana
pub fn is_samurai(g: &Game, m: PermId) -> bool {
    crate::engine::values::has_type(g, m, "samurai")
        || super::cloud::equipment_on(g, m).iter().any(|&e| name_of(g, e) == KATANA)
}

/// **Dragoon's Lance**: the equipped creature has flying during its controller's turn (+1/+0 and equip {4} are in the
/// card data; job select is `cloud::staff_etb`). The Knight type it grants matters to nothing in the game.
fn lance_flying(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "flying" && g.perm(src).attached == Some(m) && live(g, src) && g.active == Some(owner(g, src))
}

/// **Hero's Heirloom**: as long as the equipped creature is legendary, it has trample and haste (+2/+1 and equip {2}
/// are in the card data)
fn heirloom_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    (kw == "trample" || kw == "haste")
        && g.perm(src).attached == Some(m)
        && live(g, src)
        && crate::cardcode::is_legendary(g, m)
}

/// how much the AI wants a piece of Equipment put onto the battlefield and kept on a creature (best first)
fn gear_rank(name: &str) -> i32 {
    match name {
        FLAIL => 9,
        "Hero's Blade" => 8,
        "Nim Deathmantle" => 7,
        HEIRLOOM => 6,
        "Medic's Kitesail" => 5,
        "Trailblazer's Boots" => 4,
        _ => 1,
    }
}

/// **Gilgamesh, Master-at-Arms**: whenever he enters or attacks, look at the top six cards of the library; every
/// Equipment among them goes onto the battlefield (the AI always takes them all; a person is asked once, whether to
/// do it at all), the rest on the bottom in a random order; then the best of the Equipment that came in without
/// bringing its own Hero (job select pieces attach themselves to their token) is attached to a Samurai: Gilgamesh
/// himself (he is one), else a creature wearing Samurai's Katana.
fn gilgamesh_trigger(g: &mut Game, src: PermId) -> Res {
    let o = owner(g, src);
    if !live(g, src) || g.player(o).library.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "look at the top six cards for Equipment", Some(6.0))? {
        return Ok(());
    }
    let n = g.player(o).library.len().min(6);
    let top: Vec<CardId> = (0..n).map(|_| g.player_mut(o).library.pop().unwrap()).collect();
    let gear: Vec<CardId> = top.iter().copied().filter(|&c| g.db.get(c).has_subtype("equipment")).collect();
    let mut rest: Vec<CardId> = top.iter().copied().filter(|&c| !gear.contains(&c)).collect();
    g.rng.shuffle(&mut rest);
    for c in rest {
        g.player_mut(o).library.insert(0, c); // the bottom
    }
    crate::glog!(g, "    Gilgamesh looks at {n} cards: {} Equipment", gear.len());
    let mut came: Vec<PermId> = vec![];
    for c in gear {
        g.player_mut(o).seen.insert(c);
        came.push(enter(g, o, c, Enter { sick: false, ..Default::default() })?);
    }
    let own_hero = |g: &Game, e: PermId| matches!(name_of(g, e), LANCE | KATANA | "White Mage's Staff");
    let samurai: Vec<PermId> =
        g.player(o).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m) && is_samurai(g, m)).collect();
    let best =
        came.iter().copied().filter(|&e| live(g, e) && !own_hero(g, e)).max_by_key(|&e| gear_rank(name_of(g, e)));
    let to = if live(g, src) && samurai.contains(&src) { Some(src) } else { samurai.first().copied() };
    if let (Some(e), Some(t)) = (best, to) {
        super::cloud::attach(g, e, t);
    }
    Ok(())
}

fn gilgamesh_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src { gilgamesh_trigger(g, src) } else { Ok(()) }
}

fn gilgamesh_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) == p && atk.contains(&src) {
        gilgamesh_trigger(g, src)?;
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ Godo, Bandit Warlord
pub const GODO: &str = "Godo, Bandit Warlord";

/// what the Equipment card `c` is worth on Godo right now: its size, and the keywords it gives, counting haste for
/// a lot while he is summoning sick (he attacks the turn he lands, so the extra combat is used at once)
fn godo_gear_score(g: &Game, c: CardId, godo: PermId) -> f64 {
    use crate::dsl::model::Ability;
    let d = g.db.get(c);
    let sick = g.perm(godo).sick && !crate::dsl::has_kw(g, godo, "haste");
    let mut v = 0.3 * gear_rank(&d.name) as f64;
    let mut kws: Vec<&str> = vec![];
    for a in d.abilities.iter() {
        if let Ability::Static { kind, pow, tgh, keyword, .. } = a {
            if kind == "equip_bonus" {
                v += 0.5 * (pow.unwrap_or(0) + tgh.unwrap_or(0)) as f64;
            }
            if kind == "equip_keyword"
                && let Some(k) = keyword.as_deref()
            {
                kws.push(k);
            }
        }
    }
    if &*d.name == HEIRLOOM && crate::cardcode::is_legendary(g, godo) {
        kws.extend(["trample", "haste"]); // legendary: trample and haste
    }
    for k in kws {
        v += match k {
            "haste" if sick => 4.0,
            "haste" => 0.0,
            "indestructible" => 2.0,
            _ => 1.0,
        };
    }
    v
}

/// **Godo**, when he enters: you may search the library for an Equipment card, put it onto the battlefield and
/// attach it to him (a job select piece brings its Hero, but still goes on Godo), then shuffle. The AI takes the
/// Equipment that is best on him (`godo_gear_score`).
fn godo_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    let gear: Vec<CardId> =
        crate::engine::zones::searchable(g, o).into_iter().filter(|&c| g.db.get(c).has_subtype("equipment")).collect();
    if gear.is_empty() || !trigger_window(g, o, Some(src), "search for an Equipment and attach it", Some(6.0))? {
        return Ok(());
    }
    let Some(c) = max_by(&gear, |c| godo_gear_score(g, c, src)) else { return Ok(()) };
    let lib = &mut g.player_mut(o).library;
    if let Some(i) = lib.iter().position(|&x| x == c) {
        lib.remove(i);
    }
    g.player_mut(o).seen.insert(c);
    let e = enter(g, o, c, Enter { sick: false, ..Default::default() })?;
    crate::glog!(g, "    Godo fetches {} and attaches it", g.db.get(c).name);
    if live(g, e) && live(g, src) {
        super::cloud::attach(g, e, src);
    }
    let mut lib = std::mem::take(&mut g.player_mut(o).library);
    g.rng.shuffle(&mut lib);
    g.player_mut(o).library = lib;
    Ok(())
}

/// **Godo**, whenever he attacks for the first time each turn: untap him and every Samurai you control; there is an
/// additional combat phase after this one. Once per turn, so attacking again in the extra combat adds nothing.
fn godo_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !atk.contains(&src) {
        return Ok(vec![]);
    }
    let key = intern(&format!("godo_first_attack_{}", src.0));
    if !once_per_turn(g, p, key) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "untap Godo and the Samurai, additional combat", Some(6.0))? {
        return Ok(vec![]);
    }
    let team: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.is_creature(m) && (m == src || is_samurai(g, m)))
        .collect();
    for m in team {
        g.perm_mut(m).tapped = false;
    }
    g.player_mut(p).extra_combats += 1;
    crate::glog!(g, "    Godo: untap him and the Samurai, additional combat");
    Ok(vec![])
}
// ------------------------------------------------------------------ Boiling Rock Prison
/// **Boiling Rock Prison** (enters tapped, {T}: {B} or {R} are in the card data): {4}, {T}, sacrifice it: draw a card.
/// The AI does it in the end-of-turn window (so the mana would go unused), with six or more lands in play and no land
/// to play from the hand.
fn prison_options(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.land(l).tapped || !crate::impls::lands::can_pay_without(g, p, l, 4, "") {
        return Ok(vec![]);
    }
    let pl = g.player(p);
    if pl.lands.len() < 6 || pl.hand.iter().any(|&c| g.db.get(c).land) || pl.library.len() < 8 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 0.8,
        label: "Boiling Rock Prison draw".into(),
        act: Some(Action::Plan { f: prison_go, arg: l.0 as i64 }),
    }])
}

fn prison_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !g.player(p).lands.contains(&l) || g.land(l).tapped || !crate::impls::lands::pay_without(g, p, l, 4, "")? {
        return Ok(false);
    }
    let cd = g.land(l).cd;
    crate::impls::lands::sac_land(g, p, l)?;
    crate::glog!(g, "  {} sacrifices Boiling Rock Prison: draws a card", g.player(p).name);
    if crate::engine::stack::ability_window_card(g, p, cd, "draw a card", None, None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

pub fn register_gear(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if let Some(c) = card_if(r, db, GILGAMESH)? {
        c.etb = Some(gilgamesh_etb);
        c.attack = Some(gilgamesh_attack);
    }
    if let Some(c) = card_if(r, db, GODO)? {
        c.etb = Some(godo_etb);
        c.attack = Some(godo_attack);
    }
    if let Some(c) = card_if(r, db, LANCE)? {
        c.etb = Some(super::cloud::staff_etb);
        c.grant_kw = Some(lance_flying);
    }
    if let Some(c) = card_if(r, db, KATANA)? {
        c.etb = Some(super::cloud::staff_etb);
    }
    if let Some(c) = card_if(r, db, HEIRLOOM)? {
        c.grant_kw = Some(heirloom_kw);
    }
    if let Some(c) = card_if(r, db, PRISON)? {
        c.land_options = Some(prison_options);
    }
    if let Some(c) = card_if(r, db, FARMLAND)? {
        c.hand_options = Some(farmland_options);
    }
    Ok(())
}

// ================================================================== the kraum seat: where Equipment goes
/// the opponents of p that have an untapped creature able to block m, and how many of those would still have one if
/// Equipment `e` gave m evasion (None: e gives none). Counted per opponent, since Tymna draws per opponent hit.
fn blocked_by(g: &Game, p: PlayerId, e: Option<&str>, m: PermId) -> usize {
    let flying = |b: PermId| {
        let y = g.perm(b);
        y.fly || crate::dsl::has_kw(g, b, "flying") || crate::dsl::has_kw(g, b, "reach")
    };
    g.opps(p)
        .filter(|&q| g.player(q).alive)
        .filter(|&q| {
            let nonbasic = g.player(q).lands.iter().any(|&l| g.land(l).on_bf && !g.db.get(g.land(l).cd).basic);
            g.player(q).perms.iter().any(|&b| {
                let y = g.perm(b);
                if !y.on_bf || y.phased || y.tapped || !g.is_creature(b) || !crate::engine::combat::can_block(g, b, m) {
                    return false;
                }
                match e {
                    Some("Medic's Kitesail") | Some(LANCE) => flying(b),
                    Some("Trailblazer's Boots") => !nonbasic,
                    Some("Nim Deathmantle") => {
                        crate::engine::values::colors_of(g, b).has('B')
                            || g.db.get(y.cd.unwrap_or(CardId(0))).types.has(crate::cards::Types::ARTIFACT)
                    }
                    _ => true,
                }
            })
        })
        .count()
}

/// what Equipment `e` is worth on creature `m` for the kraum seat (0 for any other seat): evasion Equipment goes on a
/// creature that can't get through on its own, one point of value per opponent it newly reaches (and a loss when the
/// creature already connects, or already flies); the pieces that add size and keywords go on a creature that connects
/// anyway, since its hit is the card Tymna draws.
pub fn equip_holder_value(g: &Game, p: PlayerId, e: PermId, m: PermId) -> f64 {
    if g.player(p).key != "kraum" {
        return 0.0;
    }
    let name = name_of(g, e);
    let now = blocked_by(g, p, None, m) as f64;
    let evasion = matches!(name, "Medic's Kitesail" | LANCE | "Trailblazer's Boots" | "Nim Deathmantle");
    let mut v = 0.25 * epow(g, m) as f64;
    if evasion {
        let after = blocked_by(g, p, Some(name), m) as f64;
        v += 3.0 * (now - after);
        if now == 0.0 {
            v -= 3.0; // it connects already: the Equipment would be wasted here
        }
    } else if now == 0.0 {
        v += 2.0; // size on a creature that gets through
    }
    if name == "Shield of Kaldra" {
        // indestructible: on the creature most worth protecting (a commander, else the best threat)
        v += pval(g, m) * 0.5 + if g.perm(m).is_cmd { 3.0 } else { 0.0 };
    }
    if name == "Helm of Kaldra" {
        v += 0.5 * epow(g, m) as f64; // first strike, trample, haste: on a big attacker
    }
    if matches!(name, HEIRLOOM | "Hero's Blade") && crate::cardcode::is_legendary(g, m) {
        v += 1.0;
    }
    if g.perm(m).is_cmd && !evasion {
        v += 0.5;
    }
    v
}

/// the utility the kraum seat gives equipping `e` to `m` (added to the generic equip option's)
pub fn equip_utility_value(g: &Game, p: PlayerId, e: PermId, m: PermId) -> f64 {
    if g.player(p).key != "kraum" {
        return 0.0;
    }
    // capped so that casting a creature (priority 66 and up, utility 6.6 and up) still comes before an equip
    equip_holder_value(g, p, e, m).clamp(-3.0, 2.0)
}

/// the creature that should wear Equipment `e` (the kraum seat), if any
fn best_wearer(g: &Game, p: PlayerId, e: PermId) -> Option<PermId> {
    let cs: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m) && !g.perm(m).noatk).collect();
    max_by(&cs, |m| equip_holder_value(g, p, e, m) - if g.perm(m).sick { 0.5 } else { 0.0 })
}

/// the kraum seat's own plays (`brain::special_options`): **Conqueror's Flail** (equip {2}; it has no compiled equip
/// ability, only the tag), put on a creature before combat, so opponents can't cast spells while the team swings.
/// The Nim Deathmantle equip picks its wearer with `equip_holder_value` (in mine.rs).
pub fn kraum_special(g: &mut Game, p: PlayerId, post: bool) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if g.player(p).key != "kraum" || crate::human::is_human(g, p) || post {
        return Ok(out);
    }
    for e in g.player(p).perms.clone() {
        if !live(g, e) || name_of(g, e) != FLAIL || crate::engine::values::stopped(g, FLAIL) {
            continue;
        }
        let worn = g.perm(e).attached.filter(|&t| live(g, t) && owner(g, t) == p);
        let Some(best) = best_wearer(g, p, e) else { continue };
        if worn == Some(best)
            || (worn.is_some() && equip_holder_value(g, p, e, best) < 2.0 + equip_holder_value(g, p, e, worn.unwrap()))
        {
            continue;
        }
        if !can_pay(g, p, 2, "", false) {
            continue;
        }
        out.push(Opt {
            utility: 4.5,
            label: format!("equip {FLAIL} to {}", name_of(g, best)),
            act: Some(Action::Plan { f: flail_equip, arg: e.0 as i64 | ((best.0 as i64) << 32) }),
        });
    }
    Ok(out)
}

fn flail_equip(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let e = PermId((arg & 0xffff_ffff) as u32);
    let m = PermId((arg >> 32) as u32);
    if !live(g, e) || !live(g, m) || owner(g, m) != p || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    crate::glog!(g, "  {} equips {} to {}", g.player(p).name, g.perm(e).name, g.perm(m).name);
    crate::engine::stack::equip_to(g, p, e, m, 2)
}

// ================================================================== the kraum seat: what a tutor finds
/// the best land of `okn` for p: the colours p cannot make yet first, then Command Tower and the untapped duals
fn land_pick(g: &Game, p: PlayerId, okn: &[CardId]) -> Option<CardId> {
    let cols = |c: CardId| -> String { g.db.get(c).tags.str(Tag::C).unwrap_or("").to_string() };
    let have: String = g.player(p).lands.iter().filter(|&&l| g.land(l).on_bf).map(|&l| cols(g.land(l).cd)).collect();
    let lands: Vec<CardId> = okn.iter().copied().filter(|&c| g.db.get(c).land).collect();
    max_by(&lands, |c| {
        let cs = cols(c);
        let new = "WUBR".chars().filter(|&ch| cs.contains(ch) && !have.contains(ch)).count() as f64;
        3.0 * new + cs.len() as f64 - if g.db.get(c).tag(Tag::T) { 1.5 } else { 0.0 }
    })
}

/// Diabolic Tutor (and any tutor for any card) for the kraum seat: the card the game needs. A land when short of mana
/// (fewer than five lands in play or hand and a colour missing); a board wipe (Austere Command, Elspeth, Sunblast Angel or
/// Toxic Deluge, the first in the library) when the table's boards are far ahead;
/// when ahead, Conqueror's Flail if it isn't out yet, else the best finisher; else the evasive creature that brings
/// the next opponent into Tymna's reach, else the generic pick (None).
pub fn tutor_pick(g: &Game, p: PlayerId, kind: &str, okn: &[CardId]) -> Option<CardId> {
    if kind != "any" {
        return None;
    }
    let pl = g.player(p);
    let first = |ns: &[&str]| -> Option<CardId> { ns.iter().filter_map(|n| g.db.id(n)).find(|c| okn.contains(c)) };
    let held = |name: &str| {
        pl.hand.iter().any(|&c| &*g.db.get(c).name == name)
            || pl.perms.iter().any(|&m| live(g, m) && name_of(g, m) == name)
    };
    let lands = pl.lands.len() + pl.hand.iter().filter(|&&c| g.db.get(c).land).count();
    if lands < 5 && pl.turns <= 8 {
        return land_pick(g, p, okn);
    }
    let mine: i32 = pl.perms.iter().filter(|&&m| live(g, m) && g.is_creature(m)).map(|&m| epow(g, m)).sum();
    let theirs: i32 = g
        .opps(p)
        .filter(|&q| g.player(q).alive)
        .map(|q| {
            g.player(q).perms.iter().filter(|&&m| live(g, m) && g.is_creature(m)).map(|&m| epow(g, m)).sum::<i32>()
        })
        .max()
        .unwrap_or(0);
    const WIPES: [&str; 4] = ["Austere Command", "Elspeth, Sun's Champion", "Sunblast Angel", "Toxic Deluge"];
    if theirs >= mine + 6
        && !WIPES.iter().any(|w| held(w))
        && let Some(c) = first(&WIPES)
    {
        return Some(c);
    }
    let cres = pl.perms.iter().filter(|&&m| live(g, m) && g.is_creature(m)).count();
    if cres >= 2 && mine + 2 >= theirs {
        // a finisher (the Flail was fetched every time here and added little: it is cast 86% of the games it is drawn
        // anyway): the best one castable by next turn, else the cheapest; a Titan, Terror or Angel to close
        const FINISHERS: [&str; 10] = [
            "Grave Titan",
            "Terror of the Peaks",
            "Sun Titan",
            "Baneslayer Angel",
            "Lyra Dawnbringer",
            "Glorybringer",
            "Thundermaw Hellkite",
            GILGAMESH,
            "Sphinx of New Prahv",
            "Serra Angel",
        ];
        let reach = lands as u32 + 1;
        let fin: Vec<CardId> = FINISHERS
            .iter()
            .filter_map(|n| g.db.id(n))
            .filter(|c| okn.contains(c) && !held(&g.db.get(*c).name))
            .collect();
        if let Some(&c) = fin.iter().find(|&&c| g.db.get(c).cmc <= reach).or_else(|| fin.iter().min_by_key(|&&c| g.db.get(c).cmc)) {
            return Some(c);
        }
        if !held(FLAIL)
            && let Some(c) = first(&[FLAIL])
        {
            return Some(c);
        }
    }
    // developing: the cheapest evasive creature when fewer evasive creatures are out than opponents
    let evasive = |c: CardId| {
        let d = g.db.get(c);
        d.creature && (d.has_kw("flying") || d.tag(Tag::Fly) || d.has_kw("menace"))
    };
    let cands: Vec<CardId> = okn.iter().copied().filter(|&c| evasive(c) && !held(&g.db.get(c).name)).collect();
    if cres < 3
        && let Some(c) = max_by(&cands, |c| g.db.get(c).bomb as f64 - g.db.get(c).cmc as f64)
    {
        return Some(c);
    }
    None
}

// ------------------------------------------------------------------ Irrigated Farmland
pub const FARMLAND: &str = "Irrigated Farmland";

/// **Irrigated Farmland** (an Island Plains that enters tapped is in the card data): cycling {2}. The AI cycles it
/// when it is not needed as a land drop: five or more lands in play, or an end-of-turn window with four in play and
/// another land in hand; never to draw itself out.
fn farmland_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let pl = g.player(p);
    if post == Some(true) || !pl.hand.contains(&c) || pl.library.len() < 8 || !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    let in_play = pl.lands.len();
    let spare = pl.hand.iter().any(|&x| x != c && g.db.get(x).land);
    let ok = in_play >= 5 || (post.is_none() && in_play >= 4 && spare);
    if !ok {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: if post.is_none() { 1.4 } else { 0.5 },
        label: "Irrigated Farmland (cycling)".into(),
        act: Some(Action::Plan { f: farmland_cycle, arg: c.0 as i64 }),
    }])
}

fn farmland_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    crate::glog!(g, "  {} cycles Irrigated Farmland", g.player(p).name);
    crate::engine::zones::discard_cards(g, p, &[c])?;
    draw(g, p, 1, false)?;
    Ok(true)
}

// ================================================================== the Phyrexian-mana cards
// Dismember (-5/-5: tags `rem=shrink5 tgt=c`), Porcelain Legionnaire (3/1 first strike) and Vault Skirge (1/1 flying
// lifelink) are carried by the card data and the engine's Phyrexian-mana payment (`mana::cost_of` drops a {X/P} pip the
// mana can't pay while life is above 10, and `cast` then takes 2 life for it). Only Apostle's Blessing needs code.
pub const BLESSING: &str = "Apostle's Blessing";

/// the keyword `eot_kw` carries for protection from this colour until end of turn (read by `values::prot_colors`)
fn pro_kw(ch: char) -> Option<Sym> {
    Some(match ch {
        'W' => "pro_W",
        'U' => "pro_U",
        'B' => "pro_B",
        'R' => "pro_R",
        'G' => "pro_G",
        _ => return None,
    })
}

/// **Apostle's Blessing** ({1}{W/P} instant): target artifact or creature you control gains protection from artifacts
/// or from the colour of your choice until end of turn. The AI only casts it in response to targeted removal aimed at
/// a commander or a permanent worth 4 or more (`protect`, first thing in `ai::decks::protect_response`) and picks the
/// removal spell's own colour (its first one if it has several; "artifacts" for a colourless artifact spell). {W/P} is
/// paid with {W} when the mana is there, else with 2 life (above 6 life, since it is saving a commander or a bomb). It
/// goes through the stack, so it can be countered. The permanent keeps the protection until end of turn: damage from a
/// source of that colour (artifacts) is prevented, it can't be blocked by creatures of that colour, and later
/// targeted spells of that colour can't pick it; the removal in response is stopped. Not modeled: protection from a
/// colour against an ability (no source spell to read the colour from, so the AI doesn't cast it), and against board
/// wipes (the wipe responses are other cards').
fn blessing_protect(
    g: &mut Game,
    owner_p: PlayerId,
    m: PermId,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if actor == Some(owner_p)
        || g.humans.get(owner_p).is_some()
        || !g.perm(m).on_bf
        || g.perm(m).owner != owner_p
        || silenced(g, owner_p)
    {
        return Ok(false);
    }
    let art = g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT));
    if !g.is_creature(m) && !art {
        return Ok(false);
    }
    let Some(s) = spell else { return Ok(false) };
    let Some(c) = g.player(owner_p).hand.iter().copied().find(|&x| &*g.db.get(x).name == BLESSING) else {
        return Ok(false);
    };
    if !(g.perm(m).is_cmd || pval(g, m) >= 4.0) || !castable(g, owner_p, c, "hand") {
        return Ok(false);
    }
    // the choice: a colour of the removal spell, or artifacts when it is a colourless artifact
    let src = g.db.get(s);
    let choice = ['W', 'U', 'B', 'R', 'G'].into_iter().find(|&ch| src.colors.has(ch));
    let artifact_src = src.types.has(crate::cards::Types::ARTIFACT);
    if choice.is_none() && !artifact_src {
        return Ok(false);
    }
    // {1}{W/P}
    let by_mana = can_pay(g, owner_p, 1, "W", false);
    if !by_mana && !(g.player(owner_p).life > 6 && can_pay(g, owner_p, 1, "", false)) {
        return Ok(false);
    }
    crate::ai::decks::remove_from_hand(g, owner_p, c);
    if by_mana {
        pay(g, owner_p, 1, "W", false)?;
    } else {
        pay(g, owner_p, 1, "", false)?;
        crate::engine::life::lose_life(g, owner_p, 2, Some(owner_p), "life", None)?;
    }
    let pl = g.player_mut(owner_p);
    pl.gy.push(c);
    pl.spells_this_turn += 1;
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts {}", g.player(owner_p).name, BLESSING);
    crate::engine::cast::on_cast(g, owner_p, c)?;
    if !crate::engine::stack::counter_window(g, owner_p, c, 6.0, vec![])? || !g.perm(m).on_bf {
        return Ok(false);
    }
    match choice {
        Some(ch) => eot_kw(g, m, pro_kw(ch).unwrap()),
        None => eot_kw(g, m, intern("pro_artifacts")),
    }
    crate::glog!(
        g,
        "    {} gains protection from {}",
        g.perm(m).name,
        choice.map_or("artifacts".into(), |c| c.to_string())
    );
    g.player_mut(owner_p).stat("protection_used", 1);
    Ok(true)
}

/// protection from artifacts until end of turn (Apostle's Blessing): `m` can't be dealt damage by artifact creature
/// `other` (called from `rules2::prot_vs`)
pub fn pro_artifacts_vs(g: &Game, m: PermId, other: PermId) -> bool {
    g.perm(m).eot_kw.contains(&"pro_artifacts")
        && g.perm(other).cd.is_some_and(|c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT))
}

pub fn register_phyrexian(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if let Some(c) = card_if(r, db, BLESSING)? {
        c.prio = Some(zero_prio); // only ever cast in response to removal (`protect`)
    }
    Ok(())
}

// ================================================================== Archfiend, Skrelv, Gut Shot, White Sun's Twilight, Unlife
pub const ARCHFIEND: &str = "Archfiend of the Dross";
pub const SKRELV: &str = "Skrelv, Defector Mite";
pub const GUT_SHOT: &str = "Gut Shot";
pub const TWILIGHT: &str = "White Sun's Twilight";
pub const UNLIFE: &str = "Phyrexian Unlife";
const RESTO: &str = "Restoration Angel";

fn oil_of(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Oil)
}

/// **Archfiend of the Dross** (6/6 flier; the compiled card has the "whenever a creature an opponent controls dies, its
/// controller loses 2 life" trigger): it enters with four oil counters (`as_enters`, so a blinked Archfiend is a new
/// object with four again). At the beginning of its controller's upkeep (`archfiend_upkeep`) one comes off and, when
/// none are left, that player loses the game, for real (`lose_game`; Herald of Eternal Dawn stops it). With one
/// counter left (the upkeep that would end the game) the AI first tries to get the Archfiend out of the way: Restoration
/// Angel from the hand (flash, {3}{W}) blinks it, so the trigger finds a new object and does nothing. If nothing can
/// save it, the game is lost, as on the card. Stats: `archfiend_lost`, `archfiend_saved`. Not modeled: saving it with
/// a sacrifice or bounce effect (the deck has none), and holding Restoration Angel mana up a turn early.
fn archfiend_enters(g: &mut Game, _p: PlayerId, m: PermId) -> Res {
    g.perm_mut(m).data.set(DataKey::Oil, Val::Int(4));
    Ok(())
}

fn lose_game(g: &mut Game, p: PlayerId, why: &str) -> Res {
    if crate::impls::t0_giada::cant_lose(g, p) || !g.player(p).alive {
        return Ok(());
    }
    crate::glog!(g, "  {} loses the game ({})", g.player(p).name, why);
    let pl = g.player_mut(p);
    pl.last_kind = intern(why);
    pl.last_src = None;
    crate::engine::life::eliminate(g, p);
    crate::engine::life::check_state(g)
}

fn archfiend_save(g: &mut Game, src: PermId, p: PlayerId) -> Res<bool> {
    if g.humans.get(p).is_some() || silenced(g, p) {
        return Ok(false);
    }
    let Some(c) = g.player(p).hand.iter().copied().find(|&x| &*g.db.get(x).name == RESTO) else {
        return Ok(false);
    };
    if !can_pay(g, p, 3, "W", false) {
        return Ok(false);
    }
    pay(g, p, 3, "W", false)?;
    crate::glog!(g, "  {} casts {} in response to the oil counter trigger", g.player(p).name, RESTO);
    g.resto_target = Some(src);
    let r = crate::engine::cast::cast_card(g, p, c, "hand", crate::state::Ctx::default());
    g.resto_target = None;
    r?;
    let saved = !g.perm(src).on_bf;
    if saved {
        g.player_mut(p).stat("archfiend_saved", 1);
    }
    Ok(saved)
}

fn archfiend_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if owner(g, src) != p || !g.perm(src).on_bf || g.perm(src).phased {
        return Ok(());
    }
    if oil_of(g, src) <= 1 && archfiend_save(g, src, p)? {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "remove an oil counter", Some(5.0))? || !g.perm(src).on_bf {
        return Ok(());
    }
    let n = (oil_of(g, src) - 1).max(0);
    g.perm_mut(src).data.set(DataKey::Oil, Val::Int(n));
    crate::glog!(g, "    Archfiend of the Dross: {} oil counter(s) left", n);
    if n == 0 {
        g.player_mut(p).stat("archfiend_lost", 1);
        lose_game(g, p, "Archfiend of the Dross")?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Skrelv, Defector Mite
/// the `eot_kw` keywords for hexproof from this colour and "can't be blocked by creatures of this colour"
fn skrelv_kws(ch: char) -> Option<(Sym, Sym)> {
    Some(match ch {
        'W' => ("hx_W", "nb_W"),
        'U' => ("hx_U", "nb_U"),
        'B' => ("hx_B", "nb_B"),
        'R' => ("hx_R", "nb_R"),
        'G' => ("hx_G", "nb_G"),
        _ => return None,
    })
}

/// toxic from `eot_kw` (Skrelv's ability), a Mite token's keyword list, or Skrelv himself: the poison counters the
/// combat damage to a player gives (called from `engine/combat.rs`; the older toxic cards have their own hooks)
pub fn toxic_hit(g: &mut Game, a: PermId, d: PlayerId) {
    let x = g.perm(a);
    let mut k = i32::from(x.eot_kw.contains(&"toxic"));
    match x.cd {
        None => {
            if let Some(Val::List(v)) = x.data.get(DataKey::Kws)
                && v.iter().any(|w| matches!(w, Val::Str(s) if *s == "toxic"))
            {
                k += 1;
            }
        }
        Some(_) => k += i32::from(name_of(g, a) == SKRELV && !x.neutered),
    }
    if k == 0 || crate::engine::values::melira(g, d) {
        return;
    }
    let k = crate::cardcode::vorinclex(g, g.perm(a).owner, k);
    g.player_mut(d).poison += k as u32;
    crate::glog!(g, "    {} gets {} poison counter(s) (toxic)", g.player(d).name, k);
}

/// hexproof from a colour (Skrelv's ability): `m` can't be targeted by a source with one of these colours
pub fn hexproof_from(g: &Game, m: PermId, source: Colors) -> bool {
    [('W', "hx_W"), ('U', "hx_U"), ('B', "hx_B"), ('R', "hx_R"), ('G', "hx_G")]
        .into_iter()
        .any(|(ch, kw)| source.has(ch) && g.perm(m).eot_kw.contains(&kw))
}

/// can't be blocked by creatures of a colour (Skrelv's ability): `b` may not block `a`
pub fn colour_unblockable(g: &Game, b: PermId, a: PermId) -> bool {
    let xa = g.perm(a);
    !xa.eot_kw.is_empty()
        && ['W', 'U', 'B', 'R', 'G'].into_iter().any(|ch| {
            skrelv_kws(ch).is_some_and(|(_, nb)| xa.eot_kw.contains(&nb))
                && crate::engine::values::colors_of(g, b).has(ch)
        })
}

fn skrelv_ready(g: &Game, s: PermId, p: PlayerId) -> bool {
    let x = g.perm(s);
    x.on_bf && x.owner == p && !x.tapped && !x.sick && !x.phased && !x.neutered && !silenced(g, p)
}

/// can p pay {W/P} for Skrelv: {W}, or 2 life above `floor`
fn skrelv_can_pay(g: &Game, p: PlayerId, floor: i32) -> bool {
    can_pay(g, p, 0, "W", false) || g.player(p).life > floor
}

/// the ability: pay {W/P} and tap Skrelv; `target` (another creature of p's) gets toxic 1 and hexproof from colour
/// `ch` until end of turn, and can't be blocked by creatures of that colour. Returns false when it isn't legal.
fn skrelv_activate(g: &mut Game, s: PermId, p: PlayerId, target: PermId, ch: char, floor: i32) -> Res<bool> {
    let Some((hx, nb)) = skrelv_kws(ch) else { return Ok(false) };
    if !skrelv_ready(g, s, p)
        || target == s
        || !g.perm(target).on_bf
        || g.perm(target).owner != p
        || !g.is_creature(target)
        || !skrelv_can_pay(g, p, floor)
    {
        return Ok(false);
    }
    if can_pay(g, p, 0, "W", false) {
        pay(g, p, 0, "W", false)?;
    } else {
        crate::engine::life::lose_life(g, p, 2, Some(p), "life", None)?;
    }
    g.perm_mut(s).tapped = true;
    crate::glog!(
        g,
        "  {} uses Skrelv: {} gains toxic 1 and hexproof from {}",
        g.player(p).name,
        g.perm(target).name,
        ch
    );
    if !ability_window(g, p, Some(s), "toxic 1 and hexproof from a colour", Some(3.0), Some(target))?
        || !g.perm(target).on_bf
    {
        return Ok(true); // countered; the cost is paid
    }
    eot_kw(g, target, "toxic");
    eot_kw(g, target, hx);
    eot_kw(g, target, nb);
    g.player_mut(p).stat("skrelv_used", 1);
    Ok(true)
}

/// Skrelv in response to targeted removal (`protect`, from a spell with a colour): hexproof from that colour on the
/// commander or a creature worth 4 or more fizzles the spell. Paid with {W} or 2 life (above 6).
fn skrelv_protect(
    g: &mut Game,
    owner_p: PlayerId,
    m: PermId,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if actor == Some(owner_p) || g.humans.get(owner_p).is_some() || !g.perm(m).on_bf || g.perm(m).owner != owner_p {
        return Ok(false);
    }
    let Some(s) = spell else { return Ok(false) };
    if !g.is_creature(m) || !(g.perm(m).is_cmd || pval(g, m) >= 4.0) {
        return Ok(false);
    }
    let cols = g.db.get(s).colors;
    let Some(ch) = ['W', 'U', 'B', 'R', 'G'].into_iter().find(|&c| cols.has(c)) else { return Ok(false) };
    let Some(sk) = on_bf_named(g, owner_p, SKRELV).into_iter().find(|&k| k != m && skrelv_ready(g, k, owner_p)) else {
        return Ok(false);
    };
    if !skrelv_activate(g, sk, owner_p, m, ch, 6)? {
        return Ok(false);
    }
    g.player_mut(owner_p).stat("skrelv_protected", 1);
    Ok(hexproof_from(g, m, cols))
}

/// the AI's other use, before combat: make the best attacker unblockable by the colour whose blockers matter most
/// (post = Some(false), main phase 1). Needs a real blocker of that colour; paid with {W}, or life only above 20.
fn skrelv_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || !skrelv_ready(g, src, p) || !skrelv_can_pay(g, p, 20) {
        return Ok(vec![]);
    }
    let best = max_by(
        &g.player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| {
                m != src
                    && g.is_creature(m)
                    && !g.perm(m).sick
                    && !g.perm(m).tapped
                    && !g.perm(m).phased
                    && !g.perm(m).noatk
                    && epow(g, m) >= 2
            })
            .collect::<Vec<_>>(),
        |m| pval(g, m),
    );
    let Some(a) = best else { return Ok(vec![]) };
    // the opposing blockers that could block `a` now, by colour
    let (mut bestc, mut bestv) = (None, 0.0);
    for ch in ['W', 'U', 'B', 'R', 'G'] {
        let v: f64 = g
            .opps(p)
            .flat_map(|q| g.player(q).perms.iter().copied().collect::<Vec<_>>())
            .filter(|&b| {
                g.is_creature(b)
                    && !g.perm(b).tapped
                    && crate::engine::values::colors_of(g, b).has(ch)
                    && crate::engine::combat::can_block(g, b, a)
            })
            .map(|b| pval(g, b))
            .sum();
        if v > bestv {
            (bestc, bestv) = (Some(ch), v);
        }
    }
    let Some(ch) = bestc.filter(|_| bestv >= 2.0) else { return Ok(vec![]) };
    let arg = ((a.0 as i64) << 8) | ch as i64;
    Ok(vec![ability(
        0.9 + 0.1 * bestv.min(10.0),
        &format!("Skrelv: {} unblockable by {ch} creatures", g.perm(a).name),
        src,
        skrelv_go,
        arg,
    )])
}

fn skrelv_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (a, ch) = (PermId((arg >> 8) as u32), (arg & 0xff) as u8 as char);
    skrelv_activate(g, src, p, a, ch, 20)
}

// ------------------------------------------------------------------ Gut Shot
/// **Gut Shot** ({R/P}: 1 damage to any target, compiled; {R/P} is paid with {R}, or 2 life above 10 life by the
/// engine's Phyrexian payment). The AI kills a creature with toughness 1 that is worth a card (an elf or other mana
/// creature, a token that attacks, anything pval 0.9 or more, which an elf is) at any time it has the mana, and otherwise holds it.
fn gut_shot_options(g: &mut Game, c: CardId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    let (gn, pips) = crate::engine::mana::cost_of(g, p, c);
    if !can_pay(g, p, gn, &pips, false) {
        return Ok(vec![]);
    }
    let tg: Vec<PermId> = crate::engine::removal::legal_targets(g, p, "dmg1", "c", false, Some(c))
        .into_iter()
        .filter(|&m| g.is_creature(m) && etgh(g, m) <= 1 && !g.perm(m).phased)
        .collect();
    let Some(best) = max_by(&tg, |m| pval(g, m)) else { return Ok(vec![]) };
    let v = pval(g, best);
    if v < 0.9 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.0 + 0.5 * (v - 1.0),
        label: format!("Gut Shot -> {}", g.perm(best).name),
        act: Some(Action::Removal { card: c, targets: vec![best], kick: 0, kind: None }),
    }])
}

// ------------------------------------------------------------------ White Sun's Twilight
/// **White Sun's Twilight** ({X}{W}{W} sorcery): you gain X life and create X 1/1 colourless Phyrexian Mite artifact
/// creature tokens with toxic 1 and "can't block" (a keyword list on the token); with X 5 or more, destroy all other
/// creatures (the Mites too, as the card does: "other" means other than the spell). The generic cast (X = 0) is
/// switched off (`prio` 0); `twilight_options` casts it: a wipe at X 5+ when the opponents lose clearly more board
/// value than we do (`wipe_eval`), else X up to 4 (never a wipe by accident) for life and bodies when X is 3 or more.
fn twilight_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &crate::state::Ctx) -> Res<Sym> {
    g.dsl_on = true; // the Mites' "can't block" is read through the keyword checks
    let x = ctx.x.max(0);
    if x > 0 {
        crate::engine::life::gain(g, p, x)?;
        let mut data = crate::state::PermData::default();
        data.set(DataKey::Kws, Val::List(vec![Val::Str("toxic"), Val::Str("cant_block")]));
        let spec = Tokens {
            tgh: Some(1),
            color: Some(Colors::NONE),
            types: vec!["phyrexian", "mite", "artifact"],
            data,
            ..Tokens::new(x as u32, 1)
        };
        make_tokens(g, p, spec)?;
    }
    if x >= 5 {
        crate::engine::removal::apply_wipe(g, p, "destroy", Some(c), ctx)?;
    }
    Ok("gy")
}

fn twilight_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) {
        return Ok(vec![]);
    }
    let x_max = crate::engine::mana::total_mana(g, p, false) as i32 - 2;
    if x_max < 3 {
        return Ok(vec![]);
    }
    let (ol, ml, _) = crate::ai::decks::wipe_eval(g, p, "destroy");
    let (x, u) = if x_max >= 5 && ol > 1.2 * ml + 4.0 {
        (x_max.min(12), 2.0 + ((ol - ml) / 3.0).min(6.0))
    } else {
        let x = x_max.min(4);
        (x, 0.6 + 0.2 * x as f64)
    };
    if !can_pay(g, p, x as u32, "WW", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: u,
        label: format!("White Sun's Twilight X={x}"),
        act: Some(Action::Plan { f: twilight_go, arg: c.0 as i64 * 64 + x as i64 }),
    }])
}

fn twilight_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, x) = (CardId((arg / 64) as u16), (arg % 64) as u32);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, x, "WW", false) {
        return Ok(false);
    }
    pay(g, p, x, "WW", false)?;
    crate::engine::cast::cast_card(g, p, c, "hand", crate::state::Ctx { x: x as i32, ..Default::default() })?;
    Ok(true)
}

// ------------------------------------------------------------------ Phyrexian Unlife
/// **Phyrexian Unlife**: p doesn't lose the game for having 0 or less life (`engine/life.rs` check_state), and while at
/// 0 or less, damage to p is dealt as poison counters instead of life loss (`engine/life.rs` lose_life; ten poison
/// counters still lose). Life loss that isn't damage (Archfiend, Tymna's payment) still lowers the life total. The AI
/// casts it once, when its life is 20 or less (`unlife_prio`).
pub fn unlife(g: &Game, p: PlayerId) -> bool {
    g.player(p)
        .perms
        .iter()
        .any(|&m| g.perm(m).on_bf && !g.perm(m).phased && !g.perm(m).neutered && name_of(g, m) == UNLIFE)
}

/// damage `n` to p while p has 0 or less life under Phyrexian Unlife: poison counters instead; true if it was taken
pub fn unlife_infect(g: &mut Game, p: PlayerId, n: i32) -> bool {
    if g.player(p).life > 0 || !unlife(g, p) {
        return false;
    }
    if !crate::engine::values::melira(g, p) {
        g.player_mut(p).poison += n as u32;
        crate::glog!(g, "    {} gets {} poison counters (Phyrexian Unlife)", g.player(p).name, n);
    }
    true
}

fn unlife_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if unlife(g, p) || g.player(p).life > 20 { 0 } else { 40 }
}

pub fn register_phyrexian2(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if let Some(c) = card_if(r, db, ARCHFIEND)? {
        c.as_enters = Some(archfiend_enters);
        c.upkeep = Some(archfiend_upkeep);
        c.dies = Some(archfiend_dies);
    }
    if let Some(c) = card_if(r, db, SKRELV)? {
        c.options = Some(skrelv_options);
    }
    if let Some(c) = card_if(r, db, GUT_SHOT)? {
        c.hand_options = Some(gut_shot_options);
        c.prio = Some(zero_prio); // never a plain cast (the compiled spell would pick a face); `gut_shot_options` shoots
    }
    if let Some(c) = card_if(r, db, TWILIGHT)? {
        c.resolve = Some(twilight_resolve);
        c.hand_options = Some(twilight_options);
        c.prio = Some(zero_prio);
    }
    if let Some(c) = card_if(r, db, UNLIFE)? {
        c.prio = Some(unlife_prio);
    }
    Ok(())
}

/// whenever a creature an opponent controls dies, its controller loses 2 life (the compiled trigger resolved "its
/// controller" to nobody, so the card's override has no abilities and this hook does it)
fn archfiend_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, src);
    let q = owner(g, m);
    if q == o
        || !g.perm(src).on_bf
        || g.perm(src).phased
        || g.perm(src).neutered
        || !g.is_creature(m)
        || !g.player(q).alive
    {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "its controller loses 2 life", Some(2.0))? {
        crate::engine::life::lose_life(g, q, 2, Some(o), "drain", Some(false))?;
    }
    Ok(())
}

// ================================================================== the kraum seat: Dark Endurance
/// **Dark Endurance** ({1}{B} instant, {1} less when it targets a blocking creature): target creature gets +2/+0 and
/// indestructible until end of turn. The kraum seat casts it in response to destroy or damage removal aimed at a
/// commander or a permanent worth 4 or more (Tymna is the most-removed card of the deck); exile, -X/-X and sacrifice
/// are not stopped by indestructible, so it isn't cast against those. Never for another seat.
fn endurance_protect(
    g: &mut Game,
    owner_p: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
) -> Res<bool> {
    const ENDURANCE: &str = "Dark Endurance";
    if g.player(owner_p).key != "kraum"
        || actor.is_none()
        || actor == Some(owner_p)
        || !(kind == "destroy" || kind.starts_with("dmg"))
        || !g.perm(m).on_bf
        || g.perm(m).owner != owner_p
        || !g.is_creature(m)
        || silenced(g, owner_p)
        || crate::engine::values::indestructible(g, m)
    {
        return Ok(false);
    }
    let Some(c) = g.player(owner_p).hand.iter().copied().find(|&x| &*g.db.get(x).name == ENDURANCE) else {
        return Ok(false);
    };
    if !(g.perm(m).is_cmd || pval(g, m) >= 4.0) || !castable(g, owner_p, c, "hand") || !can_pay(g, owner_p, 1, "B", false) {
        return Ok(false);
    }
    crate::ai::decks::remove_from_hand(g, owner_p, c);
    pay(g, owner_p, 1, "B", false)?;
    let pl = g.player_mut(owner_p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts {} on {}", g.player(owner_p).name, ENDURANCE, g.perm(m).name);
    crate::engine::cast::on_cast(g, owner_p, c)?;
    let ok = crate::engine::stack::counter_window(g, owner_p, c, 6.0, vec![])?;
    g.player_mut(owner_p).gy.push(c);
    if ok && g.perm(m).on_bf {
        eot_kw(g, m, intern("indestructible"));
        g.perm_mut(m).eot_pt.0 += 2;
    }
    Ok(ok && g.perm(m).on_bf)
}

// ================================================================== Firebird, Blazing Ranger and Lightning, Security Sergeant
/// **Firebird**: whenever it attacks, the other attacking creatures get +X/+0 until end of turn, X = its power
fn firebird_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p != owner(g, src) || !atk.contains(&src) || !g.perm(src).on_bf {
        return Ok(vec![]);
    }
    let x = epow(g, src);
    if x > 0 && trigger_window(g, p, Some(src), "other attackers get +X/+0", None)? {
        for &a in atk {
            if a != src && g.perm(a).on_bf && owner(g, a) == p {
                g.perm_mut(a).eot_pt.0 += x;
            }
        }
    }
    Ok(vec![])
}

/// the `impulse_long` mark of the cards Lightning exiled: far beyond any turn count, one per Lightning
const LIGHTNING_MARK: u32 = 2_000_000;

/// **Lightning, Security Sergeant** (menace): whenever it deals combat damage to a player, exile the top card of your
/// library; you may play it for as long as you control Lightning (it waits in the hand, marked in `impulse_long`)
fn lightning_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if a != src || p != owner(g, src) || d == p || dmg <= 0 || !g.perm(src).on_bf || g.player(p).library.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "exile the top card", None)? {
        return Ok(());
    }
    let Some(top) = g.player_mut(p).library.pop() else { return Ok(()) };
    crate::glog!(g, "    Lightning exiles {} (playable while Lightning stays)", g.db.get(top).name);
    let pl = g.player_mut(p);
    pl.seen.insert(top);
    pl.hand.push(top);
    pl.impulse_long.push((top, LIGHTNING_MARK + src.0 as u32));
    Ok(())
}

/// Lightning leaves the battlefield: the cards it exiled and the owner hasn't played are exiled for good
fn lightning_leaves(g: &mut Game, src: Src, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let p = g.perm(src).owner;
    let mark = LIGHTNING_MARK + src.0 as u32;
    let mine: Vec<CardId> = g.player(p).impulse_long.iter().filter(|x| x.1 == mark).map(|x| x.0).collect();
    for c in mine {
        let pl = g.player_mut(p);
        pl.impulse_long.retain(|x| x.0 != c);
        if let Some(i) = pl.hand.iter().position(|&x| x == c) {
            pl.hand.remove(i);
            pl.exile.push(c);
            crate::glog!(g, "    {} is exiled for good (Lightning left)", g.db.get(c).name);
        }
    }
    Ok(())
}

pub fn register_audit_ysh(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if let Some(c) = card_if(r, db, "Firebird, Blazing Ranger")? {
        c.attack = Some(firebird_attack);
    }
    if let Some(c) = card_if(r, db, "Lightning, Security Sergeant")? {
        c.combat_damage = Some(lightning_damage);
        c.leaves = Some(lightning_leaves);
    }
    Ok(())
}
