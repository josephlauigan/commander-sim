//! Tier 6 batch `green2`: the green and small utility cards of the Rograkh & Thrasios and Kinnan decks
//! (rust/agents/archive/t6-batches.md). Cards only: no engine file is touched; what needs the engine is listed in
//! rust/agents/engine-needs-green2.md.
//!
//! - Chatterstorm: the Squirrel token is the compiled spell; storm is `t0_stella::storm_copies` on the cast.
//! - Cloud of Faeries: ETB untaps two lands; cycling {2} is a hand option.
//! - Noxious Revival: resolve puts the best of the caster's own graveyard cards on top of its library.
//! - Sowing Mycospawn: the cast trigger fetches a land; the kicked exile is `mycospawn_kicked` (needs engine kicker).
//! - Endurance: flash is the card's tag; the enters effect and evoke (exile a green card, sacrificed) are here.
//! - Enduring Vitality: every creature taps for any colour (extra_mana); dies as a creature: back as an enchantment.
//! - Shifting Woodland: enters tapped unless you control a Forest and {T}: {G} are data; the delirium copy is a land option.
//! - Invasion of Ikoria: the enters search (library and/or graveyard, non-Human, X) is the resolve; battle rules are an engine need.
//! - The Cabbage Merchant: Food on opposing noncreature casts, Food sacrificed on combat damage to you, two Foods tap for
//!   one mana (the tapped count lives on the Merchant, DataKey::Burden, reset at its controller's upkeep).

use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::on_cast;
use crate::engine::mana::{Source, Unit, cost_of, land_colors_now, total_mana};
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::turn::land_enters_tapped;
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::zones::{
    Enter, die, discard_cards, draw, enter, enter_token_copy, make_artifact_tokens, max_by, min_by,
};
use crate::engine::zones::{landfall, searchable};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::impls::common::sac_food;
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn remove_card(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

// ================================================================== Chatterstorm
/// Storm: when you cast it, copy it for each spell cast before it this turn
fn chatterstorm_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    crate::impls::t0_stella::storm_copies(g, p, c)
}

// ================================================================== Cloud of Faeries
/// When this creature enters, untap up to two lands: the AI untaps two tapped lands of its own (it always takes the
/// full two when it has them).
fn cloud_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || !trigger_window(g, o, Some(src), "untap up to two lands", Some(3.0))? {
        return Ok(());
    }
    let tapped: Vec<_> = g.player(o).lands.iter().copied().filter(|&l| g.land(l).tapped).take(2).collect();
    for l in &tapped {
        g.land_mut(*l).tapped = false;
    }
    crate::glog!(g, "    {} untaps {} land(s) (Cloud of Faeries)", pname(g, o), tapped.len());
    Ok(())
}

/// Cycling {2}: the AI cycles it in an opponent's end step or after combat when it has the two mana to spare.
fn cloud_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !g.player(p).hand.contains(&c) || !(post.is_none() && g.active != Some(p) || post == Some(true)) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 0.9,
        label: "cycle Cloud of Faeries".into(),
        act: Some(Action::Plan { f: cloud_cycle, arg: c.0 as i64 }),
    }])
}

fn cloud_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    crate::glog!(g, "  {} cycles Cloud of Faeries", pname(g, p));
    discard_cards(g, p, &[c])?;
    draw(g, p, 1, false)?;
    Ok(true)
}

// ================================================================== Noxious Revival
/// the nonland card in one of p's own graveyard worth putting back on top: the best by `card_worth`, a creature
/// counting a little more and mana value breaking ties (Oboro, Eternal Witness); None if there is none. Never another
/// Noxious Revival: its value is this same pick, so valuing it here would recurse without end
pub fn revival_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let gy: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| !g.db.get(c).land && &*g.db.get(c).name != "Noxious Revival")
        .collect();
    max_by(&gy, |c| {
        let d = g.db.get(c);
        card_worth(g, p, c, true) + if d.creature { 5.0 } else { 0.0 } + 0.1 * d.cmc as f64
    })
}

/// {G/P}, instant: put target card from a graveyard on top of its owner's library. The AI aims it at its own best
/// card (the engine's Phyrexian mana lets it pay 2 life); with nothing worth it, it fizzles to the graveyard.
fn revival_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = revival_pick(g, p) {
        let pl = g.player_mut(p);
        remove_card(&mut pl.gy, t);
        pl.library.push(t);
        crate::glog!(g, "    {} puts {} on top of its library (Noxious Revival)", pname(g, p), cname(g, t));
    }
    Ok(intern("gy"))
}

/// cast only to set up a draw: when the graveyard holds a card worth drawing
fn revival_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if revival_pick(g, p).is_some() { 30 } else { 0 }
}

// ================================================================== Sowing Mycospawn
/// the land card the AI fetches: Gaea's Cradle once it has two creatures, else the land adding the most colours it
/// lacks (then the one making the most colours), else the first land
pub fn mycospawn_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let lands: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| g.db.get(c).land).collect();
    // the deck's wish list first (Gaea's Cradle, then Talon Gates once Cradle is out)
    if crate::impls::t6_plans::is_t6(g, p)
        && let Some(&w) = crate::ai::plans::wish_list(g, p).iter().find(|w| lands.contains(w))
    {
        return Some(w);
    }
    let creatures = g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.is_creature(m)).count();
    let have = g.player(p).lands.iter().fold(Colors::NONE, |a, &l| a.union(land_colors_now(g, p, l)));
    max_by(&lands, |c| {
        let d = g.db.get(c);
        let cols = match d.tags.str(Tag::C).unwrap_or("") {
            "A" => g.player(p).ident,
            "C" | "" => Colors::NONE,
            s => Colors::from_letters(s),
        };
        let new_cols = "WUBRG".chars().filter(|&ch| cols.has(ch) && !have.has(ch)).count() as f64;
        let cradle = if &*d.name == "Gaea's Cradle" && creatures >= 2 { 2000.0 } else { 0.0 };
        cradle + new_cols * 100.0 + cols.count() as f64
    })
}

/// When you cast this spell, search your library for a land card, put it onto the battlefield, then shuffle. This is a
/// cast trigger, so it happens even if the spell is countered. The land enters untapped unless it says otherwise.
fn mycospawn_cast(g: &mut Game, p: PlayerId, _c: CardId) -> Res {
    if !trigger_window(g, p, None, "Sowing Mycospawn: search for a land", Some(3.0))? {
        return Ok(());
    }
    if let Some(c) = mycospawn_pick(g, p) {
        let lib = &mut g.player_mut(p).library;
        if let Some(i) = lib.iter().position(|&x| x == c) {
            lib.remove(i);
        }
        let tapped = land_enters_tapped(g, p, c);
        g.add_land(p, c, tapped);
        crate::glog!(g, "    {} fetches {} (Sowing Mycospawn)", pname(g, p), cname(g, c));
        shuffle_library(g, p);
        landfall(g, p)?;
    } else {
        shuffle_library(g, p);
    }
    Ok(())
}

/// The kicked half of the cast trigger: exile target land (the AI takes the best opposing land). Nothing calls this
/// yet: the engine has no kicker (additional cost chosen as a spell is cast, with a flag the cast trigger can read);
/// see rust/agents/engine-needs-green2.md.
pub fn mycospawn_kicked(g: &mut Game, p: PlayerId) -> Res {
    let Some((q, l)) = crate::impls::fixes::best_opp_land(g, p) else { return Ok(()) };
    if !trigger_window(g, p, None, "Sowing Mycospawn: exile target land", Some(4.0))? {
        return Ok(());
    }
    let cd = g.land(l).card();
    crate::engine::turn::remove_land(g, q, l);
    g.player_mut(q).exile.push(cd);
    crate::glog!(g, "    {} is exiled (Sowing Mycospawn, kicked)", cname(g, cd));
    Ok(())
}

// ================================================================== Endurance
const ENDURANCE: &str = "Endurance";

/// whose graveyard the AI puts on the bottom of the library: its own with Faerie Mastermind out (the decking line
/// wants a longer library to refill), else the opponent with the biggest graveyard of at least four cards (it
/// shuts off their recursion and delve); None otherwise
pub fn endurance_target(g: &Game, p: PlayerId) -> Option<PlayerId> {
    let mastermind = g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == "Faerie Mastermind");
    if mastermind && !g.player(p).gy.is_empty() {
        return Some(p);
    }
    let opps: Vec<PlayerId> = g.opps(p).filter(|&q| g.player(q).alive && g.player(q).gy.len() >= 4).collect();
    max_by(&opps, |q| g.player(q).gy.len() as f64)
}

/// Flash, reach. When this creature enters, up to one target player puts all the cards from their graveyard on the
/// bottom of their library in a random order.
fn endurance_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src {
        return Ok(());
    }
    let Some(q) = endurance_target(g, o) else { return Ok(()) };
    if !trigger_window(g, o, Some(src), "a graveyard goes to the bottom of its library", Some(3.0))? {
        return Ok(());
    }
    let mut gy = std::mem::take(&mut g.player_mut(q).gy);
    g.rng.shuffle(&mut gy);
    crate::glog!(
        g,
        "    {}'s graveyard ({} cards) goes to the bottom of its library (Endurance)",
        pname(g, q),
        gy.len()
    );
    g.player_mut(q).library.splice(0..0, gy);
    Ok(())
}

/// Evoke: exile a green card from hand (free, instant speed) when the full cost isn't payable and the enters effect
/// is wanted; the Elemental is sacrificed as it enters.
fn endurance_evoke(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let (gn, pips) = cost_of(g, p, c);
    if post.is_some() && can_pay(g, p, gn, &pips, false) {
        return Ok(vec![]);
    }
    if !g.player(p).hand.contains(&c) || endurance_target(g, p).is_none() || evoke_pitch(g, p, c).is_none() {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.5,
        label: "evoke Endurance".into(),
        act: Some(Action::Plan { f: endurance_evoke_go, arg: c.0 as i64 }),
    }])
}

/// the green card the AI exiles: the least wanted other green card in hand
fn evoke_pitch(g: &Game, p: PlayerId, c: CardId) -> Option<CardId> {
    let greens: Vec<CardId> =
        g.player(p).hand.iter().copied().filter(|&x| x != c && g.db.get(x).pips.contains('G')).collect();
    min_by(&greens, |x| card_worth(g, p, x, false))
}

fn endurance_evoke_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    let Some(x) = evoke_pitch(g, p, c) else { return Ok(false) };
    remove_card(&mut g.player_mut(p).hand, x);
    g.player_mut(p).exile.push(x);
    remove_card(&mut g.player_mut(p).hand, c);
    crate::glog!(g, "  {} evokes Endurance (exiling {})", pname(g, p), cname(g, x));
    on_cast(g, p, c)?;
    let m = enter(g, p, c, Enter { was_cast: true, ..Enter::default() })?;
    if g.perm(m).on_bf {
        die(g, m, "sac")?;
    }
    Ok(true)
}

// ================================================================== Enduring Vitality
const VITALITY: &str = "Enduring Vitality";

/// has this Enduring Vitality already come back as an enchantment (DataKey::Final marks it)
fn returned_enchantment(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Final)
}

/// Creatures you control have "{T}: Add one mana of any color." Every untapped creature that is not summoning sick
/// and does not already make mana (the engine's own `rite` path covers the creatures tagged noatk) gets a unit of
/// mana in any colour. The returned enchantment gives the ability but is not a creature itself.
fn vitality_mana(g: &Game, src: Src, p: PlayerId, units: &[Unit]) -> Vec<Unit> {
    if g.perm(src).owner != p || !g.perm(src).on_bf || g.perm(src).phased {
        return vec![];
    }
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            x.on_bf
                && !x.phased
                && g.is_creature(m)
                && !(m == src && returned_enchantment(g, m))
                && !x.tapped
                && !x.sick
                && !units.iter().any(|u| u.src == Source::Perm(m))
        })
        .map(|m| Unit { src: Source::Perm(m), cols: Colors::from_letters("WUBRG"), amt: 1 })
        .collect()
}

/// When Enduring Vitality dies, if it was a creature, return it to the battlefield under its owner's control. It's an
/// enchantment (not a creature). The engine has no way to take the creature type off a creature card, so the card
/// comes back marked (DataKey::Final) and `vitality_mana` ignores it as a creature (engine need). It is not flagged noatk: that would let the engine's `rite` mana path tap it.
fn vitality_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m != src || returned_enchantment(g, m) {
        return Ok(());
    }
    let (Some(cd), owner) = (g.perm(m).cd, g.perm(m).orig) else { return Ok(()) };
    if !g.player(owner).gy.contains(&cd) {
        return Ok(()); // exiled or otherwise gone
    }
    if !trigger_window(g, owner, Some(m), "Enduring Vitality returns as an enchantment", Some(4.0))? {
        return Ok(());
    }
    remove_card(&mut g.player_mut(owner).gy, cd);
    let n = enter(g, owner, cd, Enter::default())?;
    let x = g.perm_mut(n);
    x.data.set(DataKey::Final, Val::Bool(true));
    x.data.set(DataKey::NotCreature, Val::Bool(true)); // --- Tiers 4/5 (chulane-kinnan): an enchantment, not a creature
    crate::glog!(g, "    {} returns as an enchantment (Enduring Vitality)", pname(g, owner));
    Ok(())
}

// ================================================================== The Cabbage Merchant
const MERCHANT: &str = "The Cabbage Merchant";

/// the Foods tapped for mana so far (kept on the Merchant; the engine counts Foods only as `Player::foods`)
fn foods_tapped(g: &Game, m: PermId) -> u32 {
    g.perm(m).data.get(DataKey::Burden).map_or(0, Val::int).max(0) as u32
}

/// the Foods p could tap now
pub fn foods_untapped(g: &Game, m: PermId) -> u32 {
    g.player(g.perm(m).owner).foods.saturating_sub(foods_tapped(g, m))
}

/// Whenever an opponent casts a noncreature spell, create a Food token.
fn merchant_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    let d = g.db.get(c);
    if caster == o || d.creature || d.land || !g.perm(src).on_bf {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "create a Food", Some(2.0))? {
        make_artifact_tokens(g, o, "Food", 1)?;
    }
    Ok(())
}

/// Whenever a creature deals combat damage to you, sacrifice a Food token (not optional; nothing happens without
/// one).
fn merchant_damage(g: &mut Game, src: Src, _p: PlayerId, _a: PermId, d: PlayerId, dmg: i32) -> Res {
    let o = g.perm(src).owner;
    if d != o || dmg <= 0 || !g.perm(src).on_bf || g.player(o).foods == 0 {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "sacrifice a Food", Some(2.0))? {
        sac_food(g, o, 1)?;
        let t = foods_tapped(g, src).min(g.player(o).foods);
        g.perm_mut(src).data.set(DataKey::Burden, Val::Int(t as i64));
    }
    Ok(())
}

/// Tap two untapped Foods you control: Add one mana of any color. Offered as a source of mana worth one per pair of
/// untapped Foods. The engine pays through the Merchant (it is the source the unit names), so `merchant_tapped` taps the
/// Foods and gives the Merchant its untap back: the ability does not tap it.
fn merchant_mana(g: &Game, src: Src, p: PlayerId, _units: &[Unit]) -> Vec<Unit> {
    if g.perm(src).owner != p || !g.perm(src).on_bf || g.perm(src).phased || g.perm(src).tapped {
        return vec![];
    }
    let pairs = foods_untapped(g, src) / 2;
    if pairs == 0 {
        return vec![];
    }
    vec![Unit { src: Source::Perm(src), cols: Colors::from_letters("WUBRG"), amt: pairs }]
}

fn merchant_tapped(g: &mut Game, p: PlayerId, m: PermId, used: u32) -> Res {
    if used == 0 || foods_untapped(g, m) < 2 * used {
        return Ok(()); // tapped some other way (convoke): not the Food ability
    }
    let t = foods_tapped(g, m) + 2 * used;
    g.perm_mut(m).data.set(DataKey::Burden, Val::Int(t as i64));
    g.perm_mut(m).tapped = false;
    crate::glog!(g, "    {} taps {} Foods for mana (The Cabbage Merchant)", pname(g, p), 2 * used);
    Ok(())
}

/// the Foods untap with everything else
fn merchant_untap(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner == p {
        g.perm_mut(src).data.set(DataKey::Burden, Val::Int(0));
    }
    Ok(())
}

// ================================================================== Shifting Woodland
const WOODLAND: &str = "Shifting Woodland";

/// Delirium: four or more card types among the cards in p's graveyard
pub fn delirium(g: &Game, p: PlayerId) -> bool {
    let mut types = 0u8;
    for &c in &g.player(p).gy {
        let d = g.db.get(c);
        for (on, bit) in [
            (d.land, 1u8),
            (d.creature, 2),
            (d.instant, 4),
            (d.sorcery, 8),
            (d.types.has(Types::ARTIFACT), 16),
            (d.types.has(Types::ENCHANTMENT), 32),
            (d.types.has(Types::PLANESWALKER), 64),
        ] {
            if on {
                types |= bit;
            }
        }
    }
    types.count_ones() >= 4
}

/// the permanent card in p's graveyard the AI has Shifting Woodland copy: the best creature card (Oboro Breezecaller
/// first: recursion for the Cradle line), not a legend p already controls
pub fn woodland_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let cands: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            d.creature
                && !(d.tag(Tag::Leg)
                    && g.player(p)
                        .perms
                        .iter()
                        .any(|&m| g.perm(m).on_bf && g.perm(m).cd.is_some_and(|x| g.db.get(x).name == d.name)))
        })
        .collect();
    max_by(&cands, |c| {
        let d = g.db.get(c);
        let oboro = if &*d.name == "Oboro Breezecaller" { 100.0 } else { 0.0 };
        oboro + card_worth(g, p, c, true) + d.cmc as f64
    })
}

/// Delirium {2}{G}{G}: this land becomes a copy of target permanent card in your graveyard until end of turn. The AI
/// does it in its first main phase when the delirium is there, the mana is spare and a creature card is worth copying.
fn woodland_options(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || !g.player(p).lands.contains(&l) || g.land(l).tapped {
        return Ok(vec![]);
    }
    if !delirium(g, p) || !can_pay(g, p, 2, "GG", false) {
        return Ok(vec![]);
    }
    let Some(t) = woodland_pick(g, p) else { return Ok(vec![]) };
    if &*g.db.get(t).name != "Oboro Breezecaller" && card_worth(g, p, t, true) < 20.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.0,
        label: format!("Shifting Woodland becomes {}", cname(g, t)),
        act: Some(Action::Plan { f: woodland_go, arg: ((l.0 as i64) << 32) | t.0 as i64 }),
    }])
}

fn woodland_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, t) = (LandId((arg >> 32) as u32), CardId((arg & 0xffff_ffff) as u16));
    if !g.player(p).lands.contains(&l)
        || !g.player(p).gy.contains(&t)
        || !delirium(g, p)
        || !can_pay(g, p, 2, "GG", false)
    {
        return Ok(false);
    }
    pay(g, p, 2, "GG", false)?;
    if !ability_window(g, p, None, &format!("Shifting Woodland becomes a copy of {}", cname(g, t)), Some(3.0), None)? {
        return Ok(true);
    }
    if !g.player(p).lands.contains(&l) || !g.player(p).gy.contains(&t) {
        return Ok(true);
    }
    // the land is the creature now: it makes no mana as a land; it has been under its controller's control all along
    g.land_mut(l).tapped = true;
    crate::glog!(g, "  {} Shifting Woodland becomes a copy of {}", pname(g, p), cname(g, t));
    if let Some(m) = enter_token_copy(g, p, t)?
        && g.perm(m).on_bf
    {
        let x = g.perm_mut(m);
        x.sick = false;
        x.temp = true; // gone at the end step: the land is a land again
    }
    Ok(true)
}

// ================================================================== Invasion of Ikoria // Zilortha, Apex of Ikoria
const IKORIA: &str = "Invasion of Ikoria // Zilortha, Apex of Ikoria";

/// the non-Human creature card with mana value X or less the AI finds, in the library or the graveyard: the deck's
/// wish list first, else the most useful (card worth, then mana value). The bool is true for a library card.
pub fn ikoria_pick(g: &Game, p: PlayerId, x: i32) -> Option<(CardId, bool)> {
    let ok = |c: &CardId| {
        let d = g.db.get(*c);
        d.creature && !d.has_subtype("human") && d.cmc as i32 <= x
    };
    let mut cands: Vec<(CardId, bool)> = searchable(g, p).into_iter().filter(ok).map(|c| (c, true)).collect();
    cands.extend(g.player(p).gy.iter().copied().filter(ok).map(|c| (c, false)));
    if cands.is_empty() {
        return None;
    }
    // Tier 6: the card that completes one of the deck's lines (Oboro) comes before the wish list's other creatures
    if super::t6_plans::is_t6(g, p) && !crate::human::is_human(g, p) {
        let held = super::t6_pieces::held_names(g, p);
        if let Some(&b) =
            cands.iter().find(|(c, _)| super::t6_pieces::completes_line(g, p, &g.db.get(*c).name, &held))
        {
            return Some(b);
        }
    }
    let wish = crate::ai::plans::wish_list(g, p);
    if let Some(&b) = cands.iter().find(|(c, _)| wish.contains(c)) {
        return Some(b);
    }
    max_by(&cands, |(c, _)| card_worth(g, p, c, false) * 100.0 + g.db.get(c).cmc as f64)
}

/// {X}{G}{G}. When this Siege enters, search your library and/or graveyard for a non-Human creature card with mana
/// value X or less and put it onto the battlefield; shuffle if you searched the library. X is all the spare mana.
/// The Siege is a battle that stays on the battlefield and can be attacked; the engine has no battles, so the card
/// goes to the graveyard (engine need: battles, Zilortha's back face).
fn ikoria_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x.max(0);
    if let Some((c, lib)) = ikoria_pick(g, p, x) {
        if lib {
            remove_card(&mut g.player_mut(p).library, c);
            shuffle_library(g, p);
            g.player_mut(p).stat("tutored", 1);
        } else {
            remove_card(&mut g.player_mut(p).gy, c);
        }
        crate::glog!(g, "    {} puts {} onto the battlefield (Invasion of Ikoria, X = {x})", pname(g, p), cname(g, c));
        enter(g, p, c, Enter::default())?;
    } else if g.player(p).library.len() > 1 {
        shuffle_library(g, p); // a failed search of the library still shuffles
    }
    Ok(intern("gy"))
}

/// cast with enough mana for a two-drop: X = spare mana
fn ikoria_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if total_mana(g, p, false) >= 4 && ikoria_pick(g, p, total_mana(g, p, false) as i32 - 2).is_some() { 60 } else { 0 }
}

// ================================================================== registration
// ================================================================== Growing Rites of Itlimoc
/// **Growing Rites of Itlimoc** // **Itlimoc, Cradle of the Sun** ({2}{G} legendary enchantment, transforms into a
/// legendary land). When it enters, look at the top four cards: a creature card among them to hand (the most valuable),
/// the rest to the bottom. At the beginning of your end step, with four or more creatures, it transforms: Itlimoc is a
/// land that taps for {G}, or {G} for each creature you control (as Gaea's Cradle; the lines take it as their Cradle,
/// `t6_combos_t6::cradle_land`). Left out: the order of the cards put on the bottom.
pub const RITES: &str = "Growing Rites of Itlimoc // Itlimoc, Cradle of the Sun";

fn creatures_of_count(g: &Game, p: PlayerId) -> u32 {
    g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count() as u32
}

fn rites_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || !trigger_window(g, o, Some(src), "look at the top four, a creature card to hand", Some(3.0))? {
        return Ok(());
    }
    let n = g.player(o).library.len().min(4);
    let top: Vec<CardId> = g.player(o).library[g.player(o).library.len() - n..].to_vec();
    let pick = top
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).creature)
        .max_by(|&a, &b| card_worth(g, o, a, false).total_cmp(&card_worth(g, o, b, false)));
    let lib = &mut g.player_mut(o).library;
    lib.truncate(lib.len() - n);
    for &c in top.iter().filter(|&&c| Some(c) != pick) {
        lib.insert(0, c);
    }
    if let Some(c) = pick {
        g.player_mut(o).hand.push(c);
        crate::glog!(g, "    {} takes {} (Growing Rites of Itlimoc)", pname(g, o), g.db.get(c).name);
    }
    Ok(())
}

fn rites_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if !g.perm(src).on_bf || g.perm(src).phased || g.active != Some(o) || creatures_of_count(g, o) < 4 {
        return Ok(());
    }
    let (cd, tapped) = (g.perm(src).cd.expect("Growing Rites is a card"), g.perm(src).tapped);
    crate::engine::zones::leave(g, src)?;
    g.add_land(o, cd, tapped);
    crate::glog!(g, "    Growing Rites of Itlimoc transforms into Itlimoc, Cradle of the Sun");
    Ok(())
}

/// Itlimoc: {G}, or {G} for each creature you control (the larger)
fn itlimoc_mana(g: &Game, p: PlayerId, _l: LandId) -> u32 {
    creatures_of_count(g, p).max(1)
}

fn itlimoc_cols(_g: &Game, _p: PlayerId, _l: LandId) -> Colors {
    Colors::from_letters("G")
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if db.id(RITES).is_some() {
        let c = r.card(db, RITES)?;
        c.etb = Some(rites_etb);
        c.end_step = Some(rites_end);
        c.dyn_mana_land = Some(itlimoc_mana);
        c.land_cols = Some(itlimoc_cols);
    }
    r.card(db, "Chatterstorm")?.self_cast = Some(chatterstorm_cast);
    let c = r.card(db, "Cloud of Faeries")?;
    c.etb = Some(cloud_etb);
    c.hand_options = Some(cloud_options);
    let c = r.card(db, "Noxious Revival")?;
    c.resolve = Some(revival_resolve);
    c.prio = Some(revival_prio);
    r.card(db, "Sowing Mycospawn")?.self_cast = Some(mycospawn_cast);
    let c = r.card(db, ENDURANCE)?;
    c.etb = Some(endurance_etb);
    c.hand_options = Some(endurance_evoke);
    let c = r.card(db, VITALITY)?;
    c.extra_mana = Some(vitality_mana);
    c.self_dies = Some(vitality_dies);
    let c = r.card(db, MERCHANT)?;
    c.cast = Some(merchant_cast);
    c.combat_damage = Some(merchant_damage);
    c.extra_mana = Some(merchant_mana);
    c.on_tap_perm = Some(merchant_tapped);
    c.upkeep = Some(merchant_untap);
    r.card(db, WOODLAND)?.land_options = Some(woodland_options);
    let c = r.card(db, IKORIA)?;
    c.resolve = Some(ikoria_resolve);
    c.prio = Some(ikoria_prio);
    Ok(())
}
