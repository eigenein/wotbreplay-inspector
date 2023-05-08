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

mod commands;
mod inspect;
mod options;
mod prelude;

use clap::Parser;

use crate::options::{Command, Options};
use crate::prelude::*;

fn main() -> Result {
    match Options::parse().command {
        Command::BattleResults(options) => commands::results::handle(options),
        Command::DumpData(options) => commands::dump_data::handle(options),
    }
}
