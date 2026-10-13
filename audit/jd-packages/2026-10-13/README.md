# JD's engine packages, 2026-10-13

Paired tests (same seeds, seats and opponents), 500 games per list per tier, Tiers 1-5, look-ahead AI, loose profile.
Binary: jd-packages b8d9944 (agent jd-pkg-cards: Vorinclex modeled, Kefka Will/tutor AI, Terror of the Peaks on
tokens, Kura death modes, Archivist counts Sheoldred) + the run script a5e92ea. Script: `run.sh`; per-tier output
`pkg-<X>-t<N>.txt`; log `log.txt`. Noise of a five-tier average is about ±2 points.

| Package | Swaps | T1 | T2 | T3 | T4 | T5 | Average |
|---|---|---|---|---|---|---|---|
| C Sauron | Bedevil => Sheoldred, the Apocalypse | +5.0 | +4.8 | +0.6 | +2.8 | +0.2 | **+2.7** |
| D Titania | Yavimaya Wurm => Vorinclex, Elfhame Wurm => Kura, Cactarantula => Tireless Tracker, Penumbra Spider => Cartographer's Survey | -3.2 | +4.2 | +0.4 | +4.8 | +2.8 | **+1.8** |
| A Kefka | Twinflame => Yawgmoth's Will, Kaervek's Purge => Vampiric Tutor, Disintegrate => Mystical Tutor | -3.2 | +1.4 | +1.6 | +2.2 | +2.8 | **+1.0** |
| B Veyran | Lightning Strike => Underworld Breach, Visions of Beyond => Terror of the Peaks, Flux Channeler => Mirari | -2.0 | -5.0 | -1.6 | +3.4 | -1.4 | **-1.3** |

Baseline analyses (cut picks): `analyze-<deck>-t<N>.txt` (320 games, T1/T3/T5).

## Sephiroth: Phyrexian Altar, Phyrexian Tower, Sephiroth, Fabled SOLDIER (not applied)

Same method (500 paired games per tier), binary with PR #61's Tower fix and sacrifice-outlet AI. Scripts and output in
`seph/` (baseline analysis `seph/analyze-seph-t*.txt`).

| Test | Swaps | T1 | T2 | T3 | T4 | T5 | Average |
|---|---|---|---|---|---|---|---|
| Two cards | Swamp => Phyrexian Tower, Mind Stone => Phyrexian Altar | +2.0 | +2.4 | +1.0 | -2.0 | -1.4 | **+0.4** |
| Three cards | the two above + Flickering Hound => Sephiroth, Fabled SOLDIER | +1.0 | +3.2 | +2.2 | -2.6 | -2.4 | **+0.3** |

Both within noise (about ±2.3 for the average): no measurable gain.
