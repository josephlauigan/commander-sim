//! The line planner (AI foundations step 2, `audit/ai-foundations/step2-planner.md`): a goal-directed search for the
//! Tier 6 decks (`t6_plans::T6_KEYS`), run before the utility AI in `brain::main` and in the end-of-turn window.
//!
//! The goals are the catalogued lines (`combos::all()`): a line is reached when it is ready, affordable (the rituals
//! in hand counted) and cleared by the table (`t6_table::go_ok`), exactly the test `combos::combo_options` makes before
//! it offers a line. The planner never invents a line, and it never reads a card it can't see.
//!
//! **The search.** A breadth-first search over own-turn plays, each made for real on a copy of the game
//! (`search::clone`), at most `MAX_DEPTH` plays deep and `MAX_NODES` copies a call. Only these plays are expanded:
//! - mana: a ritual that nets mana, a rock, a Mox (Mox Diamond, Chrome Mox), Lotus Petal, a mana creature (for next
//!   turn); lands and untapped mana permanents are paid from as any cast pays, Spirit Guides count as mana in hand;
//! - a land from hand while the land drop is still to come (a land a tutor found);
//! - a tutor, for a chosen card (`Player::plan_tutor`, which `decks::tutor_pick` and the ability language's search
//!   take when the card meets the tutor's own restriction): a missing piece of one of the deck's lines, or the mana a
//!   complete line lacks (`combos::plan_mana_targets`);
//! - a permanent piece of one of the deck's lines (Kinnan, Basalt Monolith, Underworld Breach, Lion's Eye Diamond),
//!   from hand or the command zone.
//!
//! A card the deck's plan slot keeps in hand now (`t6_plans::prio` of 0 or less: Dack Fayden while the table can
//! answer it) is never played. Draw spells are not expanded: their cards are unknown. Opponents never act in a copy (`Game::plan_for`): what the
//! table may do is `go_ok`'s job, as at the real table, and their hidden hands are never consulted.
//!
//! **This turn** (`plan_this_turn`): the shortest sequence after which a line is reached. Among those of one length:
//! the most protection kept in hand, then the most cards left.
//!
//! **Next turn** (`plan_next_turn`): when nothing reaches a line this turn, the play that makes a line ready and
//! affordable on the deck's next turn. Each position the search reached is carried to the next turn (`next_turn`):
//! everything untaps, summoning sickness ends, the card the deck knows is on top is drawn (no other draw is
//! counted), a land is played if one is in hand, and free mana artifacts in hand (Lotus Petal, a Mox) are cast.
//! The rituals and Spirit Guides in hand are counted by the line's own mana test. The positions are ranked by: a line
//! ready next turn, then the most protection kept, then the most cards left, then the fewest plays (the order the
//! search found them in breaks the last ties, so the choice is deterministic). Nothing is chosen when passing already
//! leaves a line ready next turn. A plan that starts with an instant tutor waits for the end-of-turn window: the main
//! phase keeps its mana up (`Decision::Reserve`) and the window casts it.
//!
//! `Settings::planner_progress` (env PLANNER_PROGRESS=1, off by default) adds a fallback: when nothing leaves a line
//! ready next turn, a tutor plan that brings the closest line nearer (fewer piece groups missing). It measured slower
//! (a tutor spent early can't fetch whichever piece is missing later, and a tutor to the top costs the draw).
//!
//! **The land drop** (`choose_land`): at the start of the turn each land in hand, and each land a fetch land can find,
//! is played on a copy and the plan read from there. The usual land choice stands unless another reaches a line
//! sooner (this turn, then next turn): the colours a line needs are what the usual choice misses.
//!
//! The planner runs only in the real game (not in look-ahead playouts) and never touches the real game's random
//! numbers: every play is made on a copy, which has its own.

use super::act;
use crate::engine::mana::{can_pay, cost_of, ritual_net};
use crate::flow::Res;
use crate::hooks::Action;
use crate::ids::{CardId, PlayerId};
use crate::impls::combos;
use crate::state::{Game, Step as TurnStep};
use crate::tag::Tag;

/// the most plays in a plan before its line (this turn) or its end (next turn)
pub const MAX_DEPTH: usize = 6;
/// the most game copies one planner call plays on (each a play made on a copy)
pub const MAX_NODES: usize = 120;
/// engine steps a copy may take for one play (a runaway play is dropped)
const PLAY_WORK: u64 = 4_000;
/// the most cards one tutor is tried for: the pieces first, then mana
const MAX_TARGETS: usize = 6;
/// the most mana cards a tutor is tried for
const MAX_MANA_TARGETS: usize = 3;

/// one play of a plan
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// cast this card (mana or a piece) from hand, or a commander from the command zone
    Cast(CardId),
    /// cast this tutor for that card
    Tutor(CardId, CardId),
    /// play this land from hand
    Land(CardId),
    /// go for this line (an index into `combos::all()`)
    Go(usize),
}

/// where a tutor puts the card it finds
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dest {
    Hand,
    Top,
}

/// what the planner wants done at a main-phase decision
#[derive(Clone, Debug, PartialEq)]
pub enum Decision {
    /// no plan: the utility AI decides, as before
    None,
    /// make this play now: the first of the plan (`why`: `WINS`, `READY` or `CLOSER`)
    Play { step: Step, plan: Vec<Step>, why: &'static str },
    /// the plan's first play is this instant tutor at the end of the turn before ours: keep its mana up
    Reserve { tutor: CardId, plan: Vec<Step> },
}

/// does the planner decide for p here: a Tier 6 deck, the planner on (`Settings::planner`), the real game (not a
/// look-ahead playout, not a planner copy), and not a person's seat
pub fn applies(g: &Game, p: PlayerId) -> bool {
    g.settings.planner
        && !g.in_search
        && g.plan_for.is_none()
        && crate::impls::t6_plans::is_t6(g, p)
        && !crate::human::is_human(g, p)
}

// ------------------------------------------------------------------ the plays
fn name(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// What tutor `c` can find for p and where it puts it: (where, the cards in p's library it may legally take). None for
/// a card the planner doesn't model as a tutor: creature tutors that tutor as they enter (Ranger-Captain), X tutors
/// (Chord of Calling, Finale: X is all the spare mana), pile tutors (Intuition, Gifts: an opponent chooses), Gamble (a
/// random discard), and land searches. The restriction is the one the card resolves with: its hand-written resolve,
/// else its ability-language search filter (Enlightened Tutor: an artifact or enchantment), else its `tut` tag.
pub fn tutor_targets(g: &Game, p: PlayerId, c: CardId) -> Option<(Dest, Vec<CardId>)> {
    let (dest, ok) = tutor_rule(g, c)?;
    let mut v: Vec<CardId> = vec![];
    for x in crate::engine::zones::searchable(g, p) {
        if ok(x) && !v.contains(&x) {
            v.push(x);
        }
    }
    Some((dest, v))
}

/// what kind of card tutor `c` finds (see `tutor_targets`): where it puts it, and whether a card qualifies
pub type TutorRule<'a> = (Dest, Box<dyn Fn(CardId) -> bool + 'a>);

/// the rule `tutor_targets` applies, card by card (the goldfish report's best case reads it too)
pub fn tutor_rule(g: &Game, c: CardId) -> Option<TutorRule<'_>> {
    let d = g.db.get(c);
    if d.land || d.perm || d.tag(Tag::Xtutor) {
        return None;
    }
    if g.registry.get(c).is_some_and(|i| i.resolve.is_some()) {
        // hand-written spells: the two that are plain tutors
        return match &*d.name {
            "Beseech the Mirror" => Some((Dest::Hand, Box::new(|_| true))),
            "Worldly Tutor" => Some((Dest::Top, Box::new(move |x| g.db.get(x).creature))),
            _ => None,
        };
    }
    if d.has_dsl() {
        use crate::dsl::model::{Ability, To};
        for a in d.abilities.iter() {
            let Ability::Spell { effects } = a else { continue };
            let Some(e) = effects.iter().find(|e| e.kind == "search") else { continue };
            let f = e.filter.clone().unwrap_or_default();
            if f.kind.as_deref() == Some("land") {
                return None; // land searches go to the battlefield or take a basic, not the chosen card
            }
            let dest = match &e.to {
                Some(To::Zone(z)) if z == "top" => Dest::Top,
                Some(To::Zone(z)) if z == "hand" => Dest::Hand,
                None => Dest::Hand,
                _ => return None,
            };
            return Some((dest, Box::new(move |x| crate::dsl::card_matches(g, x, Some(&f)))));
        }
        return None;
    }
    if d.tag(Tag::Seal) {
        return Some((Dest::Top, Box::new(|_| true))); // Vampiric Tutor, Imperial Seal
    }
    let k = d.tags.str(Tag::Tut)?;
    let dest = if d.tag(Tag::Top) { Dest::Top } else { Dest::Hand };
    Some((dest, Box::new(move |x| crate::ai::decks::tutor_ok(g, k, x))))
}

/// a card that makes mana for the planner: a ritual that nets mana, a rock, a Mox, Lotus Petal, or a mana creature
fn mana_card(g: &Game, p: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    if d.land {
        return false;
    }
    if d.instant || d.sorcery {
        return ritual_net(g, p, c) > 0;
    }
    let t = &d.tags;
    if d.creature {
        return t.has(Tag::Dork);
    }
    t.has(Tag::Rock) || t.has(Tag::Moxd) || t.has(Tag::Chromemox)
}

/// the cards the deck knows it will draw next: its own known top cards, top last
fn known_top(g: &Game, p: PlayerId) -> Vec<CardId> {
    crate::engine::tutors::known_top(g, p)
}

/// The cards worth a tutor for p now: a card of each piece group of p's lines that p doesn't hold (in hand, on the
/// battlefield, among its lands, as a commander in the command zone), for the lines missing at most as many groups as
/// p holds tutors; then the mana a complete line lacks. The pieces first, by line order.
fn tutor_wants(g: &Game, p: PlayerId, tutors: usize) -> (Vec<CardId>, Vec<CardId>) {
    let lib = crate::engine::zones::searchable(g, p);
    let mut pieces: Vec<CardId> = vec![];
    for i in combos::audit_lines(g, p) {
        let groups = combos::audit_groups(i);
        let missing: Vec<&&[&str]> = groups.iter().filter(|grp| !combos::group_held(g, p, grp)).collect();
        // one group more than the tutors in hand: a tutor now brings the line nearer (`Settings::planner_progress`)
        let reach = tutors + usize::from(g.settings.planner_progress);
        if missing.is_empty() || missing.len() > reach {
            continue;
        }
        for grp in missing {
            for &x in &lib {
                if grp.contains(&name(g, x)) && !pieces.iter().any(|&y| name(g, y) == name(g, x)) {
                    pieces.push(x);
                }
            }
        }
    }
    let mut mana: Vec<CardId> = vec![];
    for x in combos::plan_mana_targets(g, p) {
        if !mana.iter().any(|&y| name(g, y) == name(g, x)) && !pieces.contains(&x) {
            mana.push(x);
        }
    }
    (pieces, mana)
}

/// the land drop is still to come on p's own turn
fn land_drop_left(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    g.active == Some(p) && matches!(g.step, TurnStep::Main1 | TurnStep::Main2) && pl.land_turn != pl.turns as i32
}

/// The plays the planner tries from this position, in a fixed order: tutors (each for its wanted cards), pieces, mana,
/// then lands. Only cards in `known` (the hand at the start of the plan, and what its tutors found) are played.
/// `eot`: the end-of-turn window before p's turn: instants only.
fn plays(g: &Game, p: PlayerId, known: &[CardId], eot: bool) -> Vec<Step> {
    let pl = g.player(p);
    let castable_now = |c: CardId, zone: &str| -> bool {
        let d = g.db.get(c);
        if eot && !(d.instant || d.tag(Tag::Flash)) {
            return false;
        }
        // the deck's plan slot keeps this card in hand now (Dack waits for the table's risk to drop, or for Silence)
        if crate::impls::t6_plans::prio(g, p, c).is_some_and(|x| x <= 0.0) {
            return false;
        }
        if g.cast_checks() && !crate::engine::cast::castable(g, p, c, zone) {
            return false;
        }
        let (gn, pips) = cost_of(g, p, c);
        let tax = if zone == "cmd" { pl.tax_of(c) } else { 0 };
        can_pay(g, p, gn + tax, &pips, d.tag(Tag::Convoke))
    };
    let hand: Vec<CardId> = pl.hand.iter().copied().filter(|c| known.contains(c)).collect();
    let tutors: Vec<(CardId, Dest, Vec<CardId>)> =
        hand.iter().filter_map(|&c| tutor_targets(g, p, c).map(|(d, t)| (c, d, t))).collect();
    let mut out = vec![];
    if !tutors.is_empty() {
        let (pieces, mana) = tutor_wants(g, p, tutors.len());
        let mut tried: Vec<&str> = vec![];
        for (c, _, legal) in &tutors {
            if tried.contains(&name(g, *c)) || !castable_now(*c, "hand") {
                continue; // a second copy of the same tutor finds the same cards
            }
            tried.push(name(g, *c));
            let mut n = 0;
            for &t in pieces.iter().chain(mana.iter().take(MAX_MANA_TARGETS)) {
                if n < MAX_TARGETS && legal.contains(&t) {
                    out.push(Step::Tutor(*c, t));
                    n += 1;
                }
            }
        }
    }
    let mut piece_names: Vec<&str> = vec![];
    for i in combos::audit_lines(g, p) {
        for grp in combos::audit_groups(i) {
            if !combos::group_out(g, p, grp) {
                piece_names.extend(grp.iter().copied());
            }
        }
    }
    let mut cands: Vec<(CardId, &str)> = hand.iter().map(|&c| (c, "hand")).collect();
    cands.extend(pl.cmds_in_zone().map(|c| (c, "cmd")));
    let mut seen: Vec<&str> = vec![];
    for &(c, zone) in &cands {
        let d = g.db.get(c);
        // Thassa's Oracle is cast by its line (alone, it only looks at the top cards)
        let piece = d.perm && !d.land && piece_names.contains(&&*d.name) && &*d.name != "Thassa's Oracle";
        if !piece || seen.contains(&&*d.name) || !castable_now(c, zone) {
            continue;
        }
        seen.push(&d.name);
        out.push(Step::Cast(c));
    }
    for &c in &hand {
        let d = g.db.get(c);
        if seen.contains(&&*d.name) || !mana_card(g, p, c) || !castable_now(c, "hand") {
            continue;
        }
        if d.tag(Tag::Moxd) && !pl.hand.iter().any(|&x| g.db.get(x).land) {
            continue; // Mox Diamond with no land to discard goes to the graveyard
        }
        seen.push(&d.name);
        out.push(Step::Cast(c));
    }
    if !eot && land_drop_left(g, p) {
        let mut lands: Vec<&str> = vec![];
        for &c in &hand {
            if g.db.get(c).land && !lands.contains(&name(g, c)) {
                lands.push(name(g, c));
                out.push(Step::Land(c));
            }
        }
    }
    out
}

/// play a land from p's hand (the planner's land drop): `arg` is the card
fn play_land_plan(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let pl = g.player(p);
    if !pl.hand.contains(&c) || pl.land_turn == pl.turns as i32 {
        return Ok(false);
    }
    let pl = g.player_mut(p);
    let i = pl.hand.iter().position(|&x| x == c).unwrap();
    pl.hand.remove(i);
    crate::engine::turn::play_land_card(g, p, c, "plays")?;
    Ok(true)
}

/// make one play of a plan, in a copy or in the real game: true if it was made
pub fn perform(g: &mut Game, p: PlayerId, s: Step) -> Res<bool> {
    match s {
        Step::Cast(c) => act::perform(g, p, &Action::Cast { card: c, zone: None }),
        Step::Tutor(c, t) => {
            g.player_mut(p).plan_tutor = Some(t);
            let r = act::perform(g, p, &Action::Cast { card: c, zone: None });
            g.player_mut(p).plan_tutor = None;
            r
        }
        Step::Land(c) => play_land_plan(g, p, c.0 as i64),
        Step::Go(i) => combos::attempt(g, p, i as i64),
    }
}

/// a play as the trace shows it
pub fn label(g: &Game, s: Step) -> String {
    match s {
        Step::Cast(c) => name(g, c).to_string(),
        Step::Tutor(c, t) => format!("{} for {}", name(g, c), name(g, t)),
        Step::Land(c) => format!("play {}", name(g, c)),
        Step::Go(i) => format!("combo: {}", combos::audit_name(i)),
    }
}

/// a whole plan as the trace shows it
pub fn describe(g: &Game, plan: &[Step]) -> String {
    plan.iter().map(|&s| label(g, s)).collect::<Vec<_>>().join(" -> ")
}

// ------------------------------------------------------------------ the search
/// a position the search reached: the copy, the plays that led to it, and the cards p may use (those it saw)
struct Node {
    g: Game,
    steps: Vec<Step>,
    known: Vec<CardId>,
}

/// a copy of g for the planner: no look-ahead, nobody but p acts, its own budget of engine steps
fn copy(g: &mut Game, p: PlayerId) -> Game {
    let mut g2 = super::search::clone(g);
    g2.in_search = true;
    g2.plan_for = Some(p);
    g2.board_cap = None;
    g2.work_cap = g2.work + PLAY_WORK;
    g2
}

/// a copy of a planner copy (the log and audit are already gone)
fn copy_on(g: &Game) -> Game {
    let mut g2 = g.clone();
    g2.work_cap = g2.work + PLAY_WORK;
    g2
}

/// a fingerprint of p's side of the position: two orders of the same plays reach the same one
fn signature(g: &Game, p: PlayerId) -> u64 {
    use std::hash::{Hash, Hasher};
    let pl = g.player(p);
    let mut h = std::collections::hash_map::DefaultHasher::new();
    let mut hand: Vec<u16> = pl.hand.iter().map(|c| c.0).collect();
    hand.sort_unstable();
    hand.hash(&mut h);
    let mut perms: Vec<(u16, bool)> =
        pl.perms.iter().filter_map(|&m| g.perm(m).cd.map(|c| (c.0, g.perm(m).tapped))).collect();
    perms.sort_unstable();
    perms.hash(&mut h);
    let mut lands: Vec<(u16, bool)> = pl.lands.iter().map(|&l| (g.land(l).cd.0, g.land(l).tapped)).collect();
    lands.sort_unstable();
    lands.hash(&mut h);
    let f = pl.floating;
    (f.w, f.u, f.b, f.r, f.g, f.c, f.any, f.art).hash(&mut h);
    known_top(g, p).iter().map(|c| c.0).collect::<Vec<_>>().hash(&mut h);
    (pl.life, pl.gy.len(), pl.library.len(), pl.treasures, pl.land_turn).hash(&mut h);
    h.finish()
}

/// the protection cards in p's hand (a counterspell, a protection spell, Silence / Orim's Chant)
fn protection(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.iter().filter(|&&c| crate::goldfish::protection_card(&g.db, c)).count()
}

/// the first line p reaches here with cards it knows: ready, affordable and cleared to go (`table`), or ready and
/// affordable only
fn reached(g: &Game, p: PlayerId, known: &[CardId], table: bool) -> Option<usize> {
    combos::plan_lines(g, p, table).into_iter().find(|(_, cards)| cards.iter().all(|c| known.contains(c))).map(|x| x.0)
}

/// Carry a planner copy to p's next turn, as `plan_next_turn` reads it: floating mana empties, p's lands and
/// permanents untap (not those that don't untap), summoning sickness ends, the card p knows is on top is drawn (no
/// other: the draws are unknown; none under Necropotence), a land from hand is played, and free mana artifacts in
/// hand (Lotus Petal, a Mox) are cast. Opponents' turns are not played. Returns the card drawn.
pub fn next_turn(g: &mut Game, p: PlayerId) -> Res<Option<CardId>> {
    use crate::engine::values::card_tag;
    for q in &mut g.players {
        q.floating = Default::default();
    }
    g.active = Some(p);
    g.step = TurnStep::Main1;
    g.round += 1;
    let pl = g.player_mut(p);
    pl.turns += 1;
    pl.spells_this_turn = 0;
    pl.lands_played = 0;
    pl.yawg = false;
    pl.tapped_treasures = 0;
    for l in g.player(p).lands.clone() {
        g.land_mut(l).tapped = false;
    }
    for m in g.player(p).perms.clone() {
        let keep = card_tag(g, m, Tag::Nountap);
        let x = g.perm_mut(m);
        if !keep {
            x.tapped = false;
        }
        x.sick = false;
        x.phased = false;
    }
    let mut drew = None;
    if !crate::engine::values::has(g, p, Tag::Necro) && !crate::cardcode::skip_draw(g, p) {
        let top = known_top(g, p);
        if let Some(&c) = top.last() {
            let pl = g.player_mut(p);
            pl.library.pop();
            pl.hand.push(c);
            drew = Some(c);
        }
    }
    crate::engine::turn::play_land(g, p)?;
    for c in g.player(p).hand.clone() {
        let d = g.db.get(c);
        let free = !d.creature && !d.land && (d.tags.has(Tag::Rock) || d.tag(Tag::Moxd) || d.tag(Tag::Chromemox));
        if !free || cost_of(g, p, c) != (0, String::new()) {
            continue;
        }
        if d.tag(Tag::Moxd) && !g.player(p).hand.iter().any(|&x| g.db.get(x).land) {
            continue;
        }
        if g.player(p).hand.contains(&c) {
            act::perform(g, p, &Action::Cast { card: c, zone: None })?;
        }
    }
    Ok(drew)
}

/// how far p's closest line is: the fewest piece groups p doesn't hold, over its deck's lines (`usize::MAX`: none)
pub fn distance(g: &Game, p: PlayerId) -> usize {
    combos::audit_lines(g, p)
        .into_iter()
        .map(|i| combos::audit_groups(i).iter().filter(|grp| !combos::group_held(g, p, grp)).count())
        .min()
        .unwrap_or(usize::MAX)
}

/// this position carried to p's next turn (`next_turn`): is a line ready and affordable with cards p knows, and how
/// far is the closest line (`distance`)
fn view_next(node: &Node, p: PlayerId) -> (bool, usize) {
    let mut g2 = copy_on(&node.g);
    let Ok(drew) = next_turn(&mut g2, p) else { return (false, usize::MAX) };
    if g2.over {
        return (false, usize::MAX);
    }
    let mut known = node.known.clone();
    known.extend(drew);
    (reached(&g2, p, &known, false).is_some(), distance(&g2, p))
}

/// what the search found: a plan that reaches a line this turn, else one that leaves a line ready next turn
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Found {
    pub this_turn: Option<Vec<Step>>,
    pub next_turn: Option<Vec<Step>>,
    /// the next-turn plan leaves no line ready, only the closest line nearer (`Settings::planner_progress`)
    pub closer: bool,
    /// passing already leaves a line ready next turn (no next-turn plan is needed)
    pub pass_ready: bool,
    /// copies played on
    pub nodes: usize,
}

/// The search (see the module notes). `this_turn`: look for a line reached this turn first; `next`: then, if none, for
/// the play that leaves one ready next turn. `eot`: the end-of-turn window before p's turn (instants only, no line this
/// turn).
pub fn search(g: &mut Game, p: PlayerId, this_turn: bool, next: bool, eot: bool) -> Res<Found> {
    let root = copy(g, p);
    let mut known: Vec<CardId> = root.player(p).hand.clone();
    known.extend(root.player(p).cmds_in_zone());
    let mut found = Found::default();
    let mut seen = std::collections::HashSet::new();
    seen.insert(signature(&root, p));
    let mut level = vec![Node { g: root, steps: vec![], known }];
    let mut all: Vec<Node> = vec![];
    let this_turn = this_turn && !eot;
    for depth in 0..=MAX_DEPTH {
        if this_turn {
            let mut best: Option<(usize, usize, usize, usize)> = None; // (protection, cards, node, line)
            for (k, n) in level.iter().enumerate() {
                if let Some(i) = reached(&n.g, p, &n.known, true) {
                    let key = (protection(&n.g, p), n.g.player(p).hand.len());
                    if best.is_none_or(|b| key > (b.0, b.1)) {
                        best = Some((key.0, key.1, k, i));
                    }
                }
            }
            if let Some((_, _, k, i)) = best {
                let mut steps = level[k].steps.clone();
                steps.push(Step::Go(i));
                found.this_turn = Some(steps);
                return Ok(found);
            }
        }
        if depth == MAX_DEPTH || found.nodes >= MAX_NODES {
            all.append(&mut level);
            break;
        }
        let mut next_level = vec![];
        'expand: for n in &level {
            for s in plays(&n.g, p, &n.known, eot) {
                if found.nodes >= MAX_NODES {
                    break 'expand;
                }
                found.nodes += 1;
                let mut g2 = copy_on(&n.g);
                let ok = matches!(perform(&mut g2, p, s), Ok(true));
                if !ok || g2.over || !g2.player(p).alive {
                    continue;
                }
                if !seen.insert(signature(&g2, p)) {
                    continue;
                }
                let mut steps = n.steps.clone();
                steps.push(s);
                let mut known = n.known.clone();
                if let Step::Tutor(_, t) = s {
                    known.push(t);
                }
                next_level.push(Node { g: g2, steps, known });
            }
        }
        all.append(&mut level);
        if next_level.is_empty() {
            break;
        }
        level = next_level;
    }
    if !next || all.is_empty() {
        return Ok(found);
    }
    let (ready0, dist0) = view_next(&all[0], p);
    if ready0 {
        found.pass_ready = true;
        return Ok(found); // passing already leaves a line ready next turn
    }
    // the plans that leave a line ready next turn, by protection kept, then cards left; else (`Settings::
    // planner_progress`) the tutor plans that bring the closest line nearer, nearest first, then by the same keys.
    // BFS order: an earlier (shorter) plan wins the ties
    let progress = g.settings.planner_progress;
    let mut best: Option<((bool, usize, usize, usize), usize)> = None;
    for (k, n) in all.iter().enumerate().skip(1) {
        let tutored = n.steps.iter().any(|s| matches!(s, Step::Tutor(..)));
        let (ready, dist) = view_next(n, p);
        if !ready && !(progress && tutored && dist < dist0) {
            continue;
        }
        let near = if ready { usize::MAX } else { usize::MAX - dist };
        let key = (ready, near, protection(&n.g, p), n.g.player(p).hand.len());
        if best.is_none_or(|b| key > b.0) {
            best = Some((key, k));
        }
    }
    found.next_turn = best.map(|b| all[b.1].steps.clone());
    found.closer = best.is_some_and(|b| !b.0.0);
    Ok(found)
}

/// the shortest plan that reaches a line this turn (its last play is `Step::Go`), if any
pub fn plan_this_turn(g: &mut Game, p: PlayerId) -> Res<Option<Vec<Step>>> {
    Ok(search(g, p, true, false, false)?.this_turn)
}

/// the plan this turn that leaves a line ready next turn, when nothing reaches one this turn and passing doesn't
/// (`eot`: in the end-of-turn window before p's turn)
pub fn plan_next_turn(g: &mut Game, p: PlayerId, eot: bool) -> Res<Option<Vec<Step>>> {
    Ok(search(g, p, false, true, eot)?.next_turn)
}

/// an instant tutor: in the main phase a next-turn plan that starts with one keeps its mana for the end-of-turn
/// window instead (nothing is gained by casting it now)
fn instant_tutor(g: &Game, s: Step) -> Option<CardId> {
    match s {
        Step::Tutor(c, _) if g.db.get(c).instant => Some(c),
        _ => None,
    }
}

/// the planner's decision at a main-phase decision of p's (`brain::main`)
pub fn decide(g: &mut Game, p: PlayerId) -> Res<Decision> {
    let f = search(g, p, true, true, false)?;
    if let Some(plan) = f.this_turn {
        return Ok(Decision::Play { step: plan[0], plan, why: WINS });
    }
    let Some(plan) = f.next_turn.filter(|v| !v.is_empty()) else { return Ok(Decision::None) };
    if let Some(t) = instant_tutor(g, plan[0]) {
        return Ok(Decision::Reserve { tutor: t, plan });
    }
    Ok(Decision::Play { step: plan[0], plan, why: if f.closer { CLOSER } else { READY } })
}

/// why a plan is made, as the trace says it
pub const WINS: &str = "wins this turn";
pub const READY: &str = "ready next turn";
pub const CLOSER: &str = "a line nearer next turn";

/// the planner's play in the end-of-turn window before p's turn: the first play of a plan that leaves a line ready
/// next turn, or nearer (an instant tutor, or the instant ritual that pays for it)
pub fn eot_play(g: &mut Game, p: PlayerId) -> Res<Option<(Step, Vec<Step>, &'static str)>> {
    let f = search(g, p, false, true, true)?;
    let Some(plan) = f.next_turn.filter(|v| !v.is_empty()) else { return Ok(None) };
    Ok(Some((plan[0], plan, if f.closer { CLOSER } else { READY })))
}

/// The main phase keeps the reserved tutor's mana up: an option is dropped when the tutor can't be paid for after it
/// (a cast by its cost; any other play by making it on a copy), and the tutor itself is not cast now (the
/// end-of-turn window casts it, for the planner's card).
pub fn reserve(g: &mut Game, p: PlayerId, tutor: CardId, opts: Vec<crate::hooks::Opt>) -> Vec<crate::hooks::Opt> {
    let (tg, tp) = cost_of(g, p, tutor);
    let mut out = vec![];
    for o in opts {
        let keep = match &o.act {
            None => true,
            Some(Action::Cast { card, .. }) if *card == tutor => false,
            Some(Action::Cast { card, .. }) => {
                let (gn, pips) = crate::engine::mana::seq_cost(g, p, &[*card, tutor]);
                can_pay(g, p, gn, &pips, false)
            }
            Some(a) => {
                let mut g2 = copy(g, p);
                matches!(act::perform(&mut g2, p, a), Ok(true))
                    && g2.player(p).hand.contains(&tutor)
                    && can_pay(&g2, p, tg, &tp, false)
            }
        };
        if keep {
            out.push(o);
        }
    }
    out
}

// ------------------------------------------------------------------ the land drop
/// play land `c` from p's hand, and when it is a fetch land that cracks, fetch `t` (the planner's land drop)
pub fn play_land_for(g: &mut Game, p: PlayerId, c: CardId, t: Option<CardId>) -> Res {
    let pl = g.player_mut(p);
    let Some(i) = pl.hand.iter().position(|&x| x == c) else { return Ok(()) };
    pl.hand.remove(i);
    pl.plan_tutor = t;
    let r = crate::engine::turn::play_land_card(g, p, c, "plays");
    g.player_mut(p).plan_tutor = None;
    r
}

/// how good a position is for the plan, for the land drop: a line reached this turn (the shorter plan first), else one
/// ready next turn, else the closest line nearer next turn
fn land_score(g: &mut Game, p: PlayerId) -> Res<(u8, usize)> {
    let f = search(g, p, true, true, false)?;
    Ok(match (&f.this_turn, &f.next_turn) {
        (Some(v), _) => (3, usize::MAX - v.len()),
        _ if f.pass_ready => (2, usize::MAX),
        (_, Some(v)) if !f.closer => (2, usize::MAX - v.len()),
        (_, Some(_)) => (1, 0),
        _ => (0, 0),
    })
}

/// The planner's land drop at the start of p's turn: each land in hand (each land a fetch land can find, for a fetch
/// land) is played on a copy and the plan read from there (`land_score`). None when the usual choice
/// (`turn::play_land`) scores as well as the best, or there is no choice: the usual choice stands.
pub fn choose_land(g: &mut Game, p: PlayerId) -> Res<Option<(CardId, Option<CardId>)>> {
    let pl = g.player(p);
    let mut lands: Vec<CardId> = vec![];
    for &c in &pl.hand {
        let d = g.db.get(c);
        if d.land && !d.tag(Tag::Chasm) && !lands.iter().any(|&x| name(g, x) == &*d.name) {
            lands.push(c);
        }
    }
    let mut options: Vec<(CardId, Option<CardId>)> = vec![];
    for &c in &lands {
        let fetch = g.db.get(c).tag(Tag::F).then(|| crate::engine::turn::true_fetch(name(g, c))).flatten();
        match fetch.filter(|k| *k != "basic") {
            Some(k) => {
                let ts = crate::cards::Colors::from_letters(k);
                let mut seen: Vec<&str> = vec![];
                for x in crate::engine::zones::searchable(g, p) {
                    let d = g.db.get(x);
                    if d.land && d.land_types.intersects(ts) && !seen.contains(&&*d.name) {
                        seen.push(&d.name);
                        options.push((c, Some(x)));
                    }
                }
                if seen.is_empty() {
                    options.push((c, None));
                }
            }
            None => options.push((c, None)),
        }
    }
    if options.len() < 2 {
        return Ok(None);
    }
    let main1 = |g2: &mut Game| {
        g2.step = TurnStep::Main1;
        g2.active = Some(p);
    };
    // the usual choice, made on a copy (its own random numbers): its score is the bar to beat
    let mut d = copy(g, p);
    crate::engine::turn::play_land(&mut d, p)?;
    main1(&mut d);
    let bar = land_score(&mut d, p)?;
    let mut best: Option<((u8, usize), (CardId, Option<CardId>))> = None;
    for &(c, t) in &options {
        let mut g2 = copy(g, p);
        if play_land_for(&mut g2, p, c, t).is_err() || g2.over {
            continue;
        }
        main1(&mut g2);
        let s = land_score(&mut g2, p)?;
        if s > bar && best.is_none_or(|b| s > b.0) {
            best = Some((s, (c, t)));
        }
    }
    if let Some((_, (c, t))) = best {
        let what = t.map_or(String::new(), |t| format!(" for {}", name(g, t)));
        crate::glog!(
            g,
            "      [{} plays {}{} [plan: the land drop the plan wants]]",
            g.player(p).name,
            name(g, c),
            what
        );
    }
    Ok(best.map(|b| b.1))
}
