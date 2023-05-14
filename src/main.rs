use dtfterminal::{collect_data, parse_args, render_tables, write_to_file};

fn main() -> Result<(), ()> {
    let (data1, data2, config) = parse_args();
    let ((key_diff, type_diff, value_diff, array_diff), working_context) =
        collect_data(data1, data2, &config);

    let _ = match working_context.config.write_to_file {
        Some(_) => write_to_file(
            key_diff,
            type_diff,
            value_diff,
            array_diff,
            &working_context,
        ),
        None => render_tables(
            key_diff,
            type_diff,
            value_diff,
            array_diff,
            &working_context,
        ),
    };
    Ok(())
}
