#!/usr/bin/env bash
# Prove a fix by machine: the named test FAILS (by assertion or panic) at commit R and PASSES at commit G.
#
#   scripts/red-green.sh --tree <scratch worktree> --red <commit> --green <commit> --test <name> --ledger <file>
#                        [--cargo-args "-p sim-core --test cards_x"] [--dry-run]
#   scripts/red-green.sh --ledger <file> --test <name> --coverage      (a row for a test that is coverage, not a fix)
#
# In the scratch worktree (clean, never the main checkout) it checks out R, runs `cargo test --release --no-fail-fast
# <name>` and requires: no compile error (that is NOT red), at least one test whose path is or ends with <name>, every
# matched test FAILED, cargo exit non-zero. It then checks out G and requires every matched test to be ok and cargo
# exit 0. A row (test, R, G, red evidence, green evidence) is appended to the ledger; the full outputs are kept next to
# it as <ledger>.<test>.red.log and .green.log. The tree is left at G. Exit 0 only if both halves hold; 3 = R did not
# compile or G did not compile; 1 = not red or not green.
# Cargo runs only through the lock and the memory cap, with the tree's own target dir (gates-lib/common.sh).
set -uo pipefail

GATES_PROG=red-green.sh
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=gates-lib/common.sh
source "$here/gates-lib/common.sh"

tree="" red="" green="" test="" ledger="" cargo_args="" coverage=0
while (($#)); do
  case $1 in
    --tree) tree=${2:?}; shift 2 ;;
    --red) red=${2:?}; shift 2 ;;
    --green) green=${2:?}; shift 2 ;;
    --test) test=${2:?}; shift 2 ;;
    --ledger) ledger=${2:?}; shift 2 ;;
    --cargo-args) cargo_args=${2:?}; shift 2 ;;
    --coverage) coverage=1; shift ;;
    --dry-run) DRY=1; shift ;;
    -h|--help) sed -n '2,16p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 2 ;;
    *) die "unknown argument $1" ;;
  esac
done
[[ -n $test && -n $ledger ]] || die "--test and --ledger are required"
mkdir -p "$(dirname "$ledger")" || die "cannot create the ledger directory"

row_header() {
  [[ -s $ledger ]] || { printf '| test | R | G | red evidence | green evidence |\n|---|---|---|---|---|\n' >"$ledger"; }
}

if ((coverage)); then
  row_header
  printf '| %s | - | - | coverage, no red/green | coverage, no red/green |\n' "$test" >>"$ledger"
  echo "ledger row (coverage): $test"; exit 0
fi

[[ -n $tree && -n $red && -n $green ]] || die "--tree, --red and --green are required"
tree=$(check_tree "$tree")
r=$(git -C "$tree" rev-parse --verify --quiet "$red^{commit}") || die "unknown commit: $red"
g=$(git -C "$tree" rev-parse --verify --quiet "$green^{commit}") || die "unknown commit: $green"
[[ $r != "$g" ]] || die "R and G are the same commit"
[[ -z $(git -C "$tree" status --porcelain) ]] || die "the scratch tree is not clean: $tree"
safe=${test//[^A-Za-z0-9._-]/_}
log_red=$ledger.$safe.red.log log_green=$ledger.$safe.green.log

# run_at COMMIT LOG: check COMMIT out, run the test; sets RC. Output goes to LOG.
run_at() {
  git -C "$tree" checkout -q --detach "$1" || die "cannot check out $1"
  # shellcheck disable=SC2086
  heavy "$tree" cargo test --release --no-fail-fast $cargo_args "$test" >"$2" 2>&1
  RC=$?
}

# the `test NAME ... STATUS` lines whose NAME is or ends with the test name
matched() { grep -E "^test (.*::)?$(printf '%s' "$test" | sed 's/[][\.*^$/]/\\&/g') \.\.\. " "$1" || true; }

if [[ $DRY == 1 ]]; then
  echo "DRY: git -C $tree checkout --detach $r  (red: the test must FAIL)"
  # shellcheck disable=SC2086
  heavy "$tree" cargo test --release --no-fail-fast $cargo_args "$test"
  echo "DRY: git -C $tree checkout --detach $g  (green: the test must pass)"
  # shellcheck disable=SC2086
  heavy "$tree" cargo test --release --no-fail-fast $cargo_args "$test"
  exit 0
fi

# a failing test also ends with `error: test failed`, so only these mean the build broke
compile_error() { grep -qE '^error\[E[0-9]+\]|could not compile|^error: aborting due to' "$1"; }

# ---- red
run_at "$r" "$log_red"
if compile_error "$log_red"; then echo "NOT RED: R does not compile (a compile error is not red): $log_red"; exit 3; fi
m=$(matched "$log_red")
[[ -n $m ]] || { echo "NOT RED: no test named $test ran at R: $log_red"; exit 1; }
if grep -qv 'FAILED$' <<<"$m"; then echo "NOT RED: at R a matched test did not fail: $(grep -v 'FAILED$' <<<"$m" | head -n1)"; exit 1; fi
((RC != 0)) || { echo "NOT RED: cargo exited 0 at R: $log_red"; exit 1; }
why=$(grep -m1 -E "panicked at|assertion" "$log_red" || true)
red_ev="FAILED at ${r:0:12}: ${why:-test failed} ($log_red)"

# ---- green
run_at "$g" "$log_green"
if compile_error "$log_green"; then echo "NOT GREEN: G does not compile: $log_green"; exit 3; fi
m=$(matched "$log_green")
[[ -n $m ]] || { echo "NOT GREEN: no test named $test ran at G: $log_green"; exit 1; }
if grep -qv ' ok$' <<<"$m"; then echo "NOT GREEN: at G a matched test is not ok: $(grep -v ' ok$' <<<"$m" | head -n1)"; exit 1; fi
((RC == 0)) || { echo "NOT GREEN: cargo exited $RC at G: $log_green"; exit 1; }
green_ev="ok at ${g:0:12} ($log_green)"

row_header
printf '| %s | %s | %s | %s | %s |\n' "$test" "${r:0:12}" "${g:0:12}" "$red_ev" "$green_ev" >>"$ledger"
echo "RED/GREEN ok: $test  R=${r:0:12} G=${g:0:12}  ledger $ledger"
