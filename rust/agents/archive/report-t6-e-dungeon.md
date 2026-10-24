# Report t6-e-dungeon
Mechanism: rust/crates/sim-core/src/engine/dungeon.rs (DUNGEONS table, venture, current, AI dungeon/branch choice by room value);
state.rs Player.dungeon (Option<(dungeon, room)>) + dungeons_done; t6_dack::venture calls it. Grudge stand-in removed.
Venture: enter first room of chosen dungeon if none; else move to connected room; leaving the last room completes it (counted) and starts a new one.
Full: Radiant Solar (and Preston copies). Only Radiant Solar ventures in the pools; no initiative/Undercity cards exist (Undercity Sewers/Dire Rat are unrelated).
Simplified rooms (AI avoids them): Fungi Cavern, Twisted Caverns log only; Veils/Sandfall just lose life; Oubliette discards only; Runestone Caverns draws 2; Mad Wizard's Lair draws 3 without the free cast.
Behaviour change: none for other decks. Note: the last room is now left by the next venture (old stand-in reset at once); same drain rate.
