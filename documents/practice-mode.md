# Practice mode: design

An interactive, graphical way to play one of your decks against the simulator's AI opponents, so you can practise a deck without setting up a physical game.

Status: design agreed, not yet built. Screen mockups: [Commander Practice Table](https://claude.ai/artifact/TceoMAvA5PP1zmitupru8v) (private to the repo owner).

## Goals

- **Play your deck by hand** against three AI opponents drawn from one tier of `decklists/pool/`, using the same AIs, card rules and tier decks as the simulations.
- **Fully manual play, no guidance during the game.** You tap lands, cast spells, choose targets, attack, block and respond. The game only tells you when a move is illegal and why. It never lists "good moves".
- **Learn from the AI after the game.** At every decision you make, the AI's choice is recorded silently. After the game, a review screen shows where you and the AI differed, and lets you replay from that point with the AI's choice.
- **Practice tools** that you can switch off: Hint (what the AI would do now), Undo, and saved games you can replay.
- **Runs locally with no installs,** like the rest of the repo: Python's standard library on the server, plain HTML/CSS/JS in the browser.

## Non-goals

- **Not a full rules engine.** You play against the simulator's model of Commander, with its simplifications (see [Rules model](#rules-model)). Gaps that manual play exposes get fixed in the engine, which also improves the simulations.
- **No online play** and no deck building (two people on one local network is supported: see [Two players on a network](#two-players-on-a-network)). Decks come from your deck folders (`decklists/JD/`, `Avery/`) and are edited with `update_deck` as today.
- **The simulations are unchanged.** Every new code path runs only when a human is seated. The existing test suite and the recorded sim numbers must not move.

## Starting it

```
python3 -m commander_sim.play            # opens http://127.0.0.1:8765 in your browser
python3 -m commander_sim.play --port 9000 --no-browser
python3 -m commander_sim.play --lan      # a friend on the same network can join a two-player game
```

## Screens

The mockup has one artboard per screen.

1. **New game (setup).**
   - **Your deck:** your decks, each shown with its commander's art, bracket, Game Changer count and its average win rate from the latest overnight run.
   - **The table:** the five tiers, each with its five decks and the chosen deck's sim win rate against that tier.
   - **Opponents:** three drawn at random from the tier (the default), or you pick three.
   - **Options:**
     - your seat: random, or 1st–4th
     - the opponents' interaction profile: conservative or loose
     - a seed, for replaying or sharing a game
     - on/off switches for Hint, Undo and recording the AI comparison
   - **Start game.**
2. **Preparing the table (loading).** The opponents are drawn and seated, the card rules are built and the card images are loaded. The screen shows only a looping 8-bit animation (a knight shielding against a fire-breathing dragon under the five mana colours) and one progress bar.
3. **Your turn.**
   - **The table:** the opponents across the top, your battlefield (creatures and other permanents, then lands) in the middle, and your hand, library, graveyard and exile along the bottom.
   - **Header:** a step tracker (Untap, Upkeep, Draw, Main 1, Combat, Main 2, End), plus Hint, Undo and Menu.
   - **Your mana pool** sits beside your lands, and **Pass priority** is at bottom right.
   - **Errors:** illegal moves bring up a short explanation, such as "Can't cast Sheoldred, the Apocalypse. It costs {2}{B}{B}; your mana pool has {B}{B}."
4. **Opponent's turn.**
   - **Playback:** opponents' actions play out one at a time, with Pause, Next action and 1×/2×/4× speed.
   - **Priority:** whenever you receive priority, playback stops. That includes every spell or ability an opponent puts on the stack and every declaration of attackers.
   - **Responding:** the stack is shown in the middle, with the priority order around the table ("Prosper · passed → Chulane · passed → You → Krenko"). You can tap lands and cast instants or activate abilities, or pass.
5. **After the game (review).**
   - **Summary:** the result and how the game ended, how many decisions you made, how many matched the AI, and how many the AI rated clearly worse.
   - **Differences table:** each decision where you differed, showing the turn, the situation, what you did, what the AI would have done, and the AI's estimated win chance for each. A **Try it** button reloads the game at that decision with the AI's choice. Filters switch between differences only and all decisions.
   - **Actions:** save the game, play the same seed again, or start a new game.

## Rules model

This section covers what manual play needs from the engine that the simulations have never needed.

### Turn structure and priority

- **Steps:** turns run through untap, upkeep, draw, main 1, combat (beginning, declare attackers, declare blockers, damage, end), main 2, end step and cleanup. The engine already runs these steps, and the human seat adds priority stops inside them.
- **Priority order:** the active player receives priority first. Priority then passes in turn order. The engine's response window already asks players in turn order after the caster (`g.after(p)`).
- **Resolution:** a spell or ability resolves once every player passes in succession without adding anything.
- **Your turn:** you get priority in each step, and you may act or pass. Sorcery-speed actions (spells, lands, equip, sorcery-speed abilities) are legal only in your main phases with an empty stack.
- **Opponents' turns:** you get priority whenever an opponent casts a spell, activates an ability, declares attackers, or reaches a step where you could act. There is a setting, off by default, to skip the stops where you have nothing castable.
- **Stack depth:** the engine resolves spells one at a time with a response window, not a full stack. You can respond to a spell, and opponents can respond to your response through the existing counter-war logic. Deeper chains are simplified, and the log says so when that happens.

### Mana

- **A real pool per player:** amounts of W, U, B, R, G and colourless. Mana empties at the end of each step.
- **For you:** clicking a land, rock or mana creature taps it and adds its mana to your pool. A source that can make several colours asks which one.
- **Paying:** casting pays from the pool. If the pool can't cover the cost, the move is refused with the reason. Pay-life costs, Phyrexian mana and X costs ask how to pay.
- **For the AI:** it keeps the current automatic payment (`plan_pay`). Rituals and Treasures add to the pool for both.

### Choices you make, and where they hook into the engine

| Decision | Where the engine decides today | What changes |
|---|---|---|
| Mulligan, and which cards to bottom | `ais.mulligan` | Keep/mulligan prompt, then pick cards to bottom |
| Land drop, casting, abilities, equip | `brain.main` action menu | Free play: click cards and permanents, validated by the rules check |
| Targets | inside each effect (`legal_targets` and the choosers) | Click a legal target; illegal ones are refused with the reason |
| Attackers and which player | `ais.combat` / `search.choose_attack` | Select attackers, then the player (or planeswalker) each attacks |
| Blockers | inside `ais.resolve_combat` | Assign your untapped creatures as blockers, then damage order |
| Responses | `engine.counter_window` / `search.choose_counter` | Priority stop with the stack shown |
| Discards (including hand size) | `engine.discard_cards` callers | Pick the cards |
| Sacrifices and edicts | `engine.edict` and sacrifice costs | Pick the permanent |
| Tutors and searches | `ais.tutor_pick` and the search effects | Pick from the library list |
| Scry, surveil, top-of-library order | `impl/topdeck.scry` and relatives | Arrange the cards |
| "Choose one" modes, optional triggers ("you may") | the effect's own code | Prompt with the modes, or yes/no |
| The Ring tempts you | `mine.ring_tempt` | Pick the Ring-bearer |

- **Mechanism:** each row becomes a call to the seat's controller. The AI controller calls today's code unchanged. The human controller asks you.
- **Gaps:** any choice not yet converted falls back to the AI and shows in the log as "(auto)". The Phase 1 goal is no "(auto)" choices in a normal game with your decks.

### The rules check

- **Location:** a new module, `commander_sim/play/legal.py`.
- **Input and output:** it takes an action you tried and returns either OK or a one-line reason.
- **What it checks:**
  - timing (sorcery vs instant speed, whose turn it is, empty stack)
  - costs against the pool, including commander tax and additional costs
  - one land per turn, plus extra land drops
  - summoning sickness for tap abilities and attacks
  - legal targets (hexproof, protection, shroud, ward costs)
  - blocking restrictions (flying, menace)
  - "can't cast" locks (Grand Abolisher and similar)
- **Where the data comes from:** the checks reuse the engine's own data wherever it exists: `castable`, `legal_targets`, the lock hooks and `can_block`.

## Architecture

```
browser (static HTML/CSS/JS)  <-- JSON + server-sent events -->  play/server.py (http.server, stdlib)
                                                                        |
                                                                play/session.py: one game
                                                                  engine thread runs the game
                                                                  HumanController <-> decision queue
                                                                  event stream (log -> structured events)
                                                                        |
                                              existing engine: ais.py, engine.py, ai/brain.py, ai/search.py, cards/
```

- **The game runs on a worker thread** using the existing loop (`setup_pool_game`, `_run_rounds`). The AI seats play as they do in the simulations.
- **Your seat's decisions block the game.** Each decision in your seat goes to `HumanController`. That posts a *request* (what's being decided, and the legal choices) and waits for your answer. The engine keeps its straight-line design, with no rewrite into a state machine.
- **Structured events.** The game log becomes events such as `cast`, `resolve`, `tap`, `damage`, `zone_move`, `phase` and `priority`. The browser animates them one by one at the chosen speed. While playback is paused, the event stream holds, and the engine thread waits.
- **Hidden information.** The state sent to the browser is built from your seat's point of view: opponents' hands and all libraries are counts only. Cards the rules make public, such as a revealed card, are shown.
- **Server:**
  - `ThreadingHTTPServer` serves the static files and the image cache.
  - `GET /api/state` returns the view model.
  - `POST /api/action` takes your actions; it returns OK or the rules check's reason.
  - `GET /api/events` is a server-sent events stream.
- **The browser** uses plain modules with no build step: `static/index.html`, `app.js`, `table.js`, `styles.css` and the loading GIF.

### The AI comparison (shadow log)

- **Recording:** at each decision you make, the session snapshots the game (`search.clone`, as the look-ahead AI already does) *before* your answer is applied.
- **The AI's answer:** a background thread asks the AI what it would do in that snapshot. It uses the same controller call with the AI controller, and the look-ahead AI where the simulations use it.
- **Evaluation:** both options are evaluated with the look-ahead's playouts, giving a win estimate for "your choice" and "the AI's choice".
- **Stored with each decision:** turn, step, a plain-English description of the situation, your choice, the AI's choice and both estimates. Only choices the AI would actually weigh are compared. A forced move, such as a single legal option, is recorded but not scored.
- **Spoiler-free:** the log stays hidden until the game ends. The review screen reads it, and "Try it" restores the stored snapshot with the AI's choice applied.
- **Cost:** a look-ahead decision takes a few seconds of CPU, and it runs while you're thinking or watching opponents. If you play faster than it keeps up, the queue finishes at the end of the game before the review opens, with a progress bar.

### Hint, Undo, replay

- **Hint** runs the AI's choice on the current snapshot and shows it, with the look-ahead's summary.
- **Undo** restores the snapshot taken at your previous decision. Snapshots are kept for the whole game.
- **Replay and save.** A saved game is the seed, the table settings and your list of decisions. Replaying it runs the same game.
  - **A problem to fix first:** the look-ahead AI's choices currently depend on object memory addresses. The same seed replays identically only on identical code, and even an unrelated code change can alter a game. This showed up while debugging crashes.
  - **The fix:** make the search's orderings independent of object identity, so saved games replay reliably. That helps the simulations' reproducibility too.
  - **Fallback:** save the full snapshot as a pickle alongside the decision list.

### Two players on a network

Two people, each on their own computer, play two different decks of your deck folders (`decklists/JD/`, `Avery/`) against two AI opponents from the tier.

- **One game, one engine.** The game still runs on the host's computer (the one running the server), on one engine thread. Each person's seat has its own controller; the engine already routes every decision through `human_choice(g, p)` and `controller_of(g, p)`, so it asks the right seat. Both mulligan (in seat order); priority, responses and end-of-turn windows go to each person in turn order, as at a real table.
- **Seats.** Each browser has a cookie. The host opens the table (`POST /api/new` with `two`), which waits in a lobby with a six-digit code; the friend joins with the code and a deck other than the host's. The game maps both cookies to their seats. A page on the host computer without a seat plays the host's seat (as with one player). Only the host computer can start, save, load or end games.
- **What each person sees.** The session builds a view of the table for each person's seat with every action. Requests and messages are tagged with their seat. The server keeps one history and filters it per seat: the other person's requests become "waiting for <name>", their hand is a count, and a card they tutor to hand or to the top of their library is logged by name only on their screen (`engine.log_secret`; an AI's tutor is hidden from both, which also fixes a leak in the one-player table).
- **Answers in one list.** Both people's answers go into one list in the order the game asked, with whose each was. Undo (or Try it) of one person's decision rewinds to it, which also takes back the other person's later answers, so the other person must agree first (a proposal they accept or decline). Saved games keep both decks, both answer lists and both comparisons; loading one opens the lobby again for the same second deck.
- **Per person:** Hint (only for the seat being asked), the AI comparison and the review.

### Card images

- **When:** images load on the loading screen, before the first turn, and only for the cards in this game. That's your deck, the three opponents drawn, and the tokens and emblems those decks can make. Nothing is downloaded during play.
- **Card data changes:**
  - `scryfall.slim` currently drops image links. It needs to keep `id`, `image_uris` (with `card_faces[].image_uris` for two-faced cards) and `all_parts`, which lists the tokens a card makes.
  - Existing cache entries are refreshed on first use.
- **Downloads:**
  - Images download at the `normal` size (488×680) into `data/images/<scryfall id>.jpg`.
  - They go through the existing request pacing (at most 10 requests a second) with the descriptive User-Agent.
  - Cached images are reused, so a game with decks you've played before starts almost at once.
- **Git:** `data/images/` is added to `.gitignore`.
- **Offline, or if a download fails:** the card is drawn in the page (name, cost, type, power/toughness) and the game still starts. A card that couldn't be predicted, such as one stolen from a library, is drawn the same way if it isn't cached.
- **Terms:** Scryfall's terms allow this for personal use; the page credits Scryfall.

## Code layout

```
commander_sim/play/
  __main__.py      python3 -m commander_sim.play: parse options, start the server, open the browser
  server.py        HTTP routes, the event stream, static files and the image cache
  session.py       one game: engine thread, snapshots, undo, hint, save and replay
  controller.py    the controller interface; AIController (today's code) and HumanController (requests)
  mana.py          the mana pool: tapping sources, paying costs, emptying between steps
  legal.py         the rules check and its reasons
  events.py        structured events from the engine log and zone changes
  view.py          the view model from one seat's point of view
  shadow.py        the AI comparison log and the review data
  images.py        the image list for a game, download and cache
  static/          index.html, app.js, table.js, styles.css, knight-dragon.gif
tools/knight_dragon.py   draws the loading animation (Pillow; run once, output committed)
```

**Engine changes** are kept small, and each one is a no-op when no human is seated:
- **Controller calls** at the decision points in the table above.
- **Priority stops** for the human seat.
- **The pool-based payment path** for a seat with a mana pool.
- **Structured events** emitted alongside `log()`.

## Phases

1. **Engine and a text client.**
   - **Scope:** the controller interface; every decision in the table converted; the mana pool; priority stops; the rules check with reasons; structured events; a text client (`python3 -m commander_sim.play --text`) that plays a full game in the terminal.
   - **Done when:**
     - a full game with each of your decks can be played in the terminal with no "(auto)" choices
     - each rule in the rules check has a test on a hand-built position (in the style of `tests/table.py`)
     - the simulation tests and a recorded sim fixture are unchanged
2. **The browser table.**
   - **Scope:** the server; the setup and loading screens; the image cache; the table with click-to-tap, click-to-cast, targeting, attack and block assignment; opponent playback with pause, step and speed; the error messages.
   - **Done when:** a full game can be played in the browser, matching the mockup's screens.
3. **Practice tools.**
   - **Scope:** the AI comparison log and the review screen with "Try it"; Hint; Undo; save and replay (after the determinism fix).
   - **Done when:** the review of a finished game lists every scored difference, and "Try it" continues from the right position.

## Build plan

The phases broken into steps, built in this order. Estimates are hours of working time. Progress is tracked in [practice-mode-progress.md](practice-mode-progress.md).

| Step | What gets built | What you can do after | Estimate |
|---|---|---|---|
| 1a | Game session thread, controller layer, text client from your seat (all choices still automatic), a test that the sims are unchanged | Watch a game from your seat | 2–3 h |
| 1b | Mana pool; tapping lands, rocks and dorks; paying from the pool | Tap for mana | 2–3 h |
| 1c | Your main phase by hand: land drop, casting from hand and command zone (with tax), timing, the rules check with reasons and tests | Play your own turns | 3–5 h |
| 1d | Targets; activated abilities (equip, planeswalkers, abilities such as Jace's Archivist) | Cast removal, use abilities | 3–4 h |
| 1e | Combat: attackers and defenders; blockers | Fight your own battles | 2–4 h |
| 1f | Priority on every opponent spell, ability and attack; instants in response | First fully manual game (rare choices "(auto)") | 3–4 h |
| 1g | Mulligans, discards, sacrifices and edicts, tutors, scry and surveil, modes, "you may" triggers | Nearly every generic choice is yours | 3–4 h |
| 1h | Sauron's card-specific choices (Ring, Kefka, Archivist, Swords, Breach recasting, Brain Freeze, rituals) | Sauron fully manual (Milestone 1) | 3–5 h |
| 1i | Seph (reanimation, loop shortcuts), Veyran, Marchesa | All four decks fully manual | 5–10 h |
| 2a | Server, the seat's view, event stream, a bare page | Follow a game in the browser | 3–5 h |
| 2b | Image pipeline: card-data fields, tokens, download and cache, loading screen | Images load before the game | 3–4 h |
| 2c | Setup screen | Start a game from the browser | 1–2 h |
| 2d | The table: zones, images, tapped cards, counters, hover to enlarge, log, step tracker | See the table | 4–6 h |
| 2e | Your turn: tap for mana, cast and activate, targets, error messages, pass priority | Play your turns in the browser | 4–7 h |
| 2f | Combat: attackers, defenders, blockers | Fight in the browser | 2–4 h |
| 2g | Opponents' turns: playback, pause, step and speed; priority prompt; the stack | Full game in the browser (Milestone 2) | 2–4 h |
| 3a | The look-ahead AI's choices made independent of memory addresses | A seed always replays the same game | 2–4 h |
| 3b | Snapshots and Undo | Take back moves | 2–3 h |
| 3c | Hint | Ask what the AI would do | 1–2 h |
| 3d | The AI comparison log, computed in the background | (recorded) | 3–5 h |
| 3e | Review screen with "Try it" | See where you and the AI differed (Milestone 3) | 3–4 h |
| 3f | Save and replay | Save and replay games | 1–2 h |

Total: about 55–90 hours.

## Risks and open questions

- **Rules gaps.** Manual play will hit card interactions the AIs never exercised. The plan: log them as "(auto)" or "not modelled", and fix the ones that come up in your decks first.
- **Triggered abilities.** The engine resolves triggers in its own order, and you won't choose the order of your simultaneous triggers at first. It's rarely relevant in your decks. A later addition if it matters.
- **Stack depth.** Counter wars beyond a response and a counter-response are simplified, as noted above.
- **Look-ahead cost for the shadow log.** Handled by running it in the background, as above. If it's still too slow, the review can use the heuristic AI for minor decisions and the look-ahead only for the big ones: casts, attacks and responses.
- **Thread safety.** Only the engine thread touches the game. The server reads view models that the engine thread builds at each event.
