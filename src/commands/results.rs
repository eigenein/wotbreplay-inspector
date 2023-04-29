use std::fs::File;
use std::io::{stdout, Write};

use wotbreplay_parser::models::battle_results::BattleResults;
use wotbreplay_parser::replay::Replay;

use crate::inspect::inspect;
use crate::options::BattleResultsOptions;
use crate::prelude::*;

pub fn handle(options: BattleResultsOptions) -> Result {
    let mut replay =
        Replay::open(File::open(options.path).context("failed to open the replay file")?)
            .context("failed to open the replay archive")?;
    let battle_results_dat = replay.read_battle_results_dat()?;
    let buffer = battle_results_dat.buffer.as_ref();

    if options.raw {
        let message = inspect(buffer)?;
        let output =
            serde_json::to_string_pretty(&message).context("failed to serialize the output")?;
        writeln!(stdout(), "{output}")?;
    } else {
        let message = BattleResults::from_buffer(buffer)?;
        writeln!(stdout(), "{}", serde_json::to_string_pretty(&message)?)?;
    }
    Ok(())
}
