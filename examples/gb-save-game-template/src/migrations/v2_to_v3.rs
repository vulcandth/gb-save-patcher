use gb_save_core::{Patch, PatchKind, PatchLogSink, PatchMetadata, SaveBinary, SaveResult, SymbolDatabase};

use crate::game::SAVE_VERSION_ABS_ADDRESS;

const PATCH_ID: &str = "example.migration.v2_to_v3";

#[derive(Debug)]
pub struct MigrateV2ToV3;

impl Patch for MigrateV2ToV3 {
    fn metadata(&self) -> PatchMetadata {
        PatchMetadata {
            id: PATCH_ID,
            kind: PatchKind::Migration,
            from_version: Some(2),
            to_version: Some(3),
        }
    }

    fn apply(&self, save: &mut SaveBinary, _symbols: &SymbolDatabase) -> SaveResult<()> {
        save.write_u16_le(gb_save_core::Address(SAVE_VERSION_ABS_ADDRESS), 3)
    }

    fn apply_with_log(
        &self,
        save: &mut SaveBinary,
        symbols: &SymbolDatabase,
        log: &mut dyn PatchLogSink,
    ) -> SaveResult<()> {
        log.info(PATCH_ID, "applying example migration 2 -> 3");
        self.apply(save, symbols)
    }
}

pub static MIGRATE_V2_TO_V3: MigrateV2ToV3 = MigrateV2ToV3;
