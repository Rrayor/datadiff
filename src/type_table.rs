use libdtf::diff_types::TypeDiff;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use crate::dtfterminal_types::{TableContext, TermTable};

pub struct TypeTable<'a> {
    context: &'a TableContext<'a>,
}

impl<'a> TermTable<TypeDiff> for TypeTable<'a> {
    fn create_table(&mut self, data: &[TypeDiff]) {
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
                "Type Differences",
                3,
                Alignment::Center,
            )]));
        self.context.table().add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(&self.context.working_context().file_a.name),
            TableCell::new(&self.context.working_context().file_b.name),
        ]));
    }

    fn add_rows(&mut self, data: &[TypeDiff]) {
        for td in data {
            self.context.table().add_row(Row::new(vec![
                TableCell::new(&td.key),
                TableCell::new(&td.type1),
                TableCell::new(&td.type2),
            ]));
        }
    }
}
