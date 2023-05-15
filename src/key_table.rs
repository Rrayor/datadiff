use colored::{Color, ColoredString, Colorize};
use libdtf::diff_types::KeyDiff;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use crate::dtfterminal_types::LibWorkingContext;

const CHECKMARK: &str = "\u{2713}";
const MULTIPLY: &str = "\u{00D7}";

pub struct KeyTable<'a> {
    working_context: &'a LibWorkingContext,
    pub table: Table<'a>,
}

impl<'a> KeyTable<'a> {
    pub fn new(data: &[KeyDiff], working_context: &'a LibWorkingContext) -> KeyTable<'a> {
        let mut table = KeyTable {
            working_context,
            table: Table::new(),
        };
        table.create_table_key_diff(data);
        table
    }

    fn create_table_key_diff(&mut self, data: &[KeyDiff]) {
        let mut table = Table::new();
        table.max_column_width = 80;
        table.style = TableStyle::extended();

        self.add_key_table_header();
        self.add_key_table_rows(data);

        self.table = table;
    }

    fn add_key_table_header(&mut self) {
        self.table
            .add_row(Row::new(vec![TableCell::new_with_alignment(
                "Key Differences",
                3,
                Alignment::Center,
            )]));
        self.table.add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(&self.working_context.file_a.name),
            TableCell::new(&self.working_context.file_b.name),
        ]));
    }

    pub fn add_key_table_rows(&mut self, data: &[KeyDiff]) {
        for kd in data {
            self.table.add_row(Row::new(vec![
                TableCell::new(&kd.key),
                TableCell::new(self.check_has(&self.working_context.file_a.name, kd)),
                TableCell::new(self.check_has(&self.working_context.file_b.name, kd)),
            ]));
        }
    }

    fn check_has(&self, file_name: &str, key_diff: &KeyDiff) -> ColoredString {
        if key_diff.has == file_name {
            CHECKMARK.color(Color::Green)
        } else {
            MULTIPLY.color(Color::Red)
        }
    }
}
