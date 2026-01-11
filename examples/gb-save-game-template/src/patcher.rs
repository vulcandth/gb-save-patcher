use gb_save_core::{
    resolve_migration_plan, PatchLogEntry, PatchLogSink, SaveBinary, SaveError, SaveResult,
    VecPatchLogSink,
};

use crate::{fixes::example_fix_patches, game::get_save_version, migrations::example_migrations, symbols::{supported_version_from_u16, symbols_for_version}};

const PATCHER_LOG_SOURCE: &str = "example.patcher";

/// Result of patching that also includes patch-internal logs.
#[derive(Debug)]
pub struct PatchSaveOutcome {
    pub bytes: Option<Vec<u8>>,
    pub logs: Vec<PatchLogEntry>,
    pub error: Option<String>,
}

impl From<PatchSaveOutcome> for gb_save_cli::PatchOutcome {
    fn from(value: PatchSaveOutcome) -> Self {
        Self {
            ok: value.error.is_none(),
            bytes: value.bytes,
            error: value.error,
            logs: value.logs,
        }
    }
}

/// Applies either a fix patch (`dev_type != 0`) or a version migration (`dev_type == 0`).
///
/// For fix patches, `target_version` must match the save's current version.
///
/// # Errors
/// Returns an error if the save cannot be parsed, the requested patch is unknown, or if patching
/// fails.
pub fn patch_save_bytes(bytes: Vec<u8>, target_version: u16, dev_type: u8) -> SaveResult<Vec<u8>> {
    let mut save = SaveBinary::new(bytes);

    crate::validation::validate_before_patching(&save)?;

    let current_version = get_save_version(&save)?;

    if dev_type != 0 {
        if target_version != current_version {
            return Err(SaveError::InvalidSaveState {
                reason: "fix patches do not migrate; target_version must match current save version"
                    .to_string(),
            });
        }

        let fix = example_fix_patches()
            .into_iter()
            .find(|p| p.dev_type == dev_type)
            .ok_or(SaveError::UnknownFixPatch { dev_type })?;

        let symbols = symbols_for_version(supported_version_from_u16(current_version)?)?;
        fix.patch.apply(&mut save, &symbols)?;
        return Ok(save.into_bytes());
    }

    if current_version == target_version {
        return Ok(save.into_bytes());
    }

    let migrations = example_migrations();
    let plan = resolve_migration_plan(&migrations, current_version, target_version)?;

    for patch in plan {
        let meta = patch.metadata();
        let from = meta
            .from_version
            .expect("migration patches must have from_version");
        let symbols = symbols_for_version(supported_version_from_u16(from)?)?;
        patch.apply(&mut save, &symbols)?;
    }

    Ok(save.into_bytes())
}

/// Like [`patch_save_bytes`], but captures structured logs and returns a non-error outcome type.
#[must_use]
pub fn patch_save_bytes_with_log(
    bytes: Vec<u8>,
    target_version: u16,
    dev_type: u8,
) -> PatchSaveOutcome {
    let mut log = VecPatchLogSink::new();

    let mut save = SaveBinary::new(bytes);

    if let Err(e) = crate::validation::validate_before_patching_with_log(&save, &mut log) {
        let msg = e.to_string();
        log.error(PATCHER_LOG_SOURCE, &msg);
        return PatchSaveOutcome {
            bytes: None,
            logs: log.into_entries(),
            error: Some(msg),
        };
    }

    let current_version = match get_save_version(&save) {
        Ok(v) => v,
        Err(e) => {
            let msg = e.to_string();
            log.error(PATCHER_LOG_SOURCE, &msg);
            return PatchSaveOutcome {
                bytes: None,
                logs: log.into_entries(),
                error: Some(msg),
            };
        }
    };

    if dev_type != 0 {
        if target_version != current_version {
            let msg = "fix patches do not migrate; target_version must match current save version";
            log.error(PATCHER_LOG_SOURCE, msg);
            return PatchSaveOutcome {
                bytes: None,
                logs: log.into_entries(),
                error: Some(msg.to_string()),
            };
        }

        let fix = match example_fix_patches().into_iter().find(|p| p.dev_type == dev_type) {
            Some(p) => p,
            None => {
                let msg = SaveError::UnknownFixPatch { dev_type }.to_string();
                log.error(PATCHER_LOG_SOURCE, &msg);
                return PatchSaveOutcome {
                    bytes: None,
                    logs: log.into_entries(),
                    error: Some(msg),
                };
            }
        };

        log.info(
            PATCHER_LOG_SOURCE,
            &format!(
                "applying fix patch dev_type={dev_type} id={}",
                fix.patch.metadata().id
            ),
        );

        let symbols = match supported_version_from_u16(current_version).and_then(symbols_for_version)
        {
            Ok(s) => s,
            Err(e) => {
                let msg = e.to_string();
                log.error(PATCHER_LOG_SOURCE, &msg);
                return PatchSaveOutcome {
                    bytes: None,
                    logs: log.into_entries(),
                    error: Some(msg),
                };
            }
        };

        if let Err(e) = fix.patch.apply_with_log(&mut save, &symbols, &mut log) {
            let msg = e.to_string();
            log.error(fix.patch.metadata().id, &msg);
            return PatchSaveOutcome {
                bytes: None,
                logs: log.into_entries(),
                error: Some(msg),
            };
        }

        return PatchSaveOutcome {
            bytes: Some(save.into_bytes()),
            logs: log.into_entries(),
            error: None,
        };
    }

    if current_version == target_version {
        return PatchSaveOutcome {
            bytes: Some(save.into_bytes()),
            logs: log.into_entries(),
            error: None,
        };
    }

    let migrations = example_migrations();
    let plan = match resolve_migration_plan(&migrations, current_version, target_version) {
        Ok(p) => p,
        Err(e) => {
            let msg = e.to_string();
            log.error(PATCHER_LOG_SOURCE, &msg);
            return PatchSaveOutcome {
                bytes: None,
                logs: log.into_entries(),
                error: Some(msg),
            };
        }
    };

    let plan_ids = plan
        .iter()
        .map(|p| p.metadata().id)
        .collect::<Vec<_>>()
        .join(" -> ");
    log.info(
        PATCHER_LOG_SOURCE,
        &format!("migration plan {current_version} -> {target_version}: {plan_ids}"),
    );

    for patch in plan {
        let meta = patch.metadata();
        let from = meta
            .from_version
            .expect("migration patches must have from_version");

        let symbols = match supported_version_from_u16(from).and_then(symbols_for_version) {
            Ok(s) => s,
            Err(e) => {
                let msg = e.to_string();
                log.error(PATCHER_LOG_SOURCE, &msg);
                return PatchSaveOutcome {
                    bytes: None,
                    logs: log.into_entries(),
                    error: Some(msg),
                };
            }
        };

        if let Err(e) = patch.apply_with_log(&mut save, &symbols, &mut log) {
            let msg = e.to_string();
            log.error(meta.id, &msg);
            return PatchSaveOutcome {
                bytes: None,
                logs: log.into_entries(),
                error: Some(msg),
            };
        }
    }

    PatchSaveOutcome {
        bytes: Some(save.into_bytes()),
        logs: log.into_entries(),
        error: None,
    }
}

pub fn patch_save_bytes_with_log_for_cli(
    bytes: Vec<u8>,
    target_version: u16,
    dev_type: u8,
) -> gb_save_cli::PatchOutcome {
    patch_save_bytes_with_log(bytes, target_version, dev_type).into()
}

pub fn patch_save_bytes_for_cli(
    bytes: Vec<u8>,
    target_version: u16,
    dev_type: u8,
) -> anyhow::Result<Vec<u8>> {
    patch_save_bytes(bytes, target_version, dev_type).map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn detect_version_for_cli(bytes: &[u8]) -> anyhow::Result<u16> {
    let save = SaveBinary::new(bytes.to_vec());
    get_save_version(&save).map_err(|e| anyhow::anyhow!(e.to_string()))
}

#[cfg(target_arch = "wasm32")]
pub fn patch_save_bytes_for_wasm(bytes: &[u8], target_version: u16, dev_type: u8) -> anyhow::Result<Vec<u8>> {
    patch_save_bytes(bytes.to_vec(), target_version, dev_type)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

#[cfg(target_arch = "wasm32")]
pub fn patch_save_bytes_with_log_for_wasm(bytes: &[u8], target_version: u16, dev_type: u8) -> PatchSaveOutcome {
    patch_save_bytes_with_log(bytes.to_vec(), target_version, dev_type)
}

#[cfg(target_arch = "wasm32")]
pub fn detect_version_for_wasm(bytes: &[u8]) -> anyhow::Result<u16> {
    detect_version_for_cli(bytes)
}

#[cfg(target_arch = "wasm32")]
pub fn validate_before_patching_with_log_for_patcher(
    save: &SaveBinary,
    log: &mut dyn PatchLogSink,
) -> SaveResult<()> {
    crate::validation::validate_before_patching_with_log(save, log)
}
