//! Tier 5 pool deck `ur-dragon-five-color-dragons` (EDHREC's bracket-4 average list): the cards the ability language
//! doesn't cover (rust/agents/t45-urdragon.md). "Dragon spells you cast cost {N} less" (Dragonlord's Servant,
//! Dragonspeaker Shaman, Sarkhan) and "Dragons you control have indestructible" (Call the Spirit Dragons) are compiled
//! (dsl_parse.rs, `// --- Tiers 4/5 (urdragon)`); the rest is here, with small hooks in the engine marked the same way.
//!
//! - The Ur-Dragon: eminence, in the command zone or on the battlefield: your other Dragon spells cost {1} less
//!   (mana::cost_of); one or more Dragons you control attack: draw that many, then put the best nonland permanent card
//!   from your hand onto the battlefield. Left out: a land card is never the one put onto the battlefield.
//! - Ancient Brass Dragon: combat damage to a player: a d20; creature cards from all graveyards with total mana value
//!   up to the roll come back under your control (the most mana value that fits).
//! - Ancient Copper Dragon: combat damage to a player: a d20, that many Treasures.
//! - Ancient Gold Dragon: combat damage to a player: a d20, that many 1/1 blue Faerie Dragon fliers.
//! - Ancient Silver Dragon: combat damage to a player: a d20, draw that many; no maximum hand size for the game.
//! - Call the Spirit Dragons: your upkeep: for each colour a +1/+1 counter on a Dragon of yours of that colour (as many
//!   different Dragons as can be: a matching); five different Dragons: you win.
//! - Dracogenesis: your Dragon spells cost no mana (mana::cost_of; commander tax still paid).
//! - Dragon's Hoard: a Dragon of yours enters: a gold counter; {T}, remove one: draw (at the end-of-turn window).
//! - Haven of the Spirit Dragon: {C}, or any colour for a Dragon creature spell; {2},{T}, sacrifice: the best Dragon
//!   creature card from your graveyard to hand (at the end-of-turn window). Left out: Ugin planeswalker cards.
//! - Hellkite Courser: enters: a commander of yours in the command zone onto the battlefield with haste; it goes back
//!   to the command zone at the next end step (turn::end_step, `courser_return`).
//! - Klauth, Unrivaled Ancient: attacks: X mana of any colour (X: the attackers' total power), kept all turn. Left
//!   out: it can pay for abilities too.
//! - Lathliss, Dragon Queen: another nontoken Dragon of yours enters: a 5/5 red Dragon flier; {1}{R}: Dragons +1/+0,
//!   used before an open attack with the spare mana when nothing in hand can be cast.
//! - Miirym, Sentinel Wyrm: another nontoken Dragon of yours enters: a token copy of it that isn't legendary.
//! - Morophon, the Boundless: names a creature type as it enters (the deck's most common); spells of that type cost
//!   {W}{U}{B}{R}{G} less (only coloured mana, mana::cost_of); your other creatures of that type get +1/+1.
//! - Rith, Liberated Primeval: your other Dragons have ward {2} (removal::aura_ward). Its end-step Dragon comes when an
//!   opponent's creature died during your turn. Left out: "excess damage" (the engine does not track it).
//! - Rivaz of the Claw: {T}: two mana of any colour, only for Dragon creature spells (mana.rs); once during each of
//!   your turns, cast a Dragon creature card from your graveyard; that creature is exiled when it dies (while Rivaz is
//!   on the battlefield to see it).
//! - Roaming Throne: names a creature type as it enters and is that type too; another creature of yours of that type:
//!   its triggered abilities that go on the stack trigger an additional time (stack::queue_triggers). Left out: card
//!   code that runs at once rather than as a trigger.
//! - Scion of Draco: costs {2} less per basic land type among your lands; your creatures have vigilance (white),
//!   hexproof (blue), lifelink (black), first strike (red) and trample (green).
//! - Scion of the Ur-Dragon: {2} at an opponent's end of turn with Rivaz out and no Dragon in the graveyard: the best
//!   Dragon creature card from the library into the graveyard (Rivaz casts it next turn). Left out: Scion becoming a
//!   copy.
//! - Twinflame Tyrant: damage from your sources to opponents and their permanents is doubled (combat::fiery).
//! - Ureni of the Unwritten: enters or attacks: the top eight; the best Dragon creature card among them onto the
//!   battlefield, the rest to the bottom in a random order.
//! - Urza's Incubator: names a creature type as it enters; creature spells of that type cost {2} less.

use crate::cards::{CardDb, Colors};
use crate::engine::cast::cast_card;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{card_name, colors_of, epow, has_type};
use crate::engine::zones::{Enter, Tokens, Zone, add_treasure, draw, enter, enter_token_copy_with, leave, make_tokens};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::impls::lands::{can_pay_without, pay_without, sac_land};
use crate::impls::partials::eot_kw;
use crate::state::{CopyExc, Ctx, DataKey, Game, Val};
use crate::sym::Sym;

pub const UR_DRAGON: &str = "The Ur-Dragon";
const BRASS: &str = "Ancient Brass Dragon";
const COPPER: &str = "Ancient Copper Dragon";
const GOLD: &str = "Ancient Gold Dragon";
const SILVER: &str = "Ancient Silver Dragon";
const CALL: &str = "Call the Spirit Dragons";
const DRACOGENESIS: &str = "Dracogenesis";
const HOARD: &str = "Dragon's Hoard";
const HAVEN: &str = "Haven of the Spirit Dragon";
const COURSER: &str = "Hellkite Courser";
const KLAUTH: &str = "Klauth, Unrivaled Ancient";
const LATHLISS: &str = "Lathliss, Dragon Queen";
const MIIRYM: &str = "Miirym, Sentinel Wyrm";
const MOROPHON: &str = "Morophon, the Boundless";
const RITH: &str = "Rith, Liberated Primeval";
const RIVAZ: &str = "Rivaz of the Claw";
const THRONE: &str = "Roaming Throne";
const DRACO: &str = "Scion of Draco";
const SCION: &str = "Scion of the Ur-Dragon";
const TIAMAT: &str = "Tiamat";
pub const TYRANT: &str = "Twinflame Tyrant";
const URENI: &str = "Ureni of the Unwritten";
const INCUBATOR: &str = "Urza's Incubator";

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

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// p's live permanents named `name`
fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && card_name(g, m) == Some(name)).collect()
}

/// is card c a Dragon (a changeling is every creature type)
pub fn dragon_card(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.has_subtype("dragon") || d.has_kw("changeling")
}

fn dragon_creature_card(g: &Game, c: CardId) -> bool {
    g.db.get(c).creature && dragon_card(g, c)
}

/// is permanent m a Dragon creature
fn is_dragon(g: &Game, m: PermId) -> bool {
    live(g, m) && g.is_creature(m) && has_type(g, m, "dragon")
}

fn dragons_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| is_dragon(g, m)).collect()
}

/// a creature card's worth for putting onto the battlefield: its deck priority, else its size
fn body_worth(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let d = g.db.get(c);
    card_worth(g, p, c, false).max(0.0) / 10.0 + d.cmc as f64 + d.bomb as f64 + d.pow as f64 / 2.0
}

fn d20(g: &mut Game) -> u32 {
    g.rng.below(20) as u32 + 1
}

fn mark(g: &Game, m: PermId, what: &str) -> bool {
    matches!(g.perm(m).data.get(DataKey::T45Dragon), Some(Val::Str(s)) if *s == what)
}

/// the creature type a permanent named as it entered
fn chosen(g: &Game, m: PermId) -> Option<Sym> {
    crate::impls::galadriel::ctype(g, m)
}

/// as it enters, name a creature type (the AI's: the type most of its creature cards share)
fn name_type(g: &mut Game, p: PlayerId, m: PermId) -> Res<Sym> {
    let what = g.perm(m).name;
    let t = crate::impls::galadriel::choose_type(g, p, &what)?;
    g.perm_mut(m).data.set(DataKey::Ctype, Val::Str(t));
    crate::glog!(g, "    {} names {}", what, t);
    Ok(t)
}

fn of_type(g: &Game, c: CardId, t: Sym) -> bool {
    let d = g.db.get(c);
    d.has_subtype(t) || d.has_kw("changeling")
}

// ================================================================== cost changes (mana::cost_of)
/// The Ur-Dragon's eminence: in the command zone or on the battlefield, your other Dragon spells cost {1} less (once
/// for the commander, once more for each other copy on the battlefield)
pub fn eminence(g: &Game, p: PlayerId, c: CardId) -> i32 {
    if !dragon_card(g, c) || cname(g, c) == UR_DRAGON {
        return 0;
    }
    let pl = g.player(p);
    let zone = pl.commanders().any(|x| pl.cmd_zone_of(x) && cname(g, x) == UR_DRAGON) as i32;
    -(zone + named(g, p, UR_DRAGON).len() as i32)
}

/// Dracogenesis: you may cast Dragon spells without paying their mana costs
pub fn free_dragon(g: &Game, p: PlayerId, c: CardId) -> bool {
    dragon_card(g, c) && !named(g, p, DRACOGENESIS).is_empty()
}

/// Morophon: spells of the chosen type cost {W}{U}{B}{R}{G} less (only coloured mana), once per Morophon
pub fn morophon_pips(g: &Game, p: PlayerId, c: CardId, pips: &mut String) {
    for m in named(g, p, MOROPHON) {
        if chosen(g, m).is_some_and(|t| of_type(g, c, t)) {
            for col in ['W', 'U', 'B', 'R', 'G'] {
                if let Some(i) = pips.find(col) {
                    pips.remove(i);
                }
            }
        }
    }
}

/// Urza's Incubator: creature spells of the chosen type cost {2} less
fn incubator_cost(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    let Some(t) = chosen(g, src) else { return 0 };
    if owner(g, src) == caster && live(g, src) && g.db.get(c).creature && of_type(g, c, t) { -2 } else { 0 }
}

fn incubator_as(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    name_type(g, p, m).map(|_| ())
}

// ================================================================== Morophon, the Boundless
fn morophon_as(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    name_type(g, p, m)?;
    g.selfpt = true; // the +1/+1 is read through `creature_pt`
    Ok(())
}

/// Other creatures you control of the chosen type get +1/+1 (once per Morophon)
fn morophon_pt(g: &Game, m: PermId) -> (i32, i32) {
    let p = g.perm(m).owner;
    let n = named(g, p, MOROPHON)
        .into_iter()
        .filter(|&x| x != m && chosen(g, x).is_some_and(|t| has_type(g, m, t)))
        .count() as i32;
    (n, n)
}

// ================================================================== Roaming Throne
fn throne_as(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let t = name_type(g, p, m)?;
    let x = g.perm_mut(m);
    if !x.ttypes.contains(&t) {
        x.ttypes.push(t); // it is the chosen type in addition to its other types
    }
    Ok(())
}

/// a triggered ability of another creature you control of the chosen type triggers an additional time, once per
/// Roaming Throne (stack::queue_triggers)
pub fn throne_extra(g: &Game, src: PermId) -> usize {
    let x = g.perm(src);
    if !x.on_bf || x.phased || !g.is_creature(src) {
        return 0;
    }
    named(g, x.owner, THRONE)
        .into_iter()
        .filter(|&t| t != src && chosen(g, t).is_some_and(|k| has_type(g, src, k)))
        .count()
}

// ================================================================== The Ur-Dragon
/// Whenever one or more Dragons you control attack, draw that many cards, then you may put a permanent card from your
/// hand onto the battlefield (the best nonland one)
fn ur_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let n = atk.iter().filter(|&&m| is_dragon(g, m) && g.perm(m).owner == p).count() as u32;
    if p != owner(g, src) || !live(g, src) || n == 0 {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), &format!("draw {n}, then put a permanent onto the battlefield"), Some(6.0))? {
        return Ok(vec![]);
    }
    draw(g, p, n, false)?;
    let cs: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).perm && !g.db.get(c).land).collect();
    let Some(c) = crate::engine::zones::max_by(&cs, |c| body_worth(g, p, c)) else { return Ok(vec![]) };
    let pl = g.player_mut(p);
    if let Some(i) = pl.hand.iter().position(|&x| x == c) {
        pl.hand.remove(i);
        crate::glog!(g, "    {} puts {} onto the battlefield (The Ur-Dragon)", pname(g, p), cname(g, c));
        enter(g, p, c, Enter::default())?;
    }
    Ok(vec![])
}

// ================================================================== the Ancient Dragons
fn connects(g: &Game, src: Src, p: PlayerId, a: PermId) -> bool {
    a == src && p == owner(g, src)
}

/// Ancient Copper Dragon: roll a d20, that many Treasures
fn copper_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if !connects(g, src, p, a) || !trigger_window(g, p, Some(src), "roll a d20: Treasures", Some(6.0))? {
        return Ok(());
    }
    let n = d20(g);
    crate::glog!(g, "    {} rolls {}: {} Treasures", cname(g, g.perm(src).cd.unwrap()), n, n);
    add_treasure(g, p, n as i32)
}

/// Ancient Gold Dragon: roll a d20, that many 1/1 blue Faerie Dragon creature tokens with flying
fn gold_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if !connects(g, src, p, a) || !trigger_window(g, p, Some(src), "roll a d20: Faerie Dragons", Some(6.0))? {
        return Ok(());
    }
    let n = d20(g);
    crate::glog!(g, "    Ancient Gold Dragon rolls {n}: {n} Faerie Dragons");
    let spec = Tokens {
        fly: true,
        color: Some(Colors::from_letters("U")),
        types: vec!["faerie", "dragon"],
        ..Tokens::new(n, 1)
    };
    make_tokens(g, p, spec).map(|_| ())
}

/// Ancient Silver Dragon: roll a d20, draw that many; no maximum hand size for the rest of the game
fn silver_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if !connects(g, src, p, a) || !trigger_window(g, p, Some(src), "roll a d20: draw", Some(6.0))? {
        return Ok(());
    }
    let n = d20(g);
    crate::glog!(g, "    Ancient Silver Dragon rolls {n}: draw {n}");
    g.player_mut(p).t0s.nomax = true;
    draw(g, p, n, false)
}

/// creature cards in the graveyards (card, its owner) that may enter the battlefield
fn gy_creatures(g: &Game) -> Vec<(CardId, PlayerId)> {
    if crate::impls::t45_winota::jailed(g) {
        return vec![]; // Soulless Jailer
    }
    g.players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.gy.iter().filter(|&&c| g.db.get(c).creature).map(move |&c| (c, q.id)))
        .collect()
}

/// the creature cards with the most worth whose total mana value is at most x (a knapsack over mana value)
pub fn brass_pick(g: &Game, p: PlayerId, x: u32) -> Vec<(CardId, PlayerId)> {
    let cands = gy_creatures(g);
    let x = x as usize;
    // best[w]: (worth, picks) with total mana value w or less
    let mut best: Vec<(f64, Vec<usize>)> = vec![(0.0, vec![]); x + 1];
    for (i, &(c, _)) in cands.iter().enumerate() {
        let mv = g.db.get(c).cmc as usize;
        let v = body_worth(g, p, c);
        for w in (mv..=x).rev() {
            let cand = best[w - mv].0 + v;
            if cand > best[w].0 {
                let mut picks = best[w - mv].1.clone();
                picks.push(i);
                best[w] = (cand, picks);
            }
        }
    }
    best[x].1.iter().map(|&i| cands[i]).collect()
}

/// Ancient Brass Dragon: roll a d20; creature cards with total mana value X or less from graveyards onto the
/// battlefield under your control
fn brass_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if !connects(g, src, p, a) || gy_creatures(g).is_empty() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "roll a d20: reanimate", Some(6.0))? {
        return Ok(());
    }
    let x = d20(g);
    crate::glog!(g, "    Ancient Brass Dragon rolls {x}");
    let mut picks = brass_pick(g, p, x);
    let worth: f64 = picks.iter().map(|&(c, _)| body_worth(g, p, c)).sum();
    let mut owners: Vec<PlayerId> = picks.iter().map(|x| x.1).collect();
    owners.dedup();
    let mut hated = false;
    for q in owners {
        hated |= crate::impls::common::gy_response(g, p, worth, q)?; // Tormod's Crypt and the like
    }
    if hated {
        picks = brass_pick(g, p, x);
    }
    for (c, q) in picks {
        let gy = &mut g.player_mut(q).gy;
        let Some(i) = gy.iter().position(|&y| y == c) else { continue };
        gy.remove(i);
        crate::glog!(g, "    {} returns {} to the battlefield", pname(g, p), cname(g, c));
        enter(g, p, c, Enter { orig: (q != p).then_some(q), ..Enter::default() })?;
    }
    Ok(())
}

// ================================================================== Call the Spirit Dragons
/// for each colour (W U B R G), the Dragon that gets its counter: as many different Dragons as can be (a bipartite
/// matching), a colour whose Dragons all went to other colours puts its counter on one of them again
pub fn call_assign(g: &Game, p: PlayerId) -> Vec<(char, PermId)> {
    let dragons = dragons_of(g, p);
    let cols = ['W', 'U', 'B', 'R', 'G'];
    let cand: Vec<Vec<usize>> =
        cols.iter().map(|&ch| (0..dragons.len()).filter(|&i| colors_of(g, dragons[i]).has(ch)).collect()).collect();
    // Kuhn's augmenting paths: owner_of[dragon] = colour
    let mut owner_of: Vec<Option<usize>> = vec![None; dragons.len()];
    fn augment(k: usize, cand: &[Vec<usize>], seen: &mut [bool], owner_of: &mut [Option<usize>]) -> bool {
        for &i in &cand[k] {
            if seen[i] {
                continue;
            }
            seen[i] = true;
            if owner_of[i].is_none_or(|o| augment(o, cand, seen, owner_of)) {
                owner_of[i] = Some(k);
                return true;
            }
        }
        false
    }
    for k in 0..cols.len() {
        let mut seen = vec![false; dragons.len()];
        augment(k, &cand, &mut seen, &mut owner_of);
    }
    let mut out = vec![];
    for (k, &ch) in cols.iter().enumerate() {
        if let Some(i) = owner_of.iter().position(|&o| o == Some(k)) {
            out.push((ch, dragons[i]));
        } else if let Some(&i) = cand[k].first() {
            out.push((ch, dragons[i]));
        }
    }
    out
}

fn call_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !live(g, src) || dragons_of(g, p).is_empty() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "a +1/+1 counter for each colour", Some(5.0))? {
        return Ok(());
    }
    let picks = call_assign(g, p);
    let mut got: Vec<PermId> = vec![];
    for &(_, m) in &picks {
        if live(g, m) {
            crate::cardcode::put_plus(g, m, 1);
            if !got.contains(&m) {
                got.push(m);
            }
        }
    }
    if got.len() >= 5 {
        crate::glog!(g, "    {} put counters on five Dragons", pname(g, p));
        return crate::ai::win(g, p, "Call the Spirit Dragons", Some(false));
    }
    Ok(())
}

// ================================================================== Dragon's Hoard
fn gold(g: &Game, m: PermId) -> i64 {
    match g.perm(m).data.get(DataKey::T45Dragon) {
        Some(Val::Int(n)) => *n,
        _ => 0,
    }
}

/// Whenever a Dragon you control enters, put a gold counter on Dragon's Hoard
fn hoard_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if p != owner(g, src) || !live(g, src) || !is_dragon(g, m) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "a gold counter", Some(1.0))? && live(g, src) {
        let n = gold(g, src) + 1;
        g.perm_mut(src).data.set(DataKey::T45Dragon, Val::Int(n));
    }
    Ok(())
}

/// {T}, remove a gold counter: draw a card (at the end-of-turn window, when the mana is no longer needed)
fn hoard_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || p != owner(g, src) || !live(g, src) || g.perm(src).tapped || gold(g, src) <= 0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.0,
        label: "Dragon's Hoard: draw".into(),
        act: Some(Action::Ability { src, f: hoard_go, arg: 0 }),
    }])
}

fn hoard_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || g.perm(src).tapped || gold(g, src) <= 0 {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    let n = gold(g, src) - 1;
    g.perm_mut(src).data.set(DataKey::T45Dragon, Val::Int(n));
    if ability_window(g, p, Some(src), "draw a card", None, None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

// ================================================================== Haven of the Spirit Dragon
/// {C}, or one mana of any colour for a Dragon creature spell
fn haven_colors(g: &Game, p: PlayerId, _l: LandId) -> Colors {
    match g.pay_for {
        Some(c) if dragon_creature_card(g, c) => g.player(p).ident,
        _ => Colors::NONE,
    }
}

fn best_gy_dragon(g: &Game, p: PlayerId) -> Option<CardId> {
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| dragon_creature_card(g, c)).collect();
    crate::engine::zones::max_by(&cs, |c| body_worth(g, p, c))
}

/// {2},{T}, sacrifice: the best Dragon creature card from your graveyard to your hand (at the end-of-turn window)
fn haven_options(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.land(l).tapped || !can_pay_without(g, p, l, 2, "") {
        return Ok(vec![]);
    }
    let Some(c) = best_gy_dragon(g, p) else { return Ok(vec![]) };
    if g.db.get(c).cmc < 5 {
        return Ok(vec![]); // worth a land only for a big Dragon
    }
    Ok(vec![Opt {
        utility: 1.4,
        label: "Haven of the Spirit Dragon".into(),
        act: Some(Action::Plan { f: haven_go, arg: ((c.0 as i64) << 32) | l.0 as i64 }),
    }])
}

fn haven_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, c) = (LandId((arg & 0xffff_ffff) as u32), CardId((arg >> 32) as u16));
    if !g.player(p).lands.contains(&l) || g.land(l).tapped || !g.player(p).gy.contains(&c) {
        return Ok(false);
    }
    if !pay_without(g, p, l, 2, "")? {
        return Ok(false);
    }
    let haven = g.land(l).cd;
    sac_land(g, p, l)?;
    let what = format!("{} to hand", cname(g, c));
    if crate::engine::stack::ability_window_card(g, p, haven, &what, None, None)? {
        let gy = &mut g.player_mut(p).gy;
        if let Some(i) = gy.iter().position(|&x| x == c) {
            gy.remove(i);
            g.player_mut(p).hand.push(c);
            crate::glog!(g, "  {} returns {} with Haven of the Spirit Dragon", pname(g, p), cname(g, c));
        }
    }
    Ok(true)
}

// ================================================================== Hellkite Courser
/// When it enters, you may put a commander you own from the command zone onto the battlefield. It gains haste. Return
/// it to the command zone at the beginning of the next end step.
fn courser_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != owner(g, src) {
        return Ok(());
    }
    let Some(c) = g.player(p).cmds_in_zone().max_by_key(|&c| g.db.get(c).cmc) else { return Ok(()) };
    if !trigger_window(g, p, Some(src), "your commander onto the battlefield", Some(6.0))?
        || !g.player(p).cmd_zone_of(c)
    {
        return Ok(());
    }
    g.player_mut(p).set_cmd_zone(c, false);
    let n = enter(g, p, c, Enter::default())?;
    g.perm_mut(n).is_cmd = true;
    eot_kw(g, n, "haste");
    g.perm_mut(n).data.set(DataKey::T45Dragon, Val::Str("courser"));
    crate::glog!(g, "    {} puts {} onto the battlefield (Hellkite Courser)", pname(g, p), cname(g, c));
    Ok(())
}

/// at the beginning of the end step: the commanders Hellkite Courser put onto the battlefield go back to the command
/// zone (turn.rs, each end step)
pub fn courser_return(g: &mut Game) -> Res {
    let back: Vec<PermId> = g
        .players
        .iter()
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| live(g, m) && mark(g, m, "courser"))
        .collect();
    for m in back {
        g.perm_mut(m).data.remove(DataKey::T45Dragon);
        crate::glog!(g, "    {} returns to the command zone (Hellkite Courser)", g.perm(m).name);
        leave(g, m)?;
        crate::engine::zones::to_zone_card(g, m, Zone::Lib); // a commander goes to the command zone instead
    }
    Ok(())
}

// ================================================================== Klauth, Unrivaled Ancient
/// Whenever Klauth attacks, add X mana in any combination of colours, where X is the total power of attacking creatures
/// (kept until end of turn)
fn klauth_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p != owner(g, src) || !atk.contains(&src) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "mana equal to the attackers' power", Some(3.0))? {
        return Ok(vec![]);
    }
    let x: i32 = atk.iter().filter(|&&m| g.perm(m).on_bf).map(|&m| epow(g, m).max(0)).sum();
    g.player_mut(p).floating.any += x as u32;
    crate::glog!(g, "    Klauth adds {x} mana");
    Ok(vec![])
}

// ================================================================== Lathliss, Miirym
/// another nontoken Dragon of src's controller entered
fn other_nontoken_dragon(g: &Game, src: Src, p: PlayerId, m: PermId) -> bool {
    m != src && p == owner(g, src) && live(g, src) && !g.perm(m).token && is_dragon(g, m)
}

/// Lathliss: whenever another nontoken Dragon you control enters, create a 5/5 red Dragon creature token with flying
fn lathliss_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if !other_nontoken_dragon(g, src, p, m) || !trigger_window(g, p, Some(src), "a 5/5 Dragon", Some(4.0))? {
        return Ok(());
    }
    let spec = Tokens { fly: true, color: Some(Colors::from_letters("R")), types: vec!["dragon"], ..Tokens::new(1, 5) };
    make_tokens(g, p, spec).map(|_| ())
}

/// Lathliss: {1}{R}: Dragons you control get +1/+0 until end of turn. Before an open attack (as Scourge of Valkas's
/// firebreathing), with nothing in hand to cast: as many times as the mana allows, keeping two
fn lathliss_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post != Some(false) || !live(g, src) || g.active != Some(p) {
        return Ok(vec![]);
    }
    let ds: Vec<PermId> = dragons_of(g, p)
        .into_iter()
        .filter(|&m| !g.perm(m).tapped && (!g.perm(m).sick || crate::engine::combat::has_haste(g, m)))
        .collect();
    let k = (total_mana(g, p, false) as i32 - 2) / 2;
    if ds.is_empty() || k < 1 {
        return Ok(vec![]);
    }
    let castable = g.player(p).hand.iter().any(|&c| {
        let (gn, pips) = cost_of(g, p, c);
        !g.db.get(c).land && can_pay(g, p, gn, &pips, false)
    });
    let power: i32 = ds.iter().map(|&m| epow(g, m)).sum();
    if castable || !crate::impls::lands::open_attack(g, p, power + k * ds.len() as i32, true) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 0.3 * (k as usize * ds.len()) as f64,
        label: format!("Lathliss: Dragons +{k}/+0"),
        act: Some(Action::Ability { src, f: lathliss_go, arg: k as i64 }),
    }])
}

fn lathliss_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let k = arg as u32;
    let pips = "R".repeat(k as usize);
    if !live(g, src) || !can_pay(g, p, k, &pips, false) {
        return Ok(false);
    }
    pay(g, p, k, &pips, false)?;
    if ability_window(g, p, Some(src), &format!("Dragons get +{k}/+0"), None, None)? {
        for m in dragons_of(g, p) {
            crate::impls::common::eot(g, m, k as i32, 0);
        }
    }
    Ok(true)
}

/// Miirym: whenever another nontoken Dragon you control enters, create a token that's a copy of it, except the token
/// isn't legendary
fn miirym_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if !other_nontoken_dragon(g, src, p, m) {
        return Ok(());
    }
    let Some(c) = g.perm(m).cd else { return Ok(()) };
    if !trigger_window(g, p, Some(src), &format!("a token copy of {}", cname(g, c)), Some(5.0))? {
        return Ok(());
    }
    enter_token_copy_with(g, p, c, Some(CopyExc { not_legendary: true, ..CopyExc::default() })).map(|_| ())
}

// ================================================================== Rith, Liberated Primeval
/// Other Dragons you control have ward {2} (removal::aura_ward)
pub fn rith_ward(g: &Game, m: PermId) -> u32 {
    if !g.is_creature(m) || !has_type(g, m, "dragon") {
        return 0;
    }
    2 * named(g, g.perm(m).owner, RITH).into_iter().filter(|&r| r != m).count() as u32
}

/// your end step, if an opponent's creature died this turn (the engine's stand-in for "was dealt excess damage"):
/// a 4/4 red Dragon with flying
fn rith_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let st = g.turn_stamp();
    if p != owner(g, src) || g.active != Some(p) || !g.opps(p).any(|q| g.player(q).creature_died == Some(st)) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "a 4/4 Dragon", Some(3.0))? {
        return Ok(());
    }
    let spec = Tokens { fly: true, color: Some(Colors::from_letters("R")), types: vec!["dragon"], ..Tokens::new(1, 4) };
    make_tokens(g, p, spec).map(|_| ())
}

// ================================================================== Rivaz of the Claw
/// mana restricted to Dragon creature spells (mana.rs, as Giada's to Angels)
pub fn dragon_only_mana(g: &Game, name: &str) -> bool {
    name == RIVAZ && !g.pay_for.is_some_and(|c| dragon_creature_card(g, c))
}

/// Once during each of your turns, cast a Dragon creature spell from your graveyard (the best one you can pay for)
fn rivaz_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let st = g.turn_stamp();
    if post.is_none() || p != owner(g, src) || !live(g, src) || g.active != Some(p) {
        return Ok(vec![]);
    }
    if g.perm(src).data.get(DataKey::T45Dragon) == Some(&Val::Stamp(st)) {
        return Ok(vec![]);
    }
    let cs: Vec<CardId> = g.player(p).gy.clone().into_iter().filter(|&c| dragon_creature_card(g, c)).collect();
    let mut ok = vec![];
    for c in cs {
        let (gn, pips) = cost_of(g, p, c);
        g.pay_for = Some(c);
        let can = can_pay(g, p, gn, &pips, false);
        g.pay_for = None;
        if can && crate::engine::hooks::allowed(g, p, c, "gy") {
            ok.push(c);
        }
    }
    let Some(c) = crate::engine::zones::max_by(&ok, |c| body_worth(g, p, c)) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 2.0 + body_worth(g, p, c) / 10.0,
        label: format!("Rivaz: cast {} from the graveyard", cname(g, c)),
        act: Some(Action::Ability { src, f: rivaz_go, arg: c.0 as i64 }),
    }])
}

fn rivaz_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let st = g.turn_stamp();
    if !live(g, src)
        || !g.player(p).gy.contains(&c)
        || g.perm(src).data.get(DataKey::T45Dragon) == Some(&Val::Stamp(st))
    {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, p, c);
    g.pay_for = Some(c);
    let paid = can_pay(g, p, gn, &pips, false) && pay(g, p, gn, &pips, false)?;
    g.pay_for = None;
    if !paid {
        return Ok(false);
    }
    g.perm_mut(src).data.set(DataKey::T45Dragon, Val::Stamp(st));
    crate::glog!(g, "  {} casts {} from the graveyard (Rivaz of the Claw)", pname(g, p), cname(g, c));
    let before: Vec<PermId> = g.player(p).perms.clone();
    if cast_card(g, p, c, "gy", Ctx::default())? {
        let new: Vec<PermId> = g.player(p).perms.iter().copied().filter(|m| !before.contains(m)).collect();
        for m in new {
            if g.perm(m).cd == Some(c) {
                g.perm_mut(m).data.set(DataKey::T45Dragon, Val::Str("rivaz")); // "when this creature dies, exile it"
            }
        }
    }
    Ok(true)
}

/// the creature cast with Rivaz dies: its card is exiled (a trigger: it resolves once the card is in the graveyard)
fn rivaz_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if !mark(g, m, "rivaz") || g.perm(m).token || owner(g, src) != g.perm(m).orig {
        return Ok(());
    }
    let (q, Some(c)) = (g.perm(m).orig, g.perm(m).cd) else { return Ok(()) };
    if !trigger_window(g, q, Some(src), "exile the Dragon cast from the graveyard", Some(1.0))? {
        return Ok(());
    }
    let gy = &mut g.player_mut(q).gy;
    if let Some(i) = gy.iter().rposition(|&x| x == c) {
        gy.remove(i);
        g.player_mut(q).exile.push(c);
        crate::glog!(g, "    {} is exiled (Rivaz of the Claw)", cname(g, c));
    }
    Ok(())
}

fn rivaz_mana(_g: &Game, _p: PlayerId, _m: PermId) -> u32 {
    2
}

// ================================================================== Scion of Draco
fn draco_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    -2 * crate::impls::partials::domain(g, p)
}

/// Each creature you control has vigilance if it's white, hexproof if blue, lifelink if black, first strike if red,
/// trample if green
fn draco_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    if !live(g, src) || g.perm(m).owner != owner(g, src) || !g.is_creature(m) {
        return false;
    }
    let col = match kw {
        "vigilance" => 'W',
        "hexproof" => 'U',
        "lifelink" => 'B',
        "first strike" => 'R',
        "trample" => 'G',
        _ => return false,
    };
    colors_of(g, m).has(col)
}

// ================================================================== Scion of the Ur-Dragon
/// {2}: the best Dragon creature card from the library into the graveyard, at the end-of-turn window, when Rivaz can
/// cast it on your next turn
fn scion_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.player(p).key == "scion" {
        return crate::impls::t6_scion::scion_options(g, src, p, post); // --- scion: JD's deck copies the Dragon
    }
    if post.is_some() || p != owner(g, src) || !live(g, src) || g.active == Some(p) || named(g, p, RIVAZ).is_empty() {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 2, "", false) || g.player(p).gy.iter().any(|&c| dragon_creature_card(g, c)) {
        return Ok(vec![]);
    }
    if !g.player(p).library.iter().any(|&c| dragon_creature_card(g, c)) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.5,
        label: "Scion of the Ur-Dragon: a Dragon into the graveyard".into(),
        act: Some(Action::Ability { src, f: scion_go, arg: 0 }),
    }])
}

fn scion_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    if !ability_window(g, p, Some(src), "a Dragon into the graveyard", None, None)? {
        return Ok(true);
    }
    let cs: Vec<CardId> = g.player(p).library.iter().copied().filter(|&c| dragon_creature_card(g, c)).collect();
    if let Some(c) = crate::engine::zones::max_by(&cs, |c| body_worth(g, p, c)) {
        let lib = &mut g.player_mut(p).library;
        if let Some(i) = lib.iter().position(|&x| x == c) {
            lib.remove(i);
            g.player_mut(p).gy.push(c);
            crate::glog!(g, "    {} puts {} into the graveyard (Scion of the Ur-Dragon)", pname(g, p), cname(g, c));
        }
    }
    shuffle_library(g, p);
    Ok(true)
}

// ================================================================== Tiamat
/// When Tiamat enters, if you cast it, search your library for up to five Dragon cards not named Tiamat that each have
/// different names, put them into your hand, then shuffle
fn tiamat_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != owner(g, src) || !g.last_cast_etb {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "search for five Dragons", Some(6.0))? {
        return Ok(());
    }
    let mut cs: Vec<CardId> =
        g.player(p).library.iter().copied().filter(|&c| dragon_card(g, c) && cname(g, c) != TIAMAT).collect();
    cs.sort_by(|&a, &b| body_worth(g, p, b).total_cmp(&body_worth(g, p, a)));
    let mut got: Vec<CardId> = vec![];
    for c in cs {
        if got.len() < 5 && !got.iter().any(|&x| cname(g, x) == cname(g, c)) {
            got.push(c);
        }
    }
    for &c in &got {
        let lib = &mut g.player_mut(p).library;
        if let Some(i) = lib.iter().position(|&x| x == c) {
            lib.remove(i);
            g.player_mut(p).hand.push(c);
        }
    }
    let names: Vec<&str> = got.iter().map(|&c| cname(g, c)).collect();
    crate::glog!(g, "    Tiamat finds {}", names.join(", "));
    shuffle_library(g, p);
    Ok(())
}

// ================================================================== Ureni of the Unwritten
/// look at the top eight; you may put a Dragon creature card from among them onto the battlefield; the rest to the
/// bottom in a random order
fn ureni_dig(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if !trigger_window(g, p, Some(src), "the top eight: a Dragon onto the battlefield", Some(5.0))? {
        return Ok(());
    }
    let n = g.player(p).library.len().min(8);
    let mut top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
    let ds: Vec<CardId> = top.iter().copied().filter(|&c| dragon_creature_card(g, c)).collect();
    let pick = crate::engine::zones::max_by(&ds, |c| body_worth(g, p, c));
    if let Some(c) = pick {
        top.retain(|&x| x != c);
    }
    g.rng.shuffle(&mut top);
    g.player_mut(p).library.splice(0..0, top);
    if let Some(c) = pick {
        crate::glog!(g, "    Ureni puts {} onto the battlefield", cname(g, c));
        enter(g, p, c, Enter::default())?;
    }
    Ok(())
}

fn ureni_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != owner(g, src) {
        return Ok(());
    }
    ureni_dig(g, src, p)
}

fn ureni_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p == owner(g, src) && atk.contains(&src) {
        ureni_dig(g, src, p)?;
    }
    Ok(vec![])
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let has = |n: &str| db.id(n).is_some();
    if has(UR_DRAGON) {
        r.card(db, UR_DRAGON)?.attack = Some(ur_attack);
    }
    if has(BRASS) {
        r.card(db, BRASS)?.combat_damage = Some(brass_damage);
    }
    if has(COPPER) {
        r.card(db, COPPER)?.combat_damage = Some(copper_damage);
    }
    if has(GOLD) {
        r.card(db, GOLD)?.combat_damage = Some(gold_damage);
    }
    if has(SILVER) {
        r.card(db, SILVER)?.combat_damage = Some(silver_damage);
    }
    if has(CALL) {
        r.card(db, CALL)?.upkeep = Some(call_upkeep);
    }
    if has(HOARD) {
        let c = r.card(db, HOARD)?;
        c.etb = Some(hoard_etb);
        c.options = Some(hoard_options);
    }
    if has(HAVEN) {
        let c = r.card(db, HAVEN)?;
        c.land_cols = Some(haven_colors);
        c.land_options = Some(haven_options);
    }
    if has(COURSER) {
        r.card(db, COURSER)?.etb = Some(courser_etb);
    }
    if has(KLAUTH) {
        r.card(db, KLAUTH)?.attack = Some(klauth_attack);
    }
    if has(LATHLISS) {
        let c = r.card(db, LATHLISS)?;
        c.etb = Some(lathliss_etb);
        c.options = Some(lathliss_options);
    }
    if has(MIIRYM) {
        r.card(db, MIIRYM)?.etb = Some(miirym_etb);
    }
    if has(MOROPHON) {
        r.card(db, MOROPHON)?.as_enters = Some(morophon_as);
        r.creature_pt.push(morophon_pt);
    }
    if has(RITH) {
        r.card(db, RITH)?.end_step = Some(rith_end);
    }
    if has(RIVAZ) {
        let c = r.card(db, RIVAZ)?;
        c.options = Some(rivaz_options);
        c.dies = Some(rivaz_dies);
        c.dyn_mana_perm = Some(rivaz_mana);
    }
    if has(THRONE) {
        r.card(db, THRONE)?.as_enters = Some(throne_as);
    }
    if has(DRACO) {
        let c = r.card(db, DRACO)?;
        c.self_cost = Some(draco_cost);
        c.grant_kw = Some(draco_kw);
    }
    if has(SCION) {
        r.card(db, SCION)?.options = Some(scion_options);
    }
    if has(TIAMAT) {
        r.card(db, TIAMAT)?.etb = Some(tiamat_etb);
    }
    if has(URENI) {
        let c = r.card(db, URENI)?;
        c.etb = Some(ureni_etb);
        c.attack = Some(ureni_attack);
    }
    if has(INCUBATOR) {
        let c = r.card(db, INCUBATOR)?;
        c.as_enters = Some(incubator_as);
        c.cost = Some(incubator_cost);
    }
    Ok(())
}
