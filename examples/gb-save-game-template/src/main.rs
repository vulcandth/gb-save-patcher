#![forbid(unsafe_code)]

use anyhow::Result;

struct ExampleGameCli;

impl gb_save_cli::GameCli for ExampleGameCli {
    fn detect_version(bytes: &[u8]) -> Result<u16> {
        gb_save_game_template::detect_version(bytes)
    }

    fn patch(bytes: Vec<u8>, target: u16, dev_type: u8) -> Result<Vec<u8>> {
        gb_save_game_template::patch(bytes, target, dev_type)
    }

    fn patch_with_log(bytes: Vec<u8>, target: u16, dev_type: u8) -> gb_save_cli::PatchOutcome {
        gb_save_game_template::patch_with_log(bytes, target, dev_type)
    }
}

fn main() -> Result<()> {
    gb_save_cli::run::<ExampleGameCli>()
}
