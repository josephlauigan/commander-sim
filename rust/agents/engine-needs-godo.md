# Engine needs: batch `godo`

## Godo, Bandit Warlord
Needs: no cap on additional combat phases (the combat loop stops at four combats a turn); Godo + Helm of the Host makes one more combat per hasty token, so the loop needs to run until the table is dead.
Where it would hook: `engine/combat.rs` `combat` (`g.player(p).extra_combats > 0 && ncomb < 4`, line ~1127); a bound that stops a true infinite loop (all opponents dead, or a few dozen combats) should replace the 4.
Card side done: ETB fetches an Equipment onto the battlefield (Helm, or Hammer first with a copier), first attack each turn untaps Godo and Samurai and adds one combat (`extra_combats += 1`), `impls/t6_godo.rs`. Helm's token copy (t3::helm) triggers Godo's ETB and attack as the card does.
Test to add once the engine has it: Godo + Helm with six opposing creatures-free players at 40 life: all opponents die in the one turn.
