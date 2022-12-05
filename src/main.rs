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

use crate::inspect::DynamicMessage;
use crate::options::{Command, Options};
use crate::prelude::*;

fn main() -> Result {
    let options = Options::parse();
    let mut replay = Replay::open(File::open(options.path)?)?;

    match options.command {
        Command::BattleResults { raw } => {
            let battle_results_dat = replay.read_battle_results_dat()?;
            let buffer = &mut battle_results_dat.1.as_ref();
            let dump = if raw {
                let message = DynamicMessage::decode(buffer)?;
                serde_json::to_string_pretty(&message)?
            } else {
                let message = BattleResults::parse(buffer)?;
                serde_json::to_string_pretty(&message)?
            };
            let _ = writeln!(stdout(), "{}", dump);
        }
    }

    Ok(())
}
