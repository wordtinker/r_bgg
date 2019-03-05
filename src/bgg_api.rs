use failure::{bail, Error, ResultExt};
use reqwest;
use select::document::Document;
use select::predicate::{Class, Name};
use serde_derive::{Deserialize, Serialize};
use std::{cmp, thread, time};

const PAGE_SIZE: usize = 100;

struct TopIterator {
    delay: u64,
    num_pages: usize,
    count: usize,
}

impl TopIterator {
    fn new(delay: u64, depth: usize) -> TopIterator {
        let num_pages = (depth as f64 / PAGE_SIZE as f64).ceil() as usize;
        TopIterator {
            delay,
            num_pages,
            count: 0,
        }
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
        match API::get_games_from(self.count) {
            Ok(games) => Some(Ok((games, self.count, self.num_pages))),
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct Config {
    // delay between rrquests in milliseconds
    delay: u64,
}

impl Config {
    pub fn new(delay: u64) -> Config {
        Config { delay }
    }
}

pub struct API {
    config: Config,
}

impl API {
    pub fn new(config: Config) -> API {
        API { config }
    }

    pub fn get_top(
        &self,
        depth: usize,
    ) -> impl Iterator<Item = Result<(Vec<Game>, usize, usize), Error>> {
        TopIterator::new(self.config.delay, depth)
    }

    pub fn get_next(depth: usize, offset: usize) -> Result<Vec<Game>, Error> {
        let pages = (depth as f64 / PAGE_SIZE as f64).ceil() as usize;
        let last_page = ((depth + offset) as f64 / PAGE_SIZE as f64).ceil() as usize;
        let start_page = cmp::min(pages + 1, last_page);

        let mut games: Vec<Game> = Vec::new();
        for n in start_page..=last_page {
            games.append(&mut API::get_games_from(n)?);
        }
        let skip = if (start_page - 1) * PAGE_SIZE > depth {
            0
        } else {
            depth - (start_page - 1) * PAGE_SIZE
        };
        Ok(games.into_iter().skip(skip).take(offset).collect())
    }

    fn get_games_from(n: usize) -> Result<Vec<Game>, Error> {
        let url = format!("https://boardgamegeek.com/browse/boardgame/page/{}", n);
        let resp =
            reqwest::get(&url).with_context(|_| format!("could not download page `{}`", url))?;
        let doc = Document::from_read(resp)?;
        API::filter_games(doc)
    }

    fn filter_games(doc: Document) -> Result<Vec<Game>, Error> {
        let links = doc
            .find(Class("collection_table"))
            .flat_map(|t| t.find(Class("collection_objectname")))
            .flat_map(|c| c.find(Name("div")))
            .flat_map(|div| div.find(Name("a")));

        let mut games = Vec::new();
        for link in links {
            match link.attr("href") {
                Some(href) => {
                    let id = API::href_to_id(href)?;
                    let year = match link.parent() {
                        Some(parent) => match parent.find(Name("span")).next() {
                            Some(span) => span.text(),
                            // assume something very old, at least 0 C.E.
                            _ => String::from("(0)"),
                        },
                        _ => bail!("Could not find game year: {}", href),
                    };
                    let year = &year[1..year.len() - 1];
                    let year = year
                        .parse::<isize>()
                        .with_context(|_| format!("Can't parse {}", year))?;
                    games.push(Game::new(id, link.text(), year));
                }
                _ => bail!("Could not find game id."),
            };
        }
        Ok(games)
    }

    fn href_to_id(href: &str) -> Result<usize, Error> {
        let parts: Vec<&str> = href.rsplit('/').take(2).collect();
        let id = match parts.get(1) {
            Some(x) => x.parse::<usize>()?,
            None => bail!("Can't parse id of the game: {}", href),
        };
        Ok(id)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Game {
    pub id: usize,
    pub name: String,
    pub year: isize,
}

impl Game {
    fn new(id: usize, name: String, year: isize) -> Game {
        Game { id, name, year }
    }
    pub fn url(&self) -> String {
        format!("https://boardgamegeek.com/boardgame/{}", self.id)
    }
}
