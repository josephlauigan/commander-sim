#!/usr/bin/env bash
# Record fixed-seed games for the L2 / L3 / goldfish gates. RUN BY THE COORDINATOR, never by an agent: it plays games.
# It takes no lock and sets no memory cap itself; wrap it:
#   flock /tmp/cargo-heavy.lock systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
#       scripts/trace-record.sh --mode l2 --tree <worktree> --out <dir>
#
#   scripts/trace-record.sh --mode l2|l3|goldfish --tree <worktree> --out <dir> [--seeds N] [--decks LIST]
#                           [--cards 'A;B'] [--cards-file FILE] [-P N]
#     l2        20 seeds x (each deck-tier of the tree's data/my_decks.toml + each within-tier pod t1..t6); full records and logs
#     l3        200 seeds per deck-tier; keeps only exit, panic, timeout, winner-present, last round and the per-card
#               "casts <card>" flags (no winner names, no logs, no win shares); --cards lists the cards, separated by ';'
#     goldfish  `--goldfish <first deck of --decks, default urdragon> --games <seeds, default 500> --seed 500000`
#     --decks   comma list of deck keys; `within` selects the within-tier pods (l2 only); default all
#     -P N      games in parallel (default 16)
# The tree's release binary (<tree>/rust/target/release/commander-sim, built beforehand) is copied into <out> and its
# sha256 recorded; games run as `env -i PATH HOME COMMANDER_SIM_ROOT=<tree> <copy> ...`, one game per process. The tree's
# `git status --porcelain` must be clean. Output files: see gates-lib/record.py.
set -uo pipefail

GATES_PROG=trace-record.sh
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=gates-lib/common.sh
source "$here/gates-lib/common.sh"

mode="" tree="" out="" seeds="" decks="" cards="" cards_file="" par=16
while (($#)); do
  case $1 in
    --mode) mode=${2:?}; shift 2 ;;
    --tree) tree=${2:?}; shift 2 ;;
    --out) out=${2:?}; shift 2 ;;
    --seeds) seeds=${2:?}; shift 2 ;;
    --decks) decks=${2:?}; shift 2 ;;
    --cards) cards=${2:?}; shift 2 ;;
    --cards-file) cards_file=${2:?}; shift 2 ;;
    -P) par=${2:?}; shift 2 ;;
    -h|--help) sed -n '2,19p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 2 ;;
    *) die "unknown argument $1" ;;
  esac
done
[[ $mode == l2 || $mode == l3 || $mode == goldfish ]] || die "--mode must be l2, l3 or goldfish"
[[ -n $tree && -n $out ]] || die "--tree and --out are required"
tree=$(check_tree "$tree")
[[ -z $(git -C "$tree" status --porcelain) ]] || die "the tree is not clean (git status --porcelain): $tree"
bin=$tree/rust/target/release/commander-sim
[[ -x $bin ]] || die "no built binary: $bin"
[[ -f $tree/data/my_decks.toml ]] || die "no data/my_decks.toml in $tree"
[[ ! -e $out/records.jsonl && ! -e $out/goldfish.json ]] || die "$out already holds a recording; use a fresh directory"
if [[ -n $cards_file ]]; then
  [[ -f $cards_file ]] || die "no such cards file: $cards_file"
  while IFS= read -r c; do [[ -n $c && $c != \#* ]] && cards+="${cards:+;}$c"; done <"$cards_file"
fi

mkdir -p "$out" || die "cannot create $out"
out=$(abspath "$out")
cp "$bin" "$out/commander-sim" || die "cannot copy the binary"
echo "tree $tree at $(git -C "$tree" rev-parse --short=12 HEAD); binary sha256 $(sha256sum "$out/commander-sim" | cut -d' ' -f1)"

args=(--mode "$mode" --tree "$tree" --out "$out" --bin "$out/commander-sim" --commit "$(git -C "$tree" rev-parse HEAD)" -P "$par")
[[ -n $seeds ]] && args+=(--seeds "$seeds")
[[ -n $decks ]] && args+=(--decks "$decks")
[[ -n $cards ]] && args+=(--cards "$cards")
exec python3 -I "$here/gates-lib/record.py" "${args[@]}"
