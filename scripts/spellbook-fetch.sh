#!/usr/bin/env bash
# Fetch the combos Commander Spellbook finds in every deck of data/decks.json, into data/spellbook/<key>.json.
#
# Combo data from Commander Spellbook (https://commanderspellbook.com), via its public API
# (https://backend.commanderspellbook.com/find-my-combos). Please credit them wherever this data is shown.
#
# Polite by design: one POST per deck, at most one every 2 seconds, and a deck whose card list has not changed since
# its last fetch (same request hash) is skipped. 429 responses are retried after Retry-After (or 60 s).
#
# usage: scripts/spellbook-fetch.sh [--force] [deck-key ...]   (no keys: every deck)
# needs: curl, jq, sha256sum
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
decks="$root/data/decks.json"
out="$root/data/spellbook"
api="https://backend.commanderspellbook.com/find-my-combos?limit=1000"
ua="commander-sim/0.1 (personal Commander game simulator; one request per deck, cached)"
force=0
if [[ "${1:-}" == "--force" ]]; then force=1; shift; fi
mkdir -p "$out"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

keys=("$@")
if [[ ${#keys[@]} -eq 0 ]]; then
  mapfile -t keys < <(jq -r '(.mine + .pool)[].key' "$decks")
fi

# the compact form kept per variant: only what the sim reads (results: Standalone and Contextual features only, the
# Helper / Utility ones are Spellbook's own bookkeeping)
compact='def v: {
    id,
    cards: [.uses[] | {name: .card.name, zones: .zoneLocations} + (if .mustBeCommander then {commander: true} else {} end)
            + (if .quantity > 1 then {quantity} else {} end)],
    results: [.produces[] | select(.feature.status == "S" or .feature.status == "C") | .feature.name],
    requires: [.requires[] | .template.name],
    prerequisites: ([.easyPrerequisites, .notablePrerequisites] | map(select(. != "")) | join(" ")),
    mana: .manaNeeded,
    bracket: .bracketTag,
    legal: .legalities.commander,
    popularity
  };'

first=1
for key in "${keys[@]}"; do
  deck="$(jq -c --arg k "$key" '(.mine + .pool)[] | select(.key == $k)' "$decks")"
  if [[ -z "$deck" ]]; then echo "no deck $key in data/decks.json" >&2; exit 1; fi
  # the request: the commander(s) apart, the other cards with their counts
  jq -c '[.commander, (.commander2 // empty)] as $cmd
    | {commanders: [$cmd[] | {card: ., quantity: 1}],
       main: ([.cards[] | select(. as $c | $cmd | index($c) | not)] | group_by(.) | map({card: .[0], quantity: length}))}' \
    <<<"$deck" >"$tmp/req.json"
  hash="$(sha256sum "$tmp/req.json" | cut -d' ' -f1)"
  dest="$out/$key.json"
  if [[ $force -eq 0 && -f "$dest" ]] && [[ "$(jq -r '.request_sha256' "$dest")" == "$hash" ]]; then
    echo "$key: unchanged, skipped"
    continue
  fi
  [[ $first -eq 1 ]] || sleep 2
  first=0
  for try in 1 2 3; do
    code="$(curl -sS -A "$ua" -H 'Content-Type: application/json' -X POST --data @"$tmp/req.json" \
      -D "$tmp/headers" -o "$tmp/resp.json" -w '%{http_code}' "$api")"
    if [[ "$code" == 429 ]]; then
      wait="$(grep -i '^retry-after:' "$tmp/headers" | tr -dc '0-9')"
      echo "$key: 429, waiting ${wait:-60}s" >&2
      sleep "${wait:-60}"
      continue
    fi
    break
  done
  if [[ "$code" != 200 ]]; then echo "$key: HTTP $code" >&2; head -c 500 "$tmp/resp.json" >&2; exit 1; fi
  if [[ "$(jq -r '.next' "$tmp/resp.json")" != null ]]; then
    echo "$key: the response has a next page; raise the limit" >&2; exit 1
  fi
  jq --arg key "$key" --arg hash "$hash" --arg date "$(date -u +%Y-%m-%d)" --slurpfile req "$tmp/req.json" "$compact"'
    ($req[0] | [.commanders[].card, .main[].card]) as $deck
    | {
        source: "Commander Spellbook (https://commanderspellbook.com), find-my-combos API (https://backend.commanderspellbook.com/find-my-combos)",
        credit: "Combo data by Commander Spellbook, https://commanderspellbook.com. Commander Spellbook is unofficial Fan Content permitted under the Wizards of the Coast Fan Content Policy.",
        fetched: $date,
        deck: $key,
        commanders: [$req[0].commanders[].card],
        request_sha256: $hash,
        identity: .results.identity,
        included: [.results.included[] | v],
        included_by_changing_commanders: [.results.includedByChangingCommanders[] | v],
        almost_included: [.results.almostIncluded[] | v
          | .missing = [.cards[].name | select(. as $n | $deck | index($n) | not)]]
      }' "$tmp/resp.json" >"$dest"
  echo "$key: $(jq '.included | length' "$dest") included, $(jq '.almost_included | length' "$dest") almost included"
done
