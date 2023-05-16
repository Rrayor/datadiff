use libdtf::diff_types::TypeDiff;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use crate::dtfterminal_types::LibWorkingContext;

pub struct TypeTable<'a> {
    working_context: &'a LibWorkingContext,
    pub table: Table<'a>,
}

impl<'a> TypeTable<'a> {
    pub fn new(data: &[TypeDiff], working_context: &'a LibWorkingContext) -> TypeTable<'a> {
        let mut table = TypeTable {
            working_context,
            table: Table::new(),
        };
        table.create_table(data);
        table
    }

    fn create_table(&mut self, data: &[TypeDiff]) {
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
                "Type Differences",
                3,
                Alignment::Center,
            )]));
        self.table.add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(&self.working_context.file_a.name),
            TableCell::new(&self.working_context.file_b.name),
        ]));
    }

    fn add_rows(&mut self, data: &[TypeDiff]) {
        for td in data {
            self.table.add_row(Row::new(vec![
                TableCell::new(&td.key),
                TableCell::new(&td.type1),
                TableCell::new(&td.type2),
            ]));
        }
    }
}
