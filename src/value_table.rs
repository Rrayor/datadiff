use libdtf::core::diff_types::ValueDiff;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
};

use crate::dtfterminal_types::{TableContext, TermTable, WorkingContext};
use crate::utils::prettify_data;

/// Table to display value differences in the terminal
pub struct ValueTable<'a> {
    context: TableContext<'a>,
}

impl<'a> TermTable<ValueDiff> for ValueTable<'a> {
    fn render(&self) -> String {
        self.context.render()
    }

    fn create_table(&mut self, data: &[ValueDiff]) {
        self.add_header();
        self.add_rows(data);
    }

    fn add_header(&mut self) {
        let (file_name_a_str, file_name_b_str) = self.context.working_context().get_file_names();
        let file_name_a = file_name_a_str.to_owned();
        let file_name_b = file_name_b_str.to_owned();
        self.context.add_row(Row::new(vec![TableCell::builder("Value Differences")
            .col_span(3)
            .alignment(Alignment::Center)
        ]));
        self.context.add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(file_name_a),
            TableCell::new(file_name_b),
        ]));
    }

    fn add_rows(&mut self, data: &[ValueDiff]) {
        for vd in data {
            self.context.add_row(Row::new(vec![
                TableCell::new(&vd.key),
                TableCell::new(prettify_data(
                    self.context.working_context().get_file_names(),
                    &vd.value1,
                )),
                TableCell::new(prettify_data(
                    self.context.working_context().get_file_names(),
                    &vd.value2,
                )),
            ]));
        }
    }
}

impl<'a> ValueTable<'a> {
    pub fn new(data: &[ValueDiff], working_context: &'a WorkingContext) -> ValueTable<'a> {
        let mut table = ValueTable {
            context: TableContext::new(working_context),
        };
        table.create_table(data);
        table
    }
}
