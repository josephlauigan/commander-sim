# Aragorn, the Uniter (key `aragorn`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/aragorn-uniter.md`, built 2026-11-06 from the Kefka and Sauron decks (both retired the same
day; their findings are in `audit/retired/kefka/` and `audit/retired/sauron/`). **Tiers:** t4-t5
(`data/my_decks.toml`). **Plan:** `data/plans/aragorn.toml` (PROVISIONAL: written from the deck document's game plan,
not from a paired test). **Model:** 100 cards, Full 94, Full-auto 6; Aragorn's four cast triggers are card code
(`impls/aragorn.rs`, `tests/cards_aragorn.rs`).

## 2026-11-06: why it loses (goldfish and 12 traces; diagnosis, no list change)

Raw: `audit/aragorn/2026-11-06-diag/` (`goldfish.txt`: 2,000 games each against three inert seats; `trace-t4-0..5`,
`trace-t5-0..5`: seeds 500000-500005). For scale, JD's other decks at these tiers: Sheoldred 26.6% and the retired
Sauron 24.2% at t4, Y'shtola 26.5% and Thanos 18.6% at t5.

**Too slow even unopposed.** Goldfish median kill: own turn **12** (Sheoldred 9, Thanos 10, Y'shtola 11). The t4 and
t5 tables kill it around round 8.5, so it loses the race before interaction matters unless it slows them down.
Mana is short in 20% of its own turns (31% of turns 1-4); in the t4-2 trace it sat on 3-4 lands from turn 4 to 7.

**The AI holds burn instead of using Aragorn.** In the goldfish, Lightning Bolt was payable but never offered in 2,771
card-turns, against three idle opponents, where each Bolt with Aragorn out is 6 damage. Nothing in the generic AI
adds Aragorn's trigger value to a spell's worth (the spell-payoff count covers Guttersnipe-style tags, not Aragorn).
The look-ahead also chose "stop (hold mana)" about three times a game over Skullclamp, Expressive Iteration, Lotus Petal
and the rocks.

**t5: combos at full life, often with Aragorn tapped out.** Four of the six t5 traces end in a combo in rounds 3-6
with Aragorn at 31-40 life: Kinnan on round 3 (nothing to be done), Yawgmoth on round 4, Zur on round 5 (Aragorn had
spent its mana equipping Lightning Greaves that turn), Urza on round 6 (Aragorn had used Mystic Confluence to bounce a
one-drop, Esper Sentinel, and had two cards left). The counters are used on the wrong things or not held.

**t4: Krenko overruns it.** Krenko won three of the six t4 traces, by 16-27 points past zero (44 creatures attacking
in t4-2). The deck has no creature sweeper (Cyclonic Rift only); in t4-2 it held mana up on turn 6 and spent Mana
Drain on a Goblin Ringleader while the goblins grew. Aragorn himself was removed in three of the six games.

**Expectation.** Below the other decks by about half, and part of that is the simulator: the AI does not play
Aragorn's engine (burn to the face, chaining cheap spells) and spends or holds its interaction badly. Part is the
list: slow, few finishers (single copies), no sweepers against t4's creature decks, fragile mana in four colours.

## 2026-11-06: the Breach line restored, Aura Shards cast (engine and plan fix; t4, t5, look-ahead, loose, 2,000 games per tier, seed 500000, build in commit.txt)

What changed: the Underworld Breach line (`impls/breach_line.rs`) is back for every deck holding Breach and a storm card
(it had been deleted with Kefka's and Sauron's hand AI on 11/01), and the plan file casts Aura Shards. Raw:
`audit/aragorn/2026-11-06-breach-line/analyze-t4.txt`, `analyze-t5.txt`.

t4 **11.1%** (before 10.7%), t5 **8.6%** (before 7.7%): **9.9% over t4, t5** (before 9.2%). Both moves are inside the
noise (intervals about +-1.3 points); same seeds, so the direction is real but small.

**Why so little.** The line works when it comes up: every Breach line the deck executed won (dead or decked table),
1.4% of games at t4 and 0.8% at t5. It rarely comes up: Breach and a storm card are both held on the deck's turn in
only 6.5% (t4) and 4.6% (t5) of games, and the dry run passes in about a quarter of those (falling short mostly on
mana, then graveyard fuel). Brain Freeze and Grapeshot are now cast in 3-6% of the games they are drawn (before about
1%): one Breach and two storm cards in 100 is a thin line. Aura Shards is no longer stuck in hand.

The rest of the picture is unchanged: Aragorn resolves in 90% / 85% of games (median turn 4); his red damage comes
online in 59% / 54%; the deck dies to t4's go-wide decks (Krenko, Edgar Markov, Yuriko) and t5's combos (Kinnan, Urza,
Zur, Yawgmoth: about 52%).

**Next.** The win rate is now a fair read of the list. The swaps to test: more ways to find or rebuild the Breach line
(tutors that find Breach or a storm card, more self-mill or fuel), more early interaction against t4's creature
decks, and more stack interaction or graveyard-free answers to t5's combos.

**Other decks.** The line now also plays for the pool's Breach decks (kraum-tymna-blue-breach-midrange,
rograkh-silas-turbo-breach), so Tier 5 and Tier 6 tables are a little stronger than in the 11/06 baselines.

## Baseline 2026-11-06 (first run; look-ahead, loose, 2,000 games per tier, seed 500000, build d0672b24)

t4 **10.7%** (9.4-12.1), t5 **7.7%** (6.6-9.0): **9.2% over t4, t5** (even share 25%). Raw:
`audit/aragorn/baseline-2026-11-06/aragorn.txt`.

For scale, not a paired comparison (different tiers and lists): Sauron was 26.1% over t3-t4 and Kefka 17.1% over t3 in
the 2026-11-06 baselines. Aragorn plays one tier higher than either.

## Analysis 2026-11-06 (`--analyze`, t4 and t5, 2,000 games each, seed 500000, build 5c4a78e1)

Raw: `audit/aragorn/baseline-2026-11-06/analyze-t4.txt`, `analyze-t5.txt`. Card numbers are correlation, not cause.

**How it loses.** Eliminated in 89% (t4) and 92% (t5) of games, around round 8.5.
- t4: mostly combat from go-wide decks: Krenko (combat 13.8%, triggers 8.1%), Edgar Markov (combat 11.1%), Yuriko
  (triggers 10.8%, combat 5.7%); Chulane's combo 10.2%.
- t5: mostly combos: Kinnan 15.8%, Urza 15.8%, Zur 10.5%, Yawgmoth 9.9% (about 52% together); Miirym combat 13.9%.
  The deck has counterspells (Counterspell, Mana Drain, Swan Song, Fierce Guardianship, Force of Will) but Mana Drain and
  Swan Song are among its lowest win rates when cast at t5.

**How it wins.** 99% by damage, but slowly: 19.7 (t4) and 19.8 (t5) damage dealt per game, half or more from combat;
0.40 and 0.29 opponents eliminated per game.

**The commander resolves but the engine rarely runs.** Aragorn resolves in 90% (t4) and 85% (t5) of games, median turn
4. Its red-spell damage comes online in only 59% and 54% of games (25% / 23% by turn 5). Aragorn is the card
opponents remove most (0.37 per game at t4), and its win rate when cast is about the deck's average (+0.9 / +1.2).

**The storm finish is not played.** Brain Freeze and Grapeshot are cast in about 1% of the games they are drawn. The
plan file's Breach line only points the tutors at the pieces; nothing in the generic policy plays a Breach storm turn
(the old Kefka hand AI's `breach_hold_prio` and Breach gate were never in the plan schema). The report's "Breach line"
counters read 0.0% because no code sets them any more: they say nothing either way.

**Never cast.** Aura Shards: 0.3% (t4) and 0.0% (t5) of the games it was drawn, reported "untagged": the AI has no
reason to cast it.

**What wins when it happens** (high win rate when cast): Niv-Mizzet, Parun (+32 / +26), Fiery Emancipation (+27 / +23),
Comet Storm, Triple Triad, Rhystic Study, Jeska's Will, Fact or Fiction. These are the big-mana payoffs and card
engines; Triple Triad (27%), Comet Storm (32%) and Niv-Mizzet (36%) are also often stuck in hand.

**Next.** Two AI gaps to fix before swap tests mean anything: a Breach storm turn for this deck (Brain Freeze /
Grapeshot are dead cards today), and a reason to cast Aura Shards. Then the swaps: more early interaction against t4's
go-wide decks and t5's combos.
