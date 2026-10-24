# Task: cloud-kaldra — apply JD's four swaps to Cloud's list

Read `rust/agents/T0-RULES.md` (Hard rules, Checks before you finish items 1-2). Branch cloud-kaldra. `-j 6` (another
session is using CPU). No sims. Be quick.

Swaps (JD, 10-16; test audit/cloud/2026-10-16/kaldra/KALDRA-t1.txt: T1 29.1 -> 30.3, +1.2 ±3.8, neutral):
- Helm of Kaldra -> Archangel of Tithes (JD owns it)
- Shield of Kaldra -> Open the Armory (JD owns it)
- Ghostly Prison -> Sphere of Safety (buy, $4.38; JD couldn't find Ghostly Prison)
- Duelist's Heritage -> Silverblade Paladin (buy, $0.87; JD couldn't find Duelist's Heritage)
Sword of Kaldra stays.

1. `cardgen deck cloud <new import list> --log "<text>"` (log: the four swaps, why — JD is taking Helm and Shield of
   Kaldra for another deck; Ghostly Prison / Duelist's Heritage not found — the test numbers, buys $5.25). Check the
   CHANGELOG line names the four Out and four In cards.
2. Fix every other mention in `decklists/JD/cloud-midgar-equipment.md` (Kaldra token / three-piece text, Ghostly Prison,
   Duelist's Heritage, counts in Decklist by type) and describe the four new cards accurately from Oracle text
   (Silverblade Paladin pairs with Cloud: both get double strike; Sphere of Safety: tax = number of enchantments you
   control; Open the Armory: search for an Aura or Equipment; Archangel of Tithes: attack/block taxes).
3. `binder-cards.txt`: add Helm of Kaldra and Shield of Kaldra (JD keeps them for a new deck; follow the file format).
   Nothing to bulk (Ghostly Prison and Duelist's Heritage were never bought).
4. `audit/cloud/FINDINGS.md`: one row in "What was tried" for this change (+1.2, neutral; Open the Armory cast 87%,
   Silverblade 66%, Sphere 58% (stuck 42%), Helm/Shield of Kaldra were stuck 22%/44%) and update "Current state".
5. `cardgen export`, `cardgen validate`, full `cargo test --release -j 6 --no-fail-fast`, `--all-targets` no warnings.
   Commit (no Co-Authored-By), never push. Report ≤8 lines: line 1 DONE/HANDOFF, line 2 model ID.
