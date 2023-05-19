use std::{error::Error, fs::File, io::BufReader};

use app::App;
use array_table::ArrayTable;
use clap::{ArgGroup, Parser};
use dtfterminal_types::{
    Config, ConfigBuilder, DiffCollection, DtfError, LibConfig, LibWorkingContext, ParsedArgs,
    SavedConfig, SavedContext, WorkingContext,
};
use key_table::KeyTable;
use libdtf::{
    diff_types::{self},
    read_json_file,
};
use serde_json::{Map, Value};

use diff_types::{ArrayDiff, Checker, CheckingData, KeyDiff, TypeDiff, ValueDiff, WorkingFile};
use type_table::TypeTable;
use value_table::ValueTable;

use crate::dtfterminal_types::TermTable;

mod app;
mod array_table;
pub mod dtfterminal_types;
mod file_handler;
mod key_table;
mod type_table;
mod value_table;

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

fn init() -> (DiffCollection, WorkingContext) {
    let (data1, data2, config) = parse_args();
    collect_data(data1, data2, &config)
}

fn execute(diffs: DiffCollection, working_context: &WorkingContext) -> Result<(), DtfError> {
    if working_context.config.write_to_file.is_some() {
        write_to_file(diffs, &working_context).map_err(|e| DtfError::GeneralError(Box::new(e)))?;
    } else {
        render_tables(diffs, &working_context)
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
    }

    Ok(())
}

fn parse_args() -> ParsedArgs {
    let args = Arguments::parse();

    let (data1, data2) = if args.read_from_file.is_empty() {
        let data1 = read_json_file(&args.check_files[0])
            .unwrap_or_else(|_| panic!("Couldn't read file: {}", &args.check_files[0]));
        let data2 = read_json_file(&args.check_files[1])
            .unwrap_or_else(|_| panic!("Couldn't read file: {}", &args.check_files[1]));
        (Some(data1), Some(data2))
    } else {
        (None, None)
    };

    let config = ConfigBuilder::new()
        .check_for_key_diffs(args.key_diffs)
        .check_for_type_diffs(args.type_diffs)
        .check_for_value_diffs(args.value_diffs)
        .check_for_array_diffs(args.array_diffs)
        .render_key_diffs(args.key_diffs)
        .render_type_diffs(args.type_diffs)
        .render_value_diffs(args.value_diffs)
        .render_array_diffs(args.array_diffs)
        .read_from_file(args.read_from_file)
        .write_to_file(args.write_to_file)
        .file_a(data1.clone().map(|_| args.check_files[0].clone()))
        .file_b(data2.clone().map(|_| args.check_files[1].clone()))
        .array_same_order(args.array_same_order)
        .build();

    (data1, data2, config)
}

fn collect_data(
    data1: Option<Map<String, Value>>,
    data2: Option<Map<String, Value>>,
    user_config: &Config,
) -> (DiffCollection, WorkingContext) {
    if user_config.read_from_file.is_empty() {
        perform_new_check(data1, data2, user_config).expect("Data check failed!")
    } else {
        load_saved_results(user_config).expect("Could not load saved file!")
    }
}

fn perform_new_check(
    data1: Option<Map<String, Value>>,
    data2: Option<Map<String, Value>>,
    user_config: &Config,
) -> Result<(DiffCollection, WorkingContext), Box<dyn Error>> {
    let file_a = WorkingFile::new(user_config.file_a.as_ref().unwrap().clone());
    let file_b = WorkingFile::new(user_config.file_b.as_ref().unwrap().clone());

    let lib_working_context =
        LibWorkingContext::new(file_a, file_b, LibConfig::new(user_config.array_same_order));

    let working_context = WorkingContext::new(lib_working_context, user_config.clone());

    let diffs = check_for_diffs(
        &data1.ok_or("Contents of first file are missing")?,
        &data2.ok_or("Contents of second file are missing")?,
        &working_context,
    );

    Ok((diffs, working_context))
}

fn check_for_diffs(
    data1: &Map<String, Value>,
    data2: &Map<String, Value>,
    working_context: &WorkingContext,
) -> DiffCollection {
    let key_diff = if working_context.config.check_for_key_diffs {
        let mut checking_data: CheckingData<KeyDiff> =
            CheckingData::new("", data1, data2, &working_context.lib_working_context);
        checking_data.check();
        Some(checking_data.diffs()).cloned()
    } else {
        None
    };
    let type_diff = if working_context.config.check_for_type_diffs {
        let mut checking_data: CheckingData<TypeDiff> =
            CheckingData::new("", data1, data2, &working_context.lib_working_context);
        checking_data.check();
        Some(checking_data.diffs()).cloned()
    } else {
        None
    };
    let value_diff = if working_context.config.check_for_value_diffs {
        let mut checking_data: CheckingData<ValueDiff> =
            CheckingData::new("", data1, data2, &working_context.lib_working_context);
        checking_data.check();
        Some(checking_data.diffs()).cloned()
    } else {
        None
    };
    let array_diff = if working_context.config.check_for_array_diffs {
        let mut checking_data: CheckingData<ArrayDiff> =
            CheckingData::new("", data1, data2, &working_context.lib_working_context);
        checking_data.check();
        Some(checking_data.diffs()).cloned()
    } else {
        None
    };

    (key_diff, type_diff, value_diff, array_diff)
}

fn render_tables<'a>(
    diffs: DiffCollection,
    working_context: &WorkingContext,
) -> Result<(), DtfError> {
    let (key_diff, type_diff, value_diff, array_diff) = diffs;

    let mut rendered_tables = vec![];
    if working_context.config.render_key_diffs {
        if let Some(diffs) = key_diff.filter(|kd| !kd.is_empty()) {
            let table = KeyTable::new(&diffs, &working_context.lib_working_context);
            rendered_tables.push(table.render());
        }
    }

    if working_context.config.render_type_diffs {
        if let Some(diffs) = type_diff.filter(|td| !td.is_empty()) {
            let table = TypeTable::new(&diffs, &working_context.lib_working_context);
            rendered_tables.push(table.render());
        }
    }

    if working_context.config.render_value_diffs {
        if let Some(diffs) = value_diff.filter(|vd| !vd.is_empty()) {
            let table = ValueTable::new(&diffs, &working_context.lib_working_context);
            rendered_tables.push(table.render());
        }
    }

    if working_context.config.render_array_diffs {
        if let Some(diffs) = array_diff.filter(|ad| !ad.is_empty()) {
            let table = ArrayTable::new(&diffs, &working_context.lib_working_context);
            rendered_tables.push(table.render());
        }
    }

    for table in rendered_tables {
        println!("{}", table);
    }

    Ok(())
}

// Utils

fn prettyfy_json_str(json_str: &str) -> String {
    match serde_json::from_str::<Value>(json_str) {
        Ok(json_value) => serde_json::to_string_pretty(&json_value).unwrap_or(json_str.to_owned()),
        Err(_) => json_str.to_owned(),
    }
}
