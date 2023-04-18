use std::fs::File;
use std::path::PathBuf;

use average::Mean;
use notify::event::DataChange::Content;
use notify::event::ModifyKind::Data;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sled::Db;
use wotbreplay_parser::prelude::{Player, Replay};

use crate::options::WatchOptions;
use crate::prelude::*;

pub struct WatchCommand {
    replays_path: PathBuf,
    db: Db,
    test_path: Option<PathBuf>,
}

impl WatchCommand {
    pub fn new(options: WatchOptions) -> Result<Self> {
        Ok(Self {
            replays_path: options.replays_path,
            db: sled::open(options.database_path).context("failed to open the database")?,
            test_path: options.test_path,
        })
    }

    pub fn handle(&self) -> Result {
        if let Some(test_path) = &self.test_path {
            self.handle_replay(test_path)?;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
        watcher.watch(&self.replays_path, RecursiveMode::NonRecursive)?;

        for result in rx {
            let event = result?;
            if event.kind != EventKind::Modify(Data(Content)) {
                continue;
            }
            for path in event.paths {
                self.handle_replay(&path)?;
            }
        }

        Ok(())
    }

    fn handle_replay(&self, path: &PathBuf) -> Result {
        eprintln!("Parsing {:?}…", path.file_name());
        let mut replay = Replay::open(File::open(path)?)?;
        let battle_results_dat = match replay.read_battle_results_dat() {
            Ok(battle_results_dat) => battle_results_dat,
            Err(error) => {
                eprintln!("failed to parse the replay: {:#}", error);
                return Ok(());
            }
        };

        let battle_results = battle_results_dat
            .decode_battle_results()
            .context("failed to decode the battle results")?;
        println!("Your team: #{}", battle_results.author.team_number);
        println!("Winning team: #{}", battle_results.winning_team);
        println!("Win: {}", battle_results.winning_team == battle_results.author.team_number);

        let team_rating_1 = self.calculate_team_rating(&battle_results.players, 1)?;
        println!("Team #1 rating: {team_rating_1:.6}");
        let team_rating_2 = self.calculate_team_rating(&battle_results.players, 2)?;
        println!("Team #2 rating: {team_rating_2:.6}");

        let (actual_1, actual_2) = match battle_results.winning_team {
            1 => (1.0, 0.0),
            2 => (0.0, 1.0),
            _ => (0.5, 0.5), // Draw?
        };

        const K: f64 = 0.02;
        let team_update_1 = {
            let expected_result = 1.0 / (1.0 + (team_rating_2 - team_rating_1).exp());
            println!("Team #1 expectation: {expected_result:.6}");
            K * (actual_1 - expected_result)
        };
        println!("Team #1 update: {team_update_1:.6}");
        let team_update_2 = {
            let expected_result = 1.0 / (1.0 + (team_rating_1 - team_rating_2).exp());
            println!("Team #2 expectation: {expected_result:.6}");
            K * (actual_2 - expected_result)
        };
        println!("Team #2 update: {team_update_2:.6}");

        self.update_ratings(&battle_results.players, team_update_1, team_update_2)?;

        Ok(())
    }

    fn update_ratings(&self, players: &[Player], team_update_1: f64, team_update_2: f64) -> Result {
        for player in players {
            let prior_rating = self.get_player_rating(player.account_id)?;
            let updated_rating = prior_rating
                + match player.info.team {
                    1 => team_update_1,
                    2 => team_update_2,
                    _ => unreachable!(),
                };
            println!(
                "[{}] {}: {prior_rating:.6} -> {updated_rating:.6}",
                player.info.team, player.info.nickname,
            );
            if self.test_path.is_none() {
                self.set_player_rating(player.account_id, updated_rating)?;
            }
        }
        Ok(())
    }

    fn calculate_team_rating(&self, players: &[Player], team_number: i32) -> Result<f64> {
        let mean = players
            .iter()
            .filter(|player| player.info.team == team_number)
            .map(|player| self.get_player_rating(player.account_id))
            .collect::<Result<Mean>>()?;
        Ok(mean.mean())
    }

    fn get_player_rating(&self, account_id: u32) -> Result<f64> {
        let Some(value) = self.db.get(account_id.to_be_bytes())? else {
        return Ok(0.0);
    };
        Ok(f64::from_be_bytes(value.as_ref().try_into()?))
    }

    fn set_player_rating(&self, account_id: u32, rating: f64) -> Result {
        self.db
            .insert(account_id.to_be_bytes(), &rating.to_be_bytes())?;
        Ok(())
    }
}
