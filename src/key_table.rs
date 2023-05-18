use colored::{Color, ColoredString, Colorize};
use libdtf::diff_types::KeyDiff;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
};

use crate::dtfterminal_types::{LibWorkingContext, TableContext, TermTable};

const CHECKMARK: &str = "\u{2713}";
const MULTIPLY: &str = "\u{00D7}";

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
        // TODO: This may need a cleanup. I can only hold 1 reference to self in a scope, if that's mutable.
        let file_name_a;
        let file_name_b;
        {
            file_name_a = self.context.working_context().file_a.name.as_str();
            file_name_b = self.context.working_context().file_b.name.as_str();
        }
        {
            self.context
                .add_row(Row::new(vec![TableCell::new_with_alignment(
                    "Key Differences",
                    3,
                    Alignment::Center,
                )]));
        }
        {
            self.context.add_row(Row::new(vec![
                TableCell::new("Key"),
                TableCell::new(file_name_a),
                TableCell::new(file_name_b),
            ]));
        }
    }

    fn add_rows(&mut self, data: &[KeyDiff]) {
        // TODO: This may need a cleanup. I can only hold 1 reference to self in a scope, if that's mutable.
        let file_name_a;
        let file_name_b;
        {
            file_name_a = self.context.working_context().file_a.name.as_str();
            file_name_b = self.context.working_context().file_b.name.as_str();
        }
        for kd in data {
            let a_has;
            let b_has;
            {
                a_has = self.check_has(file_name_a, kd);
                b_has = self.check_has(file_name_b, kd);
            }
            self.context.add_row(Row::new(vec![
                TableCell::new(&kd.key),
                TableCell::new(a_has),
                TableCell::new(b_has),
            ]));
        }
    }
}

impl<'a> KeyTable<'a> {
    pub fn new(data: &Vec<KeyDiff>, working_context: &'a LibWorkingContext) -> KeyTable<'a> {
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
}
