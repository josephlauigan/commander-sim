//! Tier 3 pool decks `korvold-jund-sacrifice` and `tergrid-mono-black-disruption` (EDHREC's bracket-3 average lists):
//! the cards the ability language doesn't cover (rust/agents/t3-model-korvold.md). The Reaver Cleaver, Smallpox and
//! Vicious Rumors are compiled (dsl_parse.rs, `// --- Tier 3 (korvold)`); the rest is here.
//!
//! Korvold:
//! - Rhythm of the Wild: your creature spells can't be countered; riot on your nontoken creatures (haste when it can
//!   still attack this turn, else a +1/+1 counter).
//! - Impulsive Pilferer: encore from the graveyard (a hasty token copy per opponent that must attack, sacrificed at the
//!   end step: each makes a Treasure as it dies). Left out: each copy attacks *its* opponent (the AI picks).
//! - Juri, Master of the Revue: +1/+1 counter whenever you sacrifice a permanent; dies: damage equal to its power.
//! - Marionette Apprentice: fabricate 1 (a Servo); another creature or artifact of yours to the graveyard from the
//!   battlefield (Treasure, Food and Clue tokens sacrificed too): each opponent loses 1 life.
//! - Mazirek, Kraul Death Priest: whenever any player sacrifices another permanent, +1/+1 on each of your creatures.
//! - Ruthless Technomancer: enters: may sacrifice another creature for Treasures equal to its power; {2}{B},
//!   sacrifice X artifacts: return a creature card with power X or less from your graveyard.
//! - Smaug, Wicked Worm: enters: X tapped Treasures (X: the artifacts your opponents control, their Treasure, Food and
//!   Clue tokens included); a spell you cast with Treasure mana: draw a card and lose 1 life.
//!
//! Tergrid:
//! - Blasphemous Edict: each player sacrifices thirteen creatures (opponents first, so Tergrid sees them go); {B}
//!   instead with thirteen or more creatures on the battlefield.
//! - Bottomless Pit: each player's upkeep, that player discards a card at random.
//! - Imp's Mischief: the Misdirection response (t6_counters::stack_response): a spell aimed at your key permanent goes
//!   to one of its caster's; you lose life equal to its mana value. Left out: other redirects (a counterspell's).
//! - Pox: a third (rounded up) of each player's life, hand, creatures and lands, in that order.
//! - Flare of Malice: each opponent sacrifices its creature or planeswalker of greatest mana value; or cast it by
//!   sacrificing a nontoken black creature of yours.
//! - Liliana, Waker of the Dead: +1 each player discards, an opponent who can't loses 3; -3 -X/-X (X: your graveyard);
//!   -7 the emblem (each combat on your turn, the best creature card from any graveyard, with haste).
//! - Malakir Rebirth // Malakir Mire: in response to removal or an edict on your creature: you lose 2 life and it
//!   returns tapped when it dies; or played as a land that enters tapped and taps for {B}.
//! - Mindslicer: dies: each player discards their hand.
//! - Rankle, Master of Pranks: combat damage to a player: the modes the AI wants (discard, lose 1 and draw, sacrifice).
//! - Sangromancer: an opponent discards: gain 3 life (its creature-dies half is compiled).
//! - The Raven Man: each end step, if a player discarded this turn, a 1/1 flying Bird that can't block (its {3}{B}
//!   activation is compiled).

use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable, discard_worst};
use crate::engine::combat::has_haste;
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, ability_window_card, trigger_window};
use crate::engine::values::{epow, etgh, has, indestructible, no_sac, pval, untargetable};
use crate::engine::zones::{
    Enter, Tokens, add_treasure, die, discard_cards, discard_index, draw, edict, enter, enter_token_copy,
    make_artifact_tokens_tapped, make_tokens, sac_worth,
};
use crate::flow::Res;
use crate::hooks::{Action, AltCost, Call, Event, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::impls::common::{WalkerAb, death_value, sac_food, walker};
use crate::impls::partials::{at_once, eot_kw};
use crate::impls::sephfs_cards::steal_value;
use crate::impls::t3::best_target_any;
use crate::state::{Ctx, DataKey, Game, Step, Val};
use crate::sym::Sym;
use crate::tag::Tag;

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

fn held(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == name)
}

/// p's live creatures
fn creatures_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

/// a permanent that is an artifact (a card's type, or an artifact token: Servo, Thopter)
fn is_artifact(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)) || (x.cd.is_none() && x.ttypes.contains(&"artifact"))
}

/// the living players in turn order from p (APNAP for p's spell)
fn apnap(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    let n = g.players.len();
    (0..n).map(|k| PlayerId(((p.0 as usize + k) % n) as u8)).filter(|&q| g.player(q).alive).collect()
}

/// the land q would give up first: a tapped one, then the fewest colours, a basic before a nonbasic
pub(crate) fn worst_land(g: &Game, q: PlayerId) -> Option<LandId> {
    let lands = &g.player(q).lands;
    let key = |l: LandId| {
        let d = g.db.get(g.land(l).card());
        let cols = d.tags.str(Tag::C).map_or(0, |s| s.chars().count());
        (!g.land(l).tapped, cols, !d.basic)
    };
    lands.iter().copied().min_by_key(|&l| key(l))
}

/// q sacrifices its least useful land (Smallpox, Pox): it goes to the graveyard, the sacrifice triggers see it, and
/// an opposing Tergrid takes it. Called by the ability language's land sacrifice too.
pub fn sac_worst_land(g: &mut Game, q: PlayerId) -> Res {
    let Some(l) = worst_land(g, q) else { return Ok(()) };
    let cd = g.land(l).card();
    crate::glog!(g, "    {} sacrifices {}", pname(g, q), g.db.get(cd).name);
    super::lands::sac_land(g, q, l)?;
    if !g.hooks.is_empty() {
        crate::engine::hooks::fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p: q, what: Sacrificed::Card(cd) })?;
    }
    crate::engine::zones::tergrid_steal(g, q, cd, q)
}

/// q sacrifices one artifact as a cost: a Treasure, then a Clue, a Food, then its least valuable artifact permanent
/// (not `keep`). False if it has none.
fn sac_artifact(g: &mut Game, q: PlayerId, keep: Option<PermId>) -> Res<bool> {
    let fire = |g: &mut Game, k: Sym| -> Res {
        if !g.hooks.is_empty() {
            crate::engine::hooks::fire_trigger(
                g,
                Event::Sacrifice,
                Call::Sacrifice { p: q, what: Sacrificed::Token(k) },
            )?;
        }
        Ok(())
    };
    if g.player(q).treasures > 0 {
        g.player_mut(q).sac_treasure();
        fire(g, "Treasure")?;
        return Ok(true);
    }
    if g.player(q).clues > 0 {
        g.player_mut(q).clues -= 1;
        fire(g, "Clue")?;
        return Ok(true);
    }
    if sac_food(g, q, 1)? {
        return Ok(true);
    }
    let arts: Vec<PermId> = g
        .player(q)
        .perms
        .iter()
        .copied()
        .filter(|&m| Some(m) != keep && live(g, m) && is_artifact(g, m) && !g.perm(m).is_cmd)
        .collect();
    let Some(m) = arts.into_iter().min_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b))) else { return Ok(false) };
    die(g, m, "sac")?;
    Ok(true)
}

/// the artifacts q could sacrifice to a cost: its Treasure / Clue / Food tokens, and artifact permanents worth little
fn spare_artifacts(g: &Game, q: PlayerId, keep: Option<PermId>) -> u32 {
    let pl = g.player(q);
    let perms = pl
        .perms
        .iter()
        .filter(|&&m| Some(m) != keep && live(g, m) && is_artifact(g, m) && !g.perm(m).is_cmd && pval(g, m) < 2.5)
        .count() as u32;
    pl.treasures + pl.clues + pl.foods + perms
}

/// the artifacts q controls (Smaug's X): artifact permanents and its Treasure, Food and Clue tokens
fn artifacts_of(g: &Game, q: PlayerId) -> i32 {
    let pl = g.player(q);
    let perms = pl.perms.iter().filter(|&&m| live(g, m) && is_artifact(g, m)).count() as u32;
    (perms + pl.treasures + pl.foods + pl.clues) as i32
}

/// a symmetric "each player loses / discards / sacrifices" spell or trigger: what player q gives up, in cards' worth
/// (life at 0.12 a point, a card from hand 0.8, its least valuable creatures and lands). `skip` is the card being cast
/// (not in the hand any more as it resolves).
fn give_up(
    g: &Game,
    q: PlayerId,
    life: i32,
    cards: usize,
    creatures: usize,
    lands: usize,
    skip: Option<CardId>,
) -> f64 {
    let pl = g.player(q);
    let hand = pl.hand.iter().filter(|&&c| Some(c) != skip).count();
    let mut cr: Vec<f64> = creatures_of(g, q).iter().map(|&m| sac_worth(g, m).max(0.3)).collect();
    cr.sort_by(|a, b| a.total_cmp(b));
    0.12 * life.min(pl.life) as f64
        + 0.8 * cards.min(hand) as f64
        + cr.iter().take(creatures).sum::<f64>()
        + 1.3 * lands.min(pl.lands.len()) as f64
}

/// p's Tergrid takes the opponents' sacrificed nontoken permanents and discarded permanent cards
fn tergrid_mine(g: &Game, p: PlayerId) -> bool {
    has(g, p, Tag::Tergrid)
}

// ================================================================== Rhythm of the Wild
/// Creature spells you control can't be countered.
fn rhythm_uncounterable(g: &Game, src: Src, caster: PlayerId, c: CardId) -> bool {
    caster == owner(g, src) && g.db.get(c).creature
}

/// Riot (a replacement as the creature enters, run at once): haste when it can still attack this turn (its
/// controller's turn, before combat, no haste of its own), else a +1/+1 counter.
fn rhythm_riot(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if p != owner(g, src) || m == src || !g.perm(m).on_bf || g.perm(m).token || !g.is_creature(m) {
        return Ok(());
    }
    let can_attack = g.active == Some(p) && matches!(g.step, Step::Start | Step::Main1);
    if can_attack && g.perm(m).sick && !has_haste(g, m) && epow(g, m) > 0 {
        eot_kw(g, m, "haste");
        crate::glog!(g, "    {} has riot: haste", g.perm(m).name);
    } else {
        crate::cardcode::put_plus(g, m, 1);
        crate::glog!(g, "    {} has riot: a +1/+1 counter", g.perm(m).name);
    }
    Ok(())
}

// ================================================================== Impulsive Pilferer
const PILFERER: &str = "Impulsive Pilferer";

/// Encore {3}{R} (sorcery speed, before combat): worth it with two or more opponents (a Treasure and a hasty attacker
/// each)
fn pilferer_gy(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || !g.player(p).gy.contains(&c) || !can_pay(g, p, 3, "R", false) {
        return Ok(vec![]);
    }
    let n = g.opps(p).count();
    if n == 0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 0.6 + 0.9 * n as f64,
        label: format!("{PILFERER} (encore)"),
        act: Some(Action::Plan { f: pilferer_encore, arg: c.0 as i64 }),
    }])
}

/// exile it from the graveyard: a token copy per opponent, hasty, attacking this turn if able, sacrificed at the
/// beginning of the next end step (`DataKey::EotSac`)
fn pilferer_encore(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let Some(i) = g.player(p).gy.iter().position(|&x| x == c) else { return Ok(false) };
    if !can_pay(g, p, 3, "R", false) || !pay(g, p, 3, "R", false)? {
        return Ok(false);
    }
    g.player_mut(p).gy.remove(i);
    g.player_mut(p).exile.push(c);
    crate::glog!(g, "  {} encores {}", pname(g, p), PILFERER);
    if !ability_window_card(g, p, c, "encore", None, None)? {
        return Ok(true);
    }
    let now = Val::Stamp(g.turn_stamp());
    for _ in 0..g.opps(p).count() {
        if let Some(t) = enter_token_copy(g, p, c)? {
            let x = g.perm_mut(t);
            x.sick = false;
            x.data.set(DataKey::MustAttack, now.clone());
            x.data.set(DataKey::EotSac, Val::Bool(true));
        }
    }
    Ok(true)
}

// ================================================================== Juri, Master of the Revue
/// Whenever you sacrifice a permanent, put a +1/+1 counter on Juri.
fn juri_sac(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p != owner(g, src) || what == Sacrificed::Perm(src) || !g.perm(src).on_bf {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "a +1/+1 counter", Some(1.0))? && g.perm(src).on_bf {
        crate::cardcode::put_plus(g, src, 1);
    }
    Ok(())
}

/// When Juri dies, it deals damage equal to its power (as it last existed) to any target.
fn juri_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m != src {
        return Ok(());
    }
    let n = epow(g, m);
    let o = owner(g, src);
    if n > 0 && trigger_window(g, o, Some(src), &format!("{n} damage to any target"), None)? {
        best_target_any(g, o, n)?;
    }
    Ok(())
}

// ================================================================== Marionette Apprentice
/// Fabricate 1: the AI makes the Servo (another body to sacrifice)
fn apprentice_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "fabricate 1", None)? {
        let spec = Tokens { color: Some(Colors::NONE), types: vec!["servo", "artifact"], ..Tokens::new(1, 1) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

fn apprentice_drain(g: &mut Game, src: Src) -> Res {
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "each opponent loses 1 life", None)? {
        for q in g.opps(o).collect::<Vec<_>>() {
            lose_life(g, q, 1, Some(o), "triggers", None)?;
        }
    }
    Ok(())
}

/// another creature or artifact you control is put into a graveyard from the battlefield
fn apprentice_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m == src || g.perm(m).owner != owner(g, src) || !(g.is_creature(m) || is_artifact(g, m)) {
        return Ok(());
    }
    apprentice_drain(g, src)
}

/// a Treasure, Food or Clue token of yours sacrificed (they are artifacts that go to the graveyard)
fn apprentice_sac(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p != owner(g, src) || !matches!(what, Sacrificed::Token(_)) {
        return Ok(());
    }
    apprentice_drain(g, src)
}

// ================================================================== Mazirek, Kraul Death Priest
/// Whenever a player sacrifices another permanent, put a +1/+1 counter on each creature you control.
fn mazirek_sac(g: &mut Game, src: Src, _p: PlayerId, what: Sacrificed) -> Res {
    if what == Sacrificed::Perm(src) || !g.perm(src).on_bf {
        return Ok(());
    }
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "a +1/+1 counter on each creature you control", Some(2.0))? {
        for m in creatures_of(g, o) {
            crate::cardcode::put_plus(g, m, 1);
        }
    }
    Ok(())
}

// ================================================================== Ruthless Technomancer
/// the creature the Technomancer's enters ability should sacrifice: the most Treasures for the least value (its
/// power, plus what its death gives, against what it is worth); none when nothing is worth it
fn techno_fodder(g: &Game, o: PlayerId, src: PermId) -> Option<PermId> {
    let mut best: Option<(PermId, f64)> = None;
    for m in creatures_of(g, o) {
        if m == src || g.perm(m).is_cmd || no_sac(g, m) || epow(g, m) < 2 {
            continue;
        }
        let score = 0.8 * epow(g, m) as f64 + death_value(g, o, Some(m)) - pval(g, m);
        if score >= 0.5 && best.is_none_or(|b| score > b.1) {
            best = Some((m, score));
        }
    }
    best.map(|b| b.0)
}

/// When it enters, you may sacrifice another creature you control: Treasures equal to that creature's power.
fn techno_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if techno_fodder(g, o, src).is_none() {
        return Ok(()); // the AI says no
    }
    if !trigger_window(g, o, Some(src), "sacrifice a creature for Treasures", None)? {
        return Ok(());
    }
    let Some(f) = techno_fodder(g, o, src) else { return Ok(()) };
    let n = epow(g, f);
    crate::glog!(g, "    {} sacrifices {} for {n} Treasures", pname(g, o), g.perm(f).name);
    die(g, f, "sac")?;
    add_treasure(g, o, n)
}

/// the creature card the {2}{B} ability should return: the best in the graveyard it can pay X for
fn techno_target(g: &mut Game, o: PlayerId, src: PermId) -> Option<(CardId, u32)> {
    let have = spare_artifacts(g, o, Some(src));
    let mut best: Option<(CardId, u32, f64)> = None;
    for &c in &g.player(o).gy {
        let d = g.db.get(c);
        if !d.creature {
            continue;
        }
        let x = d.pow.max(1) as u32; // X can't be 0
        let v = steal_value(g, c) - 0.4 * x as f64;
        if x <= have && best.is_none_or(|b| v > b.2) {
            best = Some((c, x, v));
        }
    }
    let (c, x, v) = best?;
    if v < 1.5 {
        return None;
    }
    // the {2}{B} must still be payable once the Treasures are gone
    let t = g.player(o).treasures.min(x);
    g.player_mut(o).treasures -= t;
    let ok = can_pay(g, o, 2, "B", false);
    g.player_mut(o).treasures += t;
    ok.then_some((c, x))
}

fn techno_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || p != owner(g, src) || g.active != Some(p) || !live(g, src) {
        return Ok(vec![]);
    }
    let Some((c, x)) = techno_target(g, p, src) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: steal_value(g, c) - 0.4 * x as f64,
        label: format!("Ruthless Technomancer: return {}", g.db.get(c).name),
        act: Some(Action::Ability { src, f: techno_go, arg: 0 }),
    }])
}

/// {2}{B}, Sacrifice X artifacts: return target creature card with power X or less from your graveyard
fn techno_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some((c, x)) = techno_target(g, p, src) else { return Ok(false) };
    for _ in 0..x {
        if !sac_artifact(g, p, Some(src))? {
            return Ok(false);
        }
    }
    if !pay(g, p, 2, "B", false)? {
        return Ok(false);
    }
    crate::glog!(g, "  {} activates Ruthless Technomancer (X={x}): {}", pname(g, p), g.db.get(c).name);
    if ability_window(g, p, Some(src), "return a creature card", None, None)?
        && let Some(i) = g.player(p).gy.iter().position(|&y| y == c)
    {
        g.player_mut(p).gy.remove(i);
        enter(g, p, c, Enter::default())?;
    }
    Ok(true)
}

// ================================================================== Smaug, Wicked Worm
/// When Smaug enters, create X tapped Treasure tokens, X: the artifacts your opponents control.
fn smaug_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let x: i32 = g.opps(o).map(|q| artifacts_of(g, q)).sum();
    if x > 0 && trigger_window(g, o, Some(src), &format!("{x} tapped Treasures"), None)? {
        let x: i32 = g.opps(o).map(|q| artifacts_of(g, q)).sum();
        make_artifact_tokens_tapped(g, o, "Treasure", x, true)?;
    }
    Ok(())
}

/// Whenever you cast a spell, if mana from a Treasure was spent to cast it, you draw a card and lose 1 life.
fn smaug_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    if caster != o || !g.cast_treasure.is_some_and(|(x, n)| x == c && n > 0) {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "draw a card and lose 1 life", None)? {
        draw(g, o, 1, false)?;
        lose_life(g, o, 1, Some(o), "other", None)?;
    }
    Ok(())
}

// ================================================================== Blasphemous Edict
const EDICT: &str = "Blasphemous Edict";

fn creatures_on_bf(g: &Game) -> usize {
    g.players.iter().filter(|pl| pl.alive).map(|pl| creatures_of(g, pl.id).len()).sum()
}

/// the creatures q sacrifices: all of them, or its thirteen least valuable
fn edict_victims(g: &Game, q: PlayerId) -> Vec<PermId> {
    let mut cr: Vec<PermId> = creatures_of(g, q).into_iter().filter(|&m| !no_sac(g, m)).collect();
    cr.sort_by(|&a, &b| sac_worth(g, a).total_cmp(&sac_worth(g, b)));
    cr.truncate(13);
    cr
}

/// Each player sacrifices thirteen creatures of their choice: the opponents first, so a Tergrid of the caster's sees
/// their creatures go before it is sacrificed itself (the sacrifices are simultaneous: it looks back)
fn edict_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut order = apnap(g, p);
    order.rotate_left(1); // the caster last
    let lists: Vec<(PlayerId, Vec<PermId>)> = order.iter().map(|&q| (q, edict_victims(g, q))).collect();
    for (q, ms) in lists {
        if !ms.is_empty() {
            crate::glog!(g, "    {} sacrifices {} creature(s)", pname(g, q), ms.len());
        }
        for m in ms {
            if g.perm(m).on_bf && g.perm(m).owner == q {
                die(g, m, "sac")?;
            }
        }
    }
    check_state(g)?;
    Ok("gy")
}

/// what the edict gains p: the opponents' creatures lost (taken by Tergrid: counted again) less p's own
fn edict_value(g: &Game, p: PlayerId) -> f64 {
    let lost = |q: PlayerId| edict_victims(g, q).iter().map(|&m| pval(g, m)).sum::<f64>();
    let mut theirs = 0.0;
    for q in g.opps(p) {
        theirs += lost(q);
        if tergrid_mine(g, p) {
            theirs += edict_victims(g, q).iter().filter(|&&m| !g.perm(m).token).map(|&m| pval(g, m)).sum::<f64>();
        }
    }
    theirs / g.opps(p).count().max(1) as f64 * 1.5 - lost(p)
}

fn edict_alt_ok(g: &Game, p: PlayerId, _c: CardId, _spell: Option<CardId>) -> bool {
    creatures_on_bf(g) >= 13 && can_pay(g, p, 0, "B", false)
}

fn edict_alt_pay(g: &mut Game, p: PlayerId, c: CardId, s: Option<CardId>) -> Res<bool> {
    if !edict_alt_ok(g, p, c, s) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)
}

const EDICT_ALT: AltCost = AltCost { ok: edict_alt_ok, pay: edict_alt_pay, lost: |_, _, _, _| 0.0 };

fn edict_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    if !(edict_alt_ok(g, p, c, None) || can_pay(g, p, 3, "BB", false)) {
        return Ok(vec![]);
    }
    let v = edict_value(g, p);
    if v < 5.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.0 + 0.4 * v,
        label: EDICT.into(),
        act: Some(Action::Plan { f: edict_go, arg: c.0 as i64 }),
    }])
}

fn edict_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    let paid = if edict_alt_ok(g, p, c, None) { edict_alt_pay(g, p, c, None)? } else { pay(g, p, 3, "BB", false)? };
    if !paid {
        return Ok(false);
    }
    cast_card(g, p, c, "hand", Ctx::default())
}

// ================================================================== Bottomless Pit
/// At the beginning of each player's upkeep, that player discards a card at random.
fn pit_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.player(p).hand.is_empty() {
        return Ok(());
    }
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "discard a card at random", None)? {
        let n = g.player(p).hand.len() as u64;
        if n > 0 {
            let i = g.rng.below(n) as usize;
            discard_index(g, p, i)?;
        }
    }
    Ok(())
}

/// cast it when it costs p little: a small hand of its own (the Pit itself aside), or a Tergrid out to take the
/// permanent cards they lose
fn pit_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let mine = g.player(p).hand.len().saturating_sub(1);
    let theirs = g.opps(p).map(|q| g.player(q).hand.len()).sum::<usize>() / g.opps(p).count().max(1);
    if tergrid_mine(g, p) || mine <= 2 || theirs >= mine + 3 { 48 } else { 0 }
}

// ================================================================== Imp's Mischief
pub const IMPS: &str = "Imp's Mischief";

/// q answers the spell on the stack (item `id`) aimed at its key permanent with Imp's Mischief ({1}{B}): the spell
/// goes to one of its caster's permanents (t6_counters::misdirect), and q loses life equal to its mana value. Only
/// with the life to spare. True if cast.
pub fn imps_mischief(g: &mut Game, q: PlayerId, id: u32) -> Res<bool> {
    let Some(c) = held(g, q, IMPS) else { return Ok(false) };
    let Some(it) = g.stack.iter().find(|x| x.id == id).cloned() else { return Ok(false) };
    let (Some(spell), Some(t)) = (it.card, it.ctx.target.filter(|_| it.ctx.face.is_none())) else { return Ok(false) };
    let mv = g.db.get(spell).cmc as i32;
    if g.perm(t).owner != q
        || !(pval(g, t) >= 4.0 || g.perm(t).is_cmd)
        || g.db.get(spell).land
        || g.player(q).life <= mv + 6
        || !super::t6_counters::would_redirect(g, q, id)
        || !can_pay(g, q, 1, "B", false)
    {
        return Ok(false);
    }
    if !super::t6_counters::pay_response(g, q, c, spell, false)? {
        return Ok(false);
    }
    let done = super::t6_counters::misdirect(g, q, id)?;
    lose_life(g, q, mv, Some(q), "other", None)?;
    Ok(done)
}

// ================================================================== Pox
/// Each player loses a third of their life, then discards a third of the cards in their hand, then sacrifices a
/// third of their creatures, then a third of their lands (rounded up each time), each step for every player in turn
/// order before the next
fn pox_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let third = |n: usize| n.div_ceil(3);
    let order = apnap(g, p);
    for &q in &order {
        let n = g.player(q).life.max(0) as usize;
        lose_life(g, q, third(n) as i32, Some(p), "other", None)?;
    }
    for &q in &order {
        let n = third(g.player(q).hand.len());
        if n > 0 && g.player(q).alive {
            discard_worst(g, q, n as u32)?;
        }
    }
    for &q in &order {
        let n = third(creatures_of(g, q).len());
        for _ in 0..n {
            edict(g, q, false)?;
        }
    }
    for &q in &order {
        let n = third(g.player(q).lands.len());
        for _ in 0..n {
            sac_worst_land(g, q)?;
        }
    }
    check_state(g)?;
    Ok("gy")
}

/// what a symmetric spell gains p: the opponents' average loss (with p's Tergrid, their permanents count twice) less
/// p's own. `f(q)` is q's (life, cards, creatures, lands) lost.
fn symmetric_gain(g: &Game, p: PlayerId, c: CardId, f: impl Fn(&Game, PlayerId) -> (i32, usize, usize, usize)) -> f64 {
    let loss = |q: PlayerId| {
        let (l, h, cr, ld) = f(g, q);
        give_up(g, q, l, h, cr, ld, Some(c))
    };
    let k = g.opps(p).count().max(1) as f64;
    let mut theirs = g.opps(p).map(loss).sum::<f64>() / k;
    if tergrid_mine(g, p) {
        theirs *= 1.6;
    }
    theirs - loss(p)
}

fn pox_counts(g: &Game, q: PlayerId) -> (i32, usize, usize, usize) {
    let pl = g.player(q);
    (
        (pl.life.max(0) + 2) / 3,
        pl.hand.len().div_ceil(3),
        creatures_of(g, q).len().div_ceil(3),
        pl.lands.len().div_ceil(3),
    )
}

fn pox_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    if symmetric_gain(g, p, c, pox_counts) >= 2.0 { 55 } else { 0 }
}

/// Smallpox (compiled): cast it when the one life, card, creature and land each hurt the opponents more
fn smallpox_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    if symmetric_gain(g, p, c, |_, _| (1, 1, 1, 1)) >= 1.0 { 50 } else { 0 }
}

// ================================================================== Flare of Malice
const FLARE: &str = "Flare of Malice";

/// q's creature or planeswalker of greatest mana value (its least valuable among those)
fn flare_victim(g: &Game, q: PlayerId) -> Option<PermId> {
    let cp: Vec<PermId> = g
        .player(q)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            live(g, m)
                && !no_sac(g, m)
                && (g.is_creature(m) || g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER)))
        })
        .collect();
    let mv = |m: PermId| g.perm(m).cd.map_or(0, |c| g.db.get(c).cmc);
    let top = cp.iter().map(|&m| mv(m)).max()?;
    cp.into_iter().filter(|&m| mv(m) == top).min_by(|&a, &b| sac_worth(g, a).total_cmp(&sac_worth(g, b)))
}

/// Each opponent sacrifices a creature or planeswalker with the greatest mana value among those they control.
fn flare_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for q in apnap(g, p).into_iter().filter(|&q| q != p) {
        if let Some(m) = flare_victim(g, q) {
            crate::glog!(g, "    {} sacrifices {} ({FLARE})", pname(g, q), g.perm(m).name);
            die(g, m, "sac")?;
        }
    }
    Ok("gy")
}

/// the nontoken black creature p would sacrifice instead of paying: its least valuable, worth under `cap`
fn flare_fodder(g: &Game, p: PlayerId, cap: f64) -> Option<PermId> {
    creatures_of(g, p)
        .into_iter()
        .filter(|&m| {
            let x = g.perm(m);
            !x.token && !x.is_cmd && !no_sac(g, m) && x.cd.is_some_and(|c| g.db.get(c).pips.contains('B'))
        })
        .filter(|&m| pval(g, m) < cap)
        .min_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)))
}

/// the value Flare takes from the opponents (their greatest-mana-value creature or planeswalker each)
fn flare_value(g: &Game, p: PlayerId) -> f64 {
    g.opps(p).filter_map(|q| flare_victim(g, q)).map(|m| pval(g, m)).sum()
}

fn flare_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    if post.is_some() && g.active != Some(p) {
        return Ok(vec![]);
    }
    let v = flare_value(g, p);
    if v < 4.0 || !(can_pay(g, p, 2, "BB", false) || flare_fodder(g, p, v - 1.5).is_some()) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.5 + 0.4 * v,
        label: FLARE.into(),
        act: Some(Action::Plan { f: flare_go, arg: c.0 as i64 }),
    }])
}

fn flare_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    if can_pay(g, p, 2, "BB", false) {
        if !pay(g, p, 2, "BB", false)? {
            return Ok(false);
        }
    } else {
        // the alternative cost: a nontoken black creature worth well under what Flare takes
        let Some(m) = flare_fodder(g, p, flare_value(g, p) - 1.5) else { return Ok(false) };
        crate::glog!(g, "  {} sacrifices {} to cast {FLARE}", pname(g, p), g.perm(m).name);
        die(g, m, "sac")?;
    }
    cast_card(g, p, c, "hand", Ctx::default())
}

// ================================================================== Liliana, Waker of the Dead
const LILIANA_EMBLEM: &str = "liliana_waker";

/// +1: each player discards a card; each opponent who can't loses 3 life
fn lili_plus_v(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let empty = g.opps(p).filter(|&q| g.player(q).hand.is_empty()).count() as f64;
    let mine = if g.player(p).hand.is_empty() { 0.0 } else { 0.6 };
    Some(1.5 + 1.0 * empty + 0.4 * g.opps(p).count() as f64 - mine)
}

fn lili_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let order = apnap(g, p);
    let empty: Vec<PlayerId> = order.iter().copied().filter(|&q| q != p && g.player(q).hand.is_empty()).collect();
    for &q in &order {
        if !g.player(q).hand.is_empty() {
            discard_worst(g, q, 1)?;
        }
    }
    for q in empty {
        lose_life(g, q, 3, Some(p), "other", None)?;
    }
    Ok(())
}

/// -3: the opposing creature the -X/-X kills that is worth most (X: cards in your graveyard)
fn lili_minus_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let x = g.player(p).gy.len() as i32;
    g.opps(p)
        .flat_map(|q| creatures_of(g, q))
        .filter(|&m| !untargetable(g, m) && etgh(g, m) <= x)
        .max_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)))
}

fn lili_minus_v(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = lili_minus_target(g, p)?;
    (pval(g, t) >= 3.0).then(|| 0.8 * pval(g, t))
}

fn lili_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let x = g.player(p).gy.len() as i32;
    if let Some(t) = lili_minus_target(g, p) {
        crate::glog!(g, "    {} gets -{x}/-{x}", g.perm(t).name);
        apply_removal(g, Some(p), t, &format!("shrink{x}"), None)?;
    }
    Ok(())
}

/// -7: the emblem (`liliana_emblem` at each beginning of combat on your turn)
fn lili_ult_v(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    (!g.player(p).emblems.contains(&LILIANA_EMBLEM)).then_some(9.0)
}

fn lili_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    g.player_mut(p).emblems.push(LILIANA_EMBLEM);
    crate::glog!(g, "    {} gets Liliana's emblem", pname(g, p));
    Ok(())
}

static LILIANA: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "each player discards", val: lili_plus_v, eff: lili_plus },
    WalkerAb { delta: -3, label: "-X/-X", val: lili_minus_v, eff: lili_minus },
    WalkerAb { delta: -7, label: "emblem", val: lili_ult_v, eff: lili_ult },
];

/// Liliana's emblem, at the beginning of combat on p's turn (engine/combat.rs): put the best creature card from any
/// graveyard onto the battlefield under p's control; it gains haste. Once per emblem.
pub fn liliana_emblem(g: &mut Game, p: PlayerId) -> Res {
    let n = g.player(p).emblems.iter().filter(|&&e| e == LILIANA_EMBLEM).count();
    for _ in 0..n {
        let mut best: Option<(CardId, PlayerId, f64)> = None;
        for pl in g.players.iter() {
            for &c in &pl.gy {
                let v = steal_value(g, c);
                if v > 0.0 && best.is_none_or(|b| v > b.2) {
                    best = Some((c, pl.id, v));
                }
            }
        }
        let Some((c, q, _)) = best else { return Ok(()) };
        if !crate::engine::stack::trigger_window(
            g,
            p,
            None,
            "Liliana's emblem: a creature card from a graveyard",
            None,
        )? {
            continue;
        }
        let Some(i) = g.player(q).gy.iter().position(|&x| x == c) else { continue };
        g.player_mut(q).gy.remove(i);
        crate::glog!(g, "    Liliana's emblem: {} returns {} under {}", pname(g, q), g.db.get(c).name, pname(g, p));
        let m = enter(g, p, c, Enter { orig: Some(q), sick: false, ..Enter::default() })?;
        g.perm_mut(m).sick = false;
        eot_kw(g, m, "haste");
    }
    Ok(())
}

// ================================================================== Malakir Rebirth // Malakir Mire
const REBIRTH: &str = "Malakir Rebirth // Malakir Mire";

/// Choose target creature. You lose 2 life. Until end of turn, it gains "When this creature dies, return it to the
/// battlefield tapped under its owner's control" (`ret_dies_tapped`, run by `sheoldred::returns_when_dies`).
fn rebirth_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    lose_life(g, p, 2, Some(p), "other", None)?;
    if let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf && g.is_creature(t)) {
        super::sheoldred::grant_eot(g, t, "ret_dies_tapped");
        crate::glog!(g, "    Malakir Rebirth on {}", g.perm(t).name);
    }
    Ok("gy")
}

/// in response to destroy / damage / -X/-X removal or an edict aimed at p's nontoken creature m worth 3 or more: Malakir
/// Rebirth ({B}) on it, with the life to spare. The removal still resolves; the creature comes back tapped.
pub fn rebirth_protect(g: &mut Game, p: PlayerId, m: PermId, kind: Sym) -> Res<bool> {
    let Some(c) = held(g, p, REBIRTH) else { return Ok(false) };
    if !(kind == "destroy" || kind.starts_with("dmg") || kind.starts_with("shrink") || kind == "sac" || kind == "edict")
        || !live(g, m)
        || !g.is_creature(m)
        || g.perm(m).token
        || g.perm(m).eot_kw.contains(&"ret_dies_tapped")
        || indestructible(g, m) && kind == "destroy"
        || pval(g, m) < 3.0
        || g.player(p).life <= 6
        || !castable(g, p, c, "hand")
        || !can_pay(g, p, 0, "B", false)
    {
        return Ok(false);
    }
    if !pay(g, p, 0, "B", false)? {
        return Ok(false);
    }
    cast_card(g, p, c, "hand", Ctx { target: Some(m), ..Ctx::default() })
}

/// the back face, Malakir Mire: played as a land when it is the land drop's only use (as the other spell // lands)
fn rebirth_land_opt(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let pl = g.player(p);
    let allowed = 1 + crate::cardcode::extra_lands(g, p) + pl.extra_land_now;
    if post.is_none() || g.active != Some(p) || pl.lands_played >= allowed || !pl.hand.contains(&c) {
        return Ok(vec![]);
    }
    if pl.lands.len() >= 5 || pl.hand.iter().any(|&x| g.db.get(x).land) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.5,
        label: "play Malakir Mire as a land".into(),
        act: Some(Action::Plan { f: rebirth_land_go, arg: c.0 as i64 }),
    }])
}

fn rebirth_land_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let allowed = 1 + crate::cardcode::extra_lands(g, p) + g.player(p).extra_land_now;
    let Some(i) = g.player(p).hand.iter().position(|&x| x == c) else { return Ok(false) };
    if g.player(p).lands_played >= allowed {
        return Ok(false);
    }
    g.player_mut(p).hand.remove(i);
    crate::engine::turn::play_land_card(g, p, c, "plays")?;
    if let Some(&l) = g.player(p).lands.last().filter(|&&l| g.land(l).card() == c) {
        g.land_mut(l).tapped = true; // This land enters tapped.
    }
    Ok(true)
}

fn mire_cols(_g: &Game, _p: PlayerId, _l: LandId) -> Colors {
    Colors::from_letters("B")
}

// ================================================================== Mindslicer
/// When Mindslicer dies, each player discards their hand.
fn mindslicer_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "each player discards their hand", None)? {
        for q in apnap(g, o) {
            let hand = g.player(q).hand.clone();
            if !hand.is_empty() {
                crate::glog!(g, "    {} discards {} card(s) (Mindslicer)", pname(g, q), hand.len());
                discard_cards(g, q, &hand)?;
            }
        }
    }
    Ok(())
}

// ================================================================== Rankle, Master of Pranks
/// the modes Rankle's controller p chooses: (discard, lose 1 and draw, sacrifice), each when it costs the opponents
/// more than p
fn rankle_modes(g: &Game, p: PlayerId, rankle: PermId) -> (bool, bool, bool) {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let terg = if tergrid_mine(g, p) { 1.6 } else { 1.0 };
    let with_cards = opps.iter().filter(|&&q| !g.player(q).hand.is_empty()).count() as f64;
    let discard = with_cards * 0.8 * terg > if g.player(p).hand.is_empty() { 0.0 } else { 0.9 };
    let draw = opps.len() <= 2 && g.player(p).life >= 10;
    let worst = |q: PlayerId| {
        creatures_of(g, q).into_iter().filter(|&m| !no_sac(g, m)).map(|m| sac_worth(g, m)).fold(f64::MAX, f64::min)
    };
    let theirs: f64 = opps.iter().map(|&q| worst(q)).filter(|v| *v < f64::MAX).sum::<f64>() * terg;
    let mine = worst(p);
    let mine = if mine == f64::MAX { 0.0 } else { mine };
    // Rankle itself is p's only creature: the sacrifice takes it
    let only_rankle = creatures_of(g, p) == vec![rankle];
    let sac = theirs > mine + 0.5 && !(only_rankle && theirs < pval(g, rankle) + 1.0);
    (discard, draw, sac)
}

/// Whenever Rankle deals combat damage to a player, choose any number: each player discards a card; each player loses
/// 1 life and draws a card; each player sacrifices a creature.
fn rankle_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if a != src || p != owner(g, src) {
        return Ok(());
    }
    let (dis, drw, sac) = rankle_modes(g, p, src);
    if !(dis || drw || sac) || !trigger_window(g, p, Some(src), "Rankle's pranks", None)? {
        return Ok(());
    }
    let (dis, drw, sac) = rankle_modes(g, p, src);
    let order = apnap(g, p);
    if dis {
        for &q in &order {
            if g.player(q).alive && !g.player(q).hand.is_empty() {
                discard_worst(g, q, 1)?;
            }
        }
    }
    if drw {
        for &q in &order {
            lose_life(g, q, 1, Some(p), "other", None)?;
        }
        for &q in &order {
            if g.player(q).alive {
                draw(g, q, 1, false)?;
            }
        }
    }
    if sac {
        for &q in &order {
            if g.player(q).alive {
                edict(g, q, false)?;
            }
        }
    }
    check_state(g)
}

// ================================================================== Sangromancer
/// Whenever an opponent discards a card, you may gain 3 life.
fn sangro_discard(g: &mut Game, src: Src, q: PlayerId, _c: CardId) -> Res {
    let o = owner(g, src);
    if q != o && trigger_window(g, o, Some(src), "gain 3 life", None)? {
        gain(g, o, 3)?;
    }
    Ok(())
}

// ================================================================== The Raven Man
/// At the beginning of each end step, if a player discarded a card this turn, create a 1/1 black Bird creature token
/// with flying and "This token can't block."
fn raven_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let now = Some(g.turn_stamp());
    if !g.players.iter().any(|pl| pl.discarded_turn == now) {
        return Ok(());
    }
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "a 1/1 Bird that can't block", None)? {
        let mut spec =
            Tokens { fly: true, color: Some(Colors::from_letters("B")), types: vec!["bird"], ..Tokens::new(1, 1) };
        spec.data.set(DataKey::Kws, Val::List(vec![Val::Str("cant_block")]));
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Rhythm of the Wild")?;
    c.uncounterable = Some(rhythm_uncounterable);
    c.etb = Some(rhythm_riot);
    at_once(c, Event::Etb, true);
    r.card(db, PILFERER)?.gy_options = Some(pilferer_gy);
    let c = r.card(db, "Juri, Master of the Revue")?;
    c.sacrifice = Some(juri_sac);
    c.self_dies = Some(juri_dies);
    let c = r.card(db, "Marionette Apprentice")?;
    c.etb = Some(apprentice_etb);
    c.dies = Some(apprentice_dies);
    c.sacrifice = Some(apprentice_sac);
    r.card(db, "Mazirek, Kraul Death Priest")?.sacrifice = Some(mazirek_sac);
    let c = r.card(db, "Ruthless Technomancer")?;
    c.etb = Some(techno_etb);
    c.options = Some(techno_options);
    let c = r.card(db, "Smaug, Wicked Worm")?;
    c.etb = Some(smaug_etb);
    c.cast = Some(smaug_cast);
    // Tergrid
    let c = r.card(db, EDICT)?;
    c.resolve = Some(edict_resolve);
    c.alt_cost = Some(EDICT_ALT);
    c.hand_options = Some(edict_options);
    c.prio = Some(|_, _, _| 0);
    let c = r.card(db, "Bottomless Pit")?;
    c.upkeep = Some(pit_upkeep);
    c.prio = Some(pit_prio);
    r.card(db, IMPS)?.prio = Some(|_, _, _| 0); // a response only (`imps_mischief`)
    let c = r.card(db, "Pox")?;
    c.resolve = Some(pox_resolve);
    c.prio = Some(pox_prio);
    r.card(db, "Smallpox")?.prio = Some(smallpox_prio);
    let c = r.card(db, FLARE)?;
    c.resolve = Some(flare_resolve);
    c.hand_options = Some(flare_options);
    c.prio = Some(|_, _, _| 0);
    walker(r, db, "Liliana, Waker of the Dead", &LILIANA)?;
    let c = r.card(db, REBIRTH)?;
    c.resolve = Some(rebirth_resolve);
    c.hand_options = Some(rebirth_land_opt);
    c.land_cols = Some(mire_cols);
    c.prio = Some(|_, _, _| 0); // a response only (`rebirth_protect`)
    r.card(db, "Mindslicer")?.self_dies = Some(mindslicer_dies);
    r.card(db, "Rankle, Master of Pranks")?.combat_damage = Some(rankle_damage);
    r.card(db, "Sangromancer")?.discard = Some(sangro_discard);
    r.card(db, "The Raven Man")?.end_step = Some(raven_end);
    Ok(())
}
