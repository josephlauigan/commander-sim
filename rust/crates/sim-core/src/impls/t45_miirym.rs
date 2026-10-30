//! Tier 5 pool deck `miirym-temur-dragons` (EDHREC's bracket-4 average list): the cards the ability language doesn't
//! cover (rust/agents/t45-miirym.md). Miirym herself and the Dragon cards the deck shares with The Ur-Dragon's old
//! list are in `impls/t45_urdragon.rs`. Savage Ventmaw's "you don't lose this mana" is compiled (dsl_parse.rs,
//! `// --- Tiers 4/5 (miirym)`): floating mana already lasts the whole turn here.
//!
//! - Astral Dragon: Project Image, when it enters: two token copies of the best noncreature, nonland, nonlegendary
//!   permanent on the battlefield (yours or a targetable one of an opponent's), each a 3/3 flying Dragon creature in
//!   addition to its other types. Left out: copying a land or a legendary permanent.
//! - Dragonborn Champion: a source you control deals 5 or more damage to a player: draw a card (combat and noncombat,
//!   read where life is lost: `life::lose_life`; runs at once).
//! - Earthquake Dragon: costs {X} less, X the total mana value of the Dragons you control (`self_cost`); {2}{G},
//!   sacrifice a land: from the graveyard to hand, at the end-of-turn window when it will cost little enough to cast
//!   next turn.
//! - Korlessa, Scale Singer: Dragon spells cast from the top of the library (as Realmwalker). Left out: looking at the
//!   top card any time gives the AI nothing it uses.
//! - Thrakkus the Butcher: attacks: each Dragon you control gets +X/+0 until end of turn, X its power.
//! - Wrathful Red Dragon: a Dragon you control is dealt damage (combat, damage removal, Blasphemous Act): that Dragon
//!   deals that much damage to a target that isn't a Dragon (`t0_giada::damaged`, `removal::apply_wipe`; at once).
//!   The AI's target: an opponent it kills, else the best non-Dragon creature it kills, else the opponent with the
//!   least life. Left out: a person controlling it gets the AI's choice.

use crate::engine::repl;
use crate::ai::brain::{Situation, card_utility, do_cast_with};
use crate::cards::{CardDb, Types};
use crate::engine::cast::castable;
use crate::engine::mana::{can_pay, cost_of};
use crate::engine::removal::apply_removal;
use crate::engine::stack::trigger_window;
use crate::engine::values::{card_name, epow, etgh, has_type, pval, untargetable};
use crate::engine::zones::{draw, enter_token_copy_with, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::combos::{Combo, Ready};
use crate::impls::lands::{can_pay_without, pay_without, sac_land};
use crate::impls::t45_urdragon::dragon_card;
use crate::state::{CopyExc, Ctx, DataKey, Game, Val};

const ASTRAL: &str = "Astral Dragon";
const CHAMPION: &str = "Dragonborn Champion";
const EARTHQUAKE: &str = "Earthquake Dragon";
const KORLESSA: &str = "Korlessa, Scale Singer";
const THRAKKUS: &str = "Thrakkus the Butcher";
pub const WRATHFUL: &str = "Wrathful Red Dragon";
const ACT: &str = "Blasphemous Act";

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, src: PermId) -> PlayerId {
    g.perm(src).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// p's live permanents named `name`
fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && card_name(g, m) == Some(name)).collect()
}

/// is permanent m a Dragon creature
fn is_dragon(g: &Game, m: PermId) -> bool {
    live(g, m) && g.is_creature(m) && has_type(g, m, "dragon")
}

fn dragons_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| is_dragon(g, m)).collect()
}

// ================================================================== Astral Dragon
/// the noncreature permanent Project Image copies: the best nonland, nonlegendary one with a card (yours, or an
/// opponent's that can be targeted)
pub fn astral_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let cands: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| {
            let x = g.perm(m);
            live(g, m)
                && !g.is_creature(m)
                && x.cd.is_some_and(|c| !g.db.get(c).land && !g.db.get(c).has_subtype("aura"))
                && !crate::cardcode::is_legendary(g, m)
                && (x.owner == p || !untargetable(g, m))
        })
        .collect();
    max_by(&cands, |m| pval(g, m))
}

/// When this creature enters, create two tokens that are copies of target noncreature permanent, except they're 3/3
/// Dragon creatures in addition to their other types, and they have flying
fn astral_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != owner(g, src) {
        return Ok(());
    }
    let Some(t) = astral_target(g, p) else { return Ok(()) };
    let what = format!("two Dragon copies of {}", g.perm(t).name);
    if !trigger_window(g, p, Some(src), &what, Some(4.0))? || !live(g, t) || g.is_creature(t) {
        return Ok(());
    }
    let Some(c) = g.perm(t).cd else { return Ok(()) };
    let exc = CopyExc {
        add_types: Types::CREATURE,
        add_subtype: Some("dragon"),
        pt: Some((3, 3)),
        fly: true,
        ..CopyExc::default()
    };
    crate::glog!(g, "    Astral Dragon: two 3/3 Dragon copies of {}", cname(g, c));
    for _ in 0..2 {
        if let Some(k) = enter_token_copy_with(g, p, c, Some(exc))? {
            g.perm_mut(k).data.set(DataKey::Anim, Val::Bool(true)); // a creature in addition to its other types
        }
    }
    Ok(())
}

// ================================================================== Dragonborn Champion
/// a source p controls dealt n damage to a player (life::lose_life): with n 5 or more, each Dragonborn Champion of p's
/// draws a card
pub fn champion_hit(g: &mut Game, p: PlayerId, n: i32) -> Res {
    if n < 5 || g.over || !g.player(p).alive {
        return Ok(());
    }
    let k = named(g, p, CHAMPION).len() as u32;
    if k > 0 {
        crate::glog!(g, "    Dragonborn Champion: {} draws {}", g.player(p).name, k);
        draw(g, p, k, false)?;
    }
    Ok(())
}

// ================================================================== Earthquake Dragon
/// the total mana value of the Dragons p controls (a token copy has its card's; other tokens 0)
fn dragons_mv(g: &Game, p: PlayerId) -> i32 {
    dragons_of(g, p).into_iter().filter_map(|m| g.perm(m).cd).map(|c| g.db.get(c).cmc as i32).sum()
}

/// This spell costs {X} less to cast, where X is the total mana value of Dragons you control
fn earthquake_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    -dragons_mv(g, p)
}

/// {2}{G}, Sacrifice a land: return this card from your graveyard to your hand. At the end-of-turn window, with five
/// or more lands and the Dragon costing no more than the lands left after the sacrifice and a land drop
fn earthquake_gy(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || !g.player(p).gy.contains(&c) || g.player(p).lands.len() < 5 {
        return Ok(vec![]);
    }
    let Some(l) = crate::impls::t3_korvold::worst_land(g, p) else { return Ok(vec![]) };
    let (gn, pips) = cost_of(g, p, c);
    if gn + pips.len() as u32 > g.player(p).lands.len() as u32 || !can_pay_without(g, p, l, 2, "G") {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.5,
        label: format!("{EARTHQUAKE}: back to hand"),
        act: Some(Action::Plan { f: earthquake_go, arg: c.0 as i64 }),
    }])
}

fn earthquake_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let Some(l) = crate::impls::t3_korvold::worst_land(g, p) else { return Ok(false) };
    if !g.player(p).gy.contains(&c) || !pay_without(g, p, l, 2, "G")? {
        return Ok(false);
    }
    sac_land(g, p, l)?;
    if crate::engine::stack::ability_window_card(g, p, c, "return it to hand", None, None)? {
        let gy = &mut g.player_mut(p).gy;
        if let Some(i) = gy.iter().position(|&x| x == c) {
            gy.remove(i);
            g.player_mut(p).hand.push(c);
            crate::glog!(g, "  {} returns {} to hand", g.player(p).name, EARTHQUAKE);
        }
    }
    Ok(true)
}

// ================================================================== Korlessa, Scale Singer
/// You may cast Dragon spells from the top of your library: the AI casts the top card when it would cast it from hand
/// (`card_utility`, a little more for the free card), as Realmwalker; one option set per player (the first Korlessa)
fn korlessa_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || !live(g, src) || named(g, p, KORLESSA).first() != Some(&src) {
        return Ok(vec![]);
    }
    let Some(&top) = g.player(p).library.last() else { return Ok(vec![]) };
    if g.db.get(top).land || !dragon_card(g, top) || (post.is_none() && !crate::engine::cast::grants_flash(g, p, top)) {
        return Ok(vec![]);
    }
    let s = Situation::new(g, p);
    let Some(u) = card_utility(g, p, &s, top) else { return Ok(vec![]) };
    let (cg, cp) = cost_of(g, p, top);
    g.pay_for = Some(top);
    let ok = can_pay(g, p, cg, &cp, false) && (!g.cast_checks() || castable(g, p, top, "lib"));
    g.pay_for = None;
    if !ok {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: u + 0.6,
        label: format!("Korlessa: cast {} from the top of the library", cname(g, top)),
        act: Some(Action::Plan { f: korlessa_cast, arg: top.0 as i64 }),
    }])
}

fn korlessa_cast(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let top = CardId(arg as u16);
    if g.player(p).library.last() != Some(&top) || named(g, p, KORLESSA).is_empty() || !dragon_card(g, top) {
        return Ok(false);
    }
    g.player_mut(p).library.pop();
    crate::glog!(g, "  {} casts {} from the top of the library (Korlessa)", g.player(p).name, cname(g, top));
    let ok = do_cast_with(g, p, top, Some("lib"), Ctx::default())?;
    if !ok {
        g.player_mut(p).library.push(top);
    }
    Ok(ok)
}

// ================================================================== Thrakkus the Butcher
/// Whenever Thrakkus attacks, double the power of each Dragon you control until end of turn
fn thrakkus_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p != owner(g, src) || !atk.contains(&src) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "double the power of each Dragon", Some(4.0))? {
        return Ok(vec![]);
    }
    for m in dragons_of(g, p) {
        let x = epow(g, m);
        if x > 0 {
            crate::impls::common::eot(g, m, x, 0);
        }
    }
    crate::glog!(g, "    Thrakkus doubles the power of each Dragon");
    Ok(vec![])
}

// ================================================================== Wrathful Red Dragon
/// the Wrathful Red Dragons that see Dragon m dealt damage (its controller's)
fn wrathfuls_for(g: &Game, m: PermId) -> usize {
    if !is_dragon(g, m) {
        return 0;
    }
    named(g, owner(g, m), WRATHFUL).len()
}

/// p's Dragon deals n damage to any target that isn't a Dragon: an opponent it kills (the one with the most life
/// that still dies), else the best non-Dragon creature it kills, else the opponent with the least life
pub fn wrathful_deal(g: &mut Game, p: PlayerId, n: i32) -> Res {
    if n <= 0 || g.over || !g.player(p).alive {
        return Ok(());
    }
    let mult = crate::engine::combat::fiery(g, p); // Twinflame Tyrant, if it is still there
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let lethal: Vec<PlayerId> = opps.iter().copied().filter(|&q| g.player(q).life <= n * mult).collect();
    if let Some(q) = max_by(&lethal, |q| g.player(q).life as f64) {
        crate::glog!(g, "    Wrathful Red Dragon: {} damage to {}", n, g.player(q).name);
        crate::engine::life::lose_life(g, q, n, Some(p), "triggers", None)?;
        return crate::engine::life::check_state(g);
    }
    let tg: Vec<PermId> = opps
        .iter()
        .flat_map(|&q| g.player(q).perms.iter().copied())
        .filter(|&m| {
            live(g, m)
                && g.is_creature(m)
                && !has_type(g, m, "dragon")
                && !untargetable(g, m)
                && !repl::no_damage(g, m)
                && etgh(g, m) <= n * mult
        })
        .collect();
    if let Some(b) = max_by(&tg, |m| pval(g, m)).filter(|&b| pval(g, b) >= 3.0) {
        crate::glog!(g, "    Wrathful Red Dragon: {} damage to {}", n, g.perm(b).name);
        return apply_removal(g, Some(p), b, &format!("dmg{}", n * mult), None);
    }
    if let Some(q) = crate::engine::zones::min_by(&opps, |q| g.player(q).life as f64) {
        crate::glog!(g, "    Wrathful Red Dragon: {} damage to {}", n, g.player(q).name);
        crate::engine::life::lose_life(g, q, n, Some(p), "triggers", None)?;
        crate::engine::life::check_state(g)?;
    }
    Ok(())
}

/// creature m was dealt n damage (t0_giada::damaged: combat and damage removal): a Dragon of a player with Wrathful
/// Red Dragon deals that much to a target that isn't a Dragon, once per Wrathful
pub fn wrathful_damaged(g: &mut Game, m: PermId, n: i32) -> Res {
    let k = wrathfuls_for(g, m);
    if n <= 0 || k == 0 {
        return Ok(());
    }
    let p = owner(g, m);
    for _ in 0..k {
        wrathful_deal(g, p, n)?;
    }
    Ok(())
}

/// before a damage wipe (Blasphemous Act: 13 to each creature) resolves: the Wrathful triggers it will set off, each
/// (controller, damage). The Dragons are dealt damage together, so a Wrathful dying in the wipe still sees them all.
pub fn wrathful_wipe(g: &Game, n: i32) -> Vec<(PlayerId, i32)> {
    let mut out = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        let k = named(g, q.id, WRATHFUL).len();
        if k == 0 {
            continue;
        }
        let hit = dragons_of(g, q.id).into_iter().filter(|&m| !repl::no_damage(g, m)).count();
        out.extend(std::iter::repeat_n((q.id, n), k * hit));
    }
    out
}

/// after the wipe: the Wrathful triggers resolve (`wrathful_wipe`)
pub fn wrathful_after_wipe(g: &mut Game, pending: Vec<(PlayerId, i32)>) -> Res {
    for (p, n) in pending {
        wrathful_deal(g, p, n)?;
    }
    Ok(())
}

// ================================================================== combo: Wrathful Red Dragon + Blasphemous Act
/// how many opponents die to `chunks` hits of `n` damage, each at one player (the least life first)
pub fn chunks_kill(lives: &mut [i32], chunks: usize, n: i32) -> usize {
    lives.sort();
    let mut left = chunks;
    let mut dead = 0;
    for &l in lives.iter() {
        let need = ((l.max(1) + n - 1) / n) as usize;
        if need > left {
            break;
        }
        left -= need;
        dead += 1;
    }
    dead
}

/// Wrathful Red Dragon on the battlefield and Blasphemous Act castable from hand: 13 damage per Dragon of yours (per
/// Wrathful) kills every opponent (the Dragons die in the wipe, so Twinflame Tyrant doubles nothing)
fn act_ready(g: &Game, p: PlayerId) -> Ready {
    let ws = named(g, p, WRATHFUL);
    let Some(act) = g.db.id(ACT).filter(|c| g.player(p).hand.contains(c)) else { return (false, vec![], vec![]) };
    if ws.is_empty() {
        return (false, vec![], vec![]);
    }
    let chunks = wrathful_wipe(g, 13).into_iter().filter(|x| x.0 == p).count();
    let mut lives: Vec<i32> = g.opps(p).map(|q| g.player(q).life).collect();
    let n = lives.len();
    if n == 0 || chunks_kill(&mut lives, chunks, 13) < n {
        return (false, vec![], vec![]);
    }
    (true, ws, vec![act])
}

/// the real Blasphemous Act and Wrathful triggers did the work (cast through the combo's counter window)
fn act_finish(g: &mut Game, _p: PlayerId) -> Res {
    crate::engine::life::check_state(g)
}

pub static COMBOS: [Combo; 1] = [Combo {
    name: "Wrathful Red Dragon + Blasphemous Act",
    groups: &[&[WRATHFUL], &[ACT]],
    ready: act_ready,
    mana: (0, ""),
    finish: Some(act_finish),
    xcost: &[],
    steps: &[],
}];

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let has = |n: &str| db.id(n).is_some();
    if has(ASTRAL) {
        r.card(db, ASTRAL)?.etb = Some(astral_etb);
    }
    if has(EARTHQUAKE) {
        let c = r.card(db, EARTHQUAKE)?;
        c.self_cost = Some(earthquake_cost);
        c.gy_options = Some(earthquake_gy);
    }
    if has(KORLESSA) {
        r.card(db, KORLESSA)?.options = Some(korlessa_options);
    }
    if has(THRAKKUS) {
        r.card(db, THRAKKUS)?.attack = Some(thrakkus_attack);
    }
    Ok(())
}
