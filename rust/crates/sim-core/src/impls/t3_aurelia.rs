//! Tier 3 deck `aurelia-boros-extra-combats` (EDHREC's bracket-3 average list): the cards the ability language can't
//! express (rust/agents/t3-model-aurelia.md). The compiled ones (Blade Historian, Avacyn, Firemane Avenger, Aurelia,
//! the Law Above) are patterns in `cardgen::dsl_parse` and `dsl` under "Tier 3 (aurelia)".
//!
//! - Chance for Glory: your creatures gain indestructible; an extra turn you lose at the end of. Cast only for a win.
//! - Trouble in Pairs: opponents' extra turns are skipped (`skips_extra_turn`, read by `turn::run_extra_turns`); the
//!   draw on an opponent's two-creature attack at you, second draw or second spell each turn.
//! - Wear // Tear: destroy an artifact or an enchantment; fused, both (the {W} paid as it resolves).
//! - Angelic Skirmisher: each combat, first strike, vigilance or lifelink for your creatures until end of turn.
//! - Battle Angels of Tyr: the three "more than each other player" rewards on combat damage (myriad is the engine's).
//! - Bruna, the Fading Light: the cast trigger returns an Angel or Human creature card from your graveyard.
//! - Fighter Class: the enters search for an Equipment, levels 2 (equip {2} less: `equip_discount`) and 3 (forced
//!   blocks).
//! - Gisela, Blade of Goldnight: damage to opponents and their permanents doubled, to you and yours halved
//!   (`gisela_scale`, read where damage is dealt: life.rs, combat.rs, removal.rs).
//! - Iroas, God of Victory: a 7/4 creature only with devotion to red and white 7+; damage to your attacking creatures
//!   is prevented (`iroas_shield`, read by `values::no_damage`). Menace is the compiled static.
//! - Odric, Lunarch Marshal: each combat, your creatures share their keywords.
//! - Slayers' Stronghold, Sunhome, Fortress of the Legion: the lands' pump abilities.
//! - Sunforger: {R}{W}, unattach: cast a red or white instant with mana value 4 or less from the library, free.

use crate::cards::{CardDb, Types};
use crate::engine::cast::{cast_card, spell_targets};
use crate::engine::combat::{can_block, first_strike, has_haste};
use crate::engine::life::gain;
use crate::engine::mana::{can_pay, pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, ability_window_card, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{epow, etgh, has_type, indestructible, once_per_turn, ptypes, pval, untargetable};
use crate::engine::zones::{Enter, draw, enter, max_by, searchable};
use crate::flow::Res;
use crate::hooks::{Action, Assign, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

const GLORY: &str = "Chance for Glory";
const TROUBLE: &str = "Trouble in Pairs";
const GISELA: &str = "Gisela, Blade of Goldnight";
const IROAS: &str = "Iroas, God of Victory";
const FIGHTER: &str = "Fighter Class";

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// p's creatures on the battlefield
fn creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

/// how many permanents named `name` p controls
fn count_named(g: &Game, p: PlayerId, name: &str) -> usize {
    let Some(c) = g.db.id(name) else { return 0 };
    g.player(p).perms.iter().filter(|&&m| live(g, m) && g.perm(m).cd == Some(c)).count()
}

/// an until-end-of-turn keyword, once
fn grant(g: &mut Game, m: PermId, kw: Sym) {
    let x = g.perm_mut(m);
    if !x.eot_kw.contains(&kw) {
        x.eot_kw.push(kw);
    }
}

// ================================================================== Chance for Glory
/// a rough count of the damage p's creatures deal next turn: power (doubled for double strike, and for a second
/// combat when Aurelia, the Warleader is out), less the biggest attackers' share, one per opposing creature that
/// could block
fn next_turn_damage(g: &Game, p: PlayerId) -> i32 {
    let combats = if count_named(g, p, "Aurelia, the Warleader") > 0 { 2 } else { 1 };
    let mut hits: Vec<i32> = creatures(g, p)
        .into_iter()
        .filter(|&m| !g.perm(m).noatk && epow(g, m) > 0)
        .map(|m| epow(g, m) * if crate::engine::combat::double_strike(g, m) { 2 } else { 1 } * combats)
        .collect();
    hits.sort_unstable_by(|a, b| b.cmp(a));
    let blockers: usize = g.opps(p).map(|q| creatures(g, q).len()).sum();
    hits.iter().skip(blockers).sum()
}

/// Creatures you control gain indestructible (for as long as they stay: no duration is given). Take an extra turn
/// after this one; at the beginning of that turn's end step, you lose the game (`turn::add_extra_turn`, as Final
/// Fortune). Nothing about Oracle is left out.
fn glory_resolve(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for m in creatures(g, p) {
        g.perm_mut(m).data.set(DataKey::Indestr, Val::Bool(true));
    }
    crate::glog!(
        g,
        "  {}: {} resolves (creatures indestructible; an extra turn, lost at its end step)",
        pname(g, p),
        cname(g, c)
    );
    crate::engine::turn::add_extra_turn(g, p, true);
    Ok("gy")
}

/// the AI casts it after its own combat, never on an extra turn, only when next turn's attack (a rough count) beats
/// the life of every opponent left: the extra turn must win
fn glory_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.doomed_turn.is_some()
        || g.extra_turns.iter().any(|&(q, _)| q == p)
        || g.active != Some(p)
        || g.step != crate::state::Step::Main2
    {
        return 0;
    }
    let life: i32 = g.opps(p).map(|q| g.player(q).life.max(0)).sum();
    if life > 0 && next_turn_damage(g, p) >= life { 85 } else { 0 }
}

// ================================================================== Trouble in Pairs
/// If an opponent would begin an extra turn, that player skips that turn instead (read by `turn::run_extra_turns`)
pub fn skips_extra_turn(g: &Game, q: PlayerId) -> bool {
    g.opps(q).any(|o| count_named(g, o, TROUBLE) > 0)
}

fn trouble_draw_card(g: &mut Game, src: Src, why: &str) -> Res {
    let o = owner(g, src);
    if live(g, src) && trigger_window(g, o, Some(src), "draw a card", Some(2.0))? {
        crate::glog!(g, "    {} draws a card (Trouble in Pairs: {why})", pname(g, o));
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// Whenever an opponent attacks you with two or more creatures, you draw a card
fn trouble_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let o = owner(g, src);
    if p == o {
        return Ok(vec![]);
    }
    let at_me = atk.iter().filter(|&&m| g.atk_targets.iter().any(|&(x, q)| x == m && q == o)).count();
    if at_me >= 2 {
        trouble_draw_card(g, src, "attacked by two or more")?;
    }
    Ok(vec![])
}

/// ... draws their second card each turn: once per opponent and turn, as soon as they have drawn two (a trigger
/// resolves after a "draw two" is over, so the count is read as "two or more", once)
fn trouble_draw(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if p == o || g.player(p).draw_st != Some(g.turn_stamp()) || g.player(p).draw_n < 2 {
        return Ok(());
    }
    let key = intern(&format!("trouble_draw_{}_{}", src.0, p.0));
    if once_per_turn(g, o, key) {
        trouble_draw_card(g, src, "an opponent's second card")?;
    }
    Ok(())
}

/// ... or casts their second spell each turn (once per opponent and turn, as the draw)
fn trouble_cast(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    let o = owner(g, src);
    if caster == o || g.player(caster).spells_this_turn < 2 {
        return Ok(());
    }
    let key = intern(&format!("trouble_cast_{}_{}", src.0, caster.0));
    if once_per_turn(g, o, key) {
        trouble_draw_card(g, src, "an opponent's second spell")?;
    }
    Ok(())
}

// ================================================================== Wear // Tear
fn is_art(g: &Game, m: PermId) -> bool {
    ptypes(g, m).has(Types::ARTIFACT)
}

fn is_ench(g: &Game, m: PermId) -> bool {
    ptypes(g, m).has(Types::ENCHANTMENT)
}

/// Wear ({1}{R}): destroy target artifact; Tear ({W}): destroy target enchantment; fuse: both. The AI casts it as
/// removal at the best opposing artifact or enchantment (tags `rem=destroy tgt=ae`, paying Wear's {1}{R}); as it
/// resolves, it fuses when the other half has a target worth it and {W} is open (the fused {1}{R}{W} in all). Left
/// out: Tear alone costs {1}{R} here, not {W}.
fn weartear_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| live(g, t) && (is_art(g, t) || is_ench(g, t))) else { return Ok("gy") };
    let first_art = is_art(g, t);
    apply_removal(g, Some(p), t, "destroy", Some(c))?;
    let other: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&m| live(g, m) && m != t && !untargetable(g, m) && !indestructible(g, m))
        .filter(|&m| if first_art { is_ench(g, m) } else { is_art(g, m) })
        .collect();
    if let Some(b) = max_by(&other, |m| pval(g, m)).filter(|&b| pval(g, b) >= 2.0)
        && can_pay(g, p, 0, "W", false)
    {
        pay(g, p, 0, "W", false)?;
        crate::glog!(g, "    {} fuses Wear // Tear: {} too", pname(g, p), g.perm(b).name);
        apply_removal(g, Some(p), b, "destroy", Some(c))?;
    }
    Ok("gy")
}

// ================================================================== Angelic Skirmisher
/// the ability the AI picks: first strike when a creature could block (or be blocked by) one of ours in a fight,
/// lifelink on its own attack into an open board, else vigilance
fn skirmisher_choice(g: &Game, o: PlayerId, active: PlayerId) -> &'static str {
    let mine = creatures(g, o);
    if mine.is_empty() {
        return "vigilance";
    }
    let foes: usize = if active == o {
        g.opps(o).map(|q| creatures(g, q).iter().filter(|&&b| !g.perm(b).tapped).count()).sum()
    } else {
        creatures(g, active).len()
    };
    if foes > 0 {
        "first strike"
    } else if active == o {
        "lifelink"
    } else {
        "vigilance"
    }
}

/// At the beginning of each combat (every player's), choose first strike, vigilance, or lifelink. Creatures you
/// control gain that ability until end of turn.
fn skirmisher_combat(g: &mut Game, src: Src, active: PlayerId) -> Res {
    if !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    let kw = skirmisher_choice(g, o, active);
    if !trigger_window(g, o, Some(src), &format!("creatures gain {kw}"), Some(2.0))? {
        return Ok(());
    }
    for m in creatures(g, o) {
        grant(g, m, kw);
    }
    crate::glog!(g, "    {}'s creatures gain {kw} (Angelic Skirmisher)", pname(g, o));
    Ok(())
}

// ================================================================== Odric, Lunarch Marshal
const ODRIC_KWS: [&str; 13] = [
    "first strike",
    "flying",
    "deathtouch",
    "double strike",
    "haste",
    "hexproof",
    "indestructible",
    "lifelink",
    "menace",
    "reach",
    "skulk",
    "trample",
    "vigilance",
];

/// does m have keyword kw for Odric (the engine keeps a few on the permanent or as tags)
fn odric_has(g: &Game, m: PermId, kw: &str) -> bool {
    let x = g.perm(m);
    crate::dsl::has_kw(g, m, kw)
        || match kw {
            "flying" => x.fly,
            "deathtouch" => x.dt,
            "vigilance" => x.vig,
            "lifelink" => x.lifelink,
            "trample" => x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Trample)),
            "reach" => x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Reach)),
            "haste" => has_haste(g, m),
            "indestructible" => indestructible(g, m),
            _ => false,
        }
}

/// At the beginning of each combat, creatures you control gain each of the thirteen keywords until end of turn if
/// a creature you control has it. Deathtouch is the permanent's flag: set with a marker, cleared at the end step
/// (`odric_end`).
fn odric_combat(g: &mut Game, src: Src, _active: PlayerId) -> Res {
    if !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    let team = creatures(g, o);
    let shared: Vec<&'static str> =
        ODRIC_KWS.iter().copied().filter(|kw| team.iter().any(|&m| odric_has(g, m, kw))).collect();
    if shared.is_empty() || team.len() < 2 || !trigger_window(g, o, Some(src), "creatures share keywords", Some(2.0))? {
        return Ok(());
    }
    for m in creatures(g, o) {
        for &kw in &shared {
            if kw == "deathtouch" {
                if !g.perm(m).dt {
                    g.perm_mut(m).dt = true;
                    grant(g, m, "odric_dt");
                }
            } else {
                grant(g, m, kw);
            }
        }
    }
    crate::glog!(g, "    {}'s creatures gain {} (Odric)", pname(g, o), shared.join(", "));
    Ok(())
}

/// the deathtouch Odric gave ends with the turn
fn odric_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let o = owner(g, src);
    for m in g.player(o).perms.clone() {
        if g.perm(m).eot_kw.contains(&"odric_dt") {
            let x = g.perm_mut(m);
            x.dt = false;
            x.eot_kw.retain(|k| *k != "odric_dt");
        }
    }
    Ok(())
}

// ================================================================== Battle Angels of Tyr
/// Whenever this creature (or a myriad token copy of it) deals combat damage to a player, draw a card if that player
/// has more cards in hand than each other player. Then you create a Treasure token if that player controls more
/// lands than each other player. Then you gain 3 life if that player has more life than each other player.
fn angels_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if a != src || owner(g, src) != p {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "draw, Treasure, life", Some(2.0))? {
        return Ok(());
    }
    let most = |g: &Game, f: &dyn Fn(&Game, PlayerId) -> usize| {
        let v = f(g, d);
        g.players.iter().filter(|q| q.alive && q.id != d).all(|q| f(g, q.id) < v)
    };
    if most(g, &|g, q| g.player(q).hand.len()) {
        draw(g, p, 1, false)?;
    }
    if most(g, &|g, q| g.player(q).lands.len()) {
        crate::engine::zones::add_treasure(g, p, 1)?;
    }
    if most(g, &|g, q| g.player(q).life.max(0) as usize) {
        gain(g, p, 3)?;
    }
    Ok(())
}

// ================================================================== Bruna, the Fading Light
/// When you cast this spell, you may return target Angel or Human creature card from your graveyard to the
/// battlefield: the AI returns the best one it has (by card worth). The meld with Gisela, the Broken Blade is not
/// modeled (the deck has no Broken Blade).
fn bruna_cast(g: &mut Game, p: PlayerId, _c: CardId) -> Res {
    let cands: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            d.creature && (d.has_subtype("angel") || d.has_subtype("human"))
        })
        .collect();
    let Some(best) = max_by(&cands, |c| card_worth(g, p, c, true) + 0.1 * g.db.get(c).cmc as f64) else {
        return Ok(());
    };
    if !crate::engine::stack::trigger_window_card(
        g,
        p,
        g.db.id("Bruna, the Fading Light").unwrap(),
        "return an Angel or Human",
    )? {
        return Ok(());
    }
    let pl = g.player_mut(p);
    let Some(i) = pl.gy.iter().position(|&x| x == best) else { return Ok(()) };
    pl.gy.remove(i);
    crate::glog!(g, "    {} returns {} to the battlefield (Bruna)", pname(g, p), cname(g, best));
    enter(g, p, best, Enter::default())?;
    Ok(())
}

// ================================================================== Fighter Class
fn class_level(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.get(DataKey::Level).map_or(1, Val::int)
}

/// p's Fighter Classes at level `lvl` or more
fn fighters(g: &Game, p: PlayerId, lvl: i64) -> Vec<PermId> {
    let Some(c) = g.db.id(FIGHTER) else { return vec![] };
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.perm(m).cd == Some(c) && class_level(g, m) >= lvl)
        .collect()
}

/// Level 2: equip abilities you activate cost {2} less to activate (per Class; read by `cloud::equip_cost`)
pub fn equip_discount(g: &Game, p: PlayerId) -> u32 {
    if g.hooks.is_empty() { 0 } else { 2 * fighters(g, p, 2).len() as u32 }
}

/// When this Class enters, search your library for an Equipment card, reveal it, put it into your hand
fn fighter_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m != src || !trigger_window(g, o, Some(src), "search for an Equipment", None)? {
        return Ok(());
    }
    let got = crate::impls::t1::tutor_named(g, o, &|g, c| g.db.get(c).has_subtype("equipment"), 1, "hand")?;
    if let Some(&c) = got.first() {
        crate::glog!(g, "    {} finds {} (Fighter Class)", pname(g, o), cname(g, c));
    }
    Ok(())
}

/// Level 2 ({1}{R}{W}) and Level 3 ({3}{R}{W}), one at a time as a sorcery. The AI takes level 2 with an Equipment
/// out or in hand, level 3 with two creatures to attack with.
fn fighter_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || owner(g, src) != p || !live(g, src) {
        return Ok(vec![]);
    }
    let level = class_level(g, src) + 1;
    let generic = match level {
        2 => 1,
        3 => 3,
        _ => return Ok(vec![]),
    };
    if !can_pay(g, p, generic, "RW", false) {
        return Ok(vec![]);
    }
    let equipment = g.player(p).perms.iter().any(|&m| live(g, m) && has_type(g, m, "equipment"))
        || g.player(p).hand.iter().any(|&c| g.db.get(c).has_subtype("equipment"));
    let ok = if level == 2 { equipment } else { creatures(g, p).len() >= 2 };
    if !ok {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: if level == 3 { 2.0 } else { 1.2 },
        label: format!("Fighter Class level {level}"),
        act: Some(Action::Ability { src, f: fighter_level_up, arg: level }),
    }])
}

fn fighter_level_up(g: &mut Game, src: PermId, p: PlayerId, level: i64) -> Res<bool> {
    let generic = if level == 2 { 1 } else { 3 };
    if !live(g, src) || owner(g, src) != p || class_level(g, src) != level - 1 || !can_pay(g, p, generic, "RW", false) {
        return Ok(false);
    }
    pay(g, p, generic, "RW", false)?;
    if ability_window(g, p, Some(src), &format!("level {level}"), None, None)? && live(g, src) {
        g.perm_mut(src).data.set(DataKey::Level, Val::Int(level));
        crate::glog!(g, "  {} levels Fighter Class to {level}", pname(g, p));
    }
    Ok(true)
}

/// Level 3: whenever a creature you control attacks, up to one target creature blocks it this combat if able. The
/// AI makes an opposing creature block each of its attackers left unblocked when the attacker kills it and lives
/// (the targets are chosen as blockers are declared, on the blocks the defender made: an approximation of choosing
/// them as attackers are declared)
fn fighter_blocks(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId, assign: &mut Assign) -> Res {
    if owner(g, src) != p || !live(g, src) || class_level(g, src) < 3 {
        return Ok(());
    }
    let mut forced: Vec<PermId> = vec![];
    for &a in atk {
        if !live(g, a) || owner(g, a) != p || assign.iter().any(|x| x.0 == a) {
            continue;
        }
        let key = intern(&format!("fighter_{}_{}_{}", src.0, a.0, g.player(p).combat_no));
        let cands: Vec<PermId> = creatures(g, d)
            .into_iter()
            .filter(|&b| {
                !g.perm(b).tapped
                    && !forced.contains(&b)
                    && !untargetable(g, b)
                    && can_block(g, b, a)
                    && (epow(g, a) >= etgh(g, b) || g.perm(a).dt)
                    && !indestructible(g, b)
                    && (indestructible(g, a)
                        || (first_strike(g, a) && !first_strike(g, b))
                        || (epow(g, b) < etgh(g, a) && !g.perm(b).dt))
            })
            .collect();
        let Some(b) = max_by(&cands, |b| pval(g, b)) else { continue };
        if !once_per_turn(g, p, key) || !trigger_window(g, p, Some(src), "target creature blocks it", Some(2.0))? {
            continue;
        }
        if !live(g, b) || !live(g, a) {
            continue;
        }
        assign.retain(|x| x.1 != b);
        assign.push((a, b));
        forced.push(b);
        crate::glog!(g, "    {} must block {} (Fighter Class)", g.perm(b).name, g.perm(a).name);
    }
    Ok(())
}

// ================================================================== Gisela, Blade of Goldnight
/// damage n about to be dealt to player `victim` or a permanent they control, after every Gisela: doubled once per
/// Gisela an opponent of theirs controls, half prevented (rounded up) once per Gisela they control. The affected
/// player orders the replacements: halves first, then doubles.
pub fn gisela_scale(g: &Game, victim: PlayerId, n: i32) -> i32 {
    if n <= 0 || g.hooks.is_empty() {
        return n;
    }
    let Some(gc) = g.db.id(GISELA) else { return n };
    let (mut halve, mut double) = (0u32, 0u32);
    for q in g.players.iter().filter(|q| q.alive) {
        let k = q.perms.iter().filter(|&&m| live(g, m) && g.perm(m).cd == Some(gc)).count() as u32;
        if q.id == victim {
            halve += k;
        } else {
            double += k;
        }
    }
    let mut n = n;
    if !g.unpreventable() {
        for _ in 0..halve {
            n /= 2;
        }
    }
    n.saturating_mul(1 << double.min(20))
}

// ================================================================== Iroas, God of Victory
/// Iroas is a 7/4 creature only while its controller's devotion to red and white is seven or more
fn iroas_update(g: &mut Game, src: PermId) {
    let o = owner(g, src);
    let dev = crate::impls::rules2::devotion(g, o, 'R') + crate::impls::rules2::devotion(g, o, 'W');
    let x = g.perm_mut(src);
    (x.pow, x.tgh) = (7, 4);
    x.data.set(DataKey::Anim, Val::Bool(dev >= 7));
}

fn iroas_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src || owner(g, m) == owner(g, src) {
        iroas_update(g, src);
    }
    Ok(())
}

fn iroas_sba(g: &mut Game, src: Src) -> Res {
    iroas_update(g, src);
    Ok(())
}

fn iroas_indestructible(_g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "indestructible" && m == src
}

/// Prevent all damage that would be dealt to attacking creatures you control (read by `values::no_damage`)
pub fn iroas_shield(g: &Game, m: PermId) -> bool {
    !g.in_combat.is_empty() && g.in_combat.contains(&m) && !g.hooks.is_empty() && count_named(g, owner(g, m), IROAS) > 0
}

// ================================================================== Slayers' Stronghold / Sunhome
fn land_ready(g: &Game, l: PermId, p: PlayerId) -> bool {
    g.perm(l).on_bf && g.player(p).lands.contains(&l) && !g.perm(l).tapped
}

fn pack(l: PermId, m: PermId) -> i64 {
    l.0 as i64 | (m.0 as i64) << 32
}

fn unpack(arg: i64) -> (PermId, PermId) {
    (PermId((arg & 0xFFFF_FFFF) as u32), PermId((arg >> 32) as u32))
}

fn can_attack_now(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    live(g, m) && g.is_creature(m) && !x.tapped && !x.noatk && epow(g, m) > 0
}

/// {R}{W}, {T}: target creature gets +2/+0 and gains vigilance and haste until end of turn. The AI uses it in its
/// first main phase to give haste to its biggest creature that came in this turn (power 3 or more)
fn stronghold_options(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || !land_ready(g, l, p) {
        return Ok(vec![]);
    }
    if !crate::impls::lands::can_pay_without(g, p, l, 0, "RW") {
        return Ok(vec![]);
    }
    let sick: Vec<PermId> = creatures(g, p)
        .into_iter()
        .filter(|&m| can_attack_now(g, m) && g.perm(m).sick && !has_haste(g, m) && epow(g, m) >= 3)
        .collect();
    let Some(best) = max_by(&sick, |m| epow(g, m) as f64) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 1.2 + 0.2 * epow(g, best) as f64,
        label: format!("Slayers' Stronghold: {} gets +2/+0, vigilance and haste", g.perm(best).name),
        act: Some(Action::Plan { f: stronghold_go, arg: pack(l, best) }),
    }])
}

fn stronghold_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, m) = unpack(arg);
    if !land_ready(g, l, p) || !live(g, m) || !crate::impls::lands::pay_without(g, p, l, 0, "RW")? {
        return Ok(false);
    }
    g.perm_mut(l).tapped = true;
    if ability_window_card(g, p, g.perm(l).card(), "+2/+0, vigilance and haste", None, Some(m))? && live(g, m) {
        let x = g.perm_mut(m);
        x.eot_pt = (x.eot_pt.0 + 2, x.eot_pt.1);
        grant(g, m, "vigilance");
        grant(g, m, "haste");
        crate::glog!(g, "    {} gets +2/+0, vigilance and haste (Slayers' Stronghold)", g.perm(m).name);
    }
    Ok(true)
}

/// {2}{R}{W}, {T}: target creature gains double strike until end of turn. The AI uses it in its first main phase on
/// its best ready attacker of power 3 or more that would get through: a flyer when no opponent has a flyer or reach
/// creature untapped, or one no opponent has an untapped creature for
fn sunhome_options(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || !land_ready(g, l, p) {
        return Ok(vec![]);
    }
    if !crate::impls::lands::can_pay_without(g, p, l, 2, "RW") {
        return Ok(vec![]);
    }
    let blockers: Vec<PermId> = g.opps(p).flat_map(|q| creatures(g, q)).filter(|&b| !g.perm(b).tapped).collect();
    let ready: Vec<PermId> = creatures(g, p)
        .into_iter()
        .filter(|&m| {
            can_attack_now(g, m)
                && (!g.perm(m).sick || has_haste(g, m))
                && epow(g, m) >= 3
                && !crate::engine::combat::double_strike(g, m)
                && !blockers.iter().any(|&b| can_block(g, b, m))
        })
        .collect();
    let Some(best) = max_by(&ready, |m| epow(g, m) as f64 + if g.perm(m).is_cmd { 2.0 } else { 0.0 }) else {
        return Ok(vec![]);
    };
    Ok(vec![Opt {
        utility: 1.0 + 0.25 * epow(g, best) as f64,
        label: format!("Sunhome: {} gains double strike", g.perm(best).name),
        act: Some(Action::Plan { f: sunhome_go, arg: pack(l, best) }),
    }])
}

fn sunhome_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, m) = unpack(arg);
    if !land_ready(g, l, p) || !live(g, m) || !crate::impls::lands::pay_without(g, p, l, 2, "RW")? {
        return Ok(false);
    }
    g.perm_mut(l).tapped = true;
    if ability_window_card(g, p, g.perm(l).card(), "double strike", None, Some(m))? && live(g, m) {
        grant(g, m, "double strike");
        crate::glog!(g, "    {} gains double strike (Sunhome)", g.perm(m).name);
    }
    Ok(true)
}

// ================================================================== Sunforger
/// a card Sunforger can find: a red or white instant with mana value 4 or less
fn forgeable(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.instant && d.cmc <= 4 && (d.colors.has('R') || d.colors.has('W'))
}

/// the removal instant Sunforger would find now and its target's value
fn forge_pick(g: &Game, p: PlayerId) -> Option<(CardId, f64)> {
    let mut best: Option<(CardId, f64)> = None;
    for c in searchable(g, p) {
        if !forgeable(g, c) || !g.db.get(c).tag(Tag::Rem) {
            continue;
        }
        let ctx = spell_targets(g, p, c, None);
        let Some(t) = ctx.target else { continue };
        let v = pval(g, t);
        if best.is_none_or(|(_, bv)| v > bv) {
            best = Some((c, v));
        }
    }
    best
}

/// {R}{W}, unattach Sunforger: search your library for a red or white instant card with mana value 4 or less and
/// cast that card without paying its mana cost. Then shuffle. The AI uses it for removal: in the end-of-turn window,
/// or in its first main phase, when an opposing permanent worth 4 or more is a target (Swords to Plowshares, Path to
/// Exile, Chaos Warp, Wear // Tear ...). Left out: the protective finds (Boros Charm, Teferi's Protection, Flawless
/// Maneuver in response to a wipe; Akroma's Will in combat).
fn sunforger_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || !live(g, src) || g.perm(src).attached.is_none() || post == Some(true) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 0, "RW", false) {
        return Ok(vec![]);
    }
    let Some((c, v)) = forge_pick(g, p).filter(|x| x.1 >= 4.0) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 1.0 + 0.3 * v,
        label: format!("Sunforger: cast {}", cname(g, c)),
        act: Some(Action::Ability { src, f: sunforger_go, arg: c.0 as i64 }),
    }])
}

fn sunforger_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !live(g, src) || g.perm(src).attached.is_none() || !can_pay(g, p, 0, "RW", false) {
        return Ok(false);
    }
    pay(g, p, 0, "RW", false)?;
    g.perm_mut(src).attached = None; // the unattach is a cost
    if !ability_window(g, p, Some(src), "search for an instant and cast it", None, None)? {
        return Ok(true);
    }
    let pl = g.player_mut(p);
    let Some(i) = pl.library.iter().position(|&x| x == c) else { return Ok(true) };
    pl.library.remove(i);
    shuffle_library(g, p);
    g.last_x = 0;
    let ctx = spell_targets(g, p, c, None);
    crate::glog!(g, "    {} casts {} without paying its mana cost (Sunforger)", pname(g, p), cname(g, c));
    cast_card(g, p, c, "lib", ctx)?;
    Ok(true)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, GLORY)?;
    c.resolve = Some(glory_resolve);
    c.prio = Some(glory_prio);
    let c = r.card(db, TROUBLE)?;
    c.attack = Some(trouble_attack);
    c.draw = Some(trouble_draw);
    c.cast = Some(trouble_cast);
    r.card(db, "Wear // Tear")?.resolve = Some(weartear_resolve);
    r.card(db, "Angelic Skirmisher")?.each_combat = Some(skirmisher_combat);
    let c = r.card(db, "Odric, Lunarch Marshal")?;
    c.each_combat = Some(odric_combat);
    c.end_step = Some(odric_end);
    r.card(db, "Battle Angels of Tyr")?.combat_damage = Some(angels_damage);
    r.card(db, "Bruna, the Fading Light")?.self_cast = Some(bruna_cast);
    let c = r.card(db, FIGHTER)?;
    c.etb = Some(fighter_etb);
    c.options = Some(fighter_options);
    c.blocks = Some(fighter_blocks);
    // Gisela's damage rule lives in `gisela_scale`; registering marks her as hooked (it reads `g.hooks`)
    r.card(db, GISELA)?.sba = Some(|_, _| Ok(()));
    let c = r.card(db, IROAS)?;
    c.etb = Some(iroas_etb);
    c.sba = Some(iroas_sba);
    c.grant_kw = Some(iroas_indestructible);
    r.card(db, "Slayers' Stronghold")?.land_options = Some(stronghold_options);
    r.card(db, "Sunhome, Fortress of the Legion")?.land_options = Some(sunhome_options);
    r.card(db, "Sunforger")?.options = Some(sunforger_options);
    Ok(())
}
