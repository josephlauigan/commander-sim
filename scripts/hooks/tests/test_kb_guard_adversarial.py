"""Adversarial fixtures. Run: python3 -m unittest scripts.hooks.tests.test_kb_guard_adversarial -v.

Failures describe desired behavior. Only accepted limits are marked expectedFailure, each with a reason comment.
Shell query strings are hook payloads, never executed. Wrapper tests execute only
stand-ins, except for a read-only invocation of the existing binary with --help.
"""
import concurrent.futures
import importlib.util
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
import unittest


ROOT = Path(__file__).resolve().parents[3]
GUARD = ROOT / "scripts/hooks/kb_guard.py"
WRAPPER = ROOT / "scripts/cardgen"
spec = importlib.util.spec_from_file_location("kb_guard_under_test", GUARD)
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)
COLORLESS = "if I wanted to make a colorless commander deck, what strategies would be some options"
GOOD = "KB: map, digest --ident C (data 2026-11-22, roles rules-only)"
WEAK = "compare commander decks"


class Env:
    """Like the existing Env helper, with both state locations isolated."""

    def __init__(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="kb-adv-")
        self.base = Path(self.tmp.name)
        self.proj = self.base / "project"
        self.state = self.base / "state"
        self.oracle = self.proj / "data/knowledge/oracle.json"
        self.oracle.parent.mkdir(parents=True)
        self.oracle.write_text(json.dumps({"cards": [
            {"name": "Example Adept"}, {"name": "Inventor's Construct"},
        ]}), encoding="utf-8")
        self.binary = self.proj / "rust/target/release/cardgen"
        self.binary.parent.mkdir(parents=True)
        self.binary.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        self.binary.chmod(0o755)
        (self.proj / "data/my_decks.toml").write_text('[example]\nname = "Example"\n', encoding="utf-8")
        self.env = dict(os.environ, CLAUDE_PROJECT_DIR=str(self.proj),
                        KB_GUARD_STATE_DIR=str(self.state), XDG_STATE_HOME=str(self.base / "xdg"),
                        PYTHONDONTWRITEBYTECODE="1")
        self.sid = "session-one"
        self.pid = "prompt-one"

    @property
    def sf(self):
        return self.state / (guard.safe_id(self.sid) + ".jsonl")

    def run(self, event, payload=None, raw=None, env=None):
        data = {"session_id": self.sid, "prompt_id": self.pid, "cwd": str(self.proj)}
        data.update(payload or {})
        stdin = json.dumps(data).encode() if raw is None else raw
        child_env = dict(self.env)
        for key, value in (env or {}).items():
            if value is None:
                child_env.pop(key, None)
            else:
                child_env[key] = str(value)
        proc = subprocess.run([sys.executable, str(GUARD), event], input=stdin,
                              capture_output=True, env=child_env, timeout=15)
        return proc.returncode, proc.stdout.decode("utf-8"), proc.stderr.decode("utf-8", errors="replace")

    def prompt(self, text=COLORLESS, **kwargs):
        return self.run("prompt", {"prompt": text, **kwargs})

    def bash(self, command="scripts/cardgen map", response=None, **kwargs):
        return self.run("tool", {"hook_event_name": "PostToolUse", "tool_name": "Bash",
                                 "tool_input": {"command": command},
                                 "tool_response": {"stdout": "catalog"} if response is None else response,
                                 **kwargs})

    def stop(self, text=GOOD, **kwargs):
        return self.run("stop", {"last_assistant_message": text, **kwargs})

    def events(self):
        if not self.sf.exists():
            return []
        return [json.loads(line) for line in self.sf.read_text(encoding="utf-8").splitlines()]

    def append(self, value):
        self.state.mkdir(exist_ok=True)
        with self.sf.open("a", encoding="utf-8") as stream:
            stream.write(json.dumps(value) + "\n")


class Base(unittest.TestCase):
    def setUp(self):
        self.e = Env()
        self.addCleanup(self.e.tmp.cleanup)

    def protocol(self, result):
        code, out, err = result
        self.assertEqual(code, 0, (out, err))
        if not out.strip():
            return {}
        value = json.loads(out)
        self.assertIsInstance(value, dict)
        self.assertNotIn("Traceback", out)
        return value

    def silent(self, result):
        self.assertEqual(self.protocol(result), {}, "must neither block nor warn")

    def blocked(self, result):
        self.assertEqual(self.protocol(result).get("decision"), "block", "expected Stop to block")

    def queries(self):
        return [event for event in self.e.events() if event["ev"] == "query"]


class StopStateTests(Base):
    def test_passed_answer_then_notification_does_not_reblock(self):
        self.e.prompt()
        self.e.bash()
        self.silent(self.e.stop())
        self.silent(self.e.prompt("<task-notification>finished</task-notification>"))
        self.silent(self.e.stop("Background work finished.", stop_hook_active=False))
        self.assertFalse([e for e in self.e.events() if e["ev"] in ("blocked", "unresolved")])

    def test_corrected_answer_then_notification_does_not_warn(self):
        self.e.prompt()
        self.blocked(self.e.stop("Unverified answer."))
        self.e.bash()
        self.silent(self.e.stop(GOOD, stop_hook_active=True))
        self.silent(self.e.prompt("<task-notification>finished</task-notification>"))
        self.silent(self.e.stop("Background work finished.", stop_hook_active=False))
        self.assertFalse([e for e in self.e.events() if e["ev"] == "unresolved"])

    def test_uncorrected_answer_blocks_only_once(self):
        self.e.prompt()
        self.blocked(self.e.stop("Unverified answer."))
        result = self.protocol(self.e.stop("Still unverified.", stop_hook_active=False))
        self.assertNotEqual(result.get("decision"), "block")
        self.assertIn("systemMessage", result)

    def test_only_immediately_previous_prompt_supplies_evidence(self):
        self.e.prompt()
        self.e.bash()
        self.e.pid = "prompt-two"
        self.e.prompt()
        self.silent(self.e.stop())
        self.e.pid = "prompt-three"
        self.e.prompt()
        self.blocked(self.e.stop())

    def test_delayed_query_from_two_prompts_ago_is_not_current_evidence(self):
        self.e.prompt()
        self.e.pid = "prompt-two"
        self.e.prompt("git status")
        self.e.pid = "prompt-three"
        self.e.prompt()
        self.e.bash(prompt_id="prompt-one")
        self.blocked(self.e.stop())

    def test_future_prompt_query_is_not_current_evidence(self):
        self.e.prompt()
        self.e.bash(prompt_id="unrelated-prompt")
        self.blocked(self.e.stop())

    def test_weak_optout_requires_three_words_and_logs(self):
        self.e.prompt(WEAK)
        self.silent(self.e.stop("KB: not applicable (only scheduling simulation work)"))
        rows = (self.e.state / "optouts.jsonl").read_text().splitlines()
        self.assertEqual(len(rows), 1)
        self.assertEqual(json.loads(rows[0])["reason"], "only scheduling simulation work")

    def test_available_catalog_rejects_unavailable_claim(self):
        self.e.prompt()
        self.e.bash()
        self.blocked(self.e.stop("KB: unavailable (no catalog here)"))

    def test_missing_catalog_accepts_unavailable_claim(self):
        self.e.oracle.unlink()
        self.protocol(self.e.prompt())
        self.silent(self.e.stop("KB: unavailable (catalog is missing)"))

    def test_finished_background_task_does_not_defer(self):
        self.e.prompt()
        self.blocked(self.e.stop("Done.", background_tasks=[{"status": "completed"}]))

    def test_running_background_task_defers_then_rechecks(self):
        self.e.prompt()
        self.silent(self.e.stop("Waiting.", background_tasks=[{"status": "running"}]))
        self.blocked(self.e.stop("Done.", background_tasks=[]))

    def test_fallback_uses_final_assistant_message_only(self):
        self.e.prompt()
        self.e.bash()
        transcript = self.e.base / "transcript.jsonl"
        transcript.write_text("\n".join(json.dumps({"type": "assistant", "message": {"content": text}})
                                         for text in (GOOD, "Done.")) + "\n")
        self.blocked(self.e.run("stop", {"transcript_path": str(transcript)}))


LINE_CASES = {
    "strong_optout": ("KB: not applicable (no card advice here)", True, COLORLESS),
    "weak_two_word_optout": ("KB: not applicable (sim scheduling)", True, WEAK),
    "weak_empty_optout": ("KB: not applicable ()", True, WEAK),
    "no_query_names": ("KB: consulted the catalog (data 2026-11-22, roles rules-only)", True, COLORLESS),
    "fenced_line": ("Example:\n```text\n" + GOOD + "\n```", True, COLORLESS),
    "blockquote_line": ("> " + GOOD, True, COLORLESS),
    "middle_line": (GOOD + "\n\nThe actual advice follows here.", True, COLORLESS),
    "earlier_paragraph_done": ("Advice.\n\n" + GOOD + "\n\nDone.", True, COLORLESS),
    "bold_closing_line": ("Advice.\n**KB:** map (data 2026-11-22, roles rules-only)", False, COLORLESS),
    "italic_closing_line": ("Advice.\n_KB:_ map (data 2026-11-22, roles rules-only)", False, COLORLESS),
    "plain_closing_line": ("Advice.\n" + GOOD + "\n\n", False, COLORLESS),
}


def line_test(text, should_block, prompt):
    def test(self):
        self.e.prompt(prompt)
        self.e.bash()
        (self.blocked if should_block else self.silent)(self.e.stop(text))
    test.__doc__ = repr(text)
    return test


class ClosingLineTests(Base):
    pass


for case, args in LINE_CASES.items():
    setattr(ClosingLineTests, "test_" + case, line_test(*args))


COMMAND_CASES = {
    "echo_is_not_execution": ('echo "cardgen map"', False),
    "comment_is_not_execution": ("# cardgen map", False),
    "git_log_is_not_execution": ("git log -- scripts/cardgen", False),
    "which_is_not_execution": ("which cardgen", False),
    "notes_are_not_execution": ("cat cardgen-notes.txt", False),
    "printed_find_is_not_execution": ("printf '%s' 'cardgen find --name X'", False),
    "after_and": ("true && cardgen map", True),
    "after_semicolon": ("true; cardgen map", True),
    "after_pipe": ("printf '' | cardgen map", True),
    "help_only": ("cardgen --help", False),
    "find_limit_json_only": ("cardgen find --limit 1 --json", False),
    "find_name_help": ("cardgen find --name X --help", False),
    "find_missing_filter_value": ("cardgen find --name", False),
    "digest_missing_ident_value": ("cardgen digest --ident", False),
    "script_path": ("scripts/cardgen map", True),
    "cargo_run_payload_only": ("cargo run -p cardgen -- map", True),
    "env_prefix": ("env FOO=1 cardgen map", True),
    "quoted_path": ('"/tmp/project/cardgen" map', True),
    "quoted_path_with_spaces": ('"/tmp/project with spaces/cardgen" map', True),
    "quoted_filter_help_word": ("cardgen find --text 'help'", True),
    "inline_variable": ("C=./rust/target/release/cardgen; $C find --ident C", True),
    "inline_variable_space_path": ('C="/tmp/project with spaces/cardgen"; "$C" find --ident C', True),
    "not_executed_or_branch": ("true || cardgen map", False),
    "not_executed_and_branch": ("false && cardgen map", False),
}


def command_test(command, expected):
    def test(self):
        self.e.prompt()
        # A short-circuited failing AND-list really exits 1; do not invent
        # a successful tool response for that shell program.
        response = None
        if command == "false && cardgen map":
            response = {"exit_code": 1, "stdout": ""}
        elif command == "true || cardgen map":
            response = {"exit_code": 0, "stdout": ""}
        self.protocol(self.e.bash(command, response=response))
        (self.silent if expected else self.blocked)(self.e.stop())
        self.assertEqual(bool(self.queries()), expected)
    test.__doc__ = command
    return test


class ToolCommandTests(Base):
    # accepted: shell variables do not persist between Bash calls, so `$C` set in an earlier call is not a runnable command
    @unittest.expectedFailure
    def test_variable_assigned_in_earlier_tool_call(self):
        # Bash tool shells do not necessarily persist variables; explicitly
        # supply a successful response, as if the caller retained its shell.
        self.e.prompt()
        self.e.bash("C=./rust/target/release/cardgen")
        self.e.bash("$C find --ident C", response={"exit_code": 0, "stdout": "catalog"})
        self.silent(self.e.stop())


# accepted: `a || cardgen map` cannot be told from a run branch by reading tokens
ACCEPTED_COMMAND_LIMITS = {"not_executed_or_branch"}
for case, args in COMMAND_CASES.items():
    t = command_test(*args)
    if case in ACCEPTED_COMMAND_LIMITS:
        t = unittest.expectedFailure(t)
    setattr(ToolCommandTests, "test_" + case, t)


FAILED_RESPONSES = {
    "exit_code": {"exit_code": 1}, "exitCode": {"exitCode": 1},
    "returncode": {"returncode": 1}, "returnCode": {"returnCode": 1},
    "code": {"code": 1}, "status": {"status": 1},
    "is_error": {"is_error": True}, "isError": {"isError": True},
    "success": {"success": False}, "interrupted": {"interrupted": True},
    "string": "Error: Exit code 1", "string_exit_code": {"exit_code": "1"},
    "stderr_only": {"stderr": "Error: Exit code 1", "stdout": ""},
    "list": [{"type": "text", "text": "Error: Exit code 1"}],
    "nested_content": {"content": [{"type": "text", "text": "Error: Exit code 1"}], "isError": True},
}


def failed_response_test(response):
    def test(self):
        self.e.prompt()
        self.protocol(self.e.bash(response=response))
        self.blocked(self.e.stop())
        self.assertFalse(self.queries())
    test.__doc__ = repr(response)
    return test


class ToolResponseTests(Base):
    def test_explicit_failure_event(self):
        self.e.prompt()
        self.e.bash(hook_event_name="PostToolUseFailure")
        self.blocked(self.e.stop())

    def test_stderr_warning_with_success_is_valid(self):
        self.e.prompt()
        self.e.bash(response={"exit_code": 0, "stdout": "catalog", "stderr": "warning: cache old"})
        self.silent(self.e.stop())

    def test_delegated_query_counts(self):
        self.e.prompt()
        self.e.bash(agent_id="delegate-one")
        self.silent(self.e.stop())
        self.assertEqual(self.queries()[0]["agent"], "delegate-one")


for case, response in FAILED_RESPONSES.items():
    setattr(ToolResponseTests, "test_failure_" + case, failed_response_test(response))


READ_CASES = {
    "map": ("data/knowledge/map.md", True),
    "digest": ("data/knowledge/digests/x.txt", True),
    "absolute_map": ("/tmp/project/data/knowledge/map.md", True),
    "backup_map": ("../data/knowledge/map.md.bak", False),
    "backup_digest": ("data/knowledge/digests/x.txt.old", False),
}


def read_test(path, expected):
    def test(self):
        self.e.prompt()
        self.e.run("tool", {"tool_name": "Read", "tool_input": {"file_path": path}, "tool_response": {}})
        (self.silent if expected else self.blocked)(self.e.stop())
    test.__doc__ = path
    return test


class ReadTests(Base):
    pass


for case, args in READ_CASES.items():
    setattr(ReadTests, "test_" + case, read_test(*args))


class ProtocolTests(Base):
    def test_malformed_empty_list_and_non_utf8(self):
        for event in ("prompt", "tool", "stop"):
            for raw in (b"{not json", b"", b"[1,2]", b"null", b"\xff\xfe"):
                with self.subTest(event=event, raw=raw):
                    self.silent(self.e.run(event, raw=raw))

    def test_one_megabyte_prompt(self):
        try:
            result = self.e.prompt(COLORLESS + " " + "x" * 1_000_000)
        except subprocess.TimeoutExpired:
            self.fail("1 MB prompt did not return a fail-open protocol response within 15 seconds")
        self.protocol(result)
        self.assertEqual(self.e.events()[-1]["tier"], "strong")

    def test_one_megabyte_prompt_with_word_boundaries(self):
        self.protocol(self.e.prompt(COLORLESS + " " + "x " * 500_000))
        self.assertEqual(self.e.events()[-1]["tier"], "strong")

    def test_missing_session_id(self):
        raw = json.dumps({"prompt": COLORLESS, "cwd": str(self.e.proj)}).encode()
        self.protocol(self.e.run("prompt", raw=raw))
        self.assertTrue((self.e.state / "nosession.jsonl").is_file())

    def test_path_traversal_and_nul_session_ids(self):
        for sid in ("../../escape", "..", "a\x00b", "/tmp/escape"):
            with self.subTest(sid=sid):
                self.protocol(self.e.prompt(session_id=sid))
        self.assertFalse((self.e.base / "escape.jsonl").exists())
        self.assertTrue(all(p.parent == self.e.state for p in self.e.state.rglob("*.jsonl")))

    def test_unwritable_state_directory(self):
        self.e.state.mkdir(mode=0o500)
        self.addCleanup(self.e.state.chmod, 0o700)
        # The effective-uid probe also makes this deterministic under root.
        if os.access(self.e.state, os.W_OK):
            code = ("import runpy, sys\nfrom unittest.mock import patch\n"
                    "sys.argv = [" + repr(str(GUARD)) + ", 'prompt']\n"
                    "with patch('os.open', side_effect=PermissionError('fixture')):\n"
                    "    runpy.run_path(" + repr(str(GUARD)) + ", run_name='__main__')\n")
            proc = subprocess.run([sys.executable, "-c", code],
                                  input=json.dumps({"prompt": COLORLESS, "session_id": self.e.sid}).encode(),
                                  capture_output=True, env=self.e.env, timeout=15)
            self.silent((proc.returncode, proc.stdout.decode(), proc.stderr.decode()))
            return
        self.silent(self.e.prompt())
        self.silent(self.e.stop("Done."))

    def test_state_path_is_a_file(self):
        self.e.state.write_text("not a directory")
        self.silent(self.e.prompt())
        self.silent(self.e.stop("Done."))

    def test_missing_state_directory_is_created(self):
        self.protocol(self.e.prompt())
        self.assertTrue(self.e.sf.is_file())

    def test_state_file_is_directory(self):
        self.e.sf.mkdir(parents=True)
        self.silent(self.e.prompt())
        self.silent(self.e.stop("Done."))

    def test_unreadable_catalog_protocol(self):
        self.e.oracle.chmod(0)
        self.addCleanup(self.e.oracle.chmod, 0o600)
        self.protocol(self.e.prompt())
        self.protocol(self.e.stop("KB: unavailable (catalog cannot be read)"))

    def test_project_environment_unset(self):
        self.protocol(self.e.run("prompt", {"prompt": COLORLESS}, env={"CLAUDE_PROJECT_DIR": None}))
        self.assertEqual(self.e.events()[-1]["tier"], "strong")

    def test_nonexistent_cwd(self):
        self.protocol(self.e.run("prompt", {"prompt": COLORLESS, "cwd": str(self.e.base / "missing")},
                                 env={"CLAUDE_PROJECT_DIR": None}))
        self.silent(self.e.stop("KB: unavailable (directory does not exist)"))

    def test_git_missing_from_path(self):
        self.protocol(self.e.run("prompt", {"prompt": COLORLESS}, env={"PATH": self.e.base / "no-path"}))
        self.e.bash()
        self.silent(self.e.stop())

    def test_xdg_fallback_stays_in_temp(self):
        self.protocol(self.e.run("prompt", {"prompt": COLORLESS}, env={"KB_GUARD_STATE_DIR": None}))
        self.assertTrue((self.e.base / "xdg/commander-sim/kb/session-one.jsonl").exists())


CLASSIFIER_CASES = {
    "colorless_fixture": (COLORLESS, "strong"),
    "rust_tests": ("run the rust tests", "none"),
    "borrow_error": ("fix the borrow error in sim-core", "none"),
    "git_status": ("git status", "none"),
    "card_name_in_path": ("replace /tmp/Example-Adept.rs with the checked-in file", "none"),
    "card_name_in_code": ("replace `Example Adept` in this unit test", "none"),
    "curly_apostrophe": ("Can Inventor’s Construct be your commander?", "weak"),
    "ascii_apostrophe": ("Can Inventor's Construct be your commander?", "weak"),
    "long_nonbrief": ("x " * 3000 + COLORLESS, "strong"),
}


def classifier_test(prompt, tier):
    def test(self):
        self.protocol(self.e.prompt(prompt))
        self.assertEqual(self.e.events()[-1]["tier"], tier)
    test.__doc__ = prompt if len(prompt) < 200 else "long non-brief prompt"
    return test


class ClassifierTests(Base):
    def test_incidental_handoff_mention_does_not_exempt_advice(self):
        self.e.prompt(COLORLESS + "; the old log happens to live in /tmp/handoff/old.md")
        self.blocked(self.e.stop("Advice with no KB."))

    # accepted: text that starts with <task-notification> is treated as a harness notification whatever the source says; no human types it
    @unittest.expectedFailure
    def test_human_notification_looking_text_does_not_exempt_advice(self):
        self.e.prompt("<task-notification>this is my question</task-notification> " + COLORLESS,
                      source="user")
        self.blocked(self.e.stop("Advice with no KB."))

    def test_real_brief_remains_exempt(self):
        self.silent(self.e.prompt("GOAL: " + COLORLESS + "\nSCOPE: tests\nDONE WHEN: passing\n" + "x" * 4100))
        self.silent(self.e.stop("/tmp/handoff/result.md"))

    def test_source_filtering(self):
        for source in ("system", "poll_event", "loop_wakeup", "schedule_wakeup"):
            with self.subTest(source=source):
                self.silent(self.e.prompt(source=source))
        self.assertEqual(self.e.events(), [])
        self.protocol(self.e.prompt(source="sdk"))
        self.assertEqual(self.e.events()[-1]["tier"], "strong")


for case, args in CLASSIFIER_CASES.items():
    setattr(ClassifierTests, "test_" + case, classifier_test(*args))


class StateTests(Base):
    def test_twenty_parallel_tool_events_preserve_complete_lines(self):
        self.e.prompt()
        with concurrent.futures.ThreadPoolExecutor(max_workers=20) as pool:
            futures = [pool.submit(self.e.bash, agent_id="agent-%02d" % i) for i in range(20)]
            for future in futures:
                self.silent(future.result())
        self.assertEqual(len(self.queries()), 20)
        self.assertEqual({e["agent"] for e in self.queries()}, {"agent-%02d" % i for i in range(20)})
        self.silent(self.e.stop())

    def test_corrupt_line_between_prompt_and_query_is_ignored(self):
        self.e.prompt()
        with self.e.sf.open("a") as stream:
            stream.write("{corrupt line\n")
        self.e.bash()
        self.silent(self.e.stop())

    def test_valid_json_nonobject_line_does_not_disable_enforcement(self):
        self.e.prompt()
        self.e.append([])
        self.blocked(self.e.stop("No KB."))

    # accepted: a state file idle for 15+ days is pruned; the next prompt starts a fresh record, so only a Stop with no prompt event is missed
    @unittest.expectedFailure
    def test_prune_does_not_erase_live_old_session(self):
        self.e.prompt()
        old = time.time() - 15 * 86400
        os.utime(self.e.sf, (old, old))
        self.e.prompt("git status", session_id="another-session")
        self.blocked(self.e.stop("No KB."))

    def test_short_shared_session_prefix_is_isolated(self):
        self.e.prompt()
        self.e.bash()
        self.e.sid = "session-two"
        self.e.prompt()
        self.blocked(self.e.stop())

    def test_long_shared_session_prefix_is_isolated(self):
        self.e.sid = "s" * 100 + "-one"
        self.e.prompt()
        self.e.bash()
        self.e.sid = "s" * 100 + "-two"
        self.e.prompt()
        self.blocked(self.e.stop())

    def test_sanitized_session_ids_do_not_collide(self):
        self.e.sid = "session/a"
        self.e.prompt()
        self.e.bash()
        self.e.sid = "session_a"
        self.e.prompt()
        self.blocked(self.e.stop())


class WrapperTests(Base):
    def setUp(self):
        super().setUp()
        self.checkout = self.e.base / "checkout with spaces"
        self.wrapper = self.checkout / "scripts/cardgen"
        self.wrapper.parent.mkdir(parents=True)
        shutil.copy2(WRAPPER, self.wrapper)
        self.main = self.e.base / "main checkout"
        (self.main / ".git").mkdir(parents=True)
        self.tools = self.e.base / "tools"
        self.tools.mkdir()
        # Fixed fake git output makes fallback fixtures independent of git config.
        self.fake_git(str(self.main / ".git"))
        self.wrapper_env = dict(self.e.env, PATH=str(self.tools) + os.pathsep + os.defpath)

    def executable(self, path, text):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        path.chmod(0o755)

    def fake_git(self, common):
        self.executable(self.tools / "git", "#!/bin/sh\nprintf '%s\\n' " + shlex.quote(common) + "\n")

    def standin(self, root):
        path = root / "rust/target/release/cardgen"
        code = "import json,sys; print(json.dumps(sys.argv[1:]))"
        self.executable(path, "#!/bin/sh\nexec " + shlex.quote(sys.executable) + " -c " + shlex.quote(code) + ' "$@"\n')
        return path

    def invoke(self, *args, path=None):
        return subprocess.run([str(path or self.wrapper), *args], capture_output=True, text=True,
                              env=self.wrapper_env, cwd=ROOT, timeout=15)

    def test_prefers_local_binary_with_spaces_and_injects_root(self):
        self.standin(self.checkout)
        self.standin(self.main)
        result = self.invoke("find", "--name", "X")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), ["--root", str(self.checkout), "find", "--name", "X"])

    def test_falls_back_to_main_binary_without_building(self):
        self.standin(self.main)
        result = self.invoke("map", "--stdout")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), ["--root", str(self.checkout), "map", "--stdout"])
        self.assertFalse((self.checkout / "rust").exists())

    def test_no_binary_prints_build_command_and_builds_nothing(self):
        marker = self.e.base / "cargo-was-run"
        self.executable(self.tools / "cargo", "#!/bin/sh\ntouch " + shlex.quote(str(marker)) + "\nexit 90\n")
        before = set(self.e.base.rglob("*"))
        result = self.invoke("map")
        self.assertEqual(result.returncode, 1)
        self.assertIn("cargo build --release -p cardgen", result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertEqual(set(self.e.base.rglob("*")), before)
        self.assertFalse(marker.exists())

    def test_explicit_root_is_preserved(self):
        self.standin(self.checkout)
        result = self.invoke("--root", "/explicit root", "map")
        self.assertEqual(json.loads(result.stdout), ["--root", "/explicit root", "map"])

    def test_root_word_in_argument_does_not_suppress_injection(self):
        self.standin(self.checkout)
        args = ["find", "--text", "the --root argument"]
        result = self.invoke(*args)
        self.assertEqual(json.loads(result.stdout), ["--root", str(self.checkout), *args])

    def test_relative_git_common_dir_fallback(self):
        self.standin(self.main)
        self.fake_git(os.path.relpath(self.main / ".git", self.checkout))
        result = self.invoke("map")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), ["--root", str(self.checkout), "map"])

    def test_external_symlink_preserves_checkout_root(self):
        self.standin(self.checkout)
        link = self.e.base / "external bin/cardgen"
        link.parent.mkdir()
        link.symlink_to(self.wrapper)
        result = self.invoke("map", path=link)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), ["--root", str(self.checkout), "map"])

    def test_existing_worktree_uses_existing_main_binary(self):
        main_binary = Path("/home/joseph/projects/commander-sim/rust/target/release/cardgen")
        self.assertTrue(os.access(main_binary, os.X_OK), "brief requires the existing main binary")
        self.assertFalse((ROOT / "rust/target/release/cardgen").exists(), "fixture needs a worktree without a binary")
        env = dict(self.e.env)
        result = subprocess.run([str(WRAPPER), "--help"], capture_output=True, text=True, env=env, timeout=15)
        direct = subprocess.run([str(main_binary), "--root", str(ROOT), "--help"],
                                capture_output=True, text=True, env=env, timeout=15)
        self.assertEqual((result.returncode, result.stdout, result.stderr),
                         (direct.returncode, direct.stdout, direct.stderr))
        self.assertIn("cardgen", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
