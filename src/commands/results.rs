use std::fs::File;
use std::io::{stdout, Write};

use wotbreplay_parser::prelude::{BattleResults, Replay};

use crate::inspect::inspect;
use crate::options::BattleResultsOptions;
use crate::prelude::*;

pub fn handle(options: BattleResultsOptions) -> Result {
    let mut replay =
        Replay::open(File::open(options.path).context("failed to open the replay file")?)
            .context("failed to open the replay archive")?;
    let battle_results_dat = replay.read_battle_results_dat()?;
    if options.raw {
        let message = inspect(battle_results_dat.1.as_ref())?;
        let output =
            serde_json::to_string_pretty(&message).context("failed to serialize the output")?;
        let _ = writeln!(stdout(), "{output}");
    } else {
        let message = BattleResults::from_buffer(battle_results_dat.1.as_ref())?;
        let _ = writeln!(stdout(), "{}", serde_json::to_string_pretty(&message)?);
    }
    Ok(())
}
