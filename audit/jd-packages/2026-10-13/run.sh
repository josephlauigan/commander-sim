#!/bin/bash
# JD's four engine packages (2026-10-13): paired test of each package against the current list, 500 games per list
# per tier, Tiers 1-5, look-ahead AI, loose profile, nice 19. Threads: JOBS (default 7, the per-session share in
# ~/projects/mtg_sims/cpu-ledger.md; raise it only for packages started after the other sessions free threads).
# Resumable: a finished tier is skipped. Never edit or merge into this worktree while it runs (decks are read live).
# Usage: run.sh [PKG ...]   (default C A B D)
cd "$(dirname "$0")/../../.."
D=audit/jd-packages/2026-10-13
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/f5d987cb-18a2-4572-90f1-24faee8ab577/scratchpad/csim-pkg2}
declare -A DECK SW
DECK[A]=kefka;   SW[A]="Twinflame=>Yawgmoth's Will|Kaervek's Purge=>Vampiric Tutor|Disintegrate=>Mystical Tutor"
DECK[B]=veyran;  SW[B]="Lightning Strike=>Underworld Breach|Visions of Beyond=>Terror of the Peaks|Flux Channeler=>Mirari"
DECK[C]=sauron;  SW[C]="Bedevil=>Sheoldred, the Apocalypse"
DECK[D]=titania; SW[D]="Yavimaya Wurm=>Vorinclex, Monstrous Raider|Elfhame Wurm=>Kura, the Boundless Sky|Cactarantula=>Tireless Tracker|Penumbra Spider=>Cartographer's Survey"
pkgs=("$@"); [ ${#pkgs[@]} -eq 0 ] && pkgs=(C A B D)
for k in "${pkgs[@]}"; do
  args=(); IFS='|' read -ra parts <<< "${SW[$k]}"; for s in "${parts[@]}"; do args+=(--swap "$s"); done
  for t in t1 t2 t3 t4 t5; do
    out=$D/pkg-$k-$t.txt
    grep -q "delta" "$out" 2>/dev/null && continue
    J=$(cat $D/jobs 2>/dev/null || echo ${JOBS:-7})
    nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 \
      "$BIN" --deck "${DECK[$k]}" --pool $t --profile loose --games 500 --ai lookahead --jobs $J --quiet "${args[@]}" > "$out" 2>&1
    echo "$(date +%H:%M) $k ${DECK[$k]} $t jobs $J exit $? $(grep -m1 'delta' "$out")" | tee -a $D/log.txt
  done
done
echo "$(date +%H:%M) DONE ${pkgs[*]}" | tee -a $D/log.txt
