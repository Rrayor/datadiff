use std::{error::Error, fs::File, io::BufReader};

use clap::{ArgGroup, Parser};
use colored::{Color, ColoredString, Colorize};
use dtfterminal_types::{
    Config, ConfigBuilder, DiffCollection, DtfError, LibConfig, LibWorkingContext, ParsedArgs,
    SavedConfig, SavedContext, WorkingContext,
};
use key_table::KeyTable;
use libdtf::{diff_types, read_json_file};
use serde_json::{Map, Value};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use diff_types::{
    ArrayDiff, ArrayDiffDesc, Checker, CheckingData, KeyDiff, TypeDiff, ValueDiff, WorkingFile,
};

pub mod dtfterminal_types;
mod key_table;

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

const CHECKMARK: &str = "\u{2713}";
const MULTIPLY: &str = "\u{00D7}";

pub fn parse_args() -> ParsedArgs {
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

pub fn collect_data(
    data1: Option<Map<String, Value>>,
    data2: Option<Map<String, Value>>,
    user_config: &Config,
) -> (DiffCollection, WorkingContext) {
    if user_config.read_from_file.is_empty() {
        collect_data_new_check(data1, data2, user_config).expect("Data check failed!")
    } else {
        collect_data_load(user_config).expect("Could not load saved file!")
    }
}

fn collect_data_new_check(
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

fn collect_data_load(
    user_config: &Config,
) -> Result<(DiffCollection, WorkingContext), Box<dyn Error>> {
    let saved_data = match read_from_file(&user_config.read_from_file) {
        Ok(data) => data,
        Err(e) => return Err(Box::new(DtfError::IoError(e.into()))),
    };
    let saved_config = saved_data.config;

    let diff_collection = (
        Some(saved_data.key_diff),
        Some(saved_data.type_diff),
        Some(saved_data.value_diff),
        Some(saved_data.array_diff),
    );

    let working_context = build_working_context_from_loaded_data(&saved_config, user_config);

    Ok((diff_collection, working_context))
}

fn build_working_context_from_loaded_data(
    saved_config: &SavedConfig,
    user_config: &Config,
) -> WorkingContext {
    let file_a = WorkingFile::new(saved_config.file_a.clone());
    let file_b = WorkingFile::new(saved_config.file_b.clone());
    let lib_working_context = LibWorkingContext::new(
        file_a,
        file_b,
        LibConfig::new(saved_config.array_same_order),
    );
    WorkingContext::new(
        lib_working_context,
        ConfigBuilder::new()
            .check_for_key_diffs(saved_config.check_for_key_diffs)
            .check_for_type_diffs(saved_config.check_for_type_diffs)
            .check_for_value_diffs(saved_config.check_for_value_diffs)
            .check_for_array_diffs(saved_config.check_for_array_diffs)
            .render_key_diffs(user_config.render_key_diffs)
            .render_type_diffs(user_config.render_type_diffs)
            .render_value_diffs(user_config.render_value_diffs)
            .render_array_diffs(user_config.render_array_diffs)
            .read_from_file(user_config.read_from_file.clone())
            .write_to_file(user_config.write_to_file.clone())
            .file_a(Some(saved_config.file_a.clone()))
            .file_b(Some(saved_config.file_b.clone()))
            .array_same_order(saved_config.array_same_order)
            .build(),
    )
}

pub fn check_for_diffs(
    data1: &Map<String, Value>,
    data2: &Map<String, Value>,
    working_context: &WorkingContext,
) -> DiffCollection {
    let key_diff = if working_context.config.check_for_key_diffs {
        let mut checking_data: CheckingData<KeyDiff> =
            CheckingData::new("", data1, data2, &working_context.lib_working_context);
        checking_data.check();
        Some(checking_data.diffs)
    } else {
        None
    };
    let type_diff = if working_context.config.check_for_type_diffs {
        let mut checking_data: CheckingData<TypeDiff> =
            CheckingData::new("", data1, data2, &working_context.lib_working_context);
        checking_data.check();
        Some(checking_data.diffs)
    } else {
        None
    };
    let value_diff = if working_context.config.check_for_value_diffs {
        let mut checking_data: CheckingData<ValueDiff> =
            CheckingData::new("", data1, data2, &working_context.lib_working_context);
        checking_data.check();
        Some(checking_data.diffs)
    } else {
        None
    };
    let array_diff = if working_context.config.check_for_array_diffs {
        let mut checking_data: CheckingData<ArrayDiff> =
            CheckingData::new("", data1, data2, &working_context.lib_working_context);
        checking_data.check();
        Some(checking_data.diffs)
    } else {
        None
    };

    (key_diff, type_diff, value_diff, array_diff)
}

pub fn read_from_file(file_path: &str) -> serde_json::Result<SavedContext> {
    let file =
        File::open(file_path).unwrap_or_else(|_| panic!("Could not open file {}", file_path));
    let reader = BufReader::new(file);
    serde_json::from_reader(reader)
}

pub fn write_to_file(
    key_diff_option: Option<Vec<KeyDiff>>,
    type_diff_option: Option<Vec<TypeDiff>>,
    value_diff_option: Option<Vec<ValueDiff>>,
    array_diff_option: Option<Vec<ArrayDiff>>,
    working_context: &WorkingContext,
) -> Result<(), DtfError> {
    let key_diff = key_diff_option.unwrap_or_default();
    let type_diff = type_diff_option.unwrap_or_default();
    let value_diff = value_diff_option.unwrap_or_default();
    let array_diff = array_diff_option.unwrap_or_default();

    if let Some(write_to_file) = working_context.config.write_to_file.clone() {
        let file = File::create(write_to_file);
        let config = &working_context.config;

        match serde_json::to_writer(
            &mut file.unwrap(),
            &SavedContext::new(
                key_diff,
                type_diff,
                value_diff,
                array_diff,
                SavedConfig::new(
                    config.check_for_key_diffs,
                    config.check_for_type_diffs,
                    config.check_for_value_diffs,
                    config.check_for_array_diffs,
                    config.file_a.clone().unwrap(),
                    config.file_b.clone().unwrap(),
                    config.array_same_order,
                ),
            ),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(DtfError::IoError(e.into())),
        }
    } else {
        Err(DtfError::DiffError(
            "File path missing for writing results!".to_owned(),
        ))
    }
}

pub fn render_tables(
    key_diff: Option<Vec<KeyDiff>>,
    type_diff: Option<Vec<TypeDiff>>,
    value_diff: Option<Vec<ValueDiff>>,
    array_diff: Option<Vec<ArrayDiff>>,
    working_context: &WorkingContext,
) -> Result<(), DtfError> {
    let mut tables = vec![];
    if working_context.config.render_key_diffs {
        if let Some(diffs) = key_diff.filter(|kd| !kd.is_empty()) {
            let table = KeyTable::new(&diffs, &working_context.lib_working_context);
            tables.push(table.table);
        }
    }

    if working_context.config.render_type_diffs {
        if let Some(diffs) = type_diff.filter(|td| !td.is_empty()) {
            let table = create_table_type_diff(&diffs, &working_context.lib_working_context);
            tables.push(table);
        }
    }

    if working_context.config.render_value_diffs {
        if let Some(diffs) = value_diff.filter(|vd| !vd.is_empty()) {
            let table = create_table_value_diff(&diffs, &working_context.lib_working_context);
            tables.push(table);
        }
    }

    if working_context.config.render_array_diffs {
        if let Some(diffs) = array_diff.filter(|ad| !ad.is_empty()) {
            let table = create_table_array_diff(&diffs, &working_context.lib_working_context);
            tables.push(table);
        }
    }

    for table in tables {
        println!("{}", table.render());
    }

    Ok(())
}

// Type table

fn create_table_type_diff<'a>(data: &[TypeDiff], working_context: &LibWorkingContext) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_type_table_header(&mut table, working_context);
    add_type_table_rows(&mut table, data);

    table
}

fn add_type_table_header(table: &mut Table, working_context: &LibWorkingContext) {
    table.add_row(Row::new(vec![TableCell::new_with_alignment(
        "Type Differences",
        3,
        Alignment::Center,
    )]));
    table.add_row(Row::new(vec![
        TableCell::new("Key"),
        TableCell::new(&working_context.file_a.name),
        TableCell::new(&working_context.file_b.name),
    ]));
}

fn add_type_table_rows(table: &mut Table, data: &[TypeDiff]) {
    for td in data {
        table.add_row(Row::new(vec![
            TableCell::new(&td.key),
            TableCell::new(&td.type1),
            TableCell::new(&td.type2),
        ]));
    }
}

// Value table

fn create_table_value_diff<'a>(
    data: &Vec<ValueDiff>,
    working_context: &LibWorkingContext,
) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_value_table_header(&mut table, working_context);
    add_value_table_rows(&mut table, data);

    table
}

fn add_value_table_header(table: &mut Table, working_context: &LibWorkingContext) {
    table.add_row(Row::new(vec![TableCell::new_with_alignment(
        "Value Differences",
        3,
        Alignment::Center,
    )]));
    table.add_row(Row::new(vec![
        TableCell::new("Key"),
        TableCell::new(&working_context.file_a.name),
        TableCell::new(&working_context.file_b.name),
    ]));
}

fn add_value_table_rows(table: &mut Table, data: &Vec<ValueDiff>) {
    for vd in data {
        table.add_row(Row::new(vec![
            TableCell::new(&vd.key),
            TableCell::new(&prettyfy_json_str(&vd.value1)),
            TableCell::new(&prettyfy_json_str(&vd.value2)),
        ]));
    }
}

// Array table

fn create_table_array_diff<'a>(
    data: &Vec<ArrayDiff>,
    working_context: &LibWorkingContext,
) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_array_table_header(&mut table, working_context);
    add_array_table_rows(&mut table, data);

    table
}

fn add_array_table_header(table: &mut Table, working_context: &LibWorkingContext) {
    table.add_row(Row::new(vec![TableCell::new_with_alignment(
        "Array Differences",
        3,
        Alignment::Center,
    )]));
    table.add_row(Row::new(vec![
        TableCell::new("Key"),
        TableCell::new(&working_context.file_a.name),
        TableCell::new(&working_context.file_b.name),
    ]));
}

fn add_array_table_rows(table: &mut Table, data: &Vec<ArrayDiff>) {
    for ad in data {
        let value_str = prettyfy_json_str(&ad.value);
        table.add_row(Row::new(vec![
            TableCell::new(&ad.key),
            TableCell::new(get_array_table_cell_value(&ad.descriptor, &value_str)),
            TableCell::new(get_array_table_cell_value(&ad.descriptor, &value_str)),
        ]));
    }
}

fn get_array_table_cell_value<'a>(descriptor: &'a ArrayDiffDesc, value_str: &'a str) -> &'a str {
    match descriptor {
        ArrayDiffDesc::AHas => value_str,
        ArrayDiffDesc::AMisses => value_str,
        ArrayDiffDesc::BHas => value_str,
        ArrayDiffDesc::BMisses => value_str,
    }
}

// Utils

fn prettyfy_json_str(json_str: &str) -> String {
    match serde_json::from_str::<Value>(json_str) {
        Ok(json_value) => serde_json::to_string_pretty(&json_value).unwrap_or(json_str.to_owned()),
        Err(_) => json_str.to_owned(),
    }
}
