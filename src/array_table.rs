use std::collections::HashMap;

use libdtf::core::diff_types::{ArrayDiff, ArrayDiffDesc};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
};

use crate::utils::prettify_data;
use crate::{
    dtfterminal_types::{LibWorkingContext, TableContext, TermTable},
    utils::is_yaml_file,
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
        let map = ArrayTable::group_by_key(data);
        let join_str = if is_yaml_file(self.get_file_names().0) {
            ""
        } else {
            ",\n"
        };

        for (key, values) in map {
            let display_values1: Vec<String> =
                self.get_display_values_by_column(&values, ArrayDiffDesc::AHas);
            let display_values2 = self.get_display_values_by_column(&values, ArrayDiffDesc::BHas);

            self.context.add_row(Row::new(vec![
                TableCell::new(key),
                TableCell::new(display_values1.join(join_str)),
                TableCell::new(display_values2.join(join_str)),
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

    fn group_by_key(data: &[ArrayDiff]) -> HashMap<&str, Vec<&ArrayDiff>> {
        let mut map = HashMap::new();

        for ad in data {
            let key = ad.key.as_str();

            if !map.contains_key(key) {
                map.insert(key, vec![]);
            }

            map.get_mut(key).unwrap().push(ad);
        }

        map
    }

    fn get_display_values_by_column(
        &self,
        values: &[&ArrayDiff],
        diff_desc: ArrayDiffDesc,
    ) -> Vec<String> {
        let file_names = self.get_file_names();
        values
            .iter()
            .filter(|ad| ad.descriptor == diff_desc)
            .map(|ad| prettify_data(file_names, ad.value.as_str()))
            .collect()
    }

    fn get_file_names(&self) -> (&str, &str) {
        let file_name_a = self.context.working_context().file_a.name.as_str();
        let file_name_b = self.context.working_context().file_b.name.as_str();
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
            TableCell::new(format!("Only {} contains", file_name_a)),
            TableCell::new(format!("Only {} contains", file_name_b)),
        ]));
    }
}
