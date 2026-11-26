#!/usr/bin/env python3
"""Claude Code hooks that enforce use of the card knowledge base (CLAUDE.md, "Card recommendations").

Events (one argument): prompt (UserPromptSubmit), tool (PostToolUse, matcher Bash|Read), stop (Stop), report.
State: append-only JSONL under ${XDG_STATE_HOME:-~/.local/state}/commander-sim/kb/<session_id>.jsonl. Stdlib only.
Fails open: any exception exits 0 with a one-line warning on stderr; exit 2 is never used.
The check is that the KB was used, not that it was used well.
"""
import json
import os
import re
import subprocess
import sys
import time

PRUNE_DAYS = 14
LOGS = ("optouts.jsonl", "unresolved.jsonl")

# ---------------------------------------------------------------------------------------------- classifier

DOMAIN = re.compile(
    r"\b(cards?|decks?|decklists?|commanders?|partners?|packages?|combos?|archetypes?|strateg(?:y|ies)|"
    r"colou?rless|mono-?\w+|staples?|tutors?|ramp|removal|wipes?|mana base|lands?|precons?|edh|cedh|mtg|"
    r"planeswalkers?|artifacts?|creatures?|dragons?|(?:five|5)[- ]colou?r|ban ?list|game changers?|azorius|dimir|rakdos|gruul|selesnya|orzhov|izzet|golgari|boros|simic|bant|esper|grixis|"
    r"jund|naya|abzan|jeskai|sultai|mardu|temur|five-colou?r|oracle text|colou?r identity)\b",
    re.I,
)
STRONG_INTENT = re.compile(
    r"\b(recommend\w*|suggest\w*|swap\w*|cuts?|buy\w*|upgrad\w+|replac\w+|adopt\w*|options?|ideas?|brainstorm\w*|"
    r"what (?:cards?|strateg\w+|commanders?)|which (?:cards?|commanders?|strateg\w+)|"
    r"(?:build|make|making|building)\b[^.?!]{0,40}\bdeck)\b",
    re.I,
)
WEAK_INTENT = re.compile(
    r"\b(add|include|instead of|alternatives?|best|better|worth|good (?:in|for)|compare|vs|versus|"
    r"should (?:i|we)|could|would|can i|build|make|legal|banned|eligible|identity|work(?:s)? (?:well|in)|fit|slot|"
    r"similar|another|any other|want|need to keep|viable|ways? to|combin\w+|rework\w*|explor\w+|move over|leftover|molded)\b",
    re.I,
)
CARD_QUESTION = re.compile(
    r"\b(is|are)\s+[\w',\- ]{2,40}?\s+(good|legal|banned|worth|playable)\b|"
    r"\bhow (?:does|do)\s+[\w',\- ]{2,40}?\s+work\b|"
    r"\bcan\s+[\w',\- ]{2,40}?\s+be\s+(?:a |your |used |played )?(?:commander|played|used)\b|"
    r"\bwhat does\s+[\w',\- ]{2,40}?\s+do\b",
    re.I,
)
# things that make "card"/"commander" mean the repository, not the game
NOISE = re.compile(
    r"commander[-_ ]sim\w*|`[^`]*`|\S*[/\\]\S*|\S+\.(?:json|md|toml|rs|py|sh|txt|jsonl)\b|"
    r"\b(?:card[-_]\w+|cardgen|cards?\.json|decks?\.json|\w+_decks?|\w+_cards?)\b",
    re.I,
)

# a swap, cut or replacement asked about (not ordered): needs a question form, no domain word
SWAP_QUESTION = re.compile(
    r"(?:\b(?:could|would|can|should|if|what if|how about)\b[^.!]{0,60}\b(?:swap\w*|cut|replac\w+|slot)\b)|"
    r"(?:\b(?:swap\w*|cut|replac\w+)\b[^.!]{0,60}\?)",
    re.I,
)

KEY_RE = re.compile(r"^\[([A-Za-z0-9_]+)\]\s*$", re.M)
NAME_RE = re.compile(r'^(?:name|commander)\s*=\s*"([^"]+)"', re.M)


def deck_words(root):
    """JD's deck keys and the first word of each deck name, from data/my_decks.toml (empty on any problem)."""
    try:
        with open(os.path.join(root, "data", "my_decks.toml"), encoding="utf-8") as f:
            text = f.read()
    except Exception:
        return []
    words = set(KEY_RE.findall(text))
    for n in NAME_RE.findall(text):
        parts = [x for x in re.split(r"[,\s]+", n.strip()) if x.lower() != "the"]
        if parts and len(parts[0]) > 3:
            words.add(parts[0])
    return sorted({w.lower().replace("_", " ") for w in words if len(w) > 3})


def is_brief(prompt):
    return "/tmp/handoff/" in prompt or (len(prompt) > 4000 and "GOAL:" in prompt)


NAME_INTENT = re.compile(r"\?|\b(want|how about|what about|keep|drop|put|add|cut|take|move|use|play|run)\b", re.I)
WORD = re.compile(r"[a-z0-9][a-z0-9'\-]*")


def has_card_name(text, names):
    """True when a catalog card name (two words or more, or eight letters or more) appears in the text."""
    if not names:
        return False
    words = WORD.findall(text.lower().replace("\u2019", "'"))
    longest = names[1]
    for i in range(len(words)):
        for n in range(1, min(longest, len(words) - i) + 1):
            if " ".join(words[i:i + n]) in names[0]:
                return True
    return False


def classify(prompt, decks=(), names=None):
    """Return (tier, brief): tier is strong, weak or none. `names` is (set of lower-case card names, longest)."""
    if is_brief(prompt):
        return "none", True
    text = NOISE.sub(" ", prompt)
    domain = bool(DOMAIN.search(text))
    if not domain and decks:
        domain = bool(re.search(r"\b(" + "|".join(re.escape(d) for d in decks) + r")\b", text, re.I))
    named = has_card_name(text, names)
    if (domain or named) and STRONG_INTENT.search(text):
        return "strong", False
    if (domain or named) and WEAK_INTENT.search(text):
        return "weak", False
    if named and NAME_INTENT.search(text):
        return "weak", False
    if SWAP_QUESTION.search(text):
        return "weak", False
    if CARD_QUESTION.search(text) and (domain or re.search(r"\b[A-Z][a-z]+ [A-Z][a-z]+", text)):
        return "weak", False
    return "none", False


def load_names(roots):
    """(names set, longest) from the catalog, cached as names.txt in the state dir; None when unavailable."""
    try:
        oracle = next((os.path.join(r, "data", "knowledge", "oracle.json") for r in roots
                       if os.path.isfile(os.path.join(r, "data", "knowledge", "oracle.json"))), None)
        if not oracle:
            return None
        cache = os.path.join(state_dir(), "names.txt")
        if not (os.path.isfile(cache) and os.path.getmtime(cache) >= os.path.getmtime(oracle)):
            with open(oracle, encoding="utf-8") as f:
                cards = json.load(f).get("cards", [])
            found = set()
            for c in cards:
                for part in str(c.get("name", "")).split(" // "):
                    n = part.lower().replace("\u2019", "'").strip()
                    w = WORD.findall(n)
                    if len(w) >= 2 or (len(w) == 1 and len(w[0]) >= 8):
                        found.add(" ".join(w))
            os.makedirs(state_dir(), exist_ok=True)
            tmp = cache + ".%d.tmp" % os.getpid()
            with open(tmp, "w", encoding="utf-8") as f:
                f.write("\n".join(sorted(found)))
            os.replace(tmp, cache)
        with open(cache, encoding="utf-8") as f:
            names = set(f.read().split("\n"))
        names.discard("")
        return names, max((n.count(" ") + 1 for n in names), default=1)
    except Exception:
        return None


# ---------------------------------------------------------------------------------------------- state files


def state_dir():
    d = os.environ.get("KB_GUARD_STATE_DIR")
    if not d:
        base = os.environ.get("XDG_STATE_HOME") or os.path.join(os.path.expanduser("~"), ".local", "state")
        d = os.path.join(base, "commander-sim", "kb")
    return d


def safe_id(s):
    return re.sub(r"[^A-Za-z0-9_.-]", "_", str(s or "nosession"))[:100]


def append(path, obj):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    line = json.dumps(obj, ensure_ascii=False) + "\n"
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    try:
        os.write(fd, line.encode("utf-8"))
    finally:
        os.close(fd)


def load(path):
    out = []
    try:
        with open(path, encoding="utf-8") as f:
            for ln in f:
                try:
                    out.append(json.loads(ln))
                except Exception:
                    pass
    except FileNotFoundError:
        pass
    return out


def session_file(data):
    return os.path.join(state_dir(), safe_id(data.get("session_id")) + ".jsonl")


def prune():
    cutoff = time.time() - PRUNE_DAYS * 86400
    d = state_dir()
    try:
        for n in os.listdir(d):
            p = os.path.join(d, n)
            if n.endswith(".jsonl") and n not in LOGS and os.path.getmtime(p) < cutoff:
                os.remove(p)
    except Exception:
        pass


# ---------------------------------------------------------------------------------------------- environment


def project_root(data):
    return os.environ.get("CLAUDE_PROJECT_DIR") or data.get("cwd") or os.getcwd()


def main_checkout(root):
    try:
        out = subprocess.run(
            ["git", "-C", root, "rev-parse", "--git-common-dir"],
            capture_output=True, text=True, timeout=5, stdin=subprocess.DEVNULL,
        )
        if out.returncode != 0:
            return None
        common = out.stdout.strip()
        if not os.path.isabs(common):
            common = os.path.join(root, common)
        return os.path.dirname(os.path.realpath(common))
    except Exception:
        return None


def kb_status(root):
    """(available, reason): the catalog (oracle.json) and a cardgen binary both have to be reachable."""
    roots = [root]
    main = main_checkout(root)
    if main and main != root:
        roots.append(main)
    if not any(os.path.isfile(os.path.join(r, "data", "knowledge", "oracle.json")) for r in roots):
        return False, "no data/knowledge/oracle.json in the project or the main checkout"
    if not any(os.access(os.path.join(r, "rust", "target", "release", "cardgen"), os.X_OK) for r in roots):
        return False, "cardgen is not built (rust/target/release/cardgen)"
    return True, ""


# ---------------------------------------------------------------------------------------------- tool detection

TOKEN = re.compile(r"[;&|]+|[^\s;&|()<>'\"`]+")
HELP = {"--help", "-h", "help"}
FIND_FLAGS = {
    "--name", "--name-contains", "--ident", "--type", "--text", "--role", "--deck", "--gc", "--cmc",
    "--status", "--in-deck", "--not-in-deck",
}
SEPARATOR = re.compile(r"^[;&|]+$")


def bash_is_query(cmd):
    """True when the command runs cardgen map, digest or find with a real argument."""
    toks = TOKEN.findall(cmd or "")
    cg_vars = set()
    for t in toks:
        m = re.match(r"^([A-Za-z_]\w*)=(.*)$", t)
        if m and os.path.basename(m.group(2).rstrip("/")) == "cardgen":
            cg_vars.add(m.group(1))

    def is_cardgen(t):
        if os.path.basename(t) == "cardgen":
            return True
        m = re.match(r"^\$\{?(\w+)\}?$", t)
        return bool(m and m.group(1) in cg_vars)

    for i, t in enumerate(toks):
        if not is_cardgen(t) or re.match(r"^[A-Za-z_]\w*=", t):
            continue
        j = i + 1
        while j < len(toks):  # options before the subcommand: `--` and `--root DIR`
            if toks[j] == "--":
                j += 1
            elif toks[j] == "--root":
                j += 2
            else:
                break
        if j >= len(toks):
            continue
        sub = toks[j]
        args = []
        for a in toks[j + 1:]:
            if SEPARATOR.match(a):
                break
            args.append(a)
        if sub not in ("map", "digest", "find") or any(a in HELP for a in args):
            continue
        if sub == "map":
            return True
        if sub == "digest" and any(a in ("--deck", "--ident") for a in args):
            return True
        if sub == "find" and any(a in FIND_FLAGS for a in args):
            return True
    return False


READ_PATH = re.compile(r"(?:^|/)data/knowledge/(?:map\.md|digests/[^/]+\.txt)$")


def response_failed(resp):
    """A Bash tool_response that shows a failure (the shape is not documented, so look at several fields)."""
    if isinstance(resp, str):
        return bool(re.match(r"\s*Error: Exit code [1-9]", resp))
    if not isinstance(resp, dict):
        return False
    for k in ("exit_code", "exitCode", "returncode", "returnCode", "code", "status"):
        v = resp.get(k)
        if isinstance(v, int) and not isinstance(v, bool) and v != 0:
            return True
    if resp.get("is_error") is True or resp.get("isError") is True or resp.get("success") is False:
        return True
    if resp.get("interrupted") is True:
        return True
    return False


def tool_is_query(data):
    name = data.get("tool_name")
    inp = data.get("tool_input") or {}
    if data.get("hook_event_name") == "PostToolUseFailure":
        return False
    if response_failed(data.get("tool_response")):
        return False
    if name == "Bash":
        return bash_is_query(inp.get("command", ""))
    if name == "Read":
        return bool(READ_PATH.search(str(inp.get("file_path", ""))))
    return False


# ---------------------------------------------------------------------------------------------- stop logic

KB_LINE = re.compile(r"(?m)^\W{0,4}KB:(.*)$")
NAMES_KB = re.compile(r"\b(map|digest|find)\b", re.I)
OPTOUT = re.compile(r"not applicable\s*\(([^)]*)\)", re.I)


def last_assistant_text(data):
    msg = data.get("last_assistant_message")
    if isinstance(msg, str) and msg.strip():
        return msg
    path = data.get("transcript_path")
    if not path:
        return ""
    last = ""
    try:
        with open(path, encoding="utf-8") as f:
            for ln in f:
                try:
                    d = json.loads(ln)
                except Exception:
                    continue
                if d.get("type") != "assistant":
                    continue
                content = (d.get("message") or {}).get("content")
                if isinstance(content, str):
                    t = content
                elif isinstance(content, list):
                    t = "\n".join(b.get("text", "") for b in content if isinstance(b, dict) and b.get("type") == "text")
                else:
                    t = ""
                if t.strip():
                    last = t
    except Exception:
        return ""
    return last


def kb_line_kind(text):
    """(kind, detail): kind is queries, optout, unavailable or none; looks at the last KB: line."""
    lines = KB_LINE.findall(text or "")
    if not lines:
        return "none", ""
    body = lines[-1].strip().strip("*_ ").strip()
    m = OPTOUT.search(body)
    if re.match(r"(?i)^\W*not applicable", body):
        if m and len(m.group(1).split()) >= 3:
            return "optout", m.group(1).strip()
        return "none", ""
    if re.match(r"(?i)^\W*unavailable", body):
        return "unavailable", body
    if NAMES_KB.search(body):
        return "queries", body
    return "none", ""


def stop_decision(data, events, sdir):
    """Return a dict to print as JSON, or None. Appends events through `sdir`'s session file as needed."""
    sf = session_file(data)
    idx = [i for i, e in enumerate(events) if e.get("ev") == "prompt"]
    if not idx:
        return None
    last = idx[-1]
    p = events[last]
    if p.get("tier") in (None, "none") or p.get("brief"):
        return None
    after = events[last + 1:]
    blocked = any(e.get("ev") == "blocked" for e in after)
    text = last_assistant_text(data)
    kind, detail = kb_line_kind(text)
    prev = idx[-2] if len(idx) > 1 else -1
    evidence = any(e.get("ev") == "query" for e in events[prev + 1:])
    unavail = bool(p.get("unavail"))
    tier = p["tier"]

    missing = []
    if unavail:
        if kind != "unavailable":
            missing.append("a closing `KB: unavailable (<reason>)` line (the knowledge base cannot be reached here)")
    elif kind == "unavailable":
        missing.append("a real KB query (the catalog is available here, so `KB: unavailable` is not accepted)")
    elif tier == "weak" and kind == "optout":
        pass
    else:
        if not evidence:
            missing.append("a knowledge base query (map, digest or find)")
        if kind != "queries":
            missing.append("a closing `KB: <queries run> (data <date>, roles <rules-only|merged>)` line naming them"
                           + ("" if tier == "weak" else " (`not applicable` is not accepted for this prompt)"))

    if not missing:
        if tier == "weak" and kind == "optout" and not unavail:
            append(os.path.join(sdir, "optouts.jsonl"), {
                "t": int(time.time()), "session": data.get("session_id"), "prompt": p.get("snippet", ""),
                "reason": detail,
            })
        return None

    # background work started for this prompt may still be reporting: recheck on the next stop
    bg = data.get("background_tasks")
    if isinstance(bg, list):
        running = [b for b in bg if not (isinstance(b, dict) and str(b.get("status", "")).lower() in
                                         ("completed", "failed", "killed", "cancelled", "done"))]
        if running and kind == "none":
            append(sf, {"ev": "deferred", "n": len(running)})
            return None

    if blocked:
        append(sf, {"ev": "unresolved", "missing": missing})
        append(os.path.join(sdir, "unresolved.jsonl"), {
            "t": int(time.time()), "session": data.get("session_id"), "prompt": p.get("snippet", ""),
            "missing": missing,
        })
        return {"systemMessage": "KB check: this card answer finished without the knowledge base (" +
                "; ".join(missing) + "). Logged as unresolved."}
    if data.get("stop_hook_active"):
        return None

    root = project_root(data)
    cg = os.path.join(root, "scripts", "cardgen")
    reason = (
        "Card knowledge base check failed: this answer is missing " + " and ".join(missing) + ".\n"
        + ("Run, for example: `" + cg + " map --stdout`, then `" + cg + " digest --deck KEY --stdout` (a deck of JD's) or "
           "`digest --ident LETTERS --stdout` / `find --ident LETTERS ...` (a new deck; C = colourless), and "
           "`find --name NAME --full` for every card or commander you named.\n" if not unavail else "")
        + "If anything in the answer you gave is wrong or unverified, post a corrected answer and say what changed. "
        + "End with the closing `KB:` line. (Asked once per question.)"
    )
    append(sf, {"ev": "blocked", "missing": missing})
    return {"decision": "block", "reason": reason}


# ---------------------------------------------------------------------------------------------- events


def inject_text(root, tier, decks_note):
    cg = os.path.join(root, "scripts", "cardgen")
    closing = "`KB: <queries run> (data <date>, roles <rules-only|merged>)`"
    if tier == "weak":
        closing += ", or `KB: not applicable (<reason>)` only if the answer names, ranks or evaluates no card"
    return (
        "KB rule (CLAUDE.md, Card recommendations): this looks like a card, commander, deck or strategy question. "
        "Answer from the card knowledge base, not memory.\n"
        f"1. Run `{cg} map --stdout`; for one of JD's decks{decks_note}`digest --deck KEY --stdout`; for a new or "
        "hypothetical deck `digest --ident LETTERS --stdout` and `find --ident LETTERS ...` (C = colourless, e.g. "
        "`find --ident C --text 'can be your commander'`).\n"
        "2. `find --name NAME --full` for every card or commander you name as a pick, example or fact.\n"
        f"3. End the answer with {closing}.\n"
        "A Stop hook checks for the queries and the KB line once per question."
    )


def ev_prompt(data):
    source = data.get("source")
    prompt = data.get("prompt")
    if source not in (None, "user", "sdk") or not isinstance(prompt, str):
        return None
    if re.match(r"\s*<(task-notification|system-reminder)", prompt):
        return None
    prune()
    root = project_root(data)
    decks = deck_words(root)
    roots = [root] + [m for m in [main_checkout(root)] if m and m != root]
    tier, brief = classify(prompt, decks, load_names(roots))
    avail, why = kb_status(root) if tier != "none" else (True, "")
    append(session_file(data), {
        "ev": "prompt", "pid": data.get("prompt_id"), "t": int(time.time()), "tier": tier, "brief": brief,
        "unavail": not avail, "snippet": prompt[:160],
    })
    if tier == "none":
        return None
    if not avail:
        ctx = (f"KB unavailable here ({why}). Tell JD, mark any card answer as from memory and unverified, and end "
               "with `KB: unavailable (<reason>)`. Do not build cardgen without JD's approval.")
    else:
        ctx = inject_text(root, tier, " (keys in data/my_decks.toml) ")
    return {"hookSpecificOutput": {"hookEventName": "UserPromptSubmit", "additionalContext": ctx}}


def ev_tool(data):
    if tool_is_query(data):
        inp = data.get("tool_input") or {}
        append(session_file(data), {
            "ev": "query", "pid": data.get("prompt_id"), "agent": data.get("agent_id"),
            "cmd": str(inp.get("command") or inp.get("file_path") or "")[:200], "t": int(time.time()),
        })
    return None


def ev_stop(data):
    events = load(session_file(data))
    return stop_decision(data, events, state_dir())


def ev_report(_data):
    d = state_dir()
    tiers = {"strong": 0, "weak": 0, "none": 0}
    count = {"brief": 0, "query": 0, "blocked": 0, "unresolved": 0, "deferred": 0}
    sessions = 0
    try:
        names = [n for n in os.listdir(d) if n.endswith(".jsonl") and n not in LOGS]
    except FileNotFoundError:
        names = []
    for n in names:
        sessions += 1
        for e in load(os.path.join(d, n)):
            ev = e.get("ev")
            if ev == "prompt":
                tiers[e.get("tier", "none")] = tiers.get(e.get("tier", "none"), 0) + 1
                count["brief"] += 1 if e.get("brief") else 0
            elif ev in count:
                count[ev] += 1
    optouts = len(load(os.path.join(d, "optouts.jsonl")))
    print(f"kb_guard: {sessions} sessions; prompts strong {tiers['strong']}, weak {tiers['weak']}, none {tiers['none']}, "
          f"briefs {count['brief']}; queries {count['query']}; blocked {count['blocked']}; unresolved "
          f"{count['unresolved']}; deferred {count['deferred']}; opt-outs {optouts}")
    return None


HANDLERS = {"prompt": ev_prompt, "tool": ev_tool, "stop": ev_stop, "report": ev_report}


def run(argv, stdin_text):
    """Return the text to print on stdout (empty for nothing). Raises on bugs; main() turns that into exit 0."""
    event = argv[1] if len(argv) > 1 else ""
    if event not in HANDLERS:
        return ""
    if event == "report":
        HANDLERS[event]({})
        return ""
    try:
        data = json.loads(stdin_text)
    except Exception:
        return ""
    if not isinstance(data, dict):
        return ""
    out = HANDLERS[event](data)
    return json.dumps(out) if out else ""


def main():
    try:
        text = run(sys.argv, sys.stdin.read() if sys.argv[1:2] != ["report"] else "")
        if text:
            sys.stdout.write(text + "\n")
    except Exception as e:  # fail open: never block a turn because the guard is broken
        sys.stderr.write(f"kb_guard: {type(e).__name__}: {e} (ignored)\n")
    sys.exit(0)


if __name__ == "__main__":
    main()
