//! Karn, Gilded Guardian ({2/W}{2/U}{2/B}{2/R}{2/G}, legendary artifact creature - Golem 5/5): card code for "When Karn
//! enters, draw a card for each color among other artifacts you control" (E3 of the karn_gilded deck work). Stub: the
//! worker of that branch writes it. Not Karn, Legacy Reforged (karn.rs, karn_cards.rs, karn_engines.rs).

use crate::cards::CardDb;
use crate::hooks::Registry;

pub const KARN_GILDED: &str = "Karn, Gilded Guardian";

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
