"""Fixtures for scripts/hooks/kb_guard.py. Run: python3 -m unittest discover -s scripts/hooks/tests -v"""
import json
import os
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
GUARD = os.path.join(os.path.dirname(HERE), "kb_guard.py")
sys.path.insert(0, os.path.dirname(HERE))
import kb_guard  # noqa: E402

COLORLESS = "if I wanted to make a colorless commander deck, what strategies would be some options"
GOOD_LINE = "KB: map, digest --ident C, find --name Kaldra (data 2026-11-22, roles rules-only)"


class Env:
    """A temp project (with a catalog and a cardgen stand-in) and a temp state dir."""

    def __init__(self, available=True):
        self.tmp = tempfile.TemporaryDirectory()
        self.proj = os.path.join(self.tmp.name, "proj")
        self.state = os.path.join(self.tmp.name, "state")
        os.makedirs(os.path.join(self.proj, "data", "knowledge"))
        os.makedirs(os.path.join(self.proj, "rust", "target", "release"))
        if available:
            open(os.path.join(self.proj, "data", "knowledge", "oracle.json"), "w").close()
            b = os.path.join(self.proj, "rust", "target", "release", "cardgen")
            open(b, "w").close()
            os.chmod(b, 0o755)
        with open(os.path.join(self.proj, "data", "my_decks.toml"), "w") as f:
            f.write('[sheoldred]\nname = "Sheoldred"\n[arzakon]\nname = "Arzakon"\n')
        self.sid = "sess1"

    def run(self, event, payload, raw=None):
        env = dict(os.environ, CLAUDE_PROJECT_DIR=self.proj, KB_GUARD_STATE_DIR=self.state)
        data = dict(payload)
        data.setdefault("session_id", self.sid)
        data.setdefault("cwd", self.proj)
        stdin = raw if raw is not None else json.dumps(data)
        p = subprocess.run([sys.executable, GUARD, event], input=stdin, capture_output=True, text=True, env=env)
        return p.returncode, p.stdout, p.stderr

    def prompt(self, text, **kw):
        return self.run("prompt", dict({"hook_event_name": "UserPromptSubmit", "prompt": text}, **kw))

    def bash(self, cmd, **kw):
        return self.run("tool", dict({"hook_event_name": "PostToolUse", "tool_name": "Bash",
                                      "tool_input": {"command": cmd}, "tool_response": {"stdout": "x"}}, **kw))

    def stop(self, msg, **kw):
        return self.run("stop", dict({"hook_event_name": "Stop", "last_assistant_message": msg}, **kw))

    def events(self):
        path = os.path.join(self.state, self.sid + ".jsonl")
        if not os.path.exists(path):
            return []
        with open(path) as f:
            return [json.loads(x) for x in f]

    def cleanup(self):
        self.tmp.cleanup()


class Base(unittest.TestCase):
    def setUp(self):
        self.e = Env()
        self.addCleanup(self.e.cleanup)

    def assertBlocks(self, res):
        code, out, _ = res
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(out).get("decision"), "block")

    def assertPasses(self, res):
        code, out, _ = res
        self.assertEqual(code, 0)
        if out.strip():
            self.assertNotEqual(json.loads(out).get("decision"), "block")


class ClassifierTests(unittest.TestCase):
    def test_fixture_is_strong(self):
        self.assertEqual(kb_guard.classify(COLORLESS)[0], "strong")

    def test_tiers(self):
        c = lambda s: kb_guard.classify(s, ["sheoldred"])[0]  # noqa: E731
        self.assertEqual(c("what should I cut from the sheoldred deck"), "strong")
        self.assertEqual(c("is Sol Ring good in my commander decks"), "weak")
        self.assertEqual(c("fix the failing cargo test in rust/"), "none")
        self.assertEqual(c("rename card-knowledge.md section"), "none")

    def test_brief_exempt(self):
        t, b = kb_guard.classify("write results to /tmp/handoff/x.md; recommend swaps for the deck")
        self.assertEqual((t, b), ("none", True))
        t, b = kb_guard.classify("GOAL: recommend a card for the deck. " + "x" * 4100)
        self.assertEqual((t, b), ("none", True))


class ToolDetection(unittest.TestCase):
    def test_cases(self):
        q = kb_guard.bash_is_query
        self.assertTrue(q("scripts/cardgen map --stdout | head -3"))
        self.assertTrue(q("./rust/target/release/cardgen find --name 'Sol Ring' --full"))
        self.assertTrue(q("C=./rust/target/release/cardgen; $C find --ident C --text 'x'"))
        self.assertTrue(q("cd rust && cargo run -p cardgen -- digest --deck sheoldred --stdout"))
        self.assertTrue(q("scripts/cardgen --root /x find --role ramp"))
        self.assertFalse(q("scripts/cardgen find --help"))
        self.assertFalse(q("scripts/cardgen digest -h"))
        self.assertFalse(q("scripts/cardgen find"))
        self.assertFalse(q("scripts/cardgen digest"))
        self.assertFalse(q("scripts/cardgen audit --deck x"))
        self.assertFalse(q("find rust/crates/cardgen -name '*.rs'"))
        self.assertFalse(q("grep map rust/crates/cardgen/src/main.rs"))


class HookFlow(Base):
    def test_prompt_injection_and_fixture(self):
        code, out, _ = self.e.prompt(COLORLESS)
        self.assertEqual(code, 0)
        ctx = json.loads(out)["hookSpecificOutput"]["additionalContext"]
        self.assertIn("scripts/cardgen", ctx)
        self.assertIn("KB:", ctx)
        self.assertLessEqual(len(ctx.splitlines()), 6)
        self.assertEqual(self.e.events()[0]["tier"], "strong")

    def test_compliant(self):
        self.e.prompt(COLORLESS)
        self.e.bash("scripts/cardgen map --stdout")
        self.assertPasses(self.e.stop("Options: ...\n\n" + GOOD_LINE))
        self.assertFalse([x for x in self.e.events() if x["ev"] == "blocked"])

    def test_noncompliant_blocks_once_then_unresolved(self):
        self.e.prompt(COLORLESS)
        r = self.e.stop("From memory: Kozilek.")
        self.assertBlocks(r)
        reason = json.loads(r[1])["reason"]
        self.assertIn("scripts/cardgen", reason)
        self.assertIn("what changed", reason)
        code, out, _ = self.e.stop("Still from memory.")
        self.assertEqual(code, 0)
        self.assertNotIn("decision", json.loads(out))
        self.assertIn("systemMessage", json.loads(out))
        self.assertTrue([x for x in self.e.events() if x["ev"] == "unresolved"])
        # third stop: still no re-block
        self.assertPasses(self.e.stop("Again."))

    def test_notification_turn_after_block_does_not_reblock(self):
        self.e.prompt(COLORLESS)
        self.assertBlocks(self.e.stop("memory"))
        # notification wakes the model: UserPromptSubmit with source=system is ignored, stop_hook_active is false
        self.e.prompt("<task-notification>done</task-notification>", source="system")
        self.assertPasses(self.e.stop("A note.", stop_hook_active=False))
        self.assertEqual(len([x for x in self.e.events() if x["ev"] == "prompt"]), 1)

    def test_line_only_and_query_only_both_block(self):
        self.e.prompt(COLORLESS)
        r = self.e.stop(GOOD_LINE)  # claim without a query
        self.assertBlocks(r)
        e2 = Env()
        self.addCleanup(e2.cleanup)
        e2.prompt(COLORLESS)
        e2.bash("scripts/cardgen map --stdout")
        self.assertBlocks(e2.stop("answer with no KB line"))

    def test_weak_optout_passes_and_is_logged(self):
        self.e.prompt("is Sol Ring good in my commander decks")
        self.assertEqual(self.e.events()[0]["tier"], "weak")
        self.assertPasses(self.e.stop("It is fine.\nKB: not applicable (general rules chat only)"))
        self.assertTrue(os.path.exists(os.path.join(self.e.state, "optouts.jsonl")))

    def test_weak_optout_without_reason_blocks(self):
        self.e.prompt("is Sol Ring good in my commander decks")
        self.assertBlocks(self.e.stop("KB: not applicable"))

    def test_strong_optout_blocks(self):
        self.e.prompt(COLORLESS)
        self.assertBlocks(self.e.stop("x\nKB: not applicable (no cards were named)"))

    def test_unavailable(self):
        e = Env(available=False)
        self.addCleanup(e.cleanup)
        code, out, _ = e.prompt(COLORLESS)
        self.assertIn("KB unavailable", json.loads(out)["hookSpecificOutput"]["additionalContext"])
        self.assertPasses(e.stop("From memory, unverified.\nKB: unavailable (no catalog in this checkout)"))
        e2 = Env(available=False)
        self.addCleanup(e2.cleanup)
        e2.prompt(COLORLESS)
        self.assertBlocks(e2.stop("no line at all"))

    def test_unavailable_not_accepted_when_catalog_present(self):
        self.e.prompt(COLORLESS)
        self.assertBlocks(self.e.stop("KB: unavailable (lazy)"))

    def test_brief_exempt(self):
        code, out, _ = self.e.prompt("GOAL: swap cards in the deck. Write results to /tmp/handoff/x.md")
        self.assertEqual((code, out.strip()), (0, ""))
        self.assertPasses(self.e.stop("/tmp/handoff/x.md"))
        self.assertFalse([x for x in self.e.events() if x["ev"] == "blocked"])

    def test_background_deferral(self):
        self.e.prompt(COLORLESS)
        r = self.e.stop("The digest is running in the background, will report.",
                        background_tasks=[{"id": "b1", "status": "running"}])
        self.assertPasses(r)
        self.assertTrue([x for x in self.e.events() if x["ev"] == "deferred"])
        self.assertFalse([x for x in self.e.events() if x["ev"] == "blocked"])
        # the later stop, with nothing running, is checked
        self.assertBlocks(self.e.stop("Here is the answer."))

    def test_indirection_counts(self):
        self.e.prompt(COLORLESS)
        self.e.bash("C=./rust/target/release/cardgen; $C find --ident C --text 'can be your commander'")
        self.assertPasses(self.e.stop(GOOD_LINE))

    def test_find_help_rejected(self):
        self.e.prompt(COLORLESS)
        self.e.bash("scripts/cardgen find --help")
        self.assertFalse([x for x in self.e.events() if x["ev"] == "query"])
        self.assertBlocks(self.e.stop(GOOD_LINE))

    def test_failed_calls_rejected(self):
        self.e.prompt(COLORLESS)
        self.e.run("tool", {"hook_event_name": "PostToolUse", "tool_name": "Bash",
                            "tool_input": {"command": "scripts/cardgen map"}, "tool_response": {"exit_code": 1}})
        self.e.run("tool", {"hook_event_name": "PostToolUseFailure", "tool_name": "Bash",
                            "tool_input": {"command": "scripts/cardgen map"}})
        self.assertFalse([x for x in self.e.events() if x["ev"] == "query"])

    def test_subagent_query_counts(self):
        self.e.prompt(COLORLESS)
        self.e.bash("scripts/cardgen digest --ident C --stdout", agent_id="agent-7")
        ev = [x for x in self.e.events() if x["ev"] == "query"]
        self.assertEqual(ev[0]["agent"], "agent-7")
        self.assertPasses(self.e.stop(GOOD_LINE))

    def test_read_of_digest_file_counts(self):
        self.e.prompt(COLORLESS)
        self.e.run("tool", {"hook_event_name": "PostToolUse", "tool_name": "Read",
                            "tool_input": {"file_path": "/x/data/knowledge/digests/sheoldred.txt"}, "tool_response": {}})
        self.assertPasses(self.e.stop(GOOD_LINE))

    def test_followup_uses_previous_prompt_queries(self):
        self.e.prompt(COLORLESS)
        self.e.bash("scripts/cardgen map --stdout")
        self.assertPasses(self.e.stop(GOOD_LINE))
        self.e.prompt("ok, which would you cut from the options")
        self.assertPasses(self.e.stop("Cut this.\nKB: map and digest from the previous answer (data 2026-11-22, roles rules-only)"))

    def test_source_filtering(self):
        for s in ("system", "poll_event", "loop_wakeup", "schedule_wakeup"):
            code, out, _ = self.e.prompt(COLORLESS, source=s)
            self.assertEqual((code, out.strip()), (0, ""))
        self.assertEqual(self.e.events(), [])
        self.e.prompt(COLORLESS, source="sdk")
        self.assertEqual(len(self.e.events()), 1)

    def test_transcript_fallback(self):
        self.e.prompt(COLORLESS)
        self.e.bash("scripts/cardgen map")
        tp = os.path.join(self.e.tmp.name, "t.jsonl")
        with open(tp, "w") as f:
            f.write(json.dumps({"type": "assistant", "message": {"content": [{"type": "text", "text": GOOD_LINE}]}}) + "\n")
        self.assertPasses(self.e.run("stop", {"hook_event_name": "Stop", "transcript_path": tp}))

    def test_malformed_stdin(self):
        for ev in ("prompt", "tool", "stop"):
            self.assertEqual(self.e.run(ev, {}, raw="{not json")[:2], (0, ""))
            self.assertEqual(self.e.run(ev, {}, raw="")[:2], (0, ""))
            self.assertEqual(self.e.run(ev, {}, raw="[1,2]")[:2], (0, ""))

    def test_prune_old_state(self):
        os.makedirs(self.e.state)
        old = os.path.join(self.e.state, "old.jsonl")
        open(old, "w").close()
        t = os.path.getmtime(old) - 15 * 86400
        os.utime(old, (t, t))
        self.e.prompt(COLORLESS)
        self.assertFalse(os.path.exists(old))

    def test_report(self):
        self.e.prompt(COLORLESS)
        env = dict(os.environ, KB_GUARD_STATE_DIR=self.e.state)
        p = subprocess.run([sys.executable, GUARD, "report"], capture_output=True, text=True, env=env)
        self.assertEqual(p.returncode, 0)
        self.assertIn("strong 1", p.stdout)


if __name__ == "__main__":
    unittest.main()
