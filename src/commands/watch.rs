use std::fs::File;
use std::path::PathBuf;

use average::Mean;
use notify::event::ModifyKind::Name;
use notify::event::RenameMode;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sled::Db;
use wotbreplay_parser::prelude::{BattleResults, BattleResultsDat, Player};

use crate::options::WatchOptions;
use crate::prelude::*;

pub struct WatchCommand {
    results_path: PathBuf,
    db: Db,
    test_path: Option<PathBuf>,
}

impl WatchCommand {
    pub fn new(options: WatchOptions) -> Result<Self> {
        let db = sled::open(options.database_path).context("failed to open the database")?;
        println!("Rated users: {}", db.len());
        Ok(Self {
            results_path: options.results_path,
            db,
            test_path: options.test_path,
        })
    }

    pub fn handle(&self) -> Result {
        if let Some(test_path) = &self.test_path {
            self.handle_battle_results(test_path)?;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
        watcher.watch(&self.results_path, RecursiveMode::NonRecursive)?;

        for result in rx {
            let event = result?;
            if event.kind != EventKind::Modify(Name(RenameMode::Any)) {
                continue;
            }
            for path in event.paths {
                {
                    let Some(path) = path.to_str() else { continue };
                    if !path.ends_with("_full.dat") {
                        continue;
                    }
                }
                if let Err(error) = self.handle_battle_results(&path) {
                    eprintln!("{error:#}");
                }
            }
        }

        Ok(())
    }

    fn handle_battle_results(&self, path: &PathBuf) -> Result {
        eprintln!("Parsing {:?}…", path.file_name());
        let battle_results: BattleResults = BattleResultsDat::from_reader(File::open(path)?)
            .context("failed to decode the battle results")?
            .try_into()?;

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

        let n_new_players =
            self.update_ratings(&battle_results.players, team_update_1, team_update_2)?;

        println!("Done {:?}, new players: {n_new_players}", path.file_name());
        Ok(())
    }

    fn update_ratings(
        &self,
        players: &[Player],
        team_update_1: f64,
        team_update_2: f64,
    ) -> Result<usize> {
        let mut n_new_players = 0;
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
                if self.set_player_rating(player.account_id, updated_rating)? {
                    n_new_players += 1;
                }
            }
        }
        Ok(n_new_players)
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

    fn set_player_rating(&self, account_id: u32, rating: f64) -> Result<bool> {
        let is_new = self
            .db
            .insert(account_id.to_be_bytes(), &rating.to_be_bytes())?
            .is_none();
        Ok(is_new)
    }
}
