use libdtf::diff_types::{ArrayDiff, ArrayDiffDesc};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use crate::{
    dtfterminal_types::{LibWorkingContext, TableContext, TermTable},
    prettyfy_json_str,
};

pub struct ArrayTable<'a> {
    context: &'a TableContext<'a>,
}

impl<'a> TermTable<ArrayDiff> for ArrayTable<'a> {
    fn create_table(&mut self, data: &[ArrayDiff]) {
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
                "Array Differences",
                3,
                Alignment::Center,
            )]));
        self.context.table().add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(&self.context.working_context().file_a.name),
            TableCell::new(&self.context.working_context().file_b.name),
        ]));
    }

    fn add_rows(&mut self, data: &[ArrayDiff]) {
        for ad in data {
            let value_str = prettyfy_json_str(&ad.value);
            self.context.table().add_row(Row::new(vec![
                TableCell::new(&ad.key),
                TableCell::new(self.get_cell_value(&ad.descriptor, &value_str)),
                TableCell::new(self.get_cell_value(&ad.descriptor, &value_str)),
            ]));
        }
    }
}

impl<'a> ArrayTable<'a> {
    pub fn new(data: &[ArrayDiff], working_context: &'a LibWorkingContext) -> ArrayTable<'a> {
        let mut table = ArrayTable {
            context: &TableContext::new(working_context),
        };
        table.create_table(data);
        table
    }

    pub fn table(&self) -> &Table {
        &self.context.table()
    }

    fn get_cell_value(&'a self, descriptor: &'a ArrayDiffDesc, value_str: &'a str) -> &'a str {
        match descriptor {
            ArrayDiffDesc::AHas => value_str,
            ArrayDiffDesc::AMisses => value_str,
            ArrayDiffDesc::BHas => value_str,
            ArrayDiffDesc::BMisses => value_str,
        }
    }
}
