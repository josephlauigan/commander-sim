//! Practice-mode opponent-effects probe (2026-11-04): every card in the opponent pool decks JD's decks face
//! (`data/decks.json` `.pool[]` whose `.tier` is in the union of `.mine[].tiers`) whose Oracle text can make *another
//! player* choose or lose something of their choice (sacrifice, discard, "of their choice", "chooses", "unless ...
//! pays", annihilator, "each player", "target player" ...) is put on a table where the person is P0 and an AI
//! opponent (P1) owns the card. P0 holds at least two of everything an effect could touch (creatures of different
//! powers, artifacts, enchantments, planeswalkers, lands, cards in hand and graveyard, floating mana and untapped
//! lands to pay a tax). P0 answers every question with the first choice (yes), and every question is recorded.
//!
//! Each card is driven the way it would hit the person, one fresh table per moment (`M`): an instant or sorcery cast by
//! P1; a permanent cast or entering under P1 (its enter trigger); then, for permanents, the moments its text may
//! trigger on: P0 casts a spell, P0 draws, P1's and P0's upkeep, end step and main phases, P1 attacks P0 (annihilator,
//! equipped creatures, combat damage), a P1 creature (or the card) dies, P1 casts a spell, a land enters, P0 attacks
//! P1, and every activated ability the engine offers P1 for the card (loyalty abilities, tap abilities).
//!
//! A card is flagged when something of P0's was sacrificed / discarded / exiled / returned or a tax was paid, with two
//! or more candidates, and P0 was asked nothing (the engine chose for the person). The same question twice in a row
//! is flagged too. Flagged cards are triaged into `TO_FIX` (a real gap) and `FALSE_ALARMS` (with why); the test fails
//! on a new flag and on a stale entry. Run with `PROBE_VERBOSE=1` for every flagged moment's detail.

mod play_common;
use play_common::*;
use sim_core::cards::Types;
use sim_core::engine::{cast, combat, stack, turn, zones};
use sim_core::state::{Ctx, Step};
use std::collections::{BTreeMap, BTreeSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

fn json(file: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(data_dir().join(file)).unwrap()).unwrap()
}

fn strip_reminders(t: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for ch in t.chars() {
        match ch {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out
}

/// the Oracle text of every face, lower case, reminder text removed
fn oracle(cache: &Value, name: &str) -> Option<String> {
    let e = cache.get(name.to_lowercase())?;
    let mut t = e["oracle_text"].as_str().unwrap_or("").to_string();
    if let Some(fs) = e["card_faces"].as_array() {
        for f in fs {
            t.push('\n');
            t.push_str(f["oracle_text"].as_str().unwrap_or(""));
        }
    }
    Some(strip_reminders(&t).to_lowercase())
}

/// Oracle text that can make another player choose or lose something of their choice
fn relevant(t: &str) -> bool {
    const W: &[&str] = &[
        "sacrifices",
        "discards",
        "chooses",
        "pays",
        "annihilator",
        "of their choice",
        "of his or her choice",
        "each player",
        "each opponent",
        "each other player",
        "target player",
        "target opponent",
        "that player",
        "defending player",
        "unless",
        "their hand",
        "their graveyard",
        "an opponent's graveyard",
    ];
    W.iter().any(|w| t.contains(w))
}

// ------------------------------------------------------------------------------------------------ the table
const P0_KEY: &str = "kefka";

fn build(key1: &str) -> (Game, Arc<ScriptHuman>) {
    let mut g: Game = table(&[P0_KEY, key1]);
    g.active = Some(P1);
    g.step = Step::Main1;
    g.log = Some(vec![]);
    for p in [P0, P1] {
        g.player_mut(p).life = 100;
    }
    // the person's battlefield: four creatures of different powers, three artifacts, three enchantments, two
    // planeswalkers, ten lands (two of each colour)
    for n in ["Llanowar Elves", "Armored Warhorse", "Goblin Roughrider", "Birds of Paradise"] {
        perm(&mut g, P0, n);
    }
    for n in ["Sol Ring", "Arcane Signet", "Mind Stone", "Parallel Lives", "Anointed Procession", "Doubling Season"] {
        perm(&mut g, P0, n);
    }
    for n in ["Chandra, Torch of Defiance", "Garruk Wildspeaker"] {
        let m = perm(&mut g, P0, n);
        g.perm_mut(m).loyalty = Some(30); // combat damage and a burn spell must not kill it: only a choice loses it
    }
    for n in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(&mut g, P0, n, 2, false);
    }
    // hand, graveyard
    hand(&mut g, P0, &["Plains", "Island", "Swamp", "Lightning Bolt", "Counterspell"]);
    for n in ["Glory Seeker", "Swordwise Centaur", "Sol Ring", "Counterspell", "Lightning Bolt", "Forest"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    pool(&mut g, P0, &[('W', 3), ('U', 3), ('B', 3), ('R', 3), ('G', 3), ('C', 3)]);
    // the opponent: two creatures, lands of every colour, something in the graveyard
    for n in ["Armored Warhorse", "Glory Seeker"] {
        perm(&mut g, P1, n);
    }
    for n in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(&mut g, P1, n, 3, false);
    }
    for n in ["Goblin Roughrider", "Swordwise Centaur", "Sol Ring"] {
        let c = take(&mut g, P1, n);
        g.player_mut(P1).gy.push(c);
    }
    let h = Arc::new(ScriptHuman::from_fn(|r: &Request| match &*r.kind {
        "priority" => Answer::Action(json!({"do": "pass"})),
        "attack" | "block" => Answer::Picks(vec![]),
        _ if r.choices.is_empty() => Answer::Yes,
        _ => Answer::Pick(0),
    }));
    g.humans.add(P0, h.clone());
    (g, h)
}

// ------------------------------------------------------------------------------------------------ detecting
#[derive(Clone, Default)]
struct Snap {
    /// P0's battlefield: (permanent, name, kinds: C creature, A artifact, E enchantment, P planeswalker, L land)
    bf: Vec<(PermId, String, String)>,
    hand: Vec<sim_core::ids::CardId>,
    gy: Vec<sim_core::ids::CardId>,
    exile: Vec<sim_core::ids::CardId>,
    tapped: usize,
    mana: u32,
    life: i32,
    /// P1: cards in hand, permanents (lands too), cards in library, Treasures
    p1: [usize; 4],
}

fn snap(g: &Game) -> Snap {
    let p = g.player(P0);
    let mut s = Snap::default();
    for &m in p.perms.iter().chain(p.lands.iter()) {
        let x = g.perm(m);
        if !x.on_bf {
            continue;
        }
        let mut k = String::new();
        if g.is_creature(m) {
            k.push('C');
        }
        if let Some(cd) = x.cd {
            let t = g.db.get(cd).types;
            for (ty, ch) in
                [(Types::ARTIFACT, 'A'), (Types::ENCHANTMENT, 'E'), (Types::PLANESWALKER, 'P'), (Types::LAND, 'L')]
            {
                if t.has(ty) {
                    k.push(ch);
                }
            }
        }
        if x.tapped {
            s.tapped += 1;
        }
        s.bf.push((m, x.name.to_string(), k));
    }
    s.hand = p.hand.clone();
    s.gy = p.gy.clone();
    s.exile = p.exile.iter().copied().collect();
    s.mana = p.pool.m.iter().sum::<u32>() + p.pool.k.iter().sum::<u32>() + p.pool.art + p.treasures;
    s.life = p.life;
    let o = g.player(P1);
    s.p1 = [o.hand.len(), o.perms.len() + o.lands.len(), o.library.len(), o.treasures as usize];
    s
}

fn missing(before: &[sim_core::ids::CardId], after: &[sim_core::ids::CardId]) -> Vec<sim_core::ids::CardId> {
    let mut left = after.to_vec();
    let mut out = vec![];
    for &c in before {
        match left.iter().position(|&x| x == c) {
            Some(i) => {
                left.remove(i);
            }
            None => out.push(c),
        }
    }
    out
}

/// what the engine took from the person, per category, when it had a choice (two or more candidates and not all
/// taken); gated by the card's text so that a targeted effect isn't counted
fn losses(g: &Game, b: &Snap, a: &Snap, text: &str) -> Vec<String> {
    let mut out = vec![];
    let has = |w: &str| text.contains(w);
    let perm_gate = has("sacrific")
        || has("choice")
        || has("chooses")
        || has("annihilator")
        || has("each player")
        || has("each opponent")
        || has("each other player");
    if perm_gate {
        let after: BTreeSet<PermId> = a.bf.iter().map(|x| x.0).collect();
        for (kind, label) in
            [('C', "creature"), ('A', "artifact"), ('E', "enchantment"), ('P', "planeswalker"), ('L', "land")]
        {
            let total = b.bf.iter().filter(|x| x.2.contains(kind)).count();
            let lost: Vec<&str> =
                b.bf.iter().filter(|x| x.2.contains(kind) && !after.contains(&x.0)).map(|x| x.1.as_str()).collect();
            if total >= 2 && !lost.is_empty() && lost.len() < total {
                out.push(format!("{label}: lost {} of {} ({})", lost.len(), total, lost.join(", ")));
            }
        }
    }
    let random_only = has("at random") && !has("choice") && !has("chooses");
    if (has("discard") || has("hand") || has("choice") || has("chooses") || has("each player") || has("each opponent"))
        && !random_only
    {
        let gone = missing(&b.hand, &a.hand);
        if b.hand.len() >= 2 && !gone.is_empty() && gone.len() < b.hand.len() {
            let n: Vec<String> = gone.iter().map(|&c| g.db.get(c).name.to_string()).collect();
            out.push(format!("hand: lost {} of {} ({})", gone.len(), b.hand.len(), n.join(", ")));
        }
    }
    let gone = missing(&b.gy, &a.gy);
    if b.gy.len() >= 2 && !gone.is_empty() && gone.len() < b.gy.len() && !has("target card") {
        let n: Vec<String> = gone.iter().map(|&c| g.db.get(c).name.to_string()).collect();
        out.push(format!("graveyard: lost {} of {} ({})", gone.len(), b.gy.len(), n.join(", ")));
    }
    if (has("unless") || has("pay")) && (a.tapped > b.tapped || a.mana < b.mana) {
        out.push(format!("tax paid (tapped {} -> {}, floating mana {} -> {})", b.tapped, a.tapped, b.mana, a.mana));
    }
    out
}

// ------------------------------------------------------------------------------------------------ moments
#[derive(Clone, Copy, Debug, PartialEq)]
enum M {
    /// P1 casts the card (a land: plays it)
    Cast,
    P0CastSpell,
    P0CastCreature,
    P0CastInstant,
    P0Draw,
    P0DrawStep,
    UpkeepP1,
    UpkeepP0,
    EndP1,
    EndP0,
    Main1P1,
    Main2P1,
    Attack,
    /// as Attack, with no planeswalker of P0's to take the damage: it connects with the player
    AttackPlayer,
    DiesOther,
    DiesSelf,
    P1CastSpell,
    P1CastCreature,
    P1Land,
    P0Land,
    P0Attack,
    /// P0 casts a spell while P1 holds the card (a counterspell) with mana up
    HeldCounter,
    /// the k-th ability P1 is offered for the card in its hand (channel, cycling ...)
    HandAbility(usize),
    /// the k-th activated ability the engine offers P1 for the card
    Ability(usize),
}

const PERMANENT_MOMENTS: &[M] = &[
    M::P0CastSpell,
    M::P0CastCreature,
    M::P0CastInstant,
    M::P0Draw,
    M::P0DrawStep,
    M::UpkeepP1,
    M::UpkeepP0,
    M::EndP1,
    M::EndP0,
    M::Main1P1,
    M::Main2P1,
    M::Attack,
    M::AttackPlayer,
    M::DiesOther,
    M::DiesSelf,
    M::P1CastSpell,
    M::P1CastCreature,
    M::P1Land,
    M::P0Land,
    M::P0Attack,
];

#[derive(Default)]
struct Out {
    asked: Vec<String>,
    auto: Vec<String>,
    lost: Vec<String>,
    /// the game log lines the moment wrote
    log: Vec<String>,
    /// the card was cast / entered and is on the battlefield, in the graveyard or exiled (it did something)
    in_play: bool,
}

/// P1's vanilla creature (another is put onto the battlefield if an enter effect took it)
fn warhorse(g: &mut Game) -> PermId {
    match g.player(P1).perms.iter().copied().find(|&x| &*g.perm(x).name == "Armored Warhorse" && g.perm(x).on_bf) {
        Some(x) => x,
        None => perm(g, P1, "Armored Warhorse"),
    }
}

fn err(e: sim_core::flow::Stop) -> String {
    format!("{e:?}")
}

fn is_ability_of(a: &sim_core::hooks::Action, m: PermId) -> bool {
    use sim_core::hooks::Action::*;
    matches!(a, Ability { src, .. } | Dsl { src, .. } | Loyalty { src, .. } | Equip { src, .. } if *src == m)
}

/// the labels of the activated abilities P1 is offered for the card once it is on the battlefield
fn abilities(card: &str, key1: &str) -> Result<Vec<String>, String> {
    let (mut g, _) = build(key1);
    let c = take(&mut g, P1, card);
    let m = enter(&mut g, c)?;
    let opts = sim_core::ai::brain::main_options(&mut g, P1, false).map_err(err)?;
    Ok(opts.iter().filter(|o| o.act.as_ref().is_some_and(|a| is_ability_of(a, m))).map(|o| o.label.clone()).collect())
}

/// the plays P1 is offered for card c in its hand other than casting it (channel, cycling, a land's hand ability)
fn hand_opts(g: &mut Game, c: sim_core::ids::CardId) -> Result<Vec<sim_core::hooks::Opt>, String> {
    use sim_core::hooks::Action::*;
    let name = g.db.get(c).name.to_lowercase();
    let opts = sim_core::ai::brain::main_options(g, P1, false).map_err(err)?;
    Ok(opts
        .into_iter()
        .filter(|o| o.label.to_lowercase().contains(&name.split(',').next().unwrap_or("").to_string()))
        .filter(|o| {
            o.act.as_ref().is_some_and(|a| !matches!(a, Cast { .. } | Removal { .. } | Wipe { .. } | Face { .. }))
        })
        .collect())
}

fn hand_abilities(card: &str, key1: &str) -> Result<Vec<String>, String> {
    let (mut g, _) = build(key1);
    let c = take(&mut g, P1, card);
    g.player_mut(P1).hand.push(c);
    Ok(hand_opts(&mut g, c)?.iter().map(|o| o.label.clone()).collect())
}

fn enter(g: &mut Game, c: sim_core::ids::CardId) -> Result<PermId, String> {
    if g.db.get(c).land {
        let l = g.add_land(P1, c, false);
        return Ok(l);
    }
    let m = zones::enter(g, P1, c, zones::Enter { sick: false, ..Default::default() }).map_err(err)?;
    stack::flush_triggers(g).map_err(err)?;
    if g.perm(m).loyalty.is_some() {
        g.perm_mut(m).loyalty = Some(30); // every loyalty ability, the ultimate too
    }
    Ok(m)
}

fn run(card: &str, key1: &str, text: &str, m: M) -> Result<Out, String> {
    let (mut g, h) = build(key1);
    let c = take(&mut g, P1, card);
    let (spell, land, creature) = {
        let d = g.db.get(c);
        (d.instant || d.sorcery, d.land, d.creature)
    };
    let attachable = text.contains("equip") || text.contains("enchant creature") || text.contains("enchanted creature");
    let mut probe: Option<PermId> = None;
    if !matches!(m, M::Cast | M::HeldCounter | M::HandAbility(_)) {
        if spell {
            return Err("an instant or sorcery has only the cast moment".into());
        }
        probe = Some(enter(&mut g, c)?);
    }
    let n_asked = h.asked().len();
    let n_told = h.told().len();
    let n_log = g.log.as_ref().map_or(0, |l| l.len());
    let mut before = snap(&g);
    let mut cast_card_of_p0: Option<sim_core::ids::CardId> = None;
    match m {
        M::Cast => {
            before = snap(&g);
            if land {
                turn::play_land_card(&mut g, P1, c, "plays").map_err(err)?;
            } else {
                g.player_mut(P1).hand.push(c);
                let mut ctx = if spell { cast::spell_targets(&g, P1, c, None) } else { Ctx::default() };
                ctx.x = 3; // an X spell (Torment of Hailfire) is cast for three
                cast::cast_card(&mut g, P1, c, "hand".into(), ctx).map_err(err)?;
            }
        }
        M::P0CastSpell | M::P0CastCreature | M::P0CastInstant => {
            let n = match m {
                M::P0CastSpell => "Sol Ring",
                M::P0CastCreature => "Armored Warhorse",
                _ => "Lightning Bolt",
            };
            let x = take(&mut g, P0, n);
            g.player_mut(P0).hand.push(x);
            cast_card_of_p0 = Some(x);
            before = snap(&g);
            g.active = Some(P0);
            let ctx = if m == M::P0CastInstant { Ctx { face: Some(P1), ..Ctx::default() } } else { Ctx::default() };
            cast::cast_card(&mut g, P0, x, "hand".into(), ctx).map_err(err)?;
        }
        M::P0Draw => zones::draw(&mut g, P0, 1, false).map_err(err)?,
        M::P0DrawStep => {
            g.active = Some(P0);
            zones::draw(&mut g, P0, 1, true).map_err(err)?
        }
        M::UpkeepP1 => turn::upkeep(&mut g, P1).map_err(err)?,
        M::UpkeepP0 => {
            g.active = Some(P0);
            turn::upkeep(&mut g, P0).map_err(err)?
        }
        M::EndP1 => turn::end_step(&mut g, P1).map_err(err)?,
        M::EndP0 => {
            g.active = Some(P0);
            turn::end_step(&mut g, P0).map_err(err)?
        }
        M::Main1P1 | M::Main2P1 => {
            use sim_core::hooks::{Call, Event};
            let e = if m == M::Main1P1 { Event::Main1 } else { Event::Main2 };
            sim_core::engine::hooks::fire_trigger(&mut g, e, Call::Player { p: P1 }).map_err(err)?
        }
        M::Attack | M::AttackPlayer => {
            let probe = probe.unwrap();
            if m == M::AttackPlayer {
                for w in g.player(P0).perms.clone() {
                    if g.perm(w).loyalty.is_some() {
                        zones::leave(&mut g, w).map_err(err)?;
                    }
                }
                before = snap(&g);
            }
            let warhorse = warhorse(&mut g);
            if attachable && !creature {
                g.perm_mut(probe).attached = Some(warhorse);
            }
            g.step = Step::Combat;
            let atk: Vec<PermId> =
                g.player(P1).perms.iter().copied().filter(|&x| g.is_creature(x) && g.perm(x).on_bf).collect();
            for &x in &atk {
                g.perm_mut(x).sick = false;
            }
            combat::attack_triggers(&mut g, P1, &atk, P0).map_err(err)?;
            stack::flush_triggers(&mut g).map_err(err)?;
            combat::resolve_combat(&mut g, P1, &atk, P0, &atk).map_err(err)?;
        }
        M::DiesOther => {
            let x = warhorse(&mut g);
            zones::die(&mut g, x, "destroy".into()).map_err(err)?;
        }
        M::DiesSelf => zones::die(&mut g, probe.unwrap(), "destroy".into()).map_err(err)?,
        M::P1CastSpell | M::P1CastCreature => {
            let n = if m == M::P1CastSpell { "Mind Stone" } else { "Goblin Roughrider" };
            let x = take(&mut g, P1, n);
            g.player_mut(P1).hand.push(x);
            cast::cast_card(&mut g, P1, x, "hand".into(), Ctx::default()).map_err(err)?;
        }
        M::P1Land | M::P0Land => {
            let p = if m == M::P1Land { P1 } else { P0 };
            let x = take(&mut g, p, "Forest");
            if p == P0 {
                g.active = Some(P0);
            }
            turn::play_land_card(&mut g, p, x, "plays").map_err(err)?;
        }
        M::P0Attack => {
            let mut mine: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&x| g.is_creature(x)).collect();
            if mine.is_empty() {
                mine.push(perm(&mut g, P0, "Armored Warhorse")); // an enter effect took the person's creatures
            }
            for &x in &mine {
                g.perm_mut(x).sick = false;
            }
            g.active = Some(P0);
            g.step = Step::Combat;
            // the declaration goes through the attack taxes (Propaganda, Ghostly Prison ...) as a person's attack does
            let atk = combat::attack_limits(&mut g, P0, P1, mine.iter().copied().take(2).collect()).map_err(err)?;
            combat::attack_triggers(&mut g, P0, &atk, P1).map_err(err)?;
            stack::flush_triggers(&mut g).map_err(err)?;
            combat::resolve_combat(&mut g, P0, &atk, P1, &atk).map_err(err)?;
        }
        M::HeldCounter => {
            g.player_mut(P1).hand.push(c);
            // a spell the opponent would answer: an instant or sorcery for a counter that names those, else any
            let n = if text.contains("instant or sorcery") { "Demonic Tutor" } else { "Rhystic Study" };
            let bolt = take(&mut g, P0, n);
            g.player_mut(P0).hand.push(bolt);
            cast_card_of_p0 = Some(bolt);
            before = snap(&g);
            g.active = Some(P0);
            cast::cast_card(&mut g, P0, bolt, "hand".into(), Ctx::default()).map_err(err)?;
        }
        M::HandAbility(k) => {
            g.player_mut(P1).hand.push(c);
            let o = hand_opts(&mut g, c)?.into_iter().nth(k).ok_or("the hand ability is no longer offered")?;
            sim_core::ai::act::perform(&mut g, P1, o.act.as_ref().unwrap()).map_err(err)?;
        }
        M::Ability(k) => {
            let probe = probe.unwrap();
            g.step = Step::Main1;
            let opts = sim_core::ai::brain::main_options(&mut g, P1, false).map_err(err)?;
            let mine: Vec<_> =
                opts.into_iter().filter(|o| o.act.as_ref().is_some_and(|a| is_ability_of(a, probe))).collect();
            let o = mine.get(k).ok_or("the ability is no longer offered")?;
            sim_core::ai::act::perform(&mut g, P1, o.act.as_ref().unwrap()).map_err(err)?;
        }
    }
    stack::flush_triggers(&mut g).map_err(err)?;
    let after = snap(&g);
    if let Some(x) = cast_card_of_p0 {
        // the card the person cast left the hand: not a loss
        let mut b = before.clone();
        if let Some(i) = b.hand.iter().position(|&y| y == x) {
            b.hand.remove(i);
        }
        before = b;
    }
    let asked: Vec<String> = h.asked()[n_asked..]
        .iter()
        .filter(|r| r.kind != "priority")
        .map(|r| {
            if std::env::var("PROBE_CHOICES").is_ok() {
                format!("{}: {} {:?}", r.kind, r.prompt, r.choices)
            } else {
                format!("{}: {}", r.kind, r.prompt)
            }
        })
        .collect();
    let auto: Vec<String> = h.told()[n_told..].iter().filter(|t| &*t.0 == "auto").map(|t| t.1.clone()).collect();
    let mut lost = losses(&g, &before, &after, text);
    // a tax on the person that was never offered: the card's owner got what it gets when the tax isn't paid
    let taxy = [
        "that player pays",
        "controller pays",
        "unless they pay",
        "unless he or she pays",
        "unless target player pays",
        "unless any player pays",
    ]
    .iter()
    .any(|w| text.contains(w));
    let (b1, a1) = (before.p1, after.p1);
    if taxy && asked.is_empty() {
        let gained = a1[0] > b1[0] || a1[1] > b1[1] || a1[3] > b1[3] || a1[2] < b1[2];
        let on_p0 = matches!(
            m,
            M::P0CastSpell | M::P0CastCreature | M::P0CastInstant | M::P0Draw | M::P0DrawStep | M::P0Attack | M::P0Land
        );
        if (on_p0 && gained) || (m == M::HeldCounter && !g.player(P1).hand.contains(&c)) {
            lost.push(format!("tax never offered (P1 hand/permanents/treasures/library {b1:?} -> {a1:?})"));
        }
    }
    let in_play = match probe {
        Some(_) => true,
        None => {
            let on_bf = g
                .player(P1)
                .perms
                .iter()
                .chain(g.player(P1).lands.iter())
                .any(|&x| g.perm(x).cd == Some(c) && g.perm(x).on_bf);
            on_bf || g.player(P1).gy.contains(&c) || g.player(P1).exile.contains(&c) || !g.player(P1).hand.contains(&c)
        }
    };
    let log: Vec<String> =
        g.log.as_ref().map_or(vec![], |l| l[n_log.min(l.len())..].iter().map(|x| x.trim().to_string()).collect());
    Ok(Out { asked, auto, lost, log, in_play })
}

/// the activated abilities of the text a player could use against the person: not mana abilities, not Equip
fn activated(t: &str) -> Vec<String> {
    t.lines()
        .map(str::trim)
        .filter(|l| {
            let Some((cost, eff)) = l.split_once(": ") else { return false };
            let loyalty = cost.starts_with('+') || cost.starts_with('\u{2212}') || cost.starts_with('-') || cost == "0";
            (cost.contains('{') || loyalty) && !eff.starts_with("add ") && !l.starts_with("equip")
        })
        .map(String::from)
        .collect()
}

fn panic_text(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()
}

fn guarded<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::set_hook(Box::new(|_| {}));
    let r = catch_unwind(AssertUnwindSafe(f));
    let _ = std::panic::take_hook();
    match r {
        Ok(r) => r,
        Err(e) => Err(format!("panicked: {}", panic_text(e))),
    }
}

#[test]
fn every_opponent_effect_asks_the_person() {
    let cache = json("scryfall_cache.json");
    let decks = json("decks.json");
    let db = db();
    let tiers: BTreeSet<String> = decks["mine"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|d| d["tiers"].as_array().unwrap().iter().map(|t| t.as_str().unwrap().to_string()))
        .collect();
    // card -> the pool decks (of the tiers JD's decks face) that play it
    let mut where_: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut ndecks = 0;
    for d in decks["pool"].as_array().unwrap() {
        if !tiers.contains(d["tier"].as_str().unwrap()) {
            continue;
        }
        ndecks += 1;
        let key = d["key"].as_str().unwrap().to_string();
        for c in d["cards"].as_array().unwrap().iter().filter_map(|c| c.as_str()) {
            let v = where_.entry(c.to_string()).or_default();
            if !v.contains(&key) {
                v.push(key.clone());
            }
        }
    }
    let verbose = std::env::var("PROBE_VERBOSE").is_ok();
    let only = std::env::var("PROBE_ONLY").ok();
    let mut probed = 0;
    let mut runs = 0;
    let mut asked_somewhere = 0;
    let mut flagged: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut twice: BTreeMap<String, String> = BTreeMap::new();
    let mut unprobed: Vec<String> = vec![];
    let mut no_ability: Vec<String> = vec![];
    let mut autos: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, keys) in &where_ {
        if let Some(o) = &only
            && name != o
        {
            continue;
        }
        let Some(text) = oracle(&cache, name) else {
            unprobed.push(format!("{name}: not in the Scryfall cache"));
            continue;
        };
        if !relevant(&text) {
            continue;
        }
        let Some(cid) = db.id(name) else {
            unprobed.push(format!("{name}: not in data/cards.json"));
            continue;
        };
        probed += 1;
        let spell = db.get(cid).instant || db.get(cid).sorcery;
        let key1 = keys[0].as_str();
        let mut moments: Vec<M> = vec![M::Cast];
        if spell && text.contains("counter target") {
            moments.push(M::HeldCounter);
        }
        if !spell {
            moments.extend_from_slice(PERMANENT_MOMENTS);
            if let Ok(v) = guarded(|| hand_abilities(name, key1)) {
                moments.extend((0..v.len()).map(M::HandAbility));
            }
            match guarded(|| abilities(name, key1)) {
                Ok(v) => {
                    if v.is_empty() && !activated(&text).is_empty() {
                        no_ability.push(format!("{name}: {}", activated(&text).join(" / ")));
                    }
                    moments.extend((0..v.len()).map(M::Ability))
                }
                Err(e) => unprobed.push(format!("{name}: abilities not listed ({e})")),
            }
        }
        let mut ok_moments = 0;
        let mut why: Vec<String> = vec![];
        let mut cast_ok = false;
        for m in moments {
            runs += 1;
            match guarded(|| run(name, key1, &text, m)) {
                Err(e) => why.push(format!("{m:?}: {e}")),
                Ok(o) => {
                    ok_moments += 1;
                    if std::env::var("PROBE_LOG").is_ok() {
                        eprintln!("LOG {name} {m:?}: asked {:?}; lost {:?}; {}", o.asked, o.lost, o.log.join(" | "));
                    }
                    if m == M::Cast {
                        cast_ok = o.in_play;
                        if !o.in_play {
                            why.push("Cast: the card did not leave the hand".into());
                        }
                    }
                    if !o.asked.is_empty() {
                        asked_somewhere += 1;
                    }
                    if !o.auto.is_empty() {
                        autos.entry(name.clone()).or_default().push(format!("{m:?}: {}", o.auto.join(" | ")));
                    }
                    if o.asked.windows(2).any(|w| w[0] == w[1]) {
                        twice.insert(name.clone(), format!("{m:?}: {}", o.asked.join(" || ")));
                        flagged
                            .entry(name.clone())
                            .or_default()
                            .push(format!("{m:?}: asked twice: {}", o.asked.join(" || ")));
                    }
                    if o.asked.is_empty() && !o.lost.is_empty() {
                        let log: Vec<&str> = o
                            .log
                            .iter()
                            .map(String::as_str)
                            .filter(|l| {
                                l.contains("sacrific")
                                    || l.contains("discard")
                                    || l.contains("pay")
                                    || l.contains("exile")
                                    || l.contains("return")
                            })
                            .take(6)
                            .collect();
                        flagged.entry(name.clone()).or_default().push(format!(
                            "{m:?}: {}   log: {}",
                            o.lost.join("; "),
                            log.join(" | ")
                        ));
                    } else if verbose && !o.asked.is_empty() {
                        eprintln!("asked {name} {m:?}: {}", o.asked.join(" || "));
                    }
                }
            }
        }
        if ok_moments == 0 || !cast_ok {
            unprobed.push(format!("{name}: [{}] {}", keys.join(","), why.join(" ; ")));
        } else if verbose && !why.is_empty() {
            eprintln!("partly driven {name}: {}", why.join(" ; "));
        }
    }
    eprintln!(
        "{} opponent decks ({} cards); probed {probed} cards ({runs} runs); {} flagged; {} unprobed cards and {} cards with an activated ability the AI is never offered; {asked_somewhere} runs asked the person something",
        ndecks,
        where_.len(),
        flagged.len(),
        unprobed.len(),
        no_ability.len()
    );
    for (n, v) in &flagged {
        eprintln!("FLAG {n}  [{}]", where_[n].join(","));
        for x in v {
            eprintln!("     {x}");
        }
    }
    for (n, v) in &twice {
        eprintln!("TWICE {n}: {v}");
    }
    if verbose {
        for (n, v) in &autos {
            eprintln!("AUTO {n}: {}", v.join(" // "));
        }
    }
    for u in &no_ability {
        eprintln!("NOABILITY {u}");
    }
    for u in &unprobed {
        eprintln!("UNPROBED {u}");
    }
    if only.is_none() {
        check(&flagged.keys().cloned().collect::<Vec<_>>());
    }
}

/// gaps found on 2026-11-04 and not yet fixed: the AI chooses (or pays, or refuses) for the person. Fixing one means
/// taking it off this list; a new gap fails the test. Where the code is: `rust/agents/report-opponent-probe.md`.
const TO_FIX: &[&str] = &[
    "Archfiend of Depravity", // keeps the two best creatures for the person: impls/t3.rs depravity
    "Braids, Cabal Minion", // sacrifices the person's least valuable artifact, creature or land: ai/decks.rs braids_sacrifice
    "Burglar Rat",          // "each opponent discards": impls/t3.rs discard_worst_for (least worth)
    "Chittering Rats",      // puts the person's least worth card on top of the library: impls/t3.rs rats
    "Council's Judgment",   // the person's vote is cast by the AI: impls/rules.rs judgment
    "Elderfang Disciple",   // "each opponent discards": impls/t3.rs discard_worst_for
    "Liliana of the Veil",  // -6: the person's pile is chosen for them: impls/rules.rs lotv_ult
    "Liliana, Dreadhorde General", // -9: the permanents the person keeps are chosen for them: impls/rules.rs ldg_ult
    "Painful Quandary",     // discards a card (or loses 5 life) for the person: impls/rules.rs quandary
    "Rakdos, Patron of Chaos", // sacrifices two permanents instead of asking: impls/t12_kaalia_sythis.rs rakdos_end
    "Single Combat",        // keeps the person's best creature or planeswalker: impls/t12_lightpaws.rs
    "Sword of Feast and Famine", // combat damage: a random discard: engine/combat.rs
    "The Tabernacle at Pendrell Vale", // pays {1} or loses each creature for the person: engine/turn.rs upkeep
    "Wandering Archaic // Explore the Vastlands", // pays {2} for the person: impls/t45_chulane_kinnan.rs archaic_cast
];

/// listed by the probe but not a gap, each with why
const FALSE_ALARMS: &[(&str, &str)] = &[
    (
        "Chain of Vapor",
        "the bounce is P1's own target (Chandra). The controller's optional 'sacrifice a land to copy it' is never offered to the person at all: not modelled (a tag `rem` bounce), see the report",
    ),
    (
        "Dack Fayden, Helping Hand",
        "the donated All-Fates Stalker enters under P1, so its enter trigger is P1's and P1 chooses the person's creature to exile; the rules say so",
    ),
    ("Deathrite Shaman", "P1's ability targets a card in the person's graveyard: P1's choice"),
    (
        "Delayed Blast Fireball",
        "2 damage to each creature the person controls: a mass effect, the creatures with toughness 2 or less die, no choice",
    ),
    ("Duress", "P1 chooses the noncreature, nonland card from the revealed hand"),
    ("Grasp of Fate", "P1's target (exile until it leaves)"),
    ("Infectious Bite", "P1's targets (the creatures fight)"),
    (
        "Living Death",
        "no choice: every creature card in the graveyard is exiled, every creature sacrificed, all of them returned",
    ),
    ("Phyresis Outbreak", "-1/-1 for each poison counter on every creature the person controls: a mass effect"),
    ("Reality Acid", "P1 enchanted Chandra; the aura leaving makes its controller sacrifice that permanent: no choice"),
    ("Riptide Gearhulk", "P1's target (a planeswalker third from the top)"),
    ("Silent-Blade Oni", "P1 looks at the person's hand and casts a spell of P1's choice from it"),
    ("Static Prison", "P1's target (exile until it leaves)"),
    ("Teferi, Time Raveler", "-3: P1 targets the person's enchantment"),
    (
        "Torment of Hailfire",
        "X = 3: the person is asked three times, the same prompt each repetition (the three 'twice' hits). The prompt could say '1 of 3'",
    ),
    (
        "Vona's Hunger",
        "the city's blessing: half of four creatures rounded up is two sacrifices, so the person is asked twice with the same prompt. The prompt could say '1 of 2'",
    ),
    ("Vraska, Betrayal's Sting", "-2: P1 targets the person's creature"),
];

fn check(found: &[String]) {
    let found: BTreeSet<&str> = found.iter().map(String::as_str).collect();
    let listed = |s: &str| TO_FIX.contains(&s) || FALSE_ALARMS.iter().any(|(n, _)| *n == s);
    let new: Vec<&&str> = found.iter().filter(|s| !listed(s)).collect();
    let fixed: Vec<&str> =
        TO_FIX.iter().chain(FALSE_ALARMS.iter().map(|(n, _)| n)).copied().filter(|s| !found.contains(*s)).collect();
    for s in &new {
        eprintln!("NEW GAP: {s}");
    }
    for s in &fixed {
        eprintln!("NO LONGER FOUND (take it off the list): {s}");
    }
    eprintln!("{} found: {} to fix, {} false alarms", found.len(), TO_FIX.len(), FALSE_ALARMS.len());
    assert!(
        new.is_empty() && fixed.is_empty(),
        "{} new gap(s), {} list entr(ies) to remove: see above",
        new.len(),
        fixed.len()
    );
}
