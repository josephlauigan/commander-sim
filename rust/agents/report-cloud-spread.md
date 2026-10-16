# Report: cloud-spread
List: Helm and Shield of Kaldra kept. New cuts: Bronze Guardian => Skullclamp (stuck in hand 41% of the games drawn, the worst),
Kor Duelist => Lightning Greaves (20.5% win rate when cast, +6.4, among the lowest of cheap cards). Data: 800-game adaptive
--analyze on the 14-swap list, T1 loose. Files: audit/cloud/wide/{wide.swaps,list.txt} (100 cards).
AI (impls/cloud.rs, gated to key cloud): holder_bonus(g,p,e,m) now per piece: Cloud +20 only for the FIRST trigger piece,
Skullclamp +14 on toughness<=1 bodies and -40 on Cloud/commander, Kemba +10 per piece (no spreading penalty), carriers
(flying +3, double strike +4, protection +2), -3 per piece already worn. equip_utility_bonus matches; wants_move is true
unless pinned. pinned(): a living weapon on its own Germ (no other gear on it) and Kaldra pieces on Kaldra never move.
Movers (Kor Outfitter, Brass Squire, Beatrix, Ardenn) use best_move/best_group_move (place score gain > 2). cloud_pick:
Batterskull at >=5 mana, Skullclamp first when a 1-toughness body exists, Husk/Greaves in order. cloud_prio arms for the
wide cards. Shared-file edits: dsl/mod.rs (holder_bonus call), impls/bulk_white.rs (Ardenn uses best_group_move).
Tests: tests/cards_cloud_spread.rs (8). Full cargo test passes; --all-targets 0 warnings; same-seed identity vs parent
a3354cc for the other 10 JD decks (300 games, seed 7, adaptive, pool all): byte-identical.
Trace (10 games, T1 loose lookahead, wide list): 100 Cloud-side equips/attaches, 45 on Cloud (was 62 of 84), Kemba 2
(was 0), Zack 10, Token 9, Helitrooper 6, Exalted Angel 6, Archon 5, Hero 4, Cid 4, Skyfisher 3. Batterskull/Husk
stayed on Germs except after the Germ died. Weakness: Kemba still rarely wears gear (cast late); win rates not measured.
