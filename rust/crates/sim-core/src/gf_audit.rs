//! --- ai-audit: loss accounting for goldfish games (`commander-sim --goldfish`). Stats only: nothing here reads the
//! random generator or changes a choice, and nothing runs outside a goldfish game (`Game::audit` is None) or in a
//! look-ahead copy (`search::clone` leaves the copy without it).
//!
//! For the deck under test it records, on each of its own turns: whether a catalogued line was ready (ready and
//! affordable, as `t6_measure` reads it) at a main-phase decision and whether the deck went for one, the castable
//! nonland cards still in hand when each main phase ended, the land drop, what it cast; for the whole game, every card
//! it saw (the kept hand and its draws; tutored cards aside), its casts, the cleanup discards and the "stop" choices.
//! `report` classifies each own turn into one bucket (`Bucket`, first match wins) and works out the crude best case.

use crate::cards::CardDb;
use crate::hooks::{Action, Opt};
use crate::ids::{CardId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

/// what one own turn of the deck under test did (raw, classified by `report`)
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TurnRec {
    pub turn: u32,
    /// a line that was ready and affordable at some main-phase decision this turn (the first one found)
    pub ready: Option<&'static str>,
    /// the deck went for a line this turn (`combos::attempt`, or the end-of-turn window after it)
    pub went: bool,
    /// castable nonland cards still in hand as a main phase ended (main 1 and main 2, deduplicated)
    pub passed: Vec<CardId>,
    /// a land card was in hand as a main phase ended
    pub land_left: bool,
    /// a land was played this turn
    pub land_played: bool,
    /// a spell was castable (hand or command zone) at some main-phase decision
    pub castable: bool,
    /// a nonland card was in hand (or the commander in the command zone) at some main-phase decision
    pub nonland: bool,
    /// a nonland card in hand (or the commander) had a cost the deck could pay at some main-phase decision
    pub payable: bool,
    /// payable nonland cards in hand that were not offered as a play (held by a rule, instants, a 0 priority)
    pub held: Vec<CardId>,
    /// the cards it cast on this turn (its own turn)
    pub cast: Vec<CardId>,
    /// it cast, played or activated something this turn
    pub acted: bool,
}

/// one "stop" chosen at a main-phase decision
#[derive(Debug, Clone, PartialEq)]
pub struct StopRec {
    pub turn: u32,
    /// the look-ahead picked it (else the heuristic's sample)
    pub search: bool,
    /// "stop" or "stop (hold mana)"
    pub label: String,
    /// the best other option by heuristic utility: (label, utility); None when stop was the only option
    pub alt: Option<(String, f64)>,
    /// the look-ahead's scores (per playout): (stop, the best other option)
    pub scores: Option<(f64, f64)>,
    /// a line was ready at this decision
    pub ready: bool,
    /// in the second main phase (the turn's last decision); else the first (combat and main 2 still to come)
    pub post: bool,
}

/// one cleanup discard to hand size
#[derive(Debug, Clone, PartialEq)]
pub struct DiscardRec {
    pub card: CardId,
    pub turn: u32,
    /// `ai::discard_keep` lists it (a combo piece or a wish-list card the engine's discard ignores)
    pub kept: bool,
    /// cards in hand before the discard
    pub hand: usize,
}

#[derive(Debug, Clone)]
pub struct Audit {
    pub me: PlayerId,
    pub turns: Vec<TurnRec>,
    /// every card the deck saw, with the first own turn it could use it: the kept hand (turn 1), then each draw
    pub seen: Vec<(CardId, u32)>,
    /// its casts: (card, own turn count at the cast)
    pub casts: Vec<(CardId, u32)>,
    pub discards: Vec<DiscardRec>,
    pub stops: Vec<StopRec>,
    /// the chosen play at decisions where a line was ready and the deck had not gone for one this turn
    pub ready_choices: Vec<String>,
    /// look-ahead decisions where the sure-win cap (d10bbb05) changed the pick
    pub cap_changed: u32,
    /// the castable cards from hand at the last main-phase decision (main_end turns them into `passed`)
    last_castable: Vec<CardId>,
    /// a line is ready at the decision being taken
    ready_now: bool,
}

fn audit_of(g: &mut Game, p: PlayerId) -> Option<&mut Audit> {
    if g.in_search {
        return None;
    }
    g.audit.as_deref_mut().filter(|a| a.me == p)
}

fn cur(g: &mut Game, p: PlayerId) -> Option<&mut TurnRec> {
    audit_of(g, p).and_then(|a| a.turns.last_mut())
}

/// a goldfish game is set up: the deck under test is the first seat that is not inert; its kept hand is seen on turn 1
pub fn start(g: &mut Game) {
    if !crate::goldfish::is_goldfish(g) {
        return;
    }
    let Some(me) = (0..g.players.len()).map(|i| PlayerId(i as u8)).find(|&p| !crate::goldfish::is_inert(g, p)) else {
        return;
    };
    let seen = g.player(me).hand.iter().map(|&c| (c, 1)).collect();
    g.audit = Some(Box::new(Audit {
        me,
        turns: vec![],
        seen,
        casts: vec![],
        discards: vec![],
        stops: vec![],
        ready_choices: vec![],
        cap_changed: 0,
        last_castable: vec![],
        ready_now: false,
    }));
}

/// the first own turn on which p can use a card it gets now: this turn on its own turn, else its next
fn use_turn(g: &Game, p: PlayerId) -> u32 {
    g.player(p).turns + u32::from(g.active != Some(p))
}

pub fn turn_start(g: &mut Game, p: PlayerId) {
    let t = g.player(p).turns;
    if let Some(a) = audit_of(g, p) {
        a.turns.push(TurnRec { turn: t, ..Default::default() });
    }
}

pub fn draw(g: &mut Game, p: PlayerId, c: CardId) {
    let t = use_turn(g, p);
    if let Some(a) = audit_of(g, p) {
        a.seen.push((c, t));
    }
}

pub fn cast(g: &mut Game, p: PlayerId, c: CardId) {
    let t = g.player(p).turns;
    let own = g.active == Some(p);
    if let Some(a) = audit_of(g, p) {
        a.casts.push((c, t));
        if own && let Some(r) = a.turns.last_mut() {
            r.acted = true;
            r.cast.push(c);
        }
    }
}

pub fn land_played(g: &mut Game, p: PlayerId) {
    if g.active == Some(p)
        && let Some(r) = cur(g, p)
    {
        r.land_played = true;
        r.acted = true;
    }
}

/// p goes for a line (combos::attempt)
pub fn went(g: &mut Game, p: PlayerId) {
    if let Some(r) = cur(g, p) {
        r.went = true;
    }
}

fn spell_card(o: &Opt) -> Option<CardId> {
    match o.act.as_ref()? {
        Action::Cast { card, .. }
        | Action::Removal { card, .. }
        | Action::Wipe { card, .. }
        | Action::Face { card, .. }
        | Action::Crackle { card, .. } => Some(*card),
        _ => None,
    }
}

/// a main-phase decision of p with these options (before the choice)
pub fn decision(g: &mut Game, p: PlayerId, opts: &[Opt]) {
    if g.active != Some(p) || audit_of(g, p).is_none() {
        return;
    }
    let ready = crate::impls::combos::audit_ready(g, p);
    let pl = g.player(p);
    let castable: Vec<CardId> = opts.iter().filter_map(spell_card).filter(|c| pl.hand.contains(c)).collect();
    let any_cast = opts.iter().any(|o| spell_card(o).is_some());
    let nonland = pl.hand.iter().any(|&c| !g.db.get(c).land) || pl.cmds_in_zone().next().is_some();
    let pay = |c: CardId| {
        let (gn, pips) = crate::engine::mana::cost_of(g, p, c);
        let tax = if pl.is_commander(c) { pl.tax_of(c) } else { 0 };
        crate::engine::mana::can_pay(g, p, gn + tax, &pips, false)
    };
    let offered: Vec<CardId> = opts.iter().filter_map(spell_card).collect();
    let mut payable = false;
    let mut held: Vec<CardId> = vec![];
    for c in pl.hand.iter().copied().chain(pl.cmds_in_zone()) {
        if g.db.get(c).land || !pay(c) {
            continue;
        }
        // an instant the AI did not offer (a counterspell, a protection spell) is no play for the main phase
        payable |= !g.db.get(c).instant || offered.contains(&c);
        if pl.hand.contains(&c) && !offered.contains(&c) {
            held.push(c);
        }
    }
    let a = audit_of(g, p).unwrap();
    a.last_castable = castable;
    let gone = a.turns.last().is_some_and(|r| r.went);
    a.ready_now = ready.is_some() && !gone;
    if let Some(r) = a.turns.last_mut() {
        if r.ready.is_none() {
            r.ready = ready;
        }
        r.castable |= any_cast;
        r.nonland |= nonland;
        r.payable |= payable || any_cast;
        for c in held {
            if !r.held.contains(&c) {
                r.held.push(c);
            }
        }
    }
}

/// the label without the trace's tags: "X -> Y [search]" -> "X"
fn short(label: &str) -> String {
    label.split(" -> ").next().unwrap_or(label).trim_end_matches(" [search]").trim_end_matches(" [must]").to_string()
}

/// p made the play `label` at a main-phase decision
pub fn acted(g: &mut Game, p: PlayerId, label: &str) {
    if g.active != Some(p) {
        return;
    }
    if let Some(a) = audit_of(g, p) {
        if a.ready_now {
            a.ready_choices.push(short(label));
        }
        if let Some(r) = a.turns.last_mut() {
            r.acted = true;
        }
    }
}

/// p chose the stop `opts[i]` at a main-phase decision (`search`: the look-ahead picked it)
pub fn stop(g: &mut Game, p: PlayerId, opts: &[Opt], i: usize, search: bool) {
    if g.active != Some(p) || audit_of(g, p).is_none() {
        return;
    }
    let alt = opts
        .iter()
        .filter(|o| o.act.is_some())
        .fold(None::<&Opt>, |b, o| if b.is_none_or(|b| o.utility > b.utility) { Some(o) } else { b })
        .map(|o| (short(&o.label), o.utility));
    let scores = search
        .then(|| {
            crate::ai::search::LAST_SCORES.with(|l| {
                let l = l.borrow();
                let s = l.iter().find(|x| x.0 == opts[i].label).map(|x| x.1)?;
                let b = l
                    .iter()
                    .filter(|x| x.0 != opts[i].label)
                    .map(|x| x.1)
                    .fold(None, |m: Option<f64>, v| Some(m.map_or(v, |m| m.max(v))))?;
                Some((s, b))
            })
        })
        .flatten();
    let t = g.player(p).turns;
    let label = opts[i].label.clone();
    let a = audit_of(g, p).unwrap();
    if a.ready_now {
        a.ready_choices.push(short(&label));
    }
    let ready = a.ready_now;
    let post = g.step == crate::state::Step::Main2;
    let a = audit_of(g, p).unwrap();
    a.stops.push(StopRec { turn: t, search, label, alt, scores, ready, post });
}

/// p's main phase is over: the castable cards it still holds are passed, and a land left in hand is noted
pub fn main_end(g: &mut Game, p: PlayerId) {
    if g.active != Some(p) || audit_of(g, p).is_none() {
        return;
    }
    let hand = g.player(p).hand.clone();
    let land = hand.iter().any(|&c| g.db.get(c).land);
    let a = audit_of(g, p).unwrap();
    let held: Vec<CardId> = std::mem::take(&mut a.last_castable).into_iter().filter(|c| hand.contains(c)).collect();
    a.ready_now = false;
    if let Some(r) = a.turns.last_mut() {
        for c in held {
            if !r.passed.contains(&c) {
                r.passed.push(c);
            }
        }
        r.land_left |= land;
    }
}

/// the cleanup discard to hand size is about to discard c
pub fn discard(g: &mut Game, p: PlayerId, c: CardId) {
    if audit_of(g, p).is_none() {
        return;
    }
    let kept = crate::ai::discard_keep(g, p).contains(&c);
    let (t, hand) = (g.player(p).turns, g.player(p).hand.len());
    crate::glog!(
        g,
        "    [audit] {} discards {} to hand size ({hand} cards in hand{})",
        g.player(p).name,
        g.db.get(c).name,
        if kept { "; discard_keep would keep it" } else { "" }
    );
    let a = audit_of(g, p).unwrap();
    a.discards.push(DiscardRec { card: c, turn: t, kept, hand });
}

/// the look-ahead's sure-win cap changed its pick
pub fn cap_changed(g: &mut Game) {
    if let Some(a) = g.audit.as_deref_mut()
        && !g.in_search
    {
        a.cap_changed += 1;
    }
}

// ------------------------------------------------------------------ the report
/// each own turn falls in one bucket, the first that matches (see `classify`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bucket {
    Won,
    ReadyNotTaken,
    PassedCastable,
    NoLandDrop,
    ManaShort,
    Progressed,
    Idle,
}

pub const BUCKETS: [Bucket; 7] = [
    Bucket::Won,
    Bucket::ReadyNotTaken,
    Bucket::PassedCastable,
    Bucket::NoLandDrop,
    Bucket::ManaShort,
    Bucket::Progressed,
    Bucket::Idle,
];

impl Bucket {
    pub fn name(self) -> &'static str {
        match self {
            Bucket::Won => "won",
            Bucket::ReadyNotTaken => "ready_not_taken",
            Bucket::PassedCastable => "passed_castable",
            Bucket::NoLandDrop => "no_land_drop",
            Bucket::ManaShort => "mana_short",
            Bucket::Progressed => "progressed",
            Bucket::Idle => "idle",
        }
    }
}

/// a card's role, by its tags (the first that fits, in this order)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    /// a piece of one of the deck's catalogued lines
    ComboPiece,
    Tutor,
    Ritual,
    /// fast mana, a rock, a mox or a dork
    FastMana,
    Draw,
    Protection,
    Land,
    Other,
}

pub const ROLES: [Role; 8] = [
    Role::ComboPiece,
    Role::Tutor,
    Role::Ritual,
    Role::FastMana,
    Role::Draw,
    Role::Protection,
    Role::Land,
    Role::Other,
];

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::ComboPiece => "combo piece",
            Role::Tutor => "tutor",
            Role::Ritual => "ritual",
            Role::FastMana => "fast mana / rock",
            Role::Draw => "draw",
            Role::Protection => "protection",
            Role::Land => "land",
            Role::Other => "other",
        }
    }
}

/// the names of the pieces of p's catalogued lines
fn piece_names(g: &Game, p: PlayerId) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = crate::impls::combos::audit_lines(g, p)
        .into_iter()
        .flat_map(|i| crate::impls::combos::audit_groups(i).iter().flat_map(|grp| grp.iter().copied()))
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// tutors the `tut` / `xtutor` tags miss (the Tier 6 plans' tutor lists, and the creature and land tutors of these
/// decks)
const TUTORS: [&str; 19] = [
    "Beseech the Mirror",
    "Intuition",
    "Gamble",
    "Diabolic Intent",
    "Crop Rotation",
    "Imperial Seal",
    "Vampiric Tutor",
    "Mystical Tutor",
    "Enlightened Tutor",
    "Demonic Tutor",
    "Ranger-Captain of Eos",
    "Eladamri's Call",
    "Chord of Calling",
    "Green Sun's Zenith",
    "Finale of Devastation",
    "Worldly Tutor",
    "Merchant Scroll",
    "Trophy Mage",
    "Wishclaw Talisman",
];

/// card-draw engines the `draw` tag misses (they draw off opponents' spells, so the inert seats make them blanks)
const DRAWS: [&str; 7] = [
    "Rhystic Study",
    "Mystic Remora",
    "Esper Sentinel",
    "Necropotence",
    "The One Ring",
    "Sylvan Library",
    "Ad Nauseam",
];

pub fn role(db: &CardDb, pieces: &[&str], c: CardId) -> Role {
    let d = db.get(c);
    if pieces.contains(&&*d.name) {
        Role::ComboPiece
    } else if d.land {
        Role::Land
    } else if d.tags.has(Tag::Tut) || d.tag(Tag::Xtutor) || TUTORS.contains(&&*d.name) {
        Role::Tutor
    } else if mana_card(db, c) && (d.instant || d.sorcery) {
        Role::Ritual
    } else if mana_card(db, c) {
        Role::FastMana
    } else if d.tags.has(Tag::Draw) || DRAWS.contains(&&*d.name) {
        Role::Draw
    } else if d.tag(Tag::Ctr) || d.tags.has(Tag::Prot) || crate::goldfish::LOCKS.contains(&&*d.name) {
        Role::Protection
    } else {
        Role::Other
    }
}

/// t6stats::is_mana_card on the card database
fn mana_card(db: &CardDb, c: CardId) -> bool {
    let d = db.get(c);
    !d.land
        && (d.tag(Tag::Rock)
            || d.tag(Tag::Dork)
            || d.tag(Tag::Fastmana)
            || d.tag(Tag::Moxd)
            || d.tag(Tag::Chromemox)
            || d.tag(Tag::Lr)
            || crate::t6stats::RITUALS.contains(&&*d.name))
}

/// one own turn, classified
#[derive(Debug, Clone, PartialEq)]
pub struct TurnOut {
    pub turn: u32,
    pub bucket: Bucket,
    /// passed_castable: the passed cards that were cast on a later turn or discarded, with their roles
    pub passed: Vec<(String, Role)>,
    /// ready_not_taken: the line that was ready
    pub line: Option<String>,
    /// payable nonland cards in hand that were never offered as a play this turn (any bucket but won)
    pub held: Vec<(String, Role)>,
    /// progressed: the most useful role among the cards it cast this turn (combo piece, tutor, ritual, fast mana, draw,
    /// protection, other, in that order), or "land only" / "abilities only" when it cast nothing
    pub progress: Option<String>,
}

/// what the report reads from one game
#[derive(Debug, Clone, PartialEq)]
pub struct AuditOut {
    pub turns: Vec<TurnOut>,
    pub discards: Vec<(String, Role, bool)>,
    pub stops: Vec<StopRec>,
    pub ready_choices: Vec<String>,
    /// the crude best case: the first own turn by which the deck had seen every piece of a line and enough mana
    /// sources for it, and that line
    pub best: Option<(u32, String)>,
    /// the same with each seen tutor standing in for one missing piece
    pub best_tutors: Option<(u32, String)>,
    pub cap_changed: u32,
}

/// classify each own turn of the deck under test (first match wins):
/// - won: the deck won on this turn (or in the end-of-turn window after it, which the report counts as this turn);
/// - ready_not_taken: a line was ready and affordable at a main-phase decision and the deck went for none this turn;
/// - passed_castable: a main phase ended with a castable nonland card in hand that the deck cast on a later own turn,
///   or discarded (any later discard, this turn's cleanup included);
/// - no_land_drop: no land played this turn, with a land card in hand as a main phase ended;
/// - mana_short: no nonland card in hand (nor the commander) had a cost the deck could pay at any main-phase decision
///   (instants count only when offered as a play: held counterspells are no main-phase play),
///   while there was a nonland card (a sorcery-speed card it could pay for but did not offer, held by a rule, is not this);
///   `held` lists the payable cards in hand that were not offered, on any turn;
/// - progressed: it cast, played or activated something;
/// - idle: anything else.
pub fn classify(g: &Game, a: &Audit, won: bool) -> Vec<TurnOut> {
    let pieces = piece_names(g, a.me);
    let n = a.turns.len();
    a.turns
        .iter()
        .enumerate()
        .map(|(k, r)| {
            let later = |c: CardId| {
                a.casts.iter().any(|&(x, t)| x == c && t > r.turn)
                    || a.discards.iter().any(|d| d.card == c && d.turn >= r.turn)
            };
            let passed: Vec<(String, Role)> = r
                .passed
                .iter()
                .copied()
                .filter(|&c| later(c))
                .map(|c| (g.db.get(c).name.to_string(), role(&g.db, &pieces, c)))
                .collect();
            let bucket = if won && k + 1 == n {
                Bucket::Won
            } else if r.ready.is_some() && !r.went {
                Bucket::ReadyNotTaken
            } else if !passed.is_empty() {
                Bucket::PassedCastable
            } else if !r.land_played && r.land_left {
                Bucket::NoLandDrop
            } else if !r.payable && r.nonland {
                Bucket::ManaShort
            } else if r.acted {
                Bucket::Progressed
            } else {
                Bucket::Idle
            };
            TurnOut {
                turn: r.turn,
                bucket,
                passed: if bucket == Bucket::PassedCastable { passed } else { vec![] },
                line: (bucket == Bucket::ReadyNotTaken).then(|| r.ready.unwrap_or("?").to_string()),
                held: if bucket == Bucket::Won {
                    vec![]
                } else {
                    r.held.iter().map(|&c| (g.db.get(c).name.to_string(), role(&g.db, &pieces, c))).collect()
                },
                progress: (bucket == Bucket::Progressed).then(|| {
                    let best = r.cast.iter().map(|&c| role(&g.db, &pieces, c)).min();
                    match best {
                        Some(x) => x.name().to_string(),
                        None if r.land_played => "land only".to_string(),
                        None => "abilities only".to_string(),
                    }
                }),
            }
        })
        .collect()
}

/// the crude best case: the first own turn T by which the deck had seen (kept hand and draws, tutored cards aside;
/// its commanders always) a card of every group of one of its lines, and mana sources >= the line's mana: its loop
/// mana plus the mana value of the cheapest seen card of each group (lands 0), all paid on turn T. Sources: the land
/// cards seen (at most T: one land drop a turn) plus the mana cards seen (rocks, dorks, moxen, rituals: 1 each).
/// `tutors`: a seen tutor (role tutor: tagged `tut` / `xtutor` or in `TUTORS`; not itself a piece) may stand in for one
/// missing group, its own
/// mana value and the fetched piece's added to the need (the cheapest tutors first; all on turn T, though a tutor to
/// the top of the library would give the piece only the turn after).
pub fn best_case(g: &Game, a: &Audit, tutors: bool) -> Option<(u32, String)> {
    use crate::impls::combos::{audit_groups, audit_lines, audit_mana, audit_name};
    let p = a.me;
    let lines = audit_lines(g, p);
    let pieces = piece_names(g, p);
    let cmdrs: Vec<CardId> = g.player(p).commanders().collect();
    let last = a.turns.len().max(1) as u32;
    for t in 1..=last {
        let seen: Vec<CardId> = a.seen.iter().filter(|x| x.1 <= t).map(|x| x.0).chain(cmdrs.iter().copied()).collect();
        let lands = seen.iter().filter(|&&c| g.db.get(c).land).count() as u32;
        let rocks = seen.iter().filter(|&&c| mana_card(&g.db, c)).count() as u32;
        let sources = lands.min(t) + rocks;
        let mut tut: Vec<u32> = if tutors {
            seen.iter().filter(|&&c| role(&g.db, &pieces, c) == Role::Tutor).map(|&c| g.db.get(c).cmc).collect()
        } else {
            vec![]
        };
        tut.sort_unstable();
        for &i in &lines {
            let mut need = audit_mana(i);
            let mut missing = 0;
            for grp in audit_groups(i) {
                let cost = seen
                    .iter()
                    .filter(|&&c| grp.contains(&&*g.db.get(c).name))
                    .map(|&c| if g.db.get(c).land { 0 } else { g.db.get(c).cmc })
                    .min();
                match cost {
                    Some(m) => need += m,
                    None => {
                        // the tutored piece is paid for too: the group's cheapest card (lands 0)
                        missing += 1;
                        need += grp
                            .iter()
                            .filter_map(|n| g.db.id(n))
                            .map(|c| if g.db.get(c).land { 0 } else { g.db.get(c).cmc })
                            .min()
                            .unwrap_or(0);
                    }
                }
            }
            if missing > tut.len() {
                continue;
            }
            need += tut[..missing].iter().sum::<u32>();
            if sources >= need {
                return Some((t, audit_name(i).to_string()));
            }
        }
    }
    None
}

/// the audit of a finished goldfish game (None outside goldfish games); `won`: the deck under test won it
pub fn report(g: &Game, won: bool) -> Option<AuditOut> {
    let a = g.audit.as_deref()?;
    let pieces = piece_names(g, a.me);
    Some(AuditOut {
        turns: classify(g, a, won),
        discards: a
            .discards
            .iter()
            .map(|d| (g.db.get(d.card).name.to_string(), role(&g.db, &pieces, d.card), d.kept))
            .collect(),
        stops: a.stops.clone(),
        ready_choices: a.ready_choices.clone(),
        best: best_case(g, a, false),
        best_tutors: best_case(g, a, true),
        cap_changed: a.cap_changed,
    })
}
