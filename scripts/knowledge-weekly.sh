#!/usr/bin/env bash
# Weekly refresh of the card knowledge base (documents/card-knowledge.md). Run by the systemd user timer
# scripts/systemd/knowledge-weekly.timer, or by hand.
#
# Stages (each recorded ok / skip / fail with its time and a reason, and written into the report):
#   1 fetch-build   fetch the repository into a SEPARATE clone ($KNOWLEDGE_CACHE/repo), reset it to $KNOWLEDGE_REF
#                   (default origin/main), build cardgen there; the clone is cardgen's --root (decks, cache, FINDINGS)
#   2 bulk-sync     Scryfall's Oracle Cards bulk file, only if its updated_at changed (the previous catalog is kept as
#                   oracle.prev.json); refuses fewer than 25,000 cards
#   3 roles         rule roles (roles --rules), then the tracked LLM cache merged in (roles --merge, add-only), so
#                   roles.json keeps the reviewed LLM roles; the table before the run is kept for the report
#   4 llm           the local LLM for cards the rules could not read (NoRole or UnknownLine routes) that miss the
#                   cache, dense Qwen3.6-27B, thinking off, into llm_roles.pending.jsonl ONLY; starts the
#                   llama-27b user unit only if the GPU is free and port 8080 is not served, stops it only if it
#                   started it; 90-minute budget, the rest carries over to next week
#   5 map-digest    map.md and a digest for every deck of `mine` in data/decks.json
#   6 report        reports/YYYY-MM-DD.md (the newest 12 are kept)
#
# STAGING. Stages 2-6 never write into the published directory while they work. Each builds its outputs in a per-run
# staging directory ($KNOWLEDGE_CACHE/staging/RUNID/STAGE, the catalog and the tracked role files linked in) through
# CARDGEN_KNOWLEDGE_DIR, and the script publishes them with renames into $KNOWLEDGE_OUT only after the WHOLE stage
# succeeded (map and every digest; rules and merge; the bulk file with its prev and lock; the report). A failed or
# killed stage leaves $KNOWLEDGE_OUT byte-identical, with no temp files in it; the next run deletes abandoned staging
# directories. The LLM stage's finished batches are published to llm_roles.pending.jsonl at the end of the stage
# (also after a spent budget or a failure), so they carry over. Publishing renames a file within one filesystem; if
# the staging directory and $KNOWLEDGE_OUT are on different filesystems the file is first copied to a temp file in
# the target directory (a kill in that copy leaves a hidden *.tmp there; both default places are under $HOME).
#
# Output goes to $KNOWLEDGE_OUT (default: data/knowledge of the main checkout, $KNOWLEDGE_MAIN), never to the clone.
# Every file it writes there is gitignored (oracle.json, oracle.prev.json, sync.lock, roles.json, map.md, digests/,
# reports/, llm_roles.pending.jsonl). The tracked role files (roles.toml, roles_overrides.toml, llm_roles.jsonl) are only
# READ from there, so they must be checked out in the main checkout.
# Exit 0 whenever the report was written, 1 only if it could not be; 0 too if another run holds the lock.
#
# NEVER AUTOMATED: commits, pushes, PRs, writes to any tracked file, `cardgen export`, sims, swap or cut proposals,
# llama.cpp or model updates, merging pending LLM rows into llm_roles.jsonl, installing or enabling the timer.
#
# The LLM stage runs only inside the night window ($KNOWLEDGE_LLM_HOURS, default 0-7: 00:00 to 07:00 local; outside it
# the stage is skipped and the rest of the job still runs, so a catch-up after a missed Sunday 04:00 leaves the GPU to JD).
# The llama-27b unit: started only when nothing answers on the port, the GPU is free (other processes hold under
# $KNOWLEDGE_GPU_BUSY_MIB, default 1024 MiB, of its video memory; see gpu_state; an unreadable answer means "unknown" and
# skips) and the unit is inactive right before the start. Ownership is a marker
# file ($KNOWLEDGE_CACHE/owns-llama-27b: run id and the unit's InvocationID), written right after a start this run
# caused. The job stops the unit only if that marker exists (on exit, on TERM/INT/HUP, and from the service's
# ExecStopPost through `--cleanup`, which also covers a killed job); an already-active unit is never stopped. Known
# limit: another actor that starts the unit in the milliseconds between the final is-active check and `systemctl
# start` makes this job believe it started it; the unit's own RuntimeMaxSec bounds a server that is left behind.
#
# usage: scripts/knowledge-weekly.sh [--dry-run] [--stage N | --only NAME] [--cleanup] [--help]
#   --dry-run       print the configuration and what each stage would do; writes nothing, calls nothing
#   --stage N       run only stage N (1-6), or --only NAME (fetch-build, bulk-sync, roles, llm, map-digest, report);
#                   stages 2-6 use the binary and clone a full run (or stage 1) left in $KNOWLEDGE_CACHE
#   --cleanup       stop the llama-27b unit if (and only if) this job's ownership marker exists, then remove it
#
# Environment (all optional): KNOWLEDGE_MAIN, KNOWLEDGE_OUT, KNOWLEDGE_CACHE, KNOWLEDGE_REPO, KNOWLEDGE_REPO_URL,
# KNOWLEDGE_REF, KNOWLEDGE_GPU_DEVICE, KNOWLEDGE_LLM_URL, KNOWLEDGE_LLM_UNIT, KNOWLEDGE_LLM_BUDGET_SECS,
# KNOWLEDGE_LLM_READY_SECS, KNOWLEDGE_LLM_HOURS, KNOWLEDGE_GPU_BUSY_MIB, KNOWLEDGE_SKIP_LLM=1, KNOWLEDGE_KEEP_REPORTS; for tests: KNOWLEDGE_SKIP_FETCH=1,
# KNOWLEDGE_CARDGEN (a prebuilt binary), KNOWLEDGE_NOW_HOUR (the clock for the night window), KNOWLEDGE_PROC_ROOT, KNOWLEDGE_BULK_FILE + KNOWLEDGE_BULK_UPDATED_AT (a local bulk file instead of
# Scryfall), KNOWLEDGE_BULK_MIN, KNOWLEDGE_NO_SCOPE=1 (no systemd-run), KNOWLEDGE_DATE. Without HOME, the places derived
# from it (KNOWLEDGE_MAIN, KNOWLEDGE_OUT, KNOWLEDGE_CACHE, KNOWLEDGE_REPO) must be given.
#
# needs: bash, git, cargo, flock, jq, curl, systemd-run, ss, timeout. Optional: notify-send.
set -euo pipefail

# ------------------------------------------------------------------------------------------------ configuration

HOME_DIR="${HOME:-}"
MAIN="${KNOWLEDGE_MAIN:-${HOME_DIR:+$HOME_DIR/projects/commander-sim}}"
OUT="${KNOWLEDGE_OUT:-${MAIN:+$MAIN/data/knowledge}}"
if [[ -n ${KNOWLEDGE_CACHE:-} ]]; then
  CACHE="$KNOWLEDGE_CACHE"
elif [[ -n ${XDG_CACHE_HOME:-} ]]; then
  CACHE="$XDG_CACHE_HOME/commander-sim-knowledge"
elif [[ -n $HOME_DIR ]]; then
  CACHE="$HOME_DIR/.cache/commander-sim-knowledge"
else
  CACHE=""
fi
REPO="${KNOWLEDGE_REPO:-${CACHE:+$CACHE/repo}}"
REF="${KNOWLEDGE_REF:-origin/main}"
SKIP_FETCH="${KNOWLEDGE_SKIP_FETCH:-0}"
if [[ -n ${CARGO_TARGET_DIR:-} ]]; then
  TARGET_DIR="$CARGO_TARGET_DIR"
else
  TARGET_DIR="${REPO:+$REPO/rust/target}"
fi
if [[ -n $TARGET_DIR ]]; then export CARGO_TARGET_DIR="$TARGET_DIR"; fi
CARDGEN="${KNOWLEDGE_CARDGEN:-${TARGET_DIR:+$TARGET_DIR/release/cardgen}}"
CARGO_LOCK="${KNOWLEDGE_CARGO_LOCK:-/tmp/cargo-heavy.lock}"
NO_SCOPE="${KNOWLEDGE_NO_SCOPE:-0}"
DATE="${KNOWLEDGE_DATE:-$(date +%F)}"
BULK_MIN="${KNOWLEDGE_BULK_MIN:-25000}"
BULK_FILE="${KNOWLEDGE_BULK_FILE:-}"
BULK_UPDATED_AT="${KNOWLEDGE_BULK_UPDATED_AT:-}"
KEEP_REPORTS="${KNOWLEDGE_KEEP_REPORTS:-12}"
# the LLM stage: dense Qwen3.6-27B behind the llama-27b user unit (scripts/systemd/llama-27b.service). The GPU is the
# B70's render node, named by PCI address (see /dev/dri/by-path); something holding it open means it is busy.
SKIP_LLM="${KNOWLEDGE_SKIP_LLM:-0}"
GPU_DEVICE="${KNOWLEDGE_GPU_DEVICE:-/dev/dri/by-path/pci-0000:04:00.0-render}"
LLM_PORT="${KNOWLEDGE_LLM_PORT:-8080}"
LLM_URL="${KNOWLEDGE_LLM_URL:-http://127.0.0.1:$LLM_PORT}"
LLM_UNIT="${KNOWLEDGE_LLM_UNIT:-llama-27b}"
LLM_MODEL_MATCH="${KNOWLEDGE_LLM_MODEL_MATCH:-Qwen3.6-27B}"
LLM_MODEL_ID="${KNOWLEDGE_LLM_MODEL_ID:-Qwen3.6-27B-Q5_K_M.gguf}"
LLM_BUDGET="${KNOWLEDGE_LLM_BUDGET_SECS:-5400}"
LLM_READY="${KNOWLEDGE_LLM_READY_SECS:-300}"
LLM_POLL="${KNOWLEDGE_LLM_POLL_SECS:-5}"
export CARDGEN_LLM_VARIANT=think
# the whole argument list of the LLM stage's cardgen call (--llm is the mode; without it cardgen refuses the call)
LLM_ARGS=(roles --llm --routes NoRole,UnknownLine --only-routed --batch 10 --no-think --pending --endpoint "$LLM_URL/v1")

STAGE_NAMES=(fetch-build bulk-sync roles llm map-digest report)
DRY=0
ONLY=""
CLEANUP=0
STATE=""
RUN_ID="$DATE-$$"
RUN_DIR=""
STAGING=""
STAGES_FILE=""
NOTE_FILE=""
OWNS="${CACHE:+$CACHE/owns-$LLM_UNIT}"
TERMINATED=0
STAGE_PID=""
KDIR=""
RUN_STATUS=""

die() {
  echo "knowledge-weekly: $*" >&2
  exit 2
}

# the memory cap every cargo and cardgen call runs under (exit 137 means the cap was hit)
if [[ $NO_SCOPE == 1 ]]; then
  SCOPE=()
else
  SCOPE=(systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0)
fi

# cardgen on the directory $KDIR (default: the published one) and the cloned repository
cg() {
  "${SCOPE[@]}" env CARDGEN_KNOWLEDGE_DIR="${KDIR:-$OUT}" "$CARDGEN" --root "$REPO" "$@"
}

# the stage's one-line result for the report; stages return 0 (ok), 10 (skip) or anything else (fail)
note() {
  printf '%s' "$*" >"$NOTE_FILE"
}

need_cardgen() {
  if [[ ! -x $CARDGEN ]]; then
    note "no cardgen binary at $CARDGEN (stage 1 failed or has not run)"
    return 10
  fi
  if [[ ! -d $REPO/data ]]; then
    note "no repository clone at $REPO (stage 1 failed or has not run)"
    return 10
  fi
}

# lines in a file, 0 if it does not exist
rows_in() {
  if [[ -f $1 ]]; then wc -l <"$1"; else echo 0; fi
}

commit_of_repo() {
  git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo unknown
}

# ------------------------------------------------------------------------------------------------ staging and publishing

# a fresh staging directory for a stage, printed
stage_dir() {
  mkdir -p "$STAGING/$1"
  printf '%s' "$STAGING/$1"
}

# Link every regular file of the published directory into a staging directory (so cardgen finds the catalog and the
# tracked role files beside it), except the names given after the directory: those are the stage's own outputs, which a
# program that writes in place (not by rename, as cardgen does) must not reach through a link.
link_inputs() {
  local dir="$1" f n skip
  shift
  mkdir -p "$dir"
  while IFS= read -r -d '' f; do
    n="${f##*/}"
    skip=0
    for x in "$@"; do
      if [[ $n == "$x" ]]; then skip=1; fi
    done
    if ((skip == 0)); then ln -s -- "$f" "$dir/$n"; fi
  done < <(find "$OUT" -maxdepth 1 -type f -print0 2>/dev/null)
}

same_fs() {
  [[ "$(stat -c %d -- "$1")" == "$(stat -c %d -- "$2")" ]]
}

# rename a finished file over its published name (a copy first, into a temp file beside the target, across filesystems)
publish_file() {
  local src="$1" dst="$2" dir tmp
  dir="$(dirname -- "$dst")"
  mkdir -p "$dir"
  if same_fs "$src" "$dir"; then
    mv -f -T -- "$src" "$dst"
  else
    tmp="$(mktemp "$dir/.publish.XXXXXX.tmp")" || return 1
    if cp -- "$src" "$tmp" && mv -f -T -- "$tmp" "$dst"; then
      rm -f -- "$src"
    else
      rm -f -- "$tmp"
      return 1
    fi
  fi
}

# a regular file a stage wrote itself (not a link to a published file)
produced() {
  [[ -f $1 && ! -L $1 ]]
}

# keep the newest $KEEP_REPORTS dated reports: DATE.md is the newest of its day, DATE.N.md (N a number) the ones it
# replaced, higher N later; everything else in reports/ is left alone
prune_reports() {
  local reports="$OUT/reports" names=() line extra i b
  mapfile -t names < <(
    find "$reports" -maxdepth 1 -type f -name '*.md' -printf '%f\n' 2>/dev/null | while IFS= read -r b; do
      if [[ $b =~ ^([0-9]{4}-[0-9]{2}-[0-9]{2})(\.([0-9]+))?\.md$ ]]; then
        printf '%s %s %s\n' "${BASH_REMATCH[1]}" "${BASH_REMATCH[3]:-99999999999999999999}" "$b"
      fi
    done | sort -k1,1 -k2,2n
  )
  extra=$((${#names[@]} - KEEP_REPORTS))
  for ((i = 0; i < extra; i++)); do
    line="${names[$i]}"
    b="${line##* }"
    rm -f -- "$reports/$b"
  done
}

# publish the staged report as reports/DATE.md: an earlier report of the day stays as DATE.N.md (a hard link, so
# DATE.md is never absent), then the retention
publish_report() {
  local staged="$1" reports="$OUT/reports" dst n max f tmp
  dst="$reports/$DATE.md"
  mkdir -p "$reports"
  tmp="$staged"
  if ! same_fs "$staged" "$reports"; then
    tmp="$(mktemp "$reports/.publish.XXXXXX.tmp")" || return 1
    cp -- "$staged" "$tmp" || {
      rm -f -- "$tmp"
      return 1
    }
  fi
  if [[ -e $dst ]]; then
    max=0
    for f in "$reports/$DATE".*.md; do
      [[ -e $f ]] || continue
      n="${f##*/}"
      n="${n#"$DATE".}"
      n="${n%.md}"
      if [[ $n =~ ^[0-9]+$ ]] && ((10#$n > max)); then max=$((10#$n)); fi
    done
    if ! ln -- "$dst" "$reports/$DATE.$((max + 1)).md" 2>/dev/null && ! cp -p -- "$dst" "$reports/$DATE.$((max + 1)).md"; then
      [[ $tmp == "$staged" ]] || rm -f -- "$tmp"
      return 1
    fi
  fi
  mv -f -T -- "$tmp" "$dst" || return 1
  prune_reports
}

publish_bulk() {
  local s="$STAGING/bulk-sync"
  # the previous catalog first, the lock before the catalog: a kill between two renames never leaves a catalog
  # that the lock and the bulk date would re-sync into a copy of itself
  if [[ -f $s/oracle.prev.json ]]; then publish_file "$s/oracle.prev.json" "$OUT/oracle.prev.json"; fi
  if produced "$s/sync.lock"; then publish_file "$s/sync.lock" "$OUT/sync.lock"; fi
  publish_file "$s/oracle.json" "$OUT/oracle.json"
}

publish_roles() {
  local s="$STAGING/roles" f
  if [[ -d $s/reports ]]; then
    for f in "$s"/reports/*; do
      if produced "$f"; then publish_file "$f" "$OUT/reports/${f##*/}"; fi
    done
  fi
  publish_file "$s/roles.json" "$OUT/roles.json"
}

publish_llm() {
  local s="$STAGING/llm" f
  # only if this run added rows (the copy of the old file is byte-identical otherwise)
  if produced "$s/llm_roles.pending.jsonl" && ! cmp -s "$s/llm_roles.pending.jsonl" "$OUT/llm_roles.pending.jsonl"; then
    publish_file "$s/llm_roles.pending.jsonl" "$OUT/llm_roles.pending.jsonl"
  fi
  if [[ -d $s/reports ]]; then
    for f in "$s"/reports/llm-*.json "$s"/reports/llm-failed.txt; do
      if produced "$f"; then publish_file "$f" "$OUT/reports/${f##*/}"; fi
    done
  fi
}

publish_map_digest() {
  local s="$STAGING/map-digest" f
  if [[ -d $s/digests ]]; then
    for f in "$s"/digests/*; do
      if produced "$f"; then publish_file "$f" "$OUT/digests/${f##*/}"; fi
    done
  fi
  publish_file "$s/map.md" "$OUT/map.md"
}

# ------------------------------------------------------------------------------------------------ LLM decisions
# Plain functions over curl, ss, systemctl and /proc, so a test can put stubs of them first in PATH and a fake /proc tree
# in $KNOWLEDGE_PROC_ROOT.

# the model ids the server on the port reports, one per line; nothing if it does not answer
llm_models() {
  curl -fsS -m 5 "$LLM_URL/v1/models" 2>/dev/null | grep -o '"id"[[:space:]]*:[[:space:]]*"[^"]*"' |
    sed 's/.*:[[:space:]]*"\(.*\)"/\1/' || true
}

llm_port_held() {
  ss -ltnH "sport = :$LLM_PORT" 2>/dev/null | grep -q .
}

# Whether other processes are using the B70: busy, free, or unknown. The render node is held open by the desktop session
# and by every client of the card even when it uses no memory, so "open" says nothing; what counts is the video memory
# the open file descriptors hold. The sum of `drm-resident-vram0` over the fdinfo of every file descriptor that points at
# the render node (or its -card sibling), each DRM client once (`drm-client-id`), is compared with
# $KNOWLEDGE_GPU_BUSY_MIB (default 1024 MiB): at or above it the GPU is busy. Free: nothing matches, or the sum is below
# it. Unknown (the stage skips): a matching descriptor's fdinfo cannot be read or has no `drm-resident-vram0`. Units
# KiB, MiB and GiB; no unit means bytes. $KNOWLEDGE_PROC_ROOT (default /proc) lets a test supply a fake tree. Only
# processes whose descriptors this user may read are seen (a single-user machine: all of them).
gpu_state() {
  local proc="${KNOWLEDGE_PROC_ROOT:-/proc}" limit="${KNOWLEDGE_GPU_BUSY_MIB:-1024}" fd link info text id val unit
  local -a targets=()
  local -A seen=()
  local total_kib=0 unknown=0 node
  # at most 9 digits, forced to base 10 ("08" is no octal error, "010" is 10 not 8)
  [[ $limit =~ ^[0-9]{1,9}$ ]] && limit=$((10#$limit)) || {
    echo unknown
    return 0
  }
  node="$(readlink -f -- "$GPU_DEVICE" 2>/dev/null)" || {
    echo unknown
    return 0
  }
  targets+=("$node")
  if [[ $GPU_DEVICE == *-render ]] && [[ -e ${GPU_DEVICE%-render}-card ]]; then
    targets+=("$(readlink -f -- "${GPU_DEVICE%-render}-card" 2>/dev/null || true)")
  fi
  for fd in "$proc"/[0-9]*/fd/*; do
    link="$(readlink -- "$fd" 2>/dev/null)" || continue
    [[ $link == "${targets[0]}" || (${#targets[@]} -gt 1 && $link == "${targets[1]}") ]] || continue
    info="${fd%/fd/*}/fdinfo/${fd##*/}"
    text="$(cat -- "$info" 2>/dev/null)" || {
      [[ -L $fd ]] || continue # the descriptor vanished (process exit race): not a holder
      unknown=1
      continue
    }
    id="$(sed -n 's/^drm-client-id:[[:space:]]*//p' <<<"$text" | head -n 1)"
    val="$(sed -n 's/^drm-resident-vram0:[[:space:]]*//p' <<<"$text" | head -n 1)"
    if [[ ! $val =~ ^([0-9]+)[[:space:]]*(KiB|MiB|GiB)?$ ]]; then
      unknown=1
      continue
    fi
    # one client once, however many descriptors it holds (no id: this descriptor alone)
    [[ -n $id ]] || id="fd:$fd"
    [[ -z ${seen[$id]:-} ]] || continue
    seen[$id]=1
    unit="${BASH_REMATCH[2]}"
    case $unit in
    KiB) total_kib=$((total_kib + BASH_REMATCH[1])) ;;
    MiB) total_kib=$((total_kib + BASH_REMATCH[1] * 1024)) ;;
    GiB) total_kib=$((total_kib + BASH_REMATCH[1] * 1048576)) ;;
    *) total_kib=$((total_kib + BASH_REMATCH[1] / 1024)) ;;
    esac
  done
  if ((total_kib >= limit * 1024)); then
    echo busy
  elif ((unknown)); then
    echo unknown
  else
    echo free
  fi
}

# whether the LLM stage may start now: the local hour must be inside $KNOWLEDGE_LLM_HOURS (START-END, default 0-7,
# that is 00:00 up to but not including 07:00; START greater than END wraps past midnight). The window gates the start
# of the stage, not its end. $KNOWLEDGE_NOW_HOUR replaces the clock (tests).
llm_in_window() {
  local hours="${KNOWLEDGE_LLM_HOURS:-0-7}" hour="${KNOWLEDGE_NOW_HOUR:-}" a b
  [[ -n $hour ]] || hour="$(date +%-H)"
  [[ $hour =~ ^[0-9]{1,2}$ ]] || return 2
  [[ $hours =~ ^([0-9]{1,2})-([0-9]{1,2})$ ]] || return 2
  a=$((10#${BASH_REMATCH[1]}))
  b=$((10#${BASH_REMATCH[2]}))
  hour=$((10#$hour))
  ((a <= 24 && b <= 24 && hour <= 23)) || return 2
  if ((a <= b)); then ((hour >= a && hour < b)); else ((hour >= a || hour < b)); fi
}

llm_unit_installed() {
  systemctl --user cat "$LLM_UNIT.service" >/dev/null 2>&1
}

# one line: "use" (a Qwen3.6-27B server already answers: use it, never stop it), "start", or "skip: <reason>"
llm_decide() {
  local ids state
  ids="$(llm_models)"
  if [[ -n $ids ]]; then
    if grep -qF -- "$LLM_MODEL_MATCH" <<<"$ids"; then echo "use"; else echo "skip: other model on the port"; fi
    return 0
  fi
  if llm_port_held; then
    echo "skip: port $LLM_PORT is held but /v1/models does not answer"
    return 0
  fi
  if [[ ! -e $GPU_DEVICE ]]; then
    echo "skip: GPU device $GPU_DEVICE not found"
    return 0
  fi
  state="$(gpu_state)"
  if [[ $state == busy ]]; then
    echo "skip: GPU busy"
  elif [[ $state != free ]]; then
    echo "skip: GPU state unknown (a descriptor on $GPU_DEVICE could not be read, or reports no video memory)"
  elif ! llm_unit_installed; then
    echo "skip: $LLM_UNIT.service is not installed (systemctl --user link scripts/systemd/$LLM_UNIT.service)"
  else
    echo "start"
  fi
}

llm_unit_state() {
  systemctl --user is-active "$LLM_UNIT.service" 2>/dev/null || true
}

llm_invocation() {
  systemctl --user show -p InvocationID --value "$LLM_UNIT.service" 2>/dev/null || true
}

# Start the unit if, right before the start, nothing answers and the unit is inactive. Returns 0 when this job started
# it (the ownership marker is written), 20 when somebody else's server is there (never stopped by this job), 1 when
# the start failed.
llm_start() {
  local state
  if [[ -n "$(llm_models)" ]]; then return 20; fi
  state="$(llm_unit_state)"
  case $state in
  active | activating | reloading | deactivating) return 20 ;;
  esac
  if ! systemctl --user start "$LLM_UNIT.service"; then
    # a start that failed after the unit began (it was inactive a moment ago): it is ours to stop
    case "$(llm_unit_state)" in
    active | activating) systemctl --user stop "$LLM_UNIT.service" >/dev/null 2>&1 || true ;;
    esac
    return 1
  fi
  printf 'run=%s\ninvocation=%s\n' "$RUN_ID" "$(llm_invocation)" >"$OWNS"
}

# Stop the unit only if the ownership marker says this job started it, and only that invocation. "own" (what the
# script's own exits use) also requires the marker to carry this run's id, so a second job that finds the lock taken
# cannot stop the first one's server; the service's ExecStopPost runs `--cleanup`, which releases whatever run's marker
# is there (the service has ended, so that run is gone).
llm_release() {
  local marked current
  [[ -n $OWNS && -f $OWNS ]] || return 0
  if [[ ${1:-} == own ]] && ! grep -qxF "run=$RUN_ID" "$OWNS"; then return 0; fi
  marked="$(sed -n 's/^invocation=//p' "$OWNS")"
  current="$(llm_invocation)"
  if [[ -z $marked || -z $current || $marked == "$current" ]]; then
    systemctl --user stop "$LLM_UNIT.service" >/dev/null 2>&1 || true
  else
    echo "knowledge-weekly: $LLM_UNIT has a newer invocation than the one this job started; left running"
  fi
  rm -f -- "$OWNS"
}

# wait until the server names the model: 0 ready, 1 timed out, 2 the unit stopped
llm_wait_ready() {
  local deadline=$((SECONDS + LLM_READY))
  while ((SECONDS < deadline)); do
    if grep -qF -- "$LLM_MODEL_MATCH" <<<"$(llm_models)"; then return 0; fi
    if ! systemctl --user is-active --quiet "$LLM_UNIT.service"; then return 2; fi
    sleep "$LLM_POLL"
  done
  return 1
}

llm_smoke() {
  local body='{"messages":[{"role":"user","content":"Reply with the single word OK."}],"max_tokens":16,"temperature":0,"chat_template_kwargs":{"enable_thinking":false}}'
  local out
  out="$(curl -fsS -m 180 -H 'Content-Type: application/json' -d "$body" "$LLM_URL/v1/chat/completions")" || return 1
  grep -q '"content"' <<<"$out"
}

# ------------------------------------------------------------------------------------------------ stages

stage_fetch_build() {
  if ((DRY)); then
    echo "  clone/fetch ${KNOWLEDGE_REPO_URL:-the origin of $MAIN} into $REPO, reset --hard to $REF, cargo build --release -p cardgen"
    echo "  (under the memory cap, behind flock $CARGO_LOCK); binary $CARDGEN"
    return 0
  fi
  if [[ $SKIP_FETCH != 1 ]]; then
    if [[ ! -d $REPO/.git ]]; then
      local url="${KNOWLEDGE_REPO_URL:-$(git -C "$MAIN" remote get-url origin 2>/dev/null || true)}"
      [[ -n $url ]] || {
        note "no KNOWLEDGE_REPO_URL and $MAIN has no origin remote"
        return 1
      }
      mkdir -p "$(dirname "$REPO")"
      git clone --quiet --no-checkout "$url" "$REPO" || {
        note "git clone of $url failed"
        return 1
      }
    fi
    if ! git -C "$REPO" fetch --quiet origin; then
      note "git fetch failed (no network?); the previous clone and build stay in use"
      return 1
    fi
    git -C "$REPO" reset --hard --quiet "$REF" || {
      note "git reset --hard $REF failed in $REPO"
      return 1
    }
    git -C "$REPO" clean -fdq
  fi
  if [[ -n ${KNOWLEDGE_CARDGEN:-} ]]; then
    [[ -x $KNOWLEDGE_CARDGEN ]] || {
      note "KNOWLEDGE_CARDGEN=$KNOWLEDGE_CARDGEN is not executable"
      return 1
    }
    note "using the prebuilt $KNOWLEDGE_CARDGEN on $(commit_of_repo)"
    return 0
  fi
  local extra=()
  read -ra extra <<<"${KNOWLEDGE_CARGO_ARGS:-}"
  local built=0
  (cd "$REPO/rust" && flock -w "${KNOWLEDGE_CARGO_LOCK_WAIT:-3600}" -E 200 "$CARGO_LOCK" "${SCOPE[@]}" cargo build --release -p cardgen "${extra[@]}") || built=$?
  if ((built == 200)); then
    note "heavy lock busy: $CARGO_LOCK was not free within ${KNOWLEDGE_CARGO_LOCK_WAIT:-3600}s; the previous binary stays in use"
    return 1
  elif ((built != 0)); then
    note "cargo build failed (see fetch-build.log); the previous binary stays in use"
    return 1
  fi
  note "built $(commit_of_repo) from $REF"
}

stage_bulk_sync() {
  if ((DRY)); then
    if [[ -n $BULK_FILE ]]; then
      echo "  cardgen bulk-sync --from-file $BULK_FILE --updated-at ${BULK_UPDATED_AT:-?} --min-count $BULK_MIN in staging"
    else
      echo "  cardgen bulk-sync --min-count $BULK_MIN in staging (skipped when the bulk updated_at is unchanged);"
    fi
    echo "  then, only on success, rename prev, lock and catalog into $OUT (the previous catalog becomes oracle.prev.json)"
    return 0
  fi
  need_cardgen || return $?
  mkdir -p "$OUT"
  local s args=(bulk-sync --min-count "$BULK_MIN") out
  s="$(stage_dir bulk-sync)"
  # the catalog and its lock are the sync's own inputs: hard links, so the sync's rename never touches the published files
  if [[ -f $OUT/oracle.json ]]; then ln -- "$OUT/oracle.json" "$s/oracle.json" 2>/dev/null || cp -- "$OUT/oracle.json" "$s/oracle.json"; fi
  if [[ -f $OUT/sync.lock ]]; then cp -- "$OUT/sync.lock" "$s/sync.lock"; fi
  if [[ -n $BULK_FILE ]]; then args+=(--from-file "$BULK_FILE" --updated-at "${BULK_UPDATED_AT:-fixture}"); fi
  out="$(KDIR=$s cg "${args[@]}")" || {
    echo "$out"
    note "bulk-sync failed: $(grep -v '^[[:space:]]*$' "$RUN_DIR/bulk-sync.log" 2>/dev/null | tail -n 1 | cut -c1-200)"
    return 1
  }
  echo "$out"
  if grep -q '^up to date' <<<"$out"; then
    note "$(grep -m1 '^up to date' <<<"$out")"
    return 10
  fi
  produced "$s/oracle.json" || {
    note "bulk-sync wrote no catalog"
    return 1
  }
  note "$(grep -m1 '^bulk updated_at' <<<"$out"); $(grep -m1 '^read ' <<<"$out" | cut -c1-100)"
}

stage_roles() {
  if ((DRY)); then
    echo "  keep roles.json as $CACHE/roles.before.json; in staging: cardgen roles --rules; then, if llm_roles.jsonl has"
    echo "  rows, cardgen roles --merge --no-think --model $LLM_MODEL_ID (the tracked cache only; pending rows stay out);"
    echo "  only when both succeeded, rename roles.json (and the merge reports) into $OUT"
    return 0
  fi
  need_cardgen || return $?
  [[ -f $OUT/oracle.json ]] || {
    note "no catalog at $OUT/oracle.json"
    return 10
  }
  [[ -f $OUT/roles.toml ]] || {
    note "$OUT/roles.toml is missing: the tracked role files must be checked out in $OUT (set KNOWLEDGE_OUT)"
    return 1
  }
  local s
  s="$(stage_dir roles)"
  link_inputs "$s" roles.json
  rm -f "$CACHE/roles.before.json"
  if [[ -f $OUT/roles.json ]]; then cp "$OUT/roles.json" "$CACHE/roles.before.json"; fi
  KDIR=$s cg roles --rules || {
    note "roles --rules failed; roles.json is the previous table"
    return 1
  }
  local merged=0
  if [[ -s $OUT/llm_roles.jsonl ]]; then
    KDIR=$s cg roles --merge --no-think --model "$LLM_MODEL_ID" || {
      note "roles --merge failed; roles.json is the previous table"
      return 1
    }
    merged=1
  fi
  produced "$s/roles.json" || {
    note "roles --rules wrote no roles.json; roles.json is the previous table"
    return 1
  }
  if ((merged)); then note "rules and the tracked LLM cache merged"; else note "rules only (no tracked LLM rows)"; fi
}

stage_llm() {
  if ((DRY)); then
    echo "  skipped if KNOWLEDGE_SKIP_LLM=1 or every routed card is cached (roles --llm --dry-run counts them)"
    echo "  else decide: a Qwen3.6-27B server answering on $LLM_URL is used and never stopped; another model on the port"
    echo "  or a GPU ($GPU_DEVICE) whose video memory other processes hold at or above ${KNOWLEDGE_GPU_BUSY_MIB:-1024} MiB, or in an"
    echo "  unknown state, skips the stage; the stage runs only in the night window (hours ${KNOWLEDGE_LLM_HOURS:-0-7}); otherwise start"
    echo "  $LLM_UNIT.service (inactive right before the start; ownership marker; stopped afterwards only because this job"
    echo "  started it), wait up to ${LLM_READY}s, smoke prompt, then"
    echo "  ${LLM_ARGS[*]} for at most ${LLM_BUDGET}s in staging; the new rows of llm_roles.pending.jsonl are renamed"
    echo "  into $OUT at the end"
    return 0
  fi
  if [[ $SKIP_LLM == 1 ]]; then
    note "KNOWLEDGE_SKIP_LLM=1"
    return 10
  fi
  # the GPU is JD's during the day: a catch-up run after a missed Sunday 04:00 skips this stage, the rest still runs
  local in_window=0
  llm_in_window || in_window=$?
  if ((in_window == 2)); then
    note "KNOWLEDGE_LLM_HOURS=${KNOWLEDGE_LLM_HOURS:-0-7} is not START-END hours (0-23): skipped to be safe"
    return 10
  elif ((in_window != 0)); then
    note "outside the night window (KNOWLEDGE_LLM_HOURS=${KNOWLEDGE_LLM_HOURS:-0-7}, local hour ${KNOWLEDGE_NOW_HOUR:-$(date +%-H)})"
    return 10
  fi
  need_cardgen || return $?
  [[ -f $OUT/oracle.json && -f $OUT/roles.toml ]] || {
    note "no catalog or role files in $OUT"
    return 10
  }
  local s
  s="$(stage_dir llm)"
  link_inputs "$s" llm_roles.pending.jsonl
  # the pending rows so far are the run's own input and output: a copy
  if [[ -f $OUT/llm_roles.pending.jsonl ]]; then cp -- "$OUT/llm_roles.pending.jsonl" "$s/llm_roles.pending.jsonl"; fi
  # nothing to do on a quiet week: the dry run counts the routed cards the cache misses without a server
  local plan todo
  plan="$(KDIR=$s cg "${LLM_ARGS[@]}" --model "$LLM_MODEL_ID" --dry-run 2>&1)" || {
    note "roles --llm --dry-run failed: $(tail -n 1 <<<"$plan" | cut -c1-160)"
    return 1
  }
  todo="$(grep -o '[0-9]* to classify' <<<"$plan" | head -n 1 | grep -o '^[0-9]*' || true)"
  if [[ -n $todo && $todo == 0 ]]; then
    note "nothing to classify: every routed card is cached"
    return 10
  fi
  local decision
  decision="$(llm_decide)"
  case "$decision" in
  use) echo "using the server already running on $LLM_URL (it stays running)" ;;
  skip:*)
    note "${decision#skip: }"
    return 10
    ;;
  start)
    trap 'llm_release own' EXIT
    local started=0
    llm_start || started=$?
    case $started in
    0) ;;
    20) echo "the unit is already active (or a server answers): using it, it stays running" ;;
    *)
      note "systemctl --user start $LLM_UNIT failed"
      return 1
      ;;
    esac
    local ready=0
    llm_wait_ready || ready=$?
    if ((ready != 0)); then
      note "$LLM_UNIT did not become ready ($([[ $ready == 2 ]] && echo 'the unit stopped' || echo "no answer in ${LLM_READY}s")); stopped it if this job started it"
      return 1
    fi
    ;;
  *)
    note "unexpected decision '$decision'"
    return 1
    ;;
  esac
  llm_smoke || {
    note "the smoke prompt failed"
    return 1
  }
  local before after rc=0
  before="$(rows_in "$OUT/llm_roles.pending.jsonl")"
  timeout --foreground --kill-after=60 "$LLM_BUDGET" "${SCOPE[@]}" env CARDGEN_KNOWLEDGE_DIR="$s" "$CARDGEN" --root "$REPO" "${LLM_ARGS[@]}" || rc=$?
  after="$(rows_in "$s/llm_roles.pending.jsonl")"
  if ((rc == 124)); then
    note "budget of ${LLM_BUDGET}s used up; pending rows $before -> $after, the rest carries over (${todo:-?} were to do)"
    return 0
  elif ((rc != 0)); then
    note "roles --llm exited $rc after $((after - before)) new pending rows: $(tail -n 1 "$RUN_DIR/llm.log" | cut -c1-160)"
    return 1
  fi
  note "$((after - before)) new pending rows ($after pending in all), ${todo:-?} were to do"
}

stage_map_digest() {
  if ((DRY)); then
    echo "  in staging: cardgen map; cardgen digest --deck KEY for every deck of mine in $REPO/data/decks.json;"
    echo "  only when all succeeded, rename them into $OUT"
    return 0
  fi
  need_cardgen || return $?
  [[ -f $OUT/oracle.json && -f $OUT/roles.json ]] || {
    note "no catalog or role table in $OUT"
    return 10
  }
  local keys key n=0 failed=() s
  keys="$(jq -r '.mine | if type == "array" then .[] | (.key // error("a deck without a key")) else error("mine is not an array") end' "$REPO/data/decks.json" 2>&1)" || {
    note "decks.json: cannot read the decks of mine: $(tail -n 1 <<<"$keys" | cut -c1-160)"
    return 1
  }
  s="$(stage_dir map-digest)"
  link_inputs "$s" map.md
  KDIR=$s cg map >/dev/null || failed+=(map)
  while IFS= read -r key; do
    [[ -n $key ]] || continue
    n=$((n + 1))
    KDIR=$s cg digest --deck "$key" >/dev/null || failed+=("$key")
  done <<<"$keys"
  if ((${#failed[@]} > 0)); then
    note "failed: ${failed[*]} (the previous files stay)"
    return 1
  fi
  produced "$s/map.md" || {
    note "map wrote no map.md (the previous files stay)"
    return 1
  }
  note "map and $n digests"
}

# a report written without cardgen, when cardgen itself is the problem: $1 is the staged file, $2 the reason
fallback_report() {
  local path="$1" tmp s status secs text
  mkdir -p "$(dirname -- "$path")"
  tmp="$(mktemp "$(dirname -- "$path")/.fallback.XXXXXX")"
  {
    echo "# Card knowledge: weekly report $DATE"
    echo
    echo "## Action needed"
    echo
    echo "- The report could not be built by cardgen, so this is the short form: $2"
    while IFS=$'\t' read -r s status secs text; do
      if [[ $status == fail ]]; then echo "- Stage \`${s//[^A-Za-z0-9_-]/_}\` failed: $text"; fi
    done <"$STAGES_FILE"
    echo
    echo "## Run"
    echo
    echo "| Stage | Status | Seconds | Note |"
    echo "|---|---|---|---|"
    while IFS=$'\t' read -r s status secs text; do
      echo "| $s | $status | $secs | ${text//|/\/} |"
    done <"$STAGES_FILE"
    echo
    echo "## New cards"
    echo
    echo "Not available in the short form."
    echo
    echo "## Bans/unbans and Game Changer changes"
    echo
    echo "Not available in the short form."
    echo
    echo "## Role changes and LLM stats"
    echo
    echo "Not available in the short form."
  } >"$tmp"
  mv -f -T -- "$tmp" "$path"
}

# build the report in staging (cardgen's, else the short one); run_stage publishes it
stage_report() {
  if ((DRY)); then
    echo "  cardgen weekly-report --date $DATE --out <staging> (inputs read from $OUT), then rename it to"
    echo "  $OUT/reports/$DATE.md (an earlier report of the day is kept as $DATE.N.md; the newest $KEEP_REPORTS stay);"
    echo "  if cardgen cannot run, a short report goes through the same publication; failures also go to notify-send"
    return 0
  fi
  local s f staged
  s="$(stage_dir report)"
  staged="$s/out/$DATE.md"
  mkdir -p "$s/out" "$s/reports"
  link_inputs "$s"
  # the merge report and the other non-dated files of reports/ are read, never written (the dated reports stay out)
  while IFS= read -r -d '' f; do
    ln -s -- "$f" "$s/reports/${f##*/}"
  done < <(find "$OUT/reports" -maxdepth 1 -type f ! -name '[0-9][0-9][0-9][0-9]-*' -print0 2>/dev/null)
  local args=(weekly-report --date "$DATE" --commit "$(commit_of_repo)" --stages "$STAGES_FILE" --llm-log "$RUN_DIR/llm.log" --out "$staged")
  if [[ -f $CACHE/roles.before.json ]]; then args+=(--roles-before "$CACHE/roles.before.json"); fi
  if [[ -x $CARDGEN && -d $REPO/data ]] && KDIR=$s cg "${args[@]}" && produced "$staged"; then
    return 0
  fi
  echo "cardgen weekly-report did not run: writing the short report" >&2
  fallback_report "$staged" "cardgen weekly-report did not run (no binary, or it failed)"
}

# ------------------------------------------------------------------------------------------------ driver

on_signal() {
  TERMINATED=1
  if [[ -n $STAGE_PID ]]; then
    kill -TERM -- "-$STAGE_PID" 2>/dev/null || kill -TERM "$STAGE_PID" 2>/dev/null || true
  fi
}

# run one stage in a process group of its own, asynchronously: the script itself waits explicitly, so a TERM sent to
# the script alone is forwarded to the stage (and its children) and reaped before anything is cleaned up. The stage's
# output goes to $RUN_DIR/NAME.log; its result is recorded unless $4 is "norecord". The stage's outputs are published
# (renamed into $OUT) here, in the script, after the stage as a whole succeeded.
run_stage() {
  local num="$1" name="$2" fn="$3" record="${4:-record}" start=$SECONDS rc status text
  if [[ -n $ONLY && $ONLY != "$num" && $ONLY != "$name" ]]; then return 0; fi
  if ((DRY)); then
    echo "stage $num $name:"
    "$fn"
    return 0
  fi
  NOTE_FILE="$RUN_DIR/$name.note"
  : >"$NOTE_FILE"
  set -m
  (
    set -e
    "$fn"
  ) >"$RUN_DIR/$name.log" 2>&1 9>&- &
  STAGE_PID=$!
  set +m
  rc=0
  while :; do
    wait "$STAGE_PID" && rc=0 || rc=$?
    kill -0 "$STAGE_PID" 2>/dev/null || break
  done
  STAGE_PID=""
  # publication: all of a stage's outputs only after the whole stage succeeded (the LLM stage's finished rows always)
  local pub=0
  case $name in
  bulk-sync) if ((rc == 0)); then publish_bulk >>"$RUN_DIR/$name.log" 2>&1 || pub=1; fi ;;
  roles) if ((rc == 0)); then publish_roles >>"$RUN_DIR/$name.log" 2>&1 || pub=1; fi ;;
  llm) publish_llm >>"$RUN_DIR/$name.log" 2>&1 || pub=1 ;;
  map-digest) if ((rc == 0)); then publish_map_digest >>"$RUN_DIR/$name.log" 2>&1 || pub=1; fi ;;
  report) if ((rc == 0)); then publish_report "$STAGING/report/out/$DATE.md" >>"$RUN_DIR/$name.log" 2>&1 || pub=1; fi ;;
  esac
  if ((pub)); then
    rc=1
    note "publishing the stage's outputs failed (see $name.log); the previous files stay"
  fi
  case $rc in
  0) status=ok ;;
  10) status=skip ;;
  *) status=fail ;;
  esac
  text="$(tr '\t\n' '  ' <"$NOTE_FILE")"
  if [[ -z $text && $status == fail ]]; then
    text="exit $rc: $(grep -v '^[[:space:]]*$' "$RUN_DIR/$name.log" | tail -n 1 | cut -c1-200)"
  fi
  if [[ $record == record ]]; then
    printf '%s\t%s\t%s\t%s\n' "$name" "$status" "$((SECONDS - start))" "$text" >>"$STAGES_FILE"
  fi
  echo "stage $num $name: $status ($((SECONDS - start))s) $text"
  if [[ $status == fail ]]; then tail -n 5 "$RUN_DIR/$name.log" >&2; fi
  if ((TERMINATED)); then exit 143; fi
  RUN_STATUS=$status
}

cleanup() {
  llm_release own
  if [[ -n $STAGING && -d $STAGING ]]; then rm -rf -- "$STAGING"; fi
  if [[ -n $STATE && $STATE != "$CACHE" && -d $STATE ]]; then rm -rf -- "$STATE"; fi
}

usage() {
  sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'
}

# the per-run state: the lock, the run directory and the staging root; false if the cache directory cannot be used
setup_state() {
  mkdir -p "$CACHE" 2>/dev/null || return 1
  { exec 9>"$CACHE/lock"; } 2>/dev/null || return 1
  if ! flock -n 9; then
    echo "knowledge-weekly: another run is active (lock $CACHE/lock); exiting"
    exit 0
  fi
  STATE="$CACHE"
  # staging directories of a run that was killed
  rm -rf -- "$CACHE/staging" 2>/dev/null || true
  RUN_DIR="$STATE/runs/$DATE"
  mkdir -p "$RUN_DIR" "$STATE/staging" 2>/dev/null || return 1
  STAGING="$STATE/staging/$RUN_ID"
  mkdir -p "$STAGING" 2>/dev/null || return 1
  STAGES_FILE="$RUN_DIR/stages.tsv"
  : >"$STAGES_FILE" || return 1
  if [[ -e $OWNS ]]; then
    echo "knowledge-weekly: a previous run left $OWNS; not stopping $LLM_UNIT here (its --cleanup and RuntimeMaxSec do)"
    rm -f -- "$OWNS"
  fi
}

main() {
  while (($#)); do
    case "$1" in
    --dry-run) DRY=1 ;;
    --cleanup) CLEANUP=1 ;;
    --stage)
      [[ ${2:-} =~ ^[1-6]$ ]] || die "--stage needs a number from 1 to 6"
      ONLY="$2"
      shift
      ;;
    --only)
      [[ -n ${2:-} ]] || die "--only needs a stage name"
      ONLY="$2"
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *) die "unknown argument '$1' (try --help)" ;;
    esac
    shift
  done
  if [[ -n $ONLY ]]; then
    local ok=0 i
    for i in "${!STAGE_NAMES[@]}"; do
      if [[ $ONLY == "${STAGE_NAMES[$i]}" || $ONLY == "$((i + 1))" ]]; then ok=1; fi
    done
    ((ok)) || die "no stage '$ONLY' (fetch-build, bulk-sync, roles, llm, map-digest, report or 1-6)"
  fi
  [[ $EUID -ne 0 ]] || die "refusing to run as root"
  if [[ -z $MAIN || -z $OUT || -z $CACHE || -z $REPO || -z $CARDGEN ]]; then
    die "HOME is not set, so the default places cannot be found: set KNOWLEDGE_MAIN, KNOWLEDGE_OUT, KNOWLEDGE_CACHE and KNOWLEDGE_REPO"
  fi

  if ((CLEANUP)); then
    llm_release
    exit 0
  fi

  if ((DRY)); then
    echo "DRY RUN: nothing is written or called"
    echo "main checkout   $MAIN"
    echo "output          $OUT   (the published directory; stages work in staging and rename into it)"
    echo "clone           $REPO  ($REF)"
    echo "cardgen         $CARDGEN"
    echo "state           $CACHE"
    echo "date            $DATE"
    run_stage 1 fetch-build stage_fetch_build
    run_stage 2 bulk-sync stage_bulk_sync
    run_stage 3 roles stage_roles
    run_stage 4 llm stage_llm
    run_stage 5 map-digest stage_map_digest
    run_stage 6 report stage_report norecord
    exit 0
  fi

  trap on_signal TERM INT HUP
  trap cleanup EXIT
  local state_ok=1
  if ! setup_state; then
    state_ok=0
    # the cache directory cannot be used (read-only, no space): the failure still gets a report, if OUT is writable
    STATE="$(mktemp -d "${TMPDIR:-/tmp}/knowledge-weekly.XXXXXX")" || exit 1
    RUN_DIR="$STATE/runs/$DATE"
    STAGING="$STATE/staging/$RUN_ID"
    mkdir -p "$RUN_DIR" "$STAGING" || exit 1
    STAGES_FILE="$RUN_DIR/stages.tsv"
    printf 'setup\tfail\t0\tcannot use the state directory %s (read-only or full); only the report is written\n' \
      "$(tr '\t\n' '  ' <<<"$CACHE")" >"$STAGES_FILE"
    echo "knowledge-weekly: cannot use $CACHE; writing the report only" >&2
  fi

  if ((state_ok)); then
    run_stage 1 fetch-build stage_fetch_build
    run_stage 2 bulk-sync stage_bulk_sync
    run_stage 3 roles stage_roles
    run_stage 4 llm stage_llm
    run_stage 5 map-digest stage_map_digest
  fi
  if [[ -z $ONLY || $ONLY == 6 || $ONLY == report ]]; then
    RUN_STATUS=""
    # the report is written even when every stage failed
    run_stage 6 report stage_report norecord
    if [[ $RUN_STATUS != ok ]]; then
      echo "knowledge-weekly: the report could not be written" >&2
      exit 1
    fi
    echo "report: $OUT/reports/$DATE.md"
  fi
  if grep -q $'\tfail\t' "$STAGES_FILE"; then
    local n
    n="$(grep -c $'\tfail\t' "$STAGES_FILE")"
    # best effort: no notification service is not a failure
    if command -v notify-send >/dev/null 2>&1; then
      notify-send "Card knowledge weekly: $n stage(s) failed" "See $OUT/reports/$DATE.md" >/dev/null 2>&1 || true
    fi
  fi
  if ((state_ok)); then
    # keep the logs of the last 12 runs
    local old
    while read -r old; do rm -rf "${CACHE:?}/runs/$old"; done < <(ls -1 "$CACHE/runs" 2>/dev/null | sort | head -n -12)
  fi
  exit 0
}

# sourced by a test: only the functions and the configuration are defined
if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  main "$@"
fi
