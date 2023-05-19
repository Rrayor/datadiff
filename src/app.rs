use crate::{
    dtfterminal_types::{DiffCollection, DtfError, WorkingContext},
    init,
};

pub struct App {
    diffs: DiffCollection,
    context: WorkingContext,
}

impl App {
    pub fn new() -> App {
        let (diffs, context) = init();
        App { diffs, context }
    }

    pub fn execute(&self) -> Result<(), DtfError> {
        if self.context.config.write_to_file.is_some() {
            write_to_file(self.diffs, &self.context)
                .map_err(|e| DtfError::GeneralError(Box::new(e)))?;
        } else {
            render_tables(self.diffs, &self.context)
                .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        }

        Ok(())
    }
}
