//! Tier 3 deck `marwyn-mono-green-elves` (EDHREC's bracket-3 average list): the cards the ability language can't run
//! (rust/agents/t3-model-marwyn.md). The general patterns (Emerald Medallion's colour discount, Joraga Warcaller's
//! per-counter lord bonus, Dwynen's attacking-Elf count, Immaculate Magistrate's per-Elf counters) are compiled in
//! cardgen's dsl_parse.rs and run in dsl/mod.rs, under `// --- Tier 3 (marwyn)`.
//!
//! - Elvish Guidance: a mana source (`rock=1:G`, as Wild Growth) making {G} for each Elf on the battlefield.
//! - Genesis Wave: X = all spare mana (`xtutor`); every permanent card with mana value X or less among the top X
//!   enters, the rest go to the graveyard.
//! - Dionus, Elvish Archdruid: an Elf of its controller's that becomes tapped on their turn untaps and gets a +1/+1
//!   counter, once a turn per Elf (`on_tapped`, called where creatures tap: attacks, mana, tap costs).
//! - Eladamri, Korvecdal: creature spells cast from the top of the library; its {G},{T}, tap two creatures ability puts
//!   the best creature card from the hand (or the top) onto the battlefield.
//! - Realmwalker: names a creature type as it enters; creature spells of that type cast from the top of the library.
//! - Galadhrim Ambush: cast when attacked: X Elf Warriors (X attackers), non-Elf creatures deal no combat damage.
//! - Galadhrim Brigade: squad {1}{G} as many times as the spare mana allows, a token copy for each.
//! - Joraga Warcaller: multikicker {1}{G} as many times as the spare mana allows, a +1/+1 counter for each.
//! - Legolas's Quick Reflexes: its "whenever this creature becomes tapped" damage (`on_tapped`); the AI casts it on a
//!   tapped mana creature that can kill an opposing creature (then taps it again for mana, which floats), and as a
//!   protection spell (hexproof) through the pool's protectors.
//! - Three Tree City: names a creature type; taps for {C}, or that many mana of one colour minus its {2} activation.
//! - Wirewood Lodge: {T}: {C}, or with an Elf mana creature already tapped, {G},{T} untaps it: it taps for that Elf's
//!   mana less the {G}.
//! - Yavimaya, Cradle of Growth: every land is a Forest (forestwalk, Nissa's Forests).
//! - Yeva, Nature's Herald: green creature spells as though they had flash.
//! - Umbral Mantle: the combo with a mana creature making four or more (Marwyn grows by itself) and Ezuri: infinite
//!   mana and infinite overruns win before combat. Its plain +2/+2 pump outside the loop is not used.

use crate::ai::brain::{Situation, card_utility, do_cast_with};
use crate::cards::{CardDb, Colors};
use crate::engine::cast::{cast_card, castable};
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::stack::ability_window;
use crate::engine::tutors::card_worth;
use crate::engine::values::{epow, etgh, has_type, once_per_turn, pval, untargetable};
use crate::engine::zones::{Enter, Tokens, enter, enter_token_copy, landfall, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::impls::combos::{Combo, Ready};
use crate::state::{Ctx, DataKey, Game, Step, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

const GUIDANCE: &str = "Elvish Guidance";
const WAVE: &str = "Genesis Wave";
const DIONUS: &str = "Dionus, Elvish Archdruid";
const ELADAMRI: &str = "Eladamri, Korvecdal";
const AMBUSH: &str = "Galadhrim Ambush";
const BRIGADE: &str = "Galadhrim Brigade";
const JORAGA: &str = "Joraga Warcaller";
const LEGOLAS: &str = "Legolas's Quick Reflexes";
const REALMWALKER: &str = "Realmwalker";
const THREE_TREE: &str = "Three Tree City";
const MANTLE: &str = "Umbral Mantle";
const LODGE: &str = "Wirewood Lodge";
const YAVIMAYA: &str = "Yavimaya, Cradle of Growth";
const YEVA: &str = "Yeva, Nature's Herald";
const EZURI: &str = "Ezuri, Renegade Leader";
const MARWYN: &str = "Marwyn, the Nurturer";

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn named(g: &Game, m: PermId, name: &str) -> bool {
    g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == name)
}

/// p's live permanent of this name
fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && named(g, m, name))
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == name)
}

fn remove_card(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

/// p's turn, in a main phase before combat
fn precombat(g: &Game, p: PlayerId) -> bool {
    g.active == Some(p) && matches!(g.step, Step::Start | Step::Main1)
}

/// the mana a creature's tap ability makes now (0 if it has none)
fn dork_mana(g: &Game, p: PlayerId, m: PermId) -> u32 {
    let Some(c) = g.perm(m).cd else { return 0 };
    if !g.db.get(c).tag(Tag::Dork) {
        return 0;
    }
    crate::cardcode::dyn_mana_perm(g, p, m).unwrap_or(1)
}

fn elves_on_bf(g: &Game) -> u32 {
    g.players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter())
        .filter(|&&m| !g.perm(m).phased && g.is_creature(m) && has_type(g, m, "elf"))
        .count() as u32
}

// ================================================================== tapping: Dionus, Legolas's Quick Reflexes
/// Called where a creature becomes tapped (the attack declaration, mana abilities, the ability language's {T} costs):
/// Dionus untaps an Elf of its controller's during that player's turn and puts a +1/+1 counter on it (once each turn
/// for each Elf); a creature under Legolas's Quick Reflexes deals damage equal to its power to an opposing creature.
pub fn on_tapped(g: &mut Game, m: PermId) -> Res {
    if !g.perm(m).on_bf || !g.perm(m).tapped {
        return Ok(());
    }
    if g.perm(m).eot_kw.contains(&"legolas_tap") {
        legolas_tapped(g, m)?;
    }
    let o = g.perm(m).owner;
    if g.active != Some(o) || !g.is_creature(m) || on_bf(g, o, DIONUS).is_none() || !has_type(g, m, "elf") {
        return Ok(());
    }
    if once_per_turn(g, o, intern(&format!("dionus_{}", m.0))) {
        g.perm_mut(m).tapped = false;
        crate::cardcode::put_plus(g, m, 1);
        crate::glog!(g, "    Dionus untaps {} and puts a +1/+1 counter on it", g.perm(m).name);
    }
    Ok(())
}

// ================================================================== Legolas's Quick Reflexes
/// Until end of turn the creature has "whenever this creature becomes tapped, it deals damage equal to its power to up
/// to one target creature": `t6_legends2::legolas_effect` marks it (an end-of-turn keyword); the AI aims it at the
/// most valuable opposing creature the damage kills, and at nothing when none dies.
pub fn legolas_mark(g: &mut Game, t: PermId) {
    crate::impls::partials::eot_kw(g, t, "legolas_tap");
}

/// the most valuable opposing creature p's n damage kills
fn kill_target(g: &Game, p: PlayerId, n: i32) -> Option<PermId> {
    let ts: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&t| live(g, t) && g.is_creature(t) && !untargetable(g, t) && etgh(g, t) <= n)
        .filter(|&t| !crate::engine::values::no_damage(g, t))
        .collect();
    max_by(&ts, |t| pval(g, t))
}

fn legolas_tapped(g: &mut Game, m: PermId) -> Res {
    let o = g.perm(m).owner;
    let n = epow(g, m);
    if n <= 0 {
        return Ok(());
    }
    let Some(t) = kill_target(g, o, n) else { return Ok(()) };
    crate::glog!(
        g,
        "    {} becomes tapped and deals {n} damage to {} (Legolas's Quick Reflexes)",
        g.perm(m).name,
        g.perm(t).name
    );
    crate::human::choices::apply_damage(g, o, crate::human::Target::Perm(t), n, LEGOLAS, "burn")
}

/// the tapped mana creature of p's worth untapping with Legolas: it makes two or more mana and its power kills an
/// opposing creature worth 3 or more
fn legolas_dork(g: &Game, p: PlayerId) -> Option<PermId> {
    let ms: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.is_creature(m) && g.perm(m).tapped && !g.perm(m).sick)
        .filter(|&m| dork_mana(g, p, m) >= 2 && kill_target(g, p, epow(g, m)).is_some_and(|t| pval(g, t) >= 3.0))
        .collect();
    max_by(&ms, |m| kill_target(g, p, epow(g, m)).map_or(0.0, |t| pval(g, t)) + dork_mana(g, p, m) as f64 * 0.3)
}

/// in p's own main phase: untap a tapped mana creature that then taps again for mana (it floats) and kills
fn legolas_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "G", false) {
        return Ok(vec![]);
    }
    let Some(m) = legolas_dork(g, p) else { return Ok(vec![]) };
    let v = kill_target(g, p, epow(g, m)).map_or(0.0, |t| pval(g, t));
    Ok(vec![plan(
        1.5 + 0.3 * v.min(20.0),
        format!("Legolas's Quick Reflexes on {}", g.perm(m).name),
        legolas_go,
        c.0 as i64,
    )])
}

fn legolas_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let Some(m) = legolas_dork(g, p) else { return Ok(false) };
    if !crate::impls::t6_legends2::legolas_cast(g, p, c, m)? {
        return Ok(false);
    }
    if live(g, m) && !g.perm(m).tapped && g.perm(m).eot_kw.contains(&"legolas_tap") {
        // tap it for mana at once: the mana floats, the damage trigger kills
        let n = dork_mana(g, p, m);
        g.perm_mut(m).tapped = true;
        g.player_mut(p).floating.g += n;
        crate::glog!(g, "    {} taps for {n} mana (it floats)", g.perm(m).name);
        on_tapped(g, m)?;
    }
    Ok(true)
}

// ================================================================== Elvish Guidance
/// {G} for each Elf on the battlefield (every player's), as its own mana source (`rock=1:G`)
fn guidance_mana(g: &Game, _p: PlayerId, _m: PermId) -> u32 {
    elves_on_bf(g)
}

// ================================================================== Genesis Wave
/// cast with X = spare mana of 5 or more (`xtutor` pays it)
fn wave_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let x = total_mana(g, p, false) as i32 - 3;
    if x >= 5 && g.player(p).library.len() as i32 > x + 5 { 50 + x.min(20) } else { 0 }
}

/// Reveal the top X; every permanent card with mana value X or less enters (lands as lands, with landfall), the rest
/// go to the graveyard
fn wave_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x.max(0) as usize;
    let n = x.min(g.player(p).library.len());
    let top: Vec<CardId> = (0..n).filter_map(|_| g.player_mut(p).library.pop()).collect();
    crate::glog!(g, "    {} reveals the top {n} cards (Genesis Wave, X = {x})", pname(g, p));
    let (put, rest): (Vec<CardId>, Vec<CardId>) =
        top.into_iter().partition(|&c| g.db.get(c).perm && g.db.get(c).cmc as usize <= x);
    g.player_mut(p).gy.extend(rest);
    let mut lands = false;
    for c in put {
        if g.over {
            break;
        }
        crate::glog!(g, "    {} enters (Genesis Wave)", g.db.get(c).name);
        if g.db.get(c).land {
            let tapped = crate::engine::turn::land_enters_tapped(g, p, c);
            g.add_land(p, c, tapped);
            lands = true;
        } else {
            enter(g, p, c, Enter::default())?;
        }
    }
    if lands && !g.over {
        landfall(g, p)?;
    }
    Ok(intern("gy"))
}

// ================================================================== casting from the top: Eladamri, Realmwalker
/// the creature type a permanent named as it entered
fn ctype(g: &Game, m: PermId) -> Option<Sym> {
    match g.perm(m).data.get(DataKey::Ctype) {
        Some(Val::Str(s)) => Some(*s),
        _ => None,
    }
}

fn realmwalker_as(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    if !named(g, m, REALMWALKER) {
        return Ok(());
    }
    let t = crate::impls::galadriel::choose_type(g, p, REALMWALKER)?;
    g.perm_mut(m).data.set(DataKey::Ctype, Val::Str(t));
    crate::glog!(g, "    Realmwalker names {t}");
    Ok(())
}

/// does src let p cast the top card c (a creature card; Realmwalker: of the type it named)?
fn top_ok(g: &Game, src: PermId, c: CardId) -> bool {
    let d = g.db.get(c);
    if !d.creature || d.land {
        return false;
    }
    if named(g, src, REALMWALKER) {
        let Some(t) = ctype(g, src) else { return false };
        return d.has_subtype(t) || d.has_kw("changeling");
    }
    true
}

/// the first of p's permanents that lets it cast the top card (one option set per player)
fn top_source(g: &Game, p: PlayerId) -> Option<PermId> {
    let &top = g.player(p).library.last()?;
    g.player(p)
        .perms
        .iter()
        .copied()
        .find(|&m| {
            // --- Tiers 1/2 (lathril): Elven Chorus casts creature spells from the top too
            live(g, m)
                && (named(g, m, ELADAMRI) || named(g, m, REALMWALKER) || named(g, m, "Elven Chorus"))
                && top_ok(g, m, top)
        })
}

/// You may cast creature spells from the top of your library: the AI casts the top card when it would cast it from the
/// hand (`card_utility`, a little more for the free card)
pub(crate) fn top_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if g.perm(src).owner != p || !live(g, src) {
        return Ok(out);
    }
    if top_source(g, p) == Some(src)
        && let Some(&top) = g.player(p).library.last()
        && (post.is_some() || crate::engine::cast::grants_flash(g, p, top))
    {
        let s = Situation::new(g, p);
        if let Some(u) = card_utility(g, p, &s, top) {
            let (cg, cp) = cost_of(g, p, top);
            g.pay_for = Some(top);
            let ok = can_pay(g, p, cg, &cp, false) && (!g.cast_checks() || castable(g, p, top, "lib"));
            g.pay_for = None;
            if ok {
                out.push(plan(
                    u + 0.6,
                    format!("cast {} from the top of the library", g.db.get(top).name),
                    top_cast,
                    top.0 as i64,
                ));
            }
        }
    }
    if named(g, src, ELADAMRI) {
        out.extend(eladamri_options(g, src, p, post));
    }
    Ok(out)
}

fn top_cast(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let top = CardId(arg as u16);
    if g.player(p).library.last() != Some(&top) || top_source(g, p).is_none() {
        return Ok(false);
    }
    g.player_mut(p).library.pop();
    crate::glog!(g, "  {} casts {} from the top of the library", pname(g, p), g.db.get(top).name);
    let ok = do_cast_with(g, p, top, Some("lib"), Ctx::default())?;
    if !ok {
        g.player_mut(p).library.push(top);
    }
    Ok(ok)
}

/// Eladamri's two creatures to tap: the least valuable untapped ones that make no mana (a mana creature is tapped
/// last, Dionus untaps Elves anyway)
fn eladamri_tappers(g: &Game, p: PlayerId, src: PermId) -> Option<(PermId, PermId)> {
    let mut cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != src && live(g, m) && g.is_creature(m) && !g.perm(m).tapped)
        .collect();
    cs.sort_by(|&a, &b| {
        let k = |m: PermId| pval(g, m) + 3.0 * dork_mana(g, p, m) as f64;
        k(a).total_cmp(&k(b))
    });
    (cs.len() >= 2).then(|| (cs[0], cs[1]))
}

/// the creature card Eladamri puts onto the battlefield: the best in the hand by mana value and worth, or the top card
/// (Some(true): from the top)
fn eladamri_pick(g: &Game, p: PlayerId) -> Option<(CardId, bool)> {
    let hand: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    let key = |c: CardId| g.db.get(c).cmc as f64 + 0.1 * card_worth(g, p, c, false);
    let best = max_by(&hand, key);
    let top = g.player(p).library.last().copied().filter(|&c| g.db.get(c).creature);
    match (best, top) {
        (Some(b), Some(t)) if key(t) > key(b) => Some((t, true)),
        (Some(b), _) => Some((b, false)),
        (None, Some(t)) => Some((t, true)),
        _ => None,
    }
}

/// {G}, {T}, tap two untapped creatures you control: reveal a card from your hand or the top of your library; a
/// creature card enters. Only during your turn: the AI uses it in its main phase for a creature of mana value 3+.
fn eladamri_options(g: &Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let x = g.perm(src);
    if post.is_none() || g.active != Some(p) || x.tapped || x.sick || !can_pay(g, p, 0, "G", false) {
        return vec![];
    }
    if eladamri_tappers(g, p, src).is_none() {
        return vec![];
    }
    let Some((c, _)) = eladamri_pick(g, p) else { return vec![] };
    let mv = g.db.get(c).cmc;
    if mv < 3 {
        return vec![];
    }
    vec![ability(
        1.0 + 0.5 * mv as f64,
        format!("Eladamri puts {} onto the battlefield", g.db.get(c).name),
        src,
        eladamri_go,
        0,
    )]
}

fn eladamri_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some((a, b)) = eladamri_tappers(g, p, src) else { return Ok(false) };
    if g.perm(src).tapped || !can_pay(g, p, 0, "G", false) || eladamri_pick(g, p).is_none() {
        return Ok(false);
    }
    pay(g, p, 0, "G", false)?;
    for m in [src, a, b] {
        g.perm_mut(m).tapped = true;
        on_tapped(g, m)?;
    }
    if !ability_window(g, p, Some(src), "put a creature card onto the battlefield", Some(4.0), None)? {
        return Ok(true);
    }
    let Some((c, top)) = eladamri_pick(g, p) else { return Ok(true) };
    if top {
        g.player_mut(p).library.pop();
    } else {
        remove_card(&mut g.player_mut(p).hand, c);
    }
    crate::glog!(g, "    {} puts {} onto the battlefield (Eladamri)", pname(g, p), g.db.get(c).name);
    enter(g, p, c, Enter::default())?;
    Ok(true)
}

// ================================================================== Galadhrim Ambush
/// combat damage by a non-Elf creature is prevented this turn (`cardcode::damage_prevented`)
pub fn elf_fog(g: &Game, m: PermId) -> bool {
    let st = g.turn_stamp();
    g.players.iter().any(|q| q.flag_turn.get(&"elf_fog") == Some(&st)) && !has_type(g, m, "elf")
}

/// Create X 1/1 green Elf Warrior tokens (X: the attacking creatures, given as it was cast); prevent all combat damage
/// non-Elf creatures would deal this turn
fn ambush_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = if ctx.x > 0 { ctx.x } else { g.in_combat.iter().filter(|&&m| live(g, m)).count() as i32 };
    if x > 0 {
        let spec = Tokens {
            color: Some(Colors::from_letters("G")),
            types: vec!["elf", "warrior"],
            warrior: true,
            ..Tokens::new(x as u32, 1)
        };
        make_tokens(g, p, spec)?;
    }
    let st = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(intern("elf_fog"), st);
    crate::glog!(g, "    non-Elf creatures deal no combat damage this turn (Galadhrim Ambush)");
    Ok(intern("gy"))
}

/// the AI at its declare-attackers priority (`ai::brain::attack_response`): Galadhrim Ambush when the non-Elf attackers
/// would take a fifth of its life, or three or more attack it
pub fn ambush_response(g: &mut Game, d: PlayerId, p: PlayerId, atk: &[PermId]) -> Res<bool> {
    let Some(c) = in_hand(g, d, AMBUSH) else { return Ok(false) };
    let st = g.turn_stamp();
    if g.fog == Some(st) || g.players.iter().any(|q| q.flag_turn.get(&"elf_fog") == Some(&st)) {
        return Ok(false);
    }
    let mine: Vec<PermId> = atk.iter().copied().filter(|&m| live(g, m) && g.perm(m).owner == p).collect();
    let incoming: i32 = mine.iter().filter(|&&m| !has_type(g, m, "elf")).map(|&m| epow(g, m).max(0)).sum();
    let life = g.player(d).life;
    if incoming * 5 < life && mine.len() < 3 && incoming < life {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, d, c);
    if !can_pay(g, d, gn, &pips, false) || !castable(g, d, c, "hand") || !pay(g, d, gn, &pips, false)? {
        return Ok(false);
    }
    crate::glog!(g, "  {} answers the attack with Galadhrim Ambush ({} attackers)", pname(g, d), atk.len());
    cast_card(g, d, c, "hand", Ctx { x: atk.len() as i32, ..Ctx::default() })?;
    Ok(true)
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

// ================================================================== squad and multikicker: Brigade, Warcaller
/// what casting c with k extra payments of {1}{G} costs p: its cost now, plus {1}{G} each (a discount the spell's own
/// cost had no generic mana to take goes to the kicker: Emerald Medallion on Joraga Warcaller)
fn kicked_cost(g: &Game, p: PlayerId, c: CardId, k: u32) -> (u32, String) {
    let (cg, cp) = cost_of(g, p, c);
    let unused = if cg == 0 { crate::dsl::cost_delta(g, p, c).min(0) } else { 0 };
    let gn = (cg as i32 + k as i32 + unused).max(0) as u32;
    (gn, format!("{cp}{}", "G".repeat(k as usize)))
}

/// the most extra payments p can make now (up to 6)
fn max_kicks(g: &mut Game, p: PlayerId, c: CardId) -> u32 {
    g.pay_for = Some(c);
    let mut k = 0;
    while k < 6 {
        let (gn, pips) = kicked_cost(g, p, c, k + 1);
        if !can_pay(g, p, gn, &pips, false) {
            break;
        }
        k += 1;
    }
    g.pay_for = None;
    k
}

/// the cast with one kick or more, worth a little more than the plain cast for each
fn kick_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() && !crate::engine::cast::grants_flash(g, p, c) {
        return Ok(vec![]);
    }
    if !g.player(p).hand.contains(&c) || (g.cast_checks() && !castable(g, p, c, "hand")) {
        return Ok(vec![]);
    }
    let k = max_kicks(g, p, c);
    if k == 0 {
        return Ok(vec![]);
    }
    let s = Situation::new(g, p);
    let Some(u) = card_utility(g, p, &s, c) else { return Ok(vec![]) };
    let name = g.db.get(c).name.to_string();
    Ok(vec![plan(u + 0.8 * k as f64, format!("{name} with {k} extra payment(s)"), kick_go, c.0 as i64)])
}

fn kick_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    let k = max_kicks(g, p, c);
    let (gn, pips) = kicked_cost(g, p, c, k);
    g.pay_for = Some(c);
    let ok = can_pay(g, p, gn, &pips, false) && pay(g, p, gn, &pips, false)?;
    g.pay_for = None;
    if !ok {
        return Ok(false);
    }
    let before: Vec<PermId> = g.player(p).perms.clone();
    if !cast_card(g, p, c, "hand", Ctx { kicks: k, ..Ctx::default() })? {
        return Ok(true);
    }
    let Some(m) = g.player(p).perms.iter().copied().find(|&m| !before.contains(&m) && g.perm(m).cd == Some(c)) else {
        return Ok(true);
    };
    if named(g, m, JORAGA) {
        // enters with a +1/+1 counter for each time it was kicked
        crate::cardcode::put_plus(g, m, k as i32);
        crate::glog!(g, "    Joraga Warcaller enters with {k} +1/+1 counter(s)");
    } else {
        // squad: when it enters, a token copy for each time the cost was paid
        for _ in 0..k {
            enter_token_copy(g, p, c)?;
        }
        crate::glog!(g, "    {} makes {k} token copies (squad)", g.db.get(c).name);
    }
    Ok(true)
}

// ================================================================== lands: Three Tree City, Wirewood Lodge, Yavimaya
/// the creature type Three Tree City named (as it entered as a land drop; else the most common type among p's
/// creatures)
fn city_type(g: &Game, p: PlayerId, l: LandId) -> Option<Sym> {
    if let Some(Val::Str(s)) = g.land(l).data.get(DataKey::Ctype) {
        return Some(*s);
    }
    let mut counts: Vec<(Sym, u32)> = vec![];
    for &m in &g.player(p).perms {
        if !g.is_creature(m) || g.perm(m).phased {
            continue;
        }
        for t in crate::engine::values::subtypes(g, m) {
            match counts.iter_mut().find(|x| x.0 == t) {
                Some(x) => x.1 += 1,
                None => counts.push((t, 1)),
            }
        }
    }
    counts.iter().max_by_key(|x| x.1).map(|x| x.0)
}

/// {2},{T}: mana of one colour equal to the creatures of the named type you control: net of the {2} (as Nykthos);
/// {C} when that is no more
fn city_amount(g: &Game, p: PlayerId, l: LandId) -> u32 {
    let Some(t) = city_type(g, p, l) else { return 1 };
    let n = g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased && has_type(g, m, t)).count();
    (n as u32).saturating_sub(2).max(1)
}

fn city_cols(g: &Game, p: PlayerId, l: LandId) -> Colors {
    if city_amount(g, p, l) > 1 { g.player(p).ident } else { Colors::NONE }
}

fn city_etb(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    let t = crate::impls::galadriel::choose_type(g, p, THREE_TREE)?;
    g.land_mut(l).data.set(DataKey::Ctype, Val::Str(t));
    crate::glog!(g, "    Three Tree City names {t}");
    Ok(())
}

/// {G},{T}: untap target Elf: with an Elf mana creature already tapped that makes 3 or more, the Lodge taps for that
/// Elf's mana less the {G} (the Elf taps again); else {C}
fn lodge_amount(g: &Game, p: PlayerId, _l: LandId) -> u32 {
    let best = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && g.perm(m).tapped && !g.perm(m).sick && has_type(g, m, "elf"))
        .map(|m| dork_mana(g, p, m))
        .max()
        .unwrap_or(0);
    if best >= 3 { best - 1 } else { 1 }
}

fn lodge_cols(g: &Game, p: PlayerId, l: LandId) -> Colors {
    if lodge_amount(g, p, l) > 1 { Colors::from_letters("G") } else { Colors::NONE }
}

/// Yavimaya, Cradle of Growth on any battlefield (and not a plain Mountain under Blood Moon): every land is a Forest
pub fn yavimaya_out(g: &Game) -> bool {
    g.players.iter().filter(|q| q.alive).any(|q| {
        q.lands.iter().any(|&l| &*g.db.get(g.land(l).cd).name == YAVIMAYA && !crate::engine::mana::moon_land(g, l))
    })
}

/// is land l a Forest (by name, by type, or by Yavimaya)?
pub fn is_forest(g: &Game, l: LandId) -> bool {
    let d = g.db.get(g.land(l).cd);
    &*d.name == "Forest" || d.has_subtype("forest") || yavimaya_out(g)
}

// ================================================================== Yeva, Nature's Herald
/// You may cast green creature spells as though they had flash (`cast::grants_flash`)
pub fn yeva_flash(g: &Game, p: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    d.creature && d.colors.has('G') && on_bf(g, p, YEVA).is_some()
}

// ================================================================== Umbral Mantle: the combo
/// a creature of p's that loops with Umbral Mantle ({3},{Q}: +2/+2): it taps for 4 or more (Priest of Titania,
/// Elvish Archdruid, Circle of Dreams Druid), or it is Marwyn (it taps for its power, which grows by 2 each loop)
/// with power 3, or with the mana for the first loops. A tapped one needs {3} to start.
fn loops(g: &Game, p: PlayerId, m: PermId) -> bool {
    let x = g.perm(m);
    if !live(g, m) || !g.is_creature(m) || (x.sick && !crate::dsl::has_kw(g, m, "haste")) {
        return false;
    }
    if x.tapped && !can_pay(g, p, 3, "", false) {
        return false;
    }
    if named(g, m, MARWYN) {
        return epow(g, m) >= 3 || (epow(g, m) >= 1 && total_mana(g, p, false) >= 4);
    }
    dork_mana(g, p, m) >= 4
}

/// before combat on p's turn: Umbral Mantle, a creature that loops with it, Ezuri (or Ezuri castable from the hand) and
/// an Elf that can attack. Infinite mana, infinite overruns ({G}{G}{G}{2}: Elves get +3/+3 and trample): p wins.
fn mantle_ready(g: &Game, p: PlayerId) -> Ready {
    let no = || (false, vec![], vec![]);
    if !precombat(g, p) {
        return no();
    }
    let Some(mantle) = on_bf(g, p, MANTLE) else { return no() };
    let cs: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| loops(g, p, m)).collect();
    let Some(dork) = max_by(&cs, |m| dork_mana(g, p, m) as f64) else { return no() };
    let attacker = g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        live(g, m)
            && g.is_creature(m)
            && has_type(g, m, "elf")
            && !x.noatk
            && (!x.sick || crate::dsl::has_kw(g, m, "haste"))
            && (m == dork || !x.tapped)
    });
    if !attacker {
        return no();
    }
    if let Some(e) = on_bf(g, p, EZURI) {
        return (true, vec![mantle, dork, e], vec![]);
    }
    if let Some(c) = in_hand(g, p, EZURI) {
        let (gn, pips) = cost_of(g, p, c);
        if can_pay(g, p, gn, &pips, false) && castable(g, p, c, "hand") {
            return (true, vec![mantle, dork], vec![c]);
        }
    }
    no()
}

pub static COMBOS: [Combo; 1] = [Combo {
    name: "Umbral Mantle + a big mana Elf + Ezuri",
    groups: &[&[MANTLE], &[MARWYN, "Priest of Titania", "Elvish Archdruid", "Circle of Dreams Druid"], &[EZURI]],
    ready: mantle_ready,
    mana: (0, ""),
    finish: None,
    xcost: &[],
    steps: &[],
}];

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let has = |n: &str| db.id(n).is_some();
    if has(GUIDANCE) {
        r.card(db, GUIDANCE)?.dyn_mana_perm = Some(guidance_mana);
    }
    if has(WAVE) {
        let c = r.card(db, WAVE)?;
        c.resolve = Some(wave_resolve);
        c.prio = Some(wave_prio);
    }
    if has(ELADAMRI) {
        r.card(db, ELADAMRI)?.options = Some(top_options);
    }
    if has(REALMWALKER) {
        let c = r.card(db, REALMWALKER)?;
        c.options = Some(top_options);
        c.as_enters = Some(realmwalker_as);
    }
    if has(AMBUSH) {
        let c = r.card(db, AMBUSH)?;
        c.resolve = Some(ambush_resolve);
        c.prio = Some(zero_prio); // cast when attacked (`ambush_response`)
    }
    for n in [BRIGADE, JORAGA] {
        if has(n) {
            r.card(db, n)?.hand_options = Some(kick_options);
        }
    }
    if has(LEGOLAS) {
        r.card(db, LEGOLAS)?.hand_options = Some(legolas_options);
    }
    if has(THREE_TREE) {
        let c = r.card(db, THREE_TREE)?;
        c.dyn_mana_land = Some(city_amount);
        c.land_cols = Some(city_cols);
        c.land_etb = Some(city_etb);
    }
    if has(LODGE) {
        let c = r.card(db, LODGE)?;
        c.dyn_mana_land = Some(lodge_amount);
        c.land_cols = Some(lodge_cols);
    }
    Ok(())
}
