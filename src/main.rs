mod importer;

use anyhow::Result;
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

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Import { path } => {
            let import_paths = importer::filter_unimported_files(path)?;
            if import_paths.is_empty() {
                println!("all statements already imported");
                Ok(())
            } else {
                println!("importing the following new statements:");
                for sp in import_paths {
                    println!("\t{:?}", sp);
                }
                Ok(())
            }
        }
    }
}
