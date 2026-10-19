# Rograkh & Silas Renn (Turbo Oracle and Breach) — Behaviour Spec (Tier 6)

Sources (all opened 2026-10-17; Moxfield and TappedOut returned HTTP 403, so no primer text from them). Pages were read
through a summarising fetch tool, so quotes are paraphrased summaries, not verbatim:

- **S1** Learn cEDH, "Rograkh Silas" decklist primer, https://learncedh.com/decklists/rograkh-silas (no date shown). The only real primer: win lines, mulligan order, weaknesses.
- **S2** EDHREC, "Building RogSi (Rograkh and Silas Renn) for cEDH", Harvey McGuinness, 2026-03-11, https://edhrec.com/articles/building-rogsi-rograkh-and-silas-renn-for-cedh
- **S3** EDHREC, "A cEDH Guide to Rograkh, Son of Rohgahh", Harvey McGuinness, 2026-06-30, https://cloudflare.edhrec.com/articles/a-cedh-guide-to-rograkh-son-of-rohgahh
- **S4** EDHREC, "EDHREC Guide to cEDH Turbo", Harvey McGuinness (no date shown), https://edhrec.com/articles/edhrec-guide-to-cedh-turbo
- **S5** Commander's Herald, "An Introduction to RogSi in cEDH", Drake Sasser, 2023-11-23, https://commandersherald.com/an-introduction-to-rogsi-in-cedh/
- **S6** cEDH Decklist Database entry "Rograkh Silas Turbo Naus" (updated 2023-03-20), https://cedh-decklist-database.com (the entry has a primer icon but the page gives no primer link; it links three Moxfield lists, all 403).
- **S7** EDHTop16 page for our exact list (Jacob Rhyne, CCS Invitational Qualifier #2, 2026-06-13): list only, no notes.
- **S9** Ondas, "Turbo Ad Nauseam primer", February 2021 (Google Doc, read as a local copy at `primers/kraum-tymna-blue-farm.txt`), https://docs.google.com/document/d/1BPpPnnfPSpnpeV_PYATqxTtJgYgigK5yhhnSBPqtyvU . General turbo Ad Nauseam primer; names Rograkh/Silas as a Grixis commander option. Tagged [primer: Ondas]. Written for the 2021 pool, so points that rely on Dockside Extortionist, Mana Crypt, Peer into the Abyss, Stitcher's Supplier or Dualcaster/Twinflame are marked "outdated".
- **S10** Natux, "[Primer] cEDH Rog Grixis Turbo" (Rograkh / Silas Renn), Moxfield, changelog to 2026-03-28, https://moxfield.com/decks/yRsS18tYsE-jVgqmK7_Z0w/primer (read as the extract `primers/rograkh-silas-natux.md` of the 43-page PDF JD supplied). **The current, dedicated primer for this deck.** Tagged [primer: Natux]. Where S10 conflicts with S9 or S1-S5, the spec follows S10. Cards S10 names that our list lacks: Necrodominance, Deadly Rollick, Slaughter Pact, Powerbalance, The One Ring, Dauthi Voidwalker, Brainstorm, Ponder, Training Center (and no basic lands).
- **S8** docs.mdbook.me/mtg/cedh/cedh-rograkh-silas: a different list (Isochron Scepter, Aetherflux, Laboratory Maniac). Used only for what a Stifle-type effect does to Oracle. Not our list.

Oracle text checked on Scryfall (2026-10-17) for every spell named below, except Mox Diamond, Chrome Mox and the dual
lands (Scryfall was rate-limiting; their text is not relied on in any test). Spec written 2026-10-17.

## Game plan
**Turbo: win on turn 1 to 3, hold almost nothing back.** [primer S1: "win within the first three turns"; S5: needs a win
attempt in the first couple of turns; S3: aims for turn one.] Rograkh costs {0}, so the deck has a legendary creature on
turn 1 every game. That turns on Mox Amber, Culling the Weak, Infernal Plunge, Diabolic Intent, Flare of Duplication,
Fierce Guardianship and Deflecting Swat [primer S2, S3, S4]. The deck makes a burst of mana from rituals, Moxen, Lion's
Eye Diamond and Birgi, digs with tutors, Ad Nauseam and Necropotence, then wins in one of three ways: Thassa's Oracle
with an emptied library, Underworld Breach with Lion's Eye Diamond and Brain Freeze, or an Ad Nauseam turn that ends in
one of those. Stance: **race**, backed by cheap protection (free counters, Hexing Squelcher). It does not lock.

Both primers: if unopposed it aims to win on turn 2, sometimes turn 1, and mulligans toward hands that do that
[primer: Natux]. But it does not go blindly: "finding an opportune moment to win is more important than raw speed", and
it may wait a turn when it expects interaction [primer: Natux]. Rograkh gives no card advantage, so when the table keeps
mana up it waits for them to tap out or for someone else to try to win first [primer: Ondas, Natux]. It is weak in long
games and 3-versus-1 grinds [primer: Natux]. Silas is "mostly there for the colours" and is usually never cast
[primer: Natux]. Rograkh also turns on Phyrexian Tower and Jeska's Will, which the earlier list of enablers missed
[primer: Natux].

## Win lines
Mana is counted from an empty board and does not include the commander (Rograkh costs 0).

| Line | Pieces (in our list) | Mana (total, colours) | Steps | Where it can be answered |
|---|---|---|---|---|
| **A. Tainted Fish** [primer S1, S2] | Thassa's Oracle; Demonic Consultation or Tainted Pact | Consultation: 3 = UU + B. Pact: 4 = UU + 1B | 1 Cast Oracle. 2 With its enter trigger on the stack, cast Consultation (name a card that is not in the library, e.g. one in hand or graveyard) or Pact (decline every card; the deck is singleton, so it exiles the whole library). 3 Trigger resolves with X (devotion to blue, 2 from Oracle) >= library size: win. Pact does not have to exile everything: Natux often stops at one of the bottom cards to keep a protection spell, which is weaker to removal and Noxious Revival [primer: Natux]; Oracle's own devotion of 2 allows a library of up to 2 cards [inference from Oracle text]. | Oracle spell (step 1): any counterspell, Pyroblast/REB (Oracle is blue). Consultation/Pact (step 2): counterspell. Trigger (step 3): only a trigger counter (Stifle effect) or removing Oracle does not stop it. Library not empty if the named card is in the library. Oracle cannot be removed once cast, and if the other half is countered it still acts as a scry [primer: Ondas]. Consultation/Pact are instants, so Dispel and Flusterstorm hit them [inference]. Opposing draw effects in response (Cephalid Coliseum, Kenrith) beat an emptied library [primer: Ondas]. |
| **B. Breach loop** [primer S1, S2, S5; preferred line: primer: Natux, Ondas] | Underworld Breach; Lion's Eye Diamond; Brain Freeze; Thassa's Oracle | 2 from outside (R + 1, for Breach); everything after is paid by LED (three of one colour per sacrifice, U for Brain Freeze and Oracle) | 1 Resolve Breach ({1}{R}) and cast LED ({0}). 2 Sacrifice LED: discard hand (Brain Freeze, Oracle and other cards go to the graveyard), add UUU. 3 Escape Brain Freeze ({1}{U} + exile 3 other graveyard cards), target yourself: storm copies = spells cast this turn, each mills 3. 4 Escape LED ({0} + exile 3), sacrifice it for UUU, escape Brain Freeze again; repeat until the library is empty. 5 Escape Oracle ({U}{U} + exile 3) with an empty library: win. If Oracle is exiled or countered: aim Brain Freeze at an opponent instead. | Breach (step 1): counterspell on a noncreature spell (Force of Will, Fierce Guardianship, An Offer, Pyroblast if blue), or enchantment removal in response to the first escape. Graveyard exile (Rest in Peace, Tormod's Crypt effects, Bojuka Bog) any time. Brain Freeze on the stack (step 3/4): counterspell, but the loop has many copies. Null Rod / Collector Ouphe: stop LED and the Moxen [primer S1]. Dispel, Flusterstorm and Miscast cannot hit Breach; Boseiju and Otawara can [primer: Ondas]. Natux: roughly 8 cards across graveyard and hand usually guarantee the win; Lotus Petal can stand in for LED; An Offer You Can't Refuse can counter the pilot's own Brain Freeze copies if LED and Petal are gone [primer: Natux]. Vexing Bauble on the board counters {0} casts (see Open questions). |
| **C. Ad Nauseam turn** [primer S1, S4; Natux: averages 30+ cards, Ondas: 20-30] | Ad Nauseam, plus line A or B pieces | 5 = {3}{B}{B}, plus rituals before it | 1 Pile fast mana (Dark/Cabal Ritual, Culling the Weak, Moxen, Petal). 2 Cast Ad Nauseam in the main phase, or at the end step before the pilot's turn if mana is short or opponents may hold interaction (also the most mana-efficient) [primer: Natux]; stop while life > 0 and a win line is in hand, though usually the pilot just goes for it [primer: Natux]. From a main phase above 25 life with no mana floating it wins about 90% [primer: Ondas]. 3 Finish with line A or B with the mana left. If the library is nearly drawn out, Oracle needs only X >= library size. | Ad Nauseam (step 2): counterspell; Fierce Guardianship / Daze / Force of Will in response. Life loss (step 2): the player's own risk. Then as line A or B. |
| **D. Necropotence plus extra turn** [primer S1] | Necropotence; Borne Upon a Wind or Valley Floodcaller (flash enablers); Final Fortune or Last Chance (extra turn, you lose at its end step; Final Fortune is useless on a Breach or Yawgmoth's Will turn [primer: Natux]); Consultation/Pact to find an enabler | 1-turn: BBB + 1U (Borne) = 5, + RR for Final Fortune. 2-turn: cast Necro turn 1 with rituals | 1 Cast Necropotence ({B}{B}{B}). 2 Pay life, exile cards. 3 At end step they enter the hand. 4 Flash enabler (Borne Upon a Wind, or Floodcaller); take the extra turn with Final Fortune and win with line A in that turn (you lose at its end step; not line B, per Natux). Natux: the first Necropotence activation takes about 10-13 cards, aiming for Borne or a tutor plus free interaction; next turn pay almost all life and win in the end step [primer: Natux]. Natux notes Necropotence clashes with the Ad Nauseam and wheel payoffs: mostly Oracle plus manual exile plus protection [primer: Natux]. | Necropotence (step 1): counterspell. Enchantment removal while it is on the board. Life total and Bauble/Null Rod-type hate. Extra-turn spell: counterspell. |
| **E. Yawgmoth's Will storm** [primer S5 names only "Brain Freeze can fuel Yawgmoth's Will"] | Yawgmoth's Will ({2}{B}); Brain Freeze; rituals/Moxen in the graveyard; Oracle | 3 + rituals; spells recast from the graveyard | Cast rituals, replay them with Will, then Brain Freeze with a high storm count at yourself, then Oracle. The primer does not give a sequence. | Will: counterspell. Graveyard exile. |

The primer lists backups that need no extra spec: Brain Freeze milling an opponent out under Breach, Mnemonic
Betrayal (cast cards from opponents' graveyards), and Praetor's Grasp (steal an opponent's library card, recur with
Breach) [primer S1]. If the pilot's own win conditions are gone, Betrayal or Grasp uses an opponent's [primer: Natux]; Betrayal is also worth casting early for value [primer: Natux]. Ondas adds that stealing an opponent's Oracle is a real win, and Betrayal with a wheel or Brain Freeze steals the table's win conditions [primer: Ondas]. Ondas's Grinding Station + Culling the Weak + Stitcher's Supplier mill line is out [primer: Ondas, outdated: no Stitcher's Supplier in our list]. Our list does **not** have Laboratory Maniac, Doomsday, Dockside Extortionist, Isochron Scepter,
Dramatic Reversal or Aetherflux Reservoir (the S8 list), so those lines are out of scope.

Tutors, by piece [Oracle text checked]:
- **Oracle:** Demonic Tutor, Vampiric Tutor (top of library, then draw), Imperial Seal (same), Gamble (then random discard), Diabolic Intent (sacrifice a creature), Beseech the Mirror (1BBB; bargain to cast <=4 for free), Demonic Counsel (only with delirium), Wishclaw Talisman (activate on your turn, an opponent gains control), Demonic Consultation naming Oracle.
- **Consultation / Pact:** Mystical Tutor (instants and sorceries; puts on top), the black tutors above.
- **Breach / LED / Brain Freeze:** the same black tutors; Mystical Tutor finds Brain Freeze or Breach? Breach is an enchantment, so no.
- **Ad Nauseam:** the black tutors; Mystical Tutor (instant).
- **Necropotence:** the black tutors; Praetor's Grasp finds only the opponents' cards.

## Mulligan
Natux's rules are the base; they replace the earlier generic order.

- **Keep** a hand with at least one payoff or tutor and enough fast mana to cast that payoff on its recommended turn.
  Safer with two payoffs/tutors, or with cheap protection for the payoff. Two land drops in two turns is nice, not
  required [primer: Natux].
- **Payoffs by recommended turn** [primer: Natux]:

  | Turn | Payoff |
  |---|---|
  | 1 | Mystic Remora |
  | 2 | Wheel of Fortune, Windfall, Necropotence, Rhystic Study (maybe Timetwister) |
  | 3 | Ad Nauseam |

  (Natux also lists Necrodominance and The One Ring; our list has neither.)
- **7 or 6 cards:** two payoffs/tutors, or interaction to protect the payoff, plus mana for the payoff on its turn; or
  one unprotected payoff that is one or two turns earlier than recommended [primer: Natux].
- **5 or 4 cards:** one payoff and mana to cast it on its turn [primer: Natux]. **3 or fewer:** usually do not go this
  deep unless the 4 was unplayable [primer: Natux]. (Ondas goes below 5 more often [primer: Ondas]; his deck is a
  different one, so the spec follows Natux.)
- **Alternative keeps:** Oracle plus a library-exile piece by turn 3; Brain Freeze plus Underworld Breach by turn 3
  [primer: Natux].
- **Ship** all-land hands, "lands and spells" hands with no fast mana and no payoff or tutor, and hands that safely play
  a rock and hold a counter: a deck built to have five mana in two or three turns is rewarded for digging for mana plus
  a payoff [primer: Ondas]. Also ship hands that need Rograkh three ways ("don't double-book your Rog": Culling the Weak,
  Infernal Plunge, Diabolic Intent, Flare of Duplication, Springleaf Drum and Paradise Mantle all want the same creature,
  and Mox Amber and Fierce Guardianship need him to stay in play) [primer S5].
- **Read the pod.** Against a table of stax and when last in turn order, do not keep a turn-2-win hand; keep a grindy
  hand [primer: Ondas]. Turn order and the opposing decks change what is keepable [primer: Natux].
- **Trap hand** (earliest impactful play is turn 3, so keep at 4 in many pods, otherwise ship): good cards but slow
  [primer: Natux]. Hands with protection and a payoff but too little mana are also kept only at 4 [primer: Natux].
- A typical keep [inference, built from the rules above]: one or two lands, two or three zero- or one-mana accelerants
  (Petal, a Mox, Sol Ring, Dark Ritual), one tutor or Ad Nauseam, plus one free counter. Rograkh is always available
  from the command zone.

## Turn by turn
No source gives a turn-by-turn script, only the order of priorities [primer S1]: deploy fast mana first, then tutor
and the win, with protection held.

- **Turns 1–2** [primer S1 for the order, inference for the details]: Rograkh first (free): always cast on turn 1,
  it bluffs free interaction, enables Mantle, and opponents rarely spend removal on it [primer: Natux]. Then lands,
  Moxen, Lotus Petal, Mox Amber (Rograkh makes it a Mox Ruby), Sol Ring/Mana Vault. Carry out the plan the opening hand
  gives as soon as possible: find a Necropotence effect or Ad Nauseam first [primer: Natux]. Remora (turn 1), Rhystic
  Study and the wheels (turn 2) are payoffs in their own right, not fillers [primer: Natux]. A first win attempt can come
  on turn 1 (S3, S1). Jam Ad Nauseam on turn 2 if the opponents tapped out for engines: only Force of Will, Force of
  Negation and Fierce Guardianship stop it reliably, and by turn 3 the counters that can stop it more than triple
  [primer: Ondas]. Jeska's Will on turn 1 if nothing better [primer: Natux].
- **Turns 3–4**: a second attempt or a recovery. Cast Necropotence or Ad Nauseam; cast many spells first and then wheel
  (Wheel of Fortune, Windfall), or wheel very early to disrupt hands [primer: Natux]. A wheel with 4+ mana late in the
  game can turn the new hand into a win attempt, and a wheel is the "hail mary" when the deck was stopped [primer: Ondas].
  Use the wheels (Wheel of Fortune, Windfall, Timetwister) to refill: S2/S5 say wheels refill the hand and give a burst of cards even if the opponents
  also draw, since the game ends before they use them. Lion's Eye Diamond in response to Wheel of Fortune [primer S5].
- **Later** (turn 4+): "not where we want to be". Dig toward a win and find the best moment to go off, potentially after
  someone else tried to go for it; later means less likely to win, but keep digging, as opponents' spells may help
  [primer: Natux]. Since Rograkh gives no card advantage, wait for opponents to tap out and forget the deck, for one to go
  for the win, or fight through with an end-step Ad Nauseam [primer: Ondas]. Grind with Rhystic Study, Mystic Remora, Necropotence, Silas Renn (artifact recursion on combat damage,
  rarely cast [S2]), and wait for a window. S1 says the deck relies on explosive openings and struggles to recover.

## Interaction
Holds (all in our list): Force of Will, Fierce Guardianship (free with a commander in play, noncreature only), Daze,
Flusterstorm, Mental Misstep (MV 1 spells), Pact of Negation, Misdirection, Deflecting Swat (free with a commander),
Mindbreak Trap (free if an opponent cast three spells), An Offer You Can't Refuse, Pyroblast, Red Elemental Blast,
Chain of Vapor, Into the Flood Maw, Snap, Flare of Duplication (copy an instant/sorcery), Defense Grid, Hexing
Squelcher (spells you control can't be countered). Named in the primers as the protection set: Fierce Guardianship,
Deflecting Swat, Flusterstorm [primer S1]; Pyroblast, REB, Daze, Defense Grid [primer S5]. Pyroblast/REB also answer
opposing Rhystic Study and Mystic Remora [primer S5]; the deck has little else beyond the free-spell set [primer S5:
"limited protection for its wins"].

Spending rules [inference unless noted]:
- Free counters (Fierce, Pact, Force, Daze, Misstep) are kept for the turn of the win attempt, not spent on early
  development spells of other players. Pact of Negation is cast only when comboing or to stop another player's win,
  never during setup; Force of Will only in critical scenarios (its cost is high); Mental Misstep protects the combo
  against one-mana interaction [primer: Natux]. Fierce Guardianship is "a solid Negate" that is free with Rograkh out,
  which lets early wins be protected [primer: Natux].
- Deflecting Swat mostly redirects an opposing counterspell (it can point one onto itself); Flare of Duplication copies
  the pilot's own tutors, or an opponent's counterspell as a counter (blowout risk); An Offer You Can't Refuse can also
  counter the pilot's own spell for two Treasures; Chain of Vapor bounces an opposing permanent so they must sacrifice a
  land, or the pilot's own fast mana only when almost sure to win [primer: Natux]. Bounce spells are also the answer to a
  stax piece such as Rule of Law [primer: Ondas].
- Hexing Squelcher stops more opposing interaction than anything else for two mana; Defense Grid is a colourless,
  sorcery-speed Silence on the combo turn that also helps opponents [primer: Natux].
- Fewer than 15 counters per deck, five of them free: a Naus turn often gives enough mana to protect the win
  [primer: Ondas].
- Pyroblast/REB hit blue permanents and spells: use them on Rhystic Study, Mystic Remora, or a blue counterspell aimed
  at the win, not on random blue spells.
- Defense Grid is cast on the pilot's own turn before the attempt, so opponents pay 3 more for responses; it also taxes
  the pilot's own spells on opposing turns [inference from Oracle text: "except during its controller's turn"].
- Hexing Squelcher before the win spell stops counterspells on the whole line.
- The deck lets most opposing spells resolve, because the plan is to win first [primer S4: proactive interaction,
  prevent stack wars].

Who to stop when two players threaten to win: Natux says Pact of Negation is for the pilot's own combo or to stop someone else winning [primer: Natux]; no source ranks the threats. [inference] Stop the player whose win is
closer than ours; when our attempt is ready, ignore the others and protect ours. Use the opponent's combo piece
as the target of a counter, not the enabler.

## When it goes for the win
[primer: Natux] Do not always go at the first possible moment; blind attempts run into interaction. Carry out the
opening hand's plan as soon as possible in most cases, but wait a turn when you expect interaction. Gitaxian Probe is
for the go-or-wait decision; finding the right moment matters more than raw speed. Go after someone else spends
interaction on their own attempt. Cast Ad Nauseam at the end step before your turn when mana is short or interaction is
likely (not so late that a control player untaps). Hexing Squelcher or Defense Grid first on the combo turn. Wishclaw
Talisman only on the win turn, since an opponent gets it. [primer: Ondas] Casting Ad Nauseam into a fed Rhystic Study
with 1-2 open mana is often right.
[primer S4]: "as soon as you have the mana", or when a window opens. [primer S1]: win within three turns. Conditions
[inference unless stated]:
- Go when a full line is in hand (or one tutor away) and mana for it is available, with at least one free
  protection spell, or when all opponents are tapped low.
- Cast Hexing Squelcher first if mana allows, then the win spell.
- Wait if: Vexing Bauble or Null Rod/Collector Ouphe is out and not removable; the graveyard line faces graveyard
  hate; an opponent holds up open blue mana with a known counter and we have no protection; the hand is missing a
  piece with no tutor.
- If the attempt fails (countered Oracle), fall back to Brain Freeze at an opponent or rebuild next turn [primer S2].

## What it fears
| Fear | Source | How it plays around it |
|---|---|---|
| Natux's order of fears, worst first: 1) stax that stops more than one spell per turn (Rule of Law effects) and cost increasers; then 2) no tutoring, 3) no multi-card draw, 4) artifact abilities shut off, 5) graveyard access taken away | primer: Natux | Win before many stax pieces land; other stax hurts less. Rule of Law stops every multi-spell line here (Oracle + Consultation, the Breach loop) [inference]; Ondas's answer is a Tainted Pact at the end step, then Oracle on the pilot's own turn [primer: Ondas; he does not say how the library draw is avoided, see Open questions]. Bounce the piece (Chain of Vapor, Snap, Into the Flood Maw) [primer: Ondas]. Long games and 3-versus-1 grinds are bad for the deck [primer: Natux]. |
| Null Rod, Collector Ouphe (artifact hate) | primer S1; ranked 4th by Natux; Ouphe also named by Ondas | Win with Oracle line A (no artifacts except mana); keep Pyroblast/REB for blue hosers only; tutor around it. |
| Graveyard hate (Rest in Peace and similar) | primer S1 weakness list names "hate pieces" generally; graveyard hate is [inference] since Breach needs the graveyard | Prefer line A or C over line B. |
| Trigger counters on Oracle (Tishana's Tidebinder, Stifle) | primer S8 | Cast Oracle only after the enabler is ready; line B backup (Brain Freeze at the table). |
| Silence, Grand Abolisher effects on the pilot's turn | primer S4 lists these as turbo's own protection; [inference] they hurt us too | Win through them with Hexing Squelcher; they do not stop line A on its own if cast before. |
| Becoming the archenemy; removal on Rograkh | primer S5; Natux: the deck plays few permanents, does not care much about targeted removal, and opponents rarely spend removal on Rograkh | Keep Rograkh's count of jobs low; he can be recast from the command zone for {2} (commander tax). |
| Blood Moon, Back to Basics | primer: Natux (hurt, but usually land too late to matter); no basics in the list | Win before they land. |
| Failure of the first attempt | primer S1, S4 | Wheels and Necropotence rebuild; this is the deck's weak point. |

## Against Godo
Godo wins with Godo (cast, {5}{R}), the enter trigger fetching Helm of the Host, the attach, then token copies each
combat; the other line is Dualcaster Mage plus a sorcery copy spell [Godo note, branch godo-cedh].

What this deck can do, step by step [inference from the Oracle text above; primer sources never discuss Godo]:

| Godo step | Answer in this list | Notes |
|---|---|---|
| Casting Godo (creature spell) | Force of Will, Daze (if Godo's player taps out), Pact of Negation, Mindbreak Trap (free if they cast three spells this turn) | Fierce Guardianship, Flusterstorm and An Offer cannot hit it (noncreature only / instant-sorcery only); Misdirection needs a target. |
| Enter trigger fetches Helm | none that stops it | Triggers cannot be countered by this list. |
| Helm attached (equip {5}, or Hammer of Nazahn) | Chain of Vapor, Into the Flood Maw (gift mode hits a nonland permanent), Snap, in response to the equip ability or the Hammer trigger: bounce Helm or Godo | Best window; each costs 1-2 mana. |
| Token copies each combat | bounce the token (Snap/Flood Maw) | Only delays; they make more. |
| Dualcaster Mage line | Counter the Mage (Force, Daze, Pact); counter the sorcery copy spell (Fierce Guardianship, Flusterstorm, An Offer); or counter the original in response to the Mage trigger so it fizzles | Defense Grid makes Mage cast on our turn cost 3 more. |
| Their Pyroblast/REB on our blue spells | Hexing Squelcher (spells can't be countered) | Pyroblast counters a blue spell: Oracle is blue. |

How this deck beats Godo: it wins on turn 1–3, before Godo's six-mana turn 2–3 [inference]. For the Godo spec:
Godo should hold Pyroblast/REB for Oracle, Fierce Guardianship, Force of Will, Daze and Misdirection (all blue), and
should expect a win from this deck as early as turn 1. Dualcaster is its fastest answer to a stack of free counters.

## Decision scenarios (become AI tests)
Format: position / right action / reason / tag.

1. **Turn 1 on the play. Hand: Watery Grave, Lotus Petal, Thassa's Oracle, Demonic Consultation, Dark Ritual, Force
   of Will, Mana Vault.** Play the land, Petal and Mana Vault only if they build toward turn 2; do not cast Oracle yet.
   Reason: line A needs three mana (UU + B) and only two sources are in play, so the attempt would fail and expose
   Oracle. Cast it on turn 2 with Force of Will backing. Tag [inference, Oracle text].
2. **Turn 2. Board: Rograkh, Underground Sea, Watery Grave, Lotus Petal. Hand: Oracle, Consultation, Fierce
   Guardianship. An opponent has three untapped Islands.** Fierce Guardianship cannot protect Oracle (it counters only
   noncreature spells). Cast Oracle only if a Force of Will or Pact of Negation is also in hand; otherwise wait a
   turn or bait with a noncreature spell. Tag [inference, Oracle text].
3. **Pod: an opponent shows a win next turn. We hold Pyroblast, one free counter, Oracle and Consultation but only
   two usable sources.** Spend the free counter on their win, not on Rhystic Study, and run line A next turn.
   Reason: stopping a win is worth more than a card, and our attempt is one turn away. Tag [inference].
4. **Hand: Breach, LED, Brain Freeze, Oracle, one more card. Vexing Bauble is on our board.** Sacrifice Bauble
   ({1}, tap: draw) before casting LED. Reason: Bauble counters any spell cast with no mana spent, which includes the
   {0} LED escape and Pact of Negation. Tag [inference, Oracle text].
5. **Ad Nauseam resolved at 40 life; Oracle and Tainted Pact are already in hand.** Stop drawing at once and run line A.
   Reason: more cards only cost life; the win is in hand. Tag [inference]; Natux: it averages 30+ cards and usually the pilot just goes for it, stopping early only to avoid dying [primer: Natux].
6. **Rograkh in play; hand: Culling the Weak, Infernal Plunge, Diabolic Intent, Mox Amber.** Use at most one sacrifice
   spell (or recast Rograkh from the command zone for {2} between them). Reason: "don't double-book your Rog". Tag
   [primer S5] for the warning, [inference] for the tax.
7. **Our turn, Underworld Breach to be cast; an opponent holds Force of Will with blue cards in hand.** Cast Hexing
   Squelcher first if mana allows (our spells can't be countered); otherwise keep Pact of Negation or Fierce Guardianship
   ready for the counter. Reason: Breach is the single point of failure for line B. Tag [inference, Oracle text].
8. **An opponent has Null Rod out; hand: Mox Amber, Lotus Petal, LED, Oracle, Pact, two lands.** Run line A from
   lands. Reason: Null Rod stops artifact abilities including LED and the Moxen; Oracle + Pact need none. Tag [primer S1:
   Null Rod is a weakness; the line choice is inference].
9. **An opponent has Rest in Peace out; hand: Breach, LED, Brain Freeze, Oracle, Pact.** Use Oracle + Pact (line A),
   not Breach. Reason: Breach needs a graveyard. Tag [inference].
10. **Opening seven: four lands, Rhystic Study, Sol Ring, Demonic Tutor.** Keep. Reason: acceleration plus a tutor
    reaches a payoff [primer S1 order]. A hand of the same lands, Study and no tutor or payoff would be a mulligan. Tag
    [primer: Natux]: a tutor and fast mana to cast a payoff on its turn.

11. **Hand (7): Ad Nauseam, Brain Freeze, Chrome Mox, Command Tower, Culling the Weak, Pact of Negation, Wishclaw Talisman.** Keep: turn-1 Ad Nauseam (Rograkh for Culling's BBBB) with Pact as backup and Wishclaw as a tutor if it is stopped. Reason: payoff, fast mana, protection. Tag [primer: Natux; example hand].
12. **Hand (7): Dark Ritual, Demonic Tutor, Gitaxian Probe, Marsh Flats, Necropotence, Underground Sea, Yawgmoth's Will.** Keep: turn-1 Necropotence with no fast mana, refill, then a big turn (Borne Upon a Wind if found) for a turn-2 end-step win, else overwhelm on turn 3. Tag [primer: Natux; example hand].
13. **Hand (4): Gitaxian Probe, Deflecting Swat, Flusterstorm, Timetwister, Scalding Tarn, Underground Sea, plus one more card.** Protection and a payoff but too little mana: keep only at 4, ship at 7. Natux's version also holds Brainstorm and Ponder (not in our list); the rule is the same. Tag [primer: Natux].
14. **Hand (7): Arcane Signet, Badlands, City of Brass, Flooded Strand, Force of Will, Rhystic Study, Necropotence.** Trap hand: good cards, but the earliest impactful play is turn 3 (Natux's version uses Necrodominance, not in our list). Ship at 7, keep at 4 in many pods. Tag [primer: Natux].
15. **Hand (7): Mystic Remora, Rhystic Study, three lands, Fierce Guardianship, Pyroblast; no fast mana, no payoff.** Mulligan. Reason: engines alone do not win before an opponent does; dig for mana plus a payoff. Tag [primer: Ondas, Natux payoff table].
16. **Turn 2. Hand: Ad Nauseam, Oracle, Pact, Gitaxian Probe, mana for a turn-2 Naus. Probe shows an opponent with 3 untapped Islands and two blue cards.** Wait a turn, or cast Ad Nauseam at the end step before your turn if mana allows. Reason: expect interaction, and a lost end-step Naus still lets you untap. Tag [primer: Natux for wait and end step].
17. **The pilot holds Pact of Negation and Force of Will during setup (turn 1, an opponent casts Rhystic Study).** Hold both. Reason: Pact only when comboing or stopping a win; Force of Will only in critical scenarios. Tag [primer: Natux].
18. **Rule of Law on an opponent's board; hand: Oracle, Pact, Consultation, Dark Ritual.** Do not run the two-spell line this turn; cast Tainted Pact at the end step of the opponents' turn and Oracle on your own turn, or bounce Rule of Law (Chain of Vapor, Snap). Tag [primer: Ondas for both answers]; the one-spell-per-turn reading is [inference].
19. **Mid game, turn 5, no tutor or payoff in hand, opponents holding open mana, another player tapped low for a combo.** Wait: let that player try, then go after they spend interaction. Rograkh gives no card advantage, so dig with draw. Tag [primer: Natux, Ondas].
20. **Turn 3: Final Fortune and Underworld Breach pieces in hand, Necropotence resolved.** Do not plan the Breach loop for the Final Fortune turn. Reason: Natux says Final Fortune is useless on Breach and Yawgmoth's Will turns; win with line A in the extra turn. Tag [primer: Natux].


## Combo tests (become scripted tests)
Mana is from the listed permanents; opponent has no responses unless stated.
1. **Line A.** Board: Underground Sea, Watery Grave, Lotus Petal, Rograkh. Hand: Thassa's Oracle, Demonic Consultation,
   Fierce Guardianship. Library 60 cards. Cast Oracle (Sea U, Petal U); with its trigger on the stack cast
   Consultation (Grave B) naming a card in hand. Expected: library exiled, trigger resolves, we win.
2. **Line A with Pact.** Same, but with Tainted Pact (1B) and one more land. Expected: library exiled, win. Check the
   deck is singleton so no two cards share a name.
3. **Line B.** Board: Rograkh, Mox Amber, Lotus Petal. Hand: Underworld Breach, Lion's Eye Diamond, Brain Freeze,
   Thassa's Oracle, one other card. Library 70. Cast Breach (R from Amber, 1 from Petal), cast LED, sacrifice it (UUU),
   escape Brain Freeze at ourselves, loop LED and Brain Freeze, escape Oracle. Expected: library empty, Oracle wins.
   Check that Oracle ends in the graveyard and is not exiled as an escape cost.
4. **Line C.** Hand: Ad Nauseam, Dark Ritual, Lotus Petal, Oracle, Pact. 40 life, lands for BB plus Dark Ritual. Cast
   Ad Nauseam, stop at 5 life or when line A is in hand. Expected: line A completes without dying.
5. **Line D.** Board: five mana (BBB + 1U available), Hand: Necropotence, Borne Upon a Wind, Final Fortune.
   Expected: Necropotence, pay life, end step cards arrive, extra turn taken; win must happen in it.
6. **Line E.** Not testable until a primer sequence is found.
7. **Negative: Bauble.** Same as test 3 but Vexing Bauble on our board, untouched. Expected: the {0} LED escape is
   countered and the loop stops.

## Open questions
- Natux and Ondas now give mulligan, turn-by-turn and go/wait rules. No source gives how to choose which opponent to stop; that stays [inference].
- Sources S2 and S5 describe Brain Freeze and Breach loops; S1 says LED makes UUU. No source gives the exact
  number of escape-cost cards needed, or whether the loop always empties the library. The engine needs to model
  graveyard size.
- Vexing Bauble: no primer says how the deck uses it. By its Oracle text it counters the deck's own {0} spells
  (Pact, LED escape, free counters). Does the pilot cast it before or after the win? Lead decides.
- Null Rod text (activated abilities of artifacts can not be activated) was not re-read on Scryfall; scenario 8 relies on it.
  abilities? Not verified; check the engine.
- Mox Diamond, Chrome Mox and dual-land text were not re-checked on Scryfall.
- Wishclaw Talisman gives an opponent control; does the pilot ever use it? Sources are silent.
- Line E (Yawgmoth's Will) has no primer sequence.
- Natux (2026) outranks Ondas (2021) here. Conflicts settled for Natux: mulligans to 3 or fewer are usually wrong (Ondas goes lower); Breach Freeze is the preferred line (Ondas also prefers Breach, our table above keeps Oracle as line A for the sources that call it main); Ad Nauseam is cast in the main phase, or at the end step when interaction is likely; Tainted Pact need not exile the whole library.
- Ondas leans on Dockside Extortionist, Mana Crypt, Peer into the Abyss, Dualcaster + Twinflame and Stitcher's Supplier lines; none apply [primer: Ondas, outdated: banned or not in our list].
- Ondas says an end-step Tainted Pact into Oracle beats stax but not how the next-turn library draw is avoided; Natux does not cover it.
- Natux names Necrodominance, Deadly Rollick, Slaughter Pact, Powerbalance and The One Ring, which our list lacks; his advice on those is not used.
