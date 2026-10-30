//! The game state: `Game`, `Player`, `Perm` (every permanent and token, land or not), and the stack.
//!
//! Layout rules (documents/archive/rust-design.md):
//! - Everything is plain data. Objects refer to each other by id (`ids.rs`), never by reference, so
//!   `Game: Clone` copies a whole game, which is what the look-ahead does at every playout.
//! - Permanents and lands live in arenas on the `Game` (`perms`, `lands`), never removed or reused; a player's
//!   battlefield is the ordered list of ids it controls (`Player::perms`, `Player::lands`), in entry order as Python
//!   keeps it.
//! - Python's attributes set on the fly (`getattr(g, 'x', default)`) are declared fields here, with the same
//!   default. They are added as the code using them is ported; the runtime inventory in
//! documents/archive/rust-design.md
//!   lists them all.
//! - Tables Python keys by `id(permanent)` (end-of-turn pumps, granted keywords) become fields of the permanent.
//! - Card-specific state on a permanent (Python's `m.data` dict) is `PermData`: a short list of typed keys.

use crate::cards::{CardDb, Colors, Types};
use crate::control::{Humans, Link};
use crate::ids::{CardId, PermId, PlayerId};
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
    /// cast for its blitz cost (Jaxis, the Troublemaker)
    Blitz,
    Bound,
    Burden,
    Charge,
    /// fate counters (Oblivion Stone)
    Fate,
    Construct,
    Copy,
    /// Spark Double entered as a copy (its own enter hook doesn't copy again)
    Copied,
    Counters,
    Crewed,
    Ctype,
    /// --- Tiers 4/5 (winota): what a permanent chose as it entered (Windcrag Siege: "mardu" or "jeskai"; Greymond,
    /// Avacyn's Stalwart: the two abilities it grants), a `Val::Str` or a `Val::List` of them
    Mode,
    /// (Dovin's controller, their turn count): damage to and from it is prevented until their next turn
    Dovin,
    /// a land made a creature by Druid Class's level 3
    DruidClass,
    /// Mogg War Marshal's echo was dealt with ('done')
    Echo,
    /// t0-zurgo: goaded (a `Val::List` of the goader and the goader's turn count)
    Goaded,
    /// t0-zurgo: a myriad token copy, exiled at end of combat
    MyriadTok,
    /// a token sacrificed at the beginning of the next end step (Lagomos, Hand of Hatred)
    EotSac,
    /// Ferocity of the Hunt was on the creature as it died: its card returns to the battlefield tapped
    Ferocity,
    /// Kaya's Ghostform was on the creature as it left: its card returns under this player (a `Val::Player`)
    Ghostform,
    /// Rabid Attack: the turn (a `Val::Stamp`) the creature draws a card when it dies
    Rabid,
    /// Dread Slaver damaged the creature (a `Val::Player`, the Slaver's controller)
    Slaver,
    /// Grave Betrayal: the creatures to return at the next end step, as [card, owner] pairs
    Betrayed,
    /// oil counters (Armored Scrapgorger)
    Oil,
    /// Ragavan was cast for its dash cost (returned to hand at the end step)
    Dash,
    Elk,
    Escaped,
    Exerted,
    Exiled,
    /// Whipcorder face down: its printed (power, toughness) (galadriel.enter_face_down)
    Facedown,
    /// Cho-Manno's Blessing: the colour chosen as it entered (on the Aura); a colour waiting for the next such Aura (on its host)
    ChosenColor,
    /// Shardmage's Rescue: the turn it entered (a `Val::Stamp`)
    Entered,
    /// a fetch land put onto the battlefield tapped by Primeval Titan: its {T} cost can not be paid until it untaps
    TitanTapped,
    FableGoblin,
    Final,
    /// --- Tiers 4/5 (chulane-kinnan): a creature card on the battlefield that is not a creature (Enduring Vitality
    /// returned as an enchantment)
    NotCreature,
    /// --- Tiers 4/5 (edgar): Clavileño made it a Demon with "when this creature dies, draw a card and create a tapped
    /// 4/3 Vampire Demon"
    DemonGift,
    /// --- Tiers 4/5 (edgar): bloodline counters (Edgar Markov's Coffin); the city's blessing (Twilight Prophet)
    Bloodline,
    Blessing,
    Flip,
    Flipped,
    /// a Wizard token made by Kuja, Genome Sorcerer: it pings when its controller casts a noncreature spell
    KujaWiz,
    Frozen,
    GarlandHit,
    Held,
    Hidden,
    Hit,
    Hostage,
    /// Venat, Heart of Hydaelyn has transformed into Hydaelyn, the Mothercrystal
    Hydaelyn,
    Imprint,
    /// a land that entered this turn ("in")
    In,
    Indestr,
    /// --- Tiers 1/2 (windgrace): ice counters removed from Dark Depths (it enters with ten)
    IceRemoved,
    /// Palantír of Orthanc's influence counters
    Influence,
    /// (player, their turn count): indestructible until that player's next turn
    IndestrUntil,
    /// a token copy made by Jaxis, the Troublemaker (sacrificed at the end step)
    JaxisCopy,
    Kaldra,
    Kicks,
    Kws,
    Land,
    Legendary,
    Level,
    Lord,
    /// the walker framework's loyalty activations this round (Python's third item of `loyalty_used`; Oath of Teferi)
    LoyaltyN,
    Lore,
    /// a Spellgorger Weird token made by Ral and the Implicit Maze (+1/+1 counter per noncreature spell)
    WeirdToken,
    Lure,
    /// Knight of the Holy Nimbus: an opponent paid {2}, it can't regenerate this turn
    Noregen,
    Manatok,
    MordDog,
    MustAttack,
    Oring,
    Prepared,
    Prevent,
    Prowess,
    Reflection,
    /// a Monster Role (Monstrous Rage) is on the creature: +1/+1 and trample
    Role,
    /// a hasty token copy made by Rionya, Fire Dancer (exiled at the end step)
    Rionya,
    Ruin,
    Serenity,
    /// Sephiroth, Fabled SOLDIER: [turn stamp, times its drain has resolved that turn]
    SephTimes,
    /// The Soul Stone has been harnessed (its upkeep reanimation is active)
    Harnessed,
    Stolen,
    /// stun counters (the untap step removes one instead of untapping)
    Stun,
    /// creatures weakened until their caster's next turn: [caster, caster's turn count, power lost] (Mouth of the Storm)
    Weaken,
    /// Elrond, Lord of Rivendell: how many times his ability has resolved, and on which turn
    Resolved,
    ResolvedAt,
    /// suspected (menace, can't block) until it leaves the battlefield
    Suspect,
    /// Mirage Mirror is a copy of another permanent until end of turn (its own card is in `phys`)
    MirageMirror,
    /// --- scion: Scion of the Ur-Dragon is a copy of a Dragon until end of turn (its own card is in `phys`)
    ScionCopy,
    /// Wall of Junk blocked this combat: it returns to its owner's hand at end of combat
    WallBlocked,
    TapOnReturn,
    Unblockable,
    /// Reconstructed Thopter came back by unearth (exiled at the end step)
    Unearth,
    Unbl,
    Used,
    Voice,
    Void,
    Voja,
    /// Susurian Voidborn was cast for its warp cost (exiled at the next end step)
    Warp,
    Wishes,
    Witch,
    /// Tier 0 Stella deck cards: Midnight Clock's hour counters (Int), Octavia's base 8/8 mark (Stamp)
    T0Stella,
    /// Calix, Destiny's Hand: this enchantment holds a -3 exile (DataKey::Oring) that returns when it leaves
    CalixLink,
    /// soulbond (Silverblade Paladin): the paired creature (a `Val::Perm`), set on both; valid only while both are on the battlefield under one controller and point at each other
    Soulbond,
    /// t45-prosper: Ob Nixilis, Captive Kingpin's last batch of life losses (a `Val::List`: work, stack pushes, the players)
    ObBatch,
    /// --- Tiers 4/5 (heliod-zur): Archivist of Oghma: library searches by its opponents still to answer
    T45Search,
    /// --- Tiers 4/5 (urza-yawgmoth): Nine-Lives Familiar's revival counters (Int)
    Revival,
    /// --- Tiers 4/5 (urdragon): Dragon's Hoard's gold counters (Int); a Dragon cast from the graveyard with Rivaz of
    /// the Claw (Bool: exiled when it dies); a commander Hellkite Courser put onto the battlefield (Bool: back to the
    /// command zone at the next end step); Rivaz's once-each-turn graveyard cast (Stamp)
    T45Dragon,
    /// --- Tiers 1/2 (isshin): the player Curse of Opulence enchants (`Val::Player`); Firebender Ascension's quest
    /// counters (`Val::Int`)
    T12Isshin,
    /// --- Tiers 1/2 (lightpaws): Pariah: the damage it has sent to the enchanted creature this turn ([Stamp, Int])
    T12Pariah,
    /// --- Tiers 1/2 (brago-meren): Reality Acid's time counters (Int); Twilight Diviner's once-each-turn trigger (Stamp)
    T12,
    /// --- Tiers 1/2 (brago-meren): the creature entered from a graveyard on this turn (Stamp; Twilight Diviner)
    T12FromGy,
}

/// A value in per-card state.
#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    None,
    Bool(bool),
    Int(i64),
    Card(CardId),
    Perm(PermId),
    Land(PermId),
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
/// What a player-level "can't cast" lock stops (Silence, Chant, Mandate, Bilbo's Gambit: every spell; Calamity's Wake,
/// Hope of Ghirapur: noncreature spells; Abeyance: instants and sorceries, and abilities that aren't mana abilities).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockKind {
    AllSpells,
    Noncreature,
    InstSorcAbilities,
    /// --- Tiers 1/2 (lightpaws): creature and planeswalker spells (Single Combat)
    CreatureWalker,
}

/// When a cast lock ends: with the turn it was set in, or when the given player's turn count passes the number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockEnd {
    Turn(TurnStamp),
    TurnOf(PlayerId, u32),
    /// --- Tiers 1/2 (lightpaws): until the end of the player's next turn, set when their turn count was the number
    /// (Single Combat: the rest of this turn, the others' turns, then that player's next one)
    ThroughNextTurn(PlayerId, u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastLock {
    pub kind: LockKind,
    pub end: LockEnd,
}

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

// ------------------------------------------------------------------ permanents and lands
/// What a copy is "except": the exceptions of a clone, a token copy or Preston's Illusions. Stored on the copy
/// (`Perm::exc`) so that the type queries (`values::subtypes`, `values::ptypes`, `colors_of`, `is_legendary`) see them.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CopyExc {
    /// card types it has in addition (Copy Artifact: enchantment; Molten Duplication: artifact)
    pub add_types: Types,
    /// a creature type it has in addition (Mockingbird: Bird; Phantasmal Image: Illusion)
    pub add_subtype: Option<Sym>,
    /// it has ONLY this creature type (Preston's Illusions)
    pub only_subtype: Option<Sym>,
    /// its power and toughness are these (Preston: 0/1)
    pub pt: Option<(i32, i32)>,
    /// its colours are these (Preston: white)
    pub colors: Option<Colors>,
    pub fly: bool,
    pub haste: bool,
    /// it is not legendary (Helm of the Host)
    pub not_legendary: bool,
}

/// An "enters as a copy of" replacement: the card that is copied, and the exceptions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CopyOf {
    pub cd: CardId,
    pub exc: CopyExc,
}

/// A permanent or a token, land or not (Python's `Perm`, every slot kept, and its `Land`). A land is put in by
/// `Game::add_land` and listed in `Player::lands`; every other permanent in `Player::perms`.
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
    /// the exceptions this permanent was copied with (added types, only one creature type, new colour, not legendary)
    pub exc: CopyExc,
}

impl Perm {
    /// the card this permanent is (a land always has one); panics on a token
    pub fn card(&self) -> CardId {
        self.cd.expect("a permanent with a card")
    }
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
    /// the second commander of a Partner pair (None for every single-commander seat); `cmd` is the first
    pub cmd2: Option<CardId>,
    pub deck_names: Arc<[CardId]>,
    /// top card last, as in Python
    pub library: Vec<CardId>,
    pub hand: Vec<CardId>,
    pub gy: Vec<CardId>,
    pub exile: crate::engine::exile::ExileZone,
    pub lands: Vec<PermId>,
    pub perms: Vec<PermId>,
    pub life: i32,
    pub alive: bool,
    pub turns: u32,
    pub cmd_in_zone: bool,
    pub tax: u32,
    /// commander damage taken, by the seat whose commander dealt it
    pub cmd_dmg: [i32; MAX_SEATS],
    /// the second commander: in the command zone, its own tax (+2 per cast from there), the damage it dealt
    pub cmd2_in_zone: bool,
    pub tax2: u32,
    pub cmd2_dmg: [i32; MAX_SEATS],
    pub treasures: u32,
    /// how many of `treasures` are tapped (Tataru Taru, "artifacts enter tapped"): they untap in the owner's untap step,
    /// make no mana until then (the ability needs {T}) but can still be sacrificed to a cost
    pub tapped_treasures: u32,
    pub clues: u32,
    /// Blood tokens (Restless Bloodseeker; impls/sheoldred.rs)
    pub bloods: u32,
    pub foods: u32,
    pub poison: u32,
    /// the mana pool: mana made and not spent yet (`engine::pool`; Python's `floatR`, `floatA` ...)
    pub mana: crate::engine::pool::Pool,
    /// practice mode: the mana a person has made and not spent (Python's `p.pool`; inert for AI seats)
    pub pool: crate::human::mana::ManaPool,
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
    /// --- Tiers 4/5 (winota): `life_locked` lasts until end of turn (Flare of Fortitude): cleared as a turn starts
    pub life_lock_eot: bool,
    /// the kind of the last life loss (for how the player was eliminated)
    pub last_kind: Sym,
    pub death_kind: Sym,
    /// a Tier 6 deck (pool or Godo): the report counters (`t6stats`) are kept for it
    pub t6: bool,
    /// this player's own turn count when it last dealt damage to an opponent
    pub hit_turn: i32,
    /// Lightning, Army of One: (its controller, that player's turn count when it connected); all damage to this player and their
    /// permanents is doubled until that controller's next turn begins (see `engine::life::stagger_mult`)
    pub stagger: Option<(PlayerId, u32)>,
    /// life lost / gained this turn (Bloodsoaked Insight, Archfiend, Y'shtola)
    pub lost_turn: Option<(TurnStamp, i32)>,
    pub gained_turn: Option<(TurnStamp, i32)>,
    /// the last turn a creature died under this player's control (Essenceknit Scholar)
    pub creature_died: Option<TurnStamp>,
    /// tokens this player created this turn: (the turn, how many) (Thalisse, Reverent Medium)
    pub tok_made: (Option<TurnStamp>, u32),
    // --- Tiers 4/5 (krenko)
    /// Goblins that entered under this player's control this turn, counted as they enter (Hobgoblin Bandit Lord):
    /// (the turn, how many)
    pub goblins_in: (Option<TurnStamp>, u32),
    /// speed (Start your engines!: 0 until a permanent with it is out, then 1 to 4) and the turn it last rose
    pub speed: u8,
    pub speed_rose: Option<TurnStamp>,
    /// emblems from planeswalker ultimates ('tamiyo', 'nissa' ...)
    pub emblems: Vec<Sym>,
    /// cards taken with Opposition Agent (castable with mana of any type)
    pub agent_ids: Vec<CardId>,
    /// the last turn a permanent of this player's left the battlefield (revolt)
    pub left_turn: Option<TurnStamp>,
    /// the last turn a card left this player's graveyard (Living History)
    pub gy_left: Option<TurnStamp>,
    /// noncombat damage was dealt to this player this turn / last turn (Command the Stage)
    pub nc_dmg_this: bool,
    pub nc_dmg_last: bool,
    /// once-per-turn flags (Python's `flag_turn`)
    pub flag_turn: IndexMap<Sym, TurnStamp>,
    /// the AI's memory of who hurt it (brain.note_damage)
    pub grudge: IndexMap<Sym, f64>,
    /// the dungeon p is in: (index into `engine::dungeon::DUNGEONS`, room index); None = not in one
    pub dungeon: Option<(usize, u8)>,
    /// dungeons p has completed this game
    pub dungeons_done: u32,
    /// cards cast this game
    pub cast_names: IndexSet<CardId>,
    /// the turn (own turn count) each card was first cast, logged at the end of each of its owner's turns;
    /// `cast_logged` is how many of `cast_names` are in it already (the analysis report's "average round cast")
    pub cast_turns: Vec<(CardId, u32)>,
    pub cast_logged: usize,
    /// cards this player has seen (drawn, tutored)
    pub seen: IndexSet<CardId>,
    /// --- ai-flow: the source of the effect resolving for this player (a spell, trigger or ability), which the flow
    /// stats credit its draws to (`t6stats::flow_source`; Tier 6 players only)
    pub flow_src: Option<CardId>,
    /// the turn draws are counted for (Narset, Parter of Veils)
    pub draw_st: Option<TurnStamp>,
    /// cards drawn this turn
    pub draw_n: u32,
    /// this turn's first draw (miracle)
    pub miracle: Option<(TurnStamp, CardId)>,
    /// Urabrask, Heretic Praetor: the next draw this turn is exiled instead
    pub urabrask: Option<TurnStamp>,
    /// the turn this player last scried or surveilled (Desperate Futurescribe)
    pub scry_turn: Option<TurnStamp>,
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
    /// the cards of this player's with mayhem discarded this turn (kefka.rs)
    pub discard_log: Option<(TurnStamp, Vec<CardId>)>,
    /// extra land drops this turn (Explore)
    pub extra_land_now: u32,
    /// "can't cast" locks on this player (Silence, Orim's Chant, Hope of Ghirapur, Abeyance ...); see `engine::cast`
    pub cast_locks: Vec<CastLock>,
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
    /// --- ai-fix: B: the cards p put on top of its library itself (Vampiric Tutor, Imperial Seal, Mystical Tutor,
    /// Enlightened Tutor), top last: p knows them until the library is shuffled or they are drawn (`tutors::known_top`)
    pub known_top: Vec<CardId>,
    /// --- ai-planner: the card the line planner chose for the tutor being cast (`ai::planner`); a tutor takes it
    /// when it meets the tutor's own restriction, else picks as usual. Set only around the planner's cast
    pub plan_tutor: Option<CardId>,
    /// the permanent number (born) when this player's last turn ended
    pub last_turn_end: u32,
    /// the graveyard at the start of this turn
    pub gy_start: Vec<CardId>,
    /// Glacial Chasm's age counters (cumulative upkeep)
    pub chasm_age: i32,
    /// t0-zurgo: the most creatures this player attacked with this turn, and the turn it last attacked with a commander
    pub atk_count: (Option<TurnStamp>, u32),
    pub cmd_atk: Option<TurnStamp>,
    /// t0-zurgo (Neriv): cards exiled by Neriv, playable on turns this player attacked with a commander (in exile, or in
    /// hand for the turn)
    pub neriv: Vec<CardId>,
    /// suspended cards (Profane Tutor)
    pub suspended: Vec<(CardId, u32)>,
    /// the Stella precon's one-shot effects (impls/t0_stella.rs)
    pub t0s: crate::impls::t0_stella::StellaState,
    /// cards playable until the end of a later turn (Prosper): (card, the turn count it lasts to)
    pub impulse_long: Vec<(CardId, u32)>,
    /// cards in exile with a silver counter (Karn, Scion of Urza; impls/karn_engines.rs)
    pub silver: Vec<CardId>,
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
    /// the turn a bomb (bomb 6+ or power 6+) first landed (ais.note_bomb: Sephiroth's plan turn)
    pub first_bomb: Option<u32>,
    /// compiled activated abilities used this round: (permanent, ability) -> (round stamp, uses)
    pub act_uses: IndexMap<(PermId, u16), (TurnStamp, u32)>,
    /// counters for reports (Python's `p.stats`)
    pub stats: IndexMap<Sym, i64>,
    /// rebound spells (Ephemerate) cast again at the next upkeep
    pub rebound: Vec<CardId>,
    /// Flashback (the card): the instants and sorceries in the graveyard that have flashback this turn (practice mode)
    /// Necropotence (practice mode): cards exiled face down with the turn count they come to hand at
    pub necro_wait: Vec<(CardId, u32)>,
    pub fb_grant: Option<(TurnStamp, Vec<CardId>)>,
    /// the turn count Jace's Archivist was last used on
    pub arch_t: Option<u32>,
    /// the Ring's level (how many times it has tempted this player, at most 4) and the chosen Ring-bearer
    pub ring_level: u32,
    pub ring_bearer: Option<PermId>,
    /// adventure cards whose adventure half was cast (Python's `adv_done`; common.adventure_options)
    pub adv_done: Vec<CardId>,
    /// combos.attempt: the combo last gone for, (turn count, combo name): once per turn each
    pub combo_turn: Option<(u32, Sym)>,
    /// combos: the turn count of the last Yawgmoth loop without a payoff (it doesn't loop twice in a turn)
    pub yawg_loop: Option<u32>,
    /// Sunfall's Incubator: its +1/+1 counters (0: none)
    pub incubator: i32,
    /// Reflector Mage: (the card name this player can't cast, until the Mage's controller has this turn count, that
    /// controller) (Python's `locked_name`)
    pub locked_name: Option<(Sym, u32, PlayerId)>,
    /// Urza's Sagas on the battlefield and their lore counters (Python's `p.sagas`)
    pub sagas: Vec<(PermId, u32)>,
    /// Hunter's Insight: (the turn, the creature): its combat damage to a player draws that many
    pub insight: Option<(TurnStamp, PermId)>,
    /// Conduit of Worlds: no more spells this turn
    pub conduit_lock: Option<TurnStamp>,
    /// Oath of Teferi: the cards it exiled, back at the end step
    pub oath_return: Vec<CardId>,
    /// Muldrotha, the Gravetide: the permanent types played from the graveyard this turn (Python's `muld_used`)
    pub muld_used: Option<(TurnStamp, Vec<Sym>)>,
    /// Sephiroth: the turn count a loop was last gone for (mine.run_loop)
    pub loop_turn: Option<u32>,
    /// Sephiroth: the turn count Tortured Existence was last used on
    pub te_used: Option<u32>,
    /// Sephiroth's Skullclamp: (turn count, uses that turn)
    pub clamp_t: Option<u32>,
    pub clamp_n: u32,
    /// Legion Loyalist's battalion this turn: tokens can't block this player's creatures (rules.evasion_blocked)
    pub loyalist_turn: Option<TurnStamp>,
    /// Embercleave: the turn this player last attacked, and with what (rules2: its cost is {1} less per attacker)
    pub attacking: Option<TurnStamp>,
    pub attackers: Vec<PermId>,
    /// Valakut Exploration: the cards it exiled this turn (held in hand until the end step)
    pub valakut_cards: Vec<CardId>,
    /// once-a-turn pumps: Battle Cry Goblin, Purphoros
    pub bcg_turn: Option<TurnStamp>,
    pub purph_turn: Option<TurnStamp>,
    /// Glimpse of Nature: the turn it was cast (Python keeps the turn in `p.glimpse`; `glimpse` stays the flag)
    pub glimpse_turn: Option<TurnStamp>,
    /// Bloodchief's Thirst: the last one this player cast was kicked (rules.thirst_cast records it for the resolve)
    pub thirst_kicked: bool,
    /// Meren of Clan Nel Toth's experience counters (Python's `p.experience`)
    pub experience: i32,
    /// energy counters (Static Prison, Aether Hub)
    pub energy: i32,
    /// Malcator: artifact tokens made this turn, (turn, how many) (Python's `art_tok`)
    pub art_tok: Option<(TurnStamp, i32)>,
    /// Reconstructed Thopter cards already unearthed (once each)
    pub unearthed: Vec<CardId>,
    /// the wipe (by `Game::cur_batch`) Marchesa, the Black Rose died in: creatures dying in the same wipe return too
    /// (Python's `p.marchesa_batch`)
    pub marchesa_batch: Option<u32>,
}

impl Player {
    /// Treasures that can make mana now
    pub fn live_treasures(&self) -> u32 {
        self.treasures.saturating_sub(self.tapped_treasures)
    }

    /// sacrifice one Treasure as a cost (a tapped one first: it can't make mana anyway)
    pub fn sac_treasure(&mut self) {
        self.tapped_treasures = self.tapped_treasures.saturating_sub(1);
        self.treasures -= 1;
    }

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
            cmd2: None,
            deck_names: cards,
            library,
            hand: vec![],
            gy: vec![],
            exile: crate::engine::exile::ExileZone::default(),
            lands: vec![],
            perms: vec![],
            life: 40,
            alive: true,
            turns: 0,
            cmd_in_zone: true,
            tax: 0,
            cmd_dmg: [0; MAX_SEATS],
            cmd2_in_zone: false,
            tax2: 0,
            cmd2_dmg: [0; MAX_SEATS],
            treasures: 0,
            tapped_treasures: 0,
            clues: 0,
            bloods: 0,
            foods: 0,
            poison: 0,
            mana: Default::default(),
            pool: Default::default(),
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
            life_lock_eot: false,
            last_kind: "other",
            death_kind: "",
            t6: crate::t6stats::is_t6_key(key),
            hit_turn: -1,
            stagger: None,
            lost_turn: None,
            gained_turn: None,
            creature_died: None,
            tok_made: (None, 0),
            goblins_in: (None, 0),
            speed: 0,
            speed_rose: None,
            emblems: vec![],
            agent_ids: vec![],
            left_turn: None,
            gy_left: None,
            nc_dmg_this: false,
            nc_dmg_last: false,
            flag_turn: IndexMap::new(),
            grudge: IndexMap::new(),
            dungeon: None,
            dungeons_done: 0,
            cast_names: IndexSet::new(),
            cast_turns: vec![],
            cast_logged: 0,
            seen: IndexSet::new(),
            flow_src: None, // --- ai-flow
            draw_st: None,
            draw_n: 0,
            miracle: None,
            urabrask: None,
            scry_turn: None,
            impulse: vec![],
            milled_keys: vec![],
            ozolith_counters: 0,
            removed_bombs: vec![],
            regen_turn: None,
            borrowed: vec![],
            lost_names: IndexMap::new(),
            discarded_turn: None,
            discard_log: None,
            extra_land_now: 0,
            cast_locks: vec![],
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
            known_top: vec![], // --- ai-fix: B
            plan_tutor: None, // --- ai-planner
            last_turn_end: 0,
            gy_start: vec![],
            chasm_age: 0,
            suspended: vec![],
            t0s: Default::default(),
            atk_count: (None, 0),
            cmd_atk: None,
            neriv: vec![],
            impulse_long: vec![],
            silver: vec![],
            erebos_t: None,
            nomax_turn: None,
            skip_turns: 0,
            unbl_all: None,
            elspeth_emblem: false,
            milestone: IndexMap::new(),
            first_bomb: None,
            act_uses: IndexMap::new(),
            rebound: vec![],
            fb_grant: None,
            necro_wait: vec![],
            arch_t: None,
            ring_level: 0,
            ring_bearer: None,
            adv_done: vec![],
            combo_turn: None,
            yawg_loop: None,
            incubator: 0,
            locked_name: None,
            sagas: vec![],
            insight: None,
            conduit_lock: None,
            oath_return: vec![],
            muld_used: None,
            loop_turn: None,
            te_used: None,
            clamp_t: None,
            clamp_n: 0,
            loyalist_turn: None,
            attacking: None,
            attackers: vec![],
            valakut_cards: vec![],
            bcg_turn: None,
            purph_turn: None,
            glimpse_turn: None,
            thirst_kicked: false,
            experience: 0,
            energy: 0,
            art_tok: None,
            unearthed: vec![],
            marchesa_batch: None,
            stats: IndexMap::new(),
        }
    }

    /// seat a second commander (Partner): it starts in the command zone, with its own tax and its own damage
    pub fn set_partner(&mut self, c2: Option<CardId>) {
        self.cmd2 = c2;
        self.cmd2_in_zone = c2.is_some();
        if let Some(c) = c2 {
            self.library.retain(|&x| x != c);
        }
    }

    /// is `c` one of this seat's commanders?
    pub fn is_commander(&self, c: CardId) -> bool {
        c == self.cmd || self.cmd2 == Some(c)
    }

    /// this seat's commanders: one, or two with Partner (the first is `cmd`)
    pub fn commanders(&self) -> impl Iterator<Item = CardId> {
        std::iter::once(self.cmd).chain(self.cmd2)
    }

    /// is this commander card in the command zone?
    pub fn cmd_zone_of(&self, c: CardId) -> bool {
        if c == self.cmd { self.cmd_in_zone } else { self.cmd2 == Some(c) && self.cmd2_in_zone }
    }

    /// is any commander in the command zone?
    pub fn any_cmd_in_zone(&self) -> bool {
        self.cmd_in_zone || (self.cmd2.is_some() && self.cmd2_in_zone)
    }

    /// the commanders now in the command zone
    pub fn cmds_in_zone(&self) -> impl Iterator<Item = CardId> {
        self.commanders().filter(|&c| self.cmd_zone_of(c))
    }

    pub fn set_cmd_zone(&mut self, c: CardId, v: bool) {
        if c == self.cmd {
            self.cmd_in_zone = v;
        } else if self.cmd2 == Some(c) {
            self.cmd2_in_zone = v;
        }
    }

    /// the commander tax of this commander card (+2 for each earlier cast from the command zone)
    pub fn tax_of(&self, c: CardId) -> u32 {
        if c == self.cmd { self.tax } else if self.cmd2 == Some(c) { self.tax2 } else { 0 }
    }

    pub fn add_tax(&mut self, c: CardId) {
        if c == self.cmd {
            self.tax += 2;
        } else if self.cmd2 == Some(c) {
            self.tax2 += 2;
        }
    }

    /// the most commander damage any one commander of seat `from` has dealt this seat
    pub fn cmd_dmg_from(&self, from: usize) -> i32 {
        self.cmd_dmg[from].max(self.cmd2_dmg[from])
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
    /// storm: the spells cast before it this turn, as counted when it was cast (the Breach line)
    pub storm: Option<i32>,
    /// Brain Freeze in the Breach line: who each copy mills
    pub mill: Vec<PlayerId>,
    /// a modal spell's chosen mode (Golgari Charm, Mardu Charm: Python's `ctx['mode']`)
    pub mode: Option<Sym>,
    /// the creature a spell is cast on without targeting it as removal (Hunter's Insight: `ctx['host']`)
    pub host: Option<PermId>,
    /// Mizzix's Mastery: the graveyard card a person chose as it was cast (Python's `ctx['mastery_pick']`)
    pub mastery_pick: Option<CardId>,
    /// the spell was cast from its owner's graveyard (flashback): set as it resolves (Retrieve the Esper)
    pub from_gy: bool,
    /// Multikicker: how many times the spell was kicked (Comet Storm)
    pub kicks: u32,
    /// the mana actually spent to cast it, after reductions and alternative costs (0 for a free cast); set by `cast_card`
    pub mana_spent: u32,
    /// some of that mana was coloured (Void Mirror, Vibrance)
    pub mana_colored: bool,
    /// a person's extra choices at cast time (human/specs.rs): more permanent targets (Comet Storm's kicks), more
    /// player targets, and graveyard cards (Victimize, Helping Hand, Rewrite Regrets). Empty for the AI.
    pub perms: Vec<PermId>,
    pub faces: Vec<PlayerId>,
    pub cards: Vec<CardId>,
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
    /// the one permanent arena: every permanent and token, land or not, by its `PermId`
    pub perms: Vec<Perm>,
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
    /// permanents the AI keeps out of its mana payment for now (a combo piece that must stay untapped: Ioreth in
    /// the Sisay DDR line); set around a step and restored after it
    pub keep_untapped: Vec<PermId>,
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
    /// the creature an Aura entering now goes on (Light-Paws fetching an Aura onto itself): Python's `g.attach_to`
    pub attach_to: Option<PermId>,
    /// what a sacrificed permanent was, for spells that count it: (mana value, toughness)
    pub sac_snapshot: Option<(u32, i32)>,
    /// Silence-style lock: (the turn, the player it doesn't stop)
    pub silence: Option<(TurnStamp, PlayerId)>,
    /// a player cast-lock was ever set this game (the gate that turns the lock checks on when no card is hooked)
    pub locks_used: bool,
    /// undying / persist returns this turn, by card (RETURN_CAP)
    pub returns_turn: Option<TurnStamp>,
    pub returns: IndexMap<CardId, u32>,
    /// the permanent dying now (Auras that care how their creature left)
    pub dying: Option<PermId>,
    /// the permanent being exiled now (Kaya's Ghostform)
    pub exiling: Option<PermId>,
    /// the last turn a creature died (Barad-dûr)
    pub died_turn: Option<TurnStamp>,
    /// creatures that died this turn: (the turn, how many) (Lagomos, Hand of Hatred)
    pub died_count: (Option<TurnStamp>, u32),
    /// Treasure mana paid since the last payment began; the spell cast with it takes it (Kalain, Reclusive Painter)
    pub treasure_paid: u32,
    /// the last spell cast and the Treasure mana paid for it (Kalain, Reclusive Painter)
    pub cast_treasure: Option<(CardId, u32)>,
    /// mana spent and whether any was coloured, since the last payment began; the spell cast with it takes it (`Ctx::mana_spent`)
    pub mana_spent: u32,
    pub mana_colored: bool,
    /// an alternative cost is being paid: its several payments add up instead of restarting the record
    pub spent_keep: bool,
    /// the mana spent to cast the permanent spell that is entering (Chalice of the Void: X is half of it)
    pub entering_spent: u32,
    /// permanents entered, numbered in order (born)
    pub enter_no: u32,
    /// permanents that entered this turn
    pub entered: Vec<(TurnStamp, PermId)>,
    /// creatures in combat now (Settle the Wreckage, Prophesied End)
    pub in_combat: Vec<PermId>,
    /// an Aura put onto the battlefield without being cast (Zur): placed by its card code
    pub aura_put: bool,
    /// Uro, Titan of Nature's Wrath is entering by escape: it isn't sacrificed (Python's `g.uro_escaping`)
    pub uro_escaping: bool,
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
    /// the Breach line's dry run worked only without its setup casts (Laboratory Maniac, Birgi, Jeska's Will)
    pub line_no_setup: bool,
    /// the Breach line's dry run for a position: (its turn, the position's numbers, does it finish the table)
    pub breach_dry: Option<(TurnStamp, Vec<u32>, bool)>,
    /// how deep the ability language's triggers are nested (MAX_DEPTH stops runaway chains)
    pub dsl_depth: u32,
    /// creatures with a power/toughness rule of their own (card code; Python's `g.selfpt`)
    pub selfpt: bool,
    /// blocking creatures in the combat being resolved
    pub blocking: Vec<PermId>,
    /// a fog this turn (Spore Frog): no combat damage
    pub fog: Option<TurnStamp>,
    /// "end the combat phase" (Mandate of Peace) was cast: the current combat skips its remaining steps (cleared by `combat`)
    pub end_combat: bool,
    /// Arachnogenesis: combat damage from non-Spider creatures is prevented this turn
    pub fog_spiders: Option<TurnStamp>,
    /// --- Tiers 4/5 (winota): creatures without flying can't block this turn (Sundering Eruption)
    pub ground_noblock: Option<TurnStamp>,
    /// day and night (Infestation Expert): 0 neither, 1 day, 2 night
    pub day_night: u8,
    /// the player whose turn began last (day and night look at the turn before)
    pub dn_last: Option<PlayerId>,
    /// Impractical Joke: damage can't be prevented this turn
    pub no_prevent: Option<TurnStamp>,
    /// attacking creatures card code made during attack triggers (Python returns them from CI.fire)
    pub new_attackers: Vec<PermId>,
    /// t0-zurgo: every attacking creature and the player it attacks (set when attackers are declared; new attackers add theirs)
    pub atk_targets: Vec<(PermId, PlayerId)>,
    /// practice mode: (attacker, planeswalker) pairs the person declared; the AI computes its own (`walker_attacks`)
    pub human_walker_atk: Vec<(PermId, PermId)>,
    /// t0-zurgo: `enter_no` when the current turn began (tokens with a larger `born` entered this turn)
    pub turn_enter: u32,
    /// the game ran out of engine steps (a runaway loop) and was stopped
    pub stopped: bool,
    /// a game with one player and no opponents to beat (Python's `goldfish`)
    pub goldfish: bool,
    /// Garland, Royal Kidnapper's steals are in play (Python's `g.garland`)
    pub garland: bool,
    /// Garland's steals: (who took it, the creature, the player it came from, the round it was taken)
    pub garland_list: Vec<(PlayerId, PermId, PlayerId, u32)>,
    /// Marchesa's returns at the next end step: (the player it returns for, the card, its owner)
    pub marchesa_due: Vec<(PlayerId, CardId, PlayerId)>,
    /// extra turns waiting (the last queued plays first): (who takes it, whether that player loses at its end step)
    pub extra_turns: Vec<(PlayerId, bool)>,
    /// the player whose current turn is one that ends in a loss (Final Fortune, Last Chance)
    pub doomed_turn: Option<PlayerId>,
    /// Memory Jar: the hands set aside, returned at the end step
    pub jar_due: Vec<(PlayerId, Vec<CardId>)>,
    /// --- Tiers 4/5 (urza-yawgmoth): Nine-Lives Familiar returns at the next end step: (controller, card, revival
    /// counters it returns with)
    pub t45_due: Vec<(PlayerId, CardId, i64)>,
    /// --- Tiers 1/2 (lathril): behold costs paid: (the spell, the card it exiled), so the card returns when the
    /// permanent leaves (Champions of the Perfect)
    pub beheld: Vec<(CardId, CardId)>,
    /// --- Tiers 1/2 (kaalia-sythis): Avacyn, Angel of Horror and Liesa's returns at the next end step: (controller,
    /// card, owner, to its owner's hand rather than the battlefield)
    pub t12_due: Vec<(PlayerId, CardId, PlayerId, bool)>,
    /// Avacyn / Liesa died in a wipe: (controller, the wipe's batch, Liesa) for the creatures dying with them
    pub t12_batch: Vec<(PlayerId, u32, bool)>,
    /// Valgavoth, Terror Eater: the one these lists belong to; the cards exiled with it (card, owner); the cards in
    /// opponents' graveyards it has seen; the cards cast through it that go back to their owner's graveyard
    pub t12_valg: Option<PermId>,
    pub t12_void: Vec<(CardId, PlayerId)>,
    pub t12_kept: Vec<(CardId, PlayerId)>,
    pub t12_pass: Vec<(CardId, PlayerId)>,
    /// a Serra's Emissary has entered this game (combat and blocks look for it from then on)
    pub t12_emissary: bool,
    /// --- Tiers 1/2 (brago-meren): creature cards put into a graveyard (died, milled, discarded, surveilled, searched
    /// for) and not seen to leave it since; `zones::enter` reads it (Twilight Diviner: "entered from a graveyard")
    pub t12_gy: Vec<CardId>,
    /// the wipe resolving now (its `batch` number), if any: creatures dying in it die together (Python's `g.batch`
    /// while not None)
    pub cur_batch: Option<u32>,
    /// permanents with hand-written card code, in entry order (Python's `g.hooks`)
    pub hooks: Vec<PermId>,
    pub stack: Vec<StackItem>,
    /// practice mode: >0 while the person is responding with something on the stack (Python's `g.responding`)
    pub responding: u32,
    /// items ever put on the stack: tells whether a player with priority did something; also the next item's id
    pub stack_pushes: u32,
    /// 'probe' while finding out which pending triggers really trigger (Python's `trig_mode`)
    pub trig_probe: bool,
    /// the stack item of the trigger being resolved (by id)
    pub trig_current: Option<u32>,
    /// ids of trigger items that have a copy which resolved (Peter Parker's Camera): the hook runs its effect again
    /// for each, before the original (`stack::take_copies`)
    pub trig_copies: Vec<u32>,
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
    /// --- ai-fix: E: in a look-ahead copy, the deciding player's own-turn count when the playout began (a win scores
    /// by how many of its own turns later it came)
    pub search_from: u32,
    /// play-by-play lines when tracing a game (`--trace`)
    pub log: Option<Vec<String>>,
    /// practice mode's people; a copy of the game never has any (see `control::Humans`)
    pub humans: Humans,
    /// practice mode: the session's log sink and abort flag (a copy has none)
    pub link: Link,
    /// practice mode: the real game's look-ahead decisions, recorded and replayed (a copy has none)
    pub search_tape: crate::ai::tape::TapeSlot,
    /// the X paid for the last X spell (card code that reads it: Walking Ballista ...)
    pub last_x: i32,
    /// creatures dealt damage this turn: (the creature, the turn) (Rooftop Assassin); cleared of old turns on write
    pub dmg_marks: Vec<(PermId, TurnStamp)>,
    /// the order each person chose for their simultaneous triggers last time (practice mode; Python's `trig_orders`)
    pub trig_orders: Vec<(Sym, Vec<String>)>,
    /// a look-ahead copy playing out one attack plan: (defender, 'filtered' | 'all' | 'none')
    pub forced_attack: Option<(PlayerId, Sym)>,
    /// look-ahead decisions taken in this game, and engine steps its playouts used (the per-game caps)
    pub search_n: u32,
    pub search_work: u64,
    /// creature lands animated until the start of the next turn: (controller, the creature, the land)
    /// (lands.animate, revert_animated)
    pub animated: Vec<(PlayerId, PermId, PermId)>,
    /// a combo's last pieces are being cast (combos.attempt): Thassa's Oracle waits for the exile spell
    pub combo_spell: bool,
    /// Tier 6: the player now going for a combo (set around `combos::attempt`): opponents answer its steps as wins
    pub t6_going: Option<PlayerId>,
    /// --- ai-planner: a line planner's copy of the game, played for this player: nobody else takes priority (the
    /// planner reads the table through `go_ok`, as the real game does, and never through the opponents' hidden hands)
    pub plan_for: Option<PlayerId>,
    /// Karn, the Great Creator + Mycosynth Lattice: the player whose lock is on
    pub lattice_lock: Option<PlayerId>,
    /// Coat of Arms has entered (partials.coat_bonus applies)
    pub coat: bool,
    /// Bloodline Keeper has entered (partials.lineage_bonus applies)
    pub lineage: bool,
    /// partials.coat_bonus's cache: (bf_ver, each creature's bonus), as Python's `g.coat_cache`
    pub coat_cache: std::cell::RefCell<Option<(u64, Vec<(PermId, i32)>)>>,
    /// Return the Favor is being cast (it doesn't copy itself): Python's `g.in_rtf`
    pub in_rtf: bool,
    /// how deep Chatterfang's Squirrels for artifact tokens are nested (t3: a token loop stops at 10)
    pub chatter_depth: u32,
    /// t2.blink: how deep blinks are nested (blink chains are combos, not loops: Python's `g.blink_depth`)
    pub blink_depth: u32,
    /// how deep Consecrated Sphinx draws are nested (two Sphinxes feed each other: t4's `g.sphinx_depth`)
    pub sphinx_depth: u32,
    /// how deep Sanguine Bond / Exquisite Blood are nested (t4's `g.bond_depth`)
    pub bond_depth: u32,
    /// creatures that died this turn while Mahadi, Emporium Master was out (t4's `g.deaths_turn`, current turn only)
    pub deaths_turn: Option<(TurnStamp, i32)>,
    /// The Eternal Wanderer's exiles, back at their owner's next end step: (owner, card) (zur's `g.zur_due`)
    pub zur_due: Vec<(PlayerId, CardId)>,
    /// the creature Restoration Angel's enter trigger blinks (Python's `g.resto_target`)
    pub resto_target: Option<PermId>,
    /// Eerie Interlude: (player, card) exiled until the beginning of the next end step (Python's `g.eot_returns`)
    pub eot_returns: Vec<(PlayerId, CardId)>,
    /// the Captain America precon's bookkeeping (impls/t0_captain.rs)
    pub t0c: crate::impls::t0_captain::T0c,
    /// --- Tiers 1/2 (lightpaws): Flickerform's exiles waiting for the next end step (impls/t12_lightpaws.rs)
    pub t12lp: crate::impls::t12_lightpaws::T12,
    /// --- ai-audit: the goldfish loss accounting of the deck under test (gf_audit.rs); None outside goldfish games
    /// and in look-ahead copies
    pub audit: Option<Box<crate::gf_audit::Audit>>,
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
            keep_untapped: vec![],
            tap_cols: Colors::NONE,
            last_cast_etb: false,
            skip_etb: false,
            rem_src: None,
            noregen: false,
            destroyer: None,
            thief_chain: vec![],
            cur_cast: None,
            cast_target: None,
            attach_to: None,
            sac_snapshot: None,
            silence: None,
            locks_used: false,
            returns_turn: None,
            returns: IndexMap::new(),
            dying: None,
            exiling: None,
            died_turn: None,
            died_count: (None, 0),
            treasure_paid: 0,
            cast_treasure: None,
            mana_spent: 0,
            mana_colored: false,
            spent_keep: false,
            entering_spent: 0,
            enter_no: 0,
            entered: vec![],
            in_combat: vec![],
            aura_put: false,
            uro_escaping: false,
            last_removed: None,
            batch: 0,
            marchesa_on: false,
            no_fang: false,
            dsl_on: false,
            jeska_mana: false,
            line_no_setup: false,
            breach_dry: None,
            dsl_depth: 0,
            selfpt: false,
            blocking: vec![],
            fog: None,
            end_combat: false,
            fog_spiders: None,
            ground_noblock: None,
            day_night: 0,
            dn_last: None,
            no_prevent: None,
            new_attackers: vec![],
            atk_targets: vec![],
            human_walker_atk: vec![],
            turn_enter: 0,
            stopped: false,
            goldfish: false,
            garland: false,
            garland_list: vec![],
            marchesa_due: vec![],
            extra_turns: vec![],
            doomed_turn: None,
            jar_due: vec![],
            t45_due: vec![],
            beheld: vec![],
            t12_due: vec![],
            t12_batch: vec![],
            t12_valg: None,
            t12_void: vec![],
            t12_kept: vec![],
            t12_pass: vec![],
            t12_emissary: false,
            t12_gy: vec![],
            cur_batch: None,
            hooks: vec![],
            stack: vec![],
            responding: 0,
            stack_pushes: 0,
            trig_probe: false,
            trig_current: None,
            trig_copies: vec![],
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
            search_from: 0, // --- ai-fix: E
            log: None,
            humans: Humans::none(),
            link: Link::default(),
            search_tape: Default::default(),
            last_x: 0,
            dmg_marks: vec![],
            trig_orders: vec![],
            forced_attack: None,
            search_n: 0,
            search_work: 0,
            animated: vec![],
            combo_spell: false,
            t6_going: None,
            plan_for: None, // --- ai-planner
            lattice_lock: None,
            coat: false,
            lineage: false,
            coat_cache: std::cell::RefCell::new(None),
            in_rtf: false,
            chatter_depth: 0,
            blink_depth: 0,
            sphinx_depth: 0,
            bond_depth: 0,
            deaths_turn: None,
            zur_due: vec![],
            resto_target: None,
            eot_returns: vec![],
            t0c: Default::default(),
            t12lp: Default::default(),
            audit: None, // --- ai-audit
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

    /// Python's `g.opps(p)`: the other living players, in seat order
    pub fn opps(&self, p: PlayerId) -> impl Iterator<Item = PlayerId> + '_ {
        self.players.iter().filter(move |q| q.alive && q.id != p).map(|q| q.id)
    }

    /// Python's `g.after(p)`: every other seat, in turn order starting after p
    pub fn after(&self, p: PlayerId) -> impl Iterator<Item = PlayerId> + '_ {
        let n = self.players.len();
        (1..n).map(move |k| PlayerId(((p.index() + k) % n) as u8))
    }

    /// damage can't be prevented this turn (Impractical Joke)
    pub fn unpreventable(&self) -> bool {
        self.no_prevent == Some(self.turn_stamp())
    }

    /// note that creature `m` was dealt damage this turn
    pub fn mark_damaged(&mut self, m: PermId) {
        let ts = self.turn_stamp();
        self.dmg_marks.retain(|x| x.1 == ts);
        if !self.dmg_marks.iter().any(|x| x.0 == m) {
            self.dmg_marks.push((m, ts));
        }
    }

    /// was creature `m` dealt damage this turn?
    pub fn was_damaged_now(&self, m: PermId) -> bool {
        let ts = self.turn_stamp();
        self.dmg_marks.iter().any(|x| x.0 == m && x.1 == ts)
    }

    pub fn turn_stamp(&self) -> TurnStamp {
        TurnStamp { round: self.round, active: self.active }
    }

    /// Do the cast checks (`engine::cast::castable`) have anything to look at: a hooked card, or a cast lock / split
    /// second ever used this game. Callers that skip `castable` when nothing could forbid a cast gate on this.
    pub fn cast_checks(&self) -> bool {
        !self.hooks.is_empty() || self.locks_used
    }

    /// One engine step (Python's `tick`): a game, or a look-ahead copy, that runs past its budget stops.
    pub fn tick(&mut self) -> crate::flow::Res {
        self.work += 1;
        if self.link.abort.is_some() && self.link.aborted() {
            return Err(crate::flow::Stop::Cancelled);
        }
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
            exc: CopyExc::default(),
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
    pub fn add_land_entering(&mut self, p: PlayerId, cd: CardId, tapped: bool) -> PermId {
        self.add_land(p, cd, tapped)
    }

    pub fn add_land(&mut self, p: PlayerId, cd: CardId, tapped: bool) -> PermId {
        // --- Tiers 1/2 (windgrace): Spelunking, "lands you control enter untapped"
        let tapped = tapped && !crate::impls::t12_windgrace::lands_enter_untapped(self, p);
        // a land is a permanent in the one arena, on the battlefield at once; it is listed in `Player::lands`
        let id = self.new_perm(p, Some(cd), crate::sym::intern(""), 0, 0);
        let x = self.perm_mut(id);
        x.tapped = tapped;
        x.sick = false;
        x.on_bf = true;
        if !self.dsl_on && crate::dsl::land_live(self.db.get(cd)) {
            self.dsl_on = true; // its compiled abilities run (Phase 3), as zones::enter does for other permanents
        }
        self.player_mut(p).lands.push(id);
        id
    }

    /// Is m a creature now (Python's `Perm.creature`): layer 4 of the characteristics pass, `chars::is_creature`.
    #[inline]
    pub fn is_creature(&self, m: PermId) -> bool {
        crate::engine::chars::is_creature(self, m)
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
