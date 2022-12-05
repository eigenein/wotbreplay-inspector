use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about)]
pub struct Options {
    #[command(subcommand)]
    pub command: Command,

    #[arg(value_name = "WOTBREPLAY_PATH")]
    pub path: PathBuf,
}

#[derive(Subcommand)]
pub enum Command {
    /// Inspect `battle_results.dat` and dump its contents as JSON.
    BattleResults {
        /// Dump the entire structure and do not try to match any tags.
        #[arg(long)]
        raw: bool,
    },
}
