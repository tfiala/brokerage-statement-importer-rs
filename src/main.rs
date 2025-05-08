mod importer;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use mongodb::Client;
use std::path::PathBuf;

use dotenvy::EnvLoader;
#[derive(Parser, Debug)]
#[command(name = "tdb")]
#[command(version, about = "trader database cli", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// The database URI to connect to. Defaults to DB_URI in .env file.
    #[arg(short, long, required = false)]
    db_uri: Option<String>,
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

#[tokio::main]
async fn main() -> Result<()> {
    // Setup the environment
    let env_map = EnvLoader::new().load()?;
    let args = Cli::parse();

    let db_uri = match args.db_uri {
        Some(uri) => uri,
        None => env_map
            .get("DB_URI")
            .expect("DATABASE_URL not set in environment")
            .to_string(),
    };
    println!("Connecting to database at: {}", db_uri);

    // Connect to the database.
    let client = Client::with_uri_str(&db_uri)
        .await
        .expect("Failed to connect to database");
    println!("Connected to database");

    // Get the database.
    let db_name = env_map
        .get("DB_NAME")
        .expect("DB_NAME not set in environment")
        .to_string();
    let _db = client.database(&db_name);
    println!("Using database: {}", db_name);

    match args.command {
        Commands::Import(import_args) => match import_args.command {
            ImportCommands::Regex { regex } => {
                let globbed_paths = glob::glob(regex.as_str())?
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
