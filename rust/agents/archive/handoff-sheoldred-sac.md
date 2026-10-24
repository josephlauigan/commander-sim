# Handoff: sheoldred-sac
Done: Tower (land option in sheoldred.rs, override Full), aristocrat_options for any main deck with outlets (not seph),
sac_in_response for dying kinds, sac_before_wipe in decks.rs wipe_response, seph_flip_bonus in death_value, Carnage Altar
fodder filter (dsl/mod.rs), stats creatures_sacrificed / seph_flips, tests cards_sheoldred_sac.rs.
Next: full cargo test, all_decks checks, smoke run, report.
Gaps: no combat-death hook (a creature dying in combat is not sacrificed in response).
