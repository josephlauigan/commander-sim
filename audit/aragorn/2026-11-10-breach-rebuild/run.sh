#!/usr/bin/env bash
# Aragorn Breach rebuild, paired swap runs (JD approved C + B1, 2026-11-10). Own tiers (t4, t5: --pool all),
# look-ahead AI and loose profile (CLI defaults), 1,500 games per list per tier (default), seed 500000, explain and
# --no-preflight (JD, 11/10: Thassa's Oracle and Lion's Eye Diamond are cast only in the Breach line, so the preflight
# stops on them by design; explain stays on). Memory cap: ulimit -v (no systemd in the cloud container). Usage: bash audit/aragorn/2026-11-10-breach-rebuild/run.sh
cd "$(dirname "$0")/../../.."
OUT=${OUT:-audit/aragorn/2026-11-10-breach-rebuild}  # OUT=<dir> writes elsewhere (the cloud session: the scratchpad until a run ends)
BIN=rust/target/release/commander-sim
git rev-parse --short HEAD > $OUT/commit.txt
B1=("Elspeth, Sun's Champion=>Lion's Eye Diamond"
    "Triple Triad=>Birgi, God of Storytelling // Harnfel, Horn of Bounty"
    "Niv-Mizzet, Parun=>Enlightened Tutor"
    "Restoration Angel=>Gamble"
    "Kitchen Finks=>Grand Abolisher")
B2=("Stormcarved Coast=>Mana Vault" "Spirebluff Canal=>Chrome Mox" "Sulfur Falls=>Grim Monolith"
    "Spark Double=>Brainstorm" "Fact or Fiction=>Ponder" "Twinned Vision=>Preordain" "Comet Storm=>Gitaxian Probe"
    "Faeburrow Elder=>Noble Hierarch" "Talrand, Sky Summoner=>Flusterstorm")
B3=("Frontier Bivouac=>Scalding Tarn" "Jungle Shrine=>Wooded Foothills" "Mystic Monastery=>Arid Mesa"
    "Seaside Citadel=>Bloodstained Mire" "Path of Ancestry=>Sacred Foundry" "Exotic Orchard=>Stomping Ground"
    "Glacial Fortress=>Breeding Pool" "Sunpetal Grove=>Hallowed Fountain")
B4=("Ephemerate=>Thought Scour" "Heroic Intervention=>Pyretic Ritual" "Mystic Confluence=>Wheel of Fortune"
    "Aura Shards=>Baral, Chief of Compliance" "Ghostly Prison=>Thassa's Oracle" "Swan Song=>Strike It Rich"
    "Skullclamp=>Cathartic Reunion")
run() {
  local name=$1; shift
  local args=(); for s in "$@"; do args+=(--swap "$s"); done
  local start=$(date +%s)
  ( ulimit -v 14000000; $BIN --deck aragorn --pool all --seed 500000 --no-preflight "${args[@]}" > $OUT/$name.txt 2>&1 < /dev/null )
  echo "$name exit=$? $(( ($(date +%s)-start)/60 ))m $(date +%H:%M)" | tee -a $OUT/status.txt
}
case "${1:-C+B1}" in
  C+B1) run C "${B1[@]}" "${B2[@]}" "${B3[@]}" "${B4[@]}"; run B1 "${B1[@]}" ;;
  C) run C "${B1[@]}" "${B2[@]}" "${B3[@]}" "${B4[@]}" ;;
  B1) run B1 "${B1[@]}" ;;
  rest) run B2 "${B2[@]}"; run B3 "${B3[@]}"; run B4 "${B4[@]}" ;;
esac
echo "ALL DONE $(date +%H:%M)" | tee -a $OUT/status.txt
