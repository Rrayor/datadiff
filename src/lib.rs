use app::App;
use clap::{ArgGroup, Parser};
use dtfterminal_types::DtfError;

mod app;
mod array_table;
pub mod dtfterminal_types;
mod file_handler;
mod html_renderer;
mod json_app;
mod key_table;
mod type_table;
mod utils;
mod value_table;
mod yaml_app;

/// Command line arguments are handled here by clap
#[derive(Default, Parser, Debug)]
#[clap(
    version,
    about,
    group(
        ArgGroup::new("diff-options")
            .required(true)
            .multiple(true)
            .args(&["key_diffs", "type_diffs", "value_diffs", "array_diffs"]),
    ),
    group(
        ArgGroup::new("file-options")
        .required(true)
        .args(&["check_files", "read_from_file"])
    ),
    group(
        ArgGroup::new("browser-options")
        .required(false)
        .requires("browser_view")
        .multiple(true)
        .args(&["printer_friendly", "no_browser_show"])
    )
)]
/// Find the difference in your data structures
struct Arguments {
    /// The files to check if not reading from saved check
    #[clap(short, value_delimiter = ' ', num_args = 2)]
    check_files: Vec<String>,
    /// Read from a JSON file created on previous check instead of checking again
    #[clap(short, default_value_t = String::new())]
    read_from_file: String,

    /// Output to json file instead of rendering tables in the terminal
    #[clap(short)]
    write_to_file: Option<String>,

    /// Browser View: Output to an HTML file instead of rendering tables in the terminal
    #[clap(short)]
    browser_view: Option<String>,

    /// Printer friendly HTML output
    #[clap(short, default_value_t = false)]
    printer_friendly: bool,

    /// Don't show HTML in browser after creation
    #[clap(short, default_value_t = false)]
    no_browser_show: bool,

    /// Check for Key differences
    #[clap(short, default_value_t = false)]
    key_diffs: bool,
    /// Check for Type differences
    #[clap(short, default_value_t = false)]
    type_diffs: bool,
    /// Check for Value differences
    #[clap(short, default_value_t = false)]
    value_diffs: bool,
    /// Check for Array differences
    #[clap(short, default_value_t = false)]
    array_diffs: bool,

    /// Do you want arrays to be the same order? If defined you will get Value differences with indexes, otherwise you will get array differences, that tell you which object contains or misses values.
    #[clap(short = 'o', default_value_t = false)]
    array_same_order: bool,
}

pub fn run() -> Result<(), DtfError> {
    App::new().execute()
}

// Utils
