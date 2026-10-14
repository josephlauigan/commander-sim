# Rules for the Tymna partner agents (2026-10-13)

Two agents work in parallel so two new four-colour partner decks can be measured against each other:
**`thrasios`** (`decklists/JD/tymna-thrasios-partners.md`, Tymna the Weaver + Thrasios, Triton Hero) and **`kraum`**
(`decklists/JD/tymna-kraum-partners.md`, Tymna + Kraum, Ludevic's Opus). `partner-engine` teaches the sim two
commanders per seat and registers the decks; `tymna-cards` models the cards. Read this file, then your brief
(`rust/agents/<task>.md`), then your handoff file `rust/agents/handoff-<task>.md` if it exists.

`rust/agents/OVERNIGHT-RULES.md` still applies in full (never run Python, worktree is your world, no wait loops, edit
only your files, how cards are modeled, checks, keeping context small, 15-line report), with these changes:
- **CPU:** the user has allowed the whole machine for this job. Use `cargo -j 10` (two agents build at once) and
  `--jobs 10` for smoke runs. Always prefix heavy commands with `nice -n 19`.
- **Memory cap:** `nice -n 19 systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 <command>`.
  Exit 137 = cap hit = a bug, never a flake.
- **Behaviour of the existing decks must not change.** Any change to shared code must leave every existing deck's
  games identical: check with same-seed runs before and after (see your brief).
- Commits: `git add -A rust/ data/ decklists/ && git commit -m "..."`, no Co-Authored-By line. Never push.
- Final report line 1: `DONE` or `HANDOFF`; line 2: your model ID.
