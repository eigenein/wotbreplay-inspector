#![warn(
    clippy::all,
    clippy::missing_const_for_fn,
    clippy::trivially_copy_pass_by_ref,
    clippy::map_unwrap_or,
    clippy::explicit_into_iter_loop,
    clippy::unused_self,
    clippy::needless_pass_by_value
)]

mod inspect;
mod options;
mod prelude;

use std::fs::File;
use std::io::{stdout, Write};

use clap::Parser;
use wotbreplay_parser::models::BattleResults;
use wotbreplay_parser::Replay;

use crate::inspect::{inspect, Inspector};
use crate::options::{Command, Options};
use crate::prelude::*;

fn main() -> Result {
    let options = Options::parse();
    let mut replay = Replay::open(File::open(options.path)?)?;

    match options.command {
        Command::BattleResults { raw } => {
            let battle_results_dat = replay.read_battle_results_dat()?;
            if raw {
                inspect(battle_results_dat.1.as_ref(), &mut Inspector::default())?;
            } else {
                let message = BattleResults::parse(battle_results_dat.1.as_ref())?;
                let _ = writeln!(stdout(), "{}", serde_json::to_string_pretty(&message)?);
            }
        }
    }

    Ok(())
}
