use gb_save_core::{Patch, PatchKind, PatchLogSink, PatchMetadata, SaveBinary, SaveError, SaveResult, SymbolDatabase};

const FIX_PATCH_ID: &str = "example.fix.dev_type_1";

#[derive(Debug)]
pub struct FixDevType1;

impl Patch for FixDevType1 {
    fn metadata(&self) -> PatchMetadata {
        PatchMetadata {
            id: FIX_PATCH_ID,
            kind: PatchKind::Fix,
            from_version: None,
            to_version: None,
        }
    }

    fn apply(&self, _save: &mut SaveBinary, _symbols: &SymbolDatabase) -> SaveResult<()> {
        Err(SaveError::NotImplemented {
            feature: "example fix patch logic".to_string(),
        })
    }

    fn apply_with_log(
        &self,
        save: &mut SaveBinary,
        symbols: &SymbolDatabase,
        log: &mut dyn PatchLogSink,
    ) -> SaveResult<()> {
        log.info(FIX_PATCH_ID, "example fix patches are stubs in the template");
        self.apply(save, symbols)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FixPatchSpec {
    pub dev_type: u8,
    pub patch: &'static dyn Patch,
}

pub static FIX_PATCH_DEV_TYPE_1: FixDevType1 = FixDevType1;

/// Returns all fix patches for this example game.
#[must_use]
pub fn example_fix_patches() -> Vec<FixPatchSpec> {
    vec![FixPatchSpec {
        dev_type: 1,
        patch: &FIX_PATCH_DEV_TYPE_1,
    }]
}
