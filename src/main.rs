use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "tdb")]
#[command(version, about = "trader database cli", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Import brokerage statements into the db
    #[command(arg_required_else_help = true)]
    Import {
        /// Statement filenames to import.
        #[arg(required = true)]
        path: Vec<PathBuf>,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Import { path } => {
            println!("importing paths: {:?}", path);
        }
    }
}
