use std::fs::File;
use std::io::{stdout, Write};

use wotbreplay_parser::replay::Replay;

use crate::options::DumpDataOptions;
use crate::prelude::Result;

pub fn handle(options: DumpDataOptions) -> Result {
    let mut replay = Replay::open(File::open(options.path)?)?;
    let data = replay.read_data()?;
    writeln!(stdout(), "{}", serde_json::to_string_pretty(&data)?)?;
    Ok(())
}
