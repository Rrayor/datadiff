use app::App;
use clap::{ArgGroup, Parser};
use dtfterminal_types::DtfError;
use serde_json::Value;

mod app;
mod array_table;
pub mod dtfterminal_types;
mod file_handler;
mod key_table;
mod type_table;
mod value_table;

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

/// Formats JSON strings
fn prettify_json_str(json_str: &str) -> String {
    match serde_json::from_str::<Value>(json_str) {
        Ok(json_value) => serde_json::to_string_pretty(&json_value).unwrap_or(json_str.to_owned()),
        Err(_) => json_str.to_owned(),
    }
}
