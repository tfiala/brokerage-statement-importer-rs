mod importer;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
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
    Import(ImportArgs),
}

#[derive(Args, Debug)]
struct ImportArgs {
    #[command(subcommand)]
    command: ImportCommands,
}

#[derive(Subcommand, Debug)]
enum ImportCommands {
    /// Select brokerage statements via a regex (i.e. file glob) expression
    #[command(arg_required_else_help = true)]
    Regex { regex: String },

    /// Select brokerage statements via one or more filenames.
    #[command(arg_required_else_help = true)]
    Files {
        /// Statement filenames to import.
        #[arg(required = true)]
        path: Vec<PathBuf>,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Import(import_args) => match import_args.command {
            ImportCommands::Regex { regex } => {
                let glob_result = glob::glob(regex.as_str())?;
                println!("glob result: {:?}", glob_result);

                let globbed_paths = glob_result
                    .filter_map(|entry| entry.ok())
                    .collect::<Vec<PathBuf>>();

                let import_paths = importer::filter_unimported_files(globbed_paths)?;
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
            ImportCommands::Files { path } => {
                if path.is_empty() {
                    Err(anyhow::anyhow!("no files provided"))
                } else {
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
        },
    }
}
