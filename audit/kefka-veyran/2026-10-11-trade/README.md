# Veyran → Kefka trade (2026-10-11)

Moving Veyran's strong but underused cards to Kefka, as functional blocks (a card and what supports it), with Veyran
refilled only from Kefka's cuts and the bulk list. Look-ahead AI, loose profile, 1,000 paired games per tier (same
seeds, seats and opponents' draws), on the code after the kefka-ai-fix merge (23667f5). `run.sh <test>` reruns one.

**The trade**

| Block | Veyran → Kefka | Kefka's slot |
|---|---|---|
| Protection | Deflecting Swat, Slip Out the Back | Arcane Omens, Triskelion |
| Graveyard | Underworld Breach | Inner Fire |
| Engines | Niv-Mizzet, the Firemind, Jin-Gitaxias, Progress Tyrant | Mystic Confluence, Splatter Technique |
| Mana | Steam Vents, Sulfur Falls, Spirebluff Canal, Stormcarved Coast | 4 Mountain |

Veyran's refills: Mystic Confluence and Splatter Technique (from Kefka); Return the Favor, Spell Pierce, Lightning
Strike (bulk); 2 Island, 2 Mountain.

**Results** (win rate before → after, change)

| Test | T1 | T2 | T3 | T4 | T5 | All |
|---|---|---|---|---|---|---|
| `veyran-full` | 25.1 → 29.7 (+4.6) | 28.6 → 28.5 (−0.1) | 22.8 → 27.8 (+5.0) | 22.6 → 23.7 (+1.1) | 15.2 → 12.9 (−2.3) | 22.9 → 24.5 (**+1.7**) |
| `kefka-full` | 26.6 → 27.7 (+1.1) | 38.1 → 39.8 (+1.7) | 24.4 → 27.8 (+3.4) | 29.4 → 29.3 (−0.1) | 21.9 → 22.4 (+0.5) | 28.1 → 29.4 (**+1.3**) |
| `kefka-protection` | +1.3 | 0.0 | +0.7 | −0.2 | +1.4 | +0.6 |

Noise is about ±3.5 points per tier (±1.6 over all tiers). Both decks gain over all tiers and neither loses more than
noise in any tier, so JD took the whole package. The Breach-only and engines-only tests were stopped as unneeded.
