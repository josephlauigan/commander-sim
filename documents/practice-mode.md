# Practice mode

An interactive, graphical way to play one of your decks against the simulator's AI opponents, so you can practise a deck without setting up a physical game. It is the `commander-practice` program (`rust/crates/practice/`) with a browser page, or a terminal client.

## Goals

- **Play your deck by hand** against three AI opponents drawn from one tier of `decklists/pool/`, using the same AIs, card rules and tier decks as the simulations (one engine, `sim-core`).
- **Fully manual play, no guidance during the game.** You tap lands, cast spells, choose targets, attack, block and respond. The game only tells you when a move is illegal and why. It never lists "good moves".
- **Learn from the AI after the game.** At every decision you make, the AI's choice is recorded silently. After the game, a review screen shows where you and the AI differed, and lets you replay from that point with the AI's choice.
- **Practice tools** that you can switch off: Hint (what the AI would do now), Undo, and saved games you can replay.
- **Runs locally with no installs**: one binary serves the page, which is plain HTML/CSS/JS with no build step.

## Non-goals

- **Not a full rules engine.** You play against the simulator's model of Commander, with its simplifications (see [Rules model](#rules-model) and [how-it-works.md](how-it-works.md#simplifications)). Gaps that manual play exposes get fixed in the engine, which also improves the simulations.
- **No online play** and no deck building (two people on one local network is supported: see [Two players on a network](#two-players-on-a-network)). Decks come from your deck folders (`decklists/JD/`, `Avery/`) and are edited with `cardgen deck`.
- **The simulations are unchanged.** Every code path for a person runs only when a human is seated; a copied game (the look-ahead's) never has one.

## Starting it

```
cd rust && cargo build --release -p practice
rust/target/release/commander-practice                     # opens http://127.0.0.1:8765 in your browser
rust/target/release/commander-practice --port 9000 --no-browser
rust/target/release/commander-practice --lan               # a friend on the same network can join a two-player game
rust/target/release/commander-practice --text --deck sauron --tier t3 [--step]   # the terminal client
```

Other options: `--data DIR` (the card data folder, default `data/`), `--static DIR` (the page, default `rust/crates/practice/static/`), and for `--text` also `--seed`, `--seat`, `--opponents A,B,C`, `--profile` and `--ai`. Saves go to `data/saves/` and card images to `data/images/` (neither is in git). Your decks' cards show the printings in `data/printings/<deck>.txt` (in git; written by `cardgen deck` from a list with set codes): each seat sees its own deck's version of a card, a stolen permanent keeps its owner's, and several printings of one basic land take turns across the copies. Other decks, and cards without a printing, show Scryfall's usual printing.

## Screens

1. **New game (setup).**
   - **The table:** the tiers t0-t6, each with its decks (five, six at Tier 6).
   - **Your opponents:** three drawn at random from the chosen tier (two in a game with a friend), always at random: a new draw when the tier or the number of players changes, and each time you come back to this screen. Hovering over one (or tapping it) shows a box with how it plays: its plan, how it wins, what to watch out for and its interaction (the deck file's "At the table" section), plus its Game Changers and combos.
   - **Your deck:** chosen after you see the opponents; each shows its commander's art, bracket, Game Changer count and its sim win rate at the chosen tier.
   - **Options:**
     - your seat: random, or 1st–4th
     - the opponents' interaction profile: loose (the default) or conservative
     - the opponent AI: look-ahead (stronger, thinks for a few seconds at its decisions) or adaptive (faster)
     - a seed, for replaying or sharing a game
     - when the game stops for your priority (*when I can respond*, *on every spell, ability and trigger*, or *at every step*), which can also be changed during the game
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

This section covers what manual play needs from the engine that the simulations never needed.

### Turn structure and priority

- **Steps:** turns run through untap, upkeep, draw, main 1, combat (beginning, declare attackers, declare blockers, damage, end), main 2, end step and cleanup. The engine runs these steps, and the human seat adds priority stops inside them.
- **Priority order:** the active player receives priority first. Priority then passes in turn order. 
- **Resolution:** a spell or ability resolves once every player passes in succession without adding anything.
- **Your turn:** you get priority in each step, and you may act or pass. Sorcery-speed actions (spells, lands, equip, sorcery-speed abilities) are legal only in your main phases with an empty stack.
- **Opponents' turns:** you get priority whenever an opponent casts a spell, activates an ability, declares attackers, or reaches a step where you could act. How often it stops is the *Stop* setting: *when I can respond* (the default: opponents' spells, abilities and triggers when you hold an answer, attacks on you, the end of each other turn, and your declare blockers step when you could act), *on every spell, ability and trigger*, or *at every step (full control)*. A change applies from your next decision, and Undo and saved games replay it exactly.
- **The stack:** spells, activated abilities and triggers go on the engine's real stack (see [how-it-works.md](how-it-works.md#the-engine)). You can respond to anything on it, and opponents can respond to your response, to any depth. When several of your triggers happen at once you choose their order.

### Mana

- **A real pool per player:** amounts of W, U, B, R, G and colourless. Mana empties at the end of each step.
- **For you:** clicking a land, rock or mana creature taps it and adds its mana to your pool. A source that can make several colours asks which one.
- **Paying:** casting pays from the pool. If the pool can't cover the cost, the move is refused with the reason. Pay-life costs, Phyrexian mana and X costs ask how to pay.
- **For the AI:** it keeps the current automatic payment (`plan_pay`). Rituals and Treasures add to the pool for both.

### Choices you make, and where they hook into the engine

| Decision | What happens |
|---|---|
| Mulligan, and which cards to bottom | Keep/mulligan prompt, then pick cards to bottom |
| Land drop, casting, abilities, equip | Free play: click cards and permanents, validated by the rules check |
| Targets | Click a legal target; illegal ones are refused with the reason |
| Attackers and which player | Select attackers, then the player (or planeswalker) each attacks |
| Blockers | Assign your untapped creatures as blockers, then damage order |
| Responses | Priority stop with the stack shown |
| Discards (including hand size) | Pick the cards |
| Sacrifices and edicts | Pick the permanent |
| Tutors and searches | Pick from the library list |
| Scry, surveil, top-of-library order | Arrange the cards |
| "Choose one" modes, optional triggers ("you may") | Prompt with the modes, or yes/no |
| The Ring tempts you | Pick the Ring-bearer |

- **Mechanism:** at each of these places the engine code first asks whether the seat has a person (`human_of(g, p)`). If so, it calls a helper in `sim-core/src/human/choices.rs` (`choose`, `yes_no`, `pick_cards`, `scry`, `pay_tax` and about twenty more) that posts a request and waits for the answer. Otherwise the AI code runs as it always has. The comment `HUMAN(phase 9)` no longer marks anything: every site is filled.
- **Gaps:** a choice that has no person branch falls back to the AI and shows in the log as "(auto)". A normal game with your decks has none.

### The rules check

- **Location:** `sim-core/src/human/legal.rs`.
- **Input and output:** it takes an action you tried and returns either OK or a one-line reason.
- **What it checks:**
  - timing (sorcery vs instant speed, whose turn it is, empty stack)
  - costs against the pool, including commander tax and additional costs
  - one land per turn, plus extra land drops
  - summoning sickness for tap abilities and attacks
  - legal targets (hexproof, protection, shroud, ward costs)
  - blocking restrictions (flying, menace)
  - "can't cast" locks (Grand Abolisher and similar)
- **Where the data comes from:** the checks reuse the engine's own functions wherever they exist: `castable`, `legal_targets`, the lock hooks and `can_block`.

## How it is built

```
browser (static HTML/CSS/JS)  <-- JSON + server-sent events -->  server/ (std::net + httparse, a thread per connection)
                                                                        |
                                                                session/: one game
                                                                  game thread runs the engine
                                                                  SessionHuman <-> answer channel
                                                                  event log (log lines -> structured events)
                                                                        |
                                              sim-core: engine/, ai/, human/, impls/
```

### Code layout

```
rust/crates/sim-core/src/human/      the person's seat (inert in simulations)
  human.rs       the main-phase loop: ask, apply through the engine's own functions, tell the reason if illegal
  legal.rs       the rules check and its reasons
  mana.rs        the mana pool: tapping sources, paying costs, emptying between steps
  combat.rs      attackers, blockers, damage order
  choices.rs     the helper functions for every other decision
  cards.rs, abilities.rs   your decks' card-specific choices and activated abilities
  karn.rs        every activated ability of the Karn deck's cards, for a person
  alt.rs         alternative costs and graveyard and land abilities (the hooks only the AI read) offered to a person
  specs.rs       what a person chooses when casting a spell that resolves by its own code (targets, modes, X)
  script.rs      ScriptHuman: answers from a list, for tests and the text client
sim-core/src/control.rs   Human, Request, Answer, Humans (who sits where)
rust/crates/practice/src/
  main.rs        options, start the server, open the browser, or run the text client
  lib.rs         the library, and practice_start(), the C entry point the iPad shell calls
  session/       one game: mod.rs (the thread, events, Undo, saves), human.rs (SessionHuman), setup.rs (seating, building the game), api_impl.rs
  view.rs        the table as one seat may see it (JSON)
  advisor/       Hint: mod.rs (main phase, attacks, counters), mulligan.rs (the keep rule and the bottom card), other.rs (every other decision)
  bundle.rs      the iPad app's first-launch data copy
  shadow.rs      the AI comparison log and the review
  server/        http.rs, routes.rs, hub.rs (the current game, lobby, autosave), sse.rs, lan.rs, api.rs
  catalog.rs     what the setup screen shows
  images.rs      card images: lookup, download, cache
  text.rs        the terminal client
  static/        index.html, app.js, table.js, styles.css, knight-dragon.gif
```

### The game thread and the person's seat

- **The game runs on its own thread** (with a large stack, since the engine recurses) using the engine's normal loop. The AI seats play as they do in the simulations.
- **Your seat's decisions block the game.** Each decision in your seat goes to `SessionHuman::ask`, which posts a *request* (what's being decided, and the legal choices) as an event and waits on a channel for your answer. The engine keeps its straight-line design, with no rewrite into a state machine. Closing the session sends `Answer::Cancel`, which unwinds the game through `Stop::Cancelled`.
- **Structured events.** Each log line becomes an event, along with `turn`, `request`, `over`, `error`, `reset` and `reviewing` events. Each event has an increasing id; the page reconnects with `Last-Event-ID` and gets what it missed. While playback is paused, the stream holds, and the game thread waits.
- **Hidden information.** `view.rs` builds the state from one seat's point of view: opponents' hands and all libraries are counts only. Cards the rules make public, such as a revealed card, are shown. Private log lines are tagged for their seat; the AI's reasoning traces stay out of the stream.
- **A panic** on the game thread is caught and becomes an `error` event; the server stays up.

### Undo, saves and replay

A game is its deck, tier, seed, seat, opponents, profile, AI mode and the list of your answers. Rust games are deterministic from the seed and the answers, so:

- **Undo** stops the thread, restarts from the seed and feeds the earlier answers back without asking (the page shows those events at once), then asks live. **Try it** is "undo, then a different answer to decision n".
- **The search tape** records the look-ahead's results during the real game and plays them back in a replay, so Undo does not search again. A mismatch ("search tape out of step") panics loudly.
- **Saved games** are JSON (settings, seed, answers, tape, AI comparison) in `data/saves/`. The game also saves itself each time it waits on you (`autosave.json`), which is what *Continue* loads.

### Hint and the AI comparison (shadow log)

- **Hint** (`advisor/`) runs the look-ahead for a main-phase, attack or counter decision on a copy of the game and shows the AI's choice with the scores. For the mulligan it applies the AI's keep rule to your hand and gives the reasons; for other decisions (discards, sacrifices, scry and surveil) it gives the AI's own pick where it has one, otherwise a sentence on what to weigh.
- **Recording:** at each main-phase or attack decision you make, the session keeps a snapshot of the game (a `Game::clone` with the AI in every seat, as the look-ahead uses) before your answer is applied.
- **The AI's answer:** worker threads (up to 4; 2 on the iPad) ask the AI what it would do in each snapshot and evaluate both options with the look-ahead's playouts, giving a win estimate for "your choice" and "the AI's choice".
- **Stored with each decision:** turn, step, a plain-English description of the situation, your choice, the AI's choice and both estimates. A forced move is recorded but not scored.
- **Spoiler-free:** the log stays hidden until the game ends. The review screen reads it, and "Try it" restores the position with the AI's choice applied. If you play faster than the workers keep up, the queue finishes at the end of the game before the review opens, with a progress bar.

### The server

No web framework: a thin layer on `std::net::TcpListener` and `httparse`, a thread per connection, about 250 lines (`server/http.rs`). It parses requests (headers, a `Content-Length` body capped at 1 MB), serves the static files with a path-traversal check, and writes the event stream.

- `GET /`, `/static/*`, `/images/*`, `/api/state`, `/api/options`, `/api/saves`, `/api/review`, `/api/events` (server-sent events, with keep-alive comments).
- `POST /api/new`, `/api/join`, `/api/answer` (carries the decision's id, so a stale or repeated answer is refused), `/api/undo`, `/api/tryit`, `/api/approve`, `/api/save`, `/api/load`, `/api/autopass`, `/api/hint`, `/api/quit`.
- `new`, `load`, `quit`, `save` and `saves` answer 403 to clients that are not on the host computer.
- Without `--lan` it binds `127.0.0.1`; with it, `0.0.0.0`, and prints the addresses a friend can use.
- `Hub` holds the current session, the lobby, the autosave and rewind approvals behind a mutex; handlers never hold the lock across a wait.

### Two players on a network

Two people, each on their own computer, play two different decks of your deck folders against two AI opponents from the tier.

- **One game, one engine.** The game runs on the host's computer, on one game thread. Each person's seat has its own `SessionHuman`; the engine routes each decision to the right seat. Both mulligan (in seat order); priority, responses and end-of-turn windows go to each person in turn order, as at a real table.
- **Seats.** Each browser has a cookie. The host opens the table (`POST /api/new` with `two`, which needs `--lan`), which waits in a lobby with a six-digit code; the friend joins with the code and a deck other than the host's. The hub maps both cookies to their seats. A page on the host computer without a seat plays the host's seat. Only the host computer can start, save, load or end games.
- **What each person sees.** The session builds a view of the table for each person's seat with every action. Requests and messages are tagged with their seat. The server keeps one history and filters it per seat: the other person's requests become "waiting for <name>", their hand is a count, and a card they tutor to hand or to the top of their library is logged by name only on their screen (an AI's tutor is hidden from both).
- **Answers in one list.** Both people's answers go into one list in the order the game asked, with whose each was. Undo (or Try it) of one person's decision rewinds to it, which also takes back the other person's later answers, so the other person must agree first (a proposal they accept or decline). Saved games keep both decks, both answer lists and both comparisons; loading one opens the lobby again for the same second deck.
- **Per person:** Hint (only for the seat being asked), the AI comparison and the review.

### Card images

- **When:** images load on the loading screen, before the first turn, and only for the cards in this game. That's your deck, the three opponents drawn, and the tokens and emblems those decks can make. Nothing is downloaded during play.
- **Lookup:** names go to Scryfall 75 to a request, with the pacing (at most 10 requests a second) and descriptive User-Agent its guidelines ask for. The `all_parts` field lists the tokens a card makes.
- **Cache:** images download at the `normal` size (488x680) into `data/images/`, with an index in `data/images/index.json` (card name to files). Cached images are reused, so a game with decks you've played before starts almost at once. `data/images/` is in `.gitignore`.
- **Offline, or if a download fails:** the card is drawn in the page (name, cost, type, power/toughness) and the game still starts. A card that couldn't be predicted, such as one stolen from a library, is drawn the same way if it isn't cached.
- **Terms:** Scryfall's terms allow this for personal use; the page credits Scryfall.

### The iPad

`practice` also builds as a static library exporting `practice_start(bundle_dir, app_dir, port)`, `practice_stop()` and `practice_set_lan(bool)` (`src/lib.rs`). `practice_start` copies the data bundled with the app into its writable folder when the build is new (`src/bundle.rs`; saves are never touched) and starts this same server on 127.0.0.1 in app mode. The Swift app in `ios-shell/` shows that port in a web view; its README has the build steps.

## Tests

`cargo test --release` in `rust/` runs them all. Practice mode's are:

- **Rules tests**, `sim-core/tests/play_*.rs`, driven by `ScriptHuman`: hand, mana, turn and the rules check, abilities and targets, combat, responses, choices, and each deck's card-specific choices.
- **Session tests**, `practice/tests/session_*.rs`: events, the seat's view, Undo, saves, replay determinism (a script of answers replays to the same view after Undo and after save and load, with and without the tape), the AI comparison.
- **HTTP tests**, `practice/tests/http_*.rs`: the server in-process on port 0 with a temp saves folder, a fake Scryfall (`PRACTICE_SCRYFALL`) and the adaptive AI, through a small client over `TcpStream`. Set `PRACTICE_URL=host:port` to run the same suite against a server that is already running. They cover the routes and their error texts, the event stream, the lobby and the LAN seats, saves, catalog and images.
- **Bot games**, `practice/tests/play_bot_games.rs`: a scripted player plays whole games with each deck.

