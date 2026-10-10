//! Card definitions (Python's `engine.CD`): immutable, one per card name, shared by every copy in every game.
//! Built from the Python export (`export::RawCard`) once per process.

use crate::export::{RawCard, TagValue};
use crate::ids::CardId;
use crate::tag::{TAG_COUNT, Tag, TagKind};
use std::collections::HashMap;

// ------------------------------------------------------------------ types and colours
/// Card types, from Python's one-letter codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Types(u8);

impl Types {
    pub const LAND: Types = Types(1);
    pub const CREATURE: Types = Types(2);
    pub const INSTANT: Types = Types(4);
    pub const SORCERY: Types = Types(8);
    pub const ARTIFACT: Types = Types(16);
    pub const ENCHANTMENT: Types = Types(32);
    pub const PLANESWALKER: Types = Types(64);

    pub fn parse(codes: &str) -> Result<Types, String> {
        let mut t = 0u8;
        for ch in codes.chars() {
            t |= match ch {
                'L' => 1,
                'C' => 2,
                'I' => 4,
                'S' => 8,
                'A' => 16,
                'E' => 32,
                'P' => 64,
                _ => return Err(format!("unknown type code {ch:?} in {codes:?}")),
            };
        }
        Ok(Types(t))
    }

    pub fn has(self, t: Types) -> bool {
        self.0 & t.0 != 0
    }
}

/// A set of colours (W U B R G).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Colors(u8);

impl Colors {
    pub const NONE: Colors = Colors(0);

    fn bit(ch: char) -> Option<u8> {
        Some(match ch {
            'W' => 1,
            'U' => 2,
            'B' => 4,
            'R' => 8,
            'G' => 16,
            _ => return None,
        })
    }

    /// The colours named in a string such as a cost's pips ("2UUB" gives U and B); other characters are ignored.
    pub fn from_letters(s: &str) -> Colors {
        Colors(s.chars().filter_map(Colors::bit).fold(0, |a, b| a | b))
    }

    pub fn has(self, ch: char) -> bool {
        Colors::bit(ch).is_some_and(|b| self.0 & b != 0)
    }

    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn union(self, o: Colors) -> Colors {
        Colors(self.0 | o.0)
    }

    pub fn intersects(self, o: Colors) -> bool {
        self.0 & o.0 != 0
    }
}

// ------------------------------------------------------------------ tags
#[derive(Debug, Clone, PartialEq)]
pub enum TagVal {
    Flag,
    Int(i32),
    Str(Box<str>),
}

/// A card's tags. Which tags it has is a bit set (`has` is one AND); the few with values keep them in a short list.
/// `order` keeps Python's dict order for the rare code that walks the tags.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tags {
    bits: [u64; TAG_COUNT.div_ceil(64)],
    ints: Box<[(Tag, i32)]>,
    strs: Box<[(Tag, Box<str>)]>,
    order: Box<[Tag]>,
}

impl Tags {
    pub fn has(&self, t: Tag) -> bool {
        let i = t as usize;
        self.bits[i / 64] >> (i % 64) & 1 == 1
    }

    pub fn int(&self, t: Tag) -> Option<i32> {
        self.ints.iter().find(|x| x.0 == t).map(|x| x.1)
    }

    /// A number tag's value, or 0 when the card doesn't have it (Python's `int(tags.get(k, 0))`).
    pub fn int_or_0(&self, t: Tag) -> i32 {
        self.int(t).unwrap_or(0)
    }

    pub fn str(&self, t: Tag) -> Option<&str> {
        self.strs.iter().find(|x| x.0 == t).map(|x| &*x.1)
    }

    pub fn get(&self, t: Tag) -> Option<TagVal> {
        if !self.has(t) {
            return None;
        }
        Some(match t.kind() {
            TagKind::Flag => TagVal::Flag,
            TagKind::Int => TagVal::Int(self.int(t).unwrap_or(0)),
            TagKind::Str => TagVal::Str(self.str(t).unwrap_or("").into()),
        })
    }

    /// Tags in the order the Python dict has them.
    pub fn iter(&self) -> impl Iterator<Item = Tag> + '_ {
        self.order.iter().copied()
    }

    pub fn from_raw<'a>(raw: impl IntoIterator<Item = (&'a String, &'a TagValue)>) -> Result<Tags, String> {
        let mut t = Tags::default();
        let (mut ints, mut strs, mut order) = (vec![], vec![], vec![]);
        for (name, v) in raw {
            let tag = Tag::from_name(name).ok_or_else(|| format!("unknown tag {name:?}"))?;
            match (tag.kind(), v) {
                (TagKind::Flag, TagValue::Flag(true)) => {}
                (TagKind::Int, TagValue::Value(s)) => {
                    ints.push((tag, s.parse::<i32>().map_err(|_| format!("tag {name}={s}: not a number"))?))
                }
                (TagKind::Str, TagValue::Value(s)) => strs.push((tag, s.as_str().into())),
                (k, v) => return Err(format!("tag {name}: {v:?} where a {k:?} was expected")),
            }
            let i = tag as usize;
            t.bits[i / 64] |= 1 << (i % 64);
            order.push(tag);
        }
        t.ints = ints.into();
        t.strs = strs.into();
        t.order = order.into();
        Ok(t)
    }
}

// ------------------------------------------------------------------ card definitions
/// The condition a land enters untapped under (Python's `ais.enters_rule`, read from the Oracle text). Basic land
/// types are kept as colours: Plains W, Island U, Swamp B, Mountain R, Forest G.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnterRule {
    /// you control a land of one of these types (check lands)
    Control(Colors),
    /// you control n or fewer other lands (fast lands)
    Fewer(u32),
    /// you control n or more other lands (slow lands)
    More(u32),
    /// you control n or more basic lands (battle lands)
    Basics(u32),
    /// you have n or more opponents (Bond lands)
    Opponents(u32),
    /// you control n or more other lands of this type
    TypesMore(Colors, u32),
    /// reveal a land card of one of these types from your hand (snarls)
    Reveal(Colors),
    /// you control a legendary creature
    Legendary,
    /// you control a planeswalker (the Annexes)
    Planeswalker,
    /// pay n life or it enters tapped (shock lands)
    Pay(i32),
}

/// basic land type name -> its colour letter
pub fn basic_type_color(t: &str) -> Option<char> {
    Some(match t {
        "plains" => 'W',
        "island" => 'U',
        "swamp" => 'B',
        "mountain" => 'R',
        "forest" => 'G',
        _ => return None,
    })
}

fn types_to_colors(v: &[serde_json::Value]) -> Colors {
    let s: String = v.iter().filter_map(|x| x.as_str()).filter_map(basic_type_color).collect();
    Colors::from_letters(&s)
}

impl EnterRule {
    fn from_json(v: &[serde_json::Value]) -> Result<EnterRule, String> {
        let kind = v.first().and_then(|x| x.as_str()).ok_or("an enters rule without a kind")?;
        let n = |i: usize| v.get(i).and_then(|x| x.as_i64()).ok_or(format!("enters rule {kind}: no number"));
        let types = |i: usize| match v.get(i) {
            Some(serde_json::Value::Array(a)) => Ok(types_to_colors(a)),
            Some(serde_json::Value::String(s)) => {
                Ok(Colors::from_letters(&basic_type_color(s).map(String::from).unwrap_or_default()))
            }
            _ => Err(format!("enters rule {kind}: no types")),
        };
        Ok(match kind {
            "control" => EnterRule::Control(types(1)?),
            "fewer" => EnterRule::Fewer(n(1)? as u32),
            "more" => EnterRule::More(n(1)? as u32),
            "basics" => EnterRule::Basics(n(1)? as u32),
            "opponents" => EnterRule::Opponents(n(1)? as u32),
            "types_more" => EnterRule::TypesMore(types(1)?, n(2)? as u32),
            "reveal" => EnterRule::Reveal(types(1)?),
            "legendary" => EnterRule::Legendary,
            "planeswalker" => EnterRule::Planeswalker,
            "pay" => EnterRule::Pay(n(1)? as i32),
            k => return Err(format!("unknown enters rule {k:?}")),
        })
    }
}

/// A land's "when this land enters" effect (Python's `land_etb_fx`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandEtb {
    Scry(i32),
    Gain(i32),
}

#[derive(Debug, Clone)]
pub struct CardDef {
    pub id: CardId,
    pub name: Box<str>,
    pub types: Types,
    pub generic: u32,
    /// coloured pips of the cost ("UUB")
    pub pips: Box<str>,
    pub cmc: u32,
    pub land: bool,
    pub creature: bool,
    pub instant: bool,
    pub sorcery: bool,
    /// a permanent spell (not instant, sorcery or land)
    pub perm: bool,
    pub tags: Tags,
    pub pow: i32,
    pub tgh: i32,
    /// how big a threat the card is (0-10), tag `bomb`
    pub bomb: i32,
    /// the colours in its cost (Python's `colors_of` for a card)
    pub colors: Colors,
    /// colour identity; None for the hand-tagged cards that don't record one
    pub identity: Option<Colors>,
    /// compiled abilities (the ability language), empty for a card without them
    pub abilities: std::sync::Arc<[crate::dsl::model::Ability]>,
    pub start_loyalty: Option<i32>,
    /// Scryfall keywords, lower case
    pub kws: Box<[Box<str>]>,
    /// subtypes, lower case
    pub subtypes: Box<[Box<str>]>,
    pub protfrom: Colors,
    pub ward: u32,
    pub game_changer: bool,
    /// where its definition came from ('manual', 'scryfall', 'scryfall+dsl' ...)
    pub source: Box<str>,
    /// the colours of its Phyrexian mana symbols, in cost order ("BB" for {B/P}{B/P})
    pub phyrexian: Box<str>,
    /// a land's "when this land enters" effects
    pub land_etb_fx: Box<[LandEtb]>,
    /// the condition a land enters untapped under, if its text has one
    pub enters_rule: Option<EnterRule>,
    /// a land's basic land types, as colours (Plains W ... Forest G)
    pub land_types: Colors,
    /// a basic land
    pub basic: bool,
    /// events its Python implementation handles (for tracking the port; empty for cards with no hooks)
    pub python_hooks: Box<[Box<str>]>,
}

impl CardDef {
    /// built from Scryfall data and auto-tagged (Python's `cd.source == 'scryfall'`)
    pub fn source_scryfall(&self) -> bool {
        &*self.source == "scryfall"
    }

    pub fn tag(&self, t: Tag) -> bool {
        self.tags.has(t)
    }

    /// has compiled abilities (Python's `if cd.dsl:`)
    pub fn has_dsl(&self) -> bool {
        !self.abilities.is_empty()
    }

    pub fn has_kw(&self, kw: &str) -> bool {
        self.kws.iter().any(|k| &**k == kw)
    }

    pub fn has_subtype(&self, s: &str) -> bool {
        self.subtypes.iter().any(|k| &**k == s)
    }

    fn from_raw(id: CardId, r: &RawCard) -> Result<CardDef, String> {
        let at = |e: String| format!("{}: {e}", r.name);
        let types = Types::parse(&r.types).map_err(at)?;
        let d = r.compute_derived();
        if d != r.derived {
            return Err(at(format!("derived fields differ from Python's: {d:?} vs {:?}", r.derived)));
        }
        let start_loyalty = match &r.start_loyalty {
            None => None,
            Some(s) => Some(s.parse::<i32>().map_err(|_| at(format!("starting loyalty {s:?}")))?),
        };
        Ok(CardDef {
            id,
            name: r.name.as_str().into(),
            types,
            generic: r.generic,
            pips: r.pips.as_str().into(),
            cmc: d.cmc,
            land: d.land,
            creature: d.creature,
            instant: d.instant,
            sorcery: d.sorcery,
            perm: d.perm,
            tags: Tags::from_raw(&r.tags).map_err(at)?,
            pow: r.pow,
            tgh: r.tgh,
            bomb: r.bomb,
            colors: Colors::from_letters(&r.pips),
            identity: r.identity.as_deref().map(Colors::from_letters),
            abilities: match &r.dsl {
                Some(v) => crate::dsl::model::parse(v).map_err(|e| at(format!("abilities: {e}")))?.into(),
                None => std::sync::Arc::from(Vec::new()),
            },
            start_loyalty,
            kws: r.kws.iter().map(|s| s.as_str().into()).collect(),
            subtypes: r.subtypes.iter().map(|s| s.as_str().into()).collect(),
            protfrom: Colors::from_letters(&r.protfrom),
            ward: r.ward,
            game_changer: r.game_changer == Some(true),
            source: r.source.as_str().into(),
            phyrexian: r.phyrexian.as_str().into(),
            land_etb_fx: r
                .land_etb_fx
                .iter()
                .map(|(k, n)| match k.as_str() {
                    "scry" => Ok(LandEtb::Scry(*n)),
                    "gain" => Ok(LandEtb::Gain(*n)),
                    _ => Err(at(format!("land enters effect {k:?}"))),
                })
                .collect::<Result<_, _>>()?,
            enters_rule: match &r.enters_rule {
                Some(v) => Some(EnterRule::from_json(v).map_err(at)?),
                None => None,
            },
            land_types: Colors::from_letters(
                &r.land_types.iter().filter_map(|t| basic_type_color(t)).collect::<String>(),
            ),
            basic: r.basic,
            python_hooks: r.python_hooks.iter().map(|s| s.as_str().into()).collect(),
        })
    }
}

/// Cards Python's card code makes on the fly (`E.CD(...)`) without putting them in the card database, so the export
/// doesn't have them; they are added after the exported cards: the land Legion's Landing transforms into
/// (t1.ADANTO: `CD('Adanto, the First Fort', 'L', '-', 'c=W')`).
pub fn engine_made() -> Vec<RawCard> {
    let mut tags = indexmap::IndexMap::new();
    tags.insert("c".to_string(), TagValue::Value("W".to_string()));
    let mut r = RawCard {
        name: "Adanto, the First Fort".into(),
        types: "L".into(),
        generic: 0,
        pips: String::new(),
        tags,
        pow: 0,
        tgh: 0,
        bomb: 0,
        dsl: None,
        start_loyalty: None,
        kws: vec![],
        subtypes: vec![],
        protfrom: String::new(),
        ward: 0,
        identity: None,
        game_changer: None,
        source: "manual".into(),
        unparsed: vec![],
        derived: crate::export::Derived {
            cmc: 0,
            land: false,
            creature: false,
            instant: false,
            sorcery: false,
            perm: false,
        },
        phyrexian: String::new(),
        land_etb_fx: vec![],
        enters_rule: None,
        land_types: vec![],
        basic: false,
        python_hooks: vec![],
        spell_prio: false,
    };
    r.derived = r.compute_derived();
    let mut out = vec![r.clone()];
    // the back faces mine.py's prepared cards cast copies of, when the card data has no card of that name
    // (engine.back_face: `CD(name, 'I' if instant else 'S', '0', '')`)
    for (name, types) in [("Ancestral Recall", "I"), ("Wild Idea", "S")] {
        let mut b = r.clone();
        b.name = name.into();
        b.types = types.into();
        b.tags = indexmap::IndexMap::new();
        b.derived = b.compute_derived();
        out.push(b);
    }
    out
}

/// Corrections to exported card definitions (bugs in the Python data, fixed in Rust only). Heliod's Pilgrim: its
/// compiled enters trigger searches for any card (filter `{}`), on top of the card code's search for an Aura, so it
/// found two cards; the compiled trigger goes, and the card code's Aura search stays.
fn data_fixes(d: &mut CardDef) {
    if &*d.name == "Heliod's Pilgrim" {
        d.abilities = d
            .abilities
            .iter()
            .filter(|a| !matches!(a, crate::dsl::model::Ability::Triggered { event, .. } if event == "etb"))
            .cloned()
            .collect();
    }
}

/// Every card definition, by id and by name.
#[derive(Debug)]
pub struct CardDb {
    cards: Vec<CardDef>,
    by_name: HashMap<Box<str>, CardId>,
}

impl CardDb {
    pub fn from_raw(raw: &[RawCard]) -> Result<CardDb, String> {
        if raw.len() > u16::MAX as usize {
            return Err(format!("{} cards: more than a CardId can number", raw.len()));
        }
        let extra: Vec<RawCard> = engine_made().into_iter().filter(|e| !raw.iter().any(|r| r.name == e.name)).collect();
        let mut cards = Vec::with_capacity(raw.len() + extra.len());
        let mut by_name = HashMap::with_capacity(raw.len() + extra.len());
        for (i, r) in raw.iter().chain(&extra).enumerate() {
            let id = CardId(i as u16);
            if by_name.insert(r.name.as_str().into(), id).is_some() {
                return Err(format!("{}: defined twice", r.name));
            }
            let mut d = CardDef::from_raw(id, r)?;
            data_fixes(&mut d);
            cards.push(d);
        }
        Ok(CardDb { cards, by_name })
    }

    pub fn load(path: &std::path::Path) -> Result<CardDb, String> {
        let raw = crate::export::load_cards(path).map_err(|e| format!("{}: {e}", path.display()))?;
        CardDb::from_raw(&raw)
    }

    pub fn get(&self, id: CardId) -> &CardDef {
        &self.cards[id.index()]
    }

    pub fn id(&self, name: &str) -> Option<CardId> {
        self.by_name.get(name).copied()
    }

    pub fn by_name(&self, name: &str) -> Option<&CardDef> {
        self.id(name).map(|i| self.get(i))
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &CardDef> {
        self.cards.iter()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::export::load_cards;
    use std::path::PathBuf;
    use std::sync::{Arc, OnceLock};

    pub fn data(f: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data").join(f)
    }

    /// The card database, loaded once for all tests.
    pub fn db() -> Arc<CardDb> {
        static DB: OnceLock<Arc<CardDb>> = OnceLock::new();
        DB.get_or_init(|| Arc::new(CardDb::load(&data("cards.json")).unwrap())).clone()
    }

    #[test]
    fn every_card_builds_and_its_tags_read_back_as_exported() {
        let raw = load_cards(&data("cards.json")).unwrap();
        let db = db();
        let extra = engine_made().iter().filter(|e| !raw.iter().any(|r| r.name == e.name)).count();
        assert_eq!(db.len(), raw.len() + extra);
        for r in &raw {
            let c = db.by_name(&r.name).unwrap();
            let names: Vec<&str> = c.tags.iter().map(|t| t.name()).collect();
            assert_eq!(names, r.tags.keys().map(|k| k.as_str()).collect::<Vec<_>>(), "{}", r.name);
            for (k, v) in &r.tags {
                let t = Tag::from_name(k).unwrap();
                assert!(c.tag(t));
                match (v, c.tags.get(t).unwrap()) {
                    (TagValue::Flag(true), TagVal::Flag) => {}
                    (TagValue::Value(s), TagVal::Int(n)) => assert_eq!(s.parse::<i32>().unwrap(), n),
                    (TagValue::Value(s), TagVal::Str(x)) => assert_eq!(s.as_str(), &*x),
                    (a, b) => panic!("{}: tag {k}: {a:?} vs {b:?}", r.name),
                }
            }
        }
    }

    #[test]
    fn a_few_cards_as_python_has_them() {
        let db = db();
        let ring = db.by_name("Sol Ring").unwrap();
        assert!(ring.types.has(Types::ARTIFACT) && ring.perm && ring.cmc == 1);
        assert_eq!(ring.tags.str(Tag::Rock), Some("2:C"));
        let circle = db.by_name("Teleportation Circle").unwrap();
        assert_eq!((circle.generic, &*circle.pips, circle.cmc), (3, "W", 4));
        assert!(circle.colors.has('W') && circle.colors.count() == 1);
        let atraxa = db.by_name("Atraxa, Grand Unifier").unwrap();
        assert!(atraxa.creature && atraxa.tag(Tag::Fly) && atraxa.pow == 7);
        assert!(db.by_name("Not A Card").is_none());
    }
}
