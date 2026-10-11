
pub mod cli_args;

use cli_args::{Args, Commands};
use pfad_core::*;

/// Runs one CLI command by calling the matching `pfad_core` function.
///
/// `List` prints its entries to stdout. `Tui` is handled by the `pfad` binary
/// and returns [`PfadError::Cli`] here.
pub fn run(args: Args) -> Result<()> {

    match args.command {
        Commands::New { file_path } => {
            pfad_core::create_file(file_path)?;
        }
        Commands::Newd { dir_path } => {
            pfad_core::create_dir(dir_path)?;
        }
        Commands::Del { file_path } => {
            pfad_core::remove_file(file_path)?;
        }
        Commands::Deld { dir_path, recursion } => {
            let recursion = if recursion {
                Recursion::Yes
            } else {
                Recursion::No
            };

            pfad_core::remove_dir(dir_path, recursion)?;
        }
        Commands::Rename { file_path, new_file_name } => {
            pfad_core::rename(file_path, new_file_name)?;
        }
        Commands::List { dir_path } => {
            let dir_list = pfad_core::list_dir(dir_path)?;
            for entry in dir_list {
                println!("{}", entry);
            }
        }
        Commands::Copy { paths, clipboard } => {
            pfad_core::copy_files(paths, clipboard)?;
        }
        Commands::Paste { dir_path, clipboard } => {
            pfad_core::paste_files(dir_path, clipboard)?;
        }
        _ => {
            return Err(PfadError::Cli("Invalid command".to_string()));
        }
    }

    Ok(())
}