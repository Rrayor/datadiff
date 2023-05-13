use clap::{ArgGroup, Parser};
use colored::{Color, ColoredString, Colorize};
use libdtf::{
    diff_types, find_array_diffs, find_key_diffs, find_type_diffs, find_value_diffs, read_json_file,
};
use serde_json::{Map, Value};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use diff_types::{ArrayDiff, ArrayDiffDesc, KeyDiff, TypeDiff, ValueDiff, WorkingFile};

pub type LibConfig = libdtf::diff_types::Config;
pub type LibWorkingContext = libdtf::diff_types::WorkingContext;

pub struct Config {
    check_for_key_diffs: bool,
    check_for_type_diffs: bool,
    check_for_value_diffs: bool,
    check_for_array_diffs: bool,
}

impl Config {
    pub fn new(
        check_for_key_diffs: bool,
        check_for_type_diffs: bool,
        check_for_value_diffs: bool,
        check_for_array_diffs: bool,
    ) -> Config {
        Config {
            check_for_key_diffs,
            check_for_type_diffs,
            check_for_value_diffs,
            check_for_array_diffs,
        }
    }
}

pub struct WorkingContext {
    lib_working_context: LibWorkingContext,
    config: Config,
}

impl WorkingContext {
    pub fn new(lib_working_context: LibWorkingContext, config: Config) -> WorkingContext {
        WorkingContext {
            lib_working_context,
            config,
        }
    }
}

#[derive(Default, Parser, Debug)]
#[clap(version, about, group(
    ArgGroup::new("diff-options")
        .required(true)
        .multiple(true)
        .args(&["key_diffs", "type_diffs", "value_diffs", "array_diffs"]),
))]
/// Find the difference in your data structures
struct Arguments {
    /// The file containing your first data structure
    file_name1: String,
    /// The file containing your second data structure
    file_name2: String,

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

pub fn parse_args() -> (Map<String, Value>, Map<String, Value>, WorkingContext) {
    let args = Arguments::parse();
    let data1 = read_json_file(&args.file_name1)
        .unwrap_or_else(|_| panic!("Couldn't read file: {}", &args.file_name1));
    let data2 = read_json_file(&args.file_name2)
        .unwrap_or_else(|_| panic!("Couldn't read file: {}", &args.file_name2));

    let file_a = WorkingFile::new(args.file_name1.to_owned());
    let file_b = WorkingFile::new(args.file_name2.to_owned());

    let config = Config::new(
        args.key_diffs,
        args.type_diffs,
        args.value_diffs,
        args.array_diffs,
    );

    let lib_working_context =
        LibWorkingContext::new(file_a, file_b, LibConfig::new(args.array_same_order));

    let working_context = WorkingContext::new(lib_working_context, config);

    (data1, data2, working_context)
}

pub fn collect_data(
    data1: &Map<String, Value>,
    data2: &Map<String, Value>,
    working_context: &WorkingContext,
) -> (
    Option<Vec<KeyDiff>>,
    Option<Vec<TypeDiff>>,
    Option<Vec<ValueDiff>>,
    Option<Vec<ArrayDiff>>,
) {
    let key_diff = working_context
        .config
        .check_for_key_diffs
        .then(|| find_key_diffs("", data1, data2, &working_context.lib_working_context));
    let type_diff = working_context
        .config
        .check_for_type_diffs
        .then(|| find_type_diffs("", data1, data2, &working_context.lib_working_context));
    let value_diff = working_context
        .config
        .check_for_value_diffs
        .then(|| find_value_diffs("", data1, data2, &working_context.lib_working_context));
    let array_diff = working_context
        .config
        .check_for_array_diffs
        .then(|| find_array_diffs("", data1, data2, &working_context.lib_working_context));

    (key_diff, type_diff, value_diff, array_diff)
}

pub fn render_tables(
    key_diff: Option<Vec<KeyDiff>>,
    type_diff: Option<Vec<TypeDiff>>,
    value_diff: Option<Vec<ValueDiff>>,
    array_diff: Option<Vec<ArrayDiff>>,
    working_context: &WorkingContext,
) {
    key_diff.map(|diffs| {
        let table = create_table_key_diff(&diffs, &working_context.lib_working_context);
        println!("{}", table.render());
    });

    type_diff.map(|diffs| {
        let table = create_table_type_diff(&diffs, &working_context.lib_working_context);
        println!("{}", table.render());
    });

    value_diff.map(|diffs| {
        let table = create_table_value_diff(&diffs, &working_context.lib_working_context);
        println!("{}", table.render());
    });

    array_diff.map(|diffs| {
        let table = create_table_array_diff(&diffs, &working_context.lib_working_context);
        println!("{}", table.render());
    });
}

// Key table

fn create_table_key_diff<'a>(
    data: &Vec<KeyDiff>,
    working_context: &LibWorkingContext,
) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_key_table_header(&mut table, working_context);
    add_key_table_rows(&mut table, &data, working_context);

    table
}

fn add_key_table_header(table: &mut Table, working_context: &LibWorkingContext) {
    table.add_row(Row::new(vec![TableCell::new_with_alignment(
        "Key Differences",
        3,
        Alignment::Center,
    )]));
    table.add_row(Row::new(vec![
        TableCell::new("Key"),
        TableCell::new(&working_context.file_a.name),
        TableCell::new(&working_context.file_b.name),
    ]));
}

fn add_key_table_rows(table: &mut Table, data: &[KeyDiff], working_context: &LibWorkingContext) {
    for kd in data {
        table.add_row(Row::new(vec![
            TableCell::new(&kd.key),
            TableCell::new(check_has(&working_context.file_a.name, kd)),
            TableCell::new(check_has(&working_context.file_b.name, kd)),
        ]));
    }
}

fn check_has(file_name: &str, key_diff: &KeyDiff) -> ColoredString {
    if key_diff.has == file_name {
        CHECKMARK.color(Color::Green)
    } else {
        MULTIPLY.color(Color::Red)
    }
}

// Type table

fn create_table_type_diff<'a>(
    data: &Vec<TypeDiff>,
    working_context: &LibWorkingContext,
) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_type_table_header(&mut table, working_context);
    add_type_table_rows(&mut table, &data);

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
    add_value_table_rows(&mut table, &data);

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
            TableCell::new(&sanitize_json_str(&vd.value1)),
            TableCell::new(&sanitize_json_str(&vd.value2)),
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
    add_array_table_rows(&mut table, &data);

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
        let value_str = sanitize_json_str(&ad.value);
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

fn sanitize_json_str(json_str: &str) -> String {
    match serde_json::from_str::<Value>(json_str) {
        Ok(json_value) => serde_json::to_string_pretty(&json_value).unwrap_or(json_str.to_owned()),
        Err(_) => json_str.to_owned(),
    }
}
