# Aragorn, the Uniter (key `aragorn`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/aragorn-uniter.md`, built 2026-11-06 from the Kefka and Sauron decks (both retired the same
day; their findings are in `audit/retired/kefka/` and `audit/retired/sauron/`). **Tiers:** t4-t5
(`data/my_decks.toml`). **Plan:** `data/plans/aragorn.toml` (PROVISIONAL: written from the deck document's game plan,
not from a paired test). **Model:** 100 cards, Full 94, Full-auto 6; Aragorn's four cast triggers are card code
(`impls/aragorn.rs`, `tests/cards_aragorn.rs`).

## Baseline 2026-11-06 (first run; look-ahead, loose, 2,000 games per tier, seed 500000, build d0672b24)

t4 **10.7%** (9.4-12.1), t5 **7.7%** (6.6-9.0): **9.2% over t4, t5** (even share 25%). Raw:
`audit/aragorn/baseline-2026-11-06/aragorn.txt`.

For scale, not a paired comparison (different tiers and lists): Sauron was 26.1% over t3-t4 and Kefka 17.1% over t3 in
the 2026-11-06 baselines. Aragorn plays one tier higher than either.

Why it is low is not measured yet: this run gives the win rate only. The next step is an `--analyze` run on t4 and t5
(how it wins and dies, when Aragorn lands, how often the Breach and Fiery Emancipation lines come online, what is
stuck in hand) before any swap.
