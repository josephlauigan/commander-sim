//! Dungeons and `venture` (rule 701.49): per-player dungeon state (`Player::dungeon`, `Player::dungeons_done`), the
//! three Adventures in the Forgotten Realms dungeons as data (rooms with the rooms they lead to, and an effect), and
//! the AI's choice of dungeon and branch. Venture: with no dungeon, enter the first room of a dungeon of your choice;
//! otherwise move to a room the current one leads to; the last room completes the dungeon (the next venture starts a
//! new one). Callers: Radiant Solar (impls/t6_dack.rs).
//!
//! Simplifications (each room the AI would not pick is still resolved, but loosely):
//! - Fungi Cavern (-4/-0) and Twisted Caverns (can't attack) only log; the AI never picks them.
//! - Veils of Fear / Sandfall Cell: each player loses the life (nobody pays the discard / sacrifice).
//! - Runestone Caverns: the top two cards are put into the hand (instead of exile-and-play); Mad Wizard's Lair draws
//!   three but the free cast is not made.

use crate::engine::repl;
use crate::cards::Colors;
use crate::engine::life::{gain, lose_life};
use crate::engine::values::epow;
use crate::engine::zones::{Tokens, add_treasure, discard_cards, draw, make_tokens};
use crate::flow::Res;
use crate::ids::PlayerId;
use crate::state::Game;
use crate::sym::intern;

/// What a room does. Names are for the log.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fx {
    Scry(i32),
    Goblin,
    Treasure,
    Counter,
    Drain,
    Draw(u32),
    Log,
    EachLoses(i32),
    Oubliette,
    Atropal,
    Gain(i32),
    Skeletons,
    Cards2,
    MadWizard,
}

pub struct Room {
    pub name: &'static str,
    pub fx: Fx,
    /// indexes (into the dungeon's room list) of the rooms this one leads to; empty = the last room
    pub next: &'static [u8],
}

pub struct Dungeon {
    pub name: &'static str,
    pub rooms: &'static [Room],
}

const fn r(name: &'static str, fx: Fx, next: &'static [u8]) -> Room {
    Room { name, fx, next }
}

pub static DUNGEONS: [Dungeon; 3] = [
    Dungeon {
        name: "Lost Mine of Phandelver",
        rooms: &[
            r("Cave Entrance", Fx::Scry(1), &[1, 2]),
            r("Goblin Lair", Fx::Goblin, &[3, 4]),
            r("Mine Tunnels", Fx::Treasure, &[4, 5]),
            r("Storeroom", Fx::Counter, &[6]),
            r("Dark Pool", Fx::Drain, &[6]),
            r("Fungi Cavern", Fx::Log, &[6]),
            r("Temple of Dumathoin", Fx::Draw(1), &[]),
        ],
    },
    Dungeon {
        name: "Tomb of Annihilation",
        rooms: &[
            r("Trapped Entry", Fx::EachLoses(1), &[1, 3]),
            r("Veils of Fear", Fx::EachLoses(2), &[2]),
            r("Sandfall Cell", Fx::EachLoses(2), &[4]),
            r("Oubliette", Fx::Oubliette, &[4]),
            r("Cradle of the Death God", Fx::Atropal, &[]),
        ],
    },
    Dungeon {
        name: "Dungeon of the Mad Mage",
        rooms: &[
            r("Yawning Portal", Fx::Gain(1), &[1]),
            r("Dungeon Level", Fx::Scry(1), &[2, 3]),
            r("Goblin Bazaar", Fx::Treasure, &[4]),
            r("Twisted Caverns", Fx::Log, &[4]),
            r("Lost Level", Fx::Scry(2), &[5, 6]),
            r("Runestone Caverns", Fx::Cards2, &[7]),
            r("Muiral's Graveyard", Fx::Skeletons, &[7]),
            r("Deep Mines", Fx::Scry(3), &[8]),
            r("Mad Wizard's Lair", Fx::MadWizard, &[]),
        ],
    },
];

/// How much the AI likes a room (higher = go there). Drain and card draw first, harmless filler next, rooms that
/// hurt the venturer or do nothing last.
fn room_value(fx: Fx) -> f64 {
    match fx {
        Fx::Drain => 5.0,
        Fx::Draw(_) | Fx::MadWizard => 4.0,
        Fx::Goblin | Fx::Skeletons | Fx::Atropal => 3.0,
        Fx::Counter | Fx::Treasure => 2.5,
        Fx::Gain(_) | Fx::Cards2 => 2.0,
        Fx::Scry(_) => 1.5,
        Fx::EachLoses(_) | Fx::Oubliette => 0.5,
        Fx::Log => 0.1,
    }
}

/// The dungeon the AI enters: the one whose best reachable room is worth most per room walked; Lost Mine unless
/// something beats its Dark Pool (nothing does at present), Tomb of Annihilation costs the venturer life too.
fn choose_dungeon(_g: &Game, _p: PlayerId) -> usize {
    let score = |d: &Dungeon| {
        let best = d.rooms.iter().map(|x| room_value(x.fx)).fold(0.0, f64::max);
        best / d.rooms.len() as f64 + if d.name == "Tomb of Annihilation" { -1.0 } else { 0.0 }
    };
    (0..DUNGEONS.len()).max_by(|&a, &b| score(&DUNGEONS[a]).total_cmp(&score(&DUNGEONS[b]))).unwrap_or(0)
}

/// Branch choice: the connected room whose path is worth most (its own value plus the best of what follows).
fn path_value(d: &Dungeon, room: u8) -> f64 {
    let x = &d.rooms[room as usize];
    room_value(x.fx) + x.next.iter().map(|&n| path_value(d, n)).fold(0.0, f64::max)
}

/// `Some((dungeon index, room index))` of p's current room.
pub fn current(g: &Game, p: PlayerId) -> Option<(usize, u8)> {
    g.player(p).dungeon
}

/// Venture into the dungeon (701.49a): enter the first room of a dungeon of your choice if you are not in one,
/// else move to a room the current one leads to. Then the room's effect happens; a room that leads nowhere
/// completes the dungeon when the venture *leaves* it, so the next venture starts a new one.
pub fn venture(g: &mut Game, p: PlayerId) -> Res {
    let (di, ri) = match g.player(p).dungeon {
        None => {
            let di = choose_dungeon(g, p);
            crate::glog!(g, "    {} ventures into {}", g.player(p).name, DUNGEONS[di].name);
            (di, 0u8)
        }
        Some((di, cur)) => {
            let d = &DUNGEONS[di];
            let next = d.rooms[cur as usize].next;
            if next.is_empty() {
                // leaving the last room: the dungeon is completed, then a new one is entered
                g.player_mut(p).dungeon = None;
                g.player_mut(p).dungeons_done += 1;
                crate::glog!(g, "    {} completes {}", g.player(p).name, d.name);
                return venture(g, p);
            }
            let to = next.iter().copied().max_by(|&a, &b| path_value(d, a).total_cmp(&path_value(d, b))).unwrap();
            (di, to)
        }
    };
    g.player_mut(p).dungeon = Some((di, ri));
    let room = &DUNGEONS[di].rooms[ri as usize];
    crate::glog!(g, "    {} ventures: {}", g.player(p).name, room.name);
    run_fx(g, p, room.fx)
}

fn run_fx(g: &mut Game, p: PlayerId, fx: Fx) -> Res {
    match fx {
        Fx::Scry(n) => crate::cardcode::scry(g, p, n, false)?,
        Fx::Goblin => {
            let spec =
                Tokens { color: Some(Colors::from_letters("R")), types: vec![intern("goblin")], ..Tokens::new(1, 1) };
            make_tokens(g, p, spec)?;
        }
        Fx::Treasure => add_treasure(g, p, 1)?,
        Fx::Counter => {
            let best = g
                .player(p)
                .perms
                .iter()
                .copied()
                .filter(|&m| g.perm(m).on_bf && g.is_creature(m))
                .max_by_key(|&m| epow(g, m));
            if let Some(m) = best {
                repl::put_plus(g, m, 1);
            }
        }
        Fx::Drain => {
            for q in g.opps(p).filter(|&q| g.player(q).alive).collect::<Vec<_>>() {
                lose_life(g, q, 1, Some(p), intern("drain"), None)?;
            }
            gain(g, p, 1)?;
        }
        Fx::Draw(n) => draw(g, p, n, false)?,
        Fx::Log => {}
        Fx::EachLoses(n) => {
            let all: Vec<PlayerId> = g.players.iter().filter(|x| x.alive).map(|x| x.id).collect();
            for q in all {
                lose_life(g, q, n, Some(p), intern("dungeon"), None)?;
            }
        }
        Fx::Oubliette => {
            // discard a card and sacrifice a creature, an artifact and a land: the discard only (the sacrifices are
            // not modeled; the AI avoids this room)
            if let Some(&c) = g.player(p).hand.last() {
                discard_cards(g, p, &[c])?;
            }
        }
        Fx::Atropal => {
            let spec = Tokens {
                color: Some(Colors::from_letters("B")),
                types: vec![intern("god"), intern("horror")],
                dt: true,
                tgh: Some(4),
                ..Tokens::new(1, 4)
            };
            make_tokens(g, p, spec)?;
        }
        Fx::Gain(n) => gain(g, p, n)?,
        Fx::Skeletons => {
            let spec =
                Tokens { color: Some(Colors::from_letters("B")), types: vec![intern("skeleton")], ..Tokens::new(2, 1) };
            make_tokens(g, p, spec)?;
        }
        Fx::Cards2 => draw(g, p, 2, false)?,
        Fx::MadWizard => draw(g, p, 3, false)?,
    }
    Ok(())
}
