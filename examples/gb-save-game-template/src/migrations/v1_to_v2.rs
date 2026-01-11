use gb_save_core::{Patch, PatchKind, PatchLogSink, PatchMetadata, SaveBinary, SaveResult, SymbolDatabase};

use crate::game::SAVE_VERSION_ABS_ADDRESS;

const PATCH_ID: &str = "example.migration.v1_to_v2";

#[derive(Debug)]
pub struct MigrateV1ToV2;

impl Patch for MigrateV1ToV2 {
    fn metadata(&self) -> PatchMetadata {
        PatchMetadata {
            id: PATCH_ID,
            kind: PatchKind::Migration,
            from_version: Some(1),
            to_version: Some(2),
        }
    }

    fn apply(&self, save: &mut SaveBinary, _symbols: &SymbolDatabase) -> SaveResult<()> {
        save.write_u16_le(gb_save_core::Address(SAVE_VERSION_ABS_ADDRESS), 2)
    }

    fn apply_with_log(
        &self,
        save: &mut SaveBinary,
        symbols: &SymbolDatabase,
        log: &mut dyn PatchLogSink,
    ) -> SaveResult<()> {
        log.info(PATCH_ID, "applying example migration 1 -> 2");
        self.apply(save, symbols)
    }
}

pub static MIGRATE_V1_TO_V2: MigrateV1ToV2 = MigrateV1ToV2;
