# Sauron audit (2026-10-04/05): findings

Why Sauron (`decklists/JD/sauron-grixis-amass.md`) sat third of JD's four decks, what was tried, and what changed.
All numbers are paired comparisons (same seeds, 1,000 games per tier, adaptive AI, loose profile) unless noted.

## What holds Sauron back

- **One engine, no protection.** The deck wins when the Army gets big (peak ≥16 in 26% of games → 58% win; ≤5 in 36%
  → 8%). It dies with an empty board (Army power median 0, one creature): combat at Tiers 1-3, combo at Tier 5.
- **Support packages don't move it.** More Army growth, attack taxes, cutting wipes, and Sword tutors were all flat
  (−1.9 to +0.6 points).
- **The Underworld Breach line wins every time it's played,** but rarely gets started: its real constraint is blue
  mana for each Brain Freeze plus starting resources. Lion's Eye Diamond fixes that (+4.6 with Breach tutors), but it's
  out of budget. Non-GC substitutes (Birgi, Laboratory Maniac, Jeska's Will, Grim Tutor, Merchant Scroll) gave +0.5 to
  +1.4. Holding mana for the combo turn wouldn't help: failed lines were usually short by more than six mana.
- Conclusion: a mid-strength deck in this simulator; no affordable rebuild found.

## Decklist changes made (10-05)

| Deck | Out → In | Tested | Game Changers |
| --- | --- | --- | --- |
| Sauron | Consecrated Sphinx → Fact or Fiction | +0.1 (no loss) | 3 → Bracket 3 |
| Sephiroth | Bolas's Citadel → Phyrexian Arena | 0.0 (no loss) | 8 |
| Y'shtola | Enslave → Consecrated Sphinx, Plea for Guidance → Bolas's Citadel, Jester's Cap → Smothering Tithe | +1.2, ahead in all tiers | 5 → Bracket 4 |
| Veyran | Crawlspace → Propaganda | the swap planned on 10-04 | 3 |

Cuts went to `bulk-cards.txt` (the binder list was merged into it). To buy: Smothering Tithe, Fact or Fiction,
Propaganda.

## Simulator changes

- AI fixes: wipes that hit nothing of the opponents' are no longer cast; Aggravated Assault keeps going while the Army
  connects with Sword of Feast and Famine (it stopped after one extra combat).
- Breach line: Lion's Eye Diamond, Laboratory Maniac / Jace, Wielder of Mysteries (drawing from an empty library
  wins), Birgi and Laboratory Maniac cast from hand first, Jeska's Will; the dry run tries with and without those;
  three spare graveyard cards in the mill plan. Tested in `tests/test_my_cards.py` (`SauronBreach`).
- Contextual priorities instead of flat numbers: `t4.sphinx_prio` (Consecrated Sphinx) and `rules.tithe_prio`
  (Smothering Tithe), used by every deck AI that casts them. Gamble, the wheels and Reforge the Soul are castable by
  Sauron's AI.

## Scripts here

```
PYTHONPATH=. python3 audit/sauron/routes.py run t3 0 200 t3.json   # how Sauron wins and loses (run tiers in parallel)
PYTHONPATH=. python3 audit/sauron/routes.py report t*.json
PYTHONPATH=. python3 audit/sauron/breach_stats.py t3 200 ["Out=>In" ...]    # Breach line funnel
PYTHONPATH=. python3 audit/sauron/breach_deficit.py t3 200 ["Out=>In" ...]  # what the Breach line is short of
PYTHONPATH=. python3 audit/sauron/army_stats.py 300                          # when Sauron lands, Army size
PYTHONPATH=. python3 audit/sauron/removal_causes.py 200                      # who removes Sauron and the Army
```
