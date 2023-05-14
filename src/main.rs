use dtfterminal::{
    collect_data, dtfterminal_types::DtfError, parse_args, render_tables, write_to_file,
};

fn main() -> Result<(), DtfError> {
    let (data1, data2, config) = parse_args();
    let ((key_diff, type_diff, value_diff, array_diff), working_context) =
        collect_data(data1, data2, &config);

    if working_context.config.write_to_file.is_some() {
        write_to_file(
            key_diff,
            type_diff,
            value_diff,
            array_diff,
            &working_context,
        )
        .map_err(|e| DtfError::GeneralError(Box::new(e)))?;
    } else {
        render_tables(
            key_diff,
            type_diff,
            value_diff,
            array_diff,
            &working_context,
        )
        .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
    }
    Ok(())
}
