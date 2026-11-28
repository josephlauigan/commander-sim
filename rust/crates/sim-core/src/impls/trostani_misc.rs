//! trostani deck (registered 2026-11-27): combat and AI pieces (Elder Gargaroth, Sunblast Angel's cast rule, Aetherflux
//! Reservoir's shot, Castle Ardenvale, and Cloudshift's protector scope). The defender filter is in `engine/combat.rs`
//! (the AI's attacker list) and Cloudshift's scope in `ai/pool.rs` (`protect_cands`, which asks `cloudshift_protects`).
//!
//! The design is /tmp/handoff/trostani-opus-design.md ("AI needs"); each rule below says what its Oracle text asks.
//! Gavony Township's AI is Batch A's.

use crate::cards::{CardDb, Colors};
use crate::engine::life::lose_life;
use crate::engine::mana::{Source, cost_of, mana_units, plan_pay};
use crate::engine::repl;
use crate::engine::stack::trigger_window;
use crate::engine::values::{indestructible, pval, threat};
use crate::engine::zones::{Tokens, draw, make_tokens};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::Game;

const GARGAROTH: &str = "Elder Gargaroth";
const SUNBLAST: &str = "Sunblast Angel";
const RESERVOIR: &str = "Aetherflux Reservoir";
const ARDENVALE: &str = "Castle Ardenvale";
const RETREAT: &str = "Felidar Retreat";

/// the deck key these AI rules belong to
const DECK: &str = "trostani";

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

/// does p control a permanent called `name` (phased in)?
fn controls(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| live(g, m) && g.perm(m).name == name)
}

fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

// ================================================================== Cloudshift (the AI's scope, read by ai/pool.rs)
/// Cloudshift is a protector for the Trostani deck: it blinks a creature out of targeted removal (the removal spell
/// fizzles) and the creature's enters trigger happens again. Every other deck holds it for value or a combo as before
/// (Karn, Gilded Guardian has its own reason: `karn_gilded::reenter_bonus`).
pub fn cloudshift_protects(g: &Game, owner: PlayerId) -> bool {
    g.player(owner).key == DECK
}

// ================================================================== Elder Gargaroth
// Reach, vigilance, trample. Whenever this creature attacks or blocks, choose one: a 3/3 green Beast token; you gain
// 3 life; draw a card. (It has no enters trigger.)
const BEAST: u8 = 0;
const LIFE: u8 = 1;
const CARD: u8 = 2;

/// the upkeep wins of this deck: a permanent that wins at the start of your upkeep with this much life
const UPKEEP_WINS: [(&str, i32); 2] = [("Felidar Sovereign", 40), ("Test of Endurance", 50)];

/// permanents that pay off a creature entering: Trostani's life (and her populate), the Avatar's life, Crusade's counters
const ENTER_PAYOFFS: [&str; 3] = ["Trostani, Selesnya's Voice", "Verdant Sun's Avatar", "Cathars' Crusade"];

/// the AI's mode: life when the gain takes it across the line of an upkeep win it has out (40 Felidar, 50 Test), a
/// Beast when a payoff for a creature entering is out or no other creature is left to block, otherwise a card
fn gargaroth_mode(g: &Game, p: PlayerId, src: PermId) -> u8 {
    let life = g.player(p).life;
    // a locked life total gains nothing, and a gain that is replaced to 0 reaches no line
    if !g.player(p).life_locked
        && let Some(gain) = repl::life_gain(g, p, 3).filter(|&n| n > 0)
    {
        for (name, line) in UPKEEP_WINS {
            if controls(g, p, name) && life < line && life + gain >= line {
                return LIFE;
            }
        }
    }
    let others = g.player(p).perms.iter().any(|&m| m != src && live(g, m) && g.is_creature(m));
    if !others || ENTER_PAYOFFS.iter().any(|n| controls(g, p, n)) {
        return BEAST;
    }
    CARD
}

fn gargaroth_modes(g: &mut Game, p: PlayerId, src: PermId) -> Res<u8> {
    if crate::human::is_human(g, p) {
        let labels = vec![
            "Create a 3/3 green Beast creature token".to_string(),
            "You gain 3 life".to_string(),
            "Draw a card".to_string(),
        ];
        let i = crate::human::choices::choose_one(g, p, "mode", "Elder Gargaroth: choose one", &labels)?;
        return Ok(i as u8);
    }
    Ok(gargaroth_mode(g, p, src))
}

/// the trigger: one mode, chosen once as it goes on the stack, done as it resolves. The source need not still be on the
/// battlefield (CR 113.7a: an ability on the stack exists independently of its source), so there is no `live(src)` guard:
/// `hooked()` already left out a source that was gone when the event fired.
fn gargaroth_trigger(g: &mut Game, src: PermId, p: PlayerId) -> Res {
    // the probe call only asks whether the trigger may go on the stack: no mode is chosen (a person is asked once)
    if g.trig_probe {
        return trigger_window(g, p, Some(src), "choose one", Some(5.0)).map(|_| ());
    }
    let mode = gargaroth_modes(g, p, src)?;
    let what = match mode {
        BEAST => "a 3/3 Beast",
        LIFE => "gain 3 life",
        _ => "draw a card",
    };
    if !trigger_window(g, p, Some(src), &format!("choose one: {what}"), Some(5.0))? {
        return Ok(());
    }
    crate::glog!(g, "  Elder Gargaroth: {what}");
    match mode {
        BEAST => {
            let spec = Tokens { color: Some(Colors::from_letters("G")), types: vec!["beast"], ..Tokens::new(1, 3) };
            make_tokens(g, p, spec)?;
        }
        LIFE => crate::engine::life::gain(g, p, 3)?,
        _ => draw(g, p, 1, false)?,
    }
    Ok(())
}

fn gargaroth_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) == p && atk.contains(&src) {
        gargaroth_trigger(g, src, p)?;
    }
    Ok(vec![])
}

fn gargaroth_blocks(
    g: &mut Game,
    src: Src,
    _p: PlayerId,
    _atk: &[PermId],
    d: PlayerId,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if owner(g, src) == d && assign.iter().any(|x| x.1 == src) {
        gargaroth_trigger(g, src, d)?;
    }
    Ok(())
}

// ================================================================== Sunblast Angel
// Flying. When this creature enters, destroy all tapped creatures (every player's, yours included).
/// The AI's cast rule. The ETB destroys the creatures that are tapped as it resolves: the opponents' attackers that
/// stayed tapped, and every creature of ours that is tapped by then, which includes the mana creatures that pay for
/// it. So: price the payment with the planner the engine pays with, count each of our creatures tapped now or tapped
/// by that payment as a loss (an indestructible one is not), count the opponents' tapped creatures as the gain, and
/// cast it when nothing of ours dies (a 4/5 flyer) or the gain clearly beats the loss. With only our own mana
/// creatures to be tapped it is never cast for them.
fn sunblast_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let (generic, pips) = cost_of(g, p, c);
    let units = mana_units(g, p, false);
    let Some((used, _)) = plan_pay(g, &units, generic, &pips) else { return 0 };
    // the creatures of ours the payment taps
    let paid: Vec<PermId> = units
        .iter()
        .zip(&used)
        .filter(|(_, n)| **n > 0)
        .filter_map(|(u, _)| match u.src {
            Source::Perm(m) if g.is_creature(m) => Some(m),
            _ => None,
        })
        .collect();
    let dies = |m: PermId| live(g, m) && g.is_creature(m) && !indestructible(g, m);
    let loss: f64 = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| dies(m) && (g.perm(m).tapped || paid.contains(&m)))
        .map(|m| pval(g, m))
        .psum();
    let gain: f64 = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| dies(m) && g.perm(m).tapped)
        .map(|m| pval(g, m))
        .psum();
    if loss <= 0.0 {
        // nothing of ours dies: a 6-mana 4/5 flyer, better with their tapped creatures to destroy
        let body = crate::ai::values::cast_prio(g, p, c);
        return if gain > 0.0 { body.max(55 + (3.0 * gain).min(25.0) as i32) } else { body };
    }
    if gain > 1.5 * loss { (45.0 + 2.0 * (gain - loss)).clamp(45.0, 80.0) as i32 } else { 0 }
}

// ================================================================== Aetherflux Reservoir
// Whenever you cast a spell, you gain 1 life for each spell you've cast this turn (the `aether` tag). Pay 50 life:
// this artifact deals 50 damage to any target. The AI shoots (JD, 2026-11-27) only to kill an opponent, and only if
// it is still alive after paying: its life must be 51 or more, the target's 50 or less.
fn aetherflux_target(g: &Game, p: PlayerId) -> Option<PlayerId> {
    if g.player(p).life <= 50 || g.player(p).life_locked {
        return None;
    }
    let kills: Vec<PlayerId> = g.opps(p).filter(|&q| shot_ok(g, p, q)).collect();
    kills.into_iter().max_by(|&a, &b| threat(g, p, a).total_cmp(&threat(g, p, b)))
}

/// q is a legal target the shot kills, and p can pay and live. The Reservoir is colourless, so only player hexproof
/// (Shalai, Leyline of Sanctity, Metropolis Reformer) and protection (the Ring) make a player an illegal target; damage
/// that is prevented (Glacial Chasm, a damage shield) or a player who can't lose (Platinum Angel, Herald of Eternal Dawn)
/// is no kill.
fn still_kills(g: &Game, p: PlayerId, q: PlayerId) -> bool {
    g.player(p).life > 50
        && !g.player(p).life_locked
        && crate::human::legal::player_targetable(g, q)
        && !crate::engine::values::player_hexproof(g, q)
        && g.player(q).life <= 50
        && !crate::engine::life::prevents_damage(g, q, Some(p))
        && !crate::impls::t0_giada::cant_lose(g, q)
}

/// the shot wins the game now: q is the last opponent alive and the shot kills it
fn wins_now(g: &Game, p: PlayerId, q: PlayerId) -> bool {
    still_kills(g, p, q) && !g.opps(p).any(|o| o != q && g.player(o).alive)
}

/// p controls an upkeep win (Felidar Sovereign 40, Test of Endurance 50) that its life reaches now and the shot's 50
/// would drop it below: p's next upkeep comes before any other chance to win, so the life is worth more held
fn holds_for_upkeep_win(g: &Game, p: PlayerId) -> bool {
    let life = g.player(p).life;
    UPKEEP_WINS.iter().any(|&(name, line)| {
        life >= line
            && life - 50 < line
            && g.player(p).perms.iter().any(|&m| live(g, m) && !g.perm(m).neutered && g.perm(m).name == name)
    })
}

/// the AI's rule for shooting q: it kills, and either it wins the game now or no upkeep win is thrown away
fn shot_ok(g: &Game, p: PlayerId, q: PlayerId) -> bool {
    still_kills(g, p, q) && (wins_now(g, p, q) || !holds_for_upkeep_win(g, p))
}

fn aetherflux_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || !live(g, src) {
        return Ok(vec![]);
    }
    let Some(q) = aetherflux_target(g, p) else { return Ok(vec![]) };
    // the shot wins the game when no other opponent is left
    let wins = !g.opps(p).any(|o| o != q && g.player(o).alive);
    let label = format!("{RESERVOIR}: pay 50 life, 50 damage to {}", g.player(q).name);
    Ok(vec![ability(if wins { 12.0 } else { 9.5 }, label, src, aetherflux_go, q.0 as i64)])
}

fn aetherflux_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let q = PlayerId(arg as u8);
    if !live(g, src) || owner(g, src) != p || !shot_ok(g, p, q) {
        return Ok(false);
    }
    lose_life(g, p, 50, Some(p), "other", None)?; // the cost
    crate::glog!(g, "  {} pays 50 life: Aetherflux Reservoir", g.player(p).name);
    crate::engine::stack::ability_run_w(g, p, Some(src), None, "50 damage", Some(9.0), None, 0.0, |g, _copy| {
        let pl = g.player_mut(p);
        pl.stat(crate::sym::intern("aether_shots"), 1);
        let turns = pl.turns;
        pl.milestone.entry(crate::sym::intern("aether")).or_insert(turns);
        crate::human::choices::apply_damage_by(
            g,
            p,
            crate::human::Target::Player(q),
            50,
            "Aetherflux Reservoir",
            "aether",
            None,
        )
        .map(|_| ())
    })?;
    Ok(true)
}

// ================================================================== Castle Ardenvale
// Enters tapped unless you control a Plains. {T}: add {W}. {2}{W}{W},{T}: create a 1/1 white Human creature token.
// The compiled ability's utility is negative (a flat cost per mana symbol), so the card has its own: the AI makes a
// Human at the end of an opponent's turn with the {2}{W}{W} it left open (the token feeds Trostani's populate), and
// the more with a payoff for a creature entering out. The land taps itself; the mana comes from other sources.
fn ardenvale_options(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some()
        || g.active == Some(p) // (an opponent's end step only, as Gavony Township's)
        || !live(g, l)
        || g.perm(l).tapped
        || !g.player(p).lands.contains(&l)
        || !crate::impls::lands::can_pay_without(g, p, l, 2, "WW")
    {
        return Ok(vec![]);
    }
    let payoffs = ENTER_PAYOFFS.iter().filter(|n| controls(g, p, n)).count() as f64;
    Ok(vec![Opt {
        utility: 2.0 + payoffs,
        label: format!("{ARDENVALE}: a 1/1 Human"),
        act: Some(Action::Plan { f: ardenvale_go, arg: l.0 as i64 }),
    }])
}

fn ardenvale_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = PermId(arg as u32);
    if !live(g, l) || g.perm(l).tapped || !g.player(p).lands.contains(&l) {
        return Ok(false);
    }
    if !crate::impls::lands::pay_without(g, p, l, 2, "WW")? {
        return Ok(false);
    }
    g.perm_mut(l).tapped = true;
    let cd = g.perm(l).card();
    crate::engine::stack::ability_run_w(g, p, None, Some(cd), "a 1/1 Human", None, None, 0.0, |g, _copy| {
        let spec = Tokens { color: Some(Colors::from_letters("W")), types: vec!["human"], ..Tokens::new(1, 1) };
        make_tokens(g, p, spec).map(|_| ())
    })?;
    Ok(true)
}

// ================================================================== Felidar Retreat
// Landfall: whenever a land you control enters, choose one: a 2/2 white Cat Beast creature token; or a +1/+1 counter on
// each creature you control, and those creatures gain vigilance until end of turn. The landfall trigger itself is
// `zones::landfall_once`; this is the AI's cast rule and its mode. The card is tagged only `landfall2`, which the derived
// worth reads as nothing, so its priority was 0 and the AI never cast it ("cast 0.0% of the games it was drawn").

/// permanents that make the Cat worth more than its 2/2 body: Trostani and the Avatar gain life equal to its toughness
const CAT_PAYOFFS: [&str; 2] = ["Trostani, Selesnya's Voice", "Verdant Sun's Avatar"];

/// permanents that make Retreat's trigger worth more: the doublers of its tokens and counters, and the counter payoffs
const RETREAT_ENGINES: [&str; 6] = [
    "Trostani, Selesnya's Voice",
    "Doubling Season",
    "Anointed Procession",
    "Cathars' Crusade",
    "Wildwood Scourge",
    "Kalonian Hydra",
];

fn creatures_out(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| live(g, m) && g.is_creature(m)).count()
}

/// The AI's cast priority. Every land that enters later is a Cat or a counter on each creature, so the worth is the land
/// drops to come: a land in hand each (more of them, more worth), an extra land drop (Exploration, Azusa), else the
/// chance the library gives one (the share of lands in it); with none to expect it is not cast. More with creatures
/// out for the counter mode, and with Trostani, a doubler or Crusade out. Capped below the plan's multipliers (60) and
/// Trostani herself (90), so a turn's mana goes to them first.
fn retreat_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let pl = g.player(p);
    let in_hand = pl.hand.iter().filter(|&&x| x != c && g.db.get(x).land).count() as i32;
    let extra = crate::cardcode::extra_lands(g, p) as i32;
    let lib_lands = pl.library.iter().filter(|&&x| g.db.get(x).land).count();
    let base = if in_hand > 0 {
        44 + 5 * in_hand.min(3)
    } else if extra > 0 && lib_lands > 0 {
        40
    } else if lib_lands * 5 >= pl.library.len() && lib_lands > 0 {
        30 // a land drop is likely next turn (a fifth of the library or more)
    } else {
        return 0; // no land to come: nothing for it to trigger on
    };
    let body = 2 * creatures_out(g, p).min(4) as i32;
    let engines = 4 * RETREAT_ENGINES.iter().filter(|n| controls(g, p, n)).count() as i32;
    (base + body + engines).min(58)
}

/// true: the counters mode; false: the Cat. Each mode in stat points, counters and tokens as the doublers would make
/// them: the Cat is a 2/2 (4 points) plus one for being a body (blink, populate, Craterhoof), more for a life payoff;
/// the counters are one point per creature, doubled by Doubling Season and the like, plus the creatures they grow
/// (Wildwood Scourge, Kalonian Hydra). With Cathars' Crusade out the Cat is the better mode: it enters, so the Crusade puts a
/// counter on every creature including the Cat itself. A tie is the Cat.
pub fn retreat_counters(g: &Game, p: PlayerId) -> bool {
    if controls(g, p, "Cathars' Crusade") {
        return false;
    }
    let creatures = creatures_out(g, p) as f64;
    if creatures == 0.0 {
        return false;
    }
    let mult = repl::counter_mult(g, p).max(1) as f64;
    // what the counters grow: each Wildwood Scourge gets a counter for each other non-Hydra creature that got one, and a
    // Kalonian Hydra's attack doubles the counters on it (about one more)
    let non_hydra = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| live(g, m) && g.is_creature(m) && !crate::engine::values::has_type(g, m, "hydra"))
        .count() as f64;
    let scourges =
        g.player(p).perms.iter().filter(|&&m| live(g, m) && g.perm(m).name == "Wildwood Scourge").count() as f64;
    let hydras = g.player(p).perms.iter().filter(|&&m| live(g, m) && g.perm(m).name == "Kalonian Hydra").count() as f64;
    let grown = scourges * (non_hydra - 1.0).max(0.0) + hydras;
    let counters = mult * (creatures + grown);
    let tokens = repl::tokens(g, p, 1) as f64;
    let payoff = CAT_PAYOFFS.iter().filter(|n| controls(g, p, n)).count() as f64;
    let cat = tokens * (2.0 + 0.5 * payoff) + 1.0;
    counters > cat
}

/// the Cat Beast token(s) of the first mode: a 2/2 white Cat Beast creature token (the doublers make more)
pub fn retreat_cat(g: &mut Game, p: PlayerId) -> Res {
    let spec = Tokens { color: Some(Colors::from_letters("W")), types: vec!["cat", "beast"], ..Tokens::new(1, 2) };
    make_tokens(g, p, spec).map(|_| ())
}

/// the second mode: a +1/+1 counter on each creature p controls (one placement for each, so every counter effect
/// applies), and those creatures gain vigilance until end of turn
pub fn retreat_counter_mode(g: &mut Game, p: PlayerId) -> Res {
    repl::resolving_with(g, |g| {
        for m in g.player(p).perms.clone() {
            if g.is_creature(m) {
                repl::put_plus(g, m, 1);
                let x = g.perm_mut(m);
                if !x.eot_kw.contains(&"vigilance") {
                    x.eot_kw.push("vigilance");
                }
            }
        }
        Ok(())
    })
}

/// the AI's choice of mode at one landfall
pub fn retreat_ai(g: &mut Game, p: PlayerId) -> Res {
    if retreat_counters(g, p) { retreat_counter_mode(g, p) } else { retreat_cat(g, p) }
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, GARGAROTH)?;
    c.attack = Some(gargaroth_attack);
    c.blocks = Some(gargaroth_blocks);
    r.card(db, SUNBLAST)?.prio = Some(sunblast_prio);
    r.card(db, RETREAT)?.prio = Some(retreat_prio);
    r.card(db, RESERVOIR)?.options = Some(aetherflux_options);
    r.card(db, ARDENVALE)?.land_options = Some(ardenvale_options);
    Ok(())
}
