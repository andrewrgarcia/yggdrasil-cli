mod args;
mod types;
mod snapshot;
mod scanner;
mod formatters;
mod diff;

use clap::Parser;
use clap::CommandFactory;
use args::{Cli, Commands};
use snapshot::run_snapshot;
use diff::run_diff;

fn main() {
    let cli = Cli::parse();

    if std::env::args().len() == 1 {
        Cli::command().print_help().unwrap();
        println!();
        return;
    }

    match cli.command {
        Some(Commands::Diff { from, to, align_tags }) => {
            run_diff(from, to, align_tags);
        }

        None => {
            let args = cli.args;
            run_snapshot(args);
        }
    }
}

