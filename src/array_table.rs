use libdtf::diff_types::{ArrayDiff, ArrayDiffDesc};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
};

use crate::{
    dtfterminal_types::{LibWorkingContext, TableContext, TermTable},
    prettyfy_json_str,
};

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
        let (file_name_a_str, file_name_b_str) = self.get_file_names();
        let file_name_a = file_name_a_str.to_owned();
        let file_name_b = file_name_b_str.to_owned();
        self.add_title_row();
        self.add_file_names_row(file_name_a, file_name_b);
    }

    fn add_rows(&mut self, data: &[ArrayDiff]) {
        for ad in data {
            let value_str = prettyfy_json_str(&ad.value);
            let (cell_value_1, cell_value_2) = self.get_cell_values(ad, value_str);
            self.context.add_row(Row::new(vec![
                TableCell::new(&ad.key),
                TableCell::new(cell_value_1),
                TableCell::new(cell_value_2),
            ]));
        }
    }
}

impl<'a> ArrayTable<'a> {
    pub fn new(data: &[ArrayDiff], working_context: &'a LibWorkingContext) -> ArrayTable<'a> {
        let mut table = ArrayTable {
            context: TableContext::new(working_context),
        };
        table.create_table(data);
        table
    }

    fn get_cell_values(&mut self, ad: &ArrayDiff, value_str: String) -> (String, String) {
        let cell_value_1 = self.get_cell_value(&ad.descriptor, &value_str).to_owned();
        let cell_value_2 = self.get_cell_value(&ad.descriptor, &value_str).to_owned();
        (cell_value_1, cell_value_2)
    }

    fn get_cell_value(&'a self, descriptor: &'a ArrayDiffDesc, value_str: &'a str) -> &'a str {
        match descriptor {
            ArrayDiffDesc::AHas => value_str,
            ArrayDiffDesc::AMisses => value_str,
            ArrayDiffDesc::BHas => value_str,
            ArrayDiffDesc::BMisses => value_str,
        }
    }

    fn get_file_names(&self) -> (&str, &str) {
        let file_name_a = self.context.working_context().file_a.name.as_str();
        let file_name_b = self.context.working_context().file_a.name.as_str();
        (file_name_a, file_name_b)
    }

    fn add_title_row(&mut self) {
        self.context
            .add_row(Row::new(vec![TableCell::new_with_alignment(
                "Array Differences",
                3,
                Alignment::Center,
            )]));
    }

    fn add_file_names_row(&mut self, file_name_a: String, file_name_b: String) {
        self.context.add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(file_name_a),
            TableCell::new(file_name_b),
        ]));
    }
}
