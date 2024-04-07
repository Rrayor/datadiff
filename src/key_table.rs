use crate::utils::{CHECKMARK, MULTIPLY};
use colored::{Color, ColoredString, Colorize};
use libdtf::core::diff_types::KeyDiff;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
};

use crate::dtfterminal_types::{LibWorkingContext, TableContext, TermTable};

pub struct KeyTable<'a> {
    context: TableContext<'a>,
}

impl<'a> TermTable<KeyDiff> for KeyTable<'a> {
    fn render(&self) -> String {
        self.context.render()
    }

    fn create_table(&mut self, data: &[KeyDiff]) {
        self.add_header();
        self.add_rows(data);
    }

    fn add_header(&mut self) {
        let (file_name_a_str, file_name_b_str) = self.get_file_names();
        let file_name_a = file_name_a_str.to_owned();
        let file_name_b = file_name_b_str.to_owned();
        self.add_title_row();
        self.add_file_name_row(file_name_a, file_name_b);
    }

    fn add_rows(&mut self, data: &[KeyDiff]) {
        let (file_name_a_str, file_name_b_str) = self.get_file_names();
        let file_name_a = file_name_a_str.to_owned();
        let file_name_b = file_name_b_str.to_owned();
        for kd in data {
            let a_has = self.check_has(file_name_a.as_str(), kd);
            let b_has = self.check_has(file_name_b.as_str(), kd);
            self.context.add_row(Row::new(vec![
                TableCell::new(&kd.key),
                TableCell::new(a_has),
                TableCell::new(b_has),
            ]));
        }
    }
}

impl<'a> KeyTable<'a> {
    pub fn new(data: &[KeyDiff], working_context: &'a LibWorkingContext) -> KeyTable<'a> {
        let mut table = KeyTable {
            context: TableContext::new(working_context),
        };
        table.create_table(data);
        table
    }

    fn check_has(&self, file_name: &str, key_diff: &KeyDiff) -> ColoredString {
        if key_diff.has == file_name {
            CHECKMARK.color(Color::Green)
        } else {
            MULTIPLY.color(Color::Red)
        }
    }

    fn get_file_names(&mut self) -> (&str, &str) {
        let file_name_a = self.context.working_context().file_a.name.as_str();
        let file_name_b = self.context.working_context().file_b.name.as_str();
        (file_name_a, file_name_b)
    }

    fn add_title_row(&mut self) {
        self.context
            .add_row(Row::new(vec![TableCell::new_with_alignment(
                "Key Differences",
                3,
                Alignment::Center,
            )]));
    }

    fn add_file_name_row(&mut self, file_name_a: String, file_name_b: String) {
        self.context.add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(file_name_a),
            TableCell::new(file_name_b),
        ]));
    }
}
