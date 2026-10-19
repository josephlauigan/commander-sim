# Handoff t6-ai-windows
Started 2026-10-18. Task `windows` in rust/agents/t6-ai.md.
## Done
- nothing yet
## Next
1. read combos.rs windows / t6_going / piece_gain; 2. baseline Godo + T6 (300 games); 3. implement; 4. tests; 5. after sims; 6. t3 lock; 7. report
## Gotchas
- baseline binary /tmp/claude-1000/t6ai-windows/base-sim built; running baseline sims (godo, t6 within) 300 games seed 1
- Written (uncommitted until built): combos::normal_play_window (end of combos.rs), hooked in stack::ability_window (src perm) and t3::helm. Mechanism: a T6 action on a piece of a line whose groups are all on bf -> `interrupted` (opps' instant removal/Tidebinder) on the pieces. Next: build, test file tests/ai_t6_windows.rs, measure vs baseline (/tmp/claude-1000/t6ai-windows/base-godo.txt, base-t6.txt seed 1), t3 lock.
- built OK; running after sims -> /tmp/claude-1000/t6ai-windows/new-godo.txt, new-t6.txt
- tests/ai_t6_windows.rs written; building+running it
- tests pass (4). now: rebuild, rerun godo+t6 after (new-*.txt), t3 lock, full tests, report
- after sims done (new-godo/new-t6). running t3 lock then full tests, then report
