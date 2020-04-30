//! StructOpt data
use std::path::PathBuf;

use clap::Shell;
use structopt::StructOpt;

use crate::the_way::filter::Filters;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "the-way",
    about = "Record, retrieve, search, and categorize code snippets"
)]
pub(crate) struct TheWayCLI {
    /// Copy snippet at <INDEX> to clipboard
    #[structopt(short = "y", long = "cp")]
    pub(crate) copy: Option<usize>,

    /// Delete snippet at <INDEX>
    #[structopt(short, long)]
    pub(crate) delete: Option<usize>,

    /// Show snippet at <INDEX>
    #[structopt(short, long)]
    pub(crate) show: Option<usize>,

    /// Change snippet at <INDEX>
    #[structopt(short, long)]
    pub(crate) change: Option<usize>,

    /// Generate shell completions
    #[structopt(long = "sh", name = "SHELL", possible_values = & Shell::variants())]
    pub(crate) complete: Option<Shell>,

    #[structopt(subcommand)]
    pub(crate) command: Option<TheWayCommand>,
}

#[derive(StructOpt, Debug)]
pub(crate) enum TheWayCommand {
    /// Fuzzy search and copy selected to clipboard
    Search {
        #[structopt(flatten)]
        filters: Filters,
    },
    /// Lists snippets
    List {
        #[structopt(flatten)]
        filters: Filters,
    },
    /// Saves (optionally filtered) snippets to a JSON file.
    Export {
        #[structopt(flatten)]
        filters: Filters,
        /// filename, writes to stdout if not given
        #[structopt(long, short, parse(from_os_str))]
        file: Option<PathBuf>,
    },
    /// Imports code snippets from a JSON file. Looks for description, language, and code fields
    Import {
        #[structopt(long, short, parse(from_os_str))]
        file: PathBuf,
    },
    /// View syntax highlighting themes (default + user-added)
    Themes {
        #[structopt(subcommand)]
        cmd: Option<ThemeCommand>,
    },
    /// Clears all data
    Clear {
        /// Don't ask for confirmation
        #[structopt(long, short)]
        force: bool,
    },
}

#[derive(StructOpt, Debug)]
pub(crate) enum ThemeCommand {
    /// Set your preferred syntax highlighting theme
    Set {
        #[structopt(long, short)]
        theme: String,
    },
    /// Add a theme from a .tmTheme file
    Add {
        #[structopt(long, short, parse(from_os_str))]
        file: PathBuf,
    },
}
