use dtfterminal::{
    collect_data, create_working_context, parse_args, read_from_file, render_tables, write_to_file,
};

fn main() -> Result<(), ()> {
    let (data1, data2, config) = parse_args();
    let working_context = create_working_context(&config);
    let (key_diff, type_diff, value_diff, array_diff) = if config.read_from_file.is_empty() {
        collect_data(&data1.unwrap(), &data2.unwrap(), &working_context)
    } else {
        let saved_context = read_from_file(&config.read_from_file).unwrap();
        (
            Some(saved_context.key_diff),
            Some(saved_context.type_diff),
            Some(saved_context.value_diff),
            Some(saved_context.array_diff),
        )
    };

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
