
use clap::{Parser, Subcommand, ValueEnum};
use std::fmt;

// unused for now, but will be used in the future for flags when needed
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, ValueEnum, Ord, Debug)]
pub enum ArgFlags {
    BURGER,
}




#[derive(Parser)]
pub struct Args{

    #[command(subcommand)]
    pub command: Commands,

    /// flags used in CLI commands
    #[arg(short,long, value_enum)]
    pub flags: Vec<ArgFlags>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Creates a new file
    New {
        file_path: String,
    },
    /// Creates a new directory
    Newd {
        dir_path: String,
    },
    /// Deletes a file
    Del {
        file_path: String,
    },
    /// Deletes a directory, if the directory is not empty then it will fail unless the -r/--recursion flag is used
    Deld {
        dir_path: String,
        #[arg(short,long)]
        recursion: bool,
    },
    /// Rename a file or directory, new name does not include path, just the new name
    Rename {
        file_path: String,
        new_file_name: String,
    },
    /// Lists the contents of the provided directory 
    List {
        dir_path: String,
    },
    /// Saves files/directories as the selection for `paste`, optionally also on the system clipboard
    Copy {
        // "paths" instead of "dir_paths", because copy takes files or directories
        #[arg(required = true, num_args = 1..)]
        paths: Vec<String>,
        /// Also put the items on the system clipboard
        #[arg(short, long)]
        clipboard: bool,
    },
    /// Pastes the copied items into a directory
    Paste {
        dir_path: String,
        /// Paste the files on the system clipboard instead of the copied selection
        #[arg(short, long)]
        clipboard: bool,
    },
    Tui,
}



impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        write!(f, "command: {:?}, flags: {:?}", self.command, self.flags)
    }
}