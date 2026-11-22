"""Mutate only a disposable scratch copy; never execute network or service commands.

The original weekly_script integration test uses std alone, so rustc can compile it
against a scratch script without rebuilding the workspace. Each test launch is capped.
The final mutation rebuilds cardgen offline against a scratch llm.rs.
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
binary = repo / 'rust/target/release/cardgen'
environment = dict(os.environ, CARGO_MANIFEST_DIR=str(manifest), CARGO_BIN_EXE_cardgen=str(binary))
result_dir = scratch / 'results'
result_dir.mkdir()
test = scratch / 'weekly-script-tests'
subprocess.run(cap + ['rustc', '--edition=2024', '--test', str(manifest / 'tests/weekly_script.rs'), '-o', str(test)],
               check=True, env=environment)
script = scratch / 'scripts/knowledge-weekly.sh'
original = script.read_text()
with (result_dir / 'baseline.log').open('w') as out:
    subprocess.run(cap + [str(test)], stdout=out, stderr=subprocess.STDOUT, check=True)
mutations = [
    ('remove-traps', lambda s: s.replace('trap llm_stop_if_started EXIT', ': # removed stage trap')
     .replace('trap cleanup EXIT', ': # removed parent trap')),
    ('remove-atomic-rename', lambda s: s.replace('mv -f "$tmp" "$path"', 'cp "$tmp" "$path"')),
    ('drop-updated-at-skip', lambda s: s.replace('    return 10\n  fi\n  note "$(grep -m1',
                                               '    return 0\n  fi\n  note "$(grep -m1')),
    ('count-sanity-one', lambda s: s.replace('KNOWLEDGE_BULK_MIN:-25000', 'KNOWLEDGE_BULK_MIN:-1')),
    ('merge-includes-pending', lambda s: s.replace('cg roles --merge --no-think', 'cg roles --merge --pending --no-think')),
]
for name, mutate in mutations:
    changed = mutate(original)
    assert changed != original, name
    script.write_text(changed)
    with (result_dir / f'{name}.log').open('w') as out:
        r = subprocess.run(cap + [str(test)], stdout=out, stderr=subprocess.STDOUT)
    print(f'{name}: existing weekly_script suite exit {r.returncode}', flush=True)
script.write_text(original)

# A genuine Rust mutation of the tracked-only loader. Original weekly_script tests
# still use the mutant cardgen binary for all their offline runs.
llm = manifest / 'src/llm.rs'
text = llm.read_text()
changed = text.replace('load_cache_with(dir, false)', 'load_cache_with(dir, true)')
assert changed != text
llm.write_text(changed)
target = repo / 'rust/target'
build_env = dict(os.environ, CARGO_TARGET_DIR=str(target))
with (result_dir / 'pending-rust-build.log').open('w') as out:
    subprocess.run(cap + ['cargo', 'build', '--release', '--offline', '-p', 'cardgen'],
                   cwd=scratch / 'rust', env=build_env, stdout=out, stderr=subprocess.STDOUT, check=True)
# Save the mutant executable separately before the normal worktree rebuilds it.
mutant_binary = scratch / 'cardgen-mutant'
shutil.copy2(target / 'release/cardgen', mutant_binary)
environment['CARGO_BIN_EXE_cardgen'] = str(mutant_binary)
subprocess.run(cap + ['rustc', '--edition=2024', '--test', str(manifest / 'tests/weekly_script.rs'), '-o', str(test)],
               check=True, env=environment)
with (result_dir / 'pending-rust-filter.log').open('w') as out:
    r = subprocess.run(cap + [str(test)], stdout=out, stderr=subprocess.STDOUT)
print(f'pending-rust-filter: existing weekly_script suite exit {r.returncode}', flush=True)
print(f'Detailed logs: {result_dir}', flush=True)
