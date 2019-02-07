use crate::bgg_api;
use crate::bgg_api::Game;
use failure::{Error, ResultExt, ensure};
use std::path::Path;
use std::fs;
use serde_json::{from_str, to_string_pretty};
use serde_derive::{Serialize, Deserialize};

const TOP_FILE_NAME: &str = "top.json";
const IGNORE_FILE_NAME: &str = "ignore.json";
const CONFIG_FILE_NAME: &str = "app.config";

pub fn create_project(name: &str) -> Result<(), Error> {
    let dir_path = Path::new(name);
    fs::create_dir(dir_path)?;
    fs::write(dir_path.join(IGNORE_FILE_NAME), "[]")?;
    fs::write(dir_path.join(TOP_FILE_NAME), "[]")?;
    let new_conf = to_string_pretty(&Config::new(100, 5))?;
    fs::write(dir_path.join(CONFIG_FILE_NAME), new_conf)?;
    Ok(())
}

pub fn get_top(depth: usize) -> Result<(), Error> {
    let config = bgg_api::Config::new(1000);
    let api = bgg_api::API::new(config);
    // Collect games
    let mut game_list = Vec::new();
    for variable in api.get_top(depth) {
        // Error will be elevated and next() will be never called again
        let (mut games_on_page, i, num_pages) = variable?;
        // TODO: remove side effect Progress.report()
        println!("Downloading page {} of {}", i, num_pages);
        game_list.append(&mut games_on_page);
    }

    let game_list = &game_list[..depth]; // crop to real depth
    let serialized = to_string_pretty(game_list)?;
    fs::write(TOP_FILE_NAME, serialized)?;
    Ok(())
}

pub fn get_slice(from: usize, to: usize, verbose: bool) -> Result<Vec<(usize, Container)>, Error> {
    ensure!(from > 0, "number cannot be smaller than 1!");
    ensure!(to > from, "choose slice limits properly!");
    // NOTE: Later move to structopt ?
    let games = fs::read_to_string(TOP_FILE_NAME)
        .with_context(|_| format!("Can't open: {}", TOP_FILE_NAME))?;
    let ignored = fs::read_to_string(IGNORE_FILE_NAME)
        .with_context(|_| format!("Can't open: {}", IGNORE_FILE_NAME))?;
    let games: Vec<Game> = from_str(&games)?;
    let ignored: Vec<Game> = from_str(&ignored)?;
    let slice: Vec<(usize, Container)> = mark_games(games, ignored)
        .into_iter()
        .enumerate()
        .filter(|(_, gc)| verbose || gc.ignored == false)
        .skip(from - 1)
        .take(to - from + 1)
        .collect();
    Ok(slice)
}

fn mark_games(games :Vec<Game>, ignored: Vec<Game>) -> Vec<Container> {
    let mut containers = Vec::new();
    for game in games {
        let ign = ignored.contains(&game);
        let container = Container::new(game, ign);
        containers.push(container);
    }
    containers
}

pub fn config() -> Result<Config, Error> {
    let conf = fs::read_to_string(CONFIG_FILE_NAME)
        .with_context(|_| format!("Can't open: {}", CONFIG_FILE_NAME))?;
    let conf = from_str(&conf)?;
    Ok(conf)
}

pub fn ignore(game: Game) -> Result<(), Error> {
    let ignored = fs::read_to_string(IGNORE_FILE_NAME)
        .with_context(|_| format!("Can't open: {}", IGNORE_FILE_NAME))?;
    let mut ignored: Vec<Game> = from_str(&ignored)?;
    ignored.push(game);
    let serialized = to_string_pretty(&ignored)?;
    fs::write(IGNORE_FILE_NAME, serialized)?;
    Ok(())
}

#[derive(Debug)]
pub struct Container {
    pub game: Game,
    pub ignored: bool
}

impl Container {
    fn new(game: Game, ignored: bool) -> Container {
        Container { game, ignored }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub depth: usize,
    pub batch_size: usize
}

impl Config {
    fn new(depth: usize, batch_size: usize) -> Config {
        Config {depth, batch_size }
    }
}
