use std::fs::File;
use std::path::PathBuf;

use average::Mean;
use notify::event::{ModifyKind, RenameMode};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use prost::Message;
use sled::Tree;
use wotbreplay_parser::prelude::{BattleResults, BattleResultsDat, Player};

use crate::options::WatchOptions;
use crate::prelude::*;

pub struct WatchCommand {
    results_path: PathBuf,
    ratings: Tree,
    test_path: Option<PathBuf>,
}

impl WatchCommand {
    pub fn new(options: WatchOptions) -> Result<Self> {
        let db = sled::open(options.database_path).context("failed to open the database")?;
        println!("Rated users: {}", db.len());
        Ok(Self {
            results_path: options.results_path,
            ratings: db.open_tree("ratings")?,
            test_path: options.test_path,
        })
    }

    pub fn handle(&self) -> Result {
        if let Some(test_path) = &self.test_path {
            self.handle_result(test_path)?;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
        watcher.watch(&self.results_path, RecursiveMode::NonRecursive)?;

        for result in rx {
            let event = result?;
            if event.kind != EventKind::Modify(ModifyKind::Name(RenameMode::Any)) {
                continue;
            }
            for path in event.paths {
                if path
                    .to_str()
                    .map_or(false, |path| path.ends_with("_full.dat"))
                {
                    if let Err(error) = self.handle_result(&path) {
                        eprintln!("{error:#}");
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_result(&self, path: &PathBuf) -> Result {
        eprintln!("Parsing {:?}…", path.file_name());
        let battle_results = BattleResultsDat::from_reader(File::open(path)?)?;
        let arena_unique_id = battle_results.arena_unique_id;
        let mut battle_results: BattleResults = battle_results.try_into()?;

        println!("Arena ID: {}", arena_unique_id);
        println!("Your team: #{}", battle_results.author.team_number);
        println!("Winner team: #{}", battle_results.winning_team);
        println!("Win: {}", battle_results.winning_team == battle_results.author.team_number);

        let (mut team_1, mut team_2) = {
            let mut players_1 = Vec::new();
            let mut players_2 = Vec::new();
            for player in battle_results.players.drain(..) {
                let rating = RatingModel::get(&self.ratings, player.account_id)?;
                match player.info.team {
                    1 => players_1.push((player, rating)),
                    2 => players_2.push((player, rating)),
                    team => panic!("{team}"),
                }
            }
            (Team(players_1), Team(players_2))
        };

        let team_rating_1 = team_1.calculate_rating();
        println!("Team #1 rating: {team_rating_1:.6}");
        let team_rating_2 = team_2.calculate_rating();
        println!("Team #2 rating: {team_rating_2:.6}");

        let (actual_1, actual_2) = match battle_results.winning_team {
            1 => (1.0, 0.0),
            2 => (0.0, 1.0),
            _ => (0.5, 0.5), // Draw?
        };

        let expectation_1 = Self::calculate_expectation(team_rating_1, team_rating_2);
        println!("Team #1 expectation: {expectation_1:.6}");
        let expectation_2 = Self::calculate_expectation(team_rating_2, team_rating_1);
        println!("Team #2 expectation: {expectation_2:.6}");

        println!("Team #1 updates:");
        let n_new_players = team_1.update_ratings(
            &self.ratings,
            actual_1 - expectation_1,
            self.test_path.is_some(),
        )?;
        println!("Team #2 updates:");
        let n_new_players = n_new_players
            + team_2.update_ratings(
                &self.ratings,
                actual_2 - expectation_2,
                self.test_path.is_some(),
            )?;

        println!("Done {arena_unique_id}, new players: {n_new_players}");
        Ok(())
    }

    fn calculate_expectation(ally_rating: f64, enemy_rating: f64) -> f64 {
        1.0 / (1.0 + (enemy_rating - ally_rating).exp())
    }
}

struct Team(pub Vec<(Player, RatingModel)>);

impl Team {
    pub fn calculate_rating(&self) -> f64 {
        self.0
            .iter()
            .map(|(_, rating)| rating.rating)
            .collect::<Mean>()
            .mean()
    }

    pub fn update_ratings(
        &mut self,
        tree: &Tree,
        rating_offset: f64,
        dry_run: bool,
    ) -> Result<usize> {
        let mut n_new_players = 0;
        for (player, rating) in self.0.iter_mut() {
            if rating.update_rating(tree, player, rating_offset, dry_run)? {
                n_new_players += 1;
            }
        }
        Ok(n_new_players)
    }
}

#[derive(Message)]
struct RatingModel {
    #[prost(uint32, tag = "1")]
    pub n_battles: u32,

    #[prost(double, tag = "2")]
    pub rating: f64,
}

impl RatingModel {
    pub fn get(tree: &Tree, account_id: u32) -> Result<Self> {
        let this = tree
            .get(account_id.to_be_bytes())?
            .map(|value| Self::decode(value.as_ref()))
            .transpose()?
            .unwrap_or_default();
        Ok(this)
    }

    pub fn update(&self, tree: &Tree, account_id: u32) -> Result<bool> {
        let last_value = tree.insert(account_id.to_be_bytes(), self.encode_to_vec())?;
        Ok(last_value.is_none())
    }

    pub fn update_rating(
        &mut self,
        tree: &Tree,
        player: &Player,
        rating_offset: f64,
        dry_run: bool,
    ) -> Result<bool> {
        /// Learning speed indexed by the number of battles.
        const K: [f64; 10] = [0.5, 0.25, 0.2, 0.15, 0.125, 0.1, 0.075, 0.05, 0.025, 0.02];

        let k = K.get(self.n_battles as usize).copied().unwrap_or(0.01);
        self.n_battles += 1;
        let last_rating = self.rating;
        self.rating += k * rating_offset;
        println!("{}: {last_rating:.6} -> {:.6} (k={k:.4})", player.info.nickname, self.rating);

        if !dry_run {
            self.update(tree, player.account_id)
        } else {
            Ok(false)
        }
    }
}
