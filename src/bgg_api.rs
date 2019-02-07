use std::{thread, time};
use reqwest;
use select::document::Document;
use select::predicate::{Name, Class};
use failure::{ResultExt, Error, bail};
use serde_derive::{Serialize, Deserialize};

struct TopIterator {
    delay: u64,
    num_pages: usize,
    count: usize
}

impl TopIterator {
    fn new(delay: u64, depth: usize) -> TopIterator {
        let page_size = 100.0;
        let num_pages = (depth as f32 / page_size).ceil() as usize;
        TopIterator {delay, num_pages, count: 0 }
    }

    fn get_games_from(n: usize) -> Result<Vec<Game>, Error> {
        let url = format!("https://boardgamegeek.com/browse/boardgame/page/{}", n);
        let resp = reqwest::get(&url)
            .with_context(|_| format!("could not download page `{}`", url))?;
        let doc = Document::from_read(resp)?;
        let games_on_page = API::filter_games(doc)?;
        Ok(games_on_page)
    }
}

impl Iterator for TopIterator {
    type Item = Result<(Vec<Game>, usize, usize), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        if self.count > self.num_pages {
            return None;
        }
        // wait before asking for result again
        thread::sleep(time::Duration::from_millis(self.delay));
        // get games
        match TopIterator::get_games_from(self.count) {
            Ok(games) => Some(Ok((games, self.count, self.num_pages))),
            Err(e) => Some(Err(e))
        }
    }
}

pub struct Config {
    /// delay between rrquests in milliseconds
    delay: u64
}

impl Config {
    pub fn new(delay: u64) -> Config {
        Config { delay }
    }
}

pub struct API {
    config: Config
}

impl API {
    pub fn new(config: Config) -> API {
        API { config }
    }

    pub fn get_top(&self, depth: usize) -> impl Iterator<Item=Result<(Vec<Game>, usize, usize), Error>> {
        TopIterator::new(self.config.delay, depth)
    }

    fn filter_games(doc: Document) -> Result<Vec<Game>, Error> {
        let links = doc
            .find(Class("collection_table"))
            .flat_map(|t| t.find(Class("collection_objectname")))
            .flat_map(|c| c.find(Name("div")))
            .flat_map(|div| div.find(Name("a")));

        let mut games = Vec::new();
        for link in links {
            let id = match link.attr("href") {
                Some(href) => API::href_to_id(href)?,
                _ => bail!("Could not find game id.")
            };
            games.push(Game::new(id, link.text()));
        }
        Ok(games)
    }

    fn href_to_id(href: &str) -> Result<usize, Error> {
        let parts: Vec<&str> = href.rsplit('/').take(2).collect();
        let id = match parts.get(1) {
            Some(x) => x.parse::<usize>()?,
            None => bail!("Can't parse id of the game: {}", href)
        };
        Ok(id)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Game {
    pub id: usize,
    pub name: String
}

impl Game {
    fn new(id: usize, name: String) -> Game {
        Game { id, name }
    }
    pub fn url(&self) -> String {
        format!("https://boardgamegeek.com/boardgame/{}", self.id)
    }
}
