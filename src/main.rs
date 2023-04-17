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

use anyhow::Context;
use clap::Parser;
use notify::event::DataChange::Content;
use notify::event::ModifyKind::Data;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use wotbreplay_parser::prelude::{BattleResults, Replay};

use crate::inspect::inspect;
use crate::options::{Command, Options};
use crate::prelude::*;

fn main() -> Result {
    let options = Options::parse();

    match options.command {
        Command::BattleResults { raw, path } => {
            let mut replay =
                Replay::open(File::open(path).context("failed to open the replay file")?)
                    .context("failed to open the replay archive")?;
            let battle_results_dat = replay.read_battle_results_dat()?;
            if raw {
                let message = inspect(battle_results_dat.1.as_ref())?;
                let output = serde_json::to_string_pretty(&message)
                    .context("failed to serialize the output")?;
                let _ = writeln!(stdout(), "{output}");
            } else {
                let message = BattleResults::from_buffer(battle_results_dat.1.as_ref())?;
                let _ = writeln!(stdout(), "{}", serde_json::to_string_pretty(&message)?);
            }
        }

        Command::Watch { path } => {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
            watcher.watch(&path, RecursiveMode::NonRecursive)?;

            for result in rx {
                let event = result?;
                if event.kind != EventKind::Modify(Data(Content)) {
                    continue;
                }
                for path in event.paths {
                    eprintln!("parsing {:?}…", path.file_name());
                    let mut replay = Replay::open(File::open(path)?)?;
                    let Ok(battle_results_dat) = replay.read_battle_results_dat() else {
                        eprintln!("failed to parse the replay");
                        continue;
                    };
                    let battle_result = battle_results_dat.decode_battle_results()?;
                    println!(
                        "team = {:?}, win = {:?}, winning_team = {}",
                        battle_result.author.team_number(),
                        battle_result.winning_team == battle_result.author.team_number,
                        battle_result.winning_team,
                    );
                }
            }
        }
    }

    Ok(())
}
