#!/bin/bash
# drives a human game on kraum through the HTTP API: lands, taps, casts commanders (cmd 0/1), passes
U=http://127.0.0.1:18765
round=-1; landed=0; tried=0; steps=0
post() { curl -s -XPOST $U/api/answer -H 'content-type: application/json' -d "{\"id\":$1,\"answer\":$2}"; }
while [ $steps -lt 1500 ]; do
  steps=$((steps+1))
  st=$(curl -s $U/api/state)
  p=$(echo "$st" | jq -c '.pending')
  if [ "$p" = "null" ]; then
    over=$(echo "$st" | jq -r '.view.over // .game.over // false')
    [ "$over" = "true" ] && { echo "game over"; break; }
    sleep 0.3; continue
  fi
  id=$(echo "$p" | jq -r .id); kind=$(echo "$p" | jq -r .request.kind)
  if [ "$kind" != "priority" ]; then
    [ "$kind" = "attack" ] && a='[]' || a=0
    post $id "$a" >/dev/null; continue
  fi
  me=$(echo "$st" | jq -c '.view.players[]|select(.you)')
  r=$(echo "$st" | jq -r '.view.round')
  if [ "$r" != "$round" ]; then round=$r; landed=0; tried=0; fi
  li=$(echo "$me" | jq -r '[.hand_land|to_entries[]|select(.value)|.key][0] // empty')
  if [ $landed = 0 ] && [ -n "$li" ]; then landed=1; post $id "{\"do\":\"land\",\"card\":$li}" >/dev/null; continue; fi
  inz=$(echo "$me" | jq -r '(.commander_in_zone or .commander2_in_zone)')
  src=$(echo "$me" | jq -r '.mana_sources[0].id // empty')
  if [ "$inz" = "true" ] && [ -n "$src" ]; then post $id "{\"do\":\"tap\",\"source\":$src}" >/dev/null; continue; fi
  pool=$(echo "$me" | jq -r '.mana_pool'); n=$(echo -n "$pool" | tr -cd "{" | wc -c)
  t1=$((3 + $(echo "$me" | jq .tax))); t2=$((5 + $(echo "$me" | jq .tax2)))
  z1=$(echo "$me" | jq -r .commander_in_zone); z2=$(echo "$me" | jq -r .commander2_in_zone)
  if [ $tried = 0 ] && [ $n -gt 0 ]; then
    if [ "$z2" = true ] && [ $n -ge $t2 ] && [[ $pool == *U* && $pool == *R* ]]; then
      tried=1; echo "R$r cast Kraum (cmd 1) pool=$pool tax2=$((t2-5))"; post $id '{"do":"cast","zone":"cmd","cmd":1}' | jq -c .; continue; fi
    if [ "$z1" = true ] && [ $n -ge $t1 ] && [[ $pool == *W* && $pool == *B* ]]; then
      tried=1; echo "R$r cast Tymna (cmd 0) pool=$pool tax=$((t1-3))"; post $id '{"do":"cast","zone":"cmd","cmd":0}' | jq -c .; continue; fi
  fi
  post $id '{"do":"pass"}' >/dev/null
done
curl -s $U/api/state | jq -c '.view.players[]|select(.you)|{commander,commander_in_zone,tax,commander2,commander2_in_zone,tax2,battlefield:[.battlefield[]|select(.commander)|.name]}'
