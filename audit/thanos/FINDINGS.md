# Thanos, the Mad Titan (key `thanos`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/cali_commander/thanos-artifact-combo.md`, built 2026-11-13 for Cali Commander (1v1, 30 life) from
the previous Thanos list, the Aragorn deck and cards taken from the other decks. Routes: Basalt Monolith + Rings of
Brighthearth, Staff of Domination + Faeburrow Elder, Tinker into Blightsteel Colossus (Sneak Attack as a second way to
put it in), and Thassa's Oracle with Demonic Consultation or Tainted Pact (Underworld Breach with Brain Freeze as a storm
line). **No sims yet.** The findings of the artifacts-and-Dragons list are in `FINDINGS-artifacts-and-dragons-list.md`,
those of the cEDH list in `FINDINGS-old-list.md`.

## What the simulator cannot do yet

- **No Cali mode:** the engine plays 4-player pods at 40 life. Cali is 1v1 at 30 life, and the player on the play skips
  their first draw.
- **No Cali ban list:** `cardgen deck` rejects Mana Crypt, Tinker and Primeval Titan as banned in Commander; Cali allows
  them. This list was written by hand and does not pass `cardgen deck` until that is changed.
- **Cards not modelled:** Tinker, Sneak Attack, Mana Crypt, Staff of Domination and the Oracle line (Thassa's Oracle
  with Demonic Consultation or Tainted Pact) need card code or catalogue combos; Haunted Ridge and An Offer You Can't
  Refuse are new.
- **Plan file:** `data/plans/thanos.toml` still describes the previous list and is rewritten with the engine work.

## Open questions

- Staff of Domination, Watery Grave and Woodland Cemetery are binder-only copies; if any is not in the binder it is a
  proxy.
- Whether proxies are allowed at the Cali events JD plays.
