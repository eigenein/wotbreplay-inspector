#![warn(
    clippy::all,
    clippy::explicit_into_iter_loop,
    clippy::manual_let_else,
    clippy::map_unwrap_or,
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self
)]

mod inspect;
mod options;
mod prelude;

use std::fs::File;
use std::io::{stdout, Write};

use clap::Parser;
use wotbreplay_parser::models::BattleResults;
use wotbreplay_parser::Replay;

use crate::inspect::inspect;
use crate::options::{Command, Options};
use crate::prelude::*;

fn main() -> Result {
    let options = Options::parse();
    let mut replay = Replay::open(File::open(options.path)?)?;

    match options.command {
        Command::BattleResults { raw } => {
            let battle_results_dat = replay.read_battle_results_dat()?;
            if raw {
                let message = inspect(battle_results_dat.1.as_ref())?;
                let _ = writeln!(stdout(), "{}", toml::to_string(&message)?);
            } else {
                let message = BattleResults::parse(battle_results_dat.1.as_ref())?;
                let _ = writeln!(stdout(), "{}", serde_json::to_string_pretty(&message)?);
            }
        }
    }

    Ok(())
}
