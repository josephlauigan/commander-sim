# Jodah leftovers in JD's and Avery's decks, 2026-10-14

Paired tests (same seeds, seats and opponents), 500 games per list per tier, Tiers 1-5, look-ahead AI, loose
profile; main 4de3b27. Noise of a five-tier average is about ±1.5-2.3 points. `run.sh` runs a queue of jobs;
`analyze-<deck>-t*.txt` are the baseline analyses used to pick the spell cuts. Seph/Atraxa is the Atraxa reanimator
deck (key `seph`), not the new Seph FS deck.

| Deck | Swaps | T1 | T2 | T3 | T4 | T5 | Average | JD |
|---|---|---|---|---|---|---|---|---|
| Sauron | Mountain => Underground River | -1.2 | -2.4 | -4.0 | -1.0 | +0.0 | **-1.7** | dropped |
| Seph/Atraxa | Orzhov Signet => Worldly Tutor | -2.0 | +0.8 | +2.2 | -1.0 | -0.4 | **-0.1** | kept |
| Y'shtola | Triplicate Spirits => Dakkon, Shadow Slayer; Swamp => Underground River; Island => Adarkar Wastes | +0.2 | -4.2 | -1.8 | +2.2 | +0.0 | **-0.7** | |
| Kefka | Decorum Dissertation => Bitter Triumph; Swamp => Sulfurous Springs; Swamp => Shivan Reef | -6.0 | -3.4 | -2.6 | +2.2 | +0.4 | **-1.9** | |
| Alela (Avery) | Gravedigger => Tymna the Weaver; Plains => Exotic Orchard; Plains => Shattered Sanctum | +1.2 | +2.2 | -0.6 | +1.4 | -3.8 | **+0.1** | |
| Galadriel (Avery) | Flicker => Dragonlord Dromoka | +0.8 | +3.0 | -0.8 | +2.2 | +0.4 | **+1.1** | |

Found on the way: the simulator never casts Kefka's Deflecting Swat (untagged; 0-5% of the games it is drawn).
