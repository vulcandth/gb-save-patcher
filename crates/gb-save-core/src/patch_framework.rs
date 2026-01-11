use crate::{SaveBinary, SaveError, SaveResult, SymbolDatabase};

/// Severity level for patch log output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatchLogLevel {
    /// Informational message.
    Info,
    /// Warning indicating a recoverable issue or unexpected state.
    Warning,
    /// Error indicating patching cannot safely proceed.
    Error,
}

/// A structured log entry emitted during patching.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatchLogEntry {
    /// The severity of the log entry.
    pub level: PatchLogLevel,
    /// A stable identifier for where the log came from (e.g. a patch id).
    pub source: &'static str,
    /// Human-readable log message.
    pub message: String,
}

impl PatchLogEntry {
    /// Creates an informational log entry.
    pub fn info(source: &'static str, message: impl Into<String>) -> Self {
        Self {
            level: PatchLogLevel::Info,
            source,
            message: message.into(),
        }
    }

    /// Creates a warning log entry.
    pub fn warning(source: &'static str, message: impl Into<String>) -> Self {
        Self {
            level: PatchLogLevel::Warning,
            source,
            message: message.into(),
        }
    }

    /// Creates an error log entry.
    pub fn error(source: &'static str, message: impl Into<String>) -> Self {
        Self {
            level: PatchLogLevel::Error,
            source,
            message: message.into(),
        }
    }
}

/// Collects patch log entries during patch application.
pub trait PatchLogSink {
    /// Records a log entry.
    fn push(&mut self, entry: PatchLogEntry);

    /// Convenience helper for emitting an info entry.
    fn info(&mut self, source: &'static str, message: &str) {
        self.push(PatchLogEntry::info(source, message));
    }

    /// Convenience helper for emitting a warning entry.
    fn warn(&mut self, source: &'static str, message: &str) {
        self.push(PatchLogEntry::warning(source, message));
    }

    /// Convenience helper for emitting an error entry.
    fn error(&mut self, source: &'static str, message: &str) {
        self.push(PatchLogEntry::error(source, message));
    }
}

/// A log sink that discards all entries.
#[derive(Debug, Default)]
pub struct NoopPatchLogSink;

impl PatchLogSink for NoopPatchLogSink {
    fn push(&mut self, _entry: PatchLogEntry) {}
}

/// A log sink that stores entries in a `Vec`.
#[derive(Debug, Default)]
pub struct VecPatchLogSink {
    entries: Vec<PatchLogEntry>,
}

impl VecPatchLogSink {
    /// Creates an empty in-memory log sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Consumes the sink and returns all collected entries.
    pub fn into_entries(self) -> Vec<PatchLogEntry> {
        self.entries
    }
}

impl PatchLogSink for VecPatchLogSink {
    fn push(&mut self, entry: PatchLogEntry) {
        self.entries.push(entry);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Describes whether a patch is a migration or a non-migrating fix.
pub enum PatchKind {
    /// A patch that converts a save from one version to a newer one.
    Migration,
    /// A patch that repairs a save without changing its version.
    Fix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Metadata used to identify and plan patches.
pub struct PatchMetadata {
    /// Stable identifier for logs and debugging.
    pub id: &'static str,
    /// Whether the patch is a migration or a fix.
    pub kind: PatchKind,
    /// Source version (for migrations).
    pub from_version: Option<u16>,
    /// Destination version (for migrations).
    pub to_version: Option<u16>,
}

/// A patch that can be applied to a save buffer.
///
/// Implementations should be deterministic and only mutate the provided `SaveBinary`.
pub trait Patch: std::fmt::Debug + Send + Sync {
    /// Returns this patch's metadata.
    fn metadata(&self) -> PatchMetadata;

    /// Applies the patch.
    ///
    /// # Errors
    /// Returns an error if the save is invalid, too small, or cannot be patched safely.
    fn apply(&self, save: &mut SaveBinary, symbols: &SymbolDatabase) -> SaveResult<()>;

    /// Applies the patch, emitting structured log entries.
    ///
    /// By default, this delegates to [`Patch::apply`] and emits no logs.
    fn apply_with_log(
        &self,
        save: &mut SaveBinary,
        symbols: &SymbolDatabase,
        log: &mut dyn PatchLogSink,
    ) -> SaveResult<()> {
        let _ = log;
        self.apply(save, symbols)
    }
}

/// Resolves a sequence of migration patches required to reach `target_version`.
///
/// The plan is built by repeatedly finding a migration patch whose `from_version` matches the
/// current step and whose `to_version` is greater than `from_version`.
///
/// # Errors
/// Returns an error if the requested direction is unsupported or if a required step is missing.
pub fn resolve_migration_plan(
    migrations: &[&'static dyn Patch],
    current_version: u16,
    target_version: u16,
) -> SaveResult<Vec<&'static dyn Patch>> {
    if current_version == target_version {
        return Ok(Vec::new());
    }

    if current_version > target_version {
        return Err(SaveError::UnsupportedMigrationDirection {
            current_version,
            target_version,
        });
    }

    let mut plan: Vec<&'static dyn Patch> = Vec::new();
    let mut v = current_version;

    while v != target_version {
        let next = migrations.iter().find(|p| {
            let meta = p.metadata();
            meta.kind == PatchKind::Migration
                && meta.from_version == Some(v)
                && meta.to_version.is_some_and(|to| to > v)
        });

        let Some(patch) = next else {
            return Err(SaveError::MissingMigrationStep {
                from_version: v,
                target_version,
            });
        };

        let meta = patch.metadata();
        let to = meta.to_version.expect("validated above");
        plan.push(*patch);
        v = to;
    }

    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyPatch {
        meta: PatchMetadata,
    }

    impl Patch for DummyPatch {
        fn metadata(&self) -> PatchMetadata {
            self.meta
        }

        fn apply(&self, _save: &mut SaveBinary, _symbols: &SymbolDatabase) -> SaveResult<()> {
            Ok(())
        }
    }

    static FIX: DummyPatch = DummyPatch {
        meta: PatchMetadata {
            id: "fix",
            kind: PatchKind::Fix,
            from_version: None,
            to_version: None,
        },
    };
    static M7_TO_8: DummyPatch = DummyPatch {
        meta: PatchMetadata {
            id: "m7_to_8",
            kind: PatchKind::Migration,
            from_version: Some(7),
            to_version: Some(8),
        },
    };
    static M8_TO_9: DummyPatch = DummyPatch {
        meta: PatchMetadata {
            id: "m8_to_9",
            kind: PatchKind::Migration,
            from_version: Some(8),
            to_version: Some(9),
        },
    };
    static M9_TO_10: DummyPatch = DummyPatch {
        meta: PatchMetadata {
            id: "m9_to_10",
            kind: PatchKind::Migration,
            from_version: Some(9),
            to_version: Some(10),
        },
    };

    #[test]
    fn resolve_plan_returns_empty_when_already_at_target() {
        let migrations: [&'static dyn Patch; 0] = [];
        let plan = resolve_migration_plan(&migrations, 9, 9).unwrap();
        assert!(plan.is_empty());
    }

    #[test]
    fn resolve_plan_errors_when_direction_is_unsupported() {
        let migrations: [&'static dyn Patch; 0] = [];
        let err = resolve_migration_plan(&migrations, 10, 9).unwrap_err();
        match err {
            SaveError::UnsupportedMigrationDirection {
                current_version,
                target_version,
            } => {
                assert_eq!(current_version, 10);
                assert_eq!(target_version, 9);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn resolve_plan_errors_when_step_is_missing() {
        let migrations: [&'static dyn Patch; 1] = [&M7_TO_8];
        let err = resolve_migration_plan(&migrations, 7, 9).unwrap_err();
        match err {
            SaveError::MissingMigrationStep {
                from_version,
                target_version,
            } => {
                assert_eq!(from_version, 8);
                assert_eq!(target_version, 9);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn resolve_plan_returns_sequential_chain_and_ignores_fix_patches() {
        let migrations: [&'static dyn Patch; 4] = [&FIX, &M7_TO_8, &M8_TO_9, &M9_TO_10];
        let plan = resolve_migration_plan(&migrations, 7, 10).unwrap();
        let ids: Vec<&'static str> = plan.iter().map(|p| p.metadata().id).collect();
        assert_eq!(ids, vec!["m7_to_8", "m8_to_9", "m9_to_10"]);
    }
}
