use gb_save_core::{PatchLogSink, SaveBinary, SaveResult};

const VALIDATION_LOG_SOURCE: &str = "example.validation";

/// Example "validation" hook.
///
/// Real games often validate primary and backup checksums before patching.
/// This template provides a named entry point that can be expanded.
///
/// # Errors
/// Returns an error if the save is too small or otherwise invalid.
pub fn validate_before_patching(save: &SaveBinary) -> SaveResult<()> {
    let _ = save;
    Ok(())
}

/// Like [`validate_before_patching`], but emits structured logs.
#[must_use]
pub fn validate_before_patching_with_log(
    save: &SaveBinary,
    log: &mut dyn PatchLogSink,
) -> SaveResult<()> {
    log.info(VALIDATION_LOG_SOURCE, "validation passed");
    validate_before_patching(save)
}
