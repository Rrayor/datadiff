use std::{collections::HashMap, fmt::format};

use libdtf::diff_types::{ArrayDiff, ArrayDiffDesc};
use serde_json::json;
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
        let mut map = HashMap::new();

        for ad in data {
            let key = ad.key.as_str();

            if !map.contains_key(key) {
                map.insert(key, vec![]);
            }

            map.get_mut(key).unwrap().push(ad);
        }

        for (key, values) in map {
            let display_values1: Vec<String> = values
                .iter()
                .filter(|ad| ad.descriptor == ArrayDiffDesc::AHas)
                .map(|ad| prettyfy_json_str(ad.value.as_str()))
                .collect();
            let display_values2: Vec<String> = values
                .into_iter()
                .filter(|ad| ad.descriptor == ArrayDiffDesc::BHas)
                .map(|ad| prettyfy_json_str(ad.value.as_str()))
                .collect();

            self.context.add_row(Row::new(vec![
                TableCell::new(key),
                TableCell::new(display_values1.join(",\n")),
                TableCell::new(display_values2.join(",\n")),
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
