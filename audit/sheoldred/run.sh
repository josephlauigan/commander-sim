#!/bin/bash
# Sheoldred (key sheoldred, bulk-only list) and the Jodah trade blocks (2026-10-11). Look-ahead AI, loose profile.
#   base        : Sheoldred against every tier, 960 games per tier, seeds 500000-500959 (pool-results.md section 0d)
#   sheo-A/B/C  : Jodah's cards into Sheoldred, paired, 1,000 games per list per tier
#   jodah-A/B/C : the same cards out of Jodah, refilled from bulk, paired, 1,000 games per list per tier
#   sheo-ABC, jodah-ABC : all three blocks at once
#   upgrade     : the upgrade package (buys) in the list with Memory Jar: Seizan, Underworld Dreams, Dark Deal, Painful
#                 Lesson for Ribskiff, Gravblade Heavy, Multiply by Zero, Locust Spray
#   upgrade2    : the first package plus the second (about $44 in all, 20 cards; see SU2)
#   gix-u2      : Gix, Yawgmoth Praetor for Hullcarver, on the list with both packages applied
#   mikokoro    : (2026-10-12, list with Mikokoro for a Swamp) Mikokoro back out for a Swamp; its value is minus the delta
# Block A (wheel): Memory Jar.             Sheoldred cuts Carnage Altar;   Jodah gets Szadek, Lord of Secrets.
# Block B (sweeper + removal): Toxic Deluge, Bitter Triumph.
#                                          Sheoldred cuts Depressurize, Multiply by Zero;
#                                          Jodah gets Lagrella, the Magpie and a spare Wander Off.
# Block C (body): Korlash, Heir to Blackblade. Sheoldred cuts Gravblade Heavy; Jodah gets Sisters of Stone Death.
# 22 worker threads by default (JOBS=11 while another session runs sims). Usage: run.sh <mode>...
cd "$(dirname "$0")/../.."
D=audit/sheoldred
SIM=./rust/target/release/commander-sim
JOBS=${JOBS:-22}  # 11 while another session runs sims
cap() { systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 "$@"; }

SA=(--swap "Carnage Altar=>Memory Jar")
SB=(--swap "Depressurize=>Toxic Deluge" --swap "Multiply by Zero=>Bitter Triumph")
SC=(--swap "Gravblade Heavy=>Korlash, Heir to Blackblade")
SU=(--swap "Ribskiff=>Seizan, Perverter of Truth" --swap "Locust Spray=>Painful Lesson" --swap "Gravblade Heavy=>Underworld Dreams" --swap "Multiply by Zero=>Dark Deal")
SU2=(--swap "Mabel, Bitter Recluse=>Howling Mine" --swap "Moseo, Vein's New Dean=>Temple Bell" --swap "Burglar Rat=>Fate Unraveler" --swap "Fathom Fleet Cutthroat=>Peer into the Abyss" --swap "Canyon Crawler=>Mind Stone" --swap "Necrosquito=>Commander's Sphere" --swap "Rampart Hunter=>Charcoal Diamond" --swap "Callous Inspector=>Phyrexian Arena" --swap "Depressurize=>Read the Bones" --swap "End of the Hunt=>Ambition's Cost" --swap "Tome of Legends=>Syphon Mind" --swap "Diary of Dreams=>Exsanguinate" --swap "Dark Endurance=>Gray Merchant of Asphodel" --swap "Lash of the Balrog=>Swiftfoot Boots" --swap "Swamp=>Geier Reach Sanitarium" --swap "Swamp=>Castle Locthwain")
JA=(--swap "Memory Jar=>Szadek, Lord of Secrets")
JB=(--swap "Toxic Deluge=>Lagrella, the Magpie" --swap "Bitter Triumph=>Wander Off")
JC=(--swap "Korlash, Heir to Blackblade=>Sisters of Stone Death")

paired() { # name deck swaps...
  local name=$1 deck=$2; shift 2
  mkdir -p $D/$name
  for t in t1 t2 t3 t4 t5; do
    cap $SIM --deck $deck --pool $t --profile loose --games 1000 --ai lookahead --jobs $JOBS --brief "$@" 2>&1 \
      | grep -v progress > $D/$name/$t.txt
    echo "$name $t exit ${PIPESTATUS[0]} $(grep -m1 'delta' $D/$name/$t.txt)" | tee -a $D/log.txt
  done
}

for mode in "$@"; do
  case $mode in
    base)
      mkdir -p $D/base
      for t in t1 t2 t3 t4 t5; do
        cap $SIM --deck sheoldred --pool $t --profile loose --games 960 --seed 500000 --ai lookahead --jobs $JOBS 2>&1 \
          | grep -v progress > $D/base/$t.txt
        echo "base $t exit ${PIPESTATUS[0]} $(grep -m1 '^Win rate' $D/base/$t.txt)" | tee -a $D/log.txt
      done ;;
    sheo-A) paired $mode sheoldred "${SA[@]}" ;;
    sheo-B) paired $mode sheoldred "${SB[@]}" ;;
    sheo-C) paired $mode sheoldred "${SC[@]}" ;;
    sheo-ABC) paired $mode sheoldred "${SA[@]}" "${SB[@]}" "${SC[@]}" ;;
    upgrade) paired $mode sheoldred "${SU[@]}" ;;
    upgrade2) paired $mode sheoldred "${SU[@]}" "${SU2[@]}" ;;
    gix-u2)
      # Gix, Yawgmoth Praetor (from Jodah) for Hullcarver, measured on the list with both upgrade packages: the doc
      # is rewritten with them for the run (cardgen deck) and restored from git afterwards
      F=decklists/JD/sheoldred-mono-black.md; X=tests/fixtures/my_decks_parsed.json
      git diff --quiet -- $F $X || { echo "uncommitted changes in $F or $X: commit them first"; exit 2; }
      trap 'git checkout -- $F $X' EXIT
      awk '/^## Import list/{f=1;next} f&&/^```$/{n++; if(n==2) exit; next} f&&n==1' $F | sed \
        -e 's/^1 Ribskiff$/1 Seizan, Perverter of Truth/' -e 's/^1 Locust Spray$/1 Painful Lesson/' \
        -e 's/^1 Gravblade Heavy$/1 Underworld Dreams/' -e 's/^1 Multiply by Zero$/1 Dark Deal/' \
        -e 's/^1 Mabel, Bitter Recluse$/1 Howling Mine/' -e "s/^1 Moseo, Vein's New Dean$/1 Temple Bell/" \
        -e 's/^1 Burglar Rat$/1 Fate Unraveler/' -e 's/^1 Fathom Fleet Cutthroat$/1 Peer into the Abyss/' \
        -e 's/^1 Canyon Crawler$/1 Mind Stone/' -e "s/^1 Necrosquito$/1 Commander's Sphere/" \
        -e 's/^1 Rampart Hunter$/1 Charcoal Diamond/' -e 's/^1 Callous Inspector$/1 Phyrexian Arena/' \
        -e 's/^1 Depressurize$/1 Read the Bones/' -e "s/^1 End of the Hunt$/1 Ambition's Cost/" \
        -e 's/^1 Tome of Legends$/1 Syphon Mind/' -e 's/^1 Diary of Dreams$/1 Exsanguinate/' \
        -e 's/^1 Dark Endurance$/1 Gray Merchant of Asphodel/' -e 's/^1 Lash of the Balrog$/1 Swiftfoot Boots/' \
        -e 's/^34 Swamp$/32 Swamp\n1 Geier Reach Sanitarium\n1 Castle Locthwain/' > $D/u2-list.txt
      ./rust/target/release/cardgen deck sheoldred $D/u2-list.txt --offline | grep -E "cards,|Game Changers"
      paired $mode sheoldred --swap "Hullcarver=>Gix, Yawgmoth Praetor"
      git checkout -- $F $X; trap - EXIT ;;
    mikokoro) paired $mode sheoldred --swap "Mikokoro, Center of the Sea=>Swamp" ;;
    jodah-A) paired $mode jodah "${JA[@]}" ;;
    jodah-B) paired $mode jodah "${JB[@]}" ;;
    jodah-C) paired $mode jodah "${JC[@]}" ;;
    jodah-ABC) paired $mode jodah "${JA[@]}" "${JB[@]}" "${JC[@]}" ;;
    *) echo "unknown mode $mode"; exit 2 ;;
  esac
done
