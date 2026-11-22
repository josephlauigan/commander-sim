"""Mutate only a disposable scratch copy; never execute network or service commands.

usage: python3 -I mutations.py REPO SCRATCH      (SCRATCH must not exist)

The scratch copy gets its own cargo target directory, so the repository's build is never touched. Each mutation
edits the scratch copy of scripts/knowledge-weekly.sh (or, for the last ones, src/llm.rs), runs the weekly test
suites (weekly_script and weekly_adversarial, every test launch under the memory cap) and prints whether some test
caught it (a non-zero exit) or not.
"""
import os
import pathlib
import shutil
import subprocess
import sys

repo = pathlib.Path(sys.argv[1]).resolve()
scratch = pathlib.Path(sys.argv[2]).resolve()
scratch.mkdir(parents=True, exist_ok=False)
cap = ['systemd-run', '--user', '--scope', '-q', '-p', 'MemoryMax=24G', '-p', 'MemorySwapMax=0']
manifest = scratch / 'rust/crates/cardgen'
manifest.mkdir(parents=True)
(scratch / 'scripts').mkdir()
shutil.copytree(repo / 'scripts', scratch / 'scripts', dirs_exist_ok=True)
shutil.copytree(repo / 'rust/crates/cardgen', manifest, dirs_exist_ok=True)
for directory in ['data', 'decklists', 'audit', 'documents']:
    (scratch / directory).symlink_to(repo / directory, target_is_directory=True)
for f in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'rustfmt.toml']:
    shutil.copy2(repo / 'rust' / f, scratch / 'rust' / f)
for c in ['sim-core', 'practice', 'sim-cli']:
    (scratch / 'rust/crates' / c).symlink_to(repo / 'rust/crates' / c, target_is_directory=True)
result_dir = scratch / 'results'
result_dir.mkdir()
env = dict(os.environ, CARGO_TARGET_DIR=str(scratch / 'target'))
tests = ['--test', 'weekly_script', '--test', 'weekly_adversarial', '--test', 'weekly', '--test', 'weekly_sync']
cargo_test = cap + ['cargo', 'test', '--release', '--offline', '-p', 'cardgen'] + tests


def run(name):
    with (result_dir / f'{name}.log').open('w') as out:
        return subprocess.run(cargo_test, cwd=scratch / 'rust', env=env, stdout=out, stderr=subprocess.STDOUT).returncode


baseline = run('baseline')
print(f'baseline: exit {baseline}', flush=True)
assert baseline == 0, 'the unmutated scratch copy must pass'

script = scratch / 'scripts/knowledge-weekly.sh'
original = script.read_text()
script_mutations = [
    # the traps that stop a server this job started
    ('remove-traps', lambda s: s.replace("trap 'llm_release own' EXIT", ': # removed stage trap').replace('trap cleanup EXIT', ': # removed parent trap')),
    # a stage writes into the published directory instead of staging
    ('map-digest-without-staging', lambda s: s.replace('s="$(stage_dir map-digest)"', 's="$OUT"')),
    ('roles-without-staging', lambda s: s.replace('s="$(stage_dir roles)"', 's="$OUT"')),
    # publish a stage's outputs although it failed
    ('publish-roles-despite-failure', lambda s: s.replace('roles) if ((rc == 0)); then publish_roles', 'roles) if ((rc >= 0)); then publish_roles')),
    ('publish-map-despite-failure', lambda s: s.replace('map-digest) if ((rc == 0)); then publish_map_digest', 'map-digest) if ((rc >= 0)); then publish_map_digest')),
    ('drop-updated-at-skip', lambda s: s.replace('    return 10\n  fi\n  produced "$s/oracle.json"', '    return 0\n  fi\n  produced "$s/oracle.json"')),
    ('count-sanity-one', lambda s: s.replace('KNOWLEDGE_BULK_MIN:-25000', 'KNOWLEDGE_BULK_MIN:-1')),
    ('merge-includes-pending', lambda s: s.replace('cg roles --merge --no-think', 'cg roles --merge --pending --no-think')),
    ('llm-call-without-mode', lambda s: s.replace('LLM_ARGS=(roles --llm --routes', 'LLM_ARGS=(roles --routes')),
    ('gpu-unknown-is-free', lambda s: s.replace('  *) echo unknown ;;', '  *) echo free ;;')),
    ('no-report-retention', lambda s: s.replace('  prune_reports\n}', '  :\n}')),
    ('claim-before-reprobe', lambda s: s.replace('  if [[ -n "$(llm_models)" ]]; then return 20; fi\n', '')),
    ('gpu-threshold-ignored', lambda s: s.replace('if ((total_kib >= limit * 1024)); then', 'if ((total_kib >= 1)); then')),
    ('gpu-unknown-state-is-free', lambda s: s.replace('  elif ((unknown)); then\n    echo unknown', '  elif ((unknown)); then\n    echo free')),
    ('night-window-removed', lambda s: s.replace('  elif ((in_window != 0)); then', '  elif false; then')),
    ('timeout-leaves-the-stage-group', lambda s: s.replace('timeout --foreground --kill-after=60', 'timeout --kill-after=60')),
    ('cargo-lock-waits-forever', lambda s: s.replace('flock -w "${KNOWLEDGE_CARGO_LOCK_WAIT:-3600}" -E 200 "$CARGO_LOCK"', 'flock "$CARGO_LOCK"')),
    ('no-exec-stop-post-cleanup', lambda s: s.replace('    llm_release\n    exit 0\n  fi\n\n  if ((DRY))', '    exit 0\n  fi\n\n  if ((DRY))')),
]
for name, mutate in script_mutations:
    changed = mutate(original)
    assert changed != original, name
    script.write_text(changed)
    code = run(name)
    print(f'{name}: {"CAUGHT" if code != 0 else "MISSED"} (exit {code})', flush=True)
script.write_text(original)

unit = scratch / 'scripts/systemd/knowledge-weekly.service'
unit_original = unit.read_text()
unit.write_text(unit_original.replace('ExecStopPost=', '#ExecStopPost='))
code = run('unit-without-exec-stop-post')
print(f'unit-without-exec-stop-post: {"CAUGHT" if code != 0 else "MISSED"} (exit {code})', flush=True)
unit.write_text(unit_original)

# genuine Rust mutations (the scratch cardgen is rebuilt)
rust_mutations = [
    ('src/llm.rs', 'pending-loaded-by-default', 'load_cache_with(dir, false)', 'load_cache_with(dir, true)'),
    ('src/weekly.rs', 'retention-sorts-by-name', 'keyed.sort();', 'keyed.sort_by(|a, b| a.1.cmp(&b.1));'),
    ('src/weekly.rs', 'ban-only-on-transition', '!is_legal(c) && !listed.contains(c.oracle_id.as_str())', 'false'),
    ('src/weekly.rs', 'first-name-wins', 'digest::resolve(&self.idx, name)', 'self.idx.lookup(name).to_vec()'),
    ('src/weekly.rs', 'impossible-dates-accepted', '&& chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()', ''),
    ('src/weekly.rs', 'markdown-not-escaped', "if matches!(c, '\\\\' | '`' | '[' | ']' | '|' | '#' | '<' | '>' | '*') {", "if false {"),
]
for rel, name, old, new in rust_mutations:
    path = manifest / rel
    text = path.read_text()
    assert old in text, name
    path.write_text(text.replace(old, new, 1))
    code = run(name)
    print(f'{name}: {"CAUGHT" if code != 0 else "MISSED"} (exit {code})', flush=True)
    path.write_text(text)
print(f'Detailed logs: {result_dir}', flush=True)
