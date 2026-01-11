mod v1_to_v2;
mod v2_to_v3;

use gb_save_core::Patch;

pub use v1_to_v2::MIGRATE_V1_TO_V2;
pub use v2_to_v3::MIGRATE_V2_TO_V3;

/// Returns all migration patches for this example game.
#[must_use]
pub fn example_migrations() -> Vec<&'static dyn Patch> {
    vec![&MIGRATE_V1_TO_V2, &MIGRATE_V2_TO_V3]
}
