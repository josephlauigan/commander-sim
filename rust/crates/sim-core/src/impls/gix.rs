//! The gix candidate deck (decklists/JD/candidates/gix.md): mono-black creatures that connect, each hit a card with
//! Gix, Yawgmoth Praetor. Its card code and AI plan. Several cards are shared with the Yawgmoth and B/R candidates
//! (this module models them for all three).
//!
//! The cards (each is Full in `cardgen audit --mine`; the verdicts are in data/card_overrides.toml):
//! - **Al Bhed Salvagers**: it or another creature or artifact you control dies: the weakest opponent loses 1, you gain
//!   1. Artifact tokens count (the engine keeps Clues, Treasures, Foods and our Blood tokens as counts: their
//!   sacrifice is the `sacrifice` event's `Sacrificed::Token`).
//! - **Necrosquito**: enters with two oil counters (`DataKey::Oil`, +1/+1 each through `self_pt`) and gets one for each
//!   other creature or artifact you control put into a graveyard (same events as Al Bhed).
//! - **Callous Inspector** (1 damage to you and a Clue when it dies), **Apex Witchstalker** (2 life on entering and
//!   dying; basic landcycling {2}), **Corpulent Corpse** (fear in `rules::evasion_blocked`; suspend 5 for {B} through
//!   the engine's suspend table, with haste when it comes out), **Dark Matter Manipulator** (the mill is compiled; +2/+0
//!   for every seven cards in the graveyard), **Gollum, Silent Slinker** (Meager Meal: +1/+1 counter and 2 life),
//!   **Grim Wanderer** (cast only if a creature died this turn: its priority is 0 otherwise), **Restless Bloodseeker**
//!   (Blood tokens as `Player::bloods`, the transform and Bloodsoaked Reveler's drain), **Salvaged Manaworker** (the
//!   colour filter in `engine::mana`), **Sangrophage** (2 life at upkeep or tapped), **Balustrade Spy**, **Deepcavern
//!   Imp** (echo by discarding), **Gravblade Heavy**, **Ulna Alley Shopkeep** (infusion), **Key to the Side-Door**
//!   (the unblockable ability; the discard-a-legend draw can't happen in a singleton deck).
//! - Spells: **Overkill** (-0/-9999: tags `rem=shrink9999`), **Depressurize** (destroys a creature whose power is 3 or
//!   less: `legal_targets` filters by name; the -3/-0 alone is a combat trick the AI never casts it for),
//!   **End of the Hunt** (an edict on the greatest mana value).
//! - Equipment (**Medic's Kitesail**, **Mask of Memory**): compiled abilities in data/card_overrides.toml; the equip
//!   choice prefers a creature that connects.
//!
//! The AI plan: creatures that connect first (evasion: flying, menace, fear), equipment once there is a creature,
//! Gix's draw only while life is above 12, and his {4}{B}{B}{B} late, with excess lands (see `partials.rs`).

use crate::ai::pool::generic_prio;
use crate::cards::{CardDb, Types};
use crate::engine::cast::{discard_worst, on_cast};
use crate::engine::combat::{can_block, kw};
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::card_worth;
use crate::engine::values::{epow, pval};
use crate::engine::zones::{die, discard_cards, draw, exile_perm, land_to_hand, max_by, mill, min_by};
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Step, Val};
use crate::sym::Sym;
use crate::tag::Tag;

const SALVAGERS: &str = "Al Bhed Salvagers";
const NECRO: &str = "Necrosquito";
const BLOODSEEKER: &str = "Restless Bloodseeker";
const MANAWORKER: &str = "Salvaged Manaworker";
const CORPSE: &str = "Corpulent Corpse";
const GOLLUM: &str = "Gollum, Silent Slinker";

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn on_bf(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// did p gain life this turn?
pub fn gained_life(g: &Game, p: PlayerId) -> bool {
    let st = g.turn_stamp();
    g.player(p).gained_turn.is_some_and(|x| x.0 == st && x.1 > 0)
}

/// the artifacts p controls: the permanents (cards and artifact tokens) plus the Treasure, Food, Clue and Blood tokens
pub fn artifact_count(g: &Game, p: PlayerId) -> i32 {
    let pl = g.player(p);
    let perms = pl
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            !x.phased
                && (x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)) || x.ttypes.contains(&"artifact"))
        })
        .count();
    (perms as u32 + pl.treasures + pl.foods + pl.clues + pl.bloods) as i32
}

/// a creature or artifact (a permanent that has just left the battlefield keeps its card)
fn creature_or_artifact(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    g.is_creature(m) || x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)) || x.ttypes.contains(&"artifact")
}

// ------------------------------------------------------------------ Al Bhed Salvagers
/// the weakest opponent (lowest life, the first of equals)
fn weakest_opp(g: &Game, p: PlayerId) -> Option<PlayerId> {
    g.opps(p).min_by_key(|&q| g.player(q).life)
}

fn salvagers_drain(g: &mut Game, src: Src) -> Res {
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "drain 1", None)? {
        return Ok(());
    }
    if let Some(q) = weakest_opp(g, o) {
        lose_life(g, q, 1, Some(o), "drain", Some(false))?;
    }
    gain(g, o, 1)
}

/// another creature or artifact you control dies
fn salvagers_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m != src && owner(g, m) == owner(g, src) && creature_or_artifact(g, m) {
        salvagers_drain(g, src)?;
    }
    Ok(())
}

/// the Salvagers themselves die
fn salvagers_self(g: &mut Game, src: Src, _m: PermId, _cause: Sym) -> Res {
    salvagers_drain(g, src)
}

/// a Clue, Treasure, Food or Blood token of its controller is sacrificed
fn salvagers_token(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p == owner(g, src) && matches!(what, Sacrificed::Token(_)) && on_bf(g, src) {
        salvagers_drain(g, src)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Necrosquito
fn oil(g: &Game, m: PermId) -> i32 {
    g.perm(m).data.int(DataKey::Oil) as i32
}

/// as it enters (before the state check sees a 0/0): two oil counters, and the size rule on
fn necro_as_enters(g: &mut Game, _p: PlayerId, m: PermId) -> Res {
    g.selfpt = true;
    g.perm_mut(m).data.set(DataKey::Oil, Val::Int(2));
    Ok(())
}

fn selfpt_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

fn selfpt_as(g: &mut Game, _p: PlayerId, _m: PermId) -> Res {
    g.selfpt = true;
    Ok(())
}

fn necro_oil(g: &mut Game, src: Src) -> Res {
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "put an oil counter on it", Some(1.0))? && on_bf(g, src) {
        let n = oil(g, src) + 1;
        g.perm_mut(src).data.set(DataKey::Oil, Val::Int(n as i64));
    }
    Ok(())
}

fn necro_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m != src && owner(g, m) == owner(g, src) && creature_or_artifact(g, m) {
        necro_oil(g, src)?;
    }
    Ok(())
}

fn necro_token(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p == owner(g, src) && matches!(what, Sacrificed::Token(_)) && on_bf(g, src) {
        necro_oil(g, src)?;
    }
    Ok(())
}

fn necro_pt(g: &Game, _p: PlayerId, m: PermId) -> (i32, i32) {
    let k = oil(g, m);
    (k, k)
}

// ------------------------------------------------------------------ Callous Inspector
/// when it dies it deals 1 damage to you and you create a Clue
fn inspector_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, m);
    if trigger_window(g, o, Some(m), "1 damage to you and a Clue", None)? {
        lose_life(g, o, 1, Some(o), "burn", None)?;
        crate::engine::zones::make_artifact_tokens(g, o, "Clue", 1)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Apex Witchstalker
fn apex_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "gain 2 life", None)? {
        gain(g, p, 2)?;
    }
    Ok(())
}

fn apex_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, m);
    if trigger_window(g, o, Some(m), "gain 2 life", None)? {
        gain(g, o, 2)?;
    }
    Ok(())
}

/// basic landcycling {2}: when the hand has no land to play and the Witchstalker can't be cast soon
fn apex_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
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
        label: "Apex Witchstalker (basic landcycling)".into(),
        act: Some(Action::Plan { f: apex_cycle, arg: c.0 as i64 }),
    }])
}

fn apex_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    crate::glog!(g, "  {} basic landcycles Apex Witchstalker", pname(g, p));
    discard_cards(g, p, &[c])?;
    land_to_hand(g, p)?;
    Ok(true)
}

// ------------------------------------------------------------------ Corpulent Corpse
/// suspend 5 for {B}: the AI suspends it in the main phase with a black mana to spare (it is not worth six mana)
fn corpse_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "B", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.1,
        label: "suspend Corpulent Corpse".into(),
        act: Some(Action::Plan { f: corpse_go, arg: c.0 as i64 }),
    }])
}

fn corpse_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    let pl = g.player_mut(p);
    let i = pl.hand.iter().position(|&x| x == c).unwrap();
    pl.hand.remove(i);
    pl.exile.push(c);
    pl.suspended.push((c, 5));
    crate::glog!(g, "  {} suspends Corpulent Corpse (5)", pname(g, p));
    Ok(true)
}

/// it came out of suspend (it enters during its owner's upkeep, where nothing else casts a creature): haste
fn corpse_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && g.active == Some(p) && g.step == Step::Start {
        let x = g.perm_mut(src);
        x.sick = false;
        if !x.eot_kw.contains(&"haste") {
            x.eot_kw.push("haste");
        }
    }
    Ok(())
}

/// hard-cast for six only with mana to spare; suspending is the plan
fn corpse_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).lands.len() >= 8 { 38 } else { 12 }
}

// ------------------------------------------------------------------ size rules: Dark Matter, Gravblade, Ulna Alley, Bloodseeker
fn dark_matter_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    (2 * (g.player(p).gy.len() / 7) as i32, 0)
}

fn gravblade_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    (i32::from(artifact_count(g, p) > 0), 0)
}

/// deathtouch is read from the permanent's flag in combat: keep it in step with the artifact count
fn gravblade_sba(g: &mut Game, src: Src) -> Res {
    let want = artifact_count(g, owner(g, src)) > 0;
    if g.perm(src).dt != want {
        g.perm_mut(src).dt = want;
    }
    Ok(())
}

fn gravblade_kw(g: &Game, src: Src, m: PermId, k: Sym) -> bool {
    m == src && k == "deathtouch" && artifact_count(g, owner(g, src)) > 0
}

fn ulna_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    (if gained_life(g, p) { 2 } else { 0 }, 0)
}

fn flipped(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Flipped)
}

fn bloodseeker_pt(g: &Game, _p: PlayerId, m: PermId) -> (i32, i32) {
    (if flipped(g, m) { 2 } else { 0 }, 0)
}

// ------------------------------------------------------------------ Sangrophage, Deepcavern Imp
/// tap it unless you pay 2 life: the AI pays while life is comfortable
fn sangro_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !on_bf(g, src) {
        return Ok(());
    }
    let pay_it = g.player(p).life > 14;
    if trigger_window(g, p, Some(src), "tap it unless you pay 2 life", Some(1.0))? && on_bf(g, src) {
        if pay_it {
            lose_life(g, p, 2, Some(p), "other", None)?;
        } else {
            g.perm_mut(src).tapped = true;
        }
    }
    Ok(())
}

/// echo, the first upkeep after it came under your control: discard a card (the AI pays with its least useful card
/// when that is junk or the hand is full) or sacrifice it
fn imp_echo(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !on_bf(g, src) || g.perm(src).data.get(DataKey::Echo) == Some(&Val::Str("done")) {
        return Ok(());
    }
    g.perm_mut(src).data.set(DataKey::Echo, Val::Str("done"));
    if !trigger_window(g, p, Some(src), "echo: discard a card or sacrifice it", Some(2.0))? || !on_bf(g, src) {
        return Ok(());
    }
    let hand = g.player(p).hand.clone();
    let worst = min_by(&hand, |c| card_worth(g, p, c, false));
    let pays = worst.is_some_and(|c| card_worth(g, p, c, false) < 40.0 || hand.len() >= 4);
    if pays {
        crate::glog!(g, "    {} pays echo for Deepcavern Imp", pname(g, p));
        discard_worst(g, p, 1)?;
    } else {
        crate::glog!(g, "    {} doesn't pay echo for Deepcavern Imp", pname(g, p));
        die(g, src, "sac")?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Balustrade Spy
/// the target player reveals cards from the top of their library until a land, and puts those cards into their
/// graveyard: the AI mills itself when a graveyard payoff is out (Dark Matter Manipulator), else the opponent with
/// the biggest library
fn spy_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || !trigger_window(g, p, Some(src), "a player reveals until a land and mills those cards", None)? {
        return Ok(());
    }
    let payoff = g
        .player(p)
        .perms
        .iter()
        .any(|&x| on_bf(g, x) && g.perm(x).cd.is_some_and(|c| &*g.db.get(c).name == "Dark Matter Manipulator"));
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let target = if payoff && !g.player(p).library.is_empty() {
        p
    } else {
        max_by(&opps, |q| g.player(q).library.len() as f64).unwrap_or(p)
    };
    let mut n = 0;
    for &c in g.player(target).library.iter().rev() {
        n += 1;
        if g.db.get(c).land {
            break;
        }
    }
    crate::glog!(g, "    Balustrade Spy mills {n} cards of {}", pname(g, target));
    mill(g, target, n)
}

// ------------------------------------------------------------------ Gollum: Meager Meal
fn meal_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || g.player(p).adv_done.contains(&c) || !can_pay(g, p, 0, "B", false)
    {
        return Ok(vec![]);
    }
    if !g.player(p).perms.iter().any(|&m| on_bf(g, m) && g.is_creature(m)) {
        return Ok(vec![]);
    }
    let payoff = g.player(p).perms.iter().any(|&m| {
        on_bf(g, m) && g.perm(m).cd.is_some_and(|cd| matches!(&*g.db.get(cd).name, BLOODSEEKER | "Ulna Alley Shopkeep"))
    });
    Ok(vec![Opt {
        utility: 1.4 + if payoff { 1.2 } else { 0.0 },
        label: "Gollum adventure: Meager Meal".into(),
        act: Some(Action::Plan { f: meal_go, arg: c.0 as i64 }),
    }])
}

fn meal_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    on_cast(g, p, c)?;
    crate::glog!(g, "  {} casts Meager Meal", pname(g, p));
    // a +1/+1 counter on the creature that connects best, and 2 life
    let cands: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| on_bf(g, m) && g.is_creature(m)).collect();
    if let Some(t) = max_by(&cands, |m| attacker_value(g, m)) {
        g.perm_mut(t).plus += 1;
    }
    gain(g, p, 2)?;
    if !g.player(p).adv_done.contains(&c) {
        g.player_mut(p).adv_done.push(c);
    }
    Ok(true)
}

// ------------------------------------------------------------------ Restless Bloodseeker and Blood tokens
/// at the beginning of your end step, if you gained life this turn: a Blood token
fn seeker_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src)
        && on_bf(g, src)
        && gained_life(g, p)
        && trigger_window(g, p, Some(src), "create a Blood token", Some(1.0))?
    {
        make_blood(g, p, 1)?;
    }
    Ok(())
}

/// n Blood tokens (a count on the player, like Clues)
pub fn make_blood(g: &mut Game, p: PlayerId, n: i32) -> Res {
    let n = n * crate::dsl::token_mult(g, p) as i32;
    if n <= 0 {
        return Ok(());
    }
    g.player_mut(p).bloods += n as u32;
    if !g.hooks.is_empty() {
        crate::engine::hooks::fire_trigger(
            g,
            Event::TokenCreated,
            Call::ArtifactTokens { p, kinds: vec!["Blood"], n },
        )?;
    }
    Ok(())
}

fn sac_blood(g: &mut Game, p: PlayerId, n: u32) -> Res<bool> {
    if g.player(p).bloods < n {
        return Ok(false);
    }
    g.player_mut(p).bloods -= n;
    for _ in 0..n {
        if !g.hooks.is_empty() {
            crate::engine::hooks::fire_trigger(
                g,
                Event::Sacrifice,
                Call::Sacrifice { p, what: Sacrificed::Token("Blood") },
            )?;
        }
    }
    Ok(true)
}

/// the Bloodseeker's plays: sacrifice two Blood tokens to transform (sorcery speed); once Bloodsoaked Reveler,
/// {4}{B}: each opponent loses 2 life and you gain 2; and a Blood's own ability ({1}, {T}, discard a card, sacrifice:
/// draw a card) when the hand has a card to spare
fn seeker_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if owner(g, src) != p || !on_bf(g, src) || g.perm(src).neutered {
        return Ok(out);
    }
    if !flipped(g, src) {
        if post == Some(false) && g.active == Some(p) && g.player(p).bloods >= 2 {
            out.push(Opt {
                utility: 2.2,
                label: "Restless Bloodseeker: transform".into(),
                act: Some(Action::Plan { f: seeker_flip, arg: src.0 as i64 }),
            });
        }
    } else if post.is_some() && can_pay(g, p, 4, "B", false) {
        if let Some(q) = weakest_opp(g, p) {
            out.push(Opt {
                utility: if g.player(q).life <= 2 { 9.0 } else { 1.0 },
                label: "Bloodsoaked Reveler: drain 2".into(),
                act: Some(Action::Plan { f: reveler_drain, arg: src.0 as i64 }),
            });
        }
    }
    // a Blood's rummage: discard a card of no use to draw a new one
    if g.player(p).bloods > 0
        && can_pay(g, p, 1, "", false)
        && !g.player(p).hand.is_empty()
        && !g.player(p).library.is_empty()
    {
        let hand = g.player(p).hand.clone();
        let worst = min_by(&hand, |c| card_worth(g, p, c, false));
        if worst.is_some_and(|c| card_worth(g, p, c, false) < 15.0) {
            out.push(Opt {
                utility: 1.1,
                label: "rummage with a Blood".into(),
                act: Some(Action::Plan { f: blood_rummage, arg: 0 }),
            });
        }
    }
    Ok(out)
}

fn seeker_flip(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !on_bf(g, m) || flipped(g, m) || g.player(p).bloods < 2 {
        return Ok(false);
    }
    sac_blood(g, p, 2)?;
    if ability_window(g, p, Some(m), "transform", Some(2.0), None)? && on_bf(g, m) {
        g.perm_mut(m).data.set(DataKey::Flipped, Val::Bool(true));
        g.selfpt = true;
        crate::glog!(g, "  Restless Bloodseeker transforms into Bloodsoaked Reveler");
    }
    Ok(true)
}

fn reveler_drain(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !on_bf(g, m) || !flipped(g, m) || !can_pay(g, p, 4, "B", false) {
        return Ok(false);
    }
    pay(g, p, 4, "B", false)?;
    if ability_window(g, p, Some(m), "each opponent loses 2 life and you gain 2", Some(3.0), None)? {
        let opps: Vec<PlayerId> = g.opps(p).collect();
        for q in opps {
            lose_life(g, q, 2, Some(p), "drain", Some(false))?;
        }
        gain(g, p, 2)?;
    }
    Ok(true)
}

fn blood_rummage(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.player(p).bloods == 0 || g.player(p).hand.is_empty() || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    discard_worst(g, p, 1)?;
    sac_blood(g, p, 1)?;
    draw(g, p, 1, false)?;
    Ok(true)
}

// ------------------------------------------------------------------ Salvaged Manaworker
/// the Manaworker that can filter mana this turn ({1}: add one mana of any colour, once each turn); `engine::mana`
/// turns one single mana into any colour of the deck's identity when a colour is missing
pub fn mana_fixer(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.on_bf
            && !x.phased
            && !x.neutered
            && !x.eot_kw.contains(&"mw_used")
            && x.cd.is_some_and(|c| &*g.db.get(c).name == MANAWORKER)
    })
}

/// the Manaworker was used to filter a mana: no second use this turn
pub fn mana_fixer_used(g: &mut Game, m: PermId) {
    g.perm_mut(m).eot_kw.push("mw_used");
}

/// the Manaworker has no enter effect; the slot only puts it among the hooked permanents (`mana_fixer` is read by
/// `engine::mana` only while some card hooks)
fn manaworker_etb(_g: &mut Game, _src: Src, _p: PlayerId, _m: PermId) -> Res {
    Ok(())
}

// ------------------------------------------------------------------ Key to the Side-Door
/// the creature the Key makes unblockable: the best attacker that could be blocked
fn key_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            on_bf(g, m)
                && g.is_creature(m)
                && !x.tapped
                && !x.noatk
                && (!x.sick || kw(g, m, "haste") || x.eot_kw.contains(&"haste"))
                && epow(g, m) >= 2
                && !kw(g, m, "unblockable")
                && g.opps(p).any(|q| {
                    g.player(q)
                        .perms
                        .iter()
                        .any(|&b| on_bf(g, b) && g.is_creature(b) && !g.perm(b).tapped && can_block(g, b, m))
                })
        })
        .collect();
    max_by(&cands, |m| attacker_value(g, m))
}

fn key_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p
        || post != Some(false)
        || g.active != Some(p)
        || !on_bf(g, src)
        || g.perm(src).tapped
        || g.perm(src).neutered
        || !can_pay(g, p, 2, "", false)
    {
        return Ok(vec![]);
    }
    let Some(t) = key_target(g, p) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 1.2 + 0.2 * epow(g, t) as f64,
        label: format!("Key to the Side-Door: {} can't be blocked", g.perm(t).name),
        act: Some(Action::Plan { f: key_go, arg: ((src.0 as i64) << 32) | t.0 as i64 }),
    }])
}

fn key_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (k, t) = (PermId((arg >> 32) as u32), PermId((arg & 0xffff_ffff) as u32));
    if !on_bf(g, k) || g.perm(k).tapped || !on_bf(g, t) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    g.perm_mut(k).tapped = true;
    if ability_window(g, p, Some(k), "target creature can't be blocked this turn", Some(2.0), Some(t))? && on_bf(g, t) {
        g.dsl_on = true; // the engine reads "unblockable" only while the ability language is on
        let e = &mut g.perm_mut(t).eot_kw;
        if !e.contains(&"unblockable") {
            e.push("unblockable");
        }
        crate::glog!(g, "    {} can't be blocked this turn", g.perm(t).name);
    }
    Ok(true)
}

// ------------------------------------------------------------------ End of the Hunt
/// the opponent whose greatest-mana-value creature or planeswalker is worth the most loses it; they exile the least
/// valuable among those tied for the greatest mana value
fn hunt_pick(g: &Game, p: PlayerId) -> Option<(PlayerId, PermId)> {
    let mut best: Option<(PlayerId, PermId, f64)> = None;
    for q in g.opps(p) {
        let cands: Vec<PermId> = g
            .player(q)
            .perms
            .iter()
            .copied()
            .filter(|&m| {
                on_bf(g, m)
                    && (g.is_creature(m) || g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER)))
            })
            .collect();
        let mv = |m: PermId| g.perm(m).cd.map_or(0, |c| g.db.get(c).cmc);
        let Some(top) = cands.iter().map(|&m| mv(m)).max() else { continue };
        let tied: Vec<PermId> = cands.into_iter().filter(|&m| mv(m) == top).collect();
        let Some(m) = min_by(&tied, |m| pval(g, m)) else { continue };
        let v = pval(g, m);
        if best.is_none_or(|b| v > b.2) {
            best = Some((q, m, v));
        }
    }
    best.map(|b| (b.0, b.1))
}

fn hunt_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &crate::state::Ctx) -> Res<Sym> {
    if let Some((q, m)) = hunt_pick(g, p) {
        crate::glog!(g, "    {} exiles {}", pname(g, q), g.perm(m).name);
        exile_perm(g, m)?;
    }
    Ok("gy")
}

fn hunt_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match hunt_pick(g, p) {
        Some((_, m)) if pval(g, m) >= 2.0 => (45.0 + 4.0 * pval(g, m)).min(75.0) as i32,
        _ => 0,
    }
}

// ------------------------------------------------------------------ Grim Wanderer
/// "cast this spell only if a creature died this turn": a 5/3 flash for two
fn wanderer_prio(g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    if g.died_turn == Some(g.turn_stamp()) { 60 } else { 0 }
}

// ------------------------------------------------------------------ the AI plan
/// how well a creature connects: evasion first, then power
pub fn attacker_value(g: &Game, m: PermId) -> f64 {
    let x = g.perm(m);
    let mut v = epow(g, m) as f64;
    if x.fly || kw(g, m, "flying") || x.eot_kw.contains(&"flying") {
        v += 4.0;
    }
    if kw(g, m, "menace") {
        v += 2.0;
    }
    if kw(g, m, "fear") {
        v += 3.0;
    }
    if kw(g, m, "unblockable") {
        v += 5.0;
    }
    v
}

fn evasive_card(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.creature && (d.has_kw("flying") || d.has_kw("menace") || d.has_kw("fear") || d.tag(Tag::Fly))
}

/// gix's cast priorities: creatures that connect first, equipment and the Key once there is a creature to carry
/// them, the rest by the generic priorities
pub fn gix_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    let base = generic_prio(g, p, c);
    match &*d.name {
        "Medic's Kitesail" | "Mask of Memory" | "Key to the Side-Door" => {
            if g.player(p).perms.iter().any(|&m| on_bf(g, m) && g.is_creature(m)) { base.max(46) } else { 0 }
        }
        _ if base > 0 && evasive_card(g, c) => base + 8,
        _ => base,
    }
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, SALVAGERS)?;
    c.dies = Some(salvagers_dies);
    c.self_dies = Some(salvagers_self);
    c.sacrifice = Some(salvagers_token);
    let c = r.card(db, NECRO)?;
    c.as_enters = Some(necro_as_enters);
    c.etb = Some(selfpt_etb);
    c.self_pt = Some(necro_pt);
    c.dies = Some(necro_dies);
    c.sacrifice = Some(necro_token);
    r.card(db, "Callous Inspector")?.self_dies = Some(inspector_dies);
    let c = r.card(db, "Apex Witchstalker")?;
    c.etb = Some(apex_etb);
    c.self_dies = Some(apex_dies);
    c.hand_options = Some(apex_hand);
    let c = r.card(db, CORPSE)?;
    c.hand_options = Some(corpse_hand);
    c.etb = Some(corpse_etb);
    c.prio = Some(corpse_prio);
    let c = r.card(db, "Dark Matter Manipulator")?;
    c.etb = Some(selfpt_etb);
    c.as_enters = Some(selfpt_as);
    c.self_pt = Some(dark_matter_pt);
    let c = r.card(db, "Gravblade Heavy")?;
    c.etb = Some(selfpt_etb);
    c.as_enters = Some(selfpt_as);
    c.self_pt = Some(gravblade_pt);
    c.sba = Some(gravblade_sba);
    c.grant_kw = Some(gravblade_kw);
    let c = r.card(db, "Ulna Alley Shopkeep")?;
    c.etb = Some(selfpt_etb);
    c.as_enters = Some(selfpt_as);
    c.self_pt = Some(ulna_pt);
    let c = r.card(db, BLOODSEEKER)?;
    c.etb = Some(selfpt_etb);
    c.as_enters = Some(selfpt_as);
    c.self_pt = Some(bloodseeker_pt);
    c.end_step = Some(seeker_end);
    c.options = Some(seeker_options);
    r.card(db, "Sangrophage")?.upkeep = Some(sangro_upkeep);
    r.card(db, "Deepcavern Imp")?.upkeep = Some(imp_echo);
    r.card(db, "Balustrade Spy")?.etb = Some(spy_etb);
    r.card(db, GOLLUM)?.hand_options = Some(meal_hand);
    r.card(db, MANAWORKER)?.etb = Some(manaworker_etb);
    r.card(db, "Key to the Side-Door")?.options = Some(key_options);
    let c = r.card(db, "End of the Hunt")?;
    c.resolve = Some(hunt_resolve);
    c.prio = Some(hunt_prio);
    r.card(db, "Grim Wanderer")?.prio = Some(wanderer_prio);
    Ok(())
}
