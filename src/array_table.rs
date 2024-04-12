use libdtf::core::diff_types::{ArrayDiff, ArrayDiffDesc};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
};

use crate::utils::{get_display_values_by_column, group_by_key};
use crate::{
    dtfterminal_types::{TableContext, TermTable, WorkingContext},
    utils::is_yaml_file,
};

/// Table to display array differences in the terminal
pub struct ArrayTable<'a> {
    context: TableContext<'a>,
}

impl<'a> TermTable<ArrayDiff> for ArrayTable<'a> {
    fn render(&self) -> String {
        self.context.render()
    }

    fn create_table(&mut self, data: &[ArrayDiff]) {
        self.add_header();
        self.add_rows(data);
    }

    fn add_header(&mut self) {
        let (file_name_a_str, file_name_b_str) = self.context.working_context().get_file_names();
        let file_name_a = file_name_a_str.to_owned();
        let file_name_b = file_name_b_str.to_owned();
        self.add_title_row();
        self.add_file_names_row(file_name_a, file_name_b);
    }

    fn add_rows(&mut self, data: &[ArrayDiff]) {
        let map = group_by_key(data);
        let file_name_a = self.context.working_context().get_file_names().0;
        let join_str = if is_yaml_file(file_name_a) { "" } else { ",\n" };

        for (key, values) in map {
            let display_values1: Vec<String> = get_display_values_by_column(
                self.context.working_context(),
                &values,
                ArrayDiffDesc::AHas,
            );
            let display_values2 = get_display_values_by_column(
                self.context.working_context(),
                &values,
                ArrayDiffDesc::BHas,
            );

            self.context.add_row(Row::new(vec![
                TableCell::new(key),
                TableCell::new(display_values1.join(join_str)),
                TableCell::new(display_values2.join(join_str)),
            ]));
        }
    }
}

impl<'a> ArrayTable<'a> {
    pub fn new(data: &[ArrayDiff], working_context: &'a WorkingContext) -> ArrayTable<'a> {
        let mut table = ArrayTable {
            context: TableContext::new(working_context),
        };
        table.create_table(data);
        table
    }

    /// Adds the header row to the table
    fn add_title_row(&mut self) {
        self.context
            .add_row(Row::new(vec![TableCell::new_with_alignment(
                "Array Differences",
                3,
                Alignment::Center,
            )]));
    }

    /// Adds the file names row to the table
    fn add_file_names_row(&mut self, file_name_a: String, file_name_b: String) {
        self.context.add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(format!("Only {} contains", file_name_a)),
            TableCell::new(format!("Only {} contains", file_name_b)),
        ]));
    }
}
