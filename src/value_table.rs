use libdtf::diff_types::ValueDiff;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use crate::{dtfterminal_types::LibWorkingContext, prettyfy_json_str};

pub struct ValueTable<'a> {
    working_context: &'a LibWorkingContext,
    pub table: Table<'a>,
}

impl<'a> ValueTable<'a> {
    pub fn new(data: &[ValueDiff], working_context: &'a LibWorkingContext) -> ValueTable<'a> {
        let mut table = ValueTable {
            working_context,
            table: Table::new(),
        };
        table.create_table(data);
        table
    }

    fn create_table(&mut self, data: &[ValueDiff]) {
        let mut table = Table::new();
        table.max_column_width = 80;
        table.style = TableStyle::extended();

        self.add_header();
        self.add_rows(data);

        self.table = table;
    }

    fn add_header(&mut self) {
        self.table
            .add_row(Row::new(vec![TableCell::new_with_alignment(
                "Value Differences",
                3,
                Alignment::Center,
            )]));
        self.table.add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(&self.working_context.file_a.name),
            TableCell::new(&self.working_context.file_b.name),
        ]));
    }

    fn add_rows(&mut self, data: &[ValueDiff]) {
        for vd in data {
            self.table.add_row(Row::new(vec![
                TableCell::new(&vd.key),
                TableCell::new(&prettyfy_json_str(&vd.value1)),
                TableCell::new(&prettyfy_json_str(&vd.value2)),
            ]));
        }
    }
}
