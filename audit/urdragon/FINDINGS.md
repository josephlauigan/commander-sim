# The Ur-Dragon (key `urdragon`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/urdragon-five-colour-dragons.md`, built 2026-11-13: an optimised high-bracket-4 Dragons list with
proxies allowed (23 Dragons, ten original duals, ten fetches, five shocks; the Garth One-Eye + Spark Double + Twinflame +
Doubling Season combo). **Tiers:** t5 (`data/my_decks.toml`). **No plan file and no sims yet.** The findings of the
retired Ur-Dragon decks are in `audit/retired/urdragon/`.

## What the simulator cannot do yet

- **Plan file:** there is none; the generic pool policy plays the deck until `data/plans/urdragon.toml` is written.
- **Cards modelled before:** most Dragons were modelled for the retired pool deck (`impls/t45_urdragon.rs`; Klauth and
  Rivaz were Approximate) and the Garth combo for the retired JD deck. Not re-checked for this list.

## Open questions

- Ten original duals are the expensive part of the mana base; the alternative is the other five shocks and triomes.
- Hellkite Tyrant against Utvara Hellkite as the 24th Dragon, depending on how many artifacts the table plays.
- Whether the four tutors (Demonic, Vampiric, Worldly, Enlightened) make the Garth combo too consistent for the table.
