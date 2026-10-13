# Task: docs-cloud-c4 — update four deck docs to their new lists (text only)

Hard rules: never run Python; no sims, no cargo builds needed (docs only); work only in your worktree (branch
`docs-cloud-c4`, from `cloud-midgar`); commit when done; don't push. Edit only the four docs named below. The import
lists and type-grouped lists in them were already updated by `cardgen deck` (commit 8137fe2): don't change which cards
are in a deck. Your job is the prose, the change logs, the source tables, the bracket sections and buy lists, so every
doc matches its list. Use `git show 8137fe2 -- decklists/JD/<file>` to see what changed, and keep each doc's style
(look at `decklists/JD/kefka-grixis-discard.md`, updated today, for the change-log and source-table format).

## 1. decklists/JD/cloud-midgar-equipment.md (rewrite most of it)
JD committed to the C3 + C4 list on 2026-10-13 (27 swaps; `audit/cloud/2026-10-13/screen/c4.swaps`). The doc's
strategy text, "Where the cards come from" and decklist commentary still describe the old list. Rewrite them:
- **Change log line** (2026-10-13): what went out and in, by group: mana (Sol Ring, Arcane Signet, Mind Stone, Fellwar
  Stone, Coalition Relic, Thought Vessel, Smothering Tithe, Solemn Simulacrum, Knight of the White Orchid, Loyal
  Warhound; 33 lands, Plaza of Heroes), swords and protection (Sword of Fire and Ice, of Feast and Famine, of Hearth and
  Home, Champion's Helm, Swiftfoot Boots, Giver of Runes, Grand Abolisher), free equips and backup carriers (Ardenn,
  Auriok Steelshaper, Kor Outfitter, Balan, Armored Skyhunter, Kor Duelist), others from the retired Jodah deck
  (Enlightened Tutor, Skyclave Apparition, Court of Ardenvale). Not measured yet (C3 alone, before the AI fixes: t1
  +2.6, t2 +5.0, t3 +0.6; `audit/cloud/2026-10-13/`).
- **Strategy:** keep Cloud's own text and the doubled-trigger idea; rewrite the plan around: ramp to 4-5 mana; Cloud
  tutors a cheap Sword or the Helm first; free equips (Ardenn each combat, Kor Outfitter, Steelshaper's discount,
  Puresteel Paladin); backup carriers when Cloud is dead (Balan, Kor Duelist, Armored Skyhunter, Bronze Guardian,
  Exalted Angel); protecting Cloud (Champion's Helm, Swiftfoot Boots, Giver of Runes, Grand Abolisher, Plaza of
  Heroes, Bastion Protector, the protection instants still in the list). Drop mentions of cut cards.
- **Bracket:** Bracket 3, two Game Changers: Smothering Tithe and Enlightened Tutor.
- **Where the cards come from** (a source table like Kefka's, grouped by color/type): From Sauron: Sword of Feast and
  Famine, Sword of Hearth and Home. From Seph: Smothering Tithe. From Sheoldred: Champion's Helm. From Kefka: Swiftfoot
  Boots. From the retired Jodah deck: Arcane Signet, Fellwar Stone, Coalition Relic, Solemn Simulacrum, Enlightened
  Tutor, Grand Abolisher, Skyclave Apparition, Court of Ardenvale, Plaza of Heroes, Sword/Shield/Helm of Kaldra. From
  the binder: Exalted Angel, Disenchant. JD's own (not on the lists): Giver of Runes, 28 Plains. Buy: Cloud, Midgar
  Mercenary ($22.33), Knight of the White Orchid ($3.90), Ardenn ($2.86), Sol Ring ($1.42), Auriok Steelshaper
  ($0.83), Loyal Warhound ($0.64), Armored Skyhunter ($0.44), Balan ($0.40), Mind Stone ($0.20), Kor Outfitter
  ($0.19), Kor Duelist ($0.05): 11 cards, about $33.26 (Scryfall, cheapest printing, 2026-10-13). Everything else:
  bulk.
- Note: Sauron's own Fellwar Stone (added 10-13) is a buy on Sauron's list; Cloud's comes from Jodah.

## 2. decklists/JD/sauron-grixis-amass.md
Out: Sword of Feast and Famine, Sword of Hearth and Home (to Cloud). In: Chaos Warp, Spell Pierce (back from bulk;
both had been cut on 10-13). Paired test, look-ahead, 500 games per tier: -1.7 over four tiers (t1 +1.6, t2 -3.4, t3
-1.2, t5 -3.6; t4 was blocked by a Conspicuous Snoop crash, since fixed); `audit/cloud/2026-10-13/screen/D1-*`. JD's
choice. Consequences to fix in the prose: **the Sword of Feast and Famine + Aggravated Assault combo is gone** (the
combo section, Rule 0 disclosure and plan milestone text), the Army no longer carries combat Swords, Chaos Warp and
Spell Pierce are back as answers. Game Changers unchanged (2).

## 3. decklists/JD/sephiroth-phyrexian-reanimator.md
Out: Smothering Tithe (to Cloud). In: Momentary Blink (bulk: a second Ephemerate-style blink for the Kitten). Paired
test, 500 games per tier: -1.8 (t1 -4.0, t2 -1.4, t3 +3.8, t4 -3.8, t5 -3.8); `audit/cloud/2026-10-13/screen/D2-*`.
JD's choice. Game Changers 8 -> 7 (still Bracket 4); fix every mention of Tithe.

## 4. decklists/JD/sheoldred-mono-black.md
Out: Champion's Helm (to Cloud). In: Carnage Altar (bulk; it was in the deck until 10-11). Paired test, 500 games per
tier: -0.6 (t1 -2.2, t2 -1.4, t3 +3.0, t4 -1.6, t5 -0.6); `audit/cloud/2026-10-13/screen/D3-*`. JD's choice. Fix the
protection text (Swiftfoot Boots and Darksteel Plate are its protection equipment now) and the source table.

## Done when
All four docs read correctly for their current lists (grep each doc for the cut cards: they may appear only in change
logs and history), committed. Reply in at most 10 lines with the commit hash and anything you were unsure about.
