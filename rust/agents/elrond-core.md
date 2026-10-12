# Task: elrond-core — Elrond and the Ring, the Elrond deck's creatures, and its AI plan

Read `rust/agents/LASTDECK-RULES.md` first. Deck: `decklists/JD/elrond-ring-tempts.md` (key `elrond`), mono-blue.
Your module: `impls/elrond.rs` (stub in place: `elrond_prio`, `elrond_special`, `elrond_protect` are already wired
into the dispatch), tests `tests/cards_elrond.rs`. You own: the Ring tempting you, stun counters, evolve, warp,
increment, toxic.

**Elrond, Lord of Rivendell** ({2}{U}, 3/3): *Whenever Elrond or another creature you control enters, scry 1. If this
is the second time this ability has resolved this turn, the Ring tempts you.* It triggers on every turn (flash
creatures and tokens on opponents' turns count), once per creature (two tokens entering = two triggers), and the Ring
tempts on the second resolution only (not the third or later). The Ring already exists for the Sauron deck: grep
`tempt` and `ring_bearer` (`state.rs`, `impls/mine.rs`, `engine/turn.rs`, `tests/play_sauron.rs`) and reuse it. The
Ring-bearer choice for elrond: the best attacker that is hard to block (flying, can't-be-blocked, hexproof), not
Gandalf while another creature is available (Gandalf, Friend of the Shire draws a card when the Ring tempts you and you
choose another creature). **Check Gandalf** (the audit says Full): flash, sorceries at instant speed, and the draw on a
tempt; fix and say so if anything is wrong.

## Cards to model (until `cardgen audit --mine` shows each Full)
Elrond, Lord of Rivendell; Cloudfin Raptor (evolve); Defiler of Dreams (draw on casting a blue permanent spell; the
optional 2 life for {U} less); Deluge Virtuoso (stun counter; its opus clause); Rapier Wit (cloud deck, but stun is
yours: tap, a stun counter on your turn, draw); Matterbending Mage (return another creature; can't be blocked on X
spells); Mindseeker Oculus (empower Jace 4: the blue-spells agent owns empower; if it isn't there yet when you get
here, leave this card for last and say so); Mouth of the Storm (ward, opponents' creatures -3/-0 until your next
turn); Orysa, Tide Choreographer (cost reduction with total toughness 10; draw two); Professor Hulk (draw on combat
damage); Rescue, Pepper Potts (return another artifact or creature you control; counter if it was an artifact);
Silver Deputy (basic land or Desert to the top; its tap ability); Soul of the Rapids (hexproof); Starbreach Whale and
Starwinder (warp: cast for the warp cost from hand, exiled at the next end step, castable from exile on a later turn;
Starwinder draws on combat damage); Textbook Tabulator (increment, surveil 2); The Wasp, Winsome Avenger (hexproof to
a Hero; tap a defender's creature when it attacks); Hope-Ender Coatl (devoid; the counter-unless-{1} cast trigger);
Dune Mover (toxic); Fear of Isolation (return a permanent you control as an additional cost).

## AI plan for elrond (in `impls/elrond.rs`, through `elrond_prio`, `elrond_special`, `elrond_protect`)
- Elrond early (turn 3); then aim for two creature entries per turn cycle: hold flash creatures (Divining Duelist,
  Hope-Ender Coatl, Rescue, The Wasp, Gandalf) for an opponent's end step when you already have one entry that turn,
  or as the second entry on your own turn.
- Cards that return your own creatures (Matterbending Mage, Peerless Ropemaster, Rescue, Fear of Isolation, Unsummon,
  Whoosh!) pick your best enter-the-battlefield creature (Ultimecia, Orysa, Mnemonic Wall, Deluge Virtuoso, Coral
  Barrier) when no opponent's creature is a better target.
- Warp creatures: cast for warp when that makes a second entry this turn or the creature can't be cast in full yet.
- Counterspells: hold up a counter when you have spare mana; spend them on wipes, big threats and combo pieces.
- Protect Elrond and the Ring-bearer: counters and bounce in response to removal (`elrond_protect`).

## Done when
The LASTDECK-RULES checks all pass, and every card above is Full. Report as those rules say; full report in
`rust/agents/report-elrond-core.md`.
