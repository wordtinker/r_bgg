mod cli;
mod bgg_api;
mod core;

use cli::Cli;
use failure::{Error, ResultExt};
use exitfailure::ExitFailure;
use prettytable::{Table, row, cell};
use std::io;
use std::process::Command;
use indicatif::ProgressBar;

fn main() -> Result<(), ExitFailure> {
    let cli = cli::from_args();
    match cli {
        Cli::Create { name } => create_project(&name)?,
        Cli::Get { depth } => get_top(depth)?,
        Cli::Top { depth, verbose } => show_slice(1, depth, verbose)?,
        Cli::Slice { from, to , verbose } => show_slice(from, to, verbose)?,
        Cli::Run { review } => run_routine(review)?,
        Cli::Prospect { } => see_future()?
    }
    Ok(())
}

fn create_project(name: &str) -> Result<(), Error> {
    core::create_project(&name)?;
    println!("Successfully created {} project", name);
    Ok(())
}

fn get_top(depth: usize) -> Result<(), Error> {
    println!("Starting download.");
    let bar = ProgressBar::new(1000);
    core::get_top(depth, |i,n| {
        bar.set_length(n as u64);
        bar.set_position(i as u64);
    })?;
    bar.finish();
    println!("Finished download.");
    Ok(())
}

fn show_slice(from: usize, to: usize, verbose: bool) -> Result<(), Error> {
    let slice = core::get_slice(from, to, verbose)?;
    print_table(&slice);
    Ok(())
}

fn see_future() -> Result<(), Error> {
    // 1. get config params
    let config = core::config()?;
    let (seen, new) = core::get_future(config.depth, config.prospect)?;
    println!("Found {} seen games and {} new games in the future.", seen, new);
    Ok(())
}

fn run_routine(review: bool) -> Result<(), Error> {
    // 1. get config params
    let config = core::config()?;

    if !review {
        // 1. download top
        get_top(config.depth)?;
    }

    loop {
        // 2. get top 5 new games from file
        let slice = core::get_slice(1, config.batch_size, false)?;
        if slice.len() == 0 { break; }
        println!("Found new games");
        print_table(&slice);
        println!("Do you want to open links: y/n?");

        let input = read_input()?;
        if input != String::from("y") { break; }

        for (_, container) in slice {
            // 3. Open link in a browser
            open_browser(&container.game.url())?;
            // 4. Add game to ignore file
            core::ignore(container.game)?;
        }
    }
    println!("Finished.");
    Ok(())
}

fn print_table(top: &Vec<(usize, core::Container)>) {
    // Create the table
    let mut table = Table::new();

    for (pos, container) in top.iter() {
        let row = if container.ignored {
            // a bit ugly due to fprmat of color specifiers
            row![FY => pos + 1, container.game.id, container.game.name]
        } else {
            row![FW => pos + 1, container.game.id, container.game.name]
        };
        table.add_row(row);
    }
    // Print the table to stdout
    table.printstd();
}

fn read_input() -> Result<String, Error> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    // NOTE: win only solution
    input.truncate(input.len() - 2);
    Ok(input)
}

fn open_browser(url: &str) -> Result<(), Error> {
    // NOTE: win only solution
    Command::new("cmd.exe")
        .arg("/C").arg("start")
        .arg("").arg(url)
        .spawn()
        .with_context(|_| "failed to launch browser")?;
        Ok(())
}
