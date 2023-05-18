use colored::{Color, ColoredString, Colorize};
use libdtf::diff_types::KeyDiff;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use crate::dtfterminal_types::{LibWorkingContext, TableContext, TermTable};

const CHECKMARK: &str = "\u{2713}";
const MULTIPLY: &str = "\u{00D7}";

pub struct KeyTable<'a> {
    context: &'a TableContext<'a>,
}

impl<'a> TermTable<KeyDiff> for KeyTable<'a> {
    fn create_table(&mut self, data: &[KeyDiff]) {
        let mut table = Table::new();
        table.max_column_width = 80;
        table.style = TableStyle::extended();

        self.add_header();
        self.add_rows(data);

        self.context.set_table(table);
    }

    fn add_header(&mut self) {
        self.context
            .table()
            .add_row(Row::new(vec![TableCell::new_with_alignment(
                "Key Differences",
                3,
                Alignment::Center,
            )]));
        self.context.table().add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(&self.context.working_context().file_a.name),
            TableCell::new(&self.context.working_context().file_b.name),
        ]));
    }

    fn add_rows(&mut self, data: &[KeyDiff]) {
        for kd in data {
            self.context.table().add_row(Row::new(vec![
                TableCell::new(&kd.key),
                TableCell::new(self.check_has(&self.context.working_context().file_a.name, kd)),
                TableCell::new(self.check_has(&self.context.working_context().file_b.name, kd)),
            ]));
        }
    }
}

impl<'a> KeyTable<'a> {
    pub fn new(working_context: &LibWorkingContext) -> KeyTable {
        KeyTable {
            context: &TableContext::new(working_context),
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
