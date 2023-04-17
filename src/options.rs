use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about)]
pub struct Options {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Inspect `battle_results.dat` and dump its contents as JSON.
    BattleResults {
        #[arg(value_name = "WOTBREPLAY_PATH")]
        path: PathBuf,

        /// Dump the entire structure and do not try to match any tags.
        #[arg(long)]
        raw: bool,
    },

    /// Watch the replays directory and print some battle results (experimental, in progress).
    Watch {
        #[arg(value_name = "WOTBREPLAYS_DIRECTORY")]
        path: PathBuf,
    },
}
