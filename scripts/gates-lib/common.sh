# Shared helpers for the gate scripts (sourced, not run). Needs bash 4+.

# the main checkout: no script here builds in it or runs in it (the tests point this at a scratch tree)
MAIN_CHECKOUT=${MAIN_CHECKOUT:-/home/joseph/projects/commander-sim}
GATES_LOCK=${GATES_LOCK:-/tmp/cargo-heavy.lock}
DRY=${DRY:-0}

die() { echo "${GATES_PROG:-gates}: $*" >&2; exit 2; }

# the absolute path of a directory, or empty when it does not exist
abspath() { (cd "$1" 2>/dev/null && pwd -P) || true; }

# a tree the scripts may build in: it exists, is a git work tree, and is not the main checkout
check_tree() {
  local t=$1 real
  [[ -n $t && -d $t ]] || die "not a directory: ${t:-<empty>}"
  real=$(abspath "$t")
  [[ $real != "$MAIN_CHECKOUT" && $real != "$MAIN_CHECKOUT"/rust* ]] || die "refusing to touch the main checkout: $t"
  git -C "$real" rev-parse --git-dir >/dev/null 2>&1 || die "not a git work tree: $t"
  echo "$real"
}

# heavy TREE CMD...: run a cargo command from TREE/rust under the shared lock and the memory cap, with the tree's own
# target directory. With DRY=1 only print it.
heavy() {
  local tree=$1; shift
  local target=$tree/rust/target
  [[ $target != "$MAIN_CHECKOUT"/* ]] || die "target dir would be the main checkout's: $target"
  if [[ $DRY == 1 ]]; then
    echo "DRY: (cd $tree/rust && flock $GATES_LOCK systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 env CARGO_TARGET_DIR=$target $*)"
    return 0
  fi
  (cd "$tree/rust" && flock "$GATES_LOCK" systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    env CARGO_TARGET_DIR="$target" "$@")
}

# short hash of a commit in a tree
short() { git -C "$1" rev-parse --short=12 "$2" 2>/dev/null || echo "?"; }
