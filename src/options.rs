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
    /// Dump `battle_results.dat` contents as JSON.
    BattleResults(BattleResultsOptions),

    /// Dump `data.wotreplay` packets as JSON lines.
    DumpData(DumpDataOptions),
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
pub struct DumpDataOptions {
    #[arg(value_name = "WOTBREPLAY_PATH")]
    pub path: PathBuf,
}
