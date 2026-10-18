//! Tier 6 "godo" batch: the cards of JD's Godo, Bandit Warlord deck (Godo + Helm of the Host extra combats) that the
//! compiled Oracle text and the engine's tags don't cover (the audit verdicts are in data/card_overrides.toml; each
//! card's doc comment says how it is modeled and what is left out; what needs the shared engine is in
//! rust/agents/engine-needs-godo.md). Helm of the Host itself is already modeled (`t3::helm`, `combat_start`).

use crate::cards::{CardDb, Types};
use crate::engine::mana::{can_pay, pay};
use crate::engine::cast::discard_worst;
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::shuffle_library;
use crate::engine::values::once_per_turn;
use crate::engine::values::epow;
use crate::engine::zones::{Enter, die, enter, max_by, min_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::human::human_of;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};
use crate::tag::Tag;
use super::t5::is_artifact;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn name_of(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

fn is_equipment_card(g: &Game, c: CardId) -> bool {
    g.db.get(c).has_subtype("equipment")
}

fn on_bf_named(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| live(g, m) && name_of(g, m) == name)
}

fn first_attack_key(src: PermId) -> Sym {
    intern(&format!("godo_first_attack_{}", src.0))
}

// ================================================================== Godo, Bandit Warlord
/// the copy cards that make Godo's second fetch worth Hammer first (the spec's line 3): on the battlefield or in hand
const COPIERS: [&str; 6] = [
    "Peter Parker's Camera",
    "Twinflame",
    "Molten Duplication",
    "Heat Shimmer",
    "Electroduplicate",
    "Cursed Mirror",
];

/// which Equipment Godo fetches: Hammer of Nazahn first when a copier is available (the copy then fetches Helm and
/// Hammer attaches it free), else Helm of the Host, else Hammer, else the AI's usual pick (None)
fn godo_wants(g: &Game, p: PlayerId) -> Option<&'static str> {
    let lib = |n: &str| g.player(p).library.iter().any(|&c| &*g.db.get(c).name == n);
    let copier = COPIERS.iter().any(|n| {
        on_bf_named(g, p, n) || g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == *n)
    });
    let helm = on_bf_named(g, p, "Helm of the Host");
    let hammer = on_bf_named(g, p, "Hammer of Nazahn");
    if copier && !hammer && !helm && lib("Hammer of Nazahn") {
        Some("Hammer of Nazahn")
    } else if !helm && lib("Helm of the Host") {
        Some("Helm of the Host")
    } else if !hammer && lib("Hammer of Nazahn") {
        Some("Hammer of Nazahn")
    } else {
        None
    }
}

/// **Godo, Bandit Warlord** ({5}{R}, 3/3, legendary Human Barbarian): when Godo enters (the card or Helm's token
/// copy), search the library for an Equipment, put it onto the battlefield, shuffle (the enter triggers of that
/// Equipment, Hammer's attach, run as it enters). Whenever Godo attacks for the first time each turn, untap it and
/// every Samurai, and there is an additional combat phase (`extra_combats`, which the engine caps at four combats a
/// turn: Godo's loop needs more, see engine-needs-godo.md).
fn godo_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "search for an Equipment, put it onto the battlefield", Some(6.0))? {
        return Ok(());
    }
    let want = godo_wants(g, o);
    let pred = move |g: &Game, c: CardId| match want {
        Some(n) => &*g.db.get(c).name == n,
        None => is_equipment_card(g, c),
    };
    let got = super::t1::tutor_named(g, o, &pred, 1, "battlefield")?;
    if human_of(g, o).is_some() {
        // the human's pick may still be in the library list
        for &c in &got {
            let lib = &mut g.player_mut(o).library;
            if let Some(i) = lib.iter().position(|&x| x == c) {
                lib.remove(i);
            }
        }
        shuffle_library(g, o);
    }
    for c in got {
        crate::glog!(g, "    Godo puts {} onto the battlefield", g.db.get(c).name);
        enter(g, o, c, Enter { sick: false, ..Enter::default() })?;
    }
    Ok(())
}

fn godo_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p
        || !atk.contains(&src)
        || g.player(p).flag_turn.get(first_attack_key(src)) == Some(&g.turn_stamp())
    {
        return Ok(vec![]);
    }
    let ok = trigger_window(g, p, Some(src), "untap Godo and the Samurai, additional combat", Some(6.0))?;
    once_per_turn(g, p, first_attack_key(src));
    if !ok {
        return Ok(vec![]);
    }
    for m in g.player(p).perms.clone() {
        if live(g, m) && g.is_creature(m) && (m == src || super::tymna_cards::is_samurai(g, m)) {
            g.perm_mut(m).tapped = false;
        }
    }
    g.player_mut(p).extra_combats += 1;
    crate::glog!(g, "    Godo untaps; an additional combat phase follows");
    Ok(vec![])
}


// ================================================================== Hammer of Nazahn
/// the creature an Equipment is attached to: Godo first (Helm's token copies the equipped creature, so Godo is the
/// one to wear it), else the strongest creature that can attack (not summoning sick), else the strongest
fn attach_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m) && owner(g, m) == p).collect();
    max_by(&cs, |m| {
        let godo = if name_of(g, m) == "Godo, Bandit Warlord" { 100.0 } else { 0.0 };
        godo + if g.perm(m).sick { 0.0 } else { 10.0 } + epow(g, m) as f64
    })
}

/// **Hammer of Nazahn** ({4} legendary Equipment): whenever Hammer or another Equipment you control enters, you may
/// attach that Equipment to target creature you control (the AI always does, to `attach_target`); equipped creature
/// gets +2/+0 and indestructible, equip {4} (the compiled abilities).
fn hammer_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if !live(g, src) || !live(g, m) || owner(g, m) != o || !g.perm(m).cd.is_some_and(|c| is_equipment_card(g, c)) {
        return Ok(());
    }
    let Some(t) = attach_target(g, o) else { return Ok(()) };
    let name = format!("attach {} to {}", g.perm(m).name, g.perm(t).name);
    if !trigger_window(g, o, Some(src), &name, Some(5.0))? {
        return Ok(());
    }
    if live(g, m) && live(g, t) && g.is_creature(t) {
        super::cloud::attach(g, m, t);
    }
    Ok(())
}

// ================================================================== Magnetic Theft
/// the Equipment worth moving and the creature to put it on: an Equipment of p's that is not attached to the
/// creature `attach_target` picks (Helm of the Host first), and that creature
fn theft_plan(g: &Game, p: PlayerId) -> Option<(PermId, PermId)> {
    let t = attach_target(g, p)?;
    let es: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&e| {
            live(g, e)
                && g.perm(e).attached != Some(t)
                && g.perm(e).cd.is_some_and(|c| is_equipment_card(g, c))
                && (g.perm(e).attached.is_none() || name_of(g, e) == "Helm of the Host")
        })
        .collect();
    let e = max_by(&es, |e| if name_of(g, e) == "Helm of the Host" { 100.0 } else { epow(g, e) as f64 })?;
    Some((e, t))
}

/// **Magnetic Theft** ({R} instant): attach target Equipment to target creature (control of the Equipment doesn't
/// change; the AI targets its own Equipment and creature only). The AI casts it when it has an Equipment to move, in
/// the Godo deck mostly Helm of the Host onto Godo for {R} instead of the equip cost.
fn theft_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if let Some((e, t)) = theft_plan(g, p) {
        super::cloud::attach(g, e, t);
    }
    Ok("gy")
}

fn theft_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if theft_plan(g, p).is_some() { 70 } else { 0 }
}

// ================================================================== Sting, the Glinting Dagger
/// **Sting, the Glinting Dagger** ({2} legendary Equipment): equipped creature gets +1/+1 and haste, equip {2} (the
/// compiled abilities); at the beginning of each combat, untap the equipped creature (modeled for its controller's
/// combat, through `combat_start`, which the engine runs after attackers are chosen; so an attacking Godo untaps at
/// once and the extra-combat untap is already the engine's). Missing, as engine needs: the beginning of an
/// opponent's combat, and "first strike while blocking or blocked by a Goblin or Orc".
fn sting_combat(g: &mut Game, src: Src, p: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !live(g, src) {
        return Ok(vec![]);
    }
    let Some(t) = g.perm(src).attached.filter(|&t| live(g, t) && g.is_creature(t) && g.perm(t).tapped) else {
        return Ok(vec![]);
    };
    let name = format!("untap {}", g.perm(t).name);
    if trigger_window(g, p, Some(src), &name, Some(3.0))? && live(g, t) {
        g.perm_mut(t).tapped = false;
        crate::glog!(g, "    Sting untaps {}", g.perm(t).name);
    }
    Ok(vec![])
}

// ================================================================== Goblin Welder / Goblin Engineer
fn is_artifact_card(g: &Game, c: CardId) -> bool {
    g.db.get(c).types.has(Types::ARTIFACT)
}

/// how much the AI wants an artifact card on the battlefield: its mana value, plus a lot for the two Equipment the
/// Godo loop is built on
fn art_value(g: &Game, c: CardId) -> f64 {
    let d = g.db.get(c);
    d.cmc as f64 + if matches!(&*d.name, "Helm of the Host" | "Hammer of Nazahn") { 5.0 } else { 0.0 }
}

/// the artifact p should give up: the cheapest, a tapped (spent) one first; None if p has none
fn sac_candidate(g: &Game, p: PlayerId) -> Option<PermId> {
    let xs: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live(g, m) && is_artifact(g, m)).collect();
    min_by(&xs, |m| g.perm(m).cd.map_or(0.0, |c| art_value(g, c)) - if g.perm(m).tapped { 1.0 } else { 0.0 })
}

/// the best artifact card in p's graveyard to bring back (mana value at most `max`; not a legend p already has)
fn gy_pick(g: &Game, p: PlayerId, max: Option<u32>) -> Option<CardId> {
    let cs: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            is_artifact_card(g, c)
                && max.is_none_or(|k| d.cmc <= k)
                && !(d.tag(Tag::Leg) && on_bf_named(g, p, &d.name))
        })
        .collect();
    max_by(&cs, |c| art_value(g, c))
}

/// the swap worth making: (artifact to give up, artifact card to bring back), when the card is worth at least 3
/// more than the artifact
fn swap_plan(g: &Game, p: PlayerId, max: Option<u32>) -> Option<(PermId, CardId)> {
    let (s, c) = (sac_candidate(g, p)?, gy_pick(g, p, max)?);
    (art_value(g, c) - g.perm(s).cd.map_or(0.0, |x| art_value(g, x)) >= 3.0).then_some((s, c))
}

fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg: 0 }) }
}

/// sacrifice artifact s and put artifact card c from the graveyard onto the battlefield, if both are still there
fn swap(g: &mut Game, p: PlayerId, s: PermId, c: CardId) -> Res {
    if !live(g, s) || !g.player(p).gy.contains(&c) {
        return Ok(());
    }
    crate::glog!(g, "    {} sacrifices {} and returns {}", g.player(p).name, g.perm(s).name, g.db.get(c).name);
    die(g, s, "sac")?;
    let gy = &mut g.player_mut(p).gy;
    if let Some(i) = gy.iter().position(|&x| x == c) {
        gy.remove(i);
        enter(g, p, c, Enter::default())?;
    }
    Ok(())
}

/// **Goblin Welder** ({R}, 1/1): {T}: choose target artifact a player controls and target artifact card in that
/// player's graveyard; if both are still legal, that player sacrifices the artifact and returns the card to the
/// battlefield. The AI uses it only on its own artifacts, when the card is worth three or more than the artifact
/// (a spent rock for Helm of the Host or Hammer of Nazahn); opponents' artifacts and graveyards are not targeted.
fn welder_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !live(g, src) || owner(g, src) != p || g.perm(src).tapped || g.perm(src).sick {
        return Ok(vec![]);
    }
    let Some((s, c)) = swap_plan(g, p, None) else { return Ok(vec![]) };
    let _ = (s, c);
    Ok(vec![ability(6.0, "Goblin Welder swap", src, welder_go)])
}

fn welder_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped || g.perm(src).sick {
        return Ok(false);
    }
    let Some((s, c)) = swap_plan(g, p, None) else { return Ok(false) };
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "swap an artifact with an artifact card in the graveyard", Some(6.0), Some(s))? {
        swap(g, p, s, c)?;
    }
    Ok(true)
}

/// which artifact the Engineer puts into the graveyard: with a Welder out, Helm (then Hammer) when it is not already
/// on the battlefield or in the graveyard; else Cursed Mirror (the primer's usual target, mana value 3, which the
/// Engineer's own ability can return); else None (the AI's pick among artifacts of mana value 3 or less)
fn engineer_wants(g: &Game, p: PlayerId) -> Option<&'static str> {
    let lib = |n: &str| g.player(p).library.iter().any(|&c| &*g.db.get(c).name == n);
    let out = |n: &str| on_bf_named(g, p, n) || g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == n);
    if on_bf_named(g, p, "Goblin Welder") {
        for n in ["Helm of the Host", "Hammer of Nazahn"] {
            if !out(n) && lib(n) {
                return Some(n);
            }
        }
    }
    (!out("Cursed Mirror") && lib("Cursed Mirror")).then_some("Cursed Mirror")
}

/// **Goblin Engineer** ({1}{R}, 1/2): when it enters, search the library for an artifact card, put it into the
/// graveyard, shuffle (see `engineer_wants`); {R}, {T}, sacrifice an artifact: return target artifact card with mana
/// value 3 or less from your graveyard to the battlefield (the AI when the card is worth three or more than the
/// artifact, `swap_plan`).
fn engineer_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "search for an artifact, put it into the graveyard", Some(4.0))? {
        return Ok(());
    }
    let want = engineer_wants(g, o);
    let pred = move |g: &Game, c: CardId| match want {
        Some(n) => &*g.db.get(c).name == n,
        None => is_artifact_card(g, c) && g.db.get(c).cmc <= 3,
    };
    let got = super::t1::tutor_named(g, o, &pred, 1, "graveyard")?;
    for c in got {
        let lib = &mut g.player_mut(o).library;
        if let Some(i) = lib.iter().position(|&x| x == c) {
            lib.remove(i);
        }
        g.player_mut(o).gy.push(c);
        crate::glog!(g, "    Goblin Engineer puts {} into the graveyard", g.db.get(c).name);
    }
    shuffle_library(g, o);
    Ok(())
}

fn engineer_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none()
        || !live(g, src)
        || owner(g, src) != p
        || g.perm(src).tapped
        || g.perm(src).sick
        || !can_pay(g, p, 0, "R", false)
        || swap_plan(g, p, Some(3)).is_none()
    {
        return Ok(vec![]);
    }
    Ok(vec![ability(5.0, "Goblin Engineer return", src, engineer_go)])
}

fn engineer_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped || g.perm(src).sick {
        return Ok(false);
    }
    let Some((mut s, c)) = swap_plan(g, p, Some(3)) else { return Ok(false) };
    if !can_pay(g, p, 0, "R", false) {
        return Ok(false);
    }
    pay(g, p, 0, "R", false)?;
    g.perm_mut(src).tapped = true;
    if !live(g, s) {
        // paying used the artifact (a Lotus Petal): sacrifice the next one, if there is another
        match sac_candidate(g, p) {
            Some(x) => s = x,
            None => return Ok(true),
        }
    }
    // the sacrifice is a cost: the artifact is gone before the ability goes on the stack
    crate::glog!(g, "    {} sacrifices {} to Goblin Engineer", g.player(p).name, g.perm(s).name);
    die(g, s, "sac")?;
    if ability_window(g, p, Some(src), "return an artifact card with mana value 3 or less", Some(5.0), None)?
        && let Some(i) = g.player(p).gy.iter().position(|&x| x == c)
    {
        g.player_mut(p).gy.remove(i);
        enter(g, p, c, Enter::default())?;
        crate::glog!(g, "    {} returns to the battlefield", g.db.get(c).name);
    }
    Ok(true)
}

// ================================================================== Tezzeret, Cruel Captain
/// how much mana tapping artifact m makes (its `rock` tag or the card code's own count), 0 if none
fn rock_amt(g: &Game, p: PlayerId, m: PermId) -> u32 {
    let Some(c) = g.perm(m).cd else { return 0 };
    let Some(rock) = g.db.get(c).tags.str(Tag::Rock) else { return 0 };
    let a = rock.split_once(':').map_or(rock, |x| x.0);
    crate::cardcode::dyn_mana_perm(g, p, m).unwrap_or_else(|| a.parse().unwrap_or(1))
}

/// the tapped artifact (a spent mana rock) or creature worth untapping, with its worth: a rock that makes two or
/// more mana, or one, or a tapped creature that is not Godo (the engine untaps him between combats)
fn tez_untap_target(g: &Game, p: PlayerId, src: PermId) -> Option<(PermId, f64)> {
    let xs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != src && live(g, m) && g.perm(m).tapped && (is_artifact(g, m) || g.is_creature(m)))
        .collect();
    let worth = |m: PermId| {
        let r = if is_artifact(g, m) { rock_amt(g, p, m) } else { 0 };
        if r >= 2 { 5.0 } else if r == 1 { 3.0 } else { 0.0 }
    };
    let b = max_by(&xs, worth)?;
    (worth(b) > 0.0).then(|| (b, worth(b)))
}

fn tez_zero_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    tez_untap_target(g, p, src).map(|x| x.1)
}

fn tez_zero(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let Some((t, _)) = tez_untap_target(g, p, src) else { return Ok(()) };
    g.perm_mut(t).tapped = false;
    if is_artifact(g, t) && g.is_creature(t) {
        crate::cardcode::add_counters(g, t, 1);
    }
    crate::glog!(g, "    Tezzeret untaps {}", g.perm(t).name);
    Ok(())
}

/// the cheap artifacts (mana value 1 or less) in p's library that p does not already have out or in hand
fn tez_finds(g: &Game, p: PlayerId) -> bool {
    let have = |c: CardId| {
        g.player(p).hand.contains(&c) || g.player(p).perms.iter().any(|&m| live(g, m) && g.perm(m).cd == Some(c))
    };
    g.player(p).library.iter().any(|&c| is_artifact_card(g, c) && g.db.get(c).cmc <= 1 && !have(c))
}

fn tez_three_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    (g.perm(src).loyalty.unwrap_or(0) >= 3 && tez_finds(g, p)).then_some(4.0)
}

fn tez_three(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    super::t1::tutor_named(g, p, &|g, c| is_artifact_card(g, c) && g.db.get(c).cmc <= 1, 1, "hand")?;
    Ok(())
}

static TEZZERET: [super::common::WalkerAb; 2] = [
    super::common::WalkerAb { delta: 0, label: "untap an artifact or creature", val: tez_zero_val, eff: tez_zero },
    super::common::WalkerAb { delta: -3, label: "search for an artifact with mana value 1 or less", val: tez_three_val, eff: tez_three },
];

/// whenever an artifact you control enters, put a loyalty counter on Tezzeret
fn tez_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if !live(g, src) || owner(g, m) != o || !is_artifact(g, m) || g.perm(src).loyalty.is_none() {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "put a loyalty counter on Tezzeret", Some(2.0))? && live(g, src) {
        let x = g.perm_mut(src);
        x.loyalty = Some(x.loyalty.unwrap_or(0) + 1);
    }
    Ok(())
}

// ================================================================== Party Thrasher
/// is there a card in p's hand the AI is glad to lose: a second land, or a spell too expensive to ever cast
fn thrasher_spare(g: &Game, p: PlayerId) -> bool {
    let hand = &g.player(p).hand;
    hand.iter().filter(|&&c| g.db.get(c).land).count() >= 2 || hand.iter().any(|&c| !g.db.get(c).land && g.db.get(c).cmc >= 7)
}

/// **Party Thrasher** ({1}{R}, 1/4): at the beginning of your first main phase, you may discard a card; if you do,
/// exile the top two cards of your library, choose one of them, and you may play it this turn (the card goes to the
/// hand and into the impulse list, which the end of turn empties; the other card stays exiled). The AI discards only
/// a spare card (`thrasher_spare`) and keeps the card it wants most (`topdeck::desire`). Missing: "noncreature spells
/// you cast from exile have convoke" (rust/agents/engine-needs-godo.md).
fn thrasher_main1(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if owner(g, src) != p || !live(g, src) || g.player(p).library.len() < 2 || !thrasher_spare(g, p) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "discard a card, exile the top two, play one", Some(3.0))? {
        return Ok(());
    }
    discard_worst(g, p, 1)?;
    let top: Vec<CardId> = (0..2).filter_map(|_| g.player_mut(p).library.pop()).collect();
    let best = max_by(&top, |c| crate::ai::topdeck::desire(g, p, c));
    for &c in &top {
        if Some(c) == best {
            let pl = g.player_mut(p);
            pl.hand.push(c);
            pl.impulse.push(c);
            pl.seen.insert(c);
            crate::glog!(g, "    Party Thrasher: {} may be played this turn", g.db.get(c).name);
        } else {
            g.player_mut(p).exile.push(c);
        }
    }
    Ok(())
}

// ================================================================== Vexing Bauble
/// **Vexing Bauble** ({1} artifact): whenever a player casts a spell, if no mana was spent to cast it, counter that
/// spell; {1}, {T}, sacrifice: draw a card (the compiled ability). Modeled as a lock (`can_cast`): while it is out,
/// nobody casts a spell that costs no mana (the Moxen, Lotus Petal, Sol Ring-style zero-drops of either player).
/// The real card lets the cast happen and counters it (the card is lost to the graveyard, and a free alternative
/// cost is caught too): that needs a counter-on-cast hook, see rust/agents/engine-needs-godo.md.
fn bauble_can_cast(g: &Game, src: Src, p: PlayerId, c: CardId, _zone: Sym) -> bool {
    if !live(g, src) || g.db.get(c).land {
        return true;
    }
    let (gn, pips) = crate::engine::mana::cost_of(g, p, c);
    gn > 0 || !pips.is_empty()
}

// ================================================================== Hexing Squelcher
/// **Hexing Squelcher** ({1}{R}, 2/2): this spell can't be countered (the `unc` tag); spells you control can't be
/// countered (the `uncounterable` slot, read for the caster); ward--pay 2 life, for itself and for the other
/// creatures you control: missing, the engine's ward is mana only (rust/agents/engine-needs-godo.md).
fn squelcher_unc(g: &Game, src: Src, p: PlayerId, _c: CardId) -> bool {
    live(g, src) && owner(g, src) == p
}

// ================================================================== Blood Moon
fn is_basic(g: &Game, cd: CardId) -> bool {
    crate::engine::mana::BASIC_NAMES.contains(&&*g.db.get(cd).name)
}

/// **Blood Moon** ({2}{R} enchantment): nonbasic lands are Mountains. The engine's `blood_moon` slot makes nonbasic
/// lands tap for {R}; a land that made two mana (Ancient Tomb, City of Traitors) makes one (`moon_amt`, through the
/// `land_mana` slot). The AI casts it when the opponents' nonbasic lands outnumber its own by three or more.
/// Missing, as engine needs: nonbasic lands lose their other abilities (Ancient Tomb's damage, Urza's Saga, Command
/// Beacon, manlands) and the Mountain land type.
fn moon_on(g: &Game, src: Src) -> bool {
    live(g, src)
}

fn moon_amt(g: &Game, src: Src, p: PlayerId, l: crate::ids::LandId) -> i32 {
    let cd = g.land(l).cd;
    if !live(g, src) || is_basic(g, cd) || g.land(l).owner != p {
        return 0;
    }
    1 - g.db.get(cd).tags.int(Tag::Amt).unwrap_or(1).max(1)
}

fn nonbasic_lands(g: &Game, p: PlayerId) -> i32 {
    g.player(p).lands.iter().filter(|&&l| !is_basic(g, g.land(l).cd)).count() as i32
}

fn moon_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let theirs: i32 = g.opps(p).map(|q| nonbasic_lands(g, q)).sum();
    if theirs - nonbasic_lands(g, p) >= 3 { 55 } else { 0 }
}

// ================================================================== Defense Grid
/// **Defense Grid** ({2} artifact): each spell costs {3} more to cast except during its controller's turn (the `cost`
/// slot, for every player, its own controller included: the Godo deck's instant-speed protection costs three more on
/// an opponent's turn). Free alternative costs are not taxed (the engine adds cost increases to the mana cost only).
fn grid_tax(g: &Game, src: Src, caster: PlayerId, _c: CardId) -> i32 {
    if live(g, src) && g.active != Some(caster) { 3 } else { 0 }
}

// ================================================================== Final Fortune / Last Chance
/// **Final Fortune** ({R}{R} instant) and **Last Chance** ({R}{R} sorcery): take an extra turn after this one; at the
/// beginning of that turn's end step, you lose the game. The engine has no extra turns (`turn.rs` runs the seats in
/// order, `skip_turns` only skips), so the spell does nothing when it resolves and the AI never casts it
/// (priority 0, no deck plan names it): casting it would only throw the card away. Both are Partial, with the engine
/// needs in rust/agents/engine-needs-godo.md.
fn extra_turn_spell(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    crate::glog!(g, "  {}: {} resolves (no extra turn: the engine has none)", g.player(p).name, g.db.get(c).name);
    Ok("gy")
}

fn never_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Godo, Bandit Warlord")?;
    c.etb = Some(godo_etb);
    c.attack = Some(godo_attack);
    r.card(db, "Hammer of Nazahn")?.etb = Some(hammer_etb);
    let c = r.card(db, "Magnetic Theft")?;
    c.resolve = Some(theft_resolve);
    c.prio = Some(theft_prio);
    r.card(db, "Sting, the Glinting Dagger")?.combat_start = Some(sting_combat);
    r.card(db, "Goblin Welder")?.options = Some(welder_options);
    super::common::walker(r, db, "Tezzeret, Cruel Captain", &TEZZERET)?;
    r.card(db, "Tezzeret, Cruel Captain")?.etb = Some(tez_etb);
    r.card(db, "Party Thrasher")?.main1 = Some(thrasher_main1);
    r.card(db, "Vexing Bauble")?.can_cast = Some(bauble_can_cast);
    r.card(db, "Hexing Squelcher")?.uncounterable = Some(squelcher_unc);
    let c = r.card(db, "Blood Moon")?;
    c.blood_moon = Some(moon_on);
    c.land_mana = Some(moon_amt);
    c.prio = Some(moon_prio);
    r.card(db, "Defense Grid")?.cost = Some(grid_tax);
    let c = r.card(db, "Final Fortune")?;
    c.resolve = Some(extra_turn_spell);
    c.prio = Some(never_prio);
    let c = r.card(db, "Goblin Engineer")?;
    c.etb = Some(engineer_etb);
    c.options = Some(engineer_options);
    Ok(())
}
