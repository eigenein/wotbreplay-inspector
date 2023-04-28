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
    BattleResults(BattleResultsOptions),

    /// Watch the replays directory and print some battle results (experimental, in progress).
    Watch(WatchOptions),
}

#[derive(Parser)]
pub struct BattleResultsOptions {
    #[arg(value_name = "WOTBREPLAY_PATH")]
    pub path: PathBuf,

    /// Dump the entire structure and do not try to match any tags.
    #[arg(long)]
    pub raw: bool,
}

#[derive(Parser)]
pub struct WatchOptions {
    #[arg(short = 'r', long, value_name = "RESULTS_DIRECTORY")]
    pub results_path: PathBuf,

    #[arg(short = 'd', long, value_name = "DATABASE_DIRECTORY")]
    pub database_path: PathBuf,

    #[arg(short = 't', long)]
    pub test_path: Option<PathBuf>,
}
