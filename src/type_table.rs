use libdtf::core::diff_types::TypeDiff;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
};

use crate::dtfterminal_types::{TableContext, TermTable, WorkingContext};

/// Table to display type differences in the terminal
pub struct TypeTable<'a> {
    context: TableContext<'a>,
}

impl<'a> TermTable<TypeDiff> for TypeTable<'a> {
    fn render(&self) -> String {
        self.context.render()
    }

    fn create_table(&mut self, data: &[TypeDiff]) {
        self.add_header();
        self.add_rows(data);
    }

    fn add_header(&mut self) {
        let (file_name_a_str, file_name_b_str) = self.context.working_context().get_file_names();
        let file_name_a = file_name_a_str.to_owned();
        let file_name_b = file_name_b_str.to_owned();
        self.context
            .add_row(Row::new(vec![TableCell::new_with_alignment(
                "Type Differences",
                3,
                Alignment::Center,
            )]));
        self.context.add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(file_name_a),
            TableCell::new(file_name_b),
        ]));
    }

    fn add_rows(&mut self, data: &[TypeDiff]) {
        for td in data {
            self.context.add_row(Row::new(vec![
                TableCell::new(&td.key),
                TableCell::new(&td.type1),
                TableCell::new(&td.type2),
            ]));
        }
    }
}

impl<'a> TypeTable<'a> {
    pub fn new(data: &[TypeDiff], working_context: &'a WorkingContext) -> TypeTable<'a> {
        let mut table = TypeTable {
            context: TableContext::new(working_context),
        };
        table.create_table(data);
        table
    }
}
