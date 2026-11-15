#!/usr/bin/env bash
# The L0 and L1 gates of a PR, judged by machine. One PASS/FAIL line per gate, with its evidence path, goes into
# <gates-dir>/<pr>.md (default /tmp/handoff/gates). Exit 0 only if every gate run in this call passed.
#
#   scripts/stage-gate.sh <worktree> --base <commit> --pr <name> --base-tree <path> [options]
#     --declared-deletions FILE   test names the PR deletes on purpose, one per line (`binary::name` or the bare name)
#     --review FILE               the Opus review file: its first line must be `VERDICT: commit`
#     --only GATE[,GATE...]       run just these gates (the other lines of the gate file are kept)
#     --gates-dir DIR             where the gate file and logs go (default /tmp/handoff/gates)
#     --dry-run                   print the cargo commands instead of running them (writes <pr>.dry-run.md)
#   gates: suite ignored names diffcheck fmt audit-mine audit-paths validate export allowlists newconsts review
#
# Cargo only runs as `flock /tmp/cargo-heavy.lock systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0
# env CARGO_TARGET_DIR=<tree>/rust/target cargo ...` from <tree>/rust (see gates-lib/common.sh). The base tree is a
# checkout of --base (the coordinator keeps one); its test lists are cached per base commit in the gates dir.
# Test seam: GATES_LIST_DIR=DIR reads base.list, branch.list, base.ignored and branch.ignored from DIR instead of cargo.
set -uo pipefail

GATES_PROG=stage-gate.sh
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=gates-lib/common.sh
source "$here/gates-lib/common.sh"
check=(python3 -I "$here/gates-lib/gatecheck.py")
ALL_GATES=(suite ignored names diffcheck fmt audit-mine audit-paths validate export allowlists newconsts review)

usage() { sed -n '2,17p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 2; }

wt_arg="" base_arg="" pr="" decl="" review="" base_tree="" only="" gates_dir=/tmp/handoff/gates
while (($#)); do
  case $1 in
    --base) base_arg=${2:?}; shift 2 ;;
    --pr) pr=${2:?}; shift 2 ;;
    --declared-deletions) decl=${2:?}; shift 2 ;;
    --review) review=${2:?}; shift 2 ;;
    --base-tree) base_tree=${2:?}; shift 2 ;;
    --only) only=${2:?}; shift 2 ;;
    --gates-dir) gates_dir=${2:?}; shift 2 ;;
    --dry-run) DRY=1; shift ;;
    -h|--help) usage ;;
    -*) die "unknown option $1" ;;
    *) [[ -z $wt_arg ]] || die "one worktree only"; wt_arg=$1; shift ;;
  esac
done
[[ -n $wt_arg && -n $base_arg && -n $pr ]] || usage
[[ $pr =~ ^[A-Za-z0-9._-]+$ ]] || die "--pr must be a plain name: $pr"

wt=$(check_tree "$wt_arg")
base=$(git -C "$wt" rev-parse --verify --quiet "$base_arg^{commit}") || die "unknown base commit: $base_arg"
head=$(git -C "$wt" rev-parse HEAD)
git -C "$wt" merge-base --is-ancestor "$base" "$head" || die "base $base_arg is not an ancestor of the branch head"
if [[ -n $base_tree ]]; then
  base_tree=$(check_tree "$base_tree")
  [[ $(git -C "$base_tree" rev-parse HEAD) == "$base" ]] || die "base tree $base_tree is not at $base_arg"
fi
[[ -z $decl || -f $decl ]] || die "no such declared-deletions file: $decl"
mkdir -p "$gates_dir" || die "cannot create $gates_dir"
gates_dir=$(abspath "$gates_dir")

if [[ -n $only ]]; then
  IFS=, read -r -a run_list <<<"$only"
  for g in "${run_list[@]}"; do [[ " ${ALL_GATES[*]} " == *" $g "* ]] || die "unknown gate: $g"; done
else
  run_list=("${ALL_GATES[@]}")
fi

gate_file=$gates_dir/$pr.md
[[ $DRY == 1 ]] && gate_file=$gates_dir/$pr.dry-run.md
declare -A line_of=()      # gate -> its PASS/FAIL line, from earlier runs and this one
declare -A ran_now=()
if [[ -f $gate_file ]]; then
  while IFS= read -r ln; do
    if [[ $ln =~ ^(PASS|FAIL|SKIP)\ ([a-z-]+): ]]; then line_of[${BASH_REMATCH[2]}]=$ln; fi
  done <"$gate_file"
fi

write_file() {
  local g n_fail=0 missing=()
  {
    echo "# Gates for $pr"
    echo "base $base ($base_arg), head $head, tree $wt, $(date '+%F %T')"
    for g in "${ALL_GATES[@]}"; do
      if [[ -n ${line_of[$g]:-} ]]; then
        echo "${line_of[$g]}"
        [[ ${line_of[$g]} == FAIL* ]] && n_fail=$((n_fail + 1))
        [[ ${line_of[$g]} == SKIP* ]] && missing+=("$g")
      else
        missing+=("$g")
      fi
    done
    if ((n_fail)); then echo "RESULT: FAIL ($n_fail failing)"
    elif ((${#missing[@]})); then echo "RESULT: INCOMPLETE (not run: ${missing[*]})"
    else echo "RESULT: PASS"; fi
  } >"$gate_file"
}

failed_now=0
# record GATE PASS|FAIL|SKIP DETAIL [EVIDENCE]
record() {
  local g=$1 st=$2 detail=$3 ev=${4:-}
  line_of[$g]="$st $g: $detail head=${head:0:12}${ev:+ | evidence: $ev}"
  ran_now[$g]=1
  [[ $st == FAIL ]] && failed_now=$((failed_now + 1))
  echo "${line_of[$g]}"
  write_file
}

# run_cargo LOGFILE TREE CMD...: the command under the lock and the cap, output into LOGFILE (or printed in a dry run)
run_cargo() {
  local log=$1 tree=$2; shift 2
  if [[ $DRY == 1 ]]; then heavy "$tree" "$@"; return 0; fi
  heavy "$tree" "$@" >"$log" 2>&1
}

# verdict_of GATE OUTFILE EVIDENCE CMD...: run CMD (a gatecheck call), keep its output in OUTFILE; its first line is the
# detail and exit 0 is PASS
verdict_of() {
  local g=$1 outf=$2 ev=$3; shift 3
  local text rc
  text=$("$@" 2>&1); rc=$?
  printf '%s\n' "$text" >"$outf"
  record "$g" "$([[ $rc == 0 ]] && echo PASS || echo FAIL)" "$(head -n1 <<<"$text")" "$ev"
}

need_base_tree() { [[ -n $base_tree ]] || die "gate $1 needs --base-tree"; }
log_of() { echo "$gates_dir/$pr-$1.log"; }

# the test lists of the base (cached per commit) and the branch: sets LIST_BASE LIST_BRANCH IGN_BASE IGN_BRANCH
make_lists() {
  need_base_tree "$1"
  if [[ -n ${GATES_LIST_DIR:-} ]]; then
    LIST_BASE=$GATES_LIST_DIR/base.list LIST_BRANCH=$GATES_LIST_DIR/branch.list
    IGN_BASE=$GATES_LIST_DIR/base.ignored IGN_BRANCH=$GATES_LIST_DIR/branch.ignored
    return 0
  fi
  local tag=${base:0:12}
  LIST_BASE=$gates_dir/base-$tag.list IGN_BASE=$gates_dir/base-$tag.ignored
  LIST_BRANCH=$(log_of list) IGN_BRANCH=$(log_of list-ignored)
  local f what
  for what in "" "--ignored"; do
    f=$LIST_BASE; [[ -n $what ]] && f=$IGN_BASE
    if [[ ! -s $f ]]; then
      run_cargo "$f.tmp" "$base_tree" cargo test --release --workspace --no-fail-fast -- --list $what \
        && { [[ $DRY == 1 ]] || mv "$f.tmp" "$f"; } || { rm -f "$f.tmp"; return 1; }
    fi
    f=$LIST_BRANCH; [[ -n $what ]] && f=$IGN_BRANCH
    run_cargo "$f" "$wt" cargo test --release --workspace --no-fail-fast -- --list $what || return 1
  done
}

ignore_count() { (cd "$1" && grep -rn '#\[ignore' rust/crates 2>/dev/null | wc -l); }

gate_suite() {
  local ev; ev=$(log_of suite)
  if [[ $DRY == 1 ]]; then run_cargo "$ev" "$wt" cargo test --release --workspace --no-fail-fast; record suite SKIP "dry run"; return; fi
  run_cargo "$ev" "$wt" cargo test --release --workspace --no-fail-fast; local rc=$?
  local text; text=$("${check[@]}" suite "$ev" 2>&1); local crc=$?
  if ((rc != 0 && crc == 0)); then text="cargo exited $rc ($(head -n1 <<<"$text")); see the log"; crc=1; fi
  record suite "$([[ $crc == 0 ]] && echo PASS || echo FAIL)" "$(head -n1 <<<"$text")" "$ev"
}

gate_ignored() {
  local ev; ev=$(log_of ignored)
  make_lists ignored || { record ignored FAIL "could not list the tests" "$ev"; return; }
  [[ $DRY == 1 ]] && { record ignored SKIP "dry run"; return; }
  verdict_of ignored "$ev" "$ev" "${check[@]}" ignored "$IGN_BASE" "$IGN_BRANCH" "$(ignore_count "$base_tree")" "$(ignore_count "$wt")"
}

gate_names() {
  local ev; ev=$(log_of names)
  make_lists names || { record names FAIL "could not list the tests" "$ev"; return; }
  [[ $DRY == 1 ]] && { record names SKIP "dry run"; return; }
  verdict_of names "$ev" "$ev" "${check[@]}" names "$LIST_BASE" "$LIST_BRANCH" ${decl:+"$decl"}
}

gate_diffcheck() {
  local ev rc; ev=$(log_of diffcheck)
  git -C "$wt" diff --check "$base" >"$ev" 2>&1; rc=$?
  record diffcheck "$([[ $rc == 0 ]] && echo PASS || echo FAIL)" "git diff --check $base_arg: exit $rc" "$ev"
}

gate_fmt() {
  local ev rc files=(); ev=$(log_of fmt)
  while IFS= read -r f; do [[ $f == *.rs && -f $wt/$f ]] && files+=("$wt/$f"); done < <(
    { git -C "$wt" diff --name-only --diff-filter=ACMR "$base"; git -C "$wt" ls-files --others --exclude-standard; } | sort -u)
  if ((${#files[@]} == 0)); then : >"$ev"; record fmt PASS "no .rs file touched" "$ev"; return; fi
  if [[ $DRY == 1 ]]; then echo "DRY: (cd $wt/rust && rustfmt --check --edition 2024 ${files[*]})"; record fmt SKIP "dry run"; return; fi
  (cd "$wt/rust" && rustfmt --check --edition 2024 "${files[@]}") >"$ev" 2>&1; rc=$?
  record fmt "$([[ $rc == 0 ]] && echo PASS || echo FAIL)" "rustfmt --check --edition 2024 on ${#files[@]} files: exit $rc" "$ev"
}

run_cardgen() { run_cargo "$1" "$wt" cargo run --release -q -p cardgen -- --root "$wt" "${@:2}"; }

gate_audit-mine() {
  local ev; ev=$(log_of audit-mine)
  run_cardgen "$ev" audit --mine; local rc=$?
  [[ $DRY == 1 ]] && { record audit-mine SKIP "dry run"; return; }
  if ((rc != 0)); then record audit-mine FAIL "cardgen audit --mine exited $rc" "$ev"; return; fi
  verdict_of audit-mine "$ev.check" "$ev" "${check[@]}" audit-mine "$ev"
}

gate_audit-paths() {
  local ev; ev=$(log_of audit-paths)
  run_cardgen "$ev" audit --paths; local rc=$?
  [[ $DRY == 1 ]] && { record audit-paths SKIP "dry run"; return; }
  if ((rc != 0)); then record audit-paths FAIL "cardgen audit --paths exited $rc" "$ev"; return; fi
  verdict_of audit-paths "$ev.check" "$ev" "${check[@]}" audit-paths "$ev"
}

gate_validate() {
  local ev; ev=$(log_of validate)
  run_cardgen "$ev" validate; local rc=$?
  [[ $DRY == 1 ]] && { record validate SKIP "dry run"; return; }
  if ((rc == 0)) && grep -q '^all decks passed' "$ev"; then record validate PASS "all decks passed" "$ev"
  else record validate FAIL "cardgen validate exited $rc" "$ev"; fi
}

gate_export() {
  local ev; ev=$(log_of export)
  if [[ $DRY == 1 ]]; then run_cardgen "$ev" export --offline; record export SKIP "dry run"; return; fi
  if [[ -n $(git -C "$wt" status --porcelain -uall) ]]; then
    git -C "$wt" status --porcelain -uall >"$ev"; record export FAIL "the tree is not clean before the export: commit first" "$ev"; return
  fi
  run_cardgen "$ev" export --offline; local rc=$?
  local dirty; dirty=$(git -C "$wt" status --porcelain -uall)
  if [[ -n $dirty ]]; then
    { echo "--- git status --porcelain after the export (restored below)"; echo "$dirty"; } >>"$ev"
    git -C "$wt" checkout -q -- . && git -C "$wt" clean -fdq
  fi
  if ((rc != 0)); then record export FAIL "cardgen export --offline exited $rc" "$ev"
  elif [[ -n $dirty ]]; then record export FAIL "the export changed $(wc -l <<<"$dirty") paths (tree restored)" "$ev"
  else record export PASS "export reproducible: tree clean after cardgen export --offline" "$ev"; fi
}

gate_allowlists() {
  need_base_tree allowlists
  verdict_of allowlists "$(log_of allowlists)" "$(log_of allowlists)" "${check[@]}" allowlists "$base_tree" "$wt"
}

gate_newconsts() {
  need_base_tree newconsts
  verdict_of newconsts "$(log_of newconsts)" "$(log_of newconsts)" "${check[@]}" newconsts "$base_tree" "$wt"
}

gate_review() {
  if [[ -z $review ]]; then record review FAIL "no --review file given"; return; fi
  verdict_of review "$(log_of review)" "$(log_of review)" "${check[@]}" verdict "$review"
}

for g in "${run_list[@]}"; do "gate_$g"; done

echo "gate file: $gate_file"
((failed_now == 0))
