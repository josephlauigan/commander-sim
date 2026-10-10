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
use indexmap::IndexMap;
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
    /// counters for reports (Python's `p.stats`)
    pub stats: IndexMap<Sym, i64>,
}

impl Player {
    pub fn new(id: PlayerId, key: Sym, ident: Colors, cmd: CardId, cards: Arc<[CardId]>) -> Player {
        let mut library: Vec<CardId> = cards.iter().copied().collect();
        if let Some(i) = library.iter().position(|&c| c == cmd) {
            library.remove(i);
        }
        Player {
            id,
            key,
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

/// A spell or ability on the stack (Python's `StackItem`).
#[derive(Debug, Clone, PartialEq)]
pub struct StackItem {
    pub controller: PlayerId,
    pub card: Option<CardId>,
    pub kind: StackKind,
    pub name: Sym,
    /// where a spell was cast from ('hand', 'cmd', 'gy' ...)
    pub zone: Sym,
    /// how much it matters to everyone, and per player
    pub imp: f64,
    pub aff: Vec<(PlayerId, f64)>,
    /// resolved by the engine's own casting code (false: by its caster's code)
    pub generic: bool,
    /// seats that chose not to counter it
    pub passed: Vec<PlayerId>,
    pub countered: bool,
    pub countered_by: Option<CardId>,
    /// the counterspell's target, when this item is a counterspell
    pub counters: Option<usize>,
}

// ------------------------------------------------------------------ the game
#[derive(Debug, Clone)]
pub struct Game {
    pub db: Arc<CardDb>,
    pub settings: Arc<Settings>,
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
    pub elim: Vec<PlayerId>,
    pub monarch: Option<PlayerId>,
    /// permanents with hand-written card code, in entry order (Python's `g.hooks`)
    pub hooks: Vec<PermId>,
    pub stack: Vec<StackItem>,
    pub stack_pushes: u32,
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
}

impl Game {
    pub fn new(db: Arc<CardDb>, settings: Arc<Settings>, players: Vec<Player>, rng: Rng) -> Game {
        let work_cap = settings.game_work;
        Game {
            db,
            settings,
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
            hooks: vec![],
            stack: vec![],
            stack_pushes: 0,
            trig_queue: vec![],
            resolving: 0,
            work: 0,
            work_cap,
            board_cap: None,
            in_search: false,
            log: None,
            humans: Humans::none(),
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

    /// Is m a creature now (Python's `Perm.creature`): a token, a creature card, or animated.
    pub fn is_creature(&self, m: PermId) -> bool {
        let m = self.perm(m);
        m.token || m.cd.is_some_and(|c| self.db.get(c).creature) || m.data.truthy(DataKey::Anim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::tests::db;
    use crate::settings::{AiMode, Profile};
    use crate::sym::intern;

    pub fn four_seat_game() -> Game {
        let db = db();
        let decks = crate::export::load_decks(&crate::cards::tests::data("decks.json")).unwrap();
        let mut seats = vec![decks.mine.iter().find(|d| d.key == "sauron").unwrap().clone()];
        seats.extend(decks.pool.iter().filter(|d| d.tier.as_deref() == Some("t1")).take(3).cloned());
        let players = seats
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let ids: Arc<[CardId]> = d.cards.iter().map(|n| db.id(n).unwrap()).collect();
                let cmd = db.id(&d.commander).unwrap();
                Player::new(PlayerId(i as u8), intern(&d.key), db.get(cmd).identity.unwrap_or_default(), cmd, ids)
            })
            .collect();
        let settings = Arc::new(Settings::new(Profile::Loose, AiMode::Lookahead, 1.0));
        Game::new(db, settings, players, Rng::named("play:500000"))
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
