//! Hand-written card code (Python's `cardimpl.py` registry and `cards/impl/*.py`).
//!
//! Python registers a function per (card name, event) with `@on(name, event)`, and the engine finds the hooked
//! permanents for an event by walking `g.hooks`. Here each card with code has a `CardImpl`: one typed function slot
//! per event, so a hook with the wrong shape doesn't compile. The `Registry` maps `CardId` to its `CardImpl`; card
//! modules fill it at startup by card name.
//!
//! Slots are added to `CardImpl` event by event, as the engine code that fires each event is ported (M2, M3).
//!
//! Triggered abilities don't run when their event fires: they wait in `Game::trig_queue` as a `Trigger`, then go on
//! the stack. Python stores the hook function and its arguments; here a `Trigger` stores the event and its
//! arguments as data (`Call`), and the hook is found again in the registry when it resolves, so a game copy
//! (which may hold pending triggers) stays plain data.

use crate::cards::CardDb;
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::sym::Sym;

/// Every event card code can hook (Python: the second argument of `@on(name, event)`, and the events the engine
/// fires), sorted by name. `TRIGGER_EVENTS` are triggered abilities and go on the stack; the rest are asked for
/// a value at once (cost changes, locks, options).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Event {
    Attack,
    AttackCap,
    AttackTax,
    Blocks,
    BloodMoon,
    CanCast,
    CardsToGy,
    Cast,
    CombatDamage,
    CombatStart,
    Copycast,
    Cost,
    CreatureToGy,
    Crew,
    Defend,
    Dies,
    Discard,
    Draw,
    EndStep,
    Etb,
    ExiledFromBf,
    ExtraLands,
    ExtraMana,
    GainLife,
    GrantKw,
    GyDies,
    GyHate,
    GyLandfall,
    GyOptions,
    HandAttack,
    HandCast,
    HandDefend,
    HandOptions,
    LandDefend,
    LandGy,
    LandMana,
    LandOptions,
    LandPlay,
    LandUpkeep,
    Landfall,
    LandsFromGy,
    LandsFromTop,
    Leaves,
    LoseLife,
    LoyaltyExtra,
    Main1,
    Main2,
    ManaLock,
    ManaTapped,
    MinCost,
    Monarch,
    Ninjutsu,
    NoArtifactMana,
    NoCreatureMana,
    NoGraveyard,
    NoLifegain,
    NonlandManaBonus,
    Options,
    PlayerHexproof,
    PreventDamage,
    ProliferateExtra,
    Proliferated,
    Rebound,
    Regenerate,
    Resolve,
    Sacrifice,
    Sba,
    SearchLimit,
    SelfDies,
    SkipDraw,
    StealDraw,
    TokenCreated,
    TokensEnter,
    TreasureBonus,
    TriggerCopies,
    Uncounterable,
    Upkeep,
}

pub const EVENTS: [(Event, &str); 77] = [
    (Event::Attack, "attack"),
    (Event::AttackCap, "attack_cap"),
    (Event::AttackTax, "attack_tax"),
    (Event::Blocks, "blocks"),
    (Event::BloodMoon, "blood_moon"),
    (Event::CanCast, "can_cast"),
    (Event::CardsToGy, "cards_to_gy"),
    (Event::Cast, "cast"),
    (Event::CombatDamage, "combat_damage"),
    (Event::CombatStart, "combat_start"),
    (Event::Copycast, "copycast"),
    (Event::Cost, "cost"),
    (Event::CreatureToGy, "creature_to_gy"),
    (Event::Crew, "crew"),
    (Event::Defend, "defend"),
    (Event::Dies, "dies"),
    (Event::Discard, "discard"),
    (Event::Draw, "draw"),
    (Event::EndStep, "end_step"),
    (Event::Etb, "etb"),
    (Event::ExiledFromBf, "exiled_from_bf"),
    (Event::ExtraLands, "extra_lands"),
    (Event::ExtraMana, "extra_mana"),
    (Event::GainLife, "gain_life"),
    (Event::GrantKw, "grant_kw"),
    (Event::GyDies, "gy_dies"),
    (Event::GyHate, "gy_hate"),
    (Event::GyLandfall, "gy_landfall"),
    (Event::GyOptions, "gy_options"),
    (Event::HandAttack, "hand_attack"),
    (Event::HandCast, "hand_cast"),
    (Event::HandDefend, "hand_defend"),
    (Event::HandOptions, "hand_options"),
    (Event::LandDefend, "land_defend"),
    (Event::LandGy, "land_gy"),
    (Event::LandMana, "land_mana"),
    (Event::LandOptions, "land_options"),
    (Event::LandPlay, "land_play"),
    (Event::LandUpkeep, "land_upkeep"),
    (Event::Landfall, "landfall"),
    (Event::LandsFromGy, "lands_from_gy"),
    (Event::LandsFromTop, "lands_from_top"),
    (Event::Leaves, "leaves"),
    (Event::LoseLife, "lose_life"),
    (Event::LoyaltyExtra, "loyalty_extra"),
    (Event::Main1, "main1"),
    (Event::Main2, "main2"),
    (Event::ManaLock, "mana_lock"),
    (Event::ManaTapped, "mana_tapped"),
    (Event::MinCost, "min_cost"),
    (Event::Monarch, "monarch"),
    (Event::Ninjutsu, "ninjutsu"),
    (Event::NoArtifactMana, "no_artifact_mana"),
    (Event::NoCreatureMana, "no_creature_mana"),
    (Event::NoGraveyard, "no_graveyard"),
    (Event::NoLifegain, "no_lifegain"),
    (Event::NonlandManaBonus, "nonland_mana_bonus"),
    (Event::Options, "options"),
    (Event::PlayerHexproof, "player_hexproof"),
    (Event::PreventDamage, "prevent_damage"),
    (Event::ProliferateExtra, "proliferate_extra"),
    (Event::Proliferated, "proliferated"),
    (Event::Rebound, "rebound"),
    (Event::Regenerate, "regenerate"),
    (Event::Resolve, "resolve"),
    (Event::Sacrifice, "sacrifice"),
    (Event::Sba, "sba"),
    (Event::SearchLimit, "search_limit"),
    (Event::SelfDies, "self_dies"),
    (Event::SkipDraw, "skip_draw"),
    (Event::StealDraw, "steal_draw"),
    (Event::TokenCreated, "token_created"),
    (Event::TokensEnter, "tokens_enter"),
    (Event::TreasureBonus, "treasure_bonus"),
    (Event::TriggerCopies, "trigger_copies"),
    (Event::Uncounterable, "uncounterable"),
    (Event::Upkeep, "upkeep"),
];

/// Python's `engine.TRIGGER_EVENTS`
pub const TRIGGER_EVENTS: [Event; 24] = [
    Event::Attack,
    Event::Blocks,
    Event::CardsToGy,
    Event::Cast,
    Event::CombatDamage,
    Event::CombatStart,
    Event::Copycast,
    Event::CreatureToGy,
    Event::Dies,
    Event::Discard,
    Event::Draw,
    Event::EndStep,
    Event::Etb,
    Event::ExiledFromBf,
    Event::GainLife,
    Event::LandGy,
    Event::Landfall,
    Event::Leaves,
    Event::LoseLife,
    Event::Proliferated,
    Event::Sacrifice,
    Event::SelfDies,
    Event::TokenCreated,
    Event::Upkeep,
];
impl Event {
    pub fn name(self) -> &'static str {
        EVENTS[self as usize].1
    }

    pub fn from_name(name: &str) -> Option<Event> {
        EVENTS.binary_search_by(|e| e.1.cmp(name)).ok().map(|i| EVENTS[i].0)
    }

    pub fn is_trigger(self) -> bool {
        TRIGGER_EVENTS.contains(&self)
    }
}

// ------------------------------------------------------------------ hook signatures
/// The permanent whose code is running (Python's `src`).
pub type Src = PermId;

/// etb(g, src, p, m): m entered under p's control
pub type EtbFn = fn(&mut Game, Src, PlayerId, PermId) -> Res;
/// leaves(g, src, m): m left the battlefield (any way)
pub type LeavesFn = fn(&mut Game, Src, PermId) -> Res;
/// dies(g, src, m, cause): any permanent m died
pub type DiesFn = fn(&mut Game, Src, PermId, Sym) -> Res;
/// upkeep, end_step, main1, main2, draw, landfall, monarch (g, src, p): something happened to or for player p
pub type PlayerFn = fn(&mut Game, Src, PlayerId) -> Res;
/// cast(g, src, caster, c): any player cast c
pub type CastFn = fn(&mut Game, Src, PlayerId, CardId) -> Res;
/// attack(g, src, p, atk, d): p attacked d with atk; returns new attacking creatures
pub type AttackFn = fn(&mut Game, Src, PlayerId, &[PermId], PlayerId) -> Res<Vec<PermId>>;
/// combat_damage(g, src, p, a, d, dmg): attacker a dealt dmg combat damage to player d
pub type CombatDamageFn = fn(&mut Game, Src, PlayerId, PermId, PlayerId, i32) -> Res;
/// options(g, src, p, post): activated abilities the AI may use (post: None at the end-of-turn window)
pub type OptionsFn = fn(&mut Game, Src, PlayerId, Option<bool>) -> Res<Vec<Opt>>;
/// cost(g, src, caster, c): generic mana added to (+) or removed from (-) c's cost
pub type CostFn = fn(&Game, Src, PlayerId, CardId) -> i32;
/// can_cast(g, src, caster, c, zone): false forbids casting c
pub type CanCastFn = fn(&Game, Src, PlayerId, CardId, Sym) -> bool;
/// attack_tax / attack_cap (g, src, attacker, d): mana per attacker / the most creatures that can attack d
pub type AttackLimitFn = fn(&Game, Src, PlayerId, PlayerId) -> Option<i32>;
/// grant_kw(g, src, m, kw): a static keyword grant
pub type GrantKwFn = fn(&Game, Src, PermId, Sym) -> bool;

/// One card's code: a slot per event it handles.
#[derive(Debug, Clone, Copy, Default)]
pub struct CardImpl {
    pub etb: Option<EtbFn>,
    pub leaves: Option<LeavesFn>,
    pub dies: Option<DiesFn>,
    pub self_dies: Option<DiesFn>,
    pub upkeep: Option<PlayerFn>,
    pub end_step: Option<PlayerFn>,
    pub main1: Option<PlayerFn>,
    pub main2: Option<PlayerFn>,
    pub draw: Option<PlayerFn>,
    pub landfall: Option<PlayerFn>,
    pub cast: Option<CastFn>,
    pub attack: Option<AttackFn>,
    pub combat_damage: Option<CombatDamageFn>,
    pub options: Option<OptionsFn>,
    pub cost: Option<CostFn>,
    pub can_cast: Option<CanCastFn>,
    pub attack_tax: Option<AttackLimitFn>,
    pub attack_cap: Option<AttackLimitFn>,
    pub grant_kw: Option<GrantKwFn>,
}

impl CardImpl {
    /// Does the card handle `e`? (only for the events with a slot so far)
    pub fn handles(&self, e: Event) -> bool {
        match e {
            Event::Etb => self.etb.is_some(),
            Event::Leaves => self.leaves.is_some(),
            Event::Dies => self.dies.is_some(),
            Event::SelfDies => self.self_dies.is_some(),
            Event::Upkeep => self.upkeep.is_some(),
            Event::EndStep => self.end_step.is_some(),
            Event::Main1 => self.main1.is_some(),
            Event::Main2 => self.main2.is_some(),
            Event::Draw => self.draw.is_some(),
            Event::Landfall => self.landfall.is_some(),
            Event::Cast => self.cast.is_some(),
            Event::Attack => self.attack.is_some(),
            Event::CombatDamage => self.combat_damage.is_some(),
            Event::Options => self.options.is_some(),
            Event::Cost => self.cost.is_some(),
            Event::CanCast => self.can_cast.is_some(),
            Event::AttackTax => self.attack_tax.is_some(),
            Event::AttackCap => self.attack_cap.is_some(),
            Event::GrantKw => self.grant_kw.is_some(),
            _ => false,
        }
    }
}

/// Every card's code, by card id.
#[derive(Debug, Default)]
pub struct Registry {
    by_card: Vec<Option<CardImpl>>,
}

impl Registry {
    pub fn new(db: &CardDb) -> Registry {
        Registry { by_card: vec![None; db.len()] }
    }

    /// Register a card's code by name (Python's `@on(name, ...)`). A name not in the card database is an error, so
    /// a typo can't silently leave a card unimplemented.
    pub fn on(&mut self, db: &CardDb, name: &str, imp: CardImpl) -> Result<(), String> {
        let id = db.id(name).ok_or_else(|| format!("card code for {name:?}, which isn't in the card database"))?;
        let slot = &mut self.by_card[id.index()];
        if slot.is_some() {
            return Err(format!("{name}: card code registered twice"));
        }
        *slot = Some(imp);
        Ok(())
    }

    pub fn get(&self, c: CardId) -> Option<&CardImpl> {
        self.by_card.get(c.index())?.as_ref()
    }
}

// ------------------------------------------------------------------ triggers
/// A triggered ability's event and arguments, as data.
#[derive(Debug, Clone, PartialEq)]
pub enum Call {
    Etb { p: PlayerId, m: PermId },
    Leaves { m: PermId },
    Dies { m: PermId, cause: Sym },
    Player { p: PlayerId },
    Cast { caster: PlayerId, c: CardId },
    Attack { p: PlayerId, atk: Vec<PermId>, d: PlayerId },
    CombatDamage { p: PlayerId, a: PermId, d: PlayerId, dmg: i32 },
}

/// A triggered ability waiting to go on the stack (Python's `Trigger`).
#[derive(Debug, Clone, PartialEq)]
pub struct Trigger {
    pub controller: PlayerId,
    pub src: PermId,
    pub event: Event,
    pub call: Call,
    pub name: Option<Sym>,
    /// it triggers for sure (the ability language's), so no probe is needed
    pub known: bool,
    pub imp: Option<f64>,
    /// an enters trigger of a permanent that was cast
    pub cast_etb: bool,
}

// ------------------------------------------------------------------ AI options
/// A play the AI may make: Python's `(utility, label, fn)`. The look-ahead finds the same option again in a copy of
/// the game by its label (and which occurrence of that label it is), so the action is data, re-created in each copy.
#[derive(Debug, Clone)]
pub struct Opt {
    pub utility: f64,
    pub label: String,
    /// None: "stop" (hold the mana, or move on)
    pub act: Option<Action>,
}

/// What an option does. Engine verbs are variants; card abilities name their card's function and arguments.
#[derive(Debug, Clone)]
pub enum Action {
    Cast { card: CardId, zone: Sym },
    Ability { src: PermId, f: AbilityFn, arg: i64 },
}

/// A card ability chosen from `options`: returns whether it did anything (Python's `fn()` returning a truth value).
pub type AbilityFn = fn(&mut Game, PermId, PlayerId, i64) -> Res<bool>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::tests::db;

    #[test]
    fn every_event_in_the_export_is_known() {
        for c in db().iter() {
            for e in c.python_hooks.iter() {
                assert!(Event::from_name(e).is_some(), "{}: event {e:?}", c.name);
            }
        }
        assert_eq!(Event::from_name("end_step"), Some(Event::EndStep));
        assert!(Event::EndStep.is_trigger() && !Event::Cost.is_trigger());
        assert_eq!(Event::Main1.name(), "main1");
    }

    fn noop(_: &mut Game, _: Src, _: PlayerId) -> Res {
        Ok(())
    }

    #[test]
    fn the_registry_finds_card_code_by_card() {
        let db = db();
        let mut r = Registry::new(&db);
        r.on(&db, "Conjurer's Closet", CardImpl { end_step: Some(noop), ..Default::default() }).unwrap();
        let closet = db.id("Conjurer's Closet").unwrap();
        assert!(r.get(closet).unwrap().handles(Event::EndStep));
        assert!(!r.get(closet).unwrap().handles(Event::Etb));
        assert!(r.get(db.id("Sol Ring").unwrap()).is_none());
        assert!(r.on(&db, "Conjurers Closet", CardImpl::default()).is_err(), "a misspelt name");
        assert!(r.on(&db, "Conjurer's Closet", CardImpl::default()).is_err(), "registered twice");
    }
}
