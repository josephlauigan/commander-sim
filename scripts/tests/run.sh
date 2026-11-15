#!/usr/bin/env bash
# Tests of the gate scripts. Plays no game and runs no cargo: `cargo` is a stand-in (fakebin/cargo) that prints canned
# output from fixtures/, `flock` and `systemd-run` are pass-through stand-ins, and `commander-sim` is a stub that prints
# a made-up game log. Run: bash scripts/tests/run.sh   (exit 0 = all pass)
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
scripts="$(cd "$here/.." && pwd)"
fx=$here/fixtures
tmp=$(mktemp -d /tmp/gates-test.XXXXXX)
trap 'rm -rf "$tmp"' EXIT
export PATH="$here/fakebin:$PATH"   # the stand-ins first: a real cargo or flock can never be reached by these tests
export FAKE_DIR=$tmp/fake
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
gc=(python3 -I "$scripts/gates-lib/gatecheck.py")
tc=(python3 -I "$scripts/trace-compare.py")
npass=0 nfail=0

ok() { npass=$((npass + 1)); }
bad() { nfail=$((nfail + 1)); echo "FAIL: $1"; [[ -n ${2:-} ]] && sed 's/^/    | /' "$2" | head -n 25; }

# expect NAME RC CMD...: the command's exit code must be RC; its output is kept in $tmp/out
expect() {
  local name=$1 want=$2; shift 2
  "$@" >"$tmp/out" 2>&1; local rc=$?
  if [[ $rc == "$want" ]]; then ok; else bad "$name (exit $rc, wanted $want)" "$tmp/out"; fi
}
# has NAME PATTERN [FILE]: the file (default: the last output) must match the extended regex
has() { if grep -qE -- "$2" "${3:-$tmp/out}"; then ok; else bad "$1 (no match for: $2)" "${3:-$tmp/out}"; fi; }
hasnt() { if grep -qE -- "$2" "${3:-$tmp/out}"; then bad "$1 (unexpected match for: $2)" "${3:-$tmp/out}"; else ok; fi; }

commit() { git -C "$1" add -A && git -C "$1" commit -q -m "$2"; }

# a copy of the base fixture tree, as a git repo with one commit
mkbase() {
  mkdir -p "$1" && cp -R "$fx/base-tree/." "$1/"
  find "$1" -name '*.fixture' | while read -r f; do mv "$f" "${f%.fixture}"; done
  git -C "$1" init -q && commit "$1" base
}
mkbranch() { git clone -q "$1" "$2" && git -C "$2" checkout -q -b work; }
sub() { python3 -I -c 'import sys;p,a,b=sys.argv[1:];s=open(p).read();assert a in s,a;open(p,"w").write(s.replace(a,b,1))' "$@"; }

tests=rust/crates/sim-core/tests

# =============================================================== gatecheck: allow-lists
echo "== allow-lists and new consts"
mkbase "$tmp/base"
expect "allowlists: identical trees pass" 0 "${gc[@]}" allowlists "$tmp/base" "$tmp/base"
has "allowlists: counts the entries" "^[0-9]+ frozen entries"

rm -rf "$tmp/b1"; mkbranch "$tmp/base" "$tmp/b1"
sub "$tmp/b1/$tests/practice_choice_probe.rs" '    ("aragorn: Deflecting Swat", "new targets for a spell on the stack"), // trailing comment ("not", "an entry")
' ''
expect "allowlists: a removed entry passes" 0 "${gc[@]}" allowlists "$tmp/base" "$tmp/b1"

rm -rf "$tmp/b2"; mkbranch "$tmp/base" "$tmp/b2"
sub "$tmp/b2/$tests/practice_choice_probe.rs" '    /* block' '    ("sheoldred: New Thing", "a new excuse"),
    /* block'
expect "allowlists: a new FALSE_ALARMS entry fails" 1 "${gc[@]}" allowlists "$tmp/base" "$tmp/b2"
has "allowlists: names the new entry" "NEW entry: sheoldred: New Thing"

rm -rf "$tmp/b3"; mkbranch "$tmp/base" "$tmp/b3"
sub "$tmp/b3/$tests/practice_choice_probe.rs" 'new targets for a spell on the stack"),
];' 'new targets, reworded"),
];'
expect "allowlists: a changed reason fails" 1 "${gc[@]}" allowlists "$tmp/base" "$tmp/b3"
has "allowlists: names the changed reason" "CHANGED reason: thanos: Deflecting Swat"

rm -rf "$tmp/b4"; mkbranch "$tmp/base" "$tmp/b4"
sub "$tmp/b4/$tests/spellbook.rs" '    ("krenko", "38-659-2816"),     // Skirk Prospector' '    ("krenko", "38-659-2816"),     // Skirk Prospector
    ("krenko", "999-999"),         // a new variant under a known deck'
expect "allowlists: a new KNOWN_GAPS variant of a known deck fails" 1 "${gc[@]}" allowlists "$tmp/base" "$tmp/b4"
has "allowlists: shows both key literals" "krenko / 999-999"

rm -rf "$tmp/b5"; mkbranch "$tmp/base" "$tmp/b5"
sub "$tmp/b5/$tests/model_gaps_a4.rs" '    "Battle Cry Goblin",' '    "Battle Cry Goblin",
    "Gilded Goose",'
expect "allowlists: a new NO_CHOICE name (single string entries) fails" 1 "${gc[@]}" allowlists "$tmp/base" "$tmp/b5"

rm -rf "$tmp/b6"; mkbranch "$tmp/base" "$tmp/b6"
sub "$tmp/b6/$tests/practice_choice_probe.rs" 'const TO_FIX: &[&str] = &[];' 'const TO_FIX: &[&str] = &["yshtola: Something"];'
expect "allowlists: a new TO_FIX entry fails" 1 "${gc[@]}" allowlists "$tmp/base" "$tmp/b6"

rm -rf "$tmp/b7"; mkbranch "$tmp/base" "$tmp/b7"
cat >"$tmp/b7/$tests/practice_ability_probe.rs" <<'EOF'
const FALSE_ALARMS: &[(&str, &str)] = &[("x: y", "z")];
EOF
expect "allowlists: a const in a file the base lacks fails" 1 "${gc[@]}" allowlists "$tmp/base" "$tmp/b7"

rm -rf "$tmp/b8"; mkbranch "$tmp/base" "$tmp/b8"
sub "$tmp/b8/$tests/model_gaps_a4.rs" '    ("Resplendent Angel", "+2/+2 and lifelink: nothing to choose"),
' ''
sub "$tmp/b8/$tests/model_gaps_a4.rs" '    ("Balan, Wandering Knight", "attaches every Equipment: nothing to choose"),' '    // reordered below
    ("Balan, Wandering Knight", "attaches every Equipment: nothing to choose"),
    ("Resplendent Angel", "+2/+2 and lifelink: nothing to choose"),'
expect "allowlists: a reorder and a comment pass" 0 "${gc[@]}" allowlists "$tmp/base" "$tmp/b8"

expect "newconsts: identical trees pass (the base's own QUIET is not new)" 0 "${gc[@]}" newconsts "$tmp/base" "$tmp/base"
rm -rf "$tmp/n1"; mkbranch "$tmp/base" "$tmp/n1"
printf 'const KNOWN_SKIPS: &[&str] = &[];\n' >"$tmp/n1/rust/crates/sim-core/src/extra.rs"
expect "newconsts: a new SKIP const fails" 1 "${gc[@]}" newconsts "$tmp/base" "$tmp/n1"
has "newconsts: names it" "new const KNOWN_SKIPS in rust/crates/sim-core/src/extra.rs"
rm -rf "$tmp/n2"; mkbranch "$tmp/base" "$tmp/n2"
printf 'fn f() {\n    pub(crate) const NO_THING: u32 = 1;\n}\n' >"$tmp/n2/rust/crates/sim-core/src/extra.rs"
expect "newconsts: an indented NO_ const in a function fails" 1 "${gc[@]}" newconsts "$tmp/base" "$tmp/n2"
rm -rf "$tmp/n3"; mkbranch "$tmp/base" "$tmp/n3"
printf 'const LIMIT: u32 = 1;\nstatic ALLOWED_MAX: u32 = 2;\n' >"$tmp/n3/rust/crates/sim-core/src/extra.rs"
expect "newconsts: a static named ALLOW* fails" 1 "${gc[@]}" newconsts "$tmp/base" "$tmp/n3"
rm -rf "$tmp/n4"; mkbranch "$tmp/base" "$tmp/n4"
printf 'const LIMIT: u32 = 1;\nconst FIXED_SEED: u64 = 2;\n' >"$tmp/n4/rust/crates/sim-core/src/extra.rs"
expect "newconsts: FIXED_SEED contains FIX: fails (the pattern is a substring match)" 1 "${gc[@]}" newconsts "$tmp/base" "$tmp/n4"
rm -rf "$tmp/n5"; mkbranch "$tmp/base" "$tmp/n5"
printf 'const LIMIT: u32 = 1;\n' >"$tmp/n5/rust/crates/sim-core/src/extra.rs"
expect "newconsts: an unrelated const passes" 0 "${gc[@]}" newconsts "$tmp/base" "$tmp/n5"

# =============================================================== gatecheck: lists, suite, verdict, audits
echo "== lists, suite, verdict, audits"
l=$fx/cargo
expect "ignored: same list passes" 0 "${gc[@]}" ignored "$l/ok.ignored" "$l/ok.ignored" 6 6
expect "ignored: fewer ignored tests pass" 0 "${gc[@]}" ignored "$l/more.ignored" "$l/ok.ignored" 6 5
expect "ignored: a newly ignored test fails" 1 "${gc[@]}" ignored "$l/ok.ignored" "$l/more.ignored" 6 6
has "ignored: names it" "ignored in the branch only: cards_x::epsilon"
expect "ignored: the #[ignore count above the base fails" 1 "${gc[@]}" ignored "$l/ok.ignored" "$l/ok.ignored" 6 7
has "ignored: reports the count" "count 7 is above the base's 6"

expect "names: identical sets pass" 0 "${gc[@]}" names "$l/ok.list" "$l/ok.list"
expect "names: an undeclared deletion fails" 1 "${gc[@]}" names "$l/ok.list" "$l/deleted.list"
has "names: names the deleted test" "deleted and not declared: cards_x::delta"
expect "names: the declared deletion passes" 0 "${gc[@]}" names "$l/ok.list" "$l/deleted.list" "$fx/declared/delta.txt"
expect "names: a bare declared name passes" 0 "${gc[@]}" names "$l/ok.list" "$l/deleted.list" "$fx/declared/delta-bare.txt"
expect "names: a declaration of a test not deleted fails" 1 "${gc[@]}" names "$l/ok.list" "$l/deleted.list" "$fx/declared/too-many.txt"
has "names: names it" "declared but not deleted: something_else"
expect "names: a declared deletion that did not happen fails" 1 "${gc[@]}" names "$l/ok.list" "$l/ok.list" "$fx/declared/delta.txt"
expect "names: deleting and adding one test is not a net zero" 1 "${gc[@]}" names "$l/ok.list" "$l/deleted.list" "$fx/declared/none.txt"
printf 'nothing here\n' >"$tmp/empty.list"
expect "names: an empty base list fails" 1 "${gc[@]}" names "$tmp/empty.list" "$l/ok.list"
has "names: counts added tests" "1 deleted \(declared 1\), 1 added" <("${gc[@]}" names "$l/ok.list" "$l/deleted.list" "$fx/declared/delta.txt")

expect "suite: all ok passes" 0 "${gc[@]}" suite "$l/ok.suite"
has "suite: sums the binaries" "0 failed, 4 passed, 1 ignored over 2 binaries"
expect "suite: a failure fails" 1 "${gc[@]}" suite "$l/bad.suite"
has "suite: names the failed test" "test gamma ... FAILED"
printf 'error: could not compile\n' >"$tmp/nobuild.log"
expect "suite: a log with no result line fails" 1 "${gc[@]}" suite "$tmp/nobuild.log"

expect "verdict: VERDICT: commit passes" 0 "${gc[@]}" verdict "$fx/reviews/commit.md"
expect "verdict: fix-first fails" 1 "${gc[@]}" verdict "$fx/reviews/fixfirst.md"
expect "verdict: the verdict must be on the first line" 1 "${gc[@]}" verdict "$fx/reviews/late.md"
expect "verdict: a missing file fails" 1 "${gc[@]}" verdict "$tmp/none.md"

expect "audit-mine: all Full passes" 0 "${gc[@]}" audit-mine "$l/ok.audit-mine"
expect "audit-mine: a card below Full fails" 1 "${gc[@]}" audit-mine "$l/bad.audit-mine"
has "audit-mine: names the deck" "urdragon: 2 not Full"
expect "audit-paths: no doubles passes (declared paths_ok splits are fine)" 0 "${gc[@]}" audit-paths "$l/ok.audit-paths"
expect "audit-paths: a double fails" 1 "${gc[@]}" audit-paths "$l/bad.audit-paths"
has "audit-paths: names it" "double: competing = 1"

# =============================================================== stage-gate.sh with the stand-in cargo
echo "== stage-gate.sh"
mkdir -p "$FAKE_DIR/cargo-out"
reset_fake() {
  rm -rf "$FAKE_DIR"; mkdir -p "$FAKE_DIR/cargo-out"; cp -R "$fx/cargo-out/." "$FAKE_DIR/cargo-out/"
  for f in "$fx"/cargo/ok.*; do
    ext=${f##*/ok.}; cp "$f" "$FAKE_DIR/base.$ext"; cp "$f" "$FAKE_DIR/branch.$ext"
  done
}
reset_fake
rm -rf "$tmp/branch"; mkbranch "$tmp/base" "$tmp/branch"
git -C "$tmp/branch" commit -q --allow-empty -m "work"
base=$(git -C "$tmp/base" rev-parse HEAD)
gd=$tmp/gates
sg=("$scripts/stage-gate.sh" "$tmp/branch" --base "$base" --pr t1 --base-tree "$tmp/base" --gates-dir "$gd" --review "$fx/reviews/commit.md")

expect "stage-gate: every gate passes, exit 0" 0 "${sg[@]}"
for g in suite ignored names diffcheck fmt audit-mine audit-paths validate export allowlists newconsts review; do
  has "stage-gate: $g line is PASS with evidence" "^PASS $g: .*evidence: $gd/t1-" "$gd/t1.md"
done
has "stage-gate: the file ends RESULT: PASS" "^RESULT: PASS" "$gd/t1.md"
has "stage-gate: cargo ran with the worktree's own target dir" "branch PWD=$tmp/branch/rust CARGO_TARGET_DIR=$tmp/branch/rust/target" "$FAKE_DIR/calls.log"
has "stage-gate: the base ran with the base tree's own target dir" "base PWD=$tmp/base/rust CARGO_TARGET_DIR=$tmp/base/rust/target" "$FAKE_DIR/calls.log"
hasnt "stage-gate: cargo never ran in the main checkout" "/home/joseph/projects/commander-sim/rust" "$FAKE_DIR/calls.log"
has "stage-gate: the base list is cached per base commit" "." "$gd/base-${base:0:12}.list"

# one failing gate each
fail_gate() { # NAME GATE SETUP-COMMAND...: run the gate script with a broken stand-in; the gate must FAIL and the exit be 1
  local name=$1 gate=$2; shift 2
  reset_fake; rm -rf "$gd"; "$@"
  "${sg[@]}" --only "$gate" >"$tmp/out" 2>&1; local rc=$?
  if [[ $rc == 1 ]] && grep -q "^FAIL $gate: " "$tmp/out"; then ok; else bad "$name (exit $rc)" "$tmp/out"; fi
}
fail_gate "stage-gate: a failing test fails the suite gate" suite cp "$fx/cargo/bad.suite" "$FAKE_DIR/branch.suite"
fail_gate "stage-gate: a newly ignored test fails" ignored cp "$fx/cargo/more.ignored" "$FAKE_DIR/branch.ignored"
fail_gate "stage-gate: an undeclared deleted test fails" names cp "$fx/cargo/deleted.list" "$FAKE_DIR/branch.list"
fail_gate "stage-gate: a card below Full fails" audit-mine cp "$fx/cargo/bad.audit-mine" "$FAKE_DIR/branch.audit-mine"
fail_gate "stage-gate: a double fails" audit-paths cp "$fx/cargo/bad.audit-paths" "$FAKE_DIR/branch.audit-paths"
fail_gate "stage-gate: a failing validate fails" validate cp "$fx/cargo/bad.validate" "$FAKE_DIR/branch.validate"

reset_fake; rm -rf "$gd"
FAKE_DIRTY=1 "${sg[@]}" --only export >"$tmp/out" 2>&1; rc=$?
if [[ $rc == 1 ]] && grep -q "^FAIL export: the export changed 2 paths (tree restored)" "$tmp/out"; then ok; else bad "stage-gate: a changed export fails (exit $rc)" "$tmp/out"; fi
expect "stage-gate: the tree is restored after a failed export" 0 test -z "$(git -C "$tmp/branch" status --porcelain)"
touch "$tmp/branch/untracked.txt"; reset_fake; rm -rf "$gd"
expect "stage-gate: a dirty tree fails the export gate" 1 "${sg[@]}" --only export
has "stage-gate: says commit first" "not clean before the export"
rm -f "$tmp/branch/untracked.txt"

# declared deletions
reset_fake; rm -rf "$gd"; cp "$fx/cargo/deleted.list" "$FAKE_DIR/branch.list"
expect "stage-gate: a declared deletion passes" 0 "${sg[@]}" --only names --declared-deletions "$fx/declared/delta.txt"
# review
rm -rf "$gd"
expect "stage-gate: fix-first review fails" 1 "$scripts/stage-gate.sh" "$tmp/branch" --base "$base" --pr t1 --gates-dir "$gd" --only review --review "$fx/reviews/fixfirst.md"
rm -rf "$gd"
expect "stage-gate: no review file fails" 1 "$scripts/stage-gate.sh" "$tmp/branch" --base "$base" --pr t1 --gates-dir "$gd" --only review
# whitespace and fmt on a touched .rs file
rm -rf "$tmp/branch2"; mkbranch "$tmp/base" "$tmp/branch2"
printf 'fn main() {\n    let x = 1;  \n}\n' >"$tmp/branch2/rust/crates/sim-core/src/ws.rs"; commit "$tmp/branch2" "ws"
rm -rf "$gd"
expect "stage-gate: trailing whitespace fails diffcheck" 1 "$scripts/stage-gate.sh" "$tmp/branch2" --base "$base" --pr t2 --gates-dir "$gd" --only diffcheck
printf 'fn main() {\n  let x   =  1;\n}\n' >"$tmp/branch2/rust/crates/sim-core/src/ws.rs"; commit "$tmp/branch2" "ugly"
expect "stage-gate: an unformatted touched .rs fails fmt" 1 "$scripts/stage-gate.sh" "$tmp/branch2" --base "$base" --pr t2 --gates-dir "$gd" --only fmt
printf 'fn main() {\n    let x = 1;\n}\n' >"$tmp/branch2/rust/crates/sim-core/src/ws.rs"; commit "$tmp/branch2" "pretty"
expect "stage-gate: a formatted touched .rs passes fmt" 0 "$scripts/stage-gate.sh" "$tmp/branch2" --base "$base" --pr t2 --gates-dir "$gd" --only fmt
# --only keeps the other lines of the gate file
rm -rf "$gd"; reset_fake
"${sg[@]}" >/dev/null 2>&1; cp "$fx/cargo/bad.validate" "$FAKE_DIR/branch.validate"
"${sg[@]}" --only validate >/dev/null 2>&1
has "stage-gate: --only keeps the other gates' lines" "^PASS suite: " "$gd/t1.md"
has "stage-gate: --only updates its own line" "^FAIL validate: " "$gd/t1.md"
has "stage-gate: the result turns FAIL" "^RESULT: FAIL \(1 failing\)" "$gd/t1.md"
# a partial run is INCOMPLETE
rm -rf "$gd"; reset_fake
"${sg[@]}" --only diffcheck >/dev/null 2>&1
has "stage-gate: a partial gate file is INCOMPLETE" "^RESULT: INCOMPLETE" "$gd/t1.md"
# the test list for a stale base tree is refused
rm -rf "$tmp/stale"; mkbase "$tmp/stale"; git -C "$tmp/stale" commit -q --allow-empty -m "other"
expect "stage-gate: a base tree not at --base is refused" 2 "$scripts/stage-gate.sh" "$tmp/branch" --base "$base" --pr t1 --gates-dir "$gd" --base-tree "$tmp/stale" --only newconsts
# the main checkout is refused
expect "stage-gate: the main checkout is refused" 2 env MAIN_CHECKOUT="$tmp/branch" "${sg[@]}" --only diffcheck
has "stage-gate: says why" "refusing to touch the main checkout"
# dry run: prints the commands, runs none
rm -rf "$gd"; reset_fake
expect "stage-gate --dry-run exits 0" 0 "${sg[@]}" --dry-run
has "dry-run: the lock and cap are in the command" "DRY: .*flock /tmp/cargo-heavy.lock systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 env CARGO_TARGET_DIR=$tmp/branch/rust/target cargo test --release --workspace --no-fail-fast"
has "dry-run: cardgen runs through cargo too" "DRY: .*cargo run --release -q -p cardgen -- --root $tmp/branch audit --mine"
has "dry-run: the list command" "cargo test --release --workspace --no-fail-fast -- --list --ignored"
has "dry-run: writes its own file" "^SKIP suite" "$gd/t1.dry-run.md"
expect "dry-run: leaves the real gate file alone" 1 test -e "$gd/t1.md"
expect "dry-run: called no cargo" 1 test -e "$FAKE_DIR/calls.log"

# =============================================================== red-green.sh
echo "== red-green.sh"
reset_fake
rg=$scripts/red-green.sh
rm -rf "$tmp/scratch"; mkdir -p "$tmp/scratch/rust"; git -C "$tmp/scratch" init -q
mkc() { printf '%s\n' "$2" >"$tmp/scratch/behavior.txt"; commit "$tmp/scratch" "$1"; git -C "$tmp/scratch" rev-parse HEAD; }
c_red=$(mkc red red); c_green=$(mkc green green); c_compile=$(mkc compile compile); c_none=$(mkc none noharness); c_red2=$(mkc red2 red)
led=$tmp/ledger/t.md
expect "red-green: R fails by assertion, G passes" 0 "$rg" --tree "$tmp/scratch" --red "$c_red" --green "$c_green" --test the::fix_test --ledger "$led"
has "red-green: the row has both hashes and the evidence" "^\| the::fix_test \| ${c_red:0:12} \| ${c_green:0:12} \| FAILED at ${c_red:0:12}: .*panicked at .*\| ok at ${c_green:0:12} " "$led"
has "red-green: the header is written once" "^\| test \| R \| G \|" "$led"
has "red-green: cargo ran in the scratch tree's own target dir, filtered on the test" "scratch PWD=$tmp/scratch/rust CARGO_TARGET_DIR=$tmp/scratch/rust/target ARGS=test --release --no-fail-fast the::fix_test" "$FAKE_DIR/calls.log"
expect "red-green: a second run appends a row" 0 "$rg" --tree "$tmp/scratch" --red "$c_red" --green "$c_green" --test fix_test --ledger "$led"
expect "red-green: the ledger has the header and two rows" 0 test "$(wc -l <"$led")" -eq 4
expect "red-green: a compile error at R is not red" 3 "$rg" --tree "$tmp/scratch" --red "$c_compile" --green "$c_green" --test the::fix_test --ledger "$led"
has "red-green: says so" "NOT RED: R does not compile"
expect "red-green: a passing test at R is not red" 1 "$rg" --tree "$tmp/scratch" --red "$c_green" --green "$c_red" --test the::fix_test --ledger "$led"
has "red-green: says not red" "NOT RED: at R a matched test did not fail|NOT RED"
expect "red-green: no matching test at R is not red" 1 "$rg" --tree "$tmp/scratch" --red "$c_none" --green "$c_green" --test the::fix_test --ledger "$led"
has "red-green: says none ran" "no test named the::fix_test ran at R"
expect "red-green: a failing test at G is not green" 1 "$rg" --tree "$tmp/scratch" --red "$c_red" --green "$c_red2" --test the::fix_test --ledger "$led"
expect "red-green: R and G the same commit is refused" 2 "$rg" --tree "$tmp/scratch" --red "$c_red" --green "$c_red" --test the::fix_test --ledger "$led"
expect "red-green: G that does not compile is exit 3" 3 "$rg" --tree "$tmp/scratch" --red "$c_red" --green "$c_compile" --test the::fix_test --ledger "$led"
expect "red-green: a failed proof writes no row" 0 test "$(wc -l <"$led")" -eq 4
touch "$tmp/scratch/dirty"
expect "red-green: a dirty scratch tree is refused" 2 "$rg" --tree "$tmp/scratch" --red "$c_red" --green "$c_green" --test the::fix_test --ledger "$led"
rm "$tmp/scratch/dirty"
expect "red-green: the main checkout is refused" 2 env MAIN_CHECKOUT="$tmp/scratch" "$rg" --tree "$tmp/scratch" --red "$c_red" --green "$c_green" --test the::fix_test --ledger "$led"
expect "red-green: --coverage writes a coverage row" 0 "$rg" --ledger "$led" --test some_card_test --coverage
has "red-green: the coverage row" "^\| some_card_test \| - \| - \| coverage, no red/green \|" "$led"
rm -f "$FAKE_DIR/calls.log"
expect "red-green --dry-run prints the commands" 0 "$rg" --tree "$tmp/scratch" --red "$c_red" --green "$c_green" --test the::fix_test --ledger "$tmp/ledger/dry.md" --dry-run
has "red-green dry-run: the capped command" "DRY: .*flock /tmp/cargo-heavy.lock systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 env CARGO_TARGET_DIR=$tmp/scratch/rust/target cargo test --release --no-fail-fast the::fix_test"
expect "red-green dry-run: called no cargo" 1 test -e "$FAKE_DIR/calls.log"

# =============================================================== trace-record.sh and trace-compare.py
echo "== trace-record.sh and trace-compare.py"
# a stand-in for commander-sim: prints a made-up game log; $COMMANDER_SIM_ROOT/variant.txt picks the behaviour
mkbin() {
  cat >"$1" <<'EOF'
#!/usr/bin/env bash
variant=$(cat "$COMMANDER_SIM_ROOT/variant.txt")
deck=within pool="" k=0 gold=""
while (($#)); do
  case $1 in
    --deck) deck=$2; shift 2 ;;
    --pool) pool=$2; shift 2 ;;
    --trace) k=$2; shift 2 ;;
    --goldfish) gold=$2; shift 2 ;;
    *) shift ;;
  esac
done
if [[ -n $gold ]]; then
  echo "=== $gold goldfish [lookahead AI, 500 games] ==="
  if [[ $variant == slow ]]; then m=5; else m=4; fi
  echo "Kill turn (own turn): median $m.0, mean 4.3 (over the 490 kills)"
  echo "How it won (share of kills): Garth loop 61.2%, Combat 38.8%"
  exit 0
fi
cast="casts Foo"
[[ $variant == reword ]] && cast="plays Foo"
decides="R1  [$deck decides: A 62%, B 38% -> A [search]]"
[[ $variant == diag && $deck == yshtola ]] && decides="R1  [$deck decides: A 70%, B 30% -> A [search]]"
winner=$deck wintype=Combo last=3 act="attacks"
[[ $variant == play && $deck == yshtola && $k == 1 ]] && act="passes"
[[ $variant == result && $deck == yshtola && $k == 2 ]] && { winner="Opp A"; wintype=Damage; }
if [[ $variant == panic && $deck == yshtola && $k == 1 ]]; then echo "panic: boom (at engine.rs:1)" >&2; exit 1; fi
if [[ $variant == timeout && $deck == yshtola && $k == 2 ]]; then winner=none wintype=timeout last=20; fi
if [[ $variant == nowinner && $deck == yshtola && $k == 3 ]]; then winner=none wintype=None; fi
echo "Play-by-play of pool game $k (seed $((500000 + k))): $deck vs $pool, loose profile, lookahead AI"
echo "Seat order: $deck, Opp A, Opp B, Opp C"
echo "R1  $deck draws a card"
echo "$decides"
if (( k % 2 == 0 )) && [[ $variant != nocast ]]; then echo "R2  $deck $cast"; fi
[[ $variant == envdump ]] && echo "R2  env AI_SEARCH=${AI_SEARCH:-unset} ROOT=$COMMANDER_SIM_ROOT"
echo "R$last  $deck $act"
echo "Winner: $winner ($wintype)"
EOF
  chmod +x "$1"
}
decks_toml='[aragorn]
tiers = ["t4", "t5"]

[yshtola]
tiers = ["t5"]
'
# mktree DIR VARIANT [decks.toml text]: a git repo shaped like a worktree with the stand-in binary built
mktree() {
  rm -rf "$1"; mkdir -p "$1/data" "$1/rust/target/release"
  printf '%s' "${3:-$decks_toml}" >"$1/data/my_decks.toml"
  printf '%s\n' "$2" >"$1/variant.txt"
  mkbin "$1/rust/target/release/commander-sim"
  printf 'rust/target/\n' >"$1/.gitignore"; git -C "$1" init -q; commit "$1" "tree $2"
}
tr_rec() { "$scripts/trace-record.sh" "$@"; }
export AI_SEARCH=9   # a stray variable in the caller's shell: the recorder must scrub it

mktree "$tmp/rb" base
printf 'fake\n' >"$tmp/rb/rust/target/release/other"  # ignored build output must not make the tree unclean
expect "record l2: base" 0 tr_rec --mode l2 --tree "$tmp/rb" --out "$tmp/rec/base" --seeds 3 -P 4
has "record l2: prints the games, the rate and the binary hash" "27 games in [0-9]+s .*games/s at -P 4\); 0 with a non-zero exit; binary sha256 [0-9a-f]{16}"
expect "record l2: 3 seeds x (3 deck-tiers + 6 within-tier pods) = 27 records" 0 test "$(wc -l <"$tmp/rec/base/records.jsonl")" -eq 27
expect "record l2: one log per game" 0 test "$(find "$tmp/rec/base/logs" -type f | wc -l)" -eq 27
has "record l2: the binary copy's hash matches the record" "$(sha256sum "$tmp/rec/base/commander-sim" | cut -d' ' -f1)" "$tmp/rec/base/records.jsonl"
has "record l2: the commit is the tree's" "$(git -C "$tmp/rb" rev-parse HEAD)" "$tmp/rec/base/meta.json"
rec_line=$(grep '"deck": "yshtola"' "$tmp/rec/base/records.jsonl" | head -n1)
for f in commit binary_sha256 exit stderr1 seat_order winner wintype last_round lines sha_full sha_play log k tier; do
  if grep -q "\"$f\":" <<<"$rec_line"; then ok; else bad "record l2: field $f missing in $rec_line"; fi
done
has "record l2: seat order from the log" '"seat_order": "yshtola, Opp A, Opp B, Opp C"' <(echo "$rec_line")
has "record l2: winner and wintype" '"winner": "yshtola"' <(echo "$rec_line")
has "record l2: last round" '"last_round": 3' <(echo "$rec_line")
has "record l2: within pods are recorded" '"deck": "within", "k": 0.*"tier": "t6"|"deck": "within".*"tier": "t6"' "$tmp/rec/base/records.jsonl"
mktree "$tmp/renv" envdump
tr_rec --mode l2 --tree "$tmp/renv" --out "$tmp/rec/env" --seeds 1 --decks yshtola >/dev/null 2>&1
has "record l2: the environment is scrubbed (AI_SEARCH does not reach the game)" "R2  env AI_SEARCH=unset ROOT=$tmp/renv" "$tmp/rec/env/logs/yshtola-t5-0.log"
expect "record l2: --decks restricts to one deck" 0 test "$(wc -l <"$tmp/rec/env/records.jsonl")" -eq 1
expect "record: refuses an existing recording" 2 tr_rec --mode l2 --tree "$tmp/rb" --out "$tmp/rec/base" --seeds 1
echo x >"$tmp/rb/dirty.txt"
expect "record: refuses a dirty tree" 2 tr_rec --mode l2 --tree "$tmp/rb" --out "$tmp/rec/dirty" --seeds 1
rm "$tmp/rb/dirty.txt"
expect "record: refuses the main checkout" 2 env MAIN_CHECKOUT="$tmp/rb" "$scripts/trace-record.sh" --mode l2 --tree "$tmp/rb" --out "$tmp/rec/m"
rm -rf "$tmp/rnobin"; mkdir -p "$tmp/rnobin/data"; : >"$tmp/rnobin/data/my_decks.toml"; git -C "$tmp/rnobin" init -q; commit "$tmp/rnobin" x
expect "record: refuses a tree with no built binary" 2 tr_rec --mode l2 --tree "$tmp/rnobin" --out "$tmp/rec/nb"
expect "record: needs a mode" 2 tr_rec --tree "$tmp/rb" --out "$tmp/rec/x"

# branch recordings, one variant each
for v in base diag play result panic reword; do
  mktree "$tmp/r-$v" "$v"; tr_rec --mode l2 --tree "$tmp/r-$v" --out "$tmp/rec/$v" --seeds 3 -P 4 >/dev/null 2>&1
done
mktree "$tmp/r-missing" base '[yshtola]
tiers = ["t5"]
'; tr_rec --mode l2 --tree "$tmp/r-missing" --out "$tmp/rec/missing" --seeds 3 >/dev/null 2>&1
mktree "$tmp/r-new" base "${decks_toml}
[newdeck]
tiers = [\"t3\"]
"; tr_rec --mode l2 --tree "$tmp/r-new" --out "$tmp/rec/new" --seeds 3 >/dev/null 2>&1
B=$tmp/rec/base

expect "compare: the same build is all SAME" 0 "${tc[@]}" "$B" "$tmp/rec/base"
mktree "$tmp/rb2" base; tr_rec --mode l2 --tree "$tmp/rb2" --out "$tmp/rec/base2" --seeds 3 -P 2 >/dev/null 2>&1
expect "compare: a second recording at another parallelism is all SAME" 0 "${tc[@]}" "$B" "$tmp/rec/base2"
has "compare: prints the class table and the totals" "^total +27 +0 +0 +0 +0 +0 +0"
has "compare: RESULT: PASS" "^RESULT: PASS \(0 failing, 0 allowed non-SAME, 27 of 27 SAME\)"

expect "compare: DIAG on a record fails" 1 "${tc[@]}" "$B" "$tmp/rec/diag"
has "compare: names the DIAG record" "DIAG +yshtola/t5 seed 0"
expect "compare: DIAG in an allowed deck passes" 0 "${tc[@]}" "$B" "$tmp/rec/diag" --allow yshtola
has "compare: lists the allowed non-SAME records" "allowed non-SAME records \(each needs a one-line reason\)"
has "compare: counts DIAG for the deck-tier" "^yshtola/t5 +0 +3 +0"

expect "compare: PLAY fails" 1 "${tc[@]}" "$B" "$tmp/rec/play"
has "compare: PLAY names the record" "PLAY +yshtola/t5 seed 1"
has "compare: PLAY gives the first differing line number" "first difference at base line 5, branch line 5"
has "compare: PLAY gives both lines" "base:   R3  yshtola attacks"
has "compare: PLAY gives the branch line" "branch: R3  yshtola passes"

expect "compare: RESULT fails" 1 "${tc[@]}" "$B" "$tmp/rec/result"
has "compare: RESULT names the record and both results" "RESULT +yshtola/t5 seed 2"
has "compare: RESULT shows the branch result" "branch \('Opp A', 'Damage', 3\)"

expect "compare: ERROR fails" 1 "${tc[@]}" "$B" "$tmp/rec/panic"
has "compare: ERROR carries the panic line" "branch exit 1: panic: boom"

expect "compare: an undeclared MISSING fails" 1 "${tc[@]}" "$B" "$tmp/rec/missing"
has "compare: MISSING in the table" "^aragorn/t4 +0 +0 +0 +0 +0 +3"
expect "compare: the declared removal passes" 0 "${tc[@]}" "$B" "$tmp/rec/missing" --expect-missing aragorn/t4 aragorn/t5
has "compare: reports the declared MISSING" "expected MISSING \(declared\): 6 records"
expect "compare: a partly declared removal still fails" 1 "${tc[@]}" "$B" "$tmp/rec/missing" --expect-missing aragorn/t4
has "compare: the other tier is the failure" "MISSING +aragorn/t5"
expect "compare: a whole deck can be declared" 0 "${tc[@]}" "$B" "$tmp/rec/missing" --expect-missing aragorn

expect "compare: NEW fails" 1 "${tc[@]}" "$B" "$tmp/rec/new"
has "compare: NEW in the table" "^newdeck/t3 +0 +0 +0 +0 +0 +0 +3"
expect "compare: NEW in an allowed deck passes" 0 "${tc[@]}" "$B" "$tmp/rec/new" --allow newdeck

expect "compare: a wording change is PLAY without a rewrite" 1 "${tc[@]}" "$B" "$tmp/rec/reword"
expect "compare: a declared rewrite of the base logs makes it SAME" 0 "${tc[@]}" "$B" "$tmp/rec/reword" --rewrite 'casts Foo -> plays Foo'
has "compare: names the rewrite" "rewrites applied to the base logs: casts Foo -> plays Foo"
expect "compare: a rewrite that does not apply changes nothing" 1 "${tc[@]}" "$B" "$tmp/rec/reword" --rewrite 'nothing here -> x'
expect "compare: an arrow without spaces splits too" 0 "${tc[@]}" "$B" "$tmp/rec/reword" --rewrite 'casts->plays'

# l3
for v in base panic timeout nowinner nocast; do
  mktree "$tmp/q-$v" "$v"; tr_rec --mode l3 --tree "$tmp/q-$v" --out "$tmp/rec/l3-$v" --seeds 4 --cards 'Foo;Bar' -P 4 >/dev/null 2>&1
done
mktree "$tmp/q-missing" base '[yshtola]
tiers = ["t5"]
'; tr_rec --mode l3 --tree "$tmp/q-missing" --out "$tmp/rec/l3-missing" --seeds 4 --cards 'Foo;Bar' >/dev/null 2>&1
L=$tmp/rec/l3-base
expect "record l3: 4 seeds x 3 deck-tiers = 12 records" 0 test "$(wc -l <"$L/records.jsonl")" -eq 12
hasnt "record l3: keeps no winner names" '"winner":' "$L/records.jsonl"
hasnt "record l3: keeps no within-tier pods" '"deck": "within"' "$L/records.jsonl"
expect "record l3: keeps no logs" 0 test "$(find "$L/logs" -type f | wc -l)" -eq 0
has "record l3: per-card cast flags" '"casts": \{"Bar": 0, "Foo": 1\}' "$L/records.jsonl"
expect "compare l3: the same build passes" 0 "${tc[@]}" "$L" "$tmp/rec/l3-base" --changed-cards Foo
has "compare l3: prints the cast counts" "Foo: total 6/6"
hasnt "compare l3: prints no win rate" "win rate|win share|wins"
expect "compare l3: a panic fails" 1 "${tc[@]}" "$L" "$tmp/rec/l3-panic"
has "compare l3: names the panic" "PANIC yshtola/t5 seed 1: exit 1: panic: boom"
expect "compare l3: a new timeout fails" 1 "${tc[@]}" "$L" "$tmp/rec/l3-timeout"
has "compare l3: lists the seed that newly times out" "seeds that newly time out: yshtola/t5 seed 2"
has "compare l3: and the count above the base" "timeouts up in yshtola/t5: base 0, branch 1 of 4"
expect "compare l3: a new no-winner game fails" 1 "${tc[@]}" "$L" "$tmp/rec/l3-nowinner"
has "compare l3: names it" "no-winner games up in yshtola/t5: base 0, branch 1 of 4"
expect "compare l3: a changed card never cast fails" 1 "${tc[@]}" "$L" "$tmp/rec/l3-nocast" --changed-cards Foo
has "compare l3: says never cast" "changed card 'Foo' was never cast in the branch"
expect "compare l3: unless explained" 0 "${tc[@]}" "$L" "$tmp/rec/l3-nocast" --changed-cards Foo --explained Foo
expect "compare l3: without --changed-cards the casts are only reported" 0 "${tc[@]}" "$L" "$tmp/rec/l3-nocast"
expect "compare l3: a card not in the recording is a problem" 1 "${tc[@]}" "$L" "$tmp/rec/l3-base" --changed-cards Baz
expect "compare l3: an undeclared missing deck fails" 1 "${tc[@]}" "$L" "$tmp/rec/l3-missing"
expect "compare l3: a declared missing deck passes" 0 "${tc[@]}" "$L" "$tmp/rec/l3-missing" --expect-missing aragorn
expect "compare: an l3 and an l2 recording are not compared" 1 "${tc[@]}" "$L" "$B"

# goldfish
mktree "$tmp/g-base" base
expect "record goldfish: base" 0 tr_rec --mode goldfish --tree "$tmp/g-base" --out "$tmp/rec/gf-base" --decks urdragon
has "record goldfish: the mean" '"mean": "4.3"' "$tmp/rec/gf-base/goldfish.json"
has "record goldfish: route shares" '"Garth loop": 61.2' "$tmp/rec/gf-base/goldfish.json"
has "record goldfish: the median" '"median": "4.0"' "$tmp/rec/gf-base/goldfish.json"
has "record goldfish: kills" '"kills": 490' "$tmp/rec/gf-base/goldfish.json"

echo
echo "$npass passed, $nfail failed"
((nfail == 0))
