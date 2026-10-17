# Report: T6 spec, Dack Fayden White Stax (2026-10-17)

Spec: audit/t6/dack-fayden-white-stax/SPEC.md. Not committed.

Sources opened: Draftsim (Oct 5 2026), EDHREC article (Oct 13 2026), MTG Rocks (Oct 5 2026), EDHREC cEDH stax and combo pages,
EDHTop16 list page (list only), cEDH Decklist Database home (no Dack entry). No Reddit thread found. Oracle text from
MTGJSON AtomicCards (Scryfall API returned HTTP 403 for this machine; not retried further).

Tag counts in the file: about 12 [primer] tags and 29 [inference] tags. Primer content covers only the win line, the
creature package, the stax/protection card names and the turn-2 speed claim. Mulligan, turn order, interaction use,
go/wait conditions and scenarios are inference.

Findings:
- Dack (6 mana, {4}{W}{W}) flips creatures equal to opponents (3), donates them; we keep the triggers. Radiant Solar ventures
  into Lost Mine, Dark Pool drains. Win is a trigger loop, not a spell.
- Primer package has Flickerwisp; our list does not (it has Werefox Bodyguard and Abdel Adrian instead). Restoration Angel,
  Bronzebeak Foragers, Boonweaver Giant, Rule of Law, Mirage Mirror are not in our list.
- Library has 7 creatures and no creature tutor, so the flip is random: Felidar in the first three is 3/7, all of
  Felidar+Preston+Solar is 1/35 unless creatures are drawn first.
- My own derivation (inference): Preston + Felidar + Solar on board + Cloudshift/Ephemerate loops. Needs a rules trace.
- Godo answers: Vexing Bauble and Void Mirror on its free mana, Trinisphere, Karn, Pithing Needle/Flute on Helm, Mandate of Peace.

Lead must decide: how closely the sim must model the Dack flip (random creature draw, donation, goad); whether to wait for a
real primer; whether the Line A adaptation without Flickerwisp is trusted before the combo author traces it.
