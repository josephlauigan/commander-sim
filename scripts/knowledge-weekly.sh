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
# Output goes to $KNOWLEDGE_OUT (default: data/knowledge of the main checkout, $KNOWLEDGE_MAIN) through
# CARDGEN_KNOWLEDGE_DIR, never to the clone. Every file it writes there is gitignored (oracle.json, oracle.prev.json,
# sync.lock, roles.json, map.md, digests/, reports/, llm_roles.pending.jsonl). The tracked role files (roles.toml,
# roles_overrides.toml, llm_roles.jsonl) are only READ from there, so they must be checked out in the main checkout.
# A stage that fails is recorded and never deletes the previous outputs (cardgen writes to a temp file and renames).
# Exit 0 whenever the report was written, 1 only if it could not be; 0 too if another run holds the lock.
#
# NEVER AUTOMATED: commits, pushes, PRs, writes to any tracked file, `cardgen export`, sims, swap or cut proposals,
# llama.cpp or model updates, merging pending LLM rows into llm_roles.jsonl, installing or enabling the timer.
#
# usage: scripts/knowledge-weekly.sh [--dry-run] [--stage N | --only NAME] [--help]
#   --dry-run       print the configuration and what each stage would do; writes nothing, calls nothing
#   --stage N       run only stage N (1-6), or --only NAME (fetch-build, bulk-sync, roles, llm, map-digest, report);
#                   stages 2-6 use the binary and clone a full run (or stage 1) left in $KNOWLEDGE_CACHE
#
# Environment (all optional): KNOWLEDGE_MAIN, KNOWLEDGE_OUT, KNOWLEDGE_CACHE, KNOWLEDGE_REPO, KNOWLEDGE_REPO_URL,
# KNOWLEDGE_REF, KNOWLEDGE_GPU_DEVICE, KNOWLEDGE_LLM_URL, KNOWLEDGE_LLM_UNIT, KNOWLEDGE_LLM_BUDGET_SECS,
# KNOWLEDGE_LLM_READY_SECS, KNOWLEDGE_SKIP_LLM=1; for tests: KNOWLEDGE_SKIP_FETCH=1, KNOWLEDGE_CARDGEN (a prebuilt
# binary), KNOWLEDGE_BULK_FILE + KNOWLEDGE_BULK_UPDATED_AT (a local bulk file instead of Scryfall), KNOWLEDGE_BULK_MIN,
# KNOWLEDGE_NO_SCOPE=1 (no systemd-run), KNOWLEDGE_DATE.
#
# needs: bash, git, cargo, flock, jq, curl, systemd-run, fuser, ss, timeout. Optional: notify-send.
set -euo pipefail

# ------------------------------------------------------------------------------------------------ configuration

MAIN="${KNOWLEDGE_MAIN:-$HOME/projects/commander-sim}"
OUT="${KNOWLEDGE_OUT:-$MAIN/data/knowledge}"
CACHE="${KNOWLEDGE_CACHE:-${XDG_CACHE_HOME:-$HOME/.cache}/commander-sim-knowledge}"
REPO="${KNOWLEDGE_REPO:-$CACHE/repo}"
REF="${KNOWLEDGE_REF:-origin/main}"
SKIP_FETCH="${KNOWLEDGE_SKIP_FETCH:-0}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO/rust/target}"
CARDGEN="${KNOWLEDGE_CARDGEN:-$CARGO_TARGET_DIR/release/cardgen}"
CARGO_LOCK="${KNOWLEDGE_CARGO_LOCK:-/tmp/cargo-heavy.lock}"
NO_SCOPE="${KNOWLEDGE_NO_SCOPE:-0}"
DATE="${KNOWLEDGE_DATE:-$(date +%F)}"
BULK_MIN="${KNOWLEDGE_BULK_MIN:-25000}"
BULK_FILE="${KNOWLEDGE_BULK_FILE:-}"
BULK_UPDATED_AT="${KNOWLEDGE_BULK_UPDATED_AT:-}"
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
LLM_ARGS=(--routes NoRole,UnknownLine --only-routed --batch 10 --no-think --pending --endpoint "$LLM_URL/v1")

STAGE_NAMES=(fetch-build bulk-sync roles llm map-digest report)
DRY=0
ONLY=""
RUN_DIR=""
STAGES_FILE=""
NOTE_FILE=""
STARTED=0
STARTED_FILE="$CACHE/llm-started"

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

# cardgen with the output directory and the cloned repository
CG=("${SCOPE[@]}" env CARDGEN_KNOWLEDGE_DIR="$OUT" "$CARDGEN" --root "$REPO")
cg() {
  "${CG[@]}" "$@"
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

# ------------------------------------------------------------------------------------------------ LLM decisions
# Plain functions over curl, ss, fuser and systemctl, so a test can put stubs of them first in PATH.

# the model ids the server on the port reports, one per line; nothing if it does not answer
llm_models() {
  curl -fsS -m 5 "$LLM_URL/v1/models" 2>/dev/null | grep -o '"id"[[:space:]]*:[[:space:]]*"[^"]*"' |
    sed 's/.*:[[:space:]]*"\(.*\)"/\1/' || true
}

llm_port_held() {
  ss -ltnH "sport = :$LLM_PORT" 2>/dev/null | grep -q .
}

# the render node is open in some process
gpu_busy() {
  fuser -s "$GPU_DEVICE" 2>/dev/null
}

llm_unit_installed() {
  systemctl --user cat "$LLM_UNIT.service" >/dev/null 2>&1
}

# one line: "use" (a Qwen3.6-27B server already answers: use it, never stop it), "start", or "skip: <reason>"
llm_decide() {
  local ids
  ids="$(llm_models)"
  if [[ -n $ids ]]; then
    if grep -qF -- "$LLM_MODEL_MATCH" <<<"$ids"; then echo "use"; else echo "skip: other model on the port"; fi
    return 0
  fi
  if llm_port_held; then
    echo "skip: port $LLM_PORT is held but /v1/models does not answer"
  elif [[ ! -e $GPU_DEVICE ]]; then
    echo "skip: GPU device $GPU_DEVICE not found"
  elif gpu_busy; then
    echo "skip: GPU busy"
  elif ! llm_unit_installed; then
    echo "skip: $LLM_UNIT.service is not installed (systemctl --user link scripts/systemd/$LLM_UNIT.service)"
  else
    echo "start"
  fi
}

llm_start() {
  # a runtime limit, so a killed job cannot leave the server holding the GPU for long
  systemctl --user set-property --runtime "$LLM_UNIT.service" RuntimeMaxSec="$((LLM_BUDGET + LLM_READY + 900))" \
    >/dev/null 2>&1 || true
  # recorded before the start: whatever the start leaves behind is ours to stop
  STARTED=1
  : >"$STARTED_FILE"
  systemctl --user start "$LLM_UNIT.service" || return 1
}

# stop the unit only if this job started it
llm_stop_if_started() {
  if [[ $STARTED == 1 || -e $STARTED_FILE ]]; then
    systemctl --user stop "$LLM_UNIT.service" >/dev/null 2>&1 || true
    rm -f "$STARTED_FILE"
    STARTED=0
  fi
}

# wait until the started server names the model: 0 ready, 1 timed out, 2 the unit died
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
  if ! (cd "$REPO/rust" && flock "$CARGO_LOCK" "${SCOPE[@]}" cargo build --release -p cardgen "${extra[@]}"); then
    note "cargo build failed (see fetch-build.log); the previous binary stays in use"
    return 1
  fi
  note "built $(commit_of_repo) from $REF"
}

stage_bulk_sync() {
  if ((DRY)); then
    if [[ -n $BULK_FILE ]]; then
      echo "  cardgen bulk-sync --from-file $BULK_FILE --updated-at ${BULK_UPDATED_AT:-?} --min-count $BULK_MIN into $OUT"
    else
      echo "  cardgen bulk-sync --min-count $BULK_MIN into $OUT (skipped when the bulk updated_at is unchanged;"
      echo "  the previous catalog is kept as oracle.prev.json)"
    fi
    return 0
  fi
  need_cardgen || return $?
  mkdir -p "$OUT"
  local args=(bulk-sync --min-count "$BULK_MIN") out
  if [[ -n $BULK_FILE ]]; then args+=(--from-file "$BULK_FILE" --updated-at "${BULK_UPDATED_AT:-fixture}"); fi
  out="$(cg "${args[@]}")" || {
    echo "$out"
    note "bulk-sync failed: $(tail -n 1 "$RUN_DIR/bulk-sync.log" 2>/dev/null | cut -c1-200)"
    return 1
  }
  echo "$out"
  if grep -q '^up to date' <<<"$out"; then
    note "$(grep -m1 '^up to date' <<<"$out")"
    return 10
  fi
  note "$(grep -m1 '^bulk updated_at' <<<"$out"); $(grep -m1 '^read ' <<<"$out" | cut -c1-100)"
}

stage_roles() {
  if ((DRY)); then
    echo "  keep roles.json as $CACHE/roles.before.json; cardgen roles --rules; then, if llm_roles.jsonl has rows,"
    echo "  cardgen roles --merge --no-think --model $LLM_MODEL_ID (the tracked cache only; pending rows stay out)"
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
  rm -f "$CACHE/roles.before.json"
  if [[ -f $OUT/roles.json ]]; then cp "$OUT/roles.json" "$CACHE/roles.before.json"; fi
  cg roles --rules || {
    note "roles --rules failed; roles.json is the previous table"
    return 1
  }
  if [[ -s $OUT/llm_roles.jsonl ]]; then
    cg roles --merge --no-think --model "$LLM_MODEL_ID" || {
      note "roles --merge failed; roles.json is rules-only"
      return 1
    }
    note "rules and the tracked LLM cache merged"
  else
    note "rules only (no tracked LLM rows)"
  fi
}

stage_llm() {
  if ((DRY)); then
    echo "  skipped if KNOWLEDGE_SKIP_LLM=1 or every routed card is cached (roles --llm --dry-run counts them)"
    echo "  else decide: a Qwen3.6-27B server answering on $LLM_URL is used and never stopped; another model on the port"
    echo "  or a GPU ($GPU_DEVICE) held by a process skips the stage; otherwise start $LLM_UNIT.service (stopped"
    echo "  afterwards only because this job started it), wait up to ${LLM_READY}s, smoke prompt, then"
    echo "  roles ${LLM_ARGS[*]} for at most ${LLM_BUDGET}s into llm_roles.pending.jsonl"
    return 0
  fi
  if [[ $SKIP_LLM == 1 ]]; then
    note "KNOWLEDGE_SKIP_LLM=1"
    return 10
  fi
  need_cardgen || return $?
  [[ -f $OUT/oracle.json && -f $OUT/roles.toml ]] || {
    note "no catalog or role files in $OUT"
    return 10
  }
  # nothing to do on a quiet week: the dry run counts the routed cards the cache misses without a server
  local plan todo
  plan="$(cg roles "${LLM_ARGS[@]}" --model "$LLM_MODEL_ID" --dry-run 2>&1)" || {
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
    trap llm_stop_if_started EXIT
    llm_start || {
      note "systemctl --user start $LLM_UNIT failed"
      return 1
    }
    local ready=0
    llm_wait_ready || ready=$?
    if ((ready != 0)); then
      note "$LLM_UNIT did not become ready ($([[ $ready == 2 ]] && echo 'the unit stopped' || echo "no answer in ${LLM_READY}s")); stopped it"
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
  timeout --kill-after=60 "$LLM_BUDGET" "${CG[@]}" roles "${LLM_ARGS[@]}" || rc=$?
  after="$(rows_in "$OUT/llm_roles.pending.jsonl")"
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
    echo "  cardgen map; cardgen digest --deck KEY for every deck of mine in $REPO/data/decks.json"
    return 0
  fi
  need_cardgen || return $?
  [[ -f $OUT/oracle.json && -f $OUT/roles.json ]] || {
    note "no catalog or role table in $OUT"
    return 10
  }
  local failed=() key n=0
  cg map >/dev/null || failed+=(map)
  while read -r key; do
    [[ -n $key ]] || continue
    n=$((n + 1))
    cg digest --deck "$key" >/dev/null || failed+=("$key")
  done < <(jq -r '.mine[].key' "$REPO/data/decks.json")
  if ((${#failed[@]} > 0)); then
    note "failed: ${failed[*]} (the previous files stay)"
    return 1
  fi
  note "map and $n digests"
}

# a report written without cardgen, when cardgen itself is the problem
fallback_report() {
  local path="$OUT/reports/$DATE.md" tmp s status secs text
  mkdir -p "$OUT/reports"
  tmp="$(mktemp "$OUT/reports/.fallback.XXXXXX")"
  {
    echo "# Card knowledge: weekly report $DATE"
    echo
    echo "## Action needed"
    echo
    echo "- The report could not be built by cardgen, so this is the short form: $1"
    while IFS=$'\t' read -r s status secs text; do
      [[ $status == fail ]] && echo "- Stage \`$s\` failed: $text"
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
  mv -f "$tmp" "$path"
}

stage_report() {
  if ((DRY)); then
    echo "  cardgen weekly-report --date $DATE (writes $OUT/reports/$DATE.md, keeps the newest 12);"
    echo "  if cardgen cannot run, a short report is written by the script; failures also go to notify-send"
    return 0
  fi
  local args=(weekly-report --date "$DATE" --commit "$(commit_of_repo)" --stages "$STAGES_FILE" --llm-log "$RUN_DIR/llm.log")
  if [[ -f $CACHE/roles.before.json ]]; then args+=(--roles-before "$CACHE/roles.before.json"); fi
  mkdir -p "$OUT/reports"
  if [[ -x $CARDGEN && -d $REPO/data ]] && cg "${args[@]}"; then
    return 0
  fi
  echo "cardgen weekly-report did not run: writing the short report" >&2
  fallback_report "cardgen weekly-report did not run (no binary, or it failed)"
}

# ------------------------------------------------------------------------------------------------ driver

# run one stage in a subshell of its own: its output goes to $RUN_DIR/NAME.log, its result to the stages file
run_stage() {
  local num="$1" name="$2" fn="$3" start=$SECONDS rc status text
  if [[ -n $ONLY && $ONLY != "$num" && $ONLY != "$name" ]]; then return 0; fi
  if ((DRY)); then
    echo "stage $num $name:"
    "$fn"
    return 0
  fi
  NOTE_FILE="$RUN_DIR/$name.note"
  : >"$NOTE_FILE"
  rc=0
  # not "if": errexit must stay on inside the subshell
  set +e
  (
    set -e
    "$fn"
  ) >"$RUN_DIR/$name.log" 2>&1
  rc=$?
  set -e
  case $rc in
  0) status=ok ;;
  10) status=skip ;;
  *) status=fail ;;
  esac
  text="$(tr '\t\n' '  ' <"$NOTE_FILE")"
  if [[ -z $text && $status == fail ]]; then
    text="exit $rc: $(grep -v '^[[:space:]]*$' "$RUN_DIR/$name.log" | tail -n 1 | cut -c1-200)"
  fi
  printf '%s\t%s\t%s\t%s\n' "$name" "$status" "$((SECONDS - start))" "$text" >>"$STAGES_FILE"
  echo "stage $num $name: $status ($((SECONDS - start))s) $text"
  if [[ $status == fail ]]; then tail -n 5 "$RUN_DIR/$name.log" >&2; fi
}

cleanup() {
  llm_stop_if_started
}

usage() {
  sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'
}

main() {
  while (($#)); do
    case "$1" in
    --dry-run) DRY=1 ;;
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

  if ((DRY)); then
    echo "DRY RUN: nothing is written or called"
    echo "main checkout   $MAIN"
    echo "output          $OUT   (CARDGEN_KNOWLEDGE_DIR)"
    echo "clone           $REPO  ($REF)"
    echo "cardgen         $CARDGEN"
    echo "state           $CACHE"
    echo "date            $DATE"
  else
    mkdir -p "$CACHE"
    exec 9>"$CACHE/lock"
    if ! flock -n 9; then
      echo "knowledge-weekly: another run is active (lock $CACHE/lock); exiting"
      exit 0
    fi
    if [[ -e $STARTED_FILE ]]; then
      echo "knowledge-weekly: a previous run left $STARTED_FILE; not stopping $LLM_UNIT (its runtime limit does)"
      rm -f "$STARTED_FILE"
    fi
    RUN_DIR="$CACHE/runs/$DATE"
    mkdir -p "$RUN_DIR"
    STAGES_FILE="$RUN_DIR/stages.tsv"
    : >"$STAGES_FILE"
    trap cleanup EXIT
    trap 'exit 143' TERM INT HUP
  fi

  run_stage 1 fetch-build stage_fetch_build
  run_stage 2 bulk-sync stage_bulk_sync
  run_stage 3 roles stage_roles
  run_stage 4 llm stage_llm
  run_stage 5 map-digest stage_map_digest
  if ((DRY)); then
    run_stage 6 report stage_report
    exit 0
  fi
  if [[ -z $ONLY || $ONLY == 6 || $ONLY == report ]]; then
    local rc=0
    # the report is written even when every stage failed
    set +e
    (
      set -e
      stage_report
    )
    rc=$?
    set -e
    if ((rc != 0)); then
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
  # keep the logs of the last 12 runs
  local old
  while read -r old; do rm -rf "${CACHE:?}/runs/$old"; done < <(ls -1 "$CACHE/runs" 2>/dev/null | sort | head -n -12)
  exit 0
}

# sourced by a test: only the functions and the configuration are defined
if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  main "$@"
fi
