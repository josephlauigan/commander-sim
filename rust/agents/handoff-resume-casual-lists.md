# Casual lists handoff

- Final Brago t2 and Korvold t3 source, export and Spellbook data are updated. Verification, combo limitations and prior evidence: `audit/casual-lists/implementation.md`.
- Current `cardgen validate` passes; every net addition audited Full. Spellbook exposed the inherited Drake / Eldrazi Displacer two-card infinite; Displacer was cut and the refreshed Brago data has no included two-card line.
- Coordinator must independently review the full branch diff and run the Spellbook guard, export equivalence and full release suite. No tests or simulations were run by this agent.
- Final lists are not measured. Run `/tmp/resume-casual-queue.sh` with a final reviewed `commander-sim` binary, review the results and record findings. Prior candidate Korvold results remain below target and do not prove final-list strength.
