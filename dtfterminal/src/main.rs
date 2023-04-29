use colored::{Color, ColoredString, Colorize};
use libdtf::{compare_objects, diff_types, read_json_file};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use diff_types::{
    ArrayDiff, ArrayDiffDesc, KeyDiff, TypeDiff, ValueDiff, WorkingContext, WorkingFile,
};

fn main() -> Result<(), ()> {
    let file_name1 = "test_data/person3.json";
    let file_name2 = "test_data/person4.json";
    let data1 = read_json_file(file_name1).expect(&format!("Couldn't read file: {}", file_name1));
    let data2 = read_json_file(file_name2).expect(&format!("Couldn't read file: {}", file_name2));
    let working_context = WorkingContext {
        file_a: WorkingFile {
            name: file_name1.to_string(),
        },
        file_b: WorkingFile {
            name: file_name2.to_string(),
        },
    };
    let (key_diff, type_diff, value_diff, array_diff) =
        compare_objects("", &data1, &data2, &working_context);

    let key_diff_table = create_table_key_diff(key_diff, &working_context);
    println!("{}", key_diff_table.render());

    let type_diff_table = create_table_type_diff(type_diff, &working_context);
    println!("{}", type_diff_table.render());

    let value_diff_table = create_table_value_diff(value_diff, &working_context);
    println!("{}", value_diff_table.render());

    let array_diff_table = create_table_array_diff(array_diff, &working_context);
    println!("{}", array_diff_table.render());

    Ok(())
}

fn create_table_key_diff<'a>(data: Vec<KeyDiff>, working_context: &WorkingContext) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_key_table_header(&mut table, &working_context);
    add_key_table_rows(&mut table, &data, &working_context);

    table
}

fn add_key_table_header(table: &mut Table, working_context: &WorkingContext) {
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

fn add_key_table_rows(table: &mut Table, data: &[KeyDiff], working_context: &WorkingContext) {
    for kd in data {
        table.add_row(Row::new(vec![
            TableCell::new(&kd.key),
            TableCell::new(check_has(&working_context.file_a.name, &kd)),
            TableCell::new(check_has(&working_context.file_b.name, &kd)),
        ]));
    }
}

fn check_has(file_name: &str, key_diff: &KeyDiff) -> ColoredString {
    if key_diff.has == file_name {
        "\u{2713}".color(Color::Green)
    } else {
        "\u{00D7}".color(Color::Red)
    }
}

fn create_table_type_diff<'a>(data: Vec<TypeDiff>, working_context: &WorkingContext) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_type_table_header(&mut table, &working_context);
    add_type_table_rows(&mut table, &data);

    table
}

fn add_type_table_header(table: &mut Table, working_context: &WorkingContext) {
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

fn create_table_value_diff<'a>(
    data: Vec<ValueDiff>,
    working_context: &WorkingContext,
) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_value_table_header(&mut table, &working_context);
    add_value_table_rows(&mut table, &data);

    table
}

fn add_value_table_header(table: &mut Table, working_context: &WorkingContext) {
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
            TableCell::new(&vd.value1),
            TableCell::new(&vd.value2),
        ]));
    }
}

fn create_table_array_diff<'a>(
    data: Vec<ArrayDiff>,
    working_context: &WorkingContext,
) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_array_table_header(&mut table, &working_context);
    add_array_table_rows(&mut table, &data);

    table
}

fn add_array_table_header(table: &mut Table, working_context: &WorkingContext) {
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
        let value_str = ad
            .value
            .replace("{", "{\n")
            .replace("},", "\n},\n")
            .replace("\"}", "\"\n}")
            .replace(",", ",\n");
        table.add_row(Row::new(vec![
            TableCell::new(&ad.key),
            TableCell::new(
                get_array_table_cell_value(&ad.descriptor, &value_str).color(Color::Green),
            ),
            TableCell::new(
                get_array_table_cell_value(&ad.descriptor, &value_str).color(Color::Red),
            ),
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
