# Task: white-rest — face-down creatures and the Cloud deck's other white cards

Read `rust/agents/LASTDECK-RULES.md` first. Deck: `decklists/JD/cloud-midgar-equipment.md` (key `cloud`), mono-white
Equipment around Cloud, Midgar Mercenary (a two-mana commander whose enter trigger searches for an Equipment). Your
module: `impls/bulk_white.rs` (stub in place), tests `tests/cards_bulk_white.rs`. You own: face-down creatures (morph,
disguise, turning face up), protection from a color, Auras on your own creatures. The cloud-core agent writes the
deck's AI plan; you make the cards work and give each a sensible default use.

## Cards to model (until `cardgen audit --mine` shows each Full)
- **Face down:** find what exists first (grep `face_down` and `morph`: `state.rs`, `engine/zones.rs`,
  `impls/green_cards.rs`, `impls/galadriel.rs`). Exalted Angel (morph {2}{W}{W}; gain life equal to damage dealt);
  Forum Familiar (disguise; face up: return another permanent you control, +1/+1 counter); Unyielding Gatekeeper
  (disguise; face up: exile another nonland permanent: yours comes back tapped, an opponent's gives them a 2/2
  Detective); Exit Specialist (Elrond deck: disguise; face up: return another creature; can't be blocked by power 3 or
  more). Disguise is a face-down 2/2 with ward {2} for {3}. Default AI: cast face down when that is the only way to
  cast it this turn or the face-up ability is worth waiting for; turn face up when the ability has a good target.
- **Enter again:** Ennis, Debate Moderator (exile another creature you control, back at the next end step; a counter
  at your end step if cards were exiled this turn); Stormfront Riders (return two creatures you control; a 1/1 Soldier
  each time a creature returns to your hand from the battlefield). For the cloud deck these target Cloud (his enter
  trigger searches again): expose a target-choice hook the cloud-core agent's plan can use, and note it in your
  handoff file.
- **Recursion:** Call a Surprise Witness (mana value 3 or less, a flying counter, a Spirit), Recommission (artifact or
  creature, mana value 3 or less; an extra counter on a creature).
- **Auras and protection:** Cho-Manno's Blessing (flash; protection from a chosen color, in response to removal of
  that color); Shardmage's Rescue (flash; hexproof the turn it enters, +1/+1).
- **Others:** Bronze Guardian (double strike, ward, other artifacts have ward {2}, power = your artifacts); Fate of
  the Sun-Cryst (costs {2} less on a tapped creature); Harsh Annotation (the Inkling for the controller); Lay Down
  Arms (mana value at most your Plains); Patriot, Shield Wielder ({2},{T}: +2/+0 and hexproof); Recruitment Officer
  (the {3}{W} look at four); Repel the Darkness (tap up to two, draw); Ultimate Magic: Holy (foretell; the
  damage-prevention rider when cast from exile; replace the Approximate verdict).

## Done when
The LASTDECK-RULES checks all pass, and every card above is Full. Report as those rules say; full report in
`rust/agents/report-white-rest.md`.
