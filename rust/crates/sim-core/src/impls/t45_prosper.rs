//! Tier 4 pool deck `prosper-rakdos-exile-treasure` (EDHREC's bracket-4 average list): the cards the ability language
//! doesn't cover (rust/agents/t45-prosper.md). Rakdos Charm's third mode is compiled (dsl_parse.rs,
//! `// --- Tiers 4/5 (prosper)`); Tibalt's Trickery's side effect is `t6_counters::tibalt_after`, called from
//! `engine::stack::resolve_counter`; the rest is here.
//!
//! "Cast from exile": a card exiled to be played waits in the hand, marked in `impulse` (this turn) or `impulse_long`
//! (until a later turn), as Prosper's own cards do. A card cast by a cascade or by Etali comes off the library
//! (zone "lib"), so it is marked for the turn as it is found (`mark_exile_cast`); `cast_from_exile` reads both, and
//! Prosper's Treasure (`t4::prosper_treasure`) uses it too.
//!
//! - Delayed Blast Fireball: 2 damage to each opponent and each creature they control, 5 if cast from exile (foretold,
//!   or exiled by Prosper, Uba Mask ...). Foretell {2} on your turn after combat; the foretold card waits in the hand
//!   (an `impulse_long` mark far ahead, as bulk_white's Holy) and costs {4}{R}{R} from a later turn on.
//! - Passionate Archaeologist: your commander creature on the battlefield deals damage equal to the mana value of each
//!   spell you cast from exile to an opponent (one it kills, else the most threatening).
//! - Uba Mask: a player who would draw exiles the card instead; on their own turn it waits in the hand, playable this
//!   turn (`impulse`); drawn on another player's turn it goes straight to exile. Left out: an instant drawn on another
//!   player's turn could be cast that turn.
//! - Blightstep Pathway // Searstep Pathway (and the other Pathways): the face is chosen as the land is played (the
//!   colour the hand needs more), and it taps for that colour only.
//! - Bonehoard Dracosaur: each upkeep, the top two cards are playable this turn; a land among them makes a 3/1 red
//!   Dinosaur, a nonland card a Treasure.
//! - Crime Novelist: whenever you sacrifice an artifact (a Treasure, Clue or Food too), a +1/+1 counter and {R}.
//! - Disciple of the Vault: whenever an artifact (anyone's, artifact tokens too) goes to a graveyard from the
//!   battlefield, an opponent loses 1 life.
//! - Etali, Primal Storm: attacks: the top card of each player's library is exiled; the AI casts the nonland ones free
//!   (not counterspells, not board wipes); the rest stay in exile.
//! - Hex Magic: the cards in your hand are exiled and playable until the end of your next turn; draw that many.
//! - Ignite the Future: the top three cards are playable until the end of your next turn. Flashed back: the AI casts
//!   the castable nonland ones free at once (lands stay playable). Left out: a free card kept for the next turn.
//! - Marionette Master: fabricate 3 (counters when there are artifacts to sacrifice, else three Servos); an artifact
//!   of yours to the graveyard from the battlefield: an opponent loses life equal to its power.
//! - Nadier's Nightblade: a token of yours dies or is sacrificed (Treasure, Clue, Food too): each opponent loses 1 life
//!   and you gain 1. Left out: a token exiled or bounced.
//! - Ob Nixilis, Captive Kingpin: one or more opponents each lose exactly 1 life: a +1/+1 counter, and the top card is
//!   playable until your next end step. Losses in one event (each opponent losing 1 to one effect) trigger it once.
//! - Rain of Riches: the first spell you cast each turn with Treasure mana has cascade (its enters Treasures are
//!   compiled).
//! - Weftstalker Ardent: another creature or artifact of yours enters (Treasure, Clue, Food tokens too): 1 damage to
//!   each opponent. Warp {R}: exiled at the next end step, then castable from exile on a later turn.
//! - Wild-Magic Sorcerer: the first spell you cast from exile each turn has cascade. Left out: a spell cast from exile
//!   before the Sorcerer arrived still counts as the first.
//! - Bolas's Citadel + Sensei's Divining Top + Aetherflux Reservoir: the combo (`COMBOS`).

use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable};
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, land_colors_now, pay};
use crate::engine::stack::trigger_window;
use crate::engine::tutors::card_worth;
use crate::engine::values::{epow, etgh, indestructible, no_damage, pval, threat};
use crate::engine::zones::{Tokens, add_treasure, die, draw, make_tokens};
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Opt, Registry, Sacrificed, Src, TrigAct, Trigger};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::impls::combos::{Combo, Ready};
use crate::impls::partials::at_once;
use crate::state::{Ctx, DataKey, Game, Step, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

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

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn opps(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    g.opps(p).collect()
}

/// a permanent that is an artifact (its card's type, or an artifact token: Servo, Thopter)
fn is_artifact(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)) || (x.cd.is_none() && x.ttypes.contains(&"artifact"))
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && g.perm(m).cd.is_some_and(|c| cname(g, c) == name))
}

/// the opponent a "target opponent" damage or life loss of n goes to: one it puts out, else the most threatening
fn pick_opp(g: &Game, p: PlayerId, n: i32) -> Option<PlayerId> {
    let qs = opps(g, p);
    if let Some(&q) = qs.iter().filter(|&&q| g.player(q).life <= n).min_by_key(|&&q| g.player(q).life) {
        return Some(q);
    }
    qs.into_iter().max_by(|&a, &b| threat(g, p, a).total_cmp(&threat(g, p, b)))
}

fn flag_now(g: &Game, p: PlayerId, key: &str) -> bool {
    g.player(p).flag_turn.get(&intern(key)) == Some(&g.turn_stamp())
}

fn set_flag(g: &mut Game, p: PlayerId, key: &str) {
    let st = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(intern(key), st);
}

fn clear_flag(g: &mut Game, p: PlayerId, key: &str) {
    g.player_mut(p).flag_turn.shift_remove(&intern(key));
}

fn xmark(c: CardId) -> String {
    format!("t45_xc_{}", c.0)
}

/// c, about to come off p's library by an effect that exiles it first (a cascade, Etali), is cast from exile this turn
pub fn mark_exile_cast(g: &mut Game, p: PlayerId, c: CardId) {
    set_flag(g, p, &xmark(c));
}

/// did p cast (or is p casting) c from exile? A card waiting in the hand marked `impulse` / `impulse_long` (Prosper,
/// Uba Mask, foretell, warp ...), or one a cascade or Etali exiled this turn (`mark_exile_cast`)
pub fn cast_from_exile(g: &Game, p: PlayerId, c: CardId) -> bool {
    let pl = g.player(p);
    pl.impulse.contains(&c) || pl.impulse_long.iter().any(|x| x.0 == c) || flag_now(g, p, &xmark(c))
}

/// an exiled card is held in the hand, playable until the end of p's turn number `until` (`impulse_long`)
fn exile_playable(g: &mut Game, p: PlayerId, c: CardId, until: u32) {
    let pl = g.player_mut(p);
    pl.hand.push(c);
    pl.seen.insert(c);
    pl.impulse_long.push((c, until));
}

/// cascade from a spell of mana value mv (`legends::cascade`), the card it finds marked as cast from exile
fn exile_cascade(g: &mut Game, o: PlayerId, mv: u32, who: &str) -> Res<bool> {
    let hit = g.player(o).library.iter().rev().copied().find(|&x| {
        let d = g.db.get(x);
        !d.land && d.cmc < mv
    });
    if let Some(h) = hit {
        mark_exile_cast(g, o, h);
    }
    crate::impls::legends::cascade(g, o, mv, who)
}

fn plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

// ================================================================== Delayed Blast Fireball
const FIREBALL: &str = "Delayed Blast Fireball";
const FIREBALL_EXILE: &str = "t45_fireball_exile";

/// the turn a card was foretold: `impulse_long` marks `u32::MAX - 1 - turn` (bulk_white's foretell)
fn foretold_turn(g: &Game, p: PlayerId, c: CardId) -> Option<u32> {
    g.player(p)
        .impulse_long
        .iter()
        .find(|x| x.0 == c && x.1 < u32::MAX && x.1 > u32::MAX / 2)
        .map(|x| u32::MAX - 1 - x.1)
}

/// foretold: {4}{R}{R} instead of {1}{R}{R} (3 more), and not castable on the turn it was foretold
fn fireball_cost(g: &Game, p: PlayerId, c: CardId) -> i32 {
    match foretold_turn(g, p, c) {
        Some(t) if t == g.player(p).turns && g.active == Some(p) => 99,
        Some(_) => 3,
        None => 0,
    }
}

/// what the Fireball does to p's opponents (and their creatures) for n damage, in cards' worth
fn fireball_value(g: &Game, p: PlayerId, n: i32) -> f64 {
    let mut v = 0.0;
    for q in opps(g, p) {
        v += 0.25 * n.min(g.player(q).life) as f64;
        if g.player(q).life <= n {
            v += 20.0;
        }
        for &m in &g.player(q).perms {
            if live(g, m) && g.is_creature(m) && etgh(g, m) <= n && !indestructible(g, m) && !no_damage(g, m) {
                v += pval(g, m).max(0.5);
            }
        }
    }
    v
}

/// cast it when it kills enough (an opponent, or creatures worth about two cards)
fn fireball_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let n = if cast_from_exile(g, p, c) { 5 } else { 2 };
    let v = fireball_value(g, p, n);
    if v >= 20.0 {
        85
    } else if v >= 3.5 {
        (40.0 + 4.0 * v).min(80.0) as i32
    } else {
        0
    }
}

fn fireball_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    if cast_from_exile(g, p, c) {
        set_flag(g, p, FIREBALL_EXILE);
    } else {
        clear_flag(g, p, FIREBALL_EXILE);
    }
    Ok(())
}

fn fireball_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = if flag_now(g, p, FIREBALL_EXILE) { 5 } else { 2 };
    clear_flag(g, p, FIREBALL_EXILE);
    crate::glog!(g, "    {FIREBALL}: {n} damage to each opponent and each creature they control");
    for q in opps(g, p) {
        for m in g.player(q).perms.clone() {
            if live(g, m) && g.is_creature(m) && etgh(g, m) <= n && !indestructible(g, m) && !no_damage(g, m) {
                die(g, m, "destroy")?;
            }
        }
    }
    for q in opps(g, p) {
        lose_life(g, q, n, Some(p), "burn", None)?;
    }
    Ok("gy")
}

/// foretell {2}: on your turn after combat, with two mana to spare
fn fireball_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(true) || g.active != Some(p) || foretold_turn(g, p, c).is_some() || cast_from_exile(g, p, c) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    Ok(vec![plan(0.8, format!("{FIREBALL} (foretell)"), fireball_foretell, c.0 as i64)])
}

fn fireball_foretell(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || foretold_turn(g, p, c).is_some() || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    if !pay(g, p, 2, "", false)? {
        return Ok(false);
    }
    let pl = g.player_mut(p);
    let mark = u32::MAX - 1 - pl.turns;
    pl.impulse_long.push((c, mark));
    crate::glog!(g, "  {} foretells {FIREBALL}", pname(g, p));
    Ok(true)
}

// ================================================================== Passionate Archaeologist
/// Commander creatures you own have "Whenever you cast a spell from exile, this creature deals damage equal to that
/// spell's mana value to target opponent."
fn archaeologist_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    let n = g.db.get(c).cmc as i32;
    if caster != o || n <= 0 || !cast_from_exile(g, o, c) {
        return Ok(());
    }
    let cmds: Vec<PermId> =
        g.player(o).perms.iter().copied().filter(|&m| live(g, m) && g.perm(m).is_cmd && g.is_creature(m)).collect();
    for m in cmds {
        let label = format!("{n} damage to target opponent ({})", g.perm(m).name);
        if trigger_window(g, o, Some(m), &label, None)?
            && live(g, m)
            && let Some(q) = pick_opp(g, o, n)
        {
            lose_life(g, q, n, Some(o), "triggers", None)?;
        }
    }
    Ok(())
}

// ================================================================== Uba Mask
const UBA: &str = "Uba Mask";

/// If a player would draw a card, that player exiles that card face up instead (`zones::draw`): on their own turn it
/// waits in the hand, playable this turn (`impulse`); on another player's turn it goes to exile. True: replaced.
pub fn uba_draw(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(uba) = g.db.id(UBA) else { return Ok(false) };
    let mask = g.players.iter().any(|q| q.alive && q.perms.iter().any(|&m| g.perm(m).cd == Some(uba) && live(g, m)));
    if !mask {
        return Ok(false);
    }
    let Some(c) = g.player_mut(p).library.pop() else { return Ok(false) };
    let mine = g.active == Some(p);
    let pl = g.player_mut(p);
    pl.seen.insert(c);
    if mine {
        pl.hand.push(c);
        pl.impulse.push(c);
    } else {
        pl.exile.push(c);
    }
    crate::glog!(
        g,
        "    {} exiles {} instead of drawing it (Uba Mask){}",
        pname(g, p),
        cname(g, c),
        if mine { ": playable this turn" } else { "" }
    );
    Ok(true)
}

// ================================================================== Pathways
/// the colour a Pathway's face gives: the one p's hand needs more, against the sources p already has
fn pathway_pick(g: &Game, p: PlayerId, l: LandId) -> char {
    let d = g.db.get(g.land(l).cd);
    let cols: Vec<char> = d.tags.str(Tag::C).unwrap_or("").chars().collect();
    let need = |col: char| -> i32 {
        let pips: i32 = g
            .player(p)
            .hand
            .iter()
            .filter(|&&c| !g.db.get(c).land)
            .map(|&c| g.db.get(c).pips.chars().filter(|&x| x == col).count() as i32)
            .sum();
        let have = g.player(p).lands.iter().filter(|&&x| x != l && land_colors_now(g, p, x).has(col)).count() as i32;
        2 * pips - 3 * have
    };
    let mut best = cols.first().copied().unwrap_or('C');
    for &col in cols.iter().skip(1) {
        if need(col) > need(best) {
            best = col;
        }
    }
    best
}

/// played as a land: the face is chosen (`pathway_pick`) and kept
fn pathway_etb(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    let col = pathway_pick(g, p, l);
    let s = intern(&col.to_string());
    g.land_mut(l).data.set(DataKey::ChosenColor, Val::Str(s));
    crate::glog!(g, "    {} plays {} as the {{{col}}} face", pname(g, p), cname(g, g.land(l).cd));
    Ok(())
}

/// a Pathway's front face colour: one put onto the battlefield without being played enters front face up
pub(crate) fn pathway_front(name: &str) -> &'static str {
    match name {
        "Barkchannel Pathway // Tidechannel Pathway" => "G",
        "Blightstep Pathway // Searstep Pathway" => "B",
        "Clearwater Pathway // Murkwater Pathway" => "U",
        "Needleverge Pathway // Pillarverge Pathway" => "R",
        "Branchloft Pathway // Boulderloft Pathway" => "G", // --- Tiers 1/2 (kaalia-sythis)
        _ => "C",
    }
}

/// the face it was played as, else its front face (never worked out again here: a pick reads the other lands'
/// colours, and an Exotic Orchard reads an opponent's Pathway, which looped without end)
fn pathway_cols(g: &Game, _p: PlayerId, l: LandId) -> Colors {
    match g.land(l).data.get(DataKey::ChosenColor) {
        Some(Val::Str(s)) => Colors::from_letters(s),
        _ => Colors::from_letters(pathway_front(&g.db.get(g.land(l).cd).name)),
    }
}

// ================================================================== Bonehoard Dracosaur
/// At the beginning of your upkeep, exile the top two cards of your library. You may play them this turn. A land
/// among them: a 3/1 red Dinosaur. A nonland card: a Treasure.
fn dracosaur_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if p != o || g.player(o).library.is_empty() || !trigger_window(g, o, Some(src), "exile the top two cards", None)? {
        return Ok(());
    }
    let (mut land, mut spell) = (false, false);
    for _ in 0..2 {
        let Some(c) = g.player_mut(o).library.pop() else { break };
        land |= g.db.get(c).land;
        spell |= !g.db.get(c).land;
        let pl = g.player_mut(o);
        pl.hand.push(c);
        pl.seen.insert(c);
        pl.impulse.push(c);
        crate::glog!(g, "    {} exiles {} (playable this turn)", pname(g, o), cname(g, c));
    }
    if land {
        let spec = Tokens { color: Some(Colors::from_letters("R")), types: vec!["dinosaur"], ..Tokens::new(1, 3) };
        make_tokens(g, o, Tokens { tgh: Some(1), ..spec })?;
    }
    if spell {
        add_treasure(g, o, 1)?;
    }
    Ok(())
}

// ================================================================== Crime Novelist
/// was an artifact sacrificed? (a Treasure, Clue, Food or Blood token, or an artifact permanent)
fn artifact_sacrificed(g: &Game, what: Sacrificed) -> bool {
    match what {
        Sacrificed::Token(_) => true,
        Sacrificed::Perm(m) => is_artifact(g, m),
        _ => false,
    }
}

/// Whenever you sacrifice an artifact, put a +1/+1 counter on it and add {R} (a triggered mana ability: at once)
fn novelist_sac(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p != owner(g, src) || !live(g, src) || !artifact_sacrificed(g, what) {
        return Ok(());
    }
    crate::cardcode::put_plus(g, src, 1);
    g.player_mut(p).floating.r += 1;
    Ok(())
}

// ================================================================== Disciple of the Vault
fn disciple_drain(g: &mut Game, src: Src) -> Res {
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "target opponent loses 1 life", None)? {
        return Ok(());
    }
    if let Some(q) = pick_opp(g, o, 1) {
        lose_life(g, q, 1, Some(o), "drain", None)?;
    }
    Ok(())
}

/// an artifact permanent (anyone's) to the graveyard from the battlefield
fn disciple_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m == src || !is_artifact(g, m) {
        return Ok(());
    }
    disciple_drain(g, src)
}

/// a Treasure, Clue or Food token (anyone's) sacrificed (permanents sacrificed die through `dies`)
fn disciple_sac(g: &mut Game, src: Src, _p: PlayerId, what: Sacrificed) -> Res {
    if !matches!(what, Sacrificed::Token(_)) {
        return Ok(());
    }
    disciple_drain(g, src)
}

// ================================================================== Etali, Primal Storm
/// a card the AI casts free off Etali: not a counterspell, not a board wipe (it would hit its own board)
fn etali_wants(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    !d.land && !d.tag(Tag::Ctr) && !d.tag(Tag::Wipe)
}

/// Whenever Etali attacks, exile the top card of each player's library, then you may cast any number of spells from
/// among those cards without paying their mana costs.
fn etali_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let o = owner(g, src);
    if p != o || !atk.contains(&src) || !trigger_window(g, o, Some(src), "exile the top card of each library", None)? {
        return Ok(vec![]);
    }
    let mut got: Vec<(CardId, PlayerId)> = vec![];
    for q in g.players.iter().filter(|x| x.alive).map(|x| x.id).collect::<Vec<_>>() {
        if let Some(c) = g.player_mut(q).library.pop() {
            g.player_mut(q).exile.push(c);
            crate::glog!(g, "    Etali exiles {} ({})", cname(g, c), pname(g, q));
            got.push((c, q));
        }
    }
    got.sort_by(|a, b| card_worth(g, o, b.0, false).total_cmp(&card_worth(g, o, a.0, false)));
    for (c, q) in got {
        if g.over || !g.player(o).alive || !etali_wants(g, c) || !castable(g, o, c, "lib") {
            continue;
        }
        let Some(i) = g.player(q).exile.iter().position(|&x| x == c) else { continue };
        g.player_mut(q).exile.remove(i);
        mark_exile_cast(g, o, c);
        g.last_x = 0; // cast free: X is 0
        crate::glog!(g, "    {} casts {} free (Etali)", pname(g, o), cname(g, c));
        let ctx = Ctx { owner: (q != o).then_some(q), ..crate::engine::cast::spell_targets(g, o, c, None) };
        cast_card(g, o, c, "lib", ctx)?;
    }
    Ok(vec![])
}

// ================================================================== Hex Magic
/// the cards really in p's hand: not ones waiting in exile (impulse marks) or taken from an opponent
fn hand_proper(g: &Game, p: PlayerId) -> Vec<CardId> {
    let pl = g.player(p);
    pl.hand
        .iter()
        .copied()
        .filter(|c| {
            !pl.impulse.contains(c)
                && !pl.impulse_long.iter().any(|x| x.0 == *c)
                && !pl.stolen.contains_key(c)
                && !pl.agent_ids.contains(c)
        })
        .collect()
}

/// Exile all the cards from your hand, then draw that many cards. Until the end of your next turn, you may play cards
/// exiled this way.
fn hex_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let cards = hand_proper(g, p);
    let until = g.player(p).turns + 1;
    let pl = g.player_mut(p);
    for &c in &cards {
        pl.impulse_long.push((c, until));
    }
    crate::glog!(
        g,
        "    {} exiles {} cards from hand (playable until next turn) and draws as many",
        pname(g, p),
        cards.len()
    );
    draw(g, p, cards.len() as u32, false)?;
    Ok("gy")
}

/// worth it with three or more cards in hand (two with Prosper out: two Treasures)
fn hex_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let k = hand_proper(g, p).into_iter().filter(|&x| x != c).count();
    let prosper = on_bf(g, p, "Prosper, Tome-Bound").is_some();
    if k >= 3 || (k >= 2 && prosper) { 46 } else { 0 }
}

// ================================================================== Ignite the Future
const IGNITE_FB: &str = "t45_ignite_fb";

/// cast from a graveyard (flashback)?
fn ignite_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let fb = g.cur_cast.as_ref().is_some_and(|x| x.0 == c && matches!(x.2, "gy" | "escape" | "mgy"));
    if fb {
        set_flag(g, p, IGNITE_FB);
    } else {
        clear_flag(g, p, IGNITE_FB);
    }
    Ok(())
}

/// Exile the top three cards of your library. Until the end of your next turn, you may play those cards. Flashed
/// back, they are free: the AI casts the castable nonland ones at once.
fn ignite_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let free = flag_now(g, p, IGNITE_FB);
    clear_flag(g, p, IGNITE_FB);
    let until = g.player(p).turns + 1;
    let mut top = vec![];
    for _ in 0..3 {
        let Some(c) = g.player_mut(p).library.pop() else { break };
        exile_playable(g, p, c, until);
        top.push(c);
    }
    crate::glog!(
        g,
        "    {} exiles {} cards (playable until the end of next turn){}",
        pname(g, p),
        top.len(),
        if free { ", free" } else { "" }
    );
    if free {
        top.sort_by(|a, b| card_worth(g, p, *b, false).total_cmp(&card_worth(g, p, *a, false)));
        for c in top {
            if g.over || !g.player(p).alive || !g.player(p).hand.contains(&c) || !etali_wants(g, c) {
                continue;
            }
            if !castable(g, p, c, "hand") {
                continue;
            }
            g.last_x = 0;
            crate::glog!(g, "    {} casts {} free (Ignite the Future)", pname(g, p), cname(g, c));
            let ctx = crate::engine::cast::spell_targets(g, p, c, None);
            cast_card(g, p, c, "hand", ctx)?;
        }
    }
    Ok("gy")
}

// ================================================================== Marionette Master
/// Fabricate 3: three +1/+1 counters when there are artifacts to sacrifice (each death drains for its power), else
/// three 1/1 Servos
fn master_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "fabricate 3", None)? || !live(g, src) {
        return Ok(());
    }
    let pl = g.player(o);
    let fodder = pl.treasures
        + pl.clues
        + pl.foods
        + pl.perms.iter().filter(|&&x| x != src && live(g, x) && is_artifact(g, x)).count() as u32;
    if fodder >= 2 {
        crate::cardcode::put_plus(g, src, 3);
        crate::glog!(g, "    fabricate 3: three +1/+1 counters");
    } else {
        let spec = Tokens { color: Some(Colors::NONE), types: vec!["servo", "artifact"], ..Tokens::new(3, 1) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

fn master_drain(g: &mut Game, src: Src) -> Res {
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "target opponent loses life equal to its power", None)? {
        return Ok(());
    }
    let n = epow(g, src);
    if n > 0
        && let Some(q) = pick_opp(g, o, n)
    {
        lose_life(g, q, n, Some(o), "drain", None)?;
    }
    Ok(())
}

/// an artifact you control is put into a graveyard from the battlefield
fn master_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m == src || g.perm(m).owner != owner(g, src) || !is_artifact(g, m) {
        return Ok(());
    }
    master_drain(g, src)
}

fn master_sac(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p != owner(g, src) || !matches!(what, Sacrificed::Token(_)) {
        return Ok(());
    }
    master_drain(g, src)
}

// ================================================================== Nadier's Nightblade
fn nightblade_drain(g: &mut Game, src: Src) -> Res {
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "each opponent loses 1 life, you gain 1", None)? {
        return Ok(());
    }
    for q in opps(g, o) {
        lose_life(g, q, 1, Some(o), "drain", None)?;
    }
    gain(g, o, 1)
}

/// a token you control leaves the battlefield: dies or is sacrificed
fn nightblade_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m == src || !g.perm(m).token || g.perm(m).owner != owner(g, src) {
        return Ok(());
    }
    nightblade_drain(g, src)
}

fn nightblade_sac(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p != owner(g, src) || !matches!(what, Sacrificed::Token(_)) {
        return Ok(());
    }
    nightblade_drain(g, src)
}

// ================================================================== Ob Nixilis, Captive Kingpin
/// (game work, stack pushes) and the opponents seen: losses with the same key are one event
fn ob_key(g: &Game) -> (i64, i64) {
    (g.work as i64, g.stack_pushes as i64)
}

/// Whenever one or more opponents each lose exactly 1 life: a +1/+1 counter, and exile the top card of your library;
/// until your next end step, you may play it. The loss is seen at once (to tell one event from the next); a new event
/// queues the trigger, which comes back here as the sentinel `Life { p: you, n: 0 }` to resolve.
fn ob_lose(g: &mut Game, src: Src, q: PlayerId, n: i32) -> Res {
    let o = owner(g, src);
    if q == o && n == 0 {
        return ob_resolve(g, src);
    }
    if q == o || n != 1 || !live(g, src) {
        return Ok(());
    }
    let (w, s) = ob_key(g);
    if let Some(Val::List(v)) = g.perm(src).data.get(DataKey::ObBatch)
        && let [Val::Int(w0), Val::Int(s0), seen @ ..] = v.as_slice()
        && (*w0, *s0) == (w, s)
        && !seen.contains(&Val::Player(q))
    {
        // another opponent losing 1 in the same event
        let mut nv = v.clone();
        nv.push(Val::Player(q));
        g.perm_mut(src).data.set(DataKey::ObBatch, Val::List(nv));
        return Ok(());
    }
    let t = Trigger {
        controller: o,
        src: Some(src),
        act: TrigAct::Hook { event: Event::LoseLife, call: Call::Life { p: o, n: 0 } },
        name: None,
        known: false,
        imp: None,
        cast_etb: false,
    };
    crate::engine::stack::queue_triggers(g, vec![t])?;
    let (w, s) = ob_key(g);
    g.perm_mut(src).data.set(DataKey::ObBatch, Val::List(vec![Val::Int(w), Val::Int(s), Val::Player(q)]));
    Ok(())
}

fn ob_resolve(g: &mut Game, src: Src) -> Res {
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "a +1/+1 counter, exile the top card (play it until your next end step)", None)?
        || !live(g, src)
    {
        return Ok(());
    }
    crate::cardcode::put_plus(g, src, 1);
    if let Some(c) = g.player_mut(o).library.pop() {
        let pl = g.player(o);
        let until = if g.active == Some(o) && g.step != Step::End { pl.turns } else { pl.turns + 1 };
        exile_playable(g, o, c, until);
        crate::glog!(g, "    {} exiles {} (playable until the next end step)", pname(g, o), cname(g, c));
    }
    Ok(())
}

// ================================================================== Rain of Riches
const RAIN_USED: &str = "t45_rain";

/// The first spell you cast each turn that mana from a Treasure was spent to cast has cascade (run at once: the
/// Treasure record is the cast's own)
fn rain_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    let treasure = g.cast_treasure.is_some_and(|x| x.0 == c && x.1 > 0);
    if caster != o || !treasure || flag_now(g, o, RAIN_USED) {
        return Ok(());
    }
    set_flag(g, o, RAIN_USED);
    let mv = g.db.get(c).cmc;
    if mv > 0 && trigger_window(g, o, Some(src), &format!("cascade (mana value below {mv})"), Some(4.0))? {
        exile_cascade(g, o, mv, "Rain of Riches")?;
    }
    Ok(())
}

// ================================================================== Weftstalker Ardent
const ARDENT: &str = "Weftstalker Ardent";

fn ardent_ping(g: &mut Game, src: Src, n: i32) -> Res {
    let o = owner(g, src);
    for k in 0..n {
        if !g.player(o).alive || !trigger_window(g, o, Some(src), "1 damage to each opponent", None)? {
            continue;
        }
        if k > 0 {
            g.tick()?; // a separate event each (Ob Nixilis)
        }
        for q in opps(g, o) {
            lose_life(g, q, 1, Some(o), "triggers", None)?;
        }
    }
    Ok(())
}

/// another creature or artifact you control enters
fn ardent_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src || p != owner(g, src) || !(g.is_creature(m) || is_artifact(g, m)) {
        return Ok(());
    }
    ardent_ping(g, src, 1)
}

/// Treasure, Clue and Food tokens are artifacts entering
fn ardent_tokens(g: &mut Game, src: Src, p: PlayerId, _kinds: &[Sym], n: i32) -> Res {
    if p != owner(g, src) || n <= 0 {
        return Ok(());
    }
    ardent_ping(g, src, n)
}

/// Warp {R} from the hand: before combat when the full cost is out of reach, or when a cast from exile pays off later
/// (Prosper, Passionate Archaeologist, Wild-Magic Sorcerer)
fn ardent_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || cast_from_exile(g, p, c) || !can_pay(g, p, 0, "R", false) {
        return Ok(vec![]);
    }
    let (cg, cp) = crate::engine::mana::cost_of(g, p, c);
    let payoff = ["Prosper, Tome-Bound", "Passionate Archaeologist", "Wild-Magic Sorcerer"]
        .iter()
        .any(|n| on_bf(g, p, n).is_some());
    if can_pay(g, p, cg, &cp, false) && !payoff {
        return Ok(vec![]);
    }
    Ok(vec![plan(2.5, format!("{ARDENT} (warp)"), ardent_warp, c.0 as i64)])
}

fn ardent_warp(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || cast_from_exile(g, p, c) || !can_pay(g, p, 0, "R", false) {
        return Ok(false);
    }
    if !pay(g, p, 0, "R", false)? {
        return Ok(false);
    }
    cast_card(g, p, c, "hand", Ctx::default())?;
    let found = g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(c));
    if let Some(m) = found {
        g.perm_mut(m).data.set(DataKey::Warp, Val::Bool(true));
    }
    Ok(true)
}

/// warped: exiled at the beginning of the next end step; castable from exile on a later turn (it waits in the hand
/// with an `impulse_long` mark that never runs out)
fn ardent_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    if !g.perm(src).on_bf || !g.perm(src).data.truthy(DataKey::Warp) {
        return Ok(());
    }
    let o = owner(g, src);
    let Some(c) = g.perm(src).cd else { return Ok(()) };
    g.perm_mut(src).data.remove(DataKey::Warp);
    crate::glog!(g, "    {ARDENT} (warp) is exiled");
    crate::engine::zones::leave(g, src)?;
    exile_playable(g, o, c, u32::MAX);
    Ok(())
}

// ================================================================== Wild-Magic Sorcerer
const SORCERER_USED: &str = "t45_wms";

/// The first spell you cast from exile each turn has cascade.
fn sorcerer_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    if caster != o || !cast_from_exile(g, o, c) || flag_now(g, o, SORCERER_USED) {
        return Ok(());
    }
    set_flag(g, o, SORCERER_USED);
    let mv = g.db.get(c).cmc;
    if mv > 0 && trigger_window(g, o, Some(src), &format!("cascade (mana value below {mv})"), Some(4.0))? {
        exile_cascade(g, o, mv, "Wild-Magic Sorcerer")?;
    }
    Ok(())
}

// ================================================================== the combo
const CITADEL: &str = "Bolas's Citadel";
const TOP: &str = "Sensei's Divining Top";
const RESERVOIR: &str = "Aetherflux Reservoir";

/// Bolas's Citadel + Sensei's Divining Top + Aetherflux Reservoir on p's battlefield, on p's turn: tap Top to draw (Top
/// goes on top), cast it off the library for 1 life, again and again; each cast gains Reservoir's growing life, the
/// library is drawn, and Reservoir shoots 50 at each opponent. Needs an untapped Top (or Top on top of the library),
/// 3 life to start and a library to draw.
fn citadel_ready(g: &Game, p: PlayerId) -> Ready {
    let no = || (false, vec![], vec![]);
    if g.active != Some(p) || !matches!(g.step, Step::Main1 | Step::Main2) || g.player(p).life < 3 {
        return no();
    }
    let (Some(c), Some(r)) = (on_bf(g, p, CITADEL), on_bf(g, p, RESERVOIR)) else { return no() };
    let top_up = on_bf(g, p, TOP).filter(|&t| !g.perm(t).tapped);
    let top_on_top = g.player(p).library.last().is_some_and(|&x| cname(g, x) == TOP);
    if g.player(p).library.len() < 15 {
        return no();
    }
    match top_up {
        Some(t) => (true, vec![c, t, r], vec![]),
        None if top_on_top => (true, vec![c, r], vec![]),
        None => no(),
    }
}

pub static COMBOS: [Combo; 1] = [Combo {
    name: "Bolas's Citadel + Sensei's Divining Top + Aetherflux Reservoir",
    groups: &[&[CITADEL], &[TOP], &[RESERVOIR]],
    ready: citadel_ready,
    mana: (0, ""),
    finish: None,
    xcost: &[],
    steps: &[],
}];

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, FIREBALL)?;
    c.resolve = Some(fireball_resolve);
    c.self_cast = Some(fireball_cast);
    c.self_cost = Some(fireball_cost);
    c.hand_options = Some(fireball_options);
    c.prio = Some(fireball_prio);
    r.card(db, "Passionate Archaeologist")?.cast = Some(archaeologist_cast);
    for name in [
        "Blightstep Pathway // Searstep Pathway",
        "Barkchannel Pathway // Tidechannel Pathway",
        "Clearwater Pathway // Murkwater Pathway",
        "Needleverge Pathway // Pillarverge Pathway",
        "Branchloft Pathway // Boulderloft Pathway", // --- Tiers 1/2 (kaalia-sythis)
    ] {
        if db.id(name).is_some() {
            let c = r.card(db, name)?;
            c.land_etb = Some(pathway_etb);
            c.land_cols = Some(pathway_cols);
        }
    }
    r.card(db, "Bonehoard Dracosaur")?.upkeep = Some(dracosaur_upkeep);
    let c = r.card(db, "Crime Novelist")?;
    c.sacrifice = Some(novelist_sac);
    at_once(c, Event::Sacrifice, true);
    let c = r.card(db, "Disciple of the Vault")?;
    c.dies = Some(disciple_dies);
    c.sacrifice = Some(disciple_sac);
    r.card(db, "Etali, Primal Storm")?.attack = Some(etali_attack);
    let c = r.card(db, "Hex Magic")?;
    c.resolve = Some(hex_resolve);
    c.prio = Some(hex_prio);
    let c = r.card(db, "Ignite the Future")?;
    c.resolve = Some(ignite_resolve);
    c.self_cast = Some(ignite_cast);
    let c = r.card(db, "Marionette Master")?;
    c.etb = Some(master_etb);
    c.dies = Some(master_dies);
    c.sacrifice = Some(master_sac);
    let c = r.card(db, "Nadier's Nightblade")?;
    c.dies = Some(nightblade_dies);
    c.sacrifice = Some(nightblade_sac);
    let c = r.card(db, "Ob Nixilis, Captive Kingpin")?;
    c.lose_life = Some(ob_lose);
    at_once(c, Event::LoseLife, true);
    let c = r.card(db, "Rain of Riches")?;
    c.cast = Some(rain_cast);
    at_once(c, Event::Cast, true);
    let c = r.card(db, ARDENT)?;
    c.etb = Some(ardent_etb);
    c.token_created = Some(ardent_tokens);
    c.hand_options = Some(ardent_options);
    c.end_step = Some(ardent_end);
    r.card(db, "Wild-Magic Sorcerer")?.cast = Some(sorcerer_cast);
    // Uba Mask has no hook of its own: `zones::draw` asks `uba_draw`
    Ok(())
}
