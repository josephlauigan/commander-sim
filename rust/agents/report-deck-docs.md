# Report: deck-docs

Branch `deck-docs` (from origin/main 51d4d31a). No list section was touched; no simulation was run.

## Per document

- **Kefka** (`decklists/JD/kefka-grixis-discard.md`). Added Palantir of Orthanc (in since 11/01) under "Making them lose
  life", in the card-draw line and in the short-library weak spot (its mill is your own library). Kept Seething Song and
  Blazing Firesinger: the list has the MDFC "Blazing Firesinger // Seething Song", so both names are current. Urborg and
  the 36-land counts were already right. "Murder" was a false hit (Murderous Mimic).
- **Karn**. No edit. Austere Command, Back to Basics, Blood Moon, Collector Ouphe, Vandalblast (and Bane of Progress, Null
  Rod) are opponents' wipes and hate pieces named in "Weak spots", on purpose. Crucible, Strip Mine, Summon: Bahamut and
  the land table already match the list; Game Changers (5) match.
- **Sauron**. Rock count "eleven" -> "ten" (two places; the list has ten). Godo package, Camera, Hammer, Celebrant and the
  infinite-combat statements already match the 11/03 list and Spellbook (Godo + Helm, Celebrant + Helm). Kept Hardened
  Scales (an analogy for Mauhur). "Dread"/"Avenge" were substring hits (Dreadhorde, Avenger).
- **Y'shtola**. Massacre Wurm removed (Urborg line now says Sheoldred, the Apocalypse); "Thirty-five spells trigger her"
  -> thirty-seven (36 noncreature spells of mana value 3+ plus Exsanguinate with X >= 1, counted from the cache);
  "Board wipes (two)" -> three, adding Toxic Deluge; Vampiric Tutor added to the Bond tutors and the Rule 0 tutor line;
  curve (7/7/22/17/7/3), card types (14 creatures + commander, 8 artifacts, 11 instants, 13 sorceries, no planeswalker)
  recounted. Conjurer's Closet already described. Kept "Counterspell(s)" (heading text only, not the card).
- **Cloud**. "Fate of the Sun-Cryst" (out 11/02) replaced by Swords to Plowshares and Path to Exile in "Removal"; "two
  tutors" in Rule 0 -> Equipment tutors (Cloud, Stoneforge Mystic, Steelshaper's Gift, Open the Armory) plus Enlightened
  Tutor; Anduril's attack trigger added to "Doubled triggers". Stoneforge, Steelshaper's Gift, Forge Anew, Judgment of
  Alexander and Galadriel's Dismissal were already described. Bracket 3 / 3 Game Changers is correct. "Tithe/Dismiss" were
  substring hits.
- **Sheoldred**. Mikaeus cost fixed ({2}{B}{B} -> {3}{B}{B}{B}); Triskelion loop spelled out (remove all counters, two pings
  at itself, undying returns it); "no two-card combos" in Rule 0 was wrong (Spellbook: Mikaeus + Triskelion; Mikaeus +
  Yawgmoth + Gray Merchant; Sheoldred or Underworld Dreams + Peer into the Abyss) and the table row now names the combo;
  "average mana value under 3" -> about 3.3 over the nonland cards. Raise Dead was already in "Recursion".
- **Tymna/Thrasios**. No edit. Natural Order is already in Finishers, Tutors, the Game Changer list (7) and Rule 0.
  "Curiosity/Wastes/Tithe" were substring hits (Talisman of Curiosity, Llanowar Wastes, Smothering Tithe).
- **Thanos**. No edit. Every flagged name is an opponent's card or hate piece in the matchup tables (Archon of Emeria,
  Drannith Magistrate, Collector Ouphe, Opposition Agent, Blood Moon, Magus of the Moon, Chalice, Trinisphere, Damping
  Sphere, Defense Grid, Cursed Totem, Pithing Needle, Vexing Bauble, Deafening Silence, Rest in Peace, Dauthi Voidwalker,
  Spellseeker, Endurance, Pyroblast, Basalt Monolith as Kinnan's combo piece, Gaea's Cradle for Rograkh/Thrasios) or
  Scion's retired copy targets in "Why Thanos instead of Scion" (Ancient Copper Dragon). 23 Game Changers match the cache.
- **Ur-Dragon** (now `decklists/JD/ur-dragon-dragon-copies.md`). New `## Combos` section after Key lines (Astral Dragon +
  Cursed Mirror; Miirym + Astral Dragon + Doubling Season; Garth + Twinflame + Spark Double + Doubling Season), each with
  pieces, mana, numbered sequence, result, win, tutors and protection, plus a "Protecting the turn" subsection. Key lines
  now point at the combos and the Astral Dragon target line says Cursed Mirror first. Intro, Copies, Garth paragraph, mana
  section (decline the Mirror's copy), table row and Bracket section no longer say "no intentional infinite combo". Kept
  Disenchant, Terror, Regrowth, Braingeyser, Shivan Dragon and Black Lotus (Garth's copy names) and Helm of the Host /
  Godo (the Godo package moved to Sauron; the bracket text says so).
- **Alela, Galadriel** (Avery). No edit. "Avenge" is a hit on Avenger en-Dal, which is in the list; Mountain/Swamp are
  Farseek's land types.

## Checked and uncertain points (Ur-Dragon combos)

- Sequences were derived from the Oracle text in `data/scryfall_cache.json` and checked against the Spellbook variants
  703-5291, 380-703-4772 and 1171-1235-3908-4772 (steps fetched from the Spellbook API on 11/04).
- Choices called out in the text: the Cursed Mirror "may become a copy" (choose Astral Dragon); the order of the two
  triggers in the Miirym line (Astral Dragon's first gives 32 Dragons); which Twinflame target to use (the Spark Double
  copy, not the legendary Garth); the Garth line relies on each token being a new object with no names chosen yet.
- Garth line mana: Spellbook lists {4}{R}{R}{R} (7); my count of the cheapest line from Oracle costs is 6 (three red).
  The text gives both and says plan on seven.

## Left for the coordinator

- `rust/crates/sim-core/src/human/urdragon.rs` line 1 (a doc comment) and `data/plans/urdragon.toml` still name the old
  file; both are in the other agent's area, so untouched. History records (CHANGELOG, audit FINDINGS, rust/agents) keep
  the old name on purpose.

## Guard

- `cargo build --release -p cardgen` and `./target/release/cardgen export --offline` (under the 12G cap): 3185 cards, 47
  decks. `git diff --stat data/` shows only `data/my_decks.toml` (the one-line `file` path); `data/decks.json` and
  `data/cards.json` are unchanged, so no list moved.
- `cargo test --release --no-fail-fast` (under the cap): exit 0, 252 test binaries ok, 0 failed.
