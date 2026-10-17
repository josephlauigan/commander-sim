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
    let (u, when) = match post {
        None => (3.0, "eot"),
        Some(true) if g.active == Some(p) => (-2.0, "main2"),
        _ => return Ok(vec![]),
    };
    if crate::engine::values::once_per_turn(g, p, crate::sym::intern(&format!("ty_th_f_{when}"))) {
        g.player_mut(p).stat(crate::sym::intern(&format!("ty_th_turns_{when}")), 1);
    }
    if !can_pay(g, p, 4, "", false) || g.player(p).library.len() < 3 {
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
        if crate::engine::mana::total_mana(g, p, false) < 4 + keep {
            return Ok(vec![]);
        }
    }
    if crate::engine::values::once_per_turn(g, p, crate::sym::intern(&format!("ty_th_g_{when}"))) {
        g.player_mut(p).stat(crate::sym::intern(&format!("ty_th_spare4_{when}")), 1);
    }
    Ok(vec![ability(u, "Thrasios: scry 1, a land or a card", src, thrasios_go, 0)])
}

fn thrasios_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !g.perm(src).on_bf || !can_pay(g, p, 4, "", false) || g.player(p).library.len() < 3 {
        return Ok(false);
    }
    g.player_mut(p).stat(crate::sym::intern("ty_thrasios_acts"), 1);
    pay(g, p, 4, "", false)?;
    if !ability_window(g, p, Some(src), "scry 1, then a land or a card", None, None)? {
        return Ok(true);
    }
    crate::cardcode::scry(g, p, 1, false)?;
    let Some(&top) = g.player(p).library.last() else { return Ok(true) };
    if g.db.get(top).land {
        crate::glog!(g, "    Thrasios reveals {}: onto the battlefield tapped", g.db.get(top).name);
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
        if !held(FLAIL)
            && let Some(c) = first(&[FLAIL])
        {
            return Some(c);
        }
        if let Some(c) = first(&["Terror of the Peaks", "Lyra Dawnbringer", GILGAMESH, "Sphinx of New Prahv"])
            .filter(|&c| !held(&g.db.get(c).name))
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
