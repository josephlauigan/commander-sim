"""Line coverage of the commander_sim package by the test suite, standard library only.

    python3 -m commander_sim.tools.linecov                     # per-module table and total
    python3 -m commander_sim.tools.linecov --missing engine    # also the uncovered lines of matching modules

It runs the tests in this process and records every line of commander_sim that executes, including in the
subprocesses the command-line tests start and in their worker processes (a sitecustomize hook in a temporary
folder on PYTHONPATH switches recording on in each of them). A line counts once it has run at all. It uses
sys.monitoring on Python 3.12+, and the slower sys.settrace before that.

Coverage says which code the tests reach, not whether they check its result: most game code is reached by
the end-to-end games, and tests/test_rules.py and tests/test_my_cards.py are what check card behaviour.
"""
import json, os, subprocess, sys, tempfile, textwrap, types
from commander_sim import ROOT

PKG = os.path.join(ROOT, 'commander_sim') + os.sep

# the recorder, shared by this process and every child (installed in children as sitecustomize.py)
RECORDER = r'''
import atexit, json, os, sys
_PKG = os.environ.get('LINECOV_PKG'); _OUT = os.environ.get('LINECOV_OUT')
_hits = {}


def _dump():
    if not _hits: return
    path = os.path.join(_OUT, f'{os.getpid()}.json')
    with open(path + '.tmp', 'w') as f:                      # a whole file or none (a worker can be killed mid-write)
        json.dump({k: sorted(v) for k, v in _hits.items()}, f)
    os.replace(path + '.tmp', path)


def _start():
    if not (_PKG and _OUT): return
    if hasattr(sys, 'monitoring'):
        M = sys.monitoring; tool = M.COVERAGE_ID
        try: M.use_tool_id(tool, 'linecov')
        except ValueError: return
        def line(code, n):
            f = code.co_filename
            if f.startswith(_PKG): _hits.setdefault(f, set()).add(n)
            return M.DISABLE
        M.register_callback(tool, M.events.LINE, line)
        M.set_events(tool, M.events.LINE)
    else:
        def local(frame, event, arg):
            if event == 'line': _hits.setdefault(frame.f_code.co_filename, set()).add(frame.f_lineno)
            return local
        def glob(frame, event, arg):
            return local if frame.f_code.co_filename.startswith(_PKG) else None
        sys.settrace(glob)
        import threading; threading.settrace(glob)
    atexit.register(_dump)
    # multiprocessing workers leave through os._exit, and clear inherited finalizers as they start: register the
    # dump again in each of them, after that clearing
    import multiprocessing.util as U
    U.register_after_fork(_KEEP, _in_worker)


def _in_worker(_):
    import multiprocessing.util as U, signal
    U.Finalize(None, _dump, exitpriority=0)                  # a normal exit (Pool.close)
    signal.signal(signal.SIGTERM, lambda *a: (_dump(), os._exit(0)))   # Pool.terminate, the with-statement's exit


class _Keep: pass
_KEEP = _Keep()
_start()
'''


def executable_lines(path):
    lines = set()
    todo = [compile(open(path).read(), path, 'exec')]
    while todo:
        co = todo.pop()
        lines.update(n for _, _, n in co.co_lines() if n) if hasattr(co, 'co_lines') else \
            lines.update(n for _, n in __import__('dis').findlinestarts(co))
        todo += [c for c in co.co_consts if isinstance(c, types.CodeType)]
    return lines


def ranges(nums):
    out, start, prev = [], None, None
    for n in sorted(nums):
        if start is None: start = prev = n
        elif n == prev + 1: prev = n
        else: out.append((start, prev)); start = prev = n
    if start is not None: out.append((start, prev))
    return ', '.join(f'{a}' if a == b else f'{a}-{b}' for a, b in out)


def main():
    missing = sys.argv[sys.argv.index('--missing') + 1] if '--missing' in sys.argv else None
    with tempfile.TemporaryDirectory() as tmp:
        hook = os.path.join(tmp, 'hook'); out = os.path.join(tmp, 'hits')
        os.makedirs(hook); os.makedirs(out)
        open(os.path.join(hook, 'sitecustomize.py'), 'w').write(textwrap.dedent(RECORDER))
        env = dict(os.environ, LINECOV_PKG=PKG, LINECOV_OUT=out,
                   PYTHONPATH=os.pathsep.join([hook, ROOT] + ([os.environ['PYTHONPATH']] if os.environ.get('PYTHONPATH') else [])))
        print('running the tests with line recording (a few minutes)...', file=sys.stderr, flush=True)
        r = subprocess.run([sys.executable, '-m', 'unittest', 'discover', '-s', 'tests', '-t', '.'], cwd=ROOT, env=env,
                           capture_output=True, text=True)
        print(r.stderr.strip().splitlines()[-1] if r.stderr.strip() else '', file=sys.stderr)
        hits = {}
        for f in os.listdir(out):
            if not f.endswith('.json'): continue
            for path, ns in json.load(open(os.path.join(out, f))).items():
                hits.setdefault(path, set()).update(ns)
    rows = []
    for dp, _, fs in os.walk(PKG):
        for f in fs:
            if f.endswith('.py'):
                p = os.path.join(dp, f)
                ex = executable_lines(p)
                rows.append((os.path.relpath(p, ROOT), ex, ex & hits.get(p, set())))
    rows.sort()
    print(f"{'module':44s} {'lines run':>16s}")
    for name, ex, hit in rows:
        print(f'{name:44s} {len(hit):6d} / {len(ex):6d}  {100 * len(hit) / max(1, len(ex)):5.1f}%')
    te = sum(len(ex) for _, ex, _ in rows); th = sum(len(h) for _, _, h in rows)
    print(f"{'TOTAL':44s} {th:6d} / {te:6d}  {100 * th / te:5.1f}%")
    if missing:
        for name, ex, hit in rows:
            if missing in name and ex - hit:
                print(f'\n{name}: not run: {ranges(ex - hit)}')
    if r.returncode != 0:
        print('\nThe tests did not all pass; run them directly to see why.', file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
