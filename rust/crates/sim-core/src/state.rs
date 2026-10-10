//! The game state: `Game`, `Player`, `Perm` (nonland permanents and tokens), `Land`, and the stack.
//!
//! Layout rules (documents/rust-design.md):
//! - Everything is plain data. Objects refer to each other by id (`ids.rs`), never by reference, so
//!   `Game: Clone` copies a whole game, which is what the look-ahead does at every playout.
//! - Permanents and lands live in arenas on the `Game` (`perms`, `lands`), never removed or reused; a player's
//!   battlefield is the ordered list of ids it controls (`Player::perms`, `Player::lands`), in entry order as Python
//!   keeps it.
//! - Python's attributes set on the fly (`getattr(g, 'x', default)`) are declared fields here, with the same
//!   default. They are added as the code using them is ported; the runtime inventory in documents/rust-design.md
//!   lists them all.
//! - Tables Python keys by `id(permanent)` (end-of-turn pumps, granted keywords) become fields of the permanent.
//! - Card-specific state on a permanent (Python's `m.data` dict) is `PermData`: a short list of typed keys.

use crate::cards::{CardDb, Colors};
use crate::control::Humans;
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::rng::Rng;
use crate::settings::Settings;
use crate::sym::Sym;
use indexmap::{IndexMap, IndexSet};
use std::sync::Arc;

/// The most seats a game can have (cmd_dmg and similar per-seat tables are arrays this long).
pub const MAX_SEATS: usize = 4;

// ------------------------------------------------------------------ per-card state
/// The keys of Python's `m.data` / `land.data` dicts, as found by the runtime inventory. New ones are added as card
/// code is ported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataKey {
    Abducted,
    Act,
    Age,
    Alliance,
    Anim,
    Artifact,
    Attacks,
    Bestow,
    Bound,
    Burden,
    Charge,
    Construct,
    Copy,
    Counters,
    Crewed,
    Ctype,
    Elk,
    Escaped,
    Exerted,
    Exiled,
    FableGoblin,
    Final,
    Flip,
    Flipped,
    Frozen,
    GarlandHit,
    Held,
    Hidden,
    Hit,
    Hostage,
    Imprint,
    /// a land that entered this turn ("in")
    In,
    Indestr,
    /// (player, their turn count): indestructible until that player's next turn
    IndestrUntil,
    Kaldra,
    Kicks,
    Kws,
    Land,
    Legendary,
    Level,
    Lord,
    Lore,
    Lure,
    Manatok,
    MordDog,
    MustAttack,
    Oring,
    Prepared,
    Prevent,
    Prowess,
    Reflection,
    Ruin,
    Serenity,
    Stolen,
    TapOnReturn,
    Unblockable,
    Unbl,
    Used,
    Voice,
    Void,
    Voja,
    Wishes,
    Witch,
}

/// A value in per-card state.
#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    None,
    Bool(bool),
    Int(i64),
    Card(CardId),
    Perm(PermId),
    Land(LandId),
    Player(PlayerId),
    Str(Sym),
    Stamp(TurnStamp),
    List(Vec<Val>),
}

impl Val {
    /// Python truthiness, for code written `if m.data.get('x'):`
    pub fn truthy(&self) -> bool {
        match self {
            Val::None => false,
            Val::Bool(b) => *b,
            Val::Int(n) => *n != 0,
            Val::Str(s) => !s.is_empty(),
            Val::List(v) => !v.is_empty(),
            _ => true,
        }
    }

    pub fn int(&self) -> i64 {
        match self {
            Val::Int(n) => *n,
            Val::Bool(b) => *b as i64,
            _ => 0,
        }
    }
}

/// Python's `data` dict on a permanent or land. Usually empty or a key or two, so a short list.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PermData(Vec<(DataKey, Val)>);

impl PermData {
    pub fn get(&self, k: DataKey) -> Option<&Val> {
        self.0.iter().find(|x| x.0 == k).map(|x| &x.1)
    }

    pub fn get_mut(&mut self, k: DataKey) -> Option<&mut Val> {
        self.0.iter_mut().find(|x| x.0 == k).map(|x| &mut x.1)
    }

    /// `data.get(k)` as Python tests it
    pub fn truthy(&self, k: DataKey) -> bool {
        self.get(k).is_some_and(Val::truthy)
    }

    pub fn int(&self, k: DataKey) -> i64 {
        self.get(k).map_or(0, Val::int)
    }

    pub fn set(&mut self, k: DataKey, v: Val) {
        match self.get_mut(k) {
            Some(x) => *x = v,
            None => self.0.push((k, v)),
        }
    }

    pub fn remove(&mut self, k: DataKey) -> Option<Val> {
        let i = self.0.iter().position(|x| x.0 == k)?;
        Some(self.0.remove(i).1)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ------------------------------------------------------------------ small value types
/// Python's `turn_stamp(g)`: (round, the active player). Used for "this turn" flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TurnStamp {
    pub round: u32,
    pub active: Option<PlayerId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Step {
    #[default]
    Start,
    Main1,
    Combat,
    Main2,
    End,
}

/// Mana that floats until a step or turn ends (Python's `floatR`, `floatA` ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Floating {
    pub w: u32,
    pub u: u32,
    pub b: u32,
    pub r: u32,
    pub g: u32,
    pub c: u32,
    /// any colour (rituals, the ability language)
    pub any: u32,
}

// ------------------------------------------------------------------ permanents and lands
/// A nonland permanent or a token (Python's `Perm`, every slot kept).
#[derive(Debug, Clone, PartialEq)]
pub struct Perm {
    pub id: PermId,
    /// None for a token
    pub cd: Option<CardId>,
    /// the controller (Python's `owner`)
    pub owner: PlayerId,
    /// the owner (Python's `orig`)
    pub orig: PlayerId,
    pub token: bool,
    /// the card's name, or the token's
    pub name: Sym,
    pub tapped: bool,
    /// summoning sick
    pub sick: bool,
    pub pow: i32,
    pub tgh: i32,
    pub fly: bool,
    /// deathtouch
    pub dt: bool,
    pub vig: bool,
    /// lifelink (Python's `life`)
    pub lifelink: bool,
    /// +1/+1 counters (negative: -1/-1 counters)
    pub plus: i32,
    pub undying: bool,
    pub army: bool,
    pub warrior: bool,
    pub noatk: bool,
    pub phased: bool,
    /// the creature an Equipment or Aura is attached to
    pub attached: Option<PermId>,
    pub age: u32,
    /// its abilities are turned off
    pub neutered: bool,
    pub is_cmd: bool,
    /// Python's `phys`: the physical card a copy came from
    pub phys: Option<CardId>,
    /// a temporary token (exiled at end of turn)
    pub temp: bool,
    pub loyalty: Option<i32>,
    pub loyalty_used: Option<TurnStamp>,
    /// a token's colours
    pub colors: Colors,
    /// a token's subtypes (lower case)
    pub ttypes: Vec<Sym>,
    pub data: PermData,
    pub born: u32,
    /// is it on the battlefield? (Python: `m in m.owner.perms`)
    pub on_bf: bool,
    /// until end of turn: power/toughness change and granted keywords (Python's `g.eot_pt`, `g.eot_kw`)
    pub eot_pt: (i32, i32),
    pub eot_kw: Vec<Sym>,
}

/// A land on the battlefield (Python's `Land`).
#[derive(Debug, Clone, PartialEq)]
pub struct Land {
    pub id: LandId,
    pub cd: CardId,
    pub owner: PlayerId,
    pub tapped: bool,
    pub data: PermData,
    pub on_bf: bool,
}

// ------------------------------------------------------------------ players
#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub id: PlayerId,
    /// the deck key ('sauron', 'krenko-mono-red-goblins')
    pub key: Sym,
    /// the name shown in logs ('Sauron', 'Krenko')
    pub name: Sym,
    pub ident: Colors,
    pub cmd: CardId,
    pub deck_names: Arc<[CardId]>,
    /// top card last, as in Python
    pub library: Vec<CardId>,
    pub hand: Vec<CardId>,
    pub gy: Vec<CardId>,
    pub exile: Vec<CardId>,
    pub lands: Vec<LandId>,
    pub perms: Vec<PermId>,
    pub life: i32,
    pub alive: bool,
    pub turns: u32,
    pub cmd_in_zone: bool,
    pub tax: u32,
    /// commander damage taken, by the seat whose commander dealt it
    pub cmd_dmg: [i32; MAX_SEATS],
    pub treasures: u32,
    pub clues: u32,
    pub foods: u32,
    pub poison: u32,
    pub floating: Floating,
    pub extra_combats: u32,
    pub combat_no: u32,
    pub spells_this_turn: u32,
    pub land_played: bool,
    pub lands_played: u32,
    pub land_turn: i32,
    pub pump: i32,
    pub pumpadd: i32,
    pub trample: bool,
    pub combo_tried: bool,
    pub haste_all: bool,
    pub decked: bool,
    pub killer: Option<PlayerId>,
    pub last_src: Option<PlayerId>,
    pub ring_prot: bool,
    pub life_locked: bool,
    /// the kind of the last life loss (for how the player was eliminated)
    pub last_kind: Sym,
    pub death_kind: Sym,
    /// this player's own turn count when it last dealt damage to an opponent
    pub hit_turn: i32,
    /// life lost / gained this turn (Bloodsoaked Insight, Archfiend, Y'shtola)
    pub lost_turn: Option<(TurnStamp, i32)>,
    pub gained_turn: Option<(TurnStamp, i32)>,
    /// emblems from planeswalker ultimates ('tamiyo', 'nissa' ...)
    pub emblems: Vec<Sym>,
    /// cards taken with Opposition Agent (castable with mana of any type)
    pub agent_ids: Vec<CardId>,
    /// the last turn a permanent of this player's left the battlefield (revolt)
    pub left_turn: Option<TurnStamp>,
    /// once-per-turn flags (Python's `flag_turn`)
    pub flag_turn: IndexMap<Sym, TurnStamp>,
    /// the AI's memory of who hurt it (brain.note_damage)
    pub grudge: IndexMap<Sym, f64>,
    /// cards cast this game
    pub cast_names: IndexSet<CardId>,
    /// cards this player has seen (drawn, tutored)
    pub seen: IndexSet<CardId>,
    /// the turn draws are counted for (Narset, Parter of Veils)
    pub draw_st: Option<TurnStamp>,
    /// cards drawn this turn
    pub draw_n: u32,
    /// this turn's first draw (miracle)
    pub miracle: Option<(TurnStamp, CardId)>,
    /// Urabrask, Heretic Praetor: the next draw this turn is exiled instead
    pub urabrask: Option<TurnStamp>,
    /// cards exiled with "you may play them this turn" (held in hand)
    pub impulse: Vec<CardId>,
    /// key spells milled (reports)
    pub milled_keys: Vec<CardId>,
    /// The Ozolith: counters it holds
    pub ozolith_counters: i32,
    /// this player's bombs that were removed
    pub removed_bombs: Vec<CardId>,
    /// a regeneration shield this turn
    pub regen_turn: Option<TurnStamp>,
    /// creatures borrowed until end of turn (free to sacrifice)
    pub borrowed: Vec<PermId>,
    /// this player's permanents removed, by name
    pub lost_names: IndexMap<Sym, u32>,
    /// the last turn this player discarded
    pub discarded_turn: Option<TurnStamp>,
    /// extra land drops this turn (Explore)
    pub extra_land_now: u32,
    /// Hope of Ghirapur: no noncreature spells until that player's turn count passes
    pub hope_lock: Option<(PlayerId, u32)>,
    /// Mistrise Village: the next spell this turn can't be countered
    pub mistrise_next: Option<TurnStamp>,
    /// pact costs owed at the next upkeep (generic, pips)
    pub pact_debts: Vec<(u32, String)>,
    /// Pact of Negation casts to pay for
    pub pacts: u32,
    /// Glimpse of Tomorrow-style cast draws
    pub glimpse: bool,
    /// spells cast this turn
    pub turn_casts: Option<(TurnStamp, Vec<CardId>)>,
    /// instants and sorceries cast this turn
    pub is_cast_n: Option<(TurnStamp, i32)>,
    /// Galvanic Iteration: copy the next instant or sorcery this turn
    pub galvanic: Option<TurnStamp>,
    /// Ral, Storm Conduit -2: copy the next instant or sorcery this turn
    pub ral_copy: Option<TurnStamp>,
    /// cards taken from other players' decks (Gonti, Hostage Taker): whose they are
    pub stolen: IndexMap<CardId, PlayerId>,
    /// Yawgmoth's Will this turn
    pub yawg: bool,
    /// the graveyard as it was when Yawgmoth's Will resolved
    pub yawg_gy: Vec<CardId>,
    /// Mana Drain: colourless mana at the next main phase
    pub drain_mana: u32,
    /// Arcane Denial: draws at the next upkeep
    pub delayed_draws: u32,
    /// the tutor being chosen puts the card on top (tutor_pick reads it)
    pub to_top: bool,
    /// the permanent number (born) when this player's last turn ended
    pub last_turn_end: u32,
    /// the graveyard at the start of this turn
    pub gy_start: Vec<CardId>,
    /// Glacial Chasm's age counters (cumulative upkeep)
    pub chasm_age: i32,
    /// suspended cards (Profane Tutor)
    pub suspended: Vec<(CardId, u32)>,
    /// cards playable until the end of a later turn (Prosper): (card, the turn count it lasts to)
    pub impulse_long: Vec<(CardId, u32)>,
    /// Erebos's draw this turn
    pub erebos_t: Option<TurnStamp>,
    /// no maximum hand size this turn (its turn count)
    pub nomax_turn: Option<u32>,
    /// turns this player skips (Ral Zarek's -7)
    pub skip_turns: u32,
    /// Venser, the Sojourner's -1: creatures can't be blocked this turn
    pub unbl_all: Option<TurnStamp>,
    /// Elspeth's emblem: creatures have flying
    pub elspeth_emblem: bool,
    /// the turn each milestone was first reached (reports: 'combo', 'engine', 'bomb_by' ...)
    pub milestone: IndexMap<Sym, u32>,
    /// compiled activated abilities used this round: (permanent, ability) -> (round stamp, uses)
    pub act_uses: IndexMap<(PermId, u16), (TurnStamp, u32)>,
    /// counters for reports (Python's `p.stats`)
    pub stats: IndexMap<Sym, i64>,
    /// rebound spells (Ephemerate) cast again at the next upkeep
    pub rebound: Vec<CardId>,
    /// the turn count Jace's Archivist was last used on
    pub arch_t: Option<u32>,
}

impl Player {
    pub fn new(id: PlayerId, key: Sym, name: Sym, ident: Colors, cmd: CardId, cards: Arc<[CardId]>) -> Player {
        let mut library: Vec<CardId> = cards.iter().copied().collect();
        if let Some(i) = library.iter().position(|&c| c == cmd) {
            library.remove(i);
        }
        Player {
            id,
            key,
            name,
            ident,
            cmd,
            deck_names: cards,
            library,
            hand: vec![],
            gy: vec![],
            exile: vec![],
            lands: vec![],
            perms: vec![],
            life: 40,
            alive: true,
            turns: 0,
            cmd_in_zone: true,
            tax: 0,
            cmd_dmg: [0; MAX_SEATS],
            treasures: 0,
            clues: 0,
            foods: 0,
            poison: 0,
            floating: Floating::default(),
            extra_combats: 0,
            combat_no: 0,
            spells_this_turn: 0,
            land_played: false,
            lands_played: 0,
            land_turn: -1,
            pump: 0,
            pumpadd: 0,
            trample: false,
            combo_tried: false,
            haste_all: false,
            decked: false,
            killer: None,
            last_src: None,
            ring_prot: false,
            life_locked: false,
            last_kind: "other",
            death_kind: "",
            hit_turn: -1,
            lost_turn: None,
            gained_turn: None,
            emblems: vec![],
            agent_ids: vec![],
            left_turn: None,
            flag_turn: IndexMap::new(),
            grudge: IndexMap::new(),
            cast_names: IndexSet::new(),
            seen: IndexSet::new(),
            draw_st: None,
            draw_n: 0,
            miracle: None,
            urabrask: None,
            impulse: vec![],
            milled_keys: vec![],
            ozolith_counters: 0,
            removed_bombs: vec![],
            regen_turn: None,
            borrowed: vec![],
            lost_names: IndexMap::new(),
            discarded_turn: None,
            extra_land_now: 0,
            hope_lock: None,
            mistrise_next: None,
            pact_debts: vec![],
            pacts: 0,
            glimpse: false,
            turn_casts: None,
            is_cast_n: None,
            galvanic: None,
            ral_copy: None,
            stolen: IndexMap::new(),
            yawg: false,
            yawg_gy: vec![],
            drain_mana: 0,
            delayed_draws: 0,
            to_top: false,
            last_turn_end: 0,
            gy_start: vec![],
            chasm_age: 0,
            suspended: vec![],
            impulse_long: vec![],
            erebos_t: None,
            nomax_turn: None,
            skip_turns: 0,
            unbl_all: None,
            elspeth_emblem: false,
            milestone: IndexMap::new(),
            act_uses: IndexMap::new(),
            rebound: vec![],
            arch_t: None,
            stats: IndexMap::new(),
        }
    }

    pub fn stat(&mut self, name: Sym, n: i64) {
        *self.stats.entry(name).or_insert(0) += n;
    }
}

// ------------------------------------------------------------------ the stack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackKind {
    Spell,
    Ability,
    Trigger,
}

/// A spell's targets and choices (Python's `ctx` dict). Fields are added as the code that sets them is ported.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ctx {
    /// the permanent it targets
    pub target: Option<PermId>,
    /// the player it targets (burn to the face)
    pub face: Option<PlayerId>,
    /// a counterspell: the stack item it counters (by `StackItem::id`)
    pub counter: Option<u32>,
    /// an ability or trigger: its source
    pub source: Option<PermId>,
    /// a trigger on the stack: what it does when it resolves
    pub trigger: Option<Box<crate::hooks::Trigger>>,
    /// X
    pub x: i32,
    /// the owner of a card cast from another player's deck (it goes back to their graveyard)
    pub owner: Option<PlayerId>,
    /// exiled instead of going to the graveyard after it resolves
    pub exile_after: bool,
    /// Desertion: the countered artifact or creature enters under this player's control
    pub desert_to: Option<PlayerId>,
    /// the removal kind, when it differs from the card's `rem` tag (a modal spell)
    pub rem_kind: Option<Sym>,
    /// an alternative cost paid the life it would cost (Snuff Out ...)
    pub paid_otherwise: bool,
    /// a reanimation spell's target and its value (counter decisions)
    pub rean_target: Option<CardId>,
    pub rean_value: f64,
    /// Muldrotha: the permanent type it was cast as
    pub muld_type: Option<Sym>,
    /// Mizzix's Mastery overloaded
    pub overload: bool,
    /// Toxic Deluge's X when a person chose it
    pub deluge_x: Option<i32>,
    /// Cyclonic Rift-style wipes of one player (rebuke): the player
    pub victim: Option<PlayerId>,
    /// a reanimation spell's target is in this player's graveyard
    pub rean_src: Option<PlayerId>,
}

/// A spell or ability on the stack (Python's `StackItem`).
#[derive(Debug, Clone, PartialEq)]
pub struct StackItem {
    /// unique within the game, so other items (a counterspell) can name it
    pub id: u32,
    pub controller: PlayerId,
    pub card: Option<CardId>,
    pub ctx: Ctx,
    /// where a spell was cast from ('hand', 'cmd', 'gy' ...); 'trigger' for triggers
    pub zone: Sym,
    /// how much it matters to everyone, and per player
    pub imp: f64,
    pub aff: Vec<(PlayerId, f64)>,
    /// resolved by the engine's own casting code (false: by its caster's code)
    pub generic: bool,
    pub kind: StackKind,
    pub name: String,
    /// seats that chose not to counter it
    pub passed: Vec<PlayerId>,
    pub countered: bool,
    pub countered_by: Option<CardId>,
}

impl StackItem {
    /// how much it matters to q (Python's `top.aff.get(q, top.imp)`)
    pub fn value_to(&self, q: PlayerId) -> f64 {
        self.aff.iter().find(|x| x.0 == q).map_or(self.imp, |x| x.1)
    }
}

// ------------------------------------------------------------------ the game
#[derive(Debug, Clone)]
pub struct Game {
    pub db: Arc<CardDb>,
    pub settings: Arc<Settings>,
    /// the card code (hooks.rs)
    pub registry: Arc<crate::hooks::Registry>,
    pub players: Vec<Player>,
    pub perms: Vec<Perm>,
    pub lands: Vec<Land>,
    /// play decisions (Python's `g.rng`, seeded `play:{seed}`)
    pub rng: Rng,
    pub round: u32,
    pub active: Option<PlayerId>,
    pub step: Step,
    pub over: bool,
    pub winner: Option<PlayerId>,
    pub wintype: Option<Sym>,
    /// eliminated players, with the round they went out
    pub elim: Vec<(PlayerId, u32)>,
    pub monarch: Option<PlayerId>,
    /// bumped whenever a permanent enters, leaves or changes control (Python's `bf_ver`)
    pub bf_ver: u64,
    /// Auras on the battlefield that the card code tracks (Python's `g.auras`)
    pub auras: Vec<PermId>,
    /// Disruptor Flutes and the card each names
    pub flutes: Vec<(PermId, CardId)>,
    /// Panoptic Mirror: the cards imprinted on each mirror
    pub imprint: Vec<(PermId, CardId)>,
    /// Shadowspear's activation: the turn opponents' permanents lose hexproof and indestructible
    pub spear: Option<TurnStamp>,
    /// the card being paid for now (Mishra's Workshop mana is for artifact spells only): Python's `PAY_FOR`
    pub pay_for: Option<CardId>,
    /// the coloured pips the source being tapped paid for (read by on-tap card code): Python's `TAP_COLS`
    pub tap_cols: Colors,
    /// triggered abilities of a permanent that entered as it was cast see it was cast (Python's `last_cast_etb`)
    pub last_cast_etb: bool,
    /// a creature entering doesn't trigger its enter effects (Venser cast as a counter)
    pub skip_etb: bool,
    /// the permanent whose removal is happening (Skyclave Apparition remembers what it exiled)
    pub rem_src: Option<PermId>,
    /// destruction can't be regenerated from (Wrath of God)
    pub noregen: bool,
    /// who destroyed the permanents dying now (Karmic Justice)
    pub destroyer: Option<PlayerId>,
    /// Notion Thief's draw replacements already applied to this draw (614.5)
    pub thief_chain: Vec<PermId>,
    /// the spell being cast now: (card, its choices, zone) (cast triggers see its targets and X)
    pub cur_cast: Option<(CardId, Ctx, Sym)>,
    /// an Aura's target chosen as it was cast
    pub cast_target: Option<PermId>,
    /// what a sacrificed permanent was, for spells that count it: (mana value, toughness)
    pub sac_snapshot: Option<(u32, i32)>,
    /// Silence-style lock: (the turn, the player it doesn't stop)
    pub silence: Option<(TurnStamp, PlayerId)>,
    /// undying / persist returns this turn, by card (RETURN_CAP)
    pub returns_turn: Option<TurnStamp>,
    pub returns: IndexMap<CardId, u32>,
    /// the permanent dying now (Auras that care how their creature left)
    pub dying: Option<PermId>,
    /// the last turn a creature died (Barad-dûr)
    pub died_turn: Option<TurnStamp>,
    /// permanents entered, numbered in order (born)
    pub enter_no: u32,
    /// permanents that entered this turn
    pub entered: Vec<(TurnStamp, PermId)>,
    /// creatures in combat now (Settle the Wreckage, Prophesied End)
    pub in_combat: Vec<PermId>,
    /// an Aura put onto the battlefield without being cast (Zur): placed by its card code
    pub aura_put: bool,
    /// the last removal: (card, owner, kind, who removed it)
    pub last_removed: Option<(Option<CardId>, PlayerId, Sym, Option<PlayerId>)>,
    /// wipes resolved so far: creatures destroyed together die simultaneously
    pub batch: u32,
    /// Marchesa's return triggers are in play
    pub marchesa_on: bool,
    /// Chatterfang doesn't add Squirrels (its own tokens)
    pub no_fang: bool,
    /// a permanent with compiled abilities has been on the battlefield (the interpreter is active)
    pub dsl_on: bool,
    /// Jeska's Will always adds mana (the Underworld Breach line)
    pub jeska_mana: bool,
    /// how deep the ability language's triggers are nested (MAX_DEPTH stops runaway chains)
    pub dsl_depth: u32,
    /// creatures with a power/toughness rule of their own (card code; Python's `g.selfpt`)
    pub selfpt: bool,
    /// blocking creatures in the combat being resolved
    pub blocking: Vec<PermId>,
    /// a fog this turn (Spore Frog): no combat damage
    pub fog: Option<TurnStamp>,
    /// attacking creatures card code made during attack triggers (Python returns them from CI.fire)
    pub new_attackers: Vec<PermId>,
    /// the game ran out of engine steps (a runaway loop) and was stopped
    pub stopped: bool,
    /// a game with one player and no opponents to beat (Python's `goldfish`)
    pub goldfish: bool,
    /// Garland, Royal Kidnapper's steals are in play (Python's `g.garland`)
    pub garland: bool,
    /// permanents with hand-written card code, in entry order (Python's `g.hooks`)
    pub hooks: Vec<PermId>,
    pub stack: Vec<StackItem>,
    /// items ever put on the stack: tells whether a player with priority did something; also the next item's id
    pub stack_pushes: u32,
    /// 'probe' while finding out which pending triggers really trigger (Python's `trig_mode`)
    pub trig_probe: bool,
    /// the stack item of the trigger being resolved (by id)
    pub trig_current: Option<u32>,
    /// the counterspell that countered the last spell (Python's `LAST_COUNTER`)
    pub last_counter: Option<CardId>,
    /// Hullbreaker Horror bounced the spell instead of countering it
    pub bounced_spell: bool,
    /// a spell made uncounterable by Mistrise Village: (card, turn)
    pub unc_cast: Option<(CardId, TurnStamp)>,
    /// practice mode: the person paid for a counterspell from their pool
    pub free_counter: bool,
    pub trig_queue: Vec<crate::hooks::Trigger>,
    /// > 0 while a spell or effect is resolving: its triggers wait until it's done
    pub resolving: u32,
    /// engine steps taken, and allowed (Python's `work`, `work_cap`; see `tick`)
    pub work: u64,
    pub work_cap: u64,
    /// look-ahead copies: stop once the table has this many permanents
    pub board_cap: Option<usize>,
    /// a copy being played forward by the look-ahead
    pub in_search: bool,
    /// play-by-play lines when tracing a game (`--trace`)
    pub log: Option<Vec<String>>,
    /// practice mode's people; a copy of the game never has any (see `control::Humans`)
    pub humans: Humans,
    /// the X paid for the last X spell (card code that reads it: Walking Ballista ...)
    pub last_x: i32,
    /// a look-ahead copy playing out one attack plan: (defender, 'filtered' | 'all' | 'none')
    pub forced_attack: Option<(PlayerId, Sym)>,
    /// look-ahead decisions taken in this game, and engine steps its playouts used (the per-game caps)
    pub search_n: u32,
    pub search_work: u64,
}

impl Game {
    pub fn new(
        db: Arc<CardDb>,
        registry: Arc<crate::hooks::Registry>,
        settings: Arc<Settings>,
        players: Vec<Player>,
        rng: Rng,
    ) -> Game {
        let work_cap = settings.game_work;
        Game {
            db,
            settings,
            registry,
            players,
            perms: vec![],
            lands: vec![],
            rng,
            round: 0,
            active: None,
            step: Step::Start,
            over: false,
            winner: None,
            wintype: None,
            elim: vec![],
            monarch: None,
            bf_ver: 0,
            auras: vec![],
            flutes: vec![],
            imprint: vec![],
            spear: None,
            pay_for: None,
            tap_cols: Colors::NONE,
            last_cast_etb: false,
            skip_etb: false,
            rem_src: None,
            noregen: false,
            destroyer: None,
            thief_chain: vec![],
            cur_cast: None,
            cast_target: None,
            sac_snapshot: None,
            silence: None,
            returns_turn: None,
            returns: IndexMap::new(),
            dying: None,
            died_turn: None,
            enter_no: 0,
            entered: vec![],
            in_combat: vec![],
            aura_put: false,
            last_removed: None,
            batch: 0,
            marchesa_on: false,
            no_fang: false,
            dsl_on: false,
            jeska_mana: false,
            dsl_depth: 0,
            selfpt: false,
            blocking: vec![],
            fog: None,
            new_attackers: vec![],
            stopped: false,
            goldfish: false,
            garland: false,
            hooks: vec![],
            stack: vec![],
            stack_pushes: 0,
            trig_probe: false,
            trig_current: None,
            last_counter: None,
            bounced_spell: false,
            unc_cast: None,
            free_counter: false,
            trig_queue: vec![],
            resolving: 0,
            work: 0,
            work_cap,
            board_cap: None,
            in_search: false,
            log: None,
            humans: Humans::none(),
            last_x: 0,
            forced_attack: None,
            search_n: 0,
            search_work: 0,
        }
    }

    pub fn player(&self, p: PlayerId) -> &Player {
        &self.players[p.index()]
    }

    pub fn player_mut(&mut self, p: PlayerId) -> &mut Player {
        &mut self.players[p.index()]
    }

    pub fn perm(&self, m: PermId) -> &Perm {
        &self.perms[m.index()]
    }

    pub fn perm_mut(&mut self, m: PermId) -> &mut Perm {
        &mut self.perms[m.index()]
    }

    pub fn land(&self, l: LandId) -> &Land {
        &self.lands[l.index()]
    }

    pub fn land_mut(&mut self, l: LandId) -> &mut Land {
        &mut self.lands[l.index()]
    }

    /// Python's `g.opps(p)`: the other living players, in seat order
    pub fn opps(&self, p: PlayerId) -> impl Iterator<Item = PlayerId> + '_ {
        self.players.iter().filter(move |q| q.alive && q.id != p).map(|q| q.id)
    }

    /// Python's `g.after(p)`: every other seat, in turn order starting after p
    pub fn after(&self, p: PlayerId) -> impl Iterator<Item = PlayerId> + '_ {
        let n = self.players.len();
        (1..n).map(move |k| PlayerId(((p.index() + k) % n) as u8))
    }

    pub fn turn_stamp(&self) -> TurnStamp {
        TurnStamp { round: self.round, active: self.active }
    }

    /// One engine step (Python's `tick`): a game, or a look-ahead copy, that runs past its budget stops.
    pub fn tick(&mut self) -> crate::flow::Res {
        self.work += 1;
        if self.work > self.work_cap {
            return Err(crate::flow::Stop::OutOfWork);
        }
        if let Some(cap) = self.board_cap
            && self.players.iter().map(|q| q.perms.len()).sum::<usize>() > cap
        {
            return Err(crate::flow::Stop::OutOfWork);
        }
        Ok(())
    }

    /// A new permanent of `cd` (or a token, `cd` None) for player p: put in the arena, not yet on the battlefield
    /// (the engine's `enter` does that). Python's `Perm.__init__`.
    pub fn new_perm(&mut self, p: PlayerId, cd: Option<CardId>, token_name: Sym, pow: i32, tgh: i32) -> PermId {
        let id = PermId(self.perms.len() as u32);
        let mut m = Perm {
            id,
            cd,
            owner: p,
            orig: p,
            token: cd.is_none(),
            name: token_name,
            tapped: false,
            sick: true,
            pow,
            tgh,
            fly: false,
            dt: false,
            vig: false,
            lifelink: false,
            plus: 0,
            undying: false,
            army: false,
            warrior: false,
            noatk: false,
            phased: false,
            attached: None,
            age: 0,
            neutered: false,
            is_cmd: false,
            phys: None,
            temp: false,
            loyalty: None,
            loyalty_used: None,
            colors: Colors::NONE,
            ttypes: vec![],
            data: PermData::default(),
            born: 0,
            on_bf: false,
            eot_pt: (0, 0),
            eot_kw: vec![],
        };
        if let Some(c) = cd {
            use crate::tag::Tag;
            let d = self.db.get(c);
            let t = &d.tags;
            m.name = crate::sym::intern(&d.name);
            m.pow = d.pow;
            m.tgh = d.tgh;
            m.fly = t.has(Tag::Fly);
            m.dt = t.has(Tag::Dt);
            m.vig = t.has(Tag::Vig);
            m.lifelink = t.has(Tag::Lifelink);
            m.warrior = t.has(Tag::Warrior);
            m.noatk = t.has(Tag::Noatk);
        }
        self.perms.push(m);
        id
    }

    /// A land put onto p's battlefield, without its enter effects (the engine's land play adds those).
    pub fn add_land_entering(&mut self, p: PlayerId, cd: CardId, tapped: bool) -> LandId {
        self.add_land(p, cd, tapped)
    }

    pub fn add_land(&mut self, p: PlayerId, cd: CardId, tapped: bool) -> LandId {
        let id = LandId(self.lands.len() as u32);
        self.lands.push(Land { id, cd, owner: p, tapped, data: PermData::default(), on_bf: true });
        self.player_mut(p).lands.push(id);
        id
    }

    /// Is m a creature now (Python's `Perm.creature`): a token, a creature card, or animated.
    pub fn is_creature(&self, m: PermId) -> bool {
        let m = self.perm(m);
        m.token || m.cd.is_some_and(|c| self.db.get(c).creature) || m.data.truthy(DataKey::Anim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sym::intern;

    pub fn four_seat_game() -> Game {
        let t1: Vec<&str> = crate::testkit::decks()
            .iter()
            .filter(|d| d.tier.as_deref() == Some("t1"))
            .take(3)
            .map(|d| &*d.key)
            .collect();
        crate::testkit::table(&["sauron", t1[0], t1[1], t1[2]])
    }

    #[test]
    fn a_table_of_four_is_seated_with_full_libraries() {
        let g = four_seat_game();
        assert_eq!(g.players.len(), 4);
        for p in &g.players {
            assert_eq!(p.library.len() + 1, p.deck_names.len(), "{}", p.key);
            assert!(!p.library.contains(&p.cmd), "the commander starts in the command zone");
            assert_eq!(p.life, 40);
        }
        assert_eq!(g.after(PlayerId(2)).collect::<Vec<_>>(), vec![PlayerId(3), PlayerId(0), PlayerId(1)]);
    }

    #[test]
    fn a_new_permanent_takes_its_cards_stats_and_keywords() {
        let mut g = four_seat_game();
        let atraxa = g.db.id("Atraxa, Grand Unifier").unwrap();
        let m = g.new_perm(PlayerId(0), Some(atraxa), intern("Token"), 1, 1);
        let p = g.perm(m);
        assert_eq!(
            (p.name, p.pow, p.tgh, p.fly, p.dt, p.vig, p.lifelink),
            ("Atraxa, Grand Unifier", 7, 7, true, true, true, true)
        );
        assert!(g.is_creature(m));
        let z = g.new_perm(PlayerId(1), None, intern("Zombie"), 2, 2);
        assert!(g.perm(z).token && g.is_creature(z));
        assert_ne!(m, z);
    }

    #[test]
    fn per_card_state_reads_like_python_dicts() {
        let mut d = PermData::default();
        assert!(!d.truthy(DataKey::Lore));
        d.set(DataKey::Lore, Val::Int(2));
        d.set(DataKey::Lore, Val::Int(3));
        assert_eq!(d.int(DataKey::Lore), 3);
        assert!(d.truthy(DataKey::Lore));
        assert_eq!(d.remove(DataKey::Lore), Some(Val::Int(3)));
        assert!(d.is_empty());
    }

    #[test]
    fn running_out_of_steps_stops_the_game() {
        let mut g = four_seat_game();
        g.work_cap = 3;
        assert!(g.tick().is_ok() && g.tick().is_ok() && g.tick().is_ok());
        assert_eq!(g.tick(), Err(crate::flow::Stop::OutOfWork));
    }
}
