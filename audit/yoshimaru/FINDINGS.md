# Yoshimaru and Tymna (key `yoshimaru`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/yoshimaru-tymna-blink.md` (Yoshimaru, Ever Faithful and Tymna the Weaver, plain Partner, white-black blink and
enter-effect midrange; 100 cards). **Tiers:** t4 and t5 (`data/my_decks.toml`). **No sims yet.**
The findings of the retired Tymna/Thrasios Doomsday deck are in `audit/retired/thrasios/`.

## What the simulator cannot do yet
- Every card is Full except Delney, Streetwise Lookout (Approximate: the extra instance for leaves that run at once or inline in the
  engine is left out; see her note in `data/audit.toml`).
- `data/plans/yoshimaru.toml` is provisional: priorities come from the guide, unmeasured; the keep rule is the generic lands 2 to 5.
  It leaves out the Archangel of Thune + Walking Ballista line and the guide's Restoration Angel + Felidar Guardian line (Spellbook
  1090-2781, listed in `KNOWN_GAPS` in `tests/spellbook.rs`).
