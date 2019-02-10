use structopt::StructOpt;

pub fn from_args() -> Cli {
    Cli::from_args()
}

#[derive(Debug, StructOpt)]
/// Works with bgg top list.
pub enum Cli {
    #[structopt(name = "create")]
    /// Creates folder and structure files for work.
    Create {
        /// Name of the project folder.
        name: String
    },
    #[structopt(name = "get")]
    /// Get n positions from top of bgg.
    Get {
        /// depth of search.
        depth: usize
    },
    #[structopt(name ="top")]
    /// Show top n games.
    Top {
        /// size of top list.
        #[structopt(default_value = "20")]
        depth: usize,
        #[structopt(short = "v")]
        /// if set, ignored positions will be shown.
        verbose: bool
    },
    #[structopt(name = "slice")]
    /// Show games from position i to j, inclusive.
    Slice {
        /// left bound of slice.
        from: usize,
        /// right bound of slice.
        to: usize,
        #[structopt(short = "v")]
        /// if set, ignored positions will be shown.
        verbose: bool
    },
    #[structopt(name = "run")]
    /// Runs a routine using config file.
    Run {
        #[structopt(short = "r")]
        /// if set, review, no redownloading will be done
        review: bool
    },
    #[structopt(name = "prospect")]
    // Peeks next bunch of games using config file.
    Prospect { }
}
