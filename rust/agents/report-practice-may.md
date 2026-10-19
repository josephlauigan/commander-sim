# Report: practice-may ("you may" triggers ask the person)

## Engine (shared file, additive only)
`engine/stack.rs`: `may_window(g, p, src, name, imp)` (default question "<card>: <name>?"), `may_window_ask(.., prompt)` and
`may_window_card(g, p, cd, name, prompt)` (source is a card, e.g. Jocasta in the graveyard). They run `trigger_window` first
(priority, responses, counters, copies) and, if it resolves and `p` is a person, ask `choices::yes_no`; no means false.
For the AI they are exactly `trigger_window`. The question comes at resolution, which is when Oracle has you choose.
Also two one-word call changes in shared files: `engine/cast.rs` (Rhystic Study's owner) and `impls/rules.rs` (Mystic Remora's owner).
Where the code skipped the trigger for a person by an AI value test, the test is now AI-only (`!is_human`).

## Converted (card: Oracle "may" phrase; file)
- Seymour Flux: "you may pay 1 life" (sheoldred.rs; the life/library test is AI-only)
- Moseo, Vein's New Dean: "return up to one target creature card" (sheoldred.rs; yes/no only, the AI still picks the card)
- Karmic Justice: "you may destroy target permanent that opponent controls" (alela.rs)
- Airlift Chaplain: "You may put a Plains card or a creature card with MV 3 or less ... into your hand. If you don't, +1/+1 counter" (alela.rs; the person picks the card or declines)
- Conjurer's Closet: "you may exile target creature you control, then return it" (t2.rs; person picks any creature or none)
- Teleportation Circle: "exile up to one target artifact or creature you control" (t2.rs; same, creatures and artifacts)
- Soulherder (same function, "exile up to one"), Flickering Hound: "exile up to one other target creature you control" (t2.rs; any creature or none)
- Scarlet Spider, Kaine: "you may discard a card. If you do, +1/+1 counter" (kefka.rs; person picks the discard)
- Stadium Tidalmage: "you may draw a card. If you do, discard a card" (kefka.rs; person picks the discard)
- Jocasta, Automaton Avenger: "you may return it to the battlefield tapped and attacking" (cloud.rs)
- Puresteel Paladin: "you may draw a card"; Magitek Scythe: "you may attach it"; Hero's Blade: "you may attach it" (AI "better holder" test AI-only);
  Armory Automaton, Beatrix, Kor Outfitter: "you may attach ..." (cloud.rs; yes/no only, the AI still picks the Equipment and creature)
- Armored Skyhunter: "You may put an Aura or Equipment card ... onto the battlefield" (bulk_white.rs; yes/no)
- Silverblade Paladin: "You may pair this creature with another unpaired creature" (bulk_white.rs; the score test is AI-only)
- Curiosity: "you may draw a card" (yshtola.rs, both the combat and the non-combat path; the library>3 test is AI-only)
- Rhystic Study (owner): "you may draw a card unless that player pays {1}" (engine/cast.rs); Mystic Remora (owner): "you may draw a card unless ... {4}" (rules.rs)
- Cactarantula, Tenured Concocter: "you may draw a card" (green_cards.rs, bites.rs)
- Gix, Yawgmoth Praetor: "its controller may pay 1 life. If they do, they draw a card" (t3.rs; AI life/library test is AI-only)
- Cursed Mirror: "you may have it become a copy of any creature" (t0_stella.rs; the person picks the creature or declines)
- Mana Vault: "you may pay {4}. If you do, untap" (fixes.rs; pays via `pay_tax`: pool, then sources)
- Ral and the Implicit Maze ch. II: "You may discard a card. If you do, exile the top two cards ..." (sweepers.rs)
- Pawn of Ulamog: "you may create a 0/1 Eldrazi Spawn" (common.rs)
- Ragavan, Nimble Pilferer: "Until end of turn, you may cast that card" (t4.rs; yes/no before paying; only permanents, as before)
- Knight of the White Orchid: "you may search your library for a Plains card" (rules2.rs)
- Mystic Sanctuary: "you may put target instant or sorcery card ... on top of your library" (lands.rs; person picks the card or declines)

## Checked, not converted
- Not "may" in Oracle: Rooftop Assassin, Soul Enervation, Phyrexian Delver, Jackdaw Savior, The Wasp, Plan for All Outcomes, Helm of the Host,
  Gonti, Thief of Sanity (mandatory with a target or a mandatory pick: the per-card target/choice task).
- Already ask: Consecrated Sphinx, Mentor of the Meek, Tymna the Weaver, Smothering Tithe (payer), Sauron's amass is mandatory, shock lands.
- Compiled (DSL) cards: Solemn Simulacrum, Cartographer, Sun Titan, Reya Dawnbringer, Farhaven Elf, Recruiter of the Guard, Zur's search, Planar Genesis,
  Restoration Angel and others have no "optional" flag in `data/cards.json` (the DSL only reads `optional` for `draw`, and no data sets it), so
  they cannot be converted from card code. Needs cardgen to set `optional` from Oracle plus a person-only yes/no in `dsl/mod.rs`; setting it on
  draws would change the AI (`library <= 12` skip), so it needs a person-only guard. Not done here.
- Inner choices left to the per-card task: which Equipment / creature for the attach cards, Moseo's card, Boughside Wanderers' card.
