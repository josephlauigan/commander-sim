# Handoff: bill-cards

## Findings
- "Full" in the audit = an entry in data/card_overrides.toml with status = "Full" + note (checked first by card_status). Adding only status+note keeps the card's data.
- Counters are `Perm::plus`. Central writers: cardcode::add_counters, dsl "counters"/"proliferate"/"double_counters", common::proliferate, zones enter (Enter.plus).
- Bill's landfall comes from the compiled DSL (target chosen by dsl::choose_target -> max pval among mine).
## Done
- Hardened Scales: cardcode::scales(g,p), applied in add_counters, dsl counters/proliferate/double_counters, common::proliferate, zones Enter.plus (not yet tested/committed)
## Next (in order)
Bill doubling ability, Greenbelt disguise, collect evidence, web-slinging, plot/foretell cards, small partials, AI plan, tests, overrides.
