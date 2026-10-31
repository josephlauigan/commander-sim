//! --- phase4-values: card values derived from what a card does (`audit/phase4/values-design.md`).
//!
//! A card's **profile** is read once from its compiled abilities (`dsl`) and its role tags (what cardgen and the hand
//! tags compile from Oracle text): mana, cards, removal, damage, board presence, engines. Its **worth** reads the
//! profile at the current state (turn, hand, the opponents' boards, combo role); its **threat** (the derived `bomb`)
//! is what it does on the board if left alone. The generic policy reads these in place of the 0-90 priority table and
//! the `bomb=` tags for every generic-policy deck (`on`; `AI_VALUES=0` reads the old ones, a list of keys puts only
//! those decks on derived values).

use crate::dsl::model::{Ability, Effect, Num, To};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::OnceLock;

// ------------------------------------------------------------------ the switch
enum Mode {
    All,
    Keys(Vec<String>),
}

fn mode() -> &'static Mode {
    static M: OnceLock<Mode> = OnceLock::new();
    M.get_or_init(|| match std::env::var("AI_VALUES").ok().as_deref().map(str::trim) {
        // Stage C: every generic-policy deck on derived values by default (the paired tests were level on t1-t5,
        // `audit/phase4/values-handoff.md`); `0` reads the old tables, which JD's decks and Tier 6 still fall back to
        None | Some("") | Some("1") => Mode::All,
        Some("0") => Mode::Keys(vec![]),
        Some(list) => Mode::Keys(list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()),
    })
}

/// a generic-policy deck (not one of JD's decks playing its hand AI, not Tier 6)
pub fn generic_deck(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    let hand = (super::is_main(pl.key) || super::jd_deck(pl.key)) && super::hand_ai_key(pl.key); // --- phase4-hand-off
    !hand && !pl.t6 && !crate::impls::t6_plans::is_t6_key(pl.key)
}

/// does p decide with derived values?
pub fn on(g: &Game, p: PlayerId) -> bool {
    generic_deck(g, p)
        && match mode() {
            Mode::All => true,
            Mode::Keys(k) => k.iter().any(|x| x == g.player(p).key),
        }
}

// ------------------------------------------------------------------ overrides
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OverrideFile {
    #[serde(default)]
    card: Vec<Override>,
}

/// one row of `data/values_overrides.toml`
#[derive(serde::Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Override {
    pub name: String,
    pub worth: Option<f64>,
    pub threat: Option<f64>,
    pub reason: String,
}

const OVERRIDES_TOML: &str = include_str!("../../../../../data/values_overrides.toml");

/// parse an override table: every row needs a reason and a worth or a threat
pub fn parse_overrides(text: &str) -> Result<Vec<Override>, String> {
    let f: OverrideFile = toml::from_str(text).map_err(|e| e.to_string())?;
    for o in &f.card {
        if o.reason.trim().is_empty() {
            return Err(format!("{}: an override needs a reason", o.name));
        }
        if o.worth.is_none() && o.threat.is_none() {
            return Err(format!("{}: an override sets worth or threat", o.name));
        }
    }
    Ok(f.card)
}

/// the override table, by card name
pub fn overrides() -> &'static HashMap<String, Override> {
    static O: OnceLock<HashMap<String, Override>> = OnceLock::new();
    O.get_or_init(|| {
        parse_overrides(OVERRIDES_TOML)
            .unwrap_or_else(|e| panic!("data/values_overrides.toml: {e}"))
            .into_iter()
            .map(|o| (o.name.clone(), o))
            .collect()
    })
}

// ------------------------------------------------------------------ the profile
/// what a card does, read from its abilities and role tags (no game state)
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Profile {
    /// mana a turn from the permanent
    pub mana: f64,
    /// one-shot mana (rituals, Treasure)
    pub burst: f64,
    /// cards drawn once
    pub cards: f64,
    /// cards a turn from an engine
    pub cards_rep: f64,
    /// single-target removal
    pub removal: bool,
    /// mass removal
    pub sweep: bool,
    pub counter: bool,
    /// life each opponent loses once
    pub damage: f64,
    /// life each opponent loses a turn
    pub damage_rep: f64,
    pub power: f64,
    pub tough: f64,
    /// evasion keywords (flying, trample, menace, shadow, unblockable ...)
    pub evasion: f64,
    pub haste: bool,
    /// other combat keywords (deathtouch, lifelink, vigilance, first/double strike, hexproof, indestructible, ward)
    pub other_kw: f64,
    /// power made in tokens once
    pub tokens: f64,
    /// power made in tokens a turn
    pub tokens_rep: f64,
    /// power added to each creature
    pub team: f64,
    /// repeatable abilities (triggers, statics, activations, card code engines)
    pub engine: f64,
    /// taxes or locks on the opponents
    pub lock: f64,
    pub tutor: bool,
    pub protect: bool,
}

fn n_of(e: &Effect) -> f64 {
    match &e.n {
        Some(Num::Int(k)) => *k as f64,
        None => 1.0,
        _ => 2.0, // X or a count: about two
    }
}

fn pow_of(e: &Effect) -> f64 {
    match &e.pow {
        Some(Num::Int(k)) => (*k as f64).max(0.5),
        _ => 1.0,
    }
}

/// effects into the profile: `rep` = they happen again (a trigger or an activation)
fn effects_into(pr: &mut Profile, effects: &[Effect], rep: bool) {
    for e in effects {
        let n = n_of(e);
        let all = || {
            let sel = e.what.as_ref().or(match &e.to {
                Some(To::Sel(s)) => Some(&**s),
                _ => None,
            });
            sel.and_then(|s| s.sel.as_deref()) == Some("all")
        };
        let to_player = || {
            let sel = e.what.as_ref().or(match &e.to {
                Some(To::Sel(s)) => Some(&**s),
                _ => None,
            });
            matches!(sel.and_then(|s| s.sel.as_deref()), Some("player") | Some("any_target"))
        };
        match e.kind.as_str() {
            "draw" | "impulse" | "narset_dig" if e.who.as_deref().unwrap_or("you") == "you" => {
                if rep {
                    pr.cards_rep += n
                } else {
                    pr.cards += n
                }
            }
            "clue" | "regrow" | "reanimate" | "free_gy_spell" => {
                if rep {
                    pr.cards_rep += 0.6 * n
                } else {
                    pr.cards += 0.8 * n
                }
            }
            "add_mana" | "treasure" => {
                if rep && e.kind == "add_mana" {
                    pr.mana += n
                } else {
                    pr.burst += n
                }
            }
            "extra_land" | "put_land" => pr.mana += 0.7,
            // a land of your own sacrificed (Roiling Regrowth): a land less
            "sacrifice"
                if e.who.as_deref() == Some("you") && e.filter.as_ref().and_then(|f| f.kind.as_deref()) == Some("land") =>
            {
                pr.mana -= n
            }
            "destroy" | "exile" | "bounce" | "tuck" | "sacrifice" if e.who.as_deref() != Some("you") => {
                if all() {
                    pr.sweep = true
                } else {
                    pr.removal = true
                }
            }
            "damage" if to_player() => {
                if rep {
                    pr.damage_rep += n
                } else {
                    pr.damage += n
                }
            }
            "damage" => {
                if all() {
                    pr.sweep = true
                } else {
                    pr.removal = true
                }
            }
            "lose_life" if e.who.as_deref() != Some("you") => {
                if rep {
                    pr.damage_rep += n
                } else {
                    pr.damage += n
                }
            }
            "counter_spell" => pr.counter = true,
            "token" | "amass" => {
                let p = n * (pow_of(e) + 0.5 * e.keywords.len() as f64);
                if rep { pr.tokens_rep += p } else { pr.tokens += p }
            }
            "counters" | "pump" | "proliferate" if rep => pr.team += 0.2,
            "search" if e.filter.as_ref().and_then(|f| f.kind.as_deref()) == Some("land") => {
                pr.mana += if matches!(e.to, Some(To::Zone(ref z)) if z == "battlefield") { n } else { 0.5 * n }
            }
            "search" => pr.tutor = true,
            "grant" | "blink" => pr.protect = true,
            "extra_combat" => pr.engine += 1.5,
            "modal" => {
                let mut best = Profile::default();
                for m in &e.modes {
                    let mut x = Profile::default();
                    effects_into(&mut x, m, rep);
                    best = best.max(&x);
                }
                *pr = pr.max(&best);
            }
            _ => {}
        }
    }
}

impl Profile {
    /// add another reading, its amounts scaled by k (flags as they are)
    fn add(&mut self, o: &Profile, k: f64) {
        self.mana += k * o.mana;
        self.burst += k * o.burst;
        self.cards += k * o.cards;
        self.cards_rep += k * o.cards_rep;
        self.removal |= o.removal;
        self.sweep |= o.sweep;
        self.counter |= o.counter;
        self.damage += k * o.damage;
        self.damage_rep += k * o.damage_rep;
        self.tokens += k * o.tokens;
        self.tokens_rep += k * o.tokens_rep;
        self.team += k * o.team;
        self.engine += k * o.engine;
        self.lock += k * o.lock;
        self.tutor |= o.tutor;
        self.protect |= o.protect;
    }

    /// the larger of each field (the tag reading and the ability reading of the same card)
    fn max(&self, o: &Profile) -> Profile {
        Profile {
            mana: self.mana.max(o.mana),
            burst: self.burst.max(o.burst),
            cards: self.cards.max(o.cards),
            cards_rep: self.cards_rep.max(o.cards_rep),
            removal: self.removal || o.removal,
            sweep: self.sweep || o.sweep,
            counter: self.counter || o.counter,
            damage: self.damage.max(o.damage),
            damage_rep: self.damage_rep.max(o.damage_rep),
            power: self.power.max(o.power),
            tough: self.tough.max(o.tough),
            evasion: self.evasion.max(o.evasion),
            haste: self.haste || o.haste,
            other_kw: self.other_kw.max(o.other_kw),
            tokens: self.tokens.max(o.tokens),
            tokens_rep: self.tokens_rep.max(o.tokens_rep),
            team: self.team.max(o.team),
            engine: self.engine.max(o.engine),
            lock: self.lock.max(o.lock),
            tutor: self.tutor || o.tutor,
            protect: self.protect || o.protect,
        }
    }
}

const EVASION: [&str; 8] = ["flying", "trample", "menace", "shadow", "horsemanship", "fear", "intimidate", "skulk"];
const OTHER_KW: [&str; 9] = [
    "deathtouch",
    "lifelink",
    "vigilance",
    "first strike",
    "double strike",
    "hexproof",
    "indestructible",
    "ward",
    "reach",
];

/// the role tags' reading
fn from_tags(g: &Game, c: CardId) -> Profile {
    let d = g.db.get(c);
    let t = &d.tags;
    let mut pr = Profile::default();
    let int1 = |k: Tag| t.int(k).unwrap_or(1).max(1) as f64;
    if let Some(r) = t.str(Tag::Rock) {
        pr.mana += r.split(':').next().and_then(|x| x.parse::<f64>().ok()).unwrap_or(1.0);
    }
    if t.has(Tag::Dork) {
        pr.mana += 1.0;
    }
    if t.has(Tag::Lr) {
        pr.mana += int1(Tag::Lr);
    }
    if t.has(Tag::Grim) {
        pr.mana += 3.0;
    }
    if t.has(Tag::Fastmana) {
        if d.perm { pr.mana += 1.0 } else { pr.burst += 2.0 }
    }
    if t.has(Tag::Treas) {
        pr.burst += int1(Tag::Treas);
    }
    if let Some(n) = t.int(Tag::Draw) {
        let n = if n > 0 { n as f64 } else { 2.0 };
        if d.perm && t.has(Tag::Eng) { pr.cards_rep += n } else { pr.cards += n }
    }
    if t.has(Tag::Eng) {
        pr.cards_rep += int1(Tag::Eng).min(3.0);
        pr.engine += 1.0;
    }
    for k in [Tag::Rhystic, Tag::Spelldraw, Tag::Drawcre, Tag::Clamp, Tag::Fable, Tag::Threedreams] {
        if t.has(k) {
            pr.cards_rep += 1.0;
            pr.engine += 1.0;
        }
    }
    if t.has(Tag::Rem) {
        pr.removal = true;
    }
    if t.has(Tag::Wipe) {
        pr.sweep = true;
    }
    if t.has(Tag::Ctr) {
        pr.counter = true;
    }
    if let Some(n) = t.int(Tag::Drainetb) {
        pr.damage += n.max(1) as f64;
    }
    for k in [Tag::Drain, Tag::Ping, Tag::Bartist, Tag::Kiln] {
        if t.has(k) {
            pr.damage_rep += int1(k).min(3.0);
            pr.engine += 1.0;
        }
    }
    if t.has(Tag::Tok) {
        pr.tokens += int1(Tag::Tok) * int1(Tag::Tokp);
    }
    if t.has(Tag::Mktok) {
        pr.tokens += 1.0;
    }
    if t.has(Tag::Tokx) {
        pr.tokens += 4.0;
    }
    for k in [Tag::Tokup, Tag::Tokatk, Tag::Spelltok, Tag::Warleader] {
        if t.has(k) {
            pr.tokens_rep += int1(k).min(3.0);
            pr.engine += 1.0;
        }
    }
    if t.has(Tag::Anth) {
        pr.team += int1(Tag::Anth);
    }
    if t.has(Tag::Crusade) {
        pr.team += 1.0;
    }
    for k in [Tag::Birgi, Tag::Rabble, Tag::Stampede, Tag::Explore, Tag::Loam, Tag::Rishkar, Tag::Sac] {
        if t.has(k) {
            pr.engine += 1.0;
        }
    }
    if t.has(Tag::Tut) || t.has(Tag::Seal) || t.has(Tag::Top) {
        pr.tutor = true;
    }
    if t.has(Tag::Prot) || t.has(Tag::Cloak) {
        pr.protect = true;
    }
    if t.has(Tag::Fly) {
        pr.evasion += 1.0;
    }
    if t.has(Tag::Trample) {
        pr.evasion += 1.0;
    }
    if t.has(Tag::Haste) {
        pr.haste = true;
    }
    for k in [Tag::Dt, Tag::Lifelink, Tag::Vig] {
        if t.has(k) {
            pr.other_kw += 1.0;
        }
    }
    pr
}

/// the compiled abilities' reading, plus the card's printed body and its card code
fn from_abilities(g: &Game, c: CardId) -> Profile {
    let d = g.db.get(c);
    let mut pr = Profile::default();
    if d.creature {
        pr.power = d.pow.max(0) as f64;
        pr.tough = d.tgh.max(0) as f64;
    }
    for k in d.kws.iter() {
        if EVASION.contains(&&**k) {
            pr.evasion += 1.0;
        } else if OTHER_KW.contains(&&**k) {
            pr.other_kw += 1.0;
        } else if &**k == "haste" {
            pr.haste = true;
        }
    }
    for a in d.abilities.iter() {
        match a {
            Ability::Spell { effects } => effects_into(&mut pr, effects, false),
            Ability::Triggered { event, source, effects, .. } => {
                let once = event == "etb" && source.as_deref() == Some("self");
                let mut x = Profile::default();
                effects_into(&mut x, effects, !once);
                x.burst += x.mana; // mana from a trigger comes when it fires, not on a tap
                x.mana = 0.0;
                // an attack or combat damage trigger needs a creature that attacks or connects
                let k = if matches!(event.as_str(), "attack" | "combat_damage") { 0.5 } else { 1.0 };
                let read = x != Profile::default();
                pr.add(&x, k);
                if !once {
                    pr.engine += if read { 1.0 } else { 0.5 }; // a trigger whose effect the profile can't read: half
                }
            }
            Ability::Activated { cost, effects, .. } => {
                let mana_only = effects.iter().all(|e| e.kind == "add_mana");
                if mana_only && cost.tap == Some(true) && cost.sac.is_none() && cost.mana.is_none() {
                    pr.mana += effects.iter().map(n_of).sum::<f64>();
                } else if mana_only {
                    pr.burst += 0.5 * effects.iter().map(n_of).sum::<f64>(); // mana for a sacrifice or a payment
                } else if cost.sac.as_deref() == Some("self") {
                    effects_into(&mut pr, effects, false);
                } else {
                    effects_into(&mut pr, effects, true);
                    pr.engine += 0.8;
                }
            }
            Ability::Loyalty { effects, .. } => {
                let mut x = Profile::default();
                effects_into(&mut x, effects, true);
                pr.add(&x, 0.5); // one loyalty ability a turn
                pr.engine += 0.7;
            }
            Ability::Static { kind, pow, mana, who, amount, .. } => match kind.as_str() {
                "note" | "equip_cost" => {}
                "equip_bonus" => pr.power += pow.unwrap_or(1).max(0) as f64, // on the creature it equips
                "equip_keyword" => pr.evasion += 1.0,
                "equip_protection" => pr.protect = true,
                // a cost change: cheaper spells for you are mana, dearer ones for the table a lock
                "cost" if who.as_deref() == Some("you") && amount.unwrap_or(0) < 0 => {
                    pr.mana += 0.8 * (-amount.unwrap_or(0)) as f64
                }
                "cost" => pr.lock += 1.0,
                "anthem" | "bma_anthem" => pr.team += pow.unwrap_or(1).max(1) as f64,
                "keyword" | "self_keyword" | "unblockable" => pr.evasion += 1.0,
                "uncounterable" | "no_lifegain" | "cant_block" => pr.lock += 0.5,
                _ if mana.is_some() => pr.mana += mana.unwrap_or(1) as f64,
                _ => pr.engine += 0.5,
            },
            Ability::Replacement { .. } => pr.engine += 0.5, // a doubler: worth what the deck feeds it
            Ability::AdditionalCost { text } if text.contains("sacrifice a land") => pr.mana -= 1.0, // Harrow
            Ability::AdditionalCost { .. } => {}
        }
    }
    if d.start_loyalty.is_some() {
        pr.engine = pr.engine.max(1.5); // a planeswalker: an ability every turn
    }
    if let Some(h) = g.registry.get(c) {
        use crate::hooks::Event;
        if h.attack_tax.is_some() || h.attack_cap.is_some() {
            pr.lock += 1.0;
        }
        const LOCK: [Event; 6] =
            [Event::Cost, Event::CanCast, Event::MinCost, Event::Uncounterable, Event::NoLifegain, Event::NoGraveyard];
        if LOCK.iter().any(|&e| h.handles(e)) {
            pr.lock += 1.0;
        }
        // the events card code runs an engine on (cardimpl.ENGINE_EVENTS, as `pool::card_threat_value` reads them);
        // any other live card code on a permanent counts as one engine too (Exploration, Coat of Arms ...)
        const ENGINE: [Event; 17] = [
            Event::Options,
            Event::Cast,
            Event::Dies,
            Event::Etb,
            Event::Attack,
            Event::Upkeep,
            Event::EndStep,
            Event::Landfall,
            Event::Draw,
            Event::Sacrifice,
            Event::CombatDamage,
            Event::TriggerCopies,
            Event::ExtraLands,
            Event::LandMana,
            Event::GrantKw,
            Event::Discard,
            Event::LandGy,
        ];
        let equipment = d.tags.has(Tag::Prot) || d.tags.has(Tag::Cloak); // the pool's equip code runs these
        if d.perm && !d.land && !equipment && (ENGINE.iter().any(|&e| h.handles(e)) || h.live()) {
            pr.engine = pr.engine.max(1.0);
        }
        if let Some(v) = h.pval {
            pr.lock = pr.lock.max((v - 3.0).max(0.0) / 2.5); // card code's own threat value, above an engine's
        }
    }
    pr
}

/// what card c does (no game state)
pub fn profile(g: &Game, c: CardId) -> Profile {
    from_tags(g, c).max(&from_abilities(g, c))
}

// ------------------------------------------------------------------ worth now
/// the turns an engine will run, in card units (more early, less late)
fn reps(round: u32) -> f64 {
    (12.0 - round as f64).clamp(3.0, 8.0) / 2.5
}

/// how much mana matters while the deck develops
fn early(turn: u32) -> f64 {
    match turn {
        0..=4 => 1.0,
        5..=6 => 0.55,
        _ => 0.25,
    }
}

/// a mana source early: it pays for a spell every turn after (a one-mana rock on turn 3 ranks with the old table's 85)
pub const MANA_W: f64 = 5.5;
pub const TUTOR_W: f64 = 3.5;
pub const PROTECT_W: f64 = 2.5;
pub const COUNTER_W: f64 = 4.0;
/// an engine the profile can't read, a turn
pub const ENGINE_W: f64 = 0.9;

/// the best permanent an opponent of p has that removal could hit, by p's reading
fn best_target(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let t = &g.db.get(c).tags;
    let ms: Vec<PermId> = if let Some(rk) = t.str(Tag::Rem) {
        crate::engine::removal::legal_targets(g, p, rk, t.str(Tag::Tgt).unwrap_or("c"), t.has(Tag::Mv4), Some(c))
            .into_iter()
            .filter(|&m| g.perm(m).owner != p)
            .collect()
    } else {
        g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).filter(|&m| !g.perm(m).phased).collect()
    };
    ms.iter().map(|&m| pval_for(g, p, m)).fold(0.0, f64::max)
}

/// the parts of c's worth now; `held` = include the held interaction (counters, removal and wipe spells, protection)
pub fn worth_parts(g: &Game, p: PlayerId, c: CardId, held: bool) -> [f64; 8] {
    let d = g.db.get(c);
    let pr = profile(g, c);
    let pl = g.player(p);
    let opps = g.opps(p).count().max(1) as f64;
    let reps = reps(g.round);
    let no_land = !pl.hand.iter().any(|&x| x != c && g.db.get(x).land);
    let mana = MANA_W * pr.mana * early(pl.turns) * if no_land { 1.25 } else { 1.0 } + 0.5 * pr.burst;
    let need = if pl.hand.len() <= 2 { 1.3 } else { 1.0 };
    let cards = pr.cards * need + pr.cards_rep * reps;
    let held_spell = !d.perm || d.instant;
    let mut removal = 0.0;
    if pr.removal && (held || d.perm) {
        let b = best_target(g, p, c);
        // no target now: worth keeping (held), nothing to cast it at (Seal of Cleansing with no artifact out)
        removal += if b > 0.0 { 2.5 + 0.35 * b } else if held { 1.5 } else { 0.0 };
    }
    if pr.sweep && (held || !held_spell) {
        let theirs: f64 = g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).map(|m| pval_for(g, p, m)).sum();
        let mine: f64 = pl.perms.iter().map(|&m| pval_for(g, p, m)).sum();
        removal += (1.5 + 0.25 * (theirs / opps - 1.2 * mine)).max(0.5);
    }
    if pr.counter && held {
        removal += COUNTER_W;
    }
    let damage = 0.45 * opps * (pr.damage + pr.damage_rep * reps);
    let creatures = pl.perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count() as f64;
    let board = 0.5 * pr.power
        + 0.2 * pr.tough
        + 0.4 * pr.evasion
        + if pr.haste { 0.3 } else { 0.0 }
        + 0.25 * pr.other_kw
        + 0.6 * pr.tokens
        + 0.35 * pr.tokens_rep * reps
        + 0.5 * pr.team * (creatures + 1.0);
    // an engine whose effect the profile reads (cards, damage, tokens a turn) is counted there; one it can't read (card
    // code, a static) is worth about a card a turn while it runs
    let explained = pr.cards_rep + pr.damage_rep + pr.tokens_rep > 0.0;
    let engine = if explained { 0.3 * pr.engine } else { ENGINE_W * reps * pr.engine.min(2.0) } + 1.5 * pr.lock;
    let mut role = 0.8 * crate::cardcode::combo_imp(g, p, c);
    if crate::cardcode::is_combo_piece(g, c) {
        role = role.max(1.0);
    }
    // protection on a permanent (boots, Greaves) needs a creature to protect
    let protects = pr.protect && (held || (d.perm && (d.creature || creatures > 0.0)));
    let misc = if pr.tutor { TUTOR_W } else { 0.0 } + if protects { PROTECT_W } else { 0.0 };
    // fit: the mana it is short of next turn, a colour of its cost with no source
    let mut fit = 0.0;
    if !d.land {
        let mana_perm = |m: PermId| {
            g.perm(m).cd.is_some_and(|x| [Tag::Rock, Tag::Dork, Tag::Fastmana].iter().any(|&k| g.db.get(x).tag(k)))
        };
        let sources = pl.lands.len() + pl.perms.iter().filter(|&&m| mana_perm(m)).count() + usize::from(!no_land);
        let (gn, pips) = crate::engine::mana::cost_of(g, p, c); // what it costs now (reductions included)
        let cost = gn as f64 + pips.chars().count() as f64;
        fit -= 0.4 * (cost - sources as f64).max(0.0);
        if pl.lands.len() >= 2 {
            let mut have = crate::cards::Colors::NONE;
            for &l in &pl.lands {
                have = have.union(crate::engine::mana::land_colors_now(g, p, l));
            }
            if d.pips.chars().any(|ch| "WUBRG".contains(ch) && !have.has(ch)) {
                fit -= 1.5;
            }
        }
    }
    [mana, cards, removal, damage, board, engine, role + misc, fit]
}

/// c's worth to p now, in card-equivalents (about 0-10); the override table's `worth` wins
pub fn worth(g: &Game, p: PlayerId, c: CardId, held: bool) -> f64 {
    if let Some(w) = overrides().get(&*g.db.get(c).name).and_then(|o| o.worth) {
        return w;
    }
    worth_parts(g, p, c, held).iter().sum::<f64>().clamp(0.0, 10.0)
}

/// the derived cast priority (the 0-90 scale `generic_prio` returns; 0 = nothing to gain by casting it now)
pub fn cast_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let w = worth(g, p, c, false);
    if w <= 0.05 { 0 } else { (20.0 + 8.0 * w).round().clamp(1.0, 90.0) as i32 }
}

/// the derived `card_worth` (the 0-90 scale `tutors::card_worth` returns): the held interaction counts
pub fn card_worth(g: &Game, p: PlayerId, c: CardId) -> f64 {
    10.0 * worth(g, p, c, true)
}

// ------------------------------------------------------------------ threat (the derived bomb)
/// how much card c does on the board if left alone (0-10, the scale of the old `bomb=` tags); no game state
pub fn threat(g: &Game, c: CardId) -> f64 {
    let d = g.db.get(c);
    if let Some(t) = overrides().get(&*d.name).and_then(|o| o.threat) {
        return t;
    }
    if !d.perm {
        return 0.0;
    }
    let pr = profile(g, c);
    let v = 0.9 * pr.power
        + 0.8 * pr.evasion
        + 0.5 * (f64::from(u8::from(pr.haste)) + pr.other_kw.min(2.0))
        + 0.5 * (pr.tough - pr.power).clamp(0.0, 3.0)
        + 1.6 * pr.engine.min(2.0)
        + 2.5 * pr.lock
        + 1.2 * pr.cards_rep
        + 0.6 * (pr.tokens_rep + pr.damage_rep).min(4.0)
        + 0.8 * pr.team
        + 1.0 * pr.mana.min(3.0);
    v.min(10.0)
}

/// the threshold the old `bomb` reads meant by "a bomb" (cardgen tagged power 5 and up)
pub const BOMB: f64 = 5.0;

/// what the reader reads as c's bomb rating: the derived threat on the derived arm, else the tag
pub fn bomb_of(g: &Game, reader: PlayerId, c: CardId) -> f64 {
    if on(g, reader) { threat(g, c) } else { g.db.get(c).bomb as f64 }
}

// ------------------------------------------------------------------ pval by a reader
thread_local! {
    /// the player whose reading `pval` gives (None: the tags', as before)
    static READER: Cell<Option<PlayerId>> = const { Cell::new(None) };
}

/// the bomb rating `pval` reads for card c: the derived threat while a derived-arm reader asks, else the tag
pub fn pval_bomb(g: &Game, c: CardId) -> f64 {
    match READER.get() {
        Some(r) if on(g, r) => {
            let t = threat(g, c);
            if t >= BOMB { t } else { 0.0 } // below a bomb the creature rule (1 + power / 2) reads it, as before
        }
        _ => g.db.get(c).bomb as f64,
    }
}

/// `pval` as `reader` reads it (identical to `pval` unless the reader is on the derived arm)
pub fn pval_for(g: &Game, reader: PlayerId, m: PermId) -> f64 {
    if !on(g, reader) {
        return crate::engine::values::pval(g, m);
    }
    let prev = READER.replace(Some(reader));
    let v = crate::engine::values::pval(g, m);
    READER.set(prev);
    v
}

/// run f with `pval` giving reader's reading (unchanged unless the reader is on the derived arm)
pub fn read_as<T>(g: &Game, reader: PlayerId, f: impl FnOnce() -> T) -> T {
    if !on(g, reader) {
        return f();
    }
    let prev = READER.replace(Some(reader));
    let v = f();
    READER.set(prev);
    v
}
